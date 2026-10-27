//! The library as a file: backups taken on demand, a copy to keep elsewhere,
//! and putting a copy back.
//!
//! The library is `asten.db`, one SQLite file. A backup is a consistent copy of
//! it (`VACUUM INTO`, so the write-ahead log is folded in). Restoring cannot
//! replace the file under a running app, so it is done at the next launch: the
//! chosen copy is validated and copied beside the library as
//! `restore-pending.db`, and the next start swaps it in after keeping the file
//! it replaces as `backups/before-restore-<time>.db`. Nothing is ever deleted
//! without a copy kept.

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    Connection,
    sqlite::{SqliteConnectOptions, SqliteConnection},
};
use tauri::State;

use crate::{DeskPaths, backup};

const PENDING: &str = "restore-pending.db";
const REPORT: &str = "restore-report.json";
const BEFORE: &str = "before-restore-";

/// One file in the backups folder.
#[derive(Clone, Debug, Serialize)]
pub struct BackupInfo {
    /// File name (the only thing a restore from the folder takes).
    pub name: String,
    /// Size in bytes.
    pub bytes: u64,
    /// When it was written.
    pub modified: String,
    /// `automatic` (taken at launch or on demand) or `before_restore`.
    pub kind: &'static str,
}

/// What a restore did, shown once after it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RestoreReport {
    /// It worked.
    pub ok: bool,
    /// What happened, in words.
    pub message: String,
    /// When.
    pub at: String,
}

/// The library's files and what can be done with them.
#[derive(Serialize)]
pub struct LibraryInfo {
    database_path: String,
    database_bytes: u64,
    backups_path: String,
    backups: Vec<BackupInfo>,
    /// A restore waits for the next launch.
    restore_pending: bool,
    /// The outcome of a restore at the last launch, if there was one.
    last_restore: Option<RestoreReport>,
}

fn info_of(path: &Path) -> Option<BackupInfo> {
    let name = path.file_name()?.to_str()?.to_owned();
    let kind = if name.starts_with(BEFORE) && name.ends_with(".db") {
        "before_restore"
    } else if name.starts_with("asten-") && name.ends_with(".db") {
        "automatic"
    } else {
        return None;
    };
    let meta = fs::metadata(path).ok()?;
    let modified: DateTime<Utc> = meta.modified().ok().map(SystemTime::into)?;
    Some(BackupInfo {
        name,
        bytes: meta.len(),
        modified: modified.to_rfc3339(),
        kind,
    })
}

/// The files in the backups folder, newest first.
pub fn list(backups: &Path) -> Vec<BackupInfo> {
    let mut found: Vec<BackupInfo> = fs::read_dir(backups)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| info_of(&entry.path()))
        .collect();
    found.sort_by(|a, b| b.modified.cmp(&a.modified));
    found
}

/// Check that a file is an Asten library this build can open.
pub async fn validate(path: &Path) -> Result<(), String> {
    let mut header = [0u8; 16];
    {
        use std::io::Read;
        let mut file = fs::File::open(path).map_err(|e| format!("cannot open the file: {e}"))?;
        file.read_exact(&mut header)
            .map_err(|_| "the file is too small to be a library".to_owned())?;
    }
    if &header != b"SQLite format 3\0" {
        return Err("this is not an Asten library (it is not a SQLite file)".into());
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(|e| format!("cannot read the file: {e}"))?;
    let check: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&mut connection)
        .await
        .map_err(|e| format!("the file is damaged: {e}"))?;
    if check.0 != "ok" {
        return Err(format!("the file is damaged: {}", check.0));
    }
    let projects: Result<(i64,), _> = sqlx::query_as("SELECT COUNT(*) FROM projects")
        .fetch_one(&mut connection)
        .await;
    if projects.is_err() {
        return Err("this is a SQLite file but not an Asten library".into());
    }
    let newest: Result<(Option<i64>,), _> =
        sqlx::query_as("SELECT MAX(version) FROM _sqlx_migrations")
            .fetch_one(&mut connection)
            .await;
    if let Ok((Some(version),)) = newest
        && version > asten_storage::latest_migration()
    {
        return Err("this library was written by a newer Asten; update Asten to open it".into());
    }
    let _ = connection.close().await;
    Ok(())
}

