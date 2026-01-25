use serde::Deserialize;
use config::{Config, File, Environment};
use std::collections::HashSet;
use std::path::PathBuf;
use crate::monitor::MonitorConfig;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct GovConfig {
    pub governance: GovernanceConfig,
    pub monitoring: MonitorConfigDto,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GovernanceConfig {
    pub min_entropy: f64,
    pub difficulty: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitorConfigDto {
    pub watch_root: String,
    pub debounce_window_ms: u64,
    pub ignore_top_level_dirs: Vec<String>,
    pub ignore_extensions: Vec<String>,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_entropy: 2.5,
            difficulty: "Normal".to_string(),
        }
    }
}

impl Default for MonitorConfigDto {
    fn default() -> Self {
        Self {
            watch_root: ".".to_string(),
            debounce_window_ms: 500,
            ignore_top_level_dirs: vec![".git".to_string(), "target".to_string(), "node_modules".to_string()],
            ignore_extensions: vec!["log".to_string(), "lock".to_string()],
        }
    }
}

impl GovConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = Config::builder()
            .set_default("governance.min_entropy", 2.5)?
            .set_default("governance.difficulty", "Normal")?
            .set_default("monitoring.watch_root", ".")?
            .set_default("monitoring.debounce_window_ms", 500)?
            .set_default("monitoring.ignore_top_level_dirs", vec![".git", "target", "node_modules"])?
            .set_default("monitoring.ignore_extensions", vec!["log", "lock"])?
            // Local config
            .add_source(File::with_name("cliff-craft").required(false))
            // Global config
            .add_source(File::with_name(&format!("{}/.config/cliff-craft/config", std::env::var("HOME").unwrap_or_else(|_| ".".into()))).required(false))
            // Env vars: CLIFF_CRAFT_GOVERNANCE_MIN_ENTROPY
            .add_source(Environment::with_prefix("CLIFF_CRAFT").separator("__"));

        builder.build()?.try_deserialize()
    }
}

// Conversion to internal MonitorConfig
impl From<MonitorConfigDto> for MonitorConfig {
    fn from(dto: MonitorConfigDto) -> Self {
        let mut ignore_top_level_dirs = HashSet::new();
        for d in dto.ignore_top_level_dirs {
            ignore_top_level_dirs.insert(d);
        }

        let mut ignore_extensions = HashSet::new();
        for e in dto.ignore_extensions {
            ignore_extensions.insert(e);
        }

        use crate::monitor::OverflowPolicy;

        MonitorConfig {
            watch_root: PathBuf::from(dto.watch_root),
            debounce_window: Duration::from_millis(dto.debounce_window_ms),
            raw_queue_capacity: 2048,
            ignore_top_level_dirs,
            ignore_extensions,
            raw_overflow: OverflowPolicy::DropNewest,
            out_overflow: OverflowPolicy::DropNewest,
            graceful_drain_max: Duration::from_millis(250),
            graceful_drain_max_events: 10_000,
        }
    }
}
