use anyhow::Result;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error};
use git_gov_core::backend::get_default_backend;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ipc;
use ipc::IpcServer;

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
    let (mouse_tx, mouse_rx) = mpsc::channel(config.mouse_buffer_size);
    let shutdown = CancellationToken::new();

    // Start hardware capture backend
    if let Some(backend) = get_default_backend() {
        backend.start(mouse_tx, shutdown.clone())?;
        info!("Hardware capture backend started");
    } else {
        warn!("No supported hardware capture backend found for this platform");
    }

    let monitor: git_gov_core::monitor::GitMonitor = git_gov_core::monitor::GitMonitor::new(config.clone(), mouse_rx, shutdown.clone())?;
    
    // Pass shared state to IPC server
    let metrics_ref = monitor.get_metrics_ref();
    let events_captured_ref = monitor.get_events_captured_ref();
    
    let ipc_server = IpcServer::new(
        "/tmp/git-gov.sock".to_string(), // TODO: Make configurable
        metrics_ref,
        events_captured_ref,
        shutdown.clone()
    );

    // Start IPC server task
    tokio::spawn(async move {
        if let Err(e) = ipc_server.start().await {
            error!("IPC Server failed: {}", e);
        }
    });

    // Start monitoring loop
    monitor.start().await?;

    Ok(())
}

async fn load_config() -> Result<git_gov_core::monitor::Config> {
    // Implementation pending
    Ok(git_gov_core::monitor::Config::default())
}