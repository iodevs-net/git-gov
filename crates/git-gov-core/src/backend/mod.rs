//! Capa de abstracción para backends de captura de eventos
//!
//! ## Arquitectura v2.0 ("El Testigo Silencioso")
//!
//! Este módulo soporta dos modos de operación:
//!
//! 1. **v2.0 (Default)**: `IdeSensorBackend` recibe eventos de extensiones de IDE
//!    via Unix socket. No requiere root, respeta privacidad.
//!
//! 2. **Legacy (feature `legacy-evdev`)**: `LinuxBackend` lee `/dev/input/event*`.
//!    Requiere root o grupo `input`. Activar solo para testing/desarrollo.

#[cfg(all(target_os = "linux", feature = "legacy-evdev"))]
pub mod linux;

pub mod ide_sensor;

use crate::mouse_sentinel::InputEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

/// Trait que define un backend de captura de eventos (legacy)
pub trait Backend: Send + Sync {
    /// Inicia la captura de eventos y los envía a través del canal tx.
    /// La captura debe detenerse cuando el shutdown es cancelado.
    fn start(&self, tx: mpsc::Sender<InputEvent>, shutdown: CancellationToken) -> Result<()>;
}

/// Backend de prueba que simula eventos de mouse
pub struct MockBackend;

impl Backend for MockBackend {
    fn start(&self, tx: mpsc::Sender<InputEvent>, shutdown: CancellationToken) -> Result<()> {
        tokio::spawn(async move {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut t = 0.0;
            
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        x += rand::random::<f64>() * 10.0 - 5.0;
                        y += rand::random::<f64>() * 10.0 - 5.0;
                        t += 0.1;
                        let _ = tx.send(InputEvent::Mouse { x, y, t }).await;
                    }
                }
            }
        });
        Ok(())
    }
}

/// Devuelve el backend predeterminado para la plataforma actual
/// 
/// ## v2.0 Behavior
/// Retorna `None` por defecto porque v2.0 usa `IdeSensorBackend` en lugar
/// de backends de hardware. El daemon debe iniciar `IdeSensorBackend` explícitamente.
/// 
/// Con feature `legacy-evdev`, intenta usar `LinuxBackend` si hay dispositivos disponibles.
pub fn get_default_backend() -> Option<Box<dyn Backend>> {
    #[cfg(all(target_os = "linux", feature = "legacy-evdev"))]
    {
        let mice = linux::LinuxBackend::discover_input_devices();
        if !mice.is_empty() {
             Some(Box::new(linux::LinuxBackend::new(None)))
        } else {
             // Fallback to mock if no real mice found
             Some(Box::new(MockBackend))
        }
    }
    
    // v2.0: No legacy backend by default
    #[cfg(not(feature = "legacy-evdev"))]
    {
        None
    }
}
