//! The desktop shell. It opens the writer's SQLite file in the OS app-data
//! directory, hosts the same loopback API the browser desk and the Word and
//! Docs guests use, and hands its own webview a desk token. One transport, no
//! parity drift between desktop and web.

use std::{net::SocketAddr, sync::Arc};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("asten_desktop=info,asten_api=info")
        .try_init();
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            let database_path = data_directory.join("asten.db");
            let (application, session) = tauri::async_runtime::block_on(async {
                let repository = asten_storage::SqliteRepository::open(&database_path).await?;
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
                    database_path: Some(database_path.display().to_string()),
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
                    database_path: database_path.display().to_string(),
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
