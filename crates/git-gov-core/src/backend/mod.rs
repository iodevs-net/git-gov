//! Capa de abstracción para backends de captura de eventos
//!
//! Este módulo define el sistema modular de captura para permitir
//! soporte multiplataforma y desacoplamiento de la lógica de análisis.

#[cfg(target_os = "linux")]
pub mod linux;

use crate::mouse_sentinel::MouseEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

/// Trait que define un backend de captura de eventos
pub trait Backend: Send + Sync {
    /// Inicia la captura de eventos y los envía a través del canal tx.
    /// La captura debe detenerse cuando el shutdown es cancelado.
    fn start(&self, tx: mpsc::Sender<MouseEvent>, shutdown: CancellationToken) -> Result<()>;
}

/// Backend de prueba que simula eventos de mouse
pub struct MockBackend;

impl Backend for MockBackend {
    fn start(&self, tx: mpsc::Sender<MouseEvent>, shutdown: CancellationToken) -> Result<()> {
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
                        let _ = tx.send(MouseEvent { x, y, t }).await;
                    }
                }
            }
        });
        Ok(())
    }
}

/// Devuelve el backend predeterminado para la plataforma actual
pub fn get_default_backend() -> Option<Box<dyn Backend>> {
    #[cfg(target_os = "linux")]
    {
        let mice = linux::LinuxBackend::discover_mice();
        if !mice.is_empty() {
             Some(Box::new(linux::LinuxBackend::new(None)))
        } else {
             // Fallback to mock if no real mice found
             Some(Box::new(MockBackend))
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        Some(Box::new(MockBackend))
    }
}
