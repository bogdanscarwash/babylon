//! Fixture crate for bsl-lint's I1 ordering-pin test — see
//! `crate-zulu/src/lib.rs`'s doc comment for the full rationale. This crate
//! directory is created AFTER `crate-zulu` on disk but sorts BEFORE it
//! alphabetically; the finding for the shared `"E-FAKE-555"` code must cite
//! this file first.

/// Real emission site — shares `"E-FAKE-555"` with crate-zulu.
pub fn other_classify(x: i32) -> &'static str {
    match x {
        0 => "E-FAKE-555",
        _ => "E-FAKE-002",
    }
}
