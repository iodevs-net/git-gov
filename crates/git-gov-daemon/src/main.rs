use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "git_gov_daemon=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting git-gov daemon");

    // Load configuration
    let config: git_gov_core::monitor::Config = load_config().await?;
    
    // Initialize monitoring
    let mut monitor: git_gov_core::monitor::GitMonitor = git_gov_core::monitor::GitMonitor::new(config.clone()).await?;
    
    // Start monitoring loop
    monitor.start().await?;

    Ok(())
}

async fn load_config() -> Result<git_gov_core::monitor::Config> {
    // Implementation pending
    Ok(git_gov_core::monitor::Config::default())
}