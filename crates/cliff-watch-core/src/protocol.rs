//! Protocolo de comunicación para GitGov (IPC)
//!
//! Define los mensajes que se intercambian entre el CLI (cliente)
//! y el Daemon (servidor) a través de Unix Domain Sockets.

use serde::{Serialize, Deserialize};

/// Peticiones que el CLI envía al Daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Solicita el estado general del daemon
    GetStatus,
    /// Solicita las métricas cinemáticas actuales
    GetMetrics,
    /// Solicita un ticket de atención para pagar un costo entrópico
    GetTicket { cost: f64 },
    /// Prueba de conexión
    Ping,
    /// Solicita datos del Witness para certificación v2.0
    GetWitness { reset: bool },
}

/// Respuestas que el Daemon envía al CLI
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Estado del daemon
    Status {
        is_running: bool,
        uptime_secs: u64,
        events_captured: usize,
    },
    /// Métricas calculadas
    Metrics {
        ldlj: f64,
        entropy: f64,
        throughput: f64,
        human_score: f64,
        coupling: f64,
        battery_level: f64,
        focus_time_mins: f64,
        edit_bursts: usize,
        is_focused: bool,
        zkp_proof: Option<String>,
        #[serde(default)]
        score_history: Vec<f64>,
    },
    /// Ticket de atención firmado
    Ticket {
        success: bool,
        signature: Option<Vec<u8>>,
        message: String,
    },
    /// Datos del Witness para el commit
    Witness {
        data: String, // JSON serializado de WitnessData
    },
    /// Respuesta a Ping
    Pong,
    /// Error en la operación
    Error(String),
}
