//! `Currency`: i128 fixed-point micro-units (spec §6.1). Overflow is a loud
//! III.11 failure — `checked_*` everywhere, never wrapping or saturating.
//!
//! Sign domain (OPEN — flagged in the Phase-1 plan's open questions): the
//! Python model constrains `Currency` non-negative
//! (`models/types.py::Currency`, `Field(ge=0.0)`); this port keeps the
//! underlying representation signed (`i128`) because intermediate deltas
//! (e.g. a dispossession transfer) are naturally signed, and does NOT
//! re-impose non-negativity as a type invariant here. If the Director or a
//! later review wants non-negativity enforced at the type level, that is a
//! narrow follow-up (a `NonNegativeCurrency` boundary wrapper), not a
//! redesign of this module.
//!
//! The four operators below are the spec-pinned set, byte-specified in
//! ``docs/reference/determinism-contract.rst`` (*Currency Operator
//! Semantics*): `± Currency`, `× Coefficient` (half-even),
//! `÷ Currency → Coefficient` (i256 intermediate, half-even),
//! `÷ integer` (half-even). A fifth, `× Ratio` (half-even), joins them per
//! `bsl-language.rst` §3.2's addendum (Director ruling 2026-08-11,
//! #492/ADR194) — see [`Currency::mul_ratio`].
use crate::scalars::{Coefficient, Ratio};
use bnum::types::I256;

/// A loud, non-recoverable-by-the-algebra overflow (III.11: run-time loud
/// failure — the caller is expected to let this propagate, per spec §9
/// "checked-arithmetic overflow panics with context").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyOverflow {
    /// The operator that overflowed, for the failure message.
    pub op: &'static str,
}

/// Fixed-point currency: the value in micro-units (1 unit = 1,000,000
/// micro-units).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Currency(i128);

const MICRO: i128 = 1_000_000;
const MICRO_F64: f64 = 1_000_000.0;

impl Currency {
    /// Wrap a raw micro-unit amount.
    #[must_use]
    pub fn from_micro_units(micro: i128) -> Self {
        Self(micro)
    }

    /// The raw micro-unit amount.
    #[must_use]
    pub fn micro_units(self) -> i128 {
        self.0
    }

