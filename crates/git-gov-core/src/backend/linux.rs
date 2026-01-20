//! Backend de captura para Linux usando evdev
//!
//! Este backend lee eventos directamente desde /dev/input/event*
//! permitiendo captura de baja latencia en Wayland y X11.

use crate::mouse_sentinel::MouseEvent;
use crate::backend::Backend;
use evdev::{Device, RelativeAxisCode, EventSummary};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::{Result, Context};
use std::path::PathBuf;
use tracing::{info, error, debug};
use std::time::UNIX_EPOCH;

pub struct LinuxBackend {
    pub device_path: Option<PathBuf>,
}

impl LinuxBackend {
    pub fn new(device_path: Option<PathBuf>) -> Self {
        Self { device_path }
    }

    /// Busca dispositivos que parezcan ser un ratón
    pub fn discover_mice() -> Vec<PathBuf> {
        let mut mice = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_str().map_or(false, |s| s.contains("event")) {
                    if let Ok(device) = Device::open(&path) {
                        if device.supported_relative_axes().map_or(false, |axes| {
                            axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
                        }) {
                            mice.push(path);
                        }
                    }
                }
            }
        }
        mice
    }
}

impl Backend for LinuxBackend {
    fn start(&self, tx: mpsc::Sender<MouseEvent>, shutdown: CancellationToken) -> Result<()> {
        let device_path = self.device_path.clone()
            .or_else(|| Self::discover_mice().first().cloned())
            .context("No mouse device found in /dev/input")?;

        info!("Starting Linux input capture on {:?}", device_path);

        let device = Device::open(&device_path)
            .with_context(|| format!("Failed to open device {:?}", device_path))?;

        let mut stream = device.into_event_stream()
            .with_context(|| format!("Failed to create event stream for {:?}", device_path))?;

        // Mover la captura a una tarea de Tokio
        tokio::spawn(async move {
            let mut x = 0.0;
            let mut y = 0.0;

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        debug!("Linux backend shutdown");
                        break;
                    }
                    
                    result = stream.next_event() => {
                        match result {
                            Ok(event) => {
                                if let EventSummary::RelativeAxis(_, axis, value) = event.destructure() {
                                    match axis {
                                        RelativeAxisCode::REL_X => x += value as f64,
                                        RelativeAxisCode::REL_Y => y += value as f64,
                                        _ => {}
                                    }
                                    
                                    // Calcular tiempo en segundos desde EPOCH
                                    let t = event.timestamp()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs_f64();

                                    // Enviar evento de movimiento relativo normalizado
                                    let mouse_event = MouseEvent {
                                        x,
                                        y,
                                        t,
                                    };
                                    
                                    if let Err(_) = tx.send(mouse_event).await {
                                        debug!("Receiver dropped, stopping capture");
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error fetching events from evdev: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
