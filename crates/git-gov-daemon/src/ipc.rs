use std::sync::{Arc, RwLock};
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
use tracing::{info, error};
use anyhow::Result;
use std::path::Path;
use std::fs;

use git_gov_core::protocol::{Request, Response};
use git_gov_core::mouse_sentinel::KinematicMetrics;
use git_gov_core::monitor::AttentionBattery;
use git_gov_core::stats::calculate_human_score;

pub struct IpcServer {
    socket_path: String,
    metrics: Arc<RwLock<Option<KinematicMetrics>>>,
    coupling_ref: Arc<RwLock<f64>>,
    battery_ref: Arc<RwLock<AttentionBattery>>,
    events_captured: Arc<RwLock<usize>>,
    shutdown: CancellationToken,
    start_time: std::time::Instant,
}

impl IpcServer {
    pub fn new(
        socket_path: String,
        metrics: Arc<RwLock<Option<KinematicMetrics>>>,
        coupling_ref: Arc<RwLock<f64>>,
        battery_ref: Arc<RwLock<AttentionBattery>>,
        events_captured: Arc<RwLock<usize>>,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            socket_path,
            metrics,
            coupling_ref,
            battery_ref,
            events_captured,
            shutdown,
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn start(self) -> Result<()> {
        if Path::new(&self.socket_path).exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IPC Server listening on {}", self.socket_path);

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    info!("IPC Server shutting down");
                    break;
                }
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((mut stream, _)) => {
                            let metrics_lock = self.metrics.clone();
                            let coupling_lock = self.coupling_ref.clone();
                            let battery_lock = self.battery_ref.clone();
                            let events_captured_lock = self.events_captured.clone();
                            let start_time = self.start_time;
                            
                            tokio::spawn(async move {
                                let mut buffer = vec![0; 1024];
                                let n = match stream.read(&mut buffer).await {
                                    Ok(n) if n > 0 => n,
                                    _ => return,
                                };

                                let request_res: Result<Request, _> = serde_json::from_slice(&buffer[..n]);
                                let response = match request_res {
                                    Ok(Request::GetStatus) => {
                                        let events = events_captured_lock.read().map(|g| *g).unwrap_or(0);
                                        Response::Status {
                                            is_running: true,
                                            uptime_secs: start_time.elapsed().as_secs(),
                                            events_captured: events,
                                        }
                                    }
                                    Ok(Request::GetMetrics) => {
                                        if let Ok(m_guard) = metrics_lock.read() {
                                            if let Some(m) = m_guard.as_ref() {
                                                // Calcular human score real basado en métricas cinemáticas
                                                let human_score = calculate_human_score(
                                                    m.burstiness,
                                                    m.ncd,
                                                );
                                                
                                                let coupling = coupling_lock.read().map(|g| *g).unwrap_or(1.0);
                                                let battery_level = battery_lock.read().map(|g| g.level).unwrap_or(0.0);

                                                Response::Metrics {
                                                    ldlj: m.ldlj,
                                                    entropy: m.velocity_entropy,
                                                    throughput: m.throughput,
                                                    human_score,
                                                    coupling,
                                                    battery_level,
                                                }
                                            } else {
                                                Response::Error("No metrics available yet".to_string())
                                            }
                                        } else {
                                            Response::Error("Failed to lock metrics".to_string())
                                        }
                                    }
                                    Ok(Request::GetTicket { cost }) => {
                                        let mut battery = battery_lock.write().map_err(|_| "Lock failed").unwrap();
                                        if battery.consume(cost) {
                                            // En un sistema real, usaríamos una llave persistente.
                                            // Para este PoC, firmamos la validez del costo.
                                            let message = format!("VALID:cost={:.2}:ts={}", cost, start_time.elapsed().as_secs());
                                            Response::Ticket {
                                                success: true,
                                                signature: Some(message.into_bytes()), // Simplificado para el PoC
                                                message: "Ticket issued. Thermodynamic balance verified.".to_string(),
                                            }
                                        } else {
                                            Response::Ticket {
                                                success: false,
                                                signature: None,
                                                message: format!(
                                                    "THERMODYNAMIC FAILURE: Required {:.2}, Battery at {:.2}. Focus more!",
                                                    cost, battery.level
                                                ),
                                            }
                                        }
                                    }
                                    Ok(Request::Ping) => Response::Pong,
                                    Err(e) => Response::Error(format!("Invalid request: {}", e)),
                                };

                                let response_json = serde_json::to_vec(&response).unwrap_or_default();
                                let _ = stream.write_all(&response_json).await;
                            });
                        }
                        Err(e) => error!("Failed to accept IPC connection: {}", e),
                    }
                }
            }
        }

        let _ = fs::remove_file(&self.socket_path);
        Ok(())
    }
}