    /// `Currency + Currency → Currency` (checked; spec §6.1).
    ///
    /// # Errors
    /// Returns [`CurrencyOverflow`] if the sum leaves the i128 range —
    /// loud, never wrapping (III.11).
    pub fn checked_add(self, other: Self) -> Result<Self, CurrencyOverflow> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(CurrencyOverflow {
                op: "Currency + Currency",
            })
    }

    /// `Currency - Currency → Currency` (checked; spec §6.1).
    ///
    /// # Errors
    /// Returns [`CurrencyOverflow`] if the difference leaves the i128
    /// range — loud, never wrapping (III.11).
    pub fn checked_sub(self, other: Self) -> Result<Self, CurrencyOverflow> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(CurrencyOverflow {
                op: "Currency - Currency",
            })
    }

    /// `Currency × Coefficient → Currency`, rounded half-even to
    /// micro-units.
    ///
    /// `Coefficient` lives on the 10⁻⁶ grid, so its exact value is the
    /// rational `numerator / 1_000_000` with an integer numerator in
    /// `[0, 1_000_000]`. This implementation recovers that integer
    /// numerator, multiplies the two integer representations directly
    /// (never casting the i128 side to `f64`, which loses precision above
    /// 2⁵³ ≈ 9.0e15 micro-units — well inside the nationwide-scale headroom
    /// §6.1 pins i128 for), and divides back down by `1_000_000` half-even
    /// in one step.
    ///
    /// # Errors
    /// Returns [`CurrencyOverflow`] if the pre-rounding product leaves the
    /// i128 range.
    pub fn mul_coefficient(self, coeff: Coefficient) -> Result<Self, CurrencyOverflow> {
        // Grid-safe recovery of the integer numerator: coeff ∈ [0,1] on the
        // 10⁻⁶ grid, so coeff·10⁶ is integer-valued up to float error and
        // bounded by 1_000_000 — the round+cast is exact by construction.
        #[allow(clippy::cast_possible_truncation)]
        let numerator = (coeff.get() * MICRO_F64).round() as i128;
        let product = self.0.checked_mul(numerator).ok_or(CurrencyOverflow {
            op: "Currency * Coefficient (pre-round)",
        })?;
        Ok(Self(round_half_even_div(product, MICRO)))
    }

    /// `Currency × Ratio → Currency`, rounded half-even to micro-units
    /// (`bsl-language.rst` §3.2 addendum, Director ruling 2026-08-11,
    /// #492/ADR194).
    ///
    /// The **only** other legal Currency-mixing operation beyond the
    /// original four (§6.1's design-doc table): a defconst-declared
    /// positive coefficient outside `Coefficient`'s `[0,1]` domain — the
    /// construct gap the Territory eviction pipeline's `rent_spike_multiplier`
    /// (declared domain `(0, ∞)`), Metabolism's `entropy_factor` and
    /// Lifecycle's `pareto_alpha`/`early_mortality_modifier`/
    /// `carceral_transition_modifier` all hit (director-gate #492). `Ratio`
    /// is the kernel's OWN pre-existing `𝔾 ∩ (0, ∞)` sort (`scalars.rs`) —
    /// reusing it, rather than minting a new bounded scalar, is what makes
    /// this "scalar multiplication, not new mathematics" (the ruling's own
    /// framing).
    ///
    /// Identical arithmetic to [`Currency::mul_coefficient`]: `Ratio` lives
    /// on the same `10⁻⁶` grid (`grid::quantize`), so its exact value is
    /// `numerator / 1_000_000` with an integer numerator — unbounded above
    /// unlike `Coefficient`'s `[0, 1_000_000]`, but still comfortably exact
    /// in `f64` for any realistic scale factor (magnitudes far beyond
    /// 2⁵³/10⁶ ≈ 9.0×10⁹ are not ratios this algebra produces), and the
    /// `checked_mul` below catches a product that would leave `i128`
    /// regardless of how large the numerator is.
    ///
    /// # Errors
    /// Returns [`CurrencyOverflow`] if the pre-rounding product leaves the
    /// i128 range.
    pub fn mul_ratio(self, ratio: Ratio) -> Result<Self, CurrencyOverflow> {
        #[allow(clippy::cast_possible_truncation)]
        let numerator = (ratio.get() * MICRO_F64).round() as i128;
        let product = self.0.checked_mul(numerator).ok_or(CurrencyOverflow {
            op: "Currency * Ratio (pre-round)",
        })?;
        Ok(Self(round_half_even_div(product, MICRO)))
    }

    /// `Currency ÷ Currency → Coefficient`, i256 intermediate, half-even.
    ///
    /// # Panics
    /// Panics if `other` is zero (division by zero), or if the true ratio
    /// falls outside `Coefficient`'s `[0, 1]` domain — both are III.11
    /// caller bugs (the algebra must guarantee `0 ≤ self/other ≤ 1` before
    /// invoking this projection), not recoverable conditions.
    #[must_use]
    pub fn div_currency(self, other: Self) -> Coefficient {
        let numerator = widen(self.0) * widen(MICRO);
        let ratio = round_half_even_div_i256(numerator, widen(other.0));
        let as_i128: i128 = ratio
            .try_into()
            .expect("i256 intermediate must fit i128 for a [0,1] ratio");
        // as_i128 ∈ [0, 1_000_000] for an in-domain ratio — exact in f64.
        #[allow(clippy::cast_precision_loss)]
        let value = as_i128 as f64 / MICRO_F64;
        Coefficient::new(value).expect("out-of-[0,1] Currency ratio: III.11 caller bug")
    }

    /// `Currency ÷ integer → Currency`, half-even.
    ///
    /// # Panics
    /// Panics if `divisor` is zero — a III.11 caller bug, not a
    /// recoverable condition.
    #[must_use]
    pub fn div_integer(self, divisor: i128) -> Self {
        Self(round_half_even_div(self.0, divisor))
    }
}

/// Half-even (banker's) rounding integer division — the `round_half_even`
/// kernel intrinsic (spec §6.2), pinned here for `Currency`'s own operators;
/// re-exported at crate root for BSL's numeric-annex use once Phase 2 wires
/// it as a callable intrinsic.
///
/// # Panics
/// Panics if `denominator` is zero, or on the (unreachable-for-`MICRO`)
/// `2 × remainder` overflow.
#[must_use]
pub fn round_half_even_div(numerator: i128, denominator: i128) -> i128 {
    let q = numerator / denominator;
    let r = numerator % denominator;
    let twice_r = r
        .checked_mul(2)
        .expect("round_half_even_div: 2*remainder overflow");
    // `unsigned_abs`, never `abs`: `i128::MIN.abs()` overflows (i128 holds
    // −2¹²⁷ but not +2¹²⁷), which panics in dev and — with release's
    // `overflow-checks = false` — WRAPS back to a negative i128, inverting
    // this comparison for every input. `unsigned_abs` returns u128, where
    // every magnitude is representable, so the compare is total.
    match twice_r.unsigned_abs().cmp(&denominator.unsigned_abs()) {
        std::cmp::Ordering::Less => q,
        std::cmp::Ordering::Greater => q + numerator.signum() * denominator.signum(),
        std::cmp::Ordering::Equal => {
            if q % 2 == 0 {
                q
            } else {
                q + numerator.signum() * denominator.signum()
            }
        }
    }
}

