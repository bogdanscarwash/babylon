//! Fixture crate B for bsl-lint's `namespace-unique` (b) integration test.

/// Real emission site — duplicates crate-a's `"E-FAKE-777"` (unallowlisted:
/// the RED-phase target) and independently emits `"E-FAKE-778"` (should
/// stay unflagged: crate-a's only `"E-FAKE-778"` occurrence is test-only).
pub fn other_classify(x: i32) -> &'static str {
    match x {
        0 => "E-FAKE-777",
        _ => "E-FAKE-778",
    }
}
