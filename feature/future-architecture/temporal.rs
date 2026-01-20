//! Temporal metrics — human vs machine cadence analysis
//!
//! This module computes time-based metrics from an ordered stream of events.
//!
//! RESPONSIBILITIES:
//! - Compute inter-event intervals.
//! - Compute mean and standard deviation.
//! - Compute burstiness B.
//!
//! NON-RESPONSIBILITIES:
//! - No classification (human / AI).
//! - No thresholds or policy.
//! - No IO, async, logging, or clocks.
//!
//! All inputs are assumed to be:
//! - Monotonic in time (caller responsibility).
//! - Already filtered / debounced upstream.

use statrs::statistics::{Mean, Variance};

/// Output of temporal analysis.
///
/// All values are *descriptive*, not normative.
#[derive(Debug, Clone)]
pub struct TemporalMetrics {
    /// Burstiness metric in [-1, 1].
    pub burstiness: f64,

    /// Mean inter-event interval (microseconds).
    pub mean_interval_us: f64,

    /// Standard deviation of inter-event intervals (microseconds).
    pub stddev_interval_us: f64,

    /// Number of intervals used in computation.
    pub sample_count: usize,
}

/// Errors are explicit and boring — as they should be.
#[derive(Debug, thiserror::Error)]
pub enum TemporalError {
    #[error("At least two timestamps are required")]
    InsufficientData,

    #[error("Non-monotonic timestamps detected")]
    NonMonotonicInput,
}

/// Compute temporal metrics from an ordered slice of timestamps (µs).
///
/// # Contract
/// - `timestamps_us` MUST be strictly increasing.
/// - Units are microseconds.
/// - Caller owns time semantics.
///
/// # Formula
/// Burstiness:
/// B = (σ - μ) / (σ + μ)
///
/// References:
/// - Goh & Barabási (2008)
pub fn analyze_timestamps(
    timestamps_us: &[u128],
) -> Result<TemporalMetrics, TemporalError> {
    if timestamps_us.len() < 2 {
        return Err(TemporalError::InsufficientData);
    }

    // Compute inter-event intervals
    let mut intervals = Vec::with_capacity(timestamps_us.len() - 1);

    for w in timestamps_us.windows(2) {
        let prev = w[0];
        let next = w[1];

        if next <= prev {
            return Err(TemporalError::NonMonotonicInput);
        }

        intervals.push((next - prev) as f64);
    }

    if intervals.len() < 1 {
        return Err(TemporalError::InsufficientData);
    }

    let mean = intervals.mean();
    let stddev = intervals.variance().sqrt();

    // Burstiness definition is undefined for μ + σ == 0
    let burstiness = if mean + stddev == 0.0 {
        0.0
    } else {
        (stddev - mean) / (stddev + mean)
    };

    Ok(TemporalMetrics {
        burstiness,
        mean_interval_us: mean,
        stddev_interval_us: stddev,
        sample_count: intervals.len(),
    })
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bursty_human_like_pattern() {
        // Highly variable intervals
        let ts = vec![
            0,
            10,
            100,
            120,
            500,
            520,
        ];

        let m = analyze_timestamps(&ts).unwrap();
        assert!(m.burstiness > 0.0);
    }

    #[test]
    fn machine_like_uniform_pattern() {
        // Uniform cadence
        let ts = vec![
            0,
            100,
            200,
            300,
            400,
            500,
        ];

        let m = analyze_timestamps(&ts).unwrap();
        assert!(m.burstiness < 0.0);
    }

    #[test]
    fn rejects_non_monotonic_input() {
        let ts = vec![0, 100, 50, 200];
        assert!(matches!(
            analyze_timestamps(&ts),
            Err(TemporalError::NonMonotonicInput)
        ));
    }
}
