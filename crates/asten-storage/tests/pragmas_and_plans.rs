//! The connection pragmas the repository promises, and proof that the hot
//! queries walk an index rather than a table.

use asten_storage::SqliteRepository;
use sqlx::Row;
use uuid::Uuid;

async fn open() -> (SqliteRepository, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("asten-plans-{}.db", Uuid::now_v7()));
    let repository = SqliteRepository::open(&path).await.unwrap();
    (repository, path)
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
}

/// Every `SELECT ... WHERE` the application runs per document or per project.
const HOT_QUERIES: &[&str] = &[
    "SELECT id FROM documents WHERE project_id = ? ORDER BY parent_id, position, created_at",
    "SELECT seq FROM block_events WHERE document_id = ? AND seq >= ? ORDER BY seq",
    "SELECT id FROM block_snapshots WHERE document_id = ? AND seq <= ? ORDER BY seq DESC LIMIT 1",
    "SELECT id FROM block_snapshots WHERE document_id = ? ORDER BY seq DESC LIMIT 1",
    "SELECT id FROM facts WHERE project_id = ? ORDER BY entity_key, predicate, created_at",
    "SELECT id FROM world_entries WHERE project_id = ? ORDER BY created_at",
    "SELECT id FROM entities WHERE project_id = ? ORDER BY kind, name",
    "SELECT id FROM timeline_events WHERE project_id = ? ORDER BY reading_order, created_at",
    "SELECT id FROM comments WHERE document_id = ? ORDER BY created_at",
    "SELECT id FROM document_ops WHERE document_id = ? ORDER BY seq",
    "SELECT id FROM document_snapshots WHERE document_id = ? ORDER BY created_at DESC",
    "SELECT id FROM suppressions WHERE project_id = ? ORDER BY created_at DESC",
    "SELECT id FROM tracked_changes WHERE document_id = ? ORDER BY created_at",
    "SELECT id FROM drafts WHERE document_id = ? ORDER BY created_at DESC",
    "SELECT settings_json FROM project_settings WHERE project_id = ?",
    "SELECT id FROM invariants WHERE project_id = ? ORDER BY created_at",
    "SELECT id FROM routes WHERE project_id = ? ORDER BY created_at",
    "SELECT id FROM baselines WHERE project_id = ? ORDER BY created_at DESC",
    "SELECT id FROM auth_tokens WHERE hash = ?",
];

#[tokio::test]
async fn pragmas_are_set_on_every_connection() {
    let (repository, path) = open().await;
    let pool = repository.pool();
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(journal, "wal");
    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(synchronous, 1, "NORMAL");
    let busy: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(busy, 5_000);
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(foreign_keys, 1);
    drop(repository);
    cleanup(&path);
}

#[tokio::test]
async fn hot_queries_search_an_index() {
    let (repository, path) = open().await;
    let pool = repository.pool();
    for sql in HOT_QUERIES {
        let plan = format!("EXPLAIN QUERY PLAN {sql}");
        let mut query = sqlx::query(&plan);
        for _ in 0..sql.matches('?').count() {
            query = query.bind("x");
        }
        let rows = query.fetch_all(pool).await.unwrap();
        let details: Vec<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("detail"))
            .collect();
        let first = details.first().cloned().unwrap_or_default();
        assert!(
            first.starts_with("SEARCH") && first.contains("USING"),
            "{sql}\n  plan: {details:?}"
        );
        assert!(
            details.iter().all(|detail| !detail.starts_with("SCAN")),
            "{sql}\n  plan: {details:?}"
        );
    }
    drop(repository);
    cleanup(&path);
}
