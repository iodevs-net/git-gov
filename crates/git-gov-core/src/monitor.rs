// crates/git-gov-core/src/monitor.rs
//! Observation layer: file edit telemetry and mouse kinematics.
//!
//! This module integrates two types of monitoring:
//! 1. **FileMonitor**: Watches the filesystem for code edits.
//! 2. **GitMonitor**: Captures mouse movements for kinematic analysis.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, RwLock,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::mouse_sentinel::{MouseSentinel, InputEvent, KinematicMetrics};

// =========================================================================
// SECTION 1: File Telemetry (FileMonitor)
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone)]
pub struct EditEvent {
    pub rel_path: PathBuf,
    /// Post-filter receipt timestamp (UNIX epoch microseconds).
    pub timestamp_us: u128,
    pub kind: EditKind,
}

#[derive(Debug, Clone)]
struct RawEvent {
    rel_path: PathBuf,
    timestamp_us: u128,
    kind: EditKind,
}

#[inline]
fn now_us() -> Option<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_micros())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shutdown {
    /// Keep running.
    Run,
    /// Stop immediately: pending raw events may be dropped.
    Immediate,
    /// Stop gracefully: drain raw queue (bounded by time + max events) before exit.
    Graceful,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Drop the newest event when the channel is full.
    DropNewest,
}

#[derive(Debug, Default, Clone)]
pub struct MonitorStatsSnapshot {
    pub raw_dropped_overflow: u64,
    pub out_dropped_overflow: u64,
    pub watcher_errors: u64,
    pub emitted: u64,
    pub debounced: u64,
}

#[derive(Debug, Default)]
struct MonitorStats {
    raw_dropped_overflow: AtomicU64,
    out_dropped_overflow: AtomicU64,
    watcher_errors: AtomicU64,
    emitted: AtomicU64,
    debounced: AtomicU64,
}