/// `i128 → I256`, the one widening this module needs.
///
/// bnum 0.14 replaced `From<primitive>` with `TryFrom<primitive>` and
/// removed the `ZERO`/`ONE` consts, so every widening is now fallible at
/// the type level. It is not fallible in fact: `I256` holds 256 bits and
/// `i128` holds 128, so **every** `i128` is representable and the `Err`
/// arm is unreachable by width. Centralised here so that invariant is
/// stated once rather than restated at five call sites.
fn widen(value: i128) -> I256 {
    I256::try_from(value).expect("every i128 fits i256 by construction — 128 bits into 256")
}

/// The same half-even division at i256 width, for `div_currency`'s
/// intermediate — kept private: the public intrinsic surface is the i128
/// form above.
fn round_half_even_div_i256(numerator: I256, denominator: I256) -> I256 {
    let q = numerator / denominator;
    let r = numerator % denominator;
    let two = widen(2);
    let twice_r = r * two; // i256 headroom: cannot overflow for i128-derived inputs
    let step = numerator.signum() * denominator.signum();
    match twice_r.abs().cmp(&denominator.abs()) {
        std::cmp::Ordering::Less => q,
        std::cmp::Ordering::Greater => q + step,
        std::cmp::Ordering::Equal => {
            if q % two == widen(0) {
                q
            } else {
                q + step
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{round_half_even_div, Currency, CurrencyOverflow};
    use crate::scalars::{Coefficient, Ratio};

    #[test]
    fn half_even_div_survives_the_most_negative_denominator() {
        // `i128::MIN.abs()` overflows: i128 holds -2^127 but not +2^127.
        // In dev that panics; in release — where this workspace declares no
        // [profile.release] and so inherits `overflow-checks = false` — it
        // WRAPS back to i128::MIN, a negative value. The comparison at the
        // heart of the rounding rule then reads "|2r| > |d|" for every
        // input and the intrinsic silently returns the wrong quotient.
        //
        // 1 / i128::MIN is ≈ -5.9e-39, which half-even-rounds to 0. The
        // wrapped path returns -1. Wrong money, quietly, in release only —
        // the III.11 failure mode exactly.
        assert_eq!(round_half_even_div(1, i128::MIN), 0);
        assert_eq!(round_half_even_div(-1, i128::MIN), 0);
        // The symmetric hazard on the numerator side: |2r| can itself reach
        // i128::MIN's magnitude for a large enough divisor.
        assert_eq!(round_half_even_div(i128::MIN, i128::MIN), 1);
    }

    #[test]
    fn add_overflow_is_loud_not_wrapping() {
        let max = Currency::from_micro_units(i128::MAX);
        let one = Currency::from_micro_units(1);
        assert_eq!(
            max.checked_add(one),
            Err(CurrencyOverflow {
                op: "Currency + Currency"
            })
        );
    }

    #[test]
    fn sub_underflow_is_loud_not_wrapping() {
        let min = Currency::from_micro_units(i128::MIN);
        let one = Currency::from_micro_units(1);
        assert_eq!(
            min.checked_sub(one),
            Err(CurrencyOverflow {
                op: "Currency - Currency"
            })
        );
    }

    #[test]
    fn round_half_even_div_ties_to_even() {
        assert_eq!(round_half_even_div(5, 2), 2); // 2.5 -> 2 (even)
        assert_eq!(round_half_even_div(7, 2), 4); // 3.5 -> 4 (even)
        assert_eq!(round_half_even_div(-5, 2), -2); // -2.5 -> -2 (even)
        assert_eq!(round_half_even_div(-7, 2), -4); // -3.5 -> -4 (even)
    }

    #[test]
    fn round_half_even_div_off_tie_rounds_to_nearest() {
        assert_eq!(round_half_even_div(7, 3), 2); // 2.33 -> 2
        assert_eq!(round_half_even_div(8, 3), 3); // 2.67 -> 3
        assert_eq!(round_half_even_div(-8, 3), -3);
    }

    #[test]
    fn div_integer_matches_round_half_even() {
        let c = Currency::from_micro_units(5);
        assert_eq!(c.div_integer(2), Currency::from_micro_units(2));
    }

    #[test]
    fn mul_coefficient_is_exact_integer_arithmetic() {
        // 10.000000 units × 0.5 = 5.000000 units, exactly.
        let c = Currency::from_micro_units(10 * 1_000_000);
        let half = Coefficient::new(0.5).unwrap();
        assert_eq!(c.mul_coefficient(half).unwrap().micro_units(), 5_000_000);
    }

    #[test]
    fn mul_coefficient_half_even_on_the_micro_unit_tie() {
        // 1 micro-unit × 0.5 = 0.5 micro-units -> 0 (even), not 1.
        let c = Currency::from_micro_units(1);
        let half = Coefficient::new(0.5).unwrap();
        assert_eq!(c.mul_coefficient(half).unwrap().micro_units(), 0);
        // 3 micro-units × 0.5 = 1.5 -> 2 (even), not 1.
        let c3 = Currency::from_micro_units(3);
        assert_eq!(c3.mul_coefficient(half).unwrap().micro_units(), 2);
    }

    #[test]
    fn mul_coefficient_survives_beyond_f64_exact_range() {
        // 2^53 + 1 micro-units is exactly the value an f64 path would
        // silently round; the integer path must not.
        let beyond = Currency::from_micro_units((1_i128 << 53) + 1);
        let one = Coefficient::new(1.0).unwrap();
        assert_eq!(
            beyond.mul_coefficient(one).unwrap().micro_units(),
            (1_i128 << 53) + 1
        );
    }

    #[test]
    fn mul_ratio_is_exact_integer_arithmetic() {
        // 10.000000 units × 1.5 = 15.000000 units, exactly — the whole
        // point: 1.5 is outside Coefficient's [0,1] domain (#492/ADR194).
        let c = Currency::from_micro_units(10 * 1_000_000);
        let one_point_five = Ratio::new(1.5).unwrap();
        assert_eq!(
            c.mul_ratio(one_point_five).unwrap().micro_units(),
            15_000_000
        );
    }

    #[test]
    fn mul_ratio_half_even_on_the_micro_unit_tie() {
        // 1 micro-unit × 0.5 would tie, but Ratio's domain is (0, ∞) with
        // no relation to Coefficient — use an equivalent tie via a Ratio
        // value: 1 micro-unit × 2.5 = 2.5 -> 2 (even), not 3.
        let c = Currency::from_micro_units(1);
        let two_point_five = Ratio::new(2.5).unwrap();
        assert_eq!(c.mul_ratio(two_point_five).unwrap().micro_units(), 2);
        // 3 micro-units × 2.5 = 7.5 -> 8 (even), not 7.
        let c3 = Currency::from_micro_units(3);
        assert_eq!(c3.mul_ratio(two_point_five).unwrap().micro_units(), 8);
    }

    #[test]
    fn mul_ratio_survives_beyond_f64_exact_range() {
        let beyond = Currency::from_micro_units((1_i128 << 53) + 1);
        let one = Ratio::new(1.0).unwrap();
        assert_eq!(
            beyond.mul_ratio(one).unwrap().micro_units(),
            (1_i128 << 53) + 1
        );
    }

    #[test]
    fn mul_ratio_accepts_values_far_outside_the_unit_interval() {
        // rent_spike_multiplier's declared domain is (0, ∞) — moddable to
        // 2.0 per the port train's fixture; a large ratio (10x, matching
        // pareto_alpha/carceral_transition_modifier's (0,10] ceiling class)
        // is ordinary arithmetic here, not a special case.
        let c = Currency::from_micro_units(1_000_000); // 1.0
        let ten = Ratio::new(10.0).unwrap();
        assert_eq!(c.mul_ratio(ten).unwrap().micro_units(), 10_000_000);
    }

    #[test]
    fn mul_ratio_overflow_is_loud() {
        let huge = Currency::from_micro_units(i128::MAX);
        let two = Ratio::new(2.0).unwrap();
        let err = huge.mul_ratio(two).unwrap_err();
        assert_eq!(err.op, "Currency * Ratio (pre-round)");
    }

    #[test]
    fn div_currency_yields_the_grid_ratio() {
        let a = Currency::from_micro_units(1_000_000); // 1.0
        let b = Currency::from_micro_units(3_000_000); // 3.0
        let ratio = a.div_currency(b);
        // 1/3 on the 10⁻⁶ grid, half-even: 333333.33... -> 333333.
        assert!((ratio.get() - 0.333_333).abs() < 1e-12);
    }

    #[test]
    fn div_currency_of_equal_values_is_exactly_one() {
        let a = Currency::from_micro_units(7_654_321);
        assert!((a.div_currency(a).get() - 1.0).abs() < 1e-15);
    }

    #[test]
    fn div_currency_survives_i128_scale_numerators() {
        // self × 10⁶ overflows i128 for large self — the i256 intermediate
        // is load-bearing, not decorative.
        let big = Currency::from_micro_units(i128::MAX / 2);
        let bigger = Currency::from_micro_units(i128::MAX);
        let ratio = big.div_currency(bigger);
        assert!((ratio.get() - 0.5).abs() < 1e-6);
    }
}
