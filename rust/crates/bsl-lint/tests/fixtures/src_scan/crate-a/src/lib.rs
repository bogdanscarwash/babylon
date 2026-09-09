//! Fixture crate A for bsl-lint's `namespace-unique` (b) integration test.
//! Doc comments may mention a code in backticks, e.g. `"E-FAKE-777"`, and
//! must NOT count as an emission site — only a real double-quoted string
//! literal outside a test block does.

/// Real emission site — deliberately duplicates crate-b's, for the
/// cross-file duplicate-E-code RED fixture (not allowlisted).
pub fn classify(x: i32) -> &'static str {
    match x {
        0 => "E-FAKE-777",
        _ => "E-FAKE-000",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_expected_code() {
        // "E-FAKE-778" here must NOT count as an emission site — it's a
        // test assertion, not a spec-code emission (the mutation-sensitive
        // control: comment out the `#[cfg(test)]` skip in scan_e_codes and
        // this line starts colliding with crate-b's real E-FAKE-778 site).
        assert_eq!(classify(0), "E-FAKE-777");
        assert_eq!("E-FAKE-778", "E-FAKE-778");
    }
}
