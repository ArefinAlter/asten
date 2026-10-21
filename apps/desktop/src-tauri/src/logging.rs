//! File logging for the shell: one file a day under app-data `logs/`, the
//! newest seven kept, plus stderr in debug builds.
//!
//! Nothing secret may reach a log. The API names request paths only (never
//! query strings, where the stream ticket travels, and never headers, where
//! the bearer token does), and the shell logs paths, ports and versions. Keep
//! it that way: no `tracing` call anywhere may be handed a token, a pairing
//! code or a passphrase.

use std::path::Path;

use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Overridable with `ASTEN_LOG`, in `RUST_LOG` syntax.
const DEFAULT_DIRECTIVES: &str =
    "warn,asten_desktop=info,asten_api=info,asten_application=info,asten_storage=info";

/// Keeps the background log writer alive; dropping it flushes the file.
pub struct LogGuard(#[allow(dead_code)] Option<WorkerGuard>);

fn filter() -> EnvFilter {
    EnvFilter::try_from_env("ASTEN_LOG").unwrap_or_else(|_| EnvFilter::new(DEFAULT_DIRECTIVES))
}

/// Start logging into `logs_dir`. Falls back to stderr alone when the
/// directory cannot be written, so a broken install still says why.
pub fn init(logs_dir: &Path) -> LogGuard {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(7)
        .filename_prefix("asten")
        .filename_suffix("log")
        .build(logs_dir);
    match appender {
        Ok(appender) => {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let file = fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_writer(writer)
                .with_filter(filter());
            let registry = tracing_subscriber::registry().with(file);
            if cfg!(debug_assertions) {
                let stderr = fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_filter(filter());
                let _ = registry.with(stderr).try_init();
            } else {
                let _ = registry.try_init();
            }
            LogGuard(Some(guard))
        }
        Err(error) => {
            let _ = fmt()
                .with_env_filter(filter())
                .with_writer(std::io::stderr)
                .try_init();
            tracing::warn!(
                %error,
                directory = %logs_dir.display(),
                "could not open the log directory; logging to stderr only"
            );
            LogGuard(None)
        }
    }
}