/// Write a backup now, whatever the age of the last one.
pub async fn backup_now(database: &Path, backups: &Path) -> Result<BackupInfo, String> {
    let path = backup::back_up_now(database, backups)
        .await
        .map_err(|e| e.to_string())?;
    info_of(&path).ok_or_else(|| "the backup was written but cannot be listed".to_owned())
}

/// A copy of the library at `target`, which the writer chose.
pub async fn export_copy(database: &Path, target: &Path) -> Result<u64, String> {
    if target.exists() {
        fs::remove_file(target).map_err(|e| format!("cannot replace {}: {e}", target.display()))?;
    }
    backup::vacuum_copy(database, target)
        .await
        .map_err(|e| e.to_string())?;
    fs::metadata(target)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

/// Validate `source` and set it aside to be swapped in at the next launch.
pub async fn schedule_restore(data: &Path, source: &Path) -> Result<(), String> {
    validate(source).await?;
    let pending = data.join(PENDING);
    let partial = data.join(format!("{PENDING}.partial"));
    let _ = fs::remove_file(&partial);
    fs::copy(source, &partial).map_err(|e| format!("cannot copy the file: {e}"))?;
    fs::rename(&partial, &pending).map_err(|e| format!("cannot set the copy aside: {e}"))?;
    let _ = fs::remove_file(data.join(REPORT));
    Ok(())
}

fn write_report(data: &Path, ok: bool, message: String) -> RestoreReport {
    let report = RestoreReport {
        ok,
        message,
        at: Local::now().to_rfc3339(),
    };
    if let Ok(text) = serde_json::to_string(&report) {
        let _ = fs::write(data.join(REPORT), text);
    }
    report
}

/// At launch, before anything opens the library: swap in a restore that was
/// asked for. The library it replaces is kept in the backups folder.
pub async fn apply_pending(paths: &DeskPaths) -> Option<RestoreReport> {
    let data = paths.database.parent()?;
    let pending = data.join(PENDING);
    if !pending.is_file() {
        return None;
    }
    if let Err(message) = validate(&pending).await {
        let _ = fs::rename(&pending, data.join("restore-rejected.db"));
        return Some(write_report(
            data,
            false,
            format!("The restore was not applied: {message}. Your library was not touched."),
        ));
    }
    let _ = fs::create_dir_all(&paths.backups);
    if paths.database.is_file() {
        let stamp = Local::now().format("%Y%m%d-%H%M%S");
        let keep = paths.backups.join(format!("{BEFORE}{stamp}.db"));
        let _ = fs::remove_file(&keep);
        if let Err(error) = backup::vacuum_copy(&paths.database, &keep).await {
            return Some(write_report(
                data,
                false,
                format!(
                    "The restore was not applied: could not keep a copy of the current library ({error}). Nothing was changed."
                ),
            ));
        }
    }
    for suffix in ["", "-wal", "-shm"] {
        let mut name = paths.database.clone().into_os_string();
        name.push(suffix);
        let old = PathBuf::from(name);
        if old.exists() {
            if let Err(error) = fs::remove_file(&old) {
                return Some(write_report(
                    data,
                    false,
                    format!(
                        "The restore was not applied: {} could not be replaced ({error}). A copy of the current library is in Backups as before-restore.",
                        old.display()
                    ),
                ));
            }
        }
    }
    match fs::rename(&pending, &paths.database) {
        Ok(()) => Some(write_report(
            data,
            true,
            "Your library was restored. The one it replaced is kept in Backups as before-restore."
                .into(),
        )),
        Err(error) => Some(write_report(
            data,
            false,
            format!(
                "The restore failed after the old library was set aside ({error}). The copy is in Backups as before-restore; copy it back to asten.db."
            ),
        )),
    }
}

fn read_report(data: &Path) -> Option<RestoreReport> {
    serde_json::from_str(&fs::read_to_string(data.join(REPORT)).ok()?).ok()
}

// ───────────────────────── commands ─────────────────────────

fn data_of(paths: &DeskPaths) -> PathBuf {
    paths
        .database
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// The library's files.
#[tauri::command]
pub fn library_info(paths: State<'_, DeskPaths>) -> LibraryInfo {
    let data = data_of(&paths);
    LibraryInfo {
        database_path: paths.database.display().to_string(),
        database_bytes: fs::metadata(&paths.database).map_or(0, |m| m.len()),
        backups_path: paths.backups.display().to_string(),
        backups: list(&paths.backups),
        restore_pending: data.join(PENDING).is_file(),
        last_restore: read_report(&data),
    }
}

/// Take a backup now.
#[tauri::command]
pub async fn library_backup_now(paths: State<'_, DeskPaths>) -> Result<BackupInfo, String> {
    backup_now(&paths.database, &paths.backups).await
}

/// Write a copy of the library where the writer chose.
#[tauri::command]
pub async fn library_export_copy(paths: State<'_, DeskPaths>, path: String) -> Result<u64, String> {
    let target = checked_target(&path, &["db", "asten-library"])?;
    export_copy(&paths.database, &target).await
}

/// Where a restore comes from.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RestoreSource {
    /// A file in the backups folder, by name.
    Backup { name: String },
    /// A file the writer picked.
    File { path: String },
}

/// Set a restore up for the next launch. The page then restarts Asten.
#[tauri::command]
pub async fn library_restore(
    paths: State<'_, DeskPaths>,
    source: RestoreSource,
) -> Result<(), String> {
    let data = data_of(&paths);
    let path = match source {
        RestoreSource::Backup { name } => {
            if name.contains(['/', '\\']) || name.contains("..") {
                return Err("that is not a backup name".into());
            }
            paths.backups.join(name)
        }
        RestoreSource::File { path } => PathBuf::from(path),
    };
    if !path.is_file() {
        return Err("that file does not exist".into());
    }
    schedule_restore(&data, &path).await
}

/// Forget the report of the last restore.
#[tauri::command]
pub fn library_dismiss_report(paths: State<'_, DeskPaths>) {
    let _ = fs::remove_file(data_of(&paths).join(REPORT));
}

fn checked_target(path: &str, extensions: &[&str]) -> Result<PathBuf, String> {
    let target = PathBuf::from(path);
    if !target.is_absolute() {
        return Err("the path must be absolute".into());
    }
    if target
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("the path must not contain ..".into());
    }
    let extension = target
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if !extensions.contains(&extension.as_str()) {
        return Err(format!("the file name must end in .{}", extensions[0]));
    }
    if target.parent().is_none_or(|p| !p.is_dir()) {
        return Err("that folder does not exist".into());
    }
    Ok(target)
}

