//! `asten-prealpha`: the hosted pre-alpha gateway. See the crate docs and
//! `deploy/pre-alpha/README.md`.

use std::net::SocketAddr;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("asten_prealpha=info,asten_api=warn")),
        )
        .init();
    let config = asten_prealpha::Config::from_env();
    let address: SocketAddr = std::env::var("PREALPHA_BIND")
        .unwrap_or_else(|_| "127.0.0.1:4318".into())
        .parse()
        .context("PREALPHA_BIND must be a socket address")?;
    let gateway = asten_prealpha::Gateway::open(config).await?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Asten pre-alpha gateway listening");
    asten_prealpha::serve(listener, gateway, async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await
}
