//! Backend de captura para Linux usando evdev
//!
//! Este backend lee eventos directamente desde /dev/input/event*
//! permitiendo captura de baja latencia en Wayland y X11.

use crate::mouse_sentinel::InputEvent;
use crate::backend::Backend;
use evdev::{Device, RelativeAxisCode, EventSummary};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::{Result, Context};
use std::path::PathBuf;
use tracing::{info, error};
use std::time::UNIX_EPOCH;

pub struct LinuxBackend {
    pub device_path: Option<PathBuf>,
}

impl LinuxBackend {
    pub fn new(device_path: Option<PathBuf>) -> Self {
        Self { device_path }
    }

    /// Busca dispositivos que parezcan ser un ratón o teclado
    pub fn discover_input_devices() -> Vec<PathBuf> {
        let mut devices = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_str().map_or(false, |s| s.contains("event")) {
                    if let Ok(device) = Device::open(&path) {
                        let is_mouse = device.supported_relative_axes().map_or(false, |axes| {
                            axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
                        });
                        let is_keyboard = device.supported_keys().map_or(false, |keys| {
                            keys.iter().next().is_some()
                        });

                        if is_mouse || is_keyboard {
                            devices.push(path);
                        }
                    }
                }
            }
        }
        devices
    }
}

impl Backend for LinuxBackend {
    fn start(&self, tx: mpsc::Sender<InputEvent>, shutdown: CancellationToken) -> Result<()> {
        let devices = if let Some(path) = &self.device_path {
            vec![path.clone()]
        } else {
            Self::discover_input_devices()
        };

        if devices.is_empty() {
             anyhow::bail!("No input devices found in /dev/input");
        }

        info!("Starting Linux input capture on {} devices", devices.len());

        for device_path in devices {
            let tx_clone = tx.clone();
            let shutdown_clone = shutdown.clone();
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
                        _ = shutdown_clone.cancelled() => break,
                        
                        result = stream.next_event() => {
                            match result {
                                Ok(event) => {
                                    let t = event.timestamp()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs_f64();

                                    let input_event = match event.destructure() {
                                        EventSummary::RelativeAxis(_, axis, value) => {
                                            match axis {
                                                RelativeAxisCode::REL_X => x += value as f64,
                                                RelativeAxisCode::REL_Y => y += value as f64,
                                                _ => return,
                                            }
                                            Some(InputEvent::Mouse { x, y, t })
                                        }
                                        EventSummary::Key(_, _, value) if value > 0 => {
                                            // Solo capturamos "pulsación" (value > 0), no liberación
                                            Some(InputEvent::Keyboard { t })
                                        }
                                        _ => None,
                                    };
                                    
                                    if let Some(ev) = input_event {
                                        if let Err(_) = tx_clone.send(ev).await {
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
        }

        Ok(())
    }
}