/// Save a file the writer named with the save dialog: an export, a project
/// file, a copy. Only the extensions Asten writes are accepted, only an
/// absolute path, at most 512 MiB.
#[tauri::command]
pub fn save_file(path: String, base64: String) -> Result<u64, String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let target = checked_target(
        &path,
        &[
            "md", "txt", "html", "fountain", "docx", "epub", "fdx", "asten", "json",
        ],
    )?;
    if base64.len() > 700 * 1024 * 1024 {
        return Err("the file is too large".into());
    }
    let bytes = STANDARD
        .decode(base64.as_bytes())
        .map_err(|e| format!("the data is not base64: {e}"))?;
    let partial = target.with_extension(format!(
        "{}.partial",
        target.extension().and_then(|e| e.to_str()).unwrap_or("tmp")
    ));
    fs::write(&partial, &bytes).map_err(|e| format!("cannot write the file: {e}"))?;
    fs::rename(&partial, &target).map_err(|e| {
        let _ = fs::remove_file(&partial);
        format!("cannot write the file: {e}")
    })?;
    Ok(bytes.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("asten-library-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn library(path: &Path, projects: &[&str]) {
        let repository = asten_storage::SqliteRepository::open(path).await.unwrap();
        let pool = repository.pool().clone();
        let app = asten_application::Application::new(std::sync::Arc::new(repository));
        for name in projects {
            app.create_project(asten_application::CreateProject {
                name: (*name).to_owned(),
                kind: asten_domain::ProjectKind::Novel,
                description: None,
            })
            .await
            .unwrap();
        }
        // Windows will not move a file a pool still holds open.
        pool.close().await;
        settle().await;
    }

    /// SQLite's worker threads let go of their files a moment after the pool closes.
    async fn settle() {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    async fn names(path: &Path) -> Vec<String> {
        let repository = asten_storage::SqliteRepository::open(path).await.unwrap();
        let pool = repository.pool().clone();
        let app = asten_application::Application::new(std::sync::Arc::new(repository));
        let mut names: Vec<String> = app
            .list_projects()
            .await
            .unwrap()
            .into_iter()
            .map(|p| p.name)
            .collect();
        names.sort();
        pool.close().await;
        settle().await;
        names
    }

    #[tokio::test]
    async fn a_backup_restores_at_the_next_launch_and_the_old_library_is_kept() {
        let dir = scratch("restore");
        let paths = DeskPaths {
            database: dir.join("asten.db"),
            logs: dir.join("logs"),
            backups: dir.join("backups"),
        };
        library(&paths.database, &["First"]).await;
        let first = backup_now(&paths.database, &paths.backups).await.unwrap();
        assert_eq!(first.kind, "automatic");
        // The library moves on.
        library(&paths.database, &["Second"]).await;
        assert_eq!(names(&paths.database).await, ["First", "Second"]);

        // Ask for the restore; nothing changes until the next launch.
        schedule_restore(&dir, &paths.backups.join(&first.name))
            .await
            .unwrap();
        assert_eq!(names(&paths.database).await, ["First", "Second"]);

        let report = apply_pending(&paths).await.unwrap();
        assert!(report.ok, "{}", report.message);
        assert_eq!(names(&paths.database).await, ["First"]);
        // The library it replaced is kept.
        let kept: Vec<_> = list(&paths.backups)
            .into_iter()
            .filter(|b| b.kind == "before_restore")
            .collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(
            names(&paths.backups.join(&kept[0].name)).await,
            ["First", "Second"]
        );
        assert!(!dir.join(PENDING).exists());
        assert!(read_report(&dir).unwrap().ok);
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn things_that_are_not_libraries_are_refused_and_nothing_is_touched() {
        let dir = scratch("refuse");
        let paths = DeskPaths {
            database: dir.join("asten.db"),
            logs: dir.join("logs"),
            backups: dir.join("backups"),
        };
        library(&paths.database, &["Mine"]).await;
        let text = dir.join("notes.db");
        fs::write(&text, b"this is just some text, not a database at all").unwrap();
        assert!(schedule_restore(&dir, &text).await.is_err());
        let other = dir.join("other.db");
        {
            let options = SqliteConnectOptions::new()
                .filename(&other)
                .create_if_missing(true);
            let mut c = SqliteConnection::connect_with(&options).await.unwrap();
            sqlx::query("CREATE TABLE x (a INTEGER)")
                .execute(&mut c)
                .await
                .unwrap();
            c.close().await.unwrap();
        }
        assert!(
            schedule_restore(&dir, &other)
                .await
                .unwrap_err()
                .contains("not an Asten library")
        );
        assert!(!dir.join(PENDING).exists());
        assert_eq!(names(&paths.database).await, ["Mine"]);
        // A pending file that went bad between asking and launching is rejected too.
        fs::copy(&text, dir.join(PENDING)).unwrap();
        let report = apply_pending(&paths).await.unwrap();
        assert!(!report.ok);
        assert_eq!(names(&paths.database).await, ["Mine"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn an_export_is_a_whole_library() {
        let dir = scratch("export");
        let database = dir.join("asten.db");
        library(&database, &["Alpha", "Beta"]).await;
        let target = dir.join("copy.db");
        let bytes = export_copy(&database, &target).await.unwrap();
        assert!(bytes > 0);
        assert_eq!(names(&target).await, ["Alpha", "Beta"]);
        // Exporting over an existing file replaces it.
        export_copy(&database, &target).await.unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_file_takes_only_what_asten_writes_and_only_absolute_paths() {
        let dir = scratch("save");
        let ok = dir.join("book.docx");
        assert_eq!(
            save_file(ok.display().to_string(), "aGVsbG8=".into()).unwrap(),
            5
        );
        assert_eq!(fs::read(&ok).unwrap(), b"hello");
        assert!(save_file("relative.docx".into(), "aGk=".into()).is_err());
        assert!(save_file(dir.join("x.exe").display().to_string(), "aGk=".into()).is_err());
        assert!(
            save_file(
                dir.join("..").join("x.docx").display().to_string(),
                "aGk=".into()
            )
            .is_err()
        );
        assert!(
            save_file(
                dir.join("missing").join("x.docx").display().to_string(),
                "aGk=".into()
            )
            .is_err()
        );
        assert!(save_file(ok.display().to_string(), "***".into()).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
