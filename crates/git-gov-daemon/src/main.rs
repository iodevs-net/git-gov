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
    let gov_config = git_gov_core::config::GovConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    
    info!("Loaded configuration: Difficulty={}, Entropy={}", 
        gov_config.governance.difficulty, 
        gov_config.governance.min_entropy
    );

    // Initialize monitoring
    let mut monitor_config = git_gov_core::monitor::GitMonitorConfig::default();
    monitor_config.analysis_interval = std::time::Duration::from_millis(gov_config.monitoring.debounce_window_ms); // Reuse debounce for analysis interval simplification or config
    monitor_config.min_entropy = gov_config.governance.min_entropy;

    let (input_tx, input_rx) = mpsc::channel(monitor_config.mouse_buffer_size);
    let (sensor_tx, sensor_rx) = mpsc::channel(100); // Canal para eventos de IDE
    let (file_tx, file_rx) = mpsc::channel(100); 
    let shutdown = CancellationToken::new();

    // Start hardware capture backend (Legacy v1.0)
    if let Some(backend) = get_default_backend() {
        backend.start(input_tx, shutdown.clone())?;
        info!("Hardware capture backend started");
    } else {
        warn!("No legacy hardware capture backend (evdev) found/enabled");
    }

    // Start IDE Sensor backend (v2.0)
    let ide_sensor = git_gov_core::backend::ide_sensor::IdeSensorBackend::new("/tmp/git-gov-sensor.sock");
    let ide_sensor_shutdown = shutdown.clone();
    tokio::spawn(async move {
        if let Err(e) = ide_sensor.start(sensor_tx, ide_sensor_shutdown).await {
            error!("IDE Sensor backend failed: {}", e);
        }
    });
    info!("IDE Sensor backend started (v2.0)");

    // Configuración del FileMonitor (Gobernanza de archivos)
    // Convertir de DTO a configuración interna
    let file_cfg: git_gov_core::monitor::MonitorConfig = gov_config.monitoring.into();
    let file_monitor = git_gov_core::monitor::FileMonitor::new(file_cfg)?;

    let monitor: git_gov_core::monitor::GitMonitor = git_gov_core::monitor::GitMonitor::new(
        monitor_config, 
        input_rx, 
        sensor_rx,
        file_rx,
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")), // watch_root
        shutdown.clone()
    )?;
    
    // Pass shared state to IPC server
    let metrics_ref = monitor.get_metrics_ref();
    let coupling_ref = monitor.get_coupling_ref();
    let battery_ref = monitor.get_battery_ref();
    let focus_tracker_ref = monitor.get_focus_tracker_ref();
    let events_captured_ref = monitor.get_events_captured_ref();
    
    // Load or create persistent identity
    let signing_key = git_gov_core::crypto::load_or_create_identity()
        .map_err(|e| anyhow::anyhow!("Failed to initialize identity: {}", e))?;

    let ipc_server = IpcServer::new(
        "/tmp/git-gov.sock".to_string(),
        metrics_ref,
        coupling_ref,
        battery_ref,
        focus_tracker_ref,
        events_captured_ref,
        shutdown.clone(),
        signing_key,
        gov_config.governance.min_entropy,
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