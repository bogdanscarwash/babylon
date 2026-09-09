//! Per-intrinsic golden vectors for `babylon_kernel::transcendental` (ADR176
//! r21, Task 1 of the #576 intrinsic-host train).
//!
//! **Zero-tolerance contract:** these are `assert_eq!` on `f64::to_bits()`,
//! never `abs(a - b) < eps`. `libm 0.2.16` at `default-features = false` is
//! pinned soft-float on every target Babylon ships (`x86_64`, `aarch64`) —
//! see `babylon_kernel::transcendental`'s module doc for the verified
//! dispatch analysis. Any drift here is a determinism regression, never
//! "the math got better" (the `rng.rs:191-192` precedent).
//!
//! Constants were filled from the first green run (2026-08-17, this train's
//! Task 1.3) and are **byte-pinned thereafter** — any later divergence is a
//! determinism regression, never "the math got better."

use babylon_kernel::transcendental::{exp, ln};

/// `exp` roster: zeros (both signs), unit inputs, a mid-range value, the two
/// boundary values nearest `f64::exp`'s overflow edge (`709.0` finite,
/// `709.782712893384` the largest finite-result argument), the underflow
/// edge (`-745.0`, near where `exp` flushes to `0.0`), and a tiny positive
/// input (`1e-300`).
#[test]
fn exp_golden_vectors() {
    let roster: [(f64, u64); 9] = [
        (0.0, 0x3ff0_0000_0000_0000),
        (-0.0, 0x3ff0_0000_0000_0000),
        (1.0, 0x4005_bf0a_8b14_576a),
        (-1.0, 0x3fd7_8b56_362c_ef38),
        (0.5, 0x3ffa_6129_8e1e_069c),
        (709.0, 0x7fdd_422d_2be5_dc9b),
        (709.782_712_893_384, 0x7fef_ffff_ffff_ff2a),
        (-745.0, 0x0000_0000_0000_0001),
        (1e-300, 0x3ff0_0000_0000_0000),
    ];
    for (x, expected_bits) in roster {
        let observed = exp(x).to_bits();
        assert_eq!(
            observed, expected_bits,
            "exp({x}) = 0x{observed:016x}, expected 0x{expected_bits:016x}"
        );
    }
}

/// `ln` roster: the identity point (`1.0`), a small integer, a fraction, the
/// smallest positive normal `f64`, another small integer, a very large
/// magnitude, and a value one ULP above `1.0` (probing rounding near the
/// zero crossing).
#[test]
fn ln_golden_vectors() {
    let roster: [(f64, u64); 7] = [
        (1.0, 0x0000_0000_0000_0000),
        (2.0, 0x3fe6_2e42_fefa_39ef),
        (0.5, 0xbfe6_2e42_fefa_39ef),
        (f64::MIN_POSITIVE, 0xc086_232b_dd7a_bcd2),
        (3.0, 0x3ff1_93ea_7aad_030a),
        (1e300, 0x4085_9634_47f8_7fb5),
        (1.000_000_000_000_000_2, 0x3caf_ffff_ffff_ffff),
    ];
    for (x, expected_bits) in roster {
        let observed = ln(x).to_bits();
        assert_eq!(
            observed, expected_bits,
            "ln({x}) = 0x{observed:016x}, expected 0x{expected_bits:016x}"
        );
    }
}
