//! `ContentDigest` (spec §7): the canonical `{defines_hash, rules_hash}`
//! pair. This module owns `defines_hash`; `rules_hash` is produced by
//! `babylon-bsl`'s canonical AST serializer (`canonical_ast::rules_hash_of`,
//! Task 12) — the dependency points bsl→kernel, so the kernel declares the
//! shape and the bsl crate supplies the value.
use sha2::{Digest, Sha256};

/// The canonical content fingerprint pair (spec §7). Both halves are
/// MANDATORY (Task 12 closed Task 7's documented interim `Option` state):
/// an empty content set still has a `rules_hash` — the §5.5 hash of zero
/// rules (`SHA-256(0x03 ‖ u32 0)`), never an absent field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDigest {
    /// SHA-256 of the canonical `GameDefines` JSON (the Phase-0 Task-1
    /// layout: sorted keys, `(",", ":")` separators, `ensure_ascii`).
    pub defines_hash: [u8; 32],
    /// SHA-256 of the canonical BSL AST serialization (`bsl-language.rst`
    /// §5.5), over the content set's rule forms sorted by rule id.
    pub rules_hash: [u8; 32],
}

/// SHA-256 over the caller-supplied canonical JSON string.
///
/// The canonical form itself (sorted keys, `(",", ":")` separators,
/// `ensure_ascii`) is the Python side's job
/// (`babylon.config.defines.canonical_defines_hash`, Phase 0 Task 1) —
/// this function trusts its input is already canonical and only does the
/// hashing, so the same canonicalization bug class Phase 0 fixed cannot
/// reappear split across two languages.
#[must_use]
pub fn defines_hash_of(canonical_json: &str) -> [u8; 32] {
    Sha256::digest(canonical_json.as_bytes()).into()
}

/// SHA-256 over arbitrary already-canonical bytes.
///
/// The kernel owns the project's one hashing convention so that content,
/// state and any future digest agree on the algorithm by construction rather
/// than by coincidence. Callers own their canonicalization; this function
/// only hashes (the same split as [`defines_hash_of`], and for the same
/// reason — a canonicalization bug split across two modules is the class
/// Phase 0 already paid to fix once).
#[must_use]
pub fn sha256_of(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::{defines_hash_of, ContentDigest};

    fn hex(bytes: [u8; 32]) -> String {
        use std::fmt::Write;
        bytes.iter().fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
    }

    /// Cross-language conformance vector — the REAL canonical JSON of a
    /// default `GameDefines()` (generated 2026-07-30 by
    /// `canonical_defines_hash`, committed as a fixture) and its Python
    /// hash. The fixture pins `hash(string)`, not `hash(current defines)`,
    /// so it stays valid as defines evolve — it proves the byte contract,
    /// not the current coefficient values. Independent cross-check: this
    /// hash is the exact `defines_hash` stamped in the 2026-07-30
    /// regression baselines.
    #[test]
    fn matches_the_python_canonical_defines_hash() {
        // The repo's end-of-file-fixer hook appends one trailing newline to
        // every committed text file; the canonical string itself has none
        // (json.dumps emits no trailing newline), so it is stripped —
        // explicitly, once — before the byte pin.
        let canonical_json = include_str!("../tests/fixtures/canonical_defines.json")
            .strip_suffix('\n')
            .expect("fixture is committed with exactly one hook-appended trailing newline");
        assert_eq!(canonical_json.len(), 25_184, "fixture length pin");
        assert_eq!(
            hex(defines_hash_of(canonical_json)),
            "4af1178032a2b420376e35c7a98ac2b151ea8fb9ee68636d1560528d5fd927d4"
        );
    }

    /// A small hand-checkable vector (`sha256(b'{"a":1}')`), so a fixture
    /// problem and a hashing problem are distinguishable at a glance.
    #[test]
    fn hashes_a_minimal_canonical_string() {
        assert_eq!(
            hex(defines_hash_of(r#"{"a":1}"#)),
            "015abd7f5cc57a2dd94b7590f04ad8084273905ee33ec5cebeae62276a97f862"
        );
    }

    #[test]
    fn both_halves_are_mandatory() {
        // The empty content set's rules_hash is SHA-256(0x03 ‖ u32 0) —
        // computed here independently of babylon-bsl (kernel must not
        // depend upward) and pinned; babylon-bsl's rules_hash_of(&[])
        // must produce this same value (its own test asserts it).
        use sha2::{Digest, Sha256};
        let empty_rules: [u8; 32] = Sha256::digest([0x03, 0x00, 0x00, 0x00, 0x00]).into();
        let digest = ContentDigest {
            defines_hash: defines_hash_of("{}"),
            rules_hash: empty_rules,
        };
        assert_eq!(
            hex(digest.rules_hash),
            "a665e6b115dd56fd3e0c89be631e6eda8e9666b822e0bd7026bf0822c4bbc68f"
        );
    }
}
