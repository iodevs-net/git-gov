//! Kinematic metrics — input dynamics analysis
//!
//! This module analyzes motion dynamics derived from discrete input events
//! (mouse, pointer, stylus, etc.).
//!
//! RESPONSIBILITIES:
//! - Accept timestamped 2D points.
//! - Compute velocity, acceleration and jerk.
//! - Derive summary statistics (means, variances).
//!
//! NON-RESPONSIBILITIES:
//! - No OS hooks.
//! - No threading.
//! - No human/AI classification.
//! - No thresholds.
//!
//! Privacy note:
//! - Only relative motion + timing.
//! - No absolute screen position meaning.

use statrs::statistics::Statistics;

/// Single kinematic event (opaque input sample)
#[derive(Debug, Clone)]
pub struct KinematicEvent {
    pub t_ms: u64,
    pub x: f64,
    pub y: f64,
}

/// Output metrics describing motion dynamics
#[derive(Debug, Clone)]
pub struct KinematicMetrics {
    /// Mean velocity magnitude
    pub mean_velocity: f64,

    /// Velocity standard deviation
    pub velocity_std: f64,

    /// Mean acceleration magnitude
    pub mean_acceleration: f64,

    /// Mean jerk magnitude (rate of acceleration change)
    pub mean_jerk: f64,

    /// Total samples analyzed
    pub samples: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum KinematicError {
    #[error("At least 3 events are required")]
    InsufficientData,

    #[error("Non-monotonic timestamps detected")]
    InvalidTimestamps,
}

/// Analyze kinematic properties of a motion trace.
///
/// Requires ≥3 events to compute jerk.
///
/// Units:
/// - velocity: units/ms
/// - acceleration: units/ms²
/// - jerk: units/ms³
pub fn analyze_kinematics(
    events: &[KinematicEvent],
) -> Result<KinematicMetrics, KinematicError> {
    if events.len() < 3 {
        return Err(KinematicError::InsufficientData);
    }

    // Validate monotonic time
    for w in events.windows(2) {
        if w[1].t_ms <= w[0].t_ms {
            return Err(KinematicError::InvalidTimestamps);
        }
    }

    let mut velocities = Vec::with_capacity(events.len() - 1);
    let mut accelerations = Vec::with_capacity(events.len() - 2);
    let mut jerks = Vec::with_capacity(events.len() - 3);

    // Velocity
    for w in events.windows(2) {
        let dt = (w[1].t_ms - w[0].t_ms) as f64;
        let dx = w[1].x - w[0].x;
        let dy = w[1].y - w[0].y;

        let v = (dx * dx + dy * dy).sqrt() / dt;
        velocities.push(v);
    }

    // Acceleration
    for w in velocities.windows(2) {
        accelerations.push(w[1] - w[0]);
    }

    // Jerk
    for w in accelerations.windows(2) {
        jerks.push(w[1] - w[0]);
    }

    Ok(KinematicMetrics {
        mean_velocity: velocities.mean(),
        velocity_std: velocities.std_dev(),
        mean_acceleration: accelerations
            .iter()
            .map(|a| a.abs())
            .collect::<Vec<_>>()
            .mean(),
        mean_jerk: jerks
            .iter()
            .map(|j| j.abs())
            .collect::<Vec<_>>()
            .mean(),
        samples: events.len(),
    })
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_motion_has_low_jerk() {
        let events = vec![
            KinematicEvent { t_ms: 0, x: 0.0, y: 0.0 },
            KinematicEvent { t_ms: 10, x: 10.0, y: 0.0 },
            KinematicEvent { t_ms: 20, x: 20.0, y: 0.0 },
            KinematicEvent { t_ms: 30, x: 30.0, y: 0.0 },
        ];

        let m = analyze_kinematics(&events).unwrap();
        assert!(m.mean_jerk < 0.01);
    }

    #[test]
    fn rejects_non_monotonic_time() {
        let events = vec![
            KinematicEvent { t_ms: 10, x: 0.0, y: 0.0 },
            KinematicEvent { t_ms: 5, x: 5.0, y: 0.0 },
            KinematicEvent { t_ms: 20, x: 10.0, y: 0.0 },
        ];

        assert!(matches!(
            analyze_kinematics(&events),
            Err(KinematicError::InvalidTimestamps)
        ));
    }

    #[test]
    fn requires_minimum_events() {
        let events = vec![
            KinematicEvent { t_ms: 0, x: 0.0, y: 0.0 },
            KinematicEvent { t_ms: 10, x: 10.0, y: 0.0 },
        ];

        assert!(matches!(
            analyze_kinematics(&events),
            Err(KinematicError::InsufficientData)
        ));
    }
}
