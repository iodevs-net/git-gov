//! Protocolo de Foco para Git-Gov v2.0 ("El Testigo Silencioso")
//!
//! Define los mensajes estandarizados que sensores externos (extensiones de IDE)
//! envían al Daemon de Git-Gov para certificar presencia y foco humano.
//!
//! ## Principios de Privacidad
//! - **NO** se transmite contenido de archivos
//! - **NO** se transmite lo que el usuario escribe (keylogging)
//! - Solo metadata: rutas de archivos, timestamps, conteos de caracteres

use serde::{Deserialize, Serialize};

// =============================================================================
// MENSAJES DEL SENSOR → DAEMON
// =============================================================================

/// Eventos que un sensor (extensión de IDE) envía al Daemon
/// 
/// Todos los mensajes usan tagging por campo `type` para facilitar parsing JSON.
/// 
/// ## Ejemplo de uso
/// ```json
/// {"type":"focus_gained","file_path":"/home/dev/project/src/main.rs","timestamp_ms":1705790000000}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SensorEvent {
    /// La ventana del IDE ganó foco (el usuario está mirando el código)
    FocusGained {
        /// Ruta absoluta del archivo activo (None si es vista general/welcome)
        file_path: Option<String>,
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
    },

    /// La ventana del IDE perdió foco (usuario cambió a otra aplicación)
    FocusLost {
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
    },

    /// Ráfaga de edición detectada
    /// 
    /// El sensor agrega ediciones durante un intervalo corto (ej: 500ms)
    /// y envía un solo EditBurst con el delta total.
    EditBurst {
        /// Ruta absoluta del archivo donde ocurrió la edición
        file_path: String,
        /// Cantidad de caracteres modificados (positivo = inserción, negativo = eliminación)
        chars_delta: i64,
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
    },

    /// Evento de navegación (scroll, cambio de archivo)
    /// 
    /// Indica "lectura activa" sin necesidad de teclear.
    Navigation {
        /// Ruta absoluta del archivo visible
        file_path: String,
        /// Tipo de navegación detectada
        nav_type: NavigationType,
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
    },

    /// Heartbeat de salud
    /// 
    /// El sensor envía esto cada ~30 segundos mientras el IDE está abierto.
    /// Permite al daemon saber que la conexión está viva y el usuario presente.
    Heartbeat {
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
    },

    /// El sensor se desconecta limpiamente (IDE cerrado)
    Disconnect {
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
    },

    /// Evento de tecleo atómico para análisis cinemático CNS v3.0
    Keystroke {
        /// Ruta absoluta del archivo
        file_path: String,
        /// Timestamp Unix en milisegundos
        timestamp_ms: u64,
        /// Metadata del tecleo (opcional, para análisis de entropía)
        metadata: KeystrokeMetadata,
    },
}

/// Metadata para eventos de tecleo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeystrokeMetadata {
    /// Carácter insertado (si está disponible)
    pub char: String,
}

/// Tipos de navegación que indican "lectura activa"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NavigationType {
    /// Scroll vertical en el archivo
    Scroll,
    /// Cambio a otro archivo en el mismo workspace
    FileSwitch,
    /// Ir a definición / referencias
    GoToDefinition,
    /// Hover sobre símbolo (lectura de documentación)
    Hover,
}

// =============================================================================
// RESPUESTAS DEL DAEMON → SENSOR
// =============================================================================

/// Respuestas que el Daemon envía al sensor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SensorResponse {
    /// Confirmación de recepción exitosa
    Ack {
        /// Nivel actual de batería de atención (0.0 - 100.0)
        battery_level: f64,
    },

    /// Error en el procesamiento
    Error {
        /// Mensaje descriptivo del error
        message: String,
    },

    /// Estado completo del daemon (respuesta a consulta)
    Status {
        /// Indica si el daemon está activo y funcionando
        is_running: bool,
        /// Segundos desde que el daemon inició
        uptime_secs: u64,
        /// Nivel actual de batería de atención
        battery_level: f64,
        /// Minutos totales de foco registrados en la sesión
        focus_time_mins: f64,
        /// Cantidad de ráfagas de edición registradas
        edit_burst_count: usize,
    },
}

// =============================================================================
// HELPERS DE SERIALIZACIÓN
// =============================================================================

impl SensorEvent {
    /// Serializa el evento a JSON (una línea, para envío por socket)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializa un evento desde JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Extrae el timestamp del evento
    pub fn timestamp_ms(&self) -> u64 {
        match self {
            SensorEvent::FocusGained { timestamp_ms, .. } => *timestamp_ms,
            SensorEvent::FocusLost { timestamp_ms } => *timestamp_ms,
            SensorEvent::EditBurst { timestamp_ms, .. } => *timestamp_ms,
            SensorEvent::Navigation { timestamp_ms, .. } => *timestamp_ms,
            SensorEvent::Heartbeat { timestamp_ms } => *timestamp_ms,
            SensorEvent::Disconnect { timestamp_ms } => *timestamp_ms,
            SensorEvent::Keystroke { timestamp_ms, .. } => *timestamp_ms,
        }
    }
}

impl SensorResponse {
    /// Serializa la respuesta a JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_gained_serialization() {
        let event = SensorEvent::FocusGained {
            file_path: Some("/home/dev/project/src/main.rs".to_string()),
            timestamp_ms: 1705790000000,
        };

        let json = event.to_json().unwrap();
        assert!(json.contains("\"type\":\"focus_gained\""));
        assert!(json.contains("main.rs"));

        let parsed = SensorEvent::from_json(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_edit_burst_serialization() {
        let event = SensorEvent::EditBurst {
            file_path: "/tmp/test.rs".to_string(),
            chars_delta: 42,
            timestamp_ms: 1705790001000,
        };

        let json = event.to_json().unwrap();
        let parsed = SensorEvent::from_json(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_focus_lost_no_file_path() {
        let event = SensorEvent::FocusLost {
            timestamp_ms: 1705790002000,
        };

        let json = event.to_json().unwrap();
        assert!(!json.contains("file_path"));
    }

    #[test]
    fn test_navigation_types() {
        let event = SensorEvent::Navigation {
            file_path: "/src/lib.rs".to_string(),
            nav_type: NavigationType::GoToDefinition,
            timestamp_ms: 1705790003000,
        };

        let json = event.to_json().unwrap();
        assert!(json.contains("\"nav_type\":\"go_to_definition\""));
    }

    #[test]
    fn test_response_ack() {
        let response = SensorResponse::Ack { battery_level: 75.5 };
        let json = response.to_json().unwrap();
        assert!(json.contains("\"type\":\"ack\""));
        assert!(json.contains("75.5"));
    }

    #[test]
    fn test_timestamp_extraction() {
        let event = SensorEvent::Heartbeat {
            timestamp_ms: 123456789,
        };
        assert_eq!(event.timestamp_ms(), 123456789);
    }
}
