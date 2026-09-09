//! Grid-quantized bounded scalar sorts (`THE_FORMALISM` II.1): `Probability`,
//! `Intensity`, `Coefficient` on `𝔾 ∩ [0,1]`; `Ideology`, `Balance` on
//! `𝔾 ∩ [-1,1]`; `Ratio` on `𝔾 ∩ (0,∞)`. Construction quantizes and
//! validates (the Gatekeeper pattern, ported from Pydantic's
//! `AfterValidator`) — an out-of-range value is a loud `Err`, never
//! silently clamped.
use crate::grid::quantize;

/// A scalar out of its sort's declared bound (III.11 load-time rejection).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutOfBoundsError {
    /// The already-quantized value that failed the bound.
    pub value: f64,
    /// The sort's inclusive lower bound (exclusive for `Ratio`).
    pub lower: f64,
    /// The sort's inclusive upper bound (`f64::INFINITY` for `Ratio`).
    pub upper: f64,
}

macro_rules! bounded_scalar {
    ($name:ident, $lower:expr, $upper:expr) => {
        #[doc = concat!("Grid-quantized scalar on 𝔾 ∩ [", stringify!($lower), ", ", stringify!($upper), "].")]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name(f64);

        impl $name {
            /// Quantize and validate `value`.
            ///
            /// # Errors
            /// Returns [`OutOfBoundsError`] if the quantized value falls
            /// outside the sort's declared bound (III.11) — never clamps.
            pub fn new(value: f64) -> Result<Self, OutOfBoundsError> {
                let q = quantize(value);
                if !($lower..=$upper).contains(&q) {
                    return Err(OutOfBoundsError {
                        value: q,
                        lower: $lower,
                        upper: $upper,
                    });
                }
                Ok(Self(q))
            }

            /// The quantized inner value.
            #[must_use]
            pub fn get(self) -> f64 {
                self.0
            }
        }
    };
}

bounded_scalar!(Probability, 0.0, 1.0);
bounded_scalar!(Intensity, 0.0, 1.0);
bounded_scalar!(Coefficient, 0.0, 1.0);
bounded_scalar!(Ideology, -1.0, 1.0);
bounded_scalar!(Balance, -1.0, 1.0);

/// Grid-quantized scalar on `𝔾 ∩ (0, ∞)` — open lower bound, unbounded
/// above. Hand-rolled rather than macro-generated: the macro's bounds
/// assume a finite closed interval; `Ratio`'s law is "finite and strictly
/// positive after quantization". A positive input below `5e-7` quantizes
/// to `0.0` and is therefore rejected — loudly, per III.11, because a
/// ratio the grid cannot represent is not a ratio the algebra may use.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ratio(f64);

impl Ratio {
    /// Quantize and validate `value`.
    ///
    /// # Errors
    /// Returns [`OutOfBoundsError`] if the quantized value is not finite
    /// and strictly positive (III.11) — never clamps.
    pub fn new(value: f64) -> Result<Self, OutOfBoundsError> {
        let q = quantize(value);
        if !q.is_finite() || q <= 0.0 {
            return Err(OutOfBoundsError {
                value: q,
                lower: 0.0,
                upper: f64::INFINITY,
            });
        }
        Ok(Self(q))
    }

    /// The quantized inner value.
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Balance, Coefficient, Ideology, Probability, Ratio};

    #[test]
    fn probability_rejects_out_of_range() {
        assert!(Probability::new(1.5).is_err());
        assert!(Probability::new(-0.001).is_err());
    }

    #[test]
    fn probability_quantizes_on_construction() {
        let p = Probability::new(0.123_456_789).unwrap();
        assert!((p.get() - 0.123_457).abs() < 1e-12);
    }

    #[test]
    fn quantization_happens_before_the_bounds_check() {
        // 1.0000004 quantizes DOWN to 1.0 — legal. 1.0000005 quantizes UP
        // to 1.000001 — rejected. The order (quantize, then check) is the
        // Python Gatekeeper's order and is observable exactly here.
        assert!(Probability::new(1.000_000_4).is_ok());
        assert!(Probability::new(1.000_000_5).is_err());
    }

    #[test]
    fn signed_sorts_accept_the_full_interval() {
        assert!(Ideology::new(-1.0).is_ok());
        assert!(Ideology::new(1.0).is_ok());
        assert!(Ideology::new(-1.000_001).is_err());
        assert!(Balance::new(-0.5).is_ok());
    }

    #[test]
    fn ratio_lower_bound_is_open() {
        assert!(Ratio::new(0.0).is_err());
        assert!(Ratio::new(-1.0).is_err());
        assert!(Ratio::new(1e-6).is_ok()); // the smallest grid-representable ratio
    }

    #[test]
    fn ratio_rejects_what_the_grid_cannot_represent() {
        // 4e-7 quantizes to 0.0: strictly positive input, but not a
        // representable positive ratio — loud rejection, never a silent 0.
        assert!(Ratio::new(4e-7).is_err());
    }

    #[test]
    fn ratio_rejects_non_finite() {
        assert!(Ratio::new(f64::INFINITY).is_err());
        assert!(Ratio::new(f64::NAN).is_err());
    }

    #[test]
    fn coefficient_shares_the_unit_interval_law() {
        assert!(Coefficient::new(0.0).is_ok());
        assert!(Coefficient::new(1.0).is_ok());
        assert!(Coefficient::new(1.000_001).is_err());
    }
}
