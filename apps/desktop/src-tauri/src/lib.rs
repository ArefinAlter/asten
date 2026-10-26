//! The desktop shell. It opens the writer's SQLite file in the OS app-data
//! directory, hosts the same loopback API the browser desk and the Word and
//! Docs guests use, and hands its own webview a desk token. One transport, no
//! parity drift between desktop and web.
//!
//! Around that it does the housekeeping a shipped app owes its writer: a copy
//! of the database at launch, a log file, one running copy at a time, and an
//! update check that runs only when asked.

mod backup;
mod logging;

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use asten_api::ApiConfig;
use asten_application::{Application, AuthMode};
use serde::Serialize;
use tauri::{Manager, State};

/// What the webview needs to talk to the embedded API.
#[derive(Clone, Debug, Serialize)]
pub struct DeskSession {
    pub origin: String,
    pub token: String,
    pub database_path: String,
    pub version: String,
}

struct DesktopState {
    application: Application,
    session: DeskSession,
}

/// Where this desk keeps its things, all under the OS app-data directory.
struct DeskPaths {
    database: PathBuf,
    logs: PathBuf,
    backups: PathBuf,
}

impl DeskPaths {
    fn under(data: &Path) -> Self {
        Self {
            database: data.join("asten.db"),
            logs: data.join("logs"),
            backups: data.join("backups"),
        }
    }
}

#[tauri::command]
fn desk_session(state: State<'_, DesktopState>) -> DeskSession {
    state.session.clone()
}

#[derive(Serialize)]
struct PairingCodeView {
    code: String,
    label: String,
    expires_at: String,
}

/// Mint a code a guest (Word, Docs, a browser) can type to pair with this desk.
#[tauri::command]
async fn start_pairing(
    state: State<'_, DesktopState>,
    label: Option<String>,
) -> Result<PairingCodeView, String> {
    let start = state
        .application
        .start_pairing(
            label.as_deref().unwrap_or("A guest"),
            asten_domain::TokenKind::Guest,
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(PairingCodeView {
        code: start.code,
        label: start.label,
        expires_at: start.expires_at.to_rfc3339(),
    })
}

/// The data folder the writer asked for, if any: `--data-dir <folder>` on the
/// command line, else the `ASTEN_DATA_DIR` environment variable. Empty values
/// are ignored.
fn data_dir_override() -> Option<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--data-dir" {
            return args.next().filter(|v| !v.is_empty()).map(PathBuf::from);
        }
    }
    std::env::var_os("ASTEN_DATA_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Bring the one running window forward when a second copy is launched.
#[cfg(desktop)]
fn focus_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    // Desktop only. The single-instance guard goes first so a second launch
    // is turned away before anything else starts (and before a second API
    // server could bind). The updater asks GitHub for `latest.json` only when
    // the writer presses the button in Settings; nothing checks on its own.
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            focus_main_window(app);
        }))
        .plugin(tauri_plugin_updater::Builder::new().build());
    builder
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // `ASTEN_DATA_DIR` (or `--data-dir <folder>`) keeps everything in one
            // folder of the writer's choosing: a portable copy on a USB stick,
            // a second profile, or a throwaway one for tests.
            let data_directory = match data_dir_override() {
                Some(dir) => dir,
                None => app.path().app_data_dir()?,
            };
            std::fs::create_dir_all(&data_directory)?;
            let paths = DeskPaths::under(&data_directory);
            std::fs::create_dir_all(&paths.logs)?;
            std::fs::create_dir_all(&paths.backups)?;
            // The guard flushes the log file at exit; managed state lives as
            // long as the app does, which is exactly as long as it is needed.
            app.manage(logging::init(&paths.logs));
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                data = %data_directory.display(),
                "Asten is starting"
            );
            let (application, session) = tauri::async_runtime::block_on(async {
                // Copy the file before anything opens it, so a migration that
                // goes wrong still leaves a known-good file to return to.
                match backup::back_up(&paths.database, &paths.backups).await {
                    Ok(backup::Outcome::Written(path)) => {
                        tracing::info!(path = %path.display(), "backed up the database");
                    }
                    Ok(backup::Outcome::Recent(path)) => {
                        tracing::debug!(path = %path.display(), "the newest backup is recent");
                    }
                    Ok(backup::Outcome::NothingToCopy) => {
                        tracing::debug!("no database yet; nothing to back up");
                    }
                    Err(error) => {
                        tracing::warn!(%error, "could not back up the database; continuing");
                    }
                }
                let repository = asten_storage::SqliteRepository::open(&paths.database).await?;
                let application = Application::with_auth(Arc::new(repository), AuthMode::Pairing);
                // Each launch gets a fresh desk token; older desk tokens are revoked.
                for token in application.list_tokens().await? {
                    if token.kind == asten_domain::TokenKind::Desk && token.revoked_at.is_none() {
                        let _ = application.revoke_token(token.id).await;
                    }
                }
                let desk = application.issue_desk_token("Desktop desk").await?;
                // Host the API on 4317 when free (guests expect it), else any port.
                let preferred: SocketAddr = "127.0.0.1:4317".parse().expect("loopback address");
                let listener = asten_api::bind_with_fallback(preferred).await?;
                let origin = format!("http://{}", listener.local_addr()?);
                let config = ApiConfig {
                    auth_mode: AuthMode::Pairing,
                    serve_desk: None,
                    database_path: Some(paths.database.display().to_string()),
                    logs_path: Some(paths.logs.display().to_string()),
                    backups_path: Some(paths.backups.display().to_string()),
                    embedded: true,
                    ..ApiConfig::default()
                };
                let serving = application.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) =
                        asten_api::serve(listener, serving, config, std::future::pending()).await
                    {
                        tracing::error!(%error, "the embedded API stopped");
                    }
                });
                let session = DeskSession {
                    origin,
                    token: desk.token,
                    database_path: paths.database.display().to_string(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                };
                Ok::<_, Box<dyn std::error::Error>>((application, session))
            })?;
            tracing::info!(origin = %session.origin, "embedded API ready");
            app.manage(DesktopState {
                application,
                session,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![desk_session, start_pairing])
        .run(tauri::generate_context!())
        .expect("error while running Asten");
}