impl MonitorStats {
    fn snapshot(&self) -> MonitorStatsSnapshot {
        MonitorStatsSnapshot {
            raw_dropped_overflow: self.raw_dropped_overflow.load(Ordering::Relaxed),
            out_dropped_overflow: self.out_dropped_overflow.load(Ordering::Relaxed),
            watcher_errors: self.watcher_errors.load(Ordering::Relaxed),
            emitted: self.emitted.load(Ordering::Relaxed),
            debounced: self.debounced.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub watch_root: PathBuf,
    pub debounce_window: Duration,
    pub raw_queue_capacity: usize,
    pub ignore_top_level_dirs: HashSet<String>,
    pub ignore_extensions: HashSet<String>,
    pub raw_overflow: OverflowPolicy,
    pub out_overflow: OverflowPolicy,
    pub graceful_drain_max: Duration,
    pub graceful_drain_max_events: usize,
}

impl MonitorConfig {
    pub fn new(watch_root: impl Into<PathBuf>) -> Self {
        let mut ignore_top_level_dirs = HashSet::new();
        ignore_top_level_dirs.insert(".git".into());
        ignore_top_level_dirs.insert("target".into());
        ignore_top_level_dirs.insert("node_modules".into());

        let mut ignore_extensions = HashSet::new();
        ignore_extensions.insert("log".into());

        Self {
            watch_root: watch_root.into(),
            debounce_window: Duration::from_millis(120),
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

struct Debouncer {
    window_us: u128,
    last_emit_us: HashMap<PathBuf, u128>,
    seen: u64,
    gc_every: u64,
}

impl Debouncer {
    fn new(window: Duration) -> Self {
        Self {
            window_us: window.as_micros() as u128,
            last_emit_us: HashMap::new(),
            seen: 0,
            gc_every: 4096,
        }
    }

    fn should_emit(&mut self, path: &Path, kind: EditKind, ts_us: u128) -> bool {
        if kind == EditKind::Delete {
            return true;
        }

        self.seen += 1;

        let allow = match self.last_emit_us.get(path) {
            Some(&prev) if ts_us.saturating_sub(prev) < self.window_us => false,
            _ => true,
        };

        if allow {
            self.last_emit_us.insert(path.to_path_buf(), ts_us);
        }

        if self.seen % self.gc_every == 0 {
            let cutoff = ts_us.saturating_sub(self.window_us.saturating_mul(16));
            self.last_emit_us.retain(|_, &mut t| t >= cutoff);
        }

        allow
    }
}

pub struct FileMonitor {
    cfg: MonitorConfig,
}

impl FileMonitor {
    pub fn new(cfg: MonitorConfig) -> Result<Self, MonitorError> {
        if !cfg.watch_root.exists() {
            return Err(MonitorError::InvalidWatchRoot(cfg.watch_root));
        }
        Ok(Self { cfg })
    }

    pub async fn run(
        self,
        out_tx: mpsc::Sender<EditEvent>,
        mut shutdown: watch::Receiver<Shutdown>,
    ) -> Result<MonitorStatsSnapshot, MonitorError> {
        let watch_root = self.cfg.watch_root.clone();
        let ignore_top = Arc::new(self.cfg.ignore_top_level_dirs.clone());
        let ignore_ext = Arc::new(self.cfg.ignore_extensions.clone());

        let stats = Arc::new(MonitorStats::default());
        let (raw_tx, mut raw_rx) = mpsc::channel::<RawEvent>(self.cfg.raw_queue_capacity);

        let stop = Arc::new(AtomicBool::new(false));
        let stop_cb = Arc::clone(&stop);

        let stats_cb = Arc::clone(&stats);
        let watch_root_cb = watch_root.clone();
        let ignore_top_cb = Arc::clone(&ignore_top);
        let ignore_ext_cb = Arc::clone(&ignore_ext);

        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if stop_cb.load(Ordering::Acquire) {
                    return;
                }

                let event = match res {
                    Ok(ev) => ev,
                    Err(_) => {
                        stats_cb.watcher_errors.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                };

                let kind = match map_kind(&event.kind) {
                    Some(k) => k,
                    None => return,
                };

                for abs_path in &event.paths {
                    let rel = match abs_path.strip_prefix(&watch_root_cb) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    let rel_norm = normalize_rel(rel);

                    if should_ignore(&rel_norm, &ignore_top_cb, &ignore_ext_cb) {
                        continue;
                    }

                    let ts = match now_us() {
                        Some(v) => v,
                        None => return,
                    };

                    let raw = RawEvent {
                        rel_path: rel_norm,
                        timestamp_us: ts,
                        kind,
                    };

                    match raw_tx.try_send(raw) {
                        Ok(_) => {}
                        Err(_) => {
                            stats_cb.raw_dropped_overflow.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            })
            .map_err(MonitorError::WatcherCreation)?;

        watcher
            .watch(&watch_root, RecursiveMode::Recursive)
            .map_err(MonitorError::WatchStart)?;

        let mut debouncer = Debouncer::new(self.cfg.debounce_window);

        if *shutdown.borrow() != Shutdown::Run {
            stop.store(true, Ordering::Release);
            drop(watcher);
            return Ok(stats.snapshot());
        }

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    let current = *shutdown.borrow();
                    match current {
                        Shutdown::Run => {}
                        Shutdown::Immediate => {
                            break;
                        }
                        Shutdown::Graceful => {
                            self.drain_gracefully(&mut raw_rx, &out_tx, &mut debouncer, &stats).await;
                            break;
                        }
                    }
                }

                maybe = raw_rx.recv() => {
                    let raw = match maybe {
                        Some(ev) => ev,
                        None => break,
                    };

                    if !debouncer.should_emit(&raw.rel_path, raw.kind, raw.timestamp_us) {
                        stats.debounced.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }

                    let ev = EditEvent {
                        rel_path: raw.rel_path,
                        timestamp_us: raw.timestamp_us,
                        kind: raw.kind,
                    };

                    if try_emit(&out_tx, ev, self.cfg.out_overflow, &stats).is_err() {
                        break;
                    }
                }
            }
        }

        stop.store(true, Ordering::Release);
        drop(watcher);

        Ok(stats.snapshot())
    }

    async fn drain_gracefully(
        &self,
        raw_rx: &mut mpsc::Receiver<RawEvent>,
        out_tx: &mpsc::Sender<EditEvent>,
        debouncer: &mut Debouncer,
        stats: &MonitorStats,
    ) {
        let start = SystemTime::now();
        let mut drained = 0usize;

        while drained < self.cfg.graceful_drain_max_events {
            if start.elapsed().unwrap_or_default() > self.cfg.graceful_drain_max {
                break;
            }

            match raw_rx.try_recv() {
                Ok(raw) => {
                    drained += 1;
                    if !debouncer.should_emit(&raw.rel_path, raw.kind, raw.timestamp_us) {
                        stats.debounced.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    let ev = EditEvent {
                        rel_path: raw.rel_path,
                        timestamp_us: raw.timestamp_us,
                        kind: raw.kind,
                    };
                    if try_emit(out_tx, ev, self.cfg.out_overflow, stats).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
}

#[inline]
fn try_emit(
    out_tx: &mpsc::Sender<EditEvent>,
    ev: EditEvent,
    policy: OverflowPolicy,
    stats: &MonitorStats,
) -> Result<(), ()> {
    match policy {
        OverflowPolicy::DropNewest => match out_tx.try_send(ev) {
            Ok(_) => {
                stats.emitted.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => Err(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                stats.out_dropped_overflow.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        },
    }
}

#[inline]
fn map_kind(kind: &EventKind) -> Option<EditKind> {
    match kind {
        EventKind::Create(_) => Some(EditKind::Create),
        EventKind::Modify(_) => Some(EditKind::Modify),
        EventKind::Remove(_) => Some(EditKind::Delete),
        _ => None,
    }
}

fn normalize_rel(p: &Path) -> PathBuf {
    let mut out: Vec<std::ffi::OsString> = Vec::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(s) => out.push(s.to_os_string()),
            _ => {}
        }
    }
    out.into_iter().collect()
}

fn should_ignore(rel: &Path, ignore_top: &HashSet<String>, ignore_ext: &HashSet<String>) -> bool {
    if let Some(first) = rel.components().next() {
        if let Component::Normal(os) = first {
            if let Some(s) = os.to_str() {
                if ignore_top.contains(s) {
                    return true;
                }
            }
        }
    }
    if let Some(ext) = rel.extension().and_then(|e| e.to_str()) {
        if ignore_ext.contains(ext) {
            return true;
        }
    }
    false
}

#[derive(Debug, Error)]
pub enum MonitorError {
    #[error("Invalid watch root: {0}")]
    InvalidWatchRoot(PathBuf),
    #[error("Failed to create watcher: {0}")]
    WatcherCreation(#[source] notify::Error),
    #[error("Failed to start watching: {0}")]
    WatchStart(#[source] notify::Error),
}

// =========================================================================
// SECTION 2: Mouse Telemetry (GitMonitor)
// =========================================================================

/// Batería de Atención (Batería Kinética)
/// 
/// Acumula "energía" basada en la complejidad del movimiento humano y
/// la pierde con el tiempo (leaky bucket). Representa el esfuerzo cognitivo
/// disponible para validar código.
#[derive(Debug, Clone)]
pub struct AttentionBattery {
    pub level: f64,
    pub capacity: f64,
    pub last_decay: SystemTime,
    pub leak_rate: f64,
    pub causal_event_count: usize, // Conteo de eventos reales procesados
}

impl AttentionBattery {
    pub fn new() -> Self {
        Self {
            level: 0.0,
            capacity: 100.0,
            last_decay: SystemTime::now(),
            leak_rate: 0.5,
            causal_event_count: 0,
        }
    }

    /// Carga la batería basándose en entropía motora detectada y validación de eventos
    pub fn charge(&mut self, motor_entropy: f64, duration: Duration, hardware_events: usize, keyboard_hits: usize) {
        self.apply_decay();
        
        // [CAUSALIDAD] Validamos que han ocurrido eventos reales
        let events_delta = hardware_events.saturating_sub(self.causal_event_count);
        if events_delta == 0 {
            return; 
        }

        // La carga es proporcional a la entropía motora Y al volumen de teclado
        // El teclado es un indicador de "trabajo duro" (sweat).
        let mouse_charge = (motor_entropy * duration.as_secs_f64() * 5.0).min(events_delta as f64 * 0.1);
        let keyboard_charge = (keyboard_hits as f64 * 0.5).min(20.0); // Cap de carga por intervalo de teclado
        
        self.level = (self.level + mouse_charge + keyboard_charge).min(self.capacity);
        self.causal_event_count = hardware_events;
    }

    /// Consume energía (Costo Entrópico)
    pub fn consume(&mut self, cost: f64) -> bool {
        self.apply_decay();
        if self.level >= cost {
            self.level -= cost;
            true
        } else {
            false
        }
    }

    fn apply_decay(&mut self) {
        let now = SystemTime::now();
        if let Ok(elapsed) = now.duration_since(self.last_decay) {
            let decay = elapsed.as_secs_f64() * self.leak_rate;
            self.level = (self.level - decay).max(0.0);
            self.last_decay = now;
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitMonitorConfig {
    pub analysis_interval: Duration,
    pub mouse_buffer_size: usize,
    pub min_entropy: f64,
}

impl Default for GitMonitorConfig {
    fn default() -> Self {
        Self {
            analysis_interval: Duration::from_secs(5),
            mouse_buffer_size: 1024,
            min_entropy: 2.5,
        }
    }
}

pub struct GitMonitor {
    shutdown: CancellationToken,
    mouse_sentinel: MouseSentinel,
    input_rx: mpsc::Receiver<InputEvent>,
    file_rx: mpsc::Receiver<EditEvent>,
    analysis_interval: Duration,
    latest_metrics: Arc<RwLock<Option<KinematicMetrics>>>,
    latest_coupling: Arc<RwLock<f64>>,
    battery: Arc<RwLock<AttentionBattery>>,
    events_captured: Arc<RwLock<usize>>,
    keyboard_hits: Arc<AtomicU64>,
    watch_root: PathBuf,
    min_entropy: f64,
}

impl GitMonitor {
    pub fn new(
        config: GitMonitorConfig,
        input_rx: mpsc::Receiver<InputEvent>,
        file_rx: mpsc::Receiver<EditEvent>,
        watch_root: PathBuf,
        shutdown: CancellationToken,
    ) -> Result<Self, GitMonitorError> {
        Ok(Self {
            shutdown,
            mouse_sentinel: MouseSentinel::new(config.mouse_buffer_size),
            input_rx,
            file_rx,
            analysis_interval: config.analysis_interval,
            latest_metrics: Arc::new(RwLock::new(None)),
            latest_coupling: Arc::new(RwLock::new(1.0)),
            battery: Arc::new(RwLock::new(AttentionBattery::new())),
            events_captured: Arc::new(RwLock::new(0)),
            keyboard_hits: Arc::new(AtomicU64::new(0)),
            watch_root,
            min_entropy: config.min_entropy,
        })
    }

    pub fn get_metrics_ref(&self) -> Arc<RwLock<Option<KinematicMetrics>>> {
        self.latest_metrics.clone()
    }

    pub fn get_battery_ref(&self) -> Arc<RwLock<AttentionBattery>> {
        self.battery.clone()
    }

    pub fn get_coupling_ref(&self) -> Arc<RwLock<f64>> {
        self.latest_coupling.clone()
    }

    pub fn get_events_captured_ref(&self) -> Arc<RwLock<usize>> {
        self.events_captured.clone()
    }

    pub async fn start(mut self) -> Result<(), GitMonitorError> {
        info!("GovMonitor (Git-Gov) started with Cognitive Coupling");
        let mut interval = tokio::time::interval(self.analysis_interval);

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    info!("Shutdown signal received");
                    break;
                }

                Some(event) = self.input_rx.recv() => {
                    match event {
                        InputEvent::Mouse { x, y, .. } => {
                            self.mouse_sentinel.capture_event(x, y);
                        }
                        InputEvent::Keyboard { .. } => {
                            self.keyboard_hits.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    if let Ok(mut count) = self.events_captured.write() {
                        *count += 1;
                    }
                }

                Some(file_event) = self.file_rx.recv() => {
                    self.handle_file_event(file_event).await;
                }

                _ = interval.tick() => {
                    self.run_analysis();
                }
            }
        }

        info!("GitMonitor stopped cleanly");
        Ok(())
    }

    async fn handle_file_event(&mut self, event: EditEvent) {
        if event.kind == EditKind::Delete {
            return;
        }

        let full_path = self.watch_root.join(&event.rel_path);
        
        // [LEAN] Solo leemos archivos pequeños o fragmentos para evitar lag de IO
        if let Ok(content) = tokio::fs::read_to_string(&full_path).await {
            use crate::complexity::estimate_entropic_cost;
            use crate::stats::calculate_coupling_score;

            let entropic_cost = estimate_entropic_cost(&content);
            
            // Obtenemos la entropía motora actual (usamos velocity_entropy como proxy)
            let motor_entropy = self.latest_metrics.read()
                .ok()
                .and_then(|m| m.as_ref().map(|metrics| (metrics.velocity_entropy / 8.0).min(1.0)))
                .unwrap_or(0.0);

            // [TERMODINÁMICA] APLICAMOS DIFICULTAD (min_entropy)
            // Default 2.5 -> Factor 1.0. Higher min_entropy -> Higher cost.
            let difficulty_factor = self.min_entropy / 2.5;
            let adjusted_cost = entropic_cost * difficulty_factor;

            let has_energy = if let Ok(mut batt) = self.battery.write() {
                batt.consume(adjusted_cost)
            } else {
                false
            };

            let coupling = calculate_coupling_score(entropic_cost / 100.0, motor_entropy);
            
            if let Ok(mut latest) = self.latest_coupling.write() {
                *latest = (*latest * 0.7) + (coupling * 0.3); // Suavizado exponencial
            }

            if has_energy {
                info!(
                    "File change validated (Energy Balance): {:?} | Cost: {:.2} | Coupling: {:.2}",
                    event.rel_path.file_name().unwrap_or_default(),
                    entropic_cost,
                    coupling
                );
            } else {
                warn!(
                    "THERMODYNAMIC ANOMALY: Code injected without enough energy! {:?} | Cost: {:.2} | Battery: LOW",
                    event.rel_path.file_name().unwrap_or_default(),
                    entropic_cost
                );
            }
        }
    }

    fn run_analysis(&mut self) {
        match self.mouse_sentinel.analyze() {
            Ok(metrics) => {
                let events = self.events_captured.read().ok().map(|g| *g).unwrap_or(0);

                // [TERMODINÁMICA] Cargamos la batería con el esfuerzo detectado Y validación causal
                if let Ok(mut batt) = self.battery.write() {
                    let k_hits = self.keyboard_hits.swap(0, Ordering::SeqCst);
                    batt.charge(metrics.velocity_entropy / 8.0, self.analysis_interval, events, k_hits as usize);
                }

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


#[derive(Debug, Error)]
pub enum GitMonitorError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Monitor error: {0}")]
    Monitor(String),
}

// =========================================================================
// SECTION 3: Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rel_lexical() {
        assert_eq!(normalize_rel(Path::new("./a/./b/../c")), PathBuf::from("a/c"));
        assert_eq!(normalize_rel(Path::new("a/b/../../c")), PathBuf::from("c"));
    }

    #[test]
    fn ignore_top_level_exact_match() {
        let cfg = MonitorConfig::new(".");
        assert!(should_ignore(Path::new(".git/config"), &cfg.ignore_top_level_dirs, &cfg.ignore_extensions));
        assert!(!should_ignore(Path::new("src/.git/config"), &cfg.ignore_top_level_dirs, &cfg.ignore_extensions));
    }

    #[test]
    fn ignore_extension_exact() {
        let cfg = MonitorConfig::new(".");
        assert!(should_ignore(Path::new("daemon.log"), &cfg.ignore_top_level_dirs, &cfg.ignore_extensions));
        assert!(!should_ignore(Path::new("daemon.logger"), &cfg.ignore_top_level_dirs, &cfg.ignore_extensions));
    }

    #[test]
    fn debounce_never_suppresses_delete() {
        let mut d = Debouncer::new(Duration::from_millis(100));
        let p = Path::new("src/main.rs");
        assert!(d.should_emit(p, EditKind::Create, 1_000_000));
        assert!(!d.should_emit(p, EditKind::Modify, 1_000_050));
        assert!(d.should_emit(p, EditKind::Delete, 1_000_060));
    }
}
