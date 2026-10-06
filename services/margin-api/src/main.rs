use std::{env, net::SocketAddr};

use anyhow::Context;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("margin_api=debug,tower_http=info")),
        )
        .init();

    let database_path = env::var("ASTEN_DATABASE_PATH")
        .or_else(|_| env::var("MARGIN_DATABASE_PATH"))
        .unwrap_or_else(|_| "data/asten.db".into());
    if let Some(parent) = std::path::Path::new(&database_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("failed to create database directory")?;
    }
    let application = margin_api::application_for_database(&database_path).await?;
    let app = margin_api::router(application);
    let address: SocketAddr = env::var("ASTEN_BIND")
        .or_else(|_| env::var("MARGIN_BIND"))
        .unwrap_or_else(|_| "127.0.0.1:4317".into())
        .parse()
        .context("ASTEN_BIND must be a valid socket address")?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(%address, %database_path, "Asten API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
