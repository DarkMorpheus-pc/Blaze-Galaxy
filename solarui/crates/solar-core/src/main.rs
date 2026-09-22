use anyhow::Result;
use solar_core::SolarPlatformDaemon;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "solar_core=info,solar_niri=info".into()),
        )
        .init();

    info!("Initializing SolarCore Platform Daemon (BlazeOS Edition)...");
    let daemon = SolarPlatformDaemon::new();
    daemon.run().await?;
    Ok(())
}
