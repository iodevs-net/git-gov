//! Monitor principal de GitGov - Integración de MouseSentinel
//!
//! Este módulo implementa el monitor principal que integra el MouseSentinel
//! con el sistema de monitoreo de GitGov, permitiendo la captura y análisis
//! de eventos de mouse en tiempo real.
//!
//! # Responsabilidades
//! - Integración con MouseSentinel para captura de eventos
//! - Ejecución periódica de análisis cinemático
//! - Manejo de señales de shutdown
//! - Logging de métricas calculadas
//!
//! # Ejemplo de uso
//! ```
//! use git_gov_core::monitor::{GitMonitor, Config};
//! use tokio::sync::mpsc;
//! use tokio_util::sync::CancellationToken;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config {
//!         analysis_interval: std::time::Duration::from_millis(100),
//!         mouse_buffer_size: 100,
//!     };
//!
//!     let (tx, rx) = mpsc::channel(100);
//!     let shutdown = CancellationToken::new();
//!
//!     let monitor = GitMonitor::new(config, rx, shutdown.clone())?;
//!     // monitor.start().await?; // Comentado para evitar bloqueo en doctest
//!     Ok(())
//! }
//! ```

use thiserror::Error;
use tokio::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use std::sync::{Arc, RwLock};

use crate::mouse_sentinel::{MouseSentinel, MouseEvent, KinematicMetrics};
// use crate::protocol::Response; // unused for now

/// Configuración del monitor
#[derive(Debug, Clone)]
pub struct Config {
    /// Intervalo entre análisis de eventos
    pub analysis_interval: Duration,
    /// Tamaño del buffer de eventos de mouse
    pub mouse_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            analysis_interval: Duration::from_secs(5),
            mouse_buffer_size: 1024,
        }
    }
}

/// Monitor principal
pub struct GitMonitor {
    shutdown: CancellationToken,
    mouse_sentinel: MouseSentinel,
    mouse_rx: mpsc::Receiver<MouseEvent>,
    analysis_interval: Duration,
    latest_metrics: Arc<RwLock<Option<KinematicMetrics>>>,
    events_captured: Arc<RwLock<usize>>,
}

impl GitMonitor {
    /// Crea un nuevo GitMonitor
    ///
    /// # Argumentos
    /// * `config` - Configuración del monitor
    /// * `mouse_rx` - Canal de recepción de eventos de mouse
    /// * `shutdown` - Token de cancelación para shutdown
    ///
    /// # Retorna
    /// * `Result<Self, GitGovError>` - Monitor creado o error
    ///
    /// # Ejemplo
    /// ```
    /// use git_gov_core::monitor::{GitMonitor, Config};
    /// use tokio::sync::mpsc;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// let config = Config::default();
    /// let (tx, rx) = mpsc::channel(100);
    /// let shutdown = CancellationToken::new();
    /// let monitor = GitMonitor::new(config, rx, shutdown)?;
    /// # Ok::<(), git_gov_core::monitor::GitGovError>(())
    /// ```
    pub fn new(
        config: Config,
        mouse_rx: mpsc::Receiver<MouseEvent>,
        shutdown: CancellationToken,
    ) -> Result<Self, GitGovError> {
        Ok(Self {
            shutdown,
            mouse_sentinel: MouseSentinel::new(config.mouse_buffer_size),
            mouse_rx,
            analysis_interval: config.analysis_interval,
            latest_metrics: Arc::new(RwLock::new(None)),
            events_captured: Arc::new(RwLock::new(0)),
        })
    }

    /// Devuelve un clon del Arc que contiene las últimas métricas
    pub fn get_metrics_ref(&self) -> Arc<RwLock<Option<KinematicMetrics>>> {
        self.latest_metrics.clone()
    }

    /// Devuelve el contador de eventos capturados
    pub fn get_events_captured_ref(&self) -> Arc<RwLock<usize>> {
        self.events_captured.clone()
    }

    /// Inicia el monitor principal
    ///
    /// Este método corre en un loop infinito hasta recibir señal de shutdown,
    /// procesando eventos de mouse y ejecutando análisis periódicamente.
    ///
    /// # Retorna
    /// * `Result<(), GitGovError>` - Ok si el shutdown fue limpio
    ///
    /// # Ejemplo
    /// ```
    /// # use git_gov_core::monitor::{GitMonitor, Config};
    /// # use tokio::sync::mpsc;
    /// # use tokio_util::sync::CancellationToken;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), git_gov_core::monitor::GitGovError> {
    /// # let config = Config::default();
    /// # let (tx, rx) = mpsc::channel(100);
    /// # let shutdown = CancellationToken::new();
    /// # let monitor = GitMonitor::new(config, rx, shutdown.clone())?;
    /// // monitor.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(mut self) -> Result<(), GitGovError> {
        info!("GitMonitor started");
        let mut interval = tokio::time::interval(self.analysis_interval);

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    info!("Shutdown signal received");
                    break;
                }

                Some(event) = self.mouse_rx.recv() => {
                    self.mouse_sentinel.capture_event(event.x, event.y);
                    if let Ok(mut count) = self.events_captured.write() {
                        *count += 1;
                    }
                }

                _ = interval.tick() => {
                    self.run_analysis();
                }
            }
        }

        info!("GitMonitor stopped cleanly");
        Ok(())
    }

    /// Ejecuta el análisis de eventos capturados
    ///
    /// Este método es llamado periódicamente para analizar los eventos
    /// acumulados y registrar las métricas calculadas.
    fn run_analysis(&mut self) {
        match self.mouse_sentinel.analyze() {
            Ok(metrics) => {
                info!(
                    "Mouse metrics | LDLJ: {:.2} | Entropy: {:.2} | Throughput: {:.2}",
                    metrics.ldlj,
                    metrics.velocity_entropy,
                    metrics.throughput
                );
                if let Ok(mut latest) = self.latest_metrics.write() {
                    *latest = Some(metrics);
                }
            }
            Err(e) => {
                warn!("Mouse analysis failed: {}", e);
            }
        }
    }
}

/// Errores del sistema
#[derive(Debug, Error)]
pub enum GitGovError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Monitor error: {0}")]
    Monitor(String),
}
