//! The Program 27 quantization retraction (`THE_FORMALISM` II.1, `L-GRID`):
//! ports `babylon.kernel.math.quantize` byte-for-byte. `ROUND_HALF_UP`, ties
//! away from zero, on the 10⁻⁶ grid.
//!
//! This is a **cross-language conformance surface**: the test vector below
//! was verified against the live Python (`babylon.kernel.math.quantize`)
//! on 2026-07-30 before this file was written — transcription discipline,
//! not assumption.

/// Decimal digits of the grid: values live on multiples of 10⁻⁶.
pub const GRID_PRECISION: u32 = 6;
const GRID: f64 = 1_000_000.0; // 10^GRID_PRECISION

/// Snap `value` onto the 10⁻⁶ grid, `ROUND_HALF_UP` (ties away from zero) —
/// the exact algorithm in `src/babylon/kernel/math.py::quantize`.
#[must_use]
pub fn quantize(value: f64) -> f64 {
    if value >= 0.0 {
        (value * GRID + 0.5).floor() / GRID
    } else {
        let q = -((-value * GRID + 0.5).floor()) / GRID;
        // A magnitude under the grid's half-step floors to zero and the
        // leading negation mints -0.0; the Python reference returns +0.0
        // there (probe 2026-07-30) and the tick hash encodes BIT PATTERNS,
        // so the signed zero must never escape.
        if q == 0.0 {
            0.0
        } else {
            q
        }
    }
}

#[cfg(test)]
mod tests {
    use super::quantize;

    /// Cross-language conformance vector: expected values computed by
    /// running `babylon.kernel.math.quantize` in Python (verified
    /// 2026-07-30: `0.123456789 -> 0.123457`, `-0.123456789 -> -0.123457`,
    /// `0.0 -> 0.0`, `1.0000005 -> 1.000001`, `-1.0000005 -> -1.000001`).
    #[test]
    fn matches_the_python_quantize_conformance_vector() {
        let cases: &[(f64, f64)] = &[
            (0.123_456_789, 0.123_457),
            (-0.123_456_789, -0.123_457),
            (0.0, 0.0),
            (1.000_000_5, 1.000_001), // half-away-from-zero tie
            (-1.000_000_5, -1.000_001),
        ];
        for &(input, expected) in cases {
            assert!(
                (quantize(input) - expected).abs() < 1e-12,
                "quantize({input}) = {}, expected {expected}",
                quantize(input)
            );
        }
    }

    #[test]
    fn is_idempotent() {
        // L-GRID: q ∘ q = q
        let v = 0.987_654_321;
        assert!((quantize(quantize(v)) - quantize(v)).abs() < 1e-15);
    }

    #[test]
    fn negative_zero_input_stays_on_the_grid() {
        // -0.0 >= 0.0 is true in IEEE-754, so it takes the positive branch;
        // the output is 0.0 (positive), not -0.0 — pinned so the tick hash's
        // bit-pattern encoding never sees a quantized -0.0.
        let q = quantize(-0.0);
        assert_eq!(q.to_bits(), 0.0_f64.to_bits());
    }

    /// Inputs in the open interval (−5·10⁻⁷, 0) floor to a zero magnitude
    /// on the negative branch, and the branch's leading negation would mint
    /// −0.0. The Python reference emits POSITIVE zero there (live probe
    /// 2026-07-30: `quantize(-4e-7)` → bits `0x0`), and the tick hash
    /// encodes bit patterns — a signed zero is a conformance break.
    #[test]
    fn small_negative_inputs_snap_to_positive_zero() {
        for v in [-4e-7, -4.9999e-7, -1e-9] {
            let q = quantize(v);
            assert_eq!(
                q.to_bits(),
                0.0_f64.to_bits(),
                "quantize({v}) leaked a signed zero"
            );
        }
        // The half-step boundary itself still rounds away from zero, per
        // the Python probe: quantize(-5e-7) -> -1e-6 on both sides.
        assert_eq!(quantize(-5e-7).to_bits(), (-1e-6_f64).to_bits());
    }
}
