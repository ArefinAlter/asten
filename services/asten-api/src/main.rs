use std::{env, net::SocketAddr};

use anyhow::Context;
use asten_api::ApiConfig;
use asten_application::AuthMode;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("asten_api=info,tower_http=info")),
        )
        .init();

    let config = ApiConfig::from_env();
    let database_path = env::var("ASTEN_DATABASE_PATH").unwrap_or_else(|_| "data/asten.db".into());
    if let Some(parent) = std::path::Path::new(&database_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("failed to create database directory")?;
    }
    let application = asten_api::application_for_database(&database_path, config.auth_mode).await?;
    let address: SocketAddr = env::var("ASTEN_BIND")
        .unwrap_or_else(|_| "127.0.0.1:4317".into())
        .parse()
        .context("ASTEN_BIND must be a valid socket address")?;
    if !address.ip().is_loopback() && config.auth_mode == AuthMode::Off {
        warn!(
            "ASTEN_AUTH=off on a non-loopback address exposes every manuscript on this network; refusing to start"
        );
        anyhow::bail!("set ASTEN_AUTH=pairing or ASTEN_AUTH=owner when binding beyond loopback");
    }
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(%address, %database_path, auth = ?config.auth_mode, "Asten API listening");
    match application.bootstrap_pairing_code().await {
        Ok(Some(start)) => {
            info!(
                "No client is paired yet. Pairing code (valid 10 minutes): {}",
                start.code
            );
            eprintln!(
                "\n  Asten pairing code: {}   (type it in the desk or a guest pane; expires {})\n",
                start.code,
                start.expires_at.format("%H:%M")
            );
        }
        Ok(None) => {}
        Err(error) => warn!(%error, "could not mint a pairing code"),
    }
    asten_api::serve(listener, application, config, shutdown_signal()).await
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
