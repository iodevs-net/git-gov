//! Structural metrics — algorithmic complexity via compression
//!
//! This module implements Normalized Compression Distance (NCD)
//! as a proxy for Kolmogorov complexity.
//!
//! RESPONSIBILITIES:
//! - Deterministic compression of byte slices.
//! - Compute NCD(x, y).
//!
//! NON-RESPONSIBILITIES:
//! - No thresholds or classification.
//! - No filesystem or Git IO.
//! - No policy decisions.
//!
//! Privacy note:
//! - Operates on opaque byte slices.
//! - Caller decides what data is admissible.

use std::io::Cursor;
use zstd::stream::encode_all;

/// Output of structural comparison.
///
/// All values are descriptive and auditable.
#[derive(Debug, Clone)]
pub struct StructuralMetrics {
    /// Normalized Compression Distance (may slightly exceed [0,1] in practice).
    pub ncd: f64,

    /// Raw size of x (bytes).
    pub raw_x: usize,

    /// Raw size of y (bytes).
    pub raw_y: usize,

    /// Compressed size of x.
    pub c_x: usize,

    /// Compressed size of y.
    pub c_y: usize,

    /// Compressed size of concatenation xy.
    pub c_xy: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum StructuralError {
    #[error("Input slices must not be empty")]
    EmptyInput,

    #[error("Compression failure")]
    CompressionFailed,
}

/// Compute Normalized Compression Distance (NCD).
///
/// NCD(x, y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))
///
/// Notes:
/// - Real-world compressors may yield NCD < 0 or > 1.
/// - Values are not clamped; interpretation is caller responsibility.
///
/// # Determinism
/// - Compression level is fixed.
/// - No dictionaries.
/// - Single-threaded encode path.
pub fn analyze_ncd(
    x: &[u8],
    y: &[u8],
) -> Result<StructuralMetrics, StructuralError> {
    if x.is_empty() || y.is_empty() {
        return Err(StructuralError::EmptyInput);
    }

    let raw_x = x.len();
    let raw_y = y.len();

    let c_x = compress_size(x)?;
    let c_y = compress_size(y)?;

    let mut xy = Vec::with_capacity(raw_x + raw_y);
    xy.extend_from_slice(x);
    xy.extend_from_slice(y);

    let c_xy = compress_size(&xy)?;

    let min_c = c_x.min(c_y) as f64;
    let max_c = c_x.max(c_y) as f64;

    let ncd = if max_c == 0.0 {
        0.0
    } else {
        (c_xy as f64 - min_c) / max_c
    };

    Ok(StructuralMetrics {
        ncd,
        raw_x,
        raw_y,
        c_x,
        c_y,
        c_xy,
    })
}

// -----------------------------------------------------------------------------
// Internal helpers
// -----------------------------------------------------------------------------

/// Deterministic compression size using zstd.
///
/// Compression parameters:
/// - Level: 3 (stable, low variance, fast)
/// - No dictionary
/// - Single-threaded
fn compress_size(data: &[u8]) -> Result<usize, StructuralError> {
    let compressed = encode_all(Cursor::new(data), 3)
        .map_err(|_| StructuralError::CompressionFailed)?;

    Ok(compressed.len())
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_inputs_have_low_ncd() {
        let x = b"fn add(a: i32, b: i32) -> i32 { a + b }";

        let m = analyze_ncd(x, x).unwrap();
        assert!(m.ncd < 0.1);
    }

    #[test]
    fn different_inputs_have_higher_ncd() {
        let x = b"fn add(a: i32, b: i32) -> i32 { a + b }";
        let y = b"SELECT user_id, COUNT(*) FROM logs GROUP BY user_id";

        let m = analyze_ncd(x, y).unwrap();
        assert!(m.ncd > 0.3);
    }

    #[test]
    fn rejects_empty_input() {
        let x = b"abc";
        let y = b"";

        assert!(matches!(
            analyze_ncd(x, y),
            Err(StructuralError::EmptyInput)
        ));
    }
}
