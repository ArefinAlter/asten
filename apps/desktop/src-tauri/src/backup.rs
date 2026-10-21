//! A copy of the writer's file at every launch, taken before anything opens it.
//!
//! `VACUUM INTO` asks SQLite itself for a consistent copy of the database as
//! of one read transaction, with whatever the write-ahead log still holds
//! folded in, so the copy is whole even when the last session ended badly.
//! Copies are named by the local time they were taken, the newest seven are
//! kept, and a launch within six hours of the last copy takes none.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use chrono::Local;
use sqlx::{
    Connection,
    sqlite::{SqliteConnectOptions, SqliteConnection},
};

const KEEP: usize = 7;
const MIN_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const PREFIX: &str = "asten-";
const SUFFIX: &str = ".db";
const PARTIAL: &str = ".partial";

/// What a launch-time backup did.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A new copy was written here.
    Written(PathBuf),
    /// The newest copy, here, is less than six hours old; none was taken.
    Recent(PathBuf),
    /// There is no database yet.
    NothingToCopy,
}

#[derive(Debug)]
pub enum BackupError {
    Io(io::Error),
    Sql(sqlx::Error),
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "file system: {error}"),
            Self::Sql(error) => write!(f, "sqlite: {error}"),
        }
    }
}

impl std::error::Error for BackupError {}

impl From<io::Error> for BackupError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<sqlx::Error> for BackupError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sql(error)
    }
}

/// Copy `database` into `backups` unless the newest copy there is recent.
pub async fn back_up(database: &Path, backups: &Path) -> Result<Outcome, BackupError> {
    if !database.is_file() {
        return Ok(Outcome::NothingToCopy);
    }
    fs::create_dir_all(backups)?;
    if let Some(newest) = existing_backups(backups)?.pop()
        && age_of(&newest).is_some_and(|age| age < MIN_INTERVAL)
    {
        return Ok(Outcome::Recent(newest));
    }
    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let target = backups.join(format!("{PREFIX}{stamp}{SUFFIX}"));
    let partial = backups.join(format!("{PREFIX}{stamp}{SUFFIX}{PARTIAL}"));
    // `VACUUM INTO` refuses to overwrite, so the copy is written under a
    // working name and renamed once it is whole; a leftover is cleared first.
    let _ = fs::remove_file(&partial);
    if let Err(error) = vacuum_into(database, &partial).await {
        let _ = fs::remove_file(&partial);
        return Err(error);
    }
    fs::rename(&partial, &target)?;
    prune(backups)?;
    Ok(Outcome::Written(target))
}

async fn vacuum_into(database: &Path, target: &Path) -> Result<(), BackupError> {
    let options = SqliteConnectOptions::new()
        .filename(database)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    // The path is a SQL string literal; the only character to escape is the quote.
    let literal = target.to_string_lossy().replace('\'', "''");
    let sql = format!("VACUUM INTO '{literal}'");
    sqlx::query(&sql).execute(&mut connection).await?;
    connection.close().await?;
    Ok(())
}

/// Backups in this directory, oldest first: the names sort by time.
fn existing_backups(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_backup(path))
        .collect();
    found.sort();
    Ok(found)
}

fn file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

fn is_backup(path: &Path) -> bool {
    file_name(path).is_some_and(|name| {
        name.starts_with(PREFIX)
            && name.ends_with(SUFFIX)
            && name.len() > PREFIX.len() + SUFFIX.len()
    }) && path.is_file()
}

fn age_of(path: &Path) -> Option<Duration> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    SystemTime::now().duration_since(modified).ok()
}

/// Keep the newest `KEEP` copies and remove the rest, along with any copy an
/// interrupted run left half-written.
fn prune(dir: &Path) -> io::Result<()> {
    let found = existing_backups(dir)?;
    let surplus = found.len().saturating_sub(KEEP);
    for old in &found[..surplus] {
        fs::remove_file(old)?;
    }
    for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
        let path = entry.path();
        if file_name(&path).is_some_and(|name| name.starts_with(PREFIX) && name.ends_with(PARTIAL))
        {
            let _ = fs::remove_file(&path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use sqlx::sqlite::SqliteJournalMode;

    use super::*;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "asten-backup-{}-{}-{name}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    async fn seed(database: &Path) {
        let options = SqliteConnectOptions::new()
            .filename(database)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);
        let mut connection = SqliteConnection::connect_with(&options).await.unwrap();
        sqlx::query("CREATE TABLE pages (id INTEGER PRIMARY KEY, body TEXT NOT NULL)")
            .execute(&mut connection)
            .await
            .unwrap();
        sqlx::query("INSERT INTO pages (body) VALUES ('The lamp was lit.'), ('The lamp was out.')")
            .execute(&mut connection)
            .await
            .unwrap();
        connection.close().await.unwrap();
    }

    async fn count_pages(database: &Path) -> i64 {
        let options = SqliteConnectOptions::new()
            .filename(database)
            .create_if_missing(false);
        let mut connection = SqliteConnection::connect_with(&options).await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pages")
            .fetch_one(&mut connection)
            .await
            .unwrap();
        connection.close().await.unwrap();
        row.0
    }

    #[tokio::test]
    async fn nothing_to_copy_without_a_database() {
        let dir = scratch("none");
        let outcome = back_up(&dir.join("asten.db"), &dir.join("backups"))
            .await
            .unwrap();
        assert_eq!(outcome, Outcome::NothingToCopy);
        assert!(!dir.join("backups").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn copies_once_then_waits_six_hours() {
        let dir = scratch("copy");
        let database = dir.join("asten.db");
        seed(&database).await;
        let backups = dir.join("backups");
        let first = back_up(&database, &backups).await.unwrap();
        let Outcome::Written(copy) = first else {
            panic!("expected a copy, got {first:?}");
        };
        assert!(is_backup(&copy));
        assert_eq!(count_pages(&copy).await, 2);
        // The source is untouched and still opens.
        assert_eq!(count_pages(&database).await, 2);
        let second = back_up(&database, &backups).await.unwrap();
        assert_eq!(second, Outcome::Recent(copy));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn keeps_the_newest_seven_and_clears_partials() {
        let dir = scratch("prune");
        for day in 1..=10 {
            fs::write(dir.join(format!("asten-202601{day:02}-120000.db")), b"x").unwrap();
        }
        fs::write(dir.join("asten-20260101-000000.db.partial"), b"x").unwrap();
        fs::write(dir.join("notes.txt"), b"x").unwrap();
        prune(&dir).unwrap();
        let kept = existing_backups(&dir).unwrap();
        assert_eq!(kept.len(), KEEP);
        assert!(kept[0].ends_with("asten-20260104-120000.db"));
        assert!(kept[KEEP - 1].ends_with("asten-20260110-120000.db"));
        assert!(!dir.join("asten-20260101-000000.db.partial").exists());
        assert!(dir.join("notes.txt").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
