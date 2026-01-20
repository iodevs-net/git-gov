use anyhow::Result;
use tokio::sync::{mpsc, watch};
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
    let config: git_gov_core::monitor::GitMonitorConfig = load_config().await?;
    
    // Initialize monitoring
    let (mouse_tx, mouse_rx) = mpsc::channel(config.mouse_buffer_size);
    let (file_tx, file_rx) = mpsc::channel(100); // Canal para eventos de archivo
    let shutdown = CancellationToken::new();

    // Start hardware capture backend
    if let Some(backend) = get_default_backend() {
        backend.start(mouse_tx, shutdown.clone())?;
        info!("Hardware capture backend started");
    } else {
        warn!("No supported hardware capture backend found for this platform");
    }

    // Configuración del FileMonitor (Gobernanza de archivos)
    let watch_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let file_cfg = git_gov_core::monitor::MonitorConfig::new(watch_root.clone());
    let file_monitor = git_gov_core::monitor::FileMonitor::new(file_cfg)?;

    let monitor: git_gov_core::monitor::GitMonitor = git_gov_core::monitor::GitMonitor::new(
        config.clone(), 
        mouse_rx, 
        file_rx,
        watch_root,
        shutdown.clone()
    )?;
    
    // Pass shared state to IPC server
    let metrics_ref = monitor.get_metrics_ref();
    let coupling_ref = monitor.get_coupling_ref();
    let battery_ref = monitor.get_battery_ref();
    let events_captured_ref = monitor.get_events_captured_ref();
    
    let ipc_server = IpcServer::new(
        "/tmp/git-gov.sock".to_string(),
        metrics_ref,
        coupling_ref,
        battery_ref,
        events_captured_ref,
        shutdown.clone()
    );

    // Start IPC server task
    tokio::spawn(async move {
        if let Err(e) = ipc_server.start().await {
            error!("IPC Server failed: {}", e);
        }
    });

    // Start FileMonitor task
    let _file_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let (_shutdown_tx, shutdown_rx) = watch::channel(git_gov_core::monitor::Shutdown::Run);
        if let Err(e) = file_monitor.run(file_tx, shutdown_rx).await {
            error!("FileMonitor failed: {}", e);
        }
    });

    // Start monitoring loop (GovMonitor)
    monitor.start().await?;

    Ok(())
}

async fn load_config() -> Result<git_gov_core::monitor::GitMonitorConfig> {
    // Implementation pending
    Ok(git_gov_core::monitor::GitMonitorConfig::default())
}