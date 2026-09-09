//! The content-set manifest reader (`content-sets.toml`, issue #652 Task 4,
//! plan §4) — schema-versioned TOML, one `[[set]]` row per co-loading unit,
//! and the schema-2 permanent `[[kernel_slot]]` ledger.
//! **Named `content_manifest`, not `manifest`**, to stay clear of
//! `babylon_bsl::manifest`, which is a completely different thing: the
//! IN-LANGUAGE `(manifest …)` scenario form (`bsl-language.rst` §2.9).
//! Task 4 built the file; this module is its first Rust reader (§5.1: the
//! manifest is build metadata `bsl-ls` needs to resolve a `.bsl` file to
//! its content set, never content itself).
//!
//! **Read-only, observes-only (global constraint 1).** This module never
//! writes `content-sets.toml`; it is the DECLARATION (§4.3's own direction
//! note), and `bsl-ls` is one more reader alongside the Rust `include_str!`
//! call sites and the Python sync guard
//! (`tests/unit/reference/test_content_set_manifest_sync.py`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use babylon_tick::kernel_slot::KernelSlotLedgerErrorV1 as KernelSlotLedgerError;
use babylon_tick::kernel_slot::{
    match_kernel_slot_reservation_v1, validate_kernel_slot_ledger_v1, KernelSlotReservationV1,
};
use serde::Deserialize;

pub(crate) use babylon_tick::kernel_slot::KernelSlotReservationMatchV1 as KernelSlotReservationMatch;

/// The sole accepted `content-sets.toml` schema.
pub const CONTENT_SET_MANIFEST_SCHEMA_V2: u32 = 2;

/// The whole manifest: `content-sets.toml`'s top-level shape (plan §4.1).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentSetManifest {
    /// The manifest format version. Only schema 2 is accepted.
    pub schema: u32,
    /// Permanent append-only finite-kernel slot reservations.
    #[serde(rename = "kernel_slot", default)]
    pub kernel_slots: Vec<KernelSlotReservation>,
    /// One row per co-loading unit — `set` in the TOML, `sets` here (the
    /// plural Rust callers actually want).
    #[serde(rename = "set", default)]
    pub sets: Vec<ContentSet>,
    /// `.bsl`/`.bscn` paths that belong to no set yet, each with a reason
    /// (§4.1's own `[orphans]` table) — keyed by content-root-relative path.
    #[serde(default)]
    pub orphans: HashMap<String, String>,
}

/// One permanent binding from a rule-local draw slot to a sample identity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelSlotReservation {
    /// Continuous global append-only ledger position.
    pub ordinal: u32,
    /// Governed mechanic rule `QName`.
    pub rule: String,
    /// Stable finite-kernel sample `QName`.
    pub sample: String,
    /// Rule-local append-only literal draw slot.
    pub slot: u32,
}

/// One `[[set]]` row: a scenario (plus optional preludes) and the rule
/// sources that load against it as one unit.
#[derive(Debug, Clone, Deserialize)]
pub struct ContentSet {
    /// The set's id, e.g. `"control-ratio/conformance"`.
    pub id: String,
    /// The scenario path, relative to the manifest file's own directory
    /// (§4.1).
    pub scenario: String,
    /// Prelude paths, in load order, same relativity as `scenario`.
    #[serde(default)]
    pub prelude: Vec<String>,
    /// Rule-source paths — a SET, not a load order (§4.1's own comment: the
    /// loader sorts rule ids into ascending byte order before firing).
    pub rules: Vec<String>,
    /// Repo-relative paths of the Rust (or other) consumers that
    /// `include_str!` this row's files (D146).
    #[serde(default)]
    pub consumers: Vec<String>,
    /// Required by the sync guard (§4.3 row 3) when `consumers` is empty;
    /// not otherwise interpreted here.
    #[serde(default)]
    pub note: Option<String>,
}

/// A `content-sets.toml` read/parse failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentManifestError {
    /// The file could not be read from disk.
    Io {
        /// The path that failed.
        path: PathBuf,
        /// The underlying I/O error, rendered (never carried as a raw
        /// `io::Error` — this type stays `PartialEq`-free-of-surprises and
        /// easy to assert on in tests).
        detail: String,
    },
    /// The file's contents are not valid TOML, or not this shape.
    Malformed {
        /// The path that failed.
        path: PathBuf,
        /// What `toml`'s own parser reported.
        detail: String,
    },
    /// The manifest selected an obsolete or unknown schema.
    UnsupportedSchema {
        /// The path that declared the unsupported schema.
        path: PathBuf,
        /// The only schema this reader accepts.
        expected: u32,
        /// The schema found in the manifest.
        actual: u32,
    },
    /// The permanent kernel-slot ledger violated its append-only shape.
    KernelSlotLedger {
        /// The manifest path that owns the invalid ledger.
        path: PathBuf,
        /// The first deterministic structural refusal.
        error: KernelSlotLedgerError,
    },
}

impl std::fmt::Display for ContentManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, detail } => {
                write!(f, "reading {}: {detail}", path.display())
            }
            Self::Malformed { path, detail } => {
                write!(
                    f,
                    "parsing {} as content-sets.toml: {detail}",
                    path.display()
                )
            }
            Self::UnsupportedSchema {
                path,
                expected,
                actual,
            } => write!(
                f,
                "parsing {} as content-sets.toml: unsupported schema {actual}; expected {expected}",
                path.display()
            ),
            Self::KernelSlotLedger { path, error } => write!(
                f,
                "parsing {} as content-sets.toml: {error}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ContentManifestError {}

impl ContentSetManifest {
    /// Parse an already-read manifest source. The pure half of
    /// [`Self::load`] — the half a unit test exercises without touching a
    /// filesystem.
    ///
    /// # Errors
    ///
    /// [`ContentManifestError::Malformed`] when `text` is not valid TOML,
    /// [`ContentManifestError::UnsupportedSchema`] when it is not schema 2,
    /// or [`ContentManifestError::KernelSlotLedger`] when its permanent
    /// reservation history is not canonical.
    pub fn parse(path: &Path, text: &str) -> Result<Self, ContentManifestError> {
        let manifest: Self = toml::from_str(text).map_err(|e| ContentManifestError::Malformed {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;
        manifest.validate(path)?;
        Ok(manifest)
    }

    /// Read and parse `path` (`content-sets.toml`'s own location).
    ///
    /// # Errors
    ///
    /// [`ContentManifestError::Io`] on a read failure,
    /// [`ContentManifestError::Malformed`] on a TOML parse failure,
    /// [`ContentManifestError::UnsupportedSchema`] for any schema except 2,
    /// or [`ContentManifestError::KernelSlotLedger`] for a noncanonical
    /// permanent reservation history.
    pub fn load(path: &Path) -> Result<Self, ContentManifestError> {
        let text = std::fs::read_to_string(path).map_err(|e| ContentManifestError::Io {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;
        Self::parse(path, &text)
    }

    /// Every `[[set]]` row whose `scenario`, any `prelude` entry, or any
    /// `rules` entry equals `content_relative_path` (a path relative to the
    /// manifest's own directory — the content root, §4.1). A `.bscn` used
    /// as a prelude by several sets, or a `.bsl` shared by several sets
    /// (`carceral-arc-conformance.bscn`'s two rule packs, §4.2), can match
    /// more than one row — bounded by `self.sets.len()` (Power-of-10 rule 2).
    #[must_use]
    pub fn sets_for(&self, content_relative_path: &str) -> Vec<&ContentSet> {
        self.sets
            .iter()
            .filter(|set| {
                set.scenario == content_relative_path
                    || set.prelude.iter().any(|p| p == content_relative_path)
                    || set.rules.iter().any(|r| r == content_relative_path)
            })
            .collect()
    }

    fn validate(&self, path: &Path) -> Result<(), ContentManifestError> {
        if self.schema != CONTENT_SET_MANIFEST_SCHEMA_V2 {
            return Err(ContentManifestError::UnsupportedSchema {
                path: path.to_path_buf(),
                expected: CONTENT_SET_MANIFEST_SCHEMA_V2,
                actual: self.schema,
            });
        }
        self.validate_kernel_slots()
            .map_err(|error| ContentManifestError::KernelSlotLedger {
                path: path.to_path_buf(),
                error,
            })
    }

    fn validate_kernel_slots(&self) -> Result<(), KernelSlotLedgerError> {
        validate_kernel_slot_ledger_v1(&self.borrowed_kernel_slots())
    }

    #[must_use]
    pub(crate) fn match_kernel_slot(
        &self,
        rule: &str,
        sample: &str,
        slot: u32,
    ) -> KernelSlotReservationMatch<'_> {
        match_kernel_slot_reservation_v1(&self.borrowed_kernel_slots(), rule, sample, slot)
    }

    pub(crate) fn borrowed_kernel_slots(&self) -> Vec<KernelSlotReservationV1<'_>> {
        self.kernel_slots
            .iter()
            .map(|reservation| KernelSlotReservationV1 {
                ordinal: reservation.ordinal,
                rule: &reservation.rule,
                sample: &reservation.sample,
                slot: reservation.slot,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ContentManifestError, ContentSetManifest, KernelSlotLedgerError, KernelSlotReservationMatch,
    };
    use babylon_tick::kernel_slot::BUNDLED_KERNEL_SLOT_RESERVATIONS_V1;
    use std::path::Path;

    fn manifest_path() -> &'static Path {
        Path::new("content-sets.toml")
    }

    fn parse(rows: &str) -> Result<ContentSetManifest, ContentManifestError> {
        ContentSetManifest::parse(manifest_path(), &format!("schema = 2\n{rows}"))
    }

    #[test]
    fn schema_two_keeps_a_permanent_reservation_without_a_live_kernel() {
        let manifest = parse(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
"#,
        )
        .expect("schema-two manifest");
        assert_eq!(manifest.schema, 2);
        assert_eq!(manifest.kernel_slots.len(), 1);
        assert_eq!(manifest.kernel_slots[0].ordinal, 0);
        assert_eq!(manifest.kernel_slots[0].rule, "struggle/spark-mechanic");
        assert_eq!(manifest.kernel_slots[0].sample, "struggle/spark");
        assert_eq!(manifest.kernel_slots[0].slot, 0);
    }

    #[test]
    fn checked_in_manifest_matches_the_runtime_owned_permanent_ledger() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../babylon-tick/content/content-sets.toml");
        let manifest = ContentSetManifest::load(&path).expect("checked-in content manifest");
        let observed = manifest.borrowed_kernel_slots();

        assert_eq!(observed, BUNDLED_KERNEL_SLOT_RESERVATIONS_V1);
    }

    #[test]
    fn schema_one_has_no_fallback() {
        let error = ContentSetManifest::parse(manifest_path(), "schema = 1")
            .expect_err("schema one must be obsolete");
        assert!(matches!(
            error,
            ContentManifestError::UnsupportedSchema {
                expected: 2,
                actual: 1,
                ..
            }
        ));
    }

    #[test]
    fn noncanonical_top_level_kernel_ledger_key_is_manifest_owned() {
        let error = ContentSetManifest::parse(
            manifest_path(),
            r#"
schema = 2
[[kernel_slots]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
"#,
        )
        .expect_err("schema two has one canonical kernel_slot key");
        assert!(matches!(
            error,
            ContentManifestError::Malformed { path, .. } if path.as_path() == manifest_path()
        ));
    }

    #[test]
    fn ordinal_and_slot_are_literal_u32_values() {
        for (field, value) in [
            ("ordinal", "-1"),
            ("ordinal", "4294967296"),
            ("slot", "-1"),
            ("slot", "0.0"),
            ("slot", "4294967296"),
        ] {
            let row = format!(
                r#"
[[kernel_slot]]
ordinal = {}
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = {}
"#,
                if field == "ordinal" { value } else { "0" },
                if field == "slot" { value } else { "0" }
            );
            assert!(matches!(
                parse(&row),
                Err(ContentManifestError::Malformed { .. })
            ));
        }
    }

    #[test]
    fn ledger_ordinal_must_be_continuous_and_in_document_order() {
        let reordered = parse(
            r#"
[[kernel_slot]]
ordinal = 1
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
"#,
        )
        .expect_err("first ordinal cannot be one");
        assert!(matches!(
            reordered,
            ContentManifestError::KernelSlotLedger {
                error: KernelSlotLedgerError::Ordinal {
                    position: 0,
                    expected: 0,
                    actual: 1,
                },
                ..
            }
        ));

        let gap = parse(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
[[kernel_slot]]
ordinal = 2
rule = "vitality/next"
sample = "vitality/next"
slot = 0
"#,
        )
        .expect_err("ordinal one cannot be skipped");
        assert!(matches!(
            gap,
            ContentManifestError::KernelSlotLedger {
                error: KernelSlotLedgerError::Ordinal {
                    position: 1,
                    expected: 1,
                    actual: 2,
                },
                ..
            }
        ));
    }

    #[test]
    fn duplicate_binding_is_a_collision_and_changed_sample_is_a_rebind() {
        let collision = parse(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
[[kernel_slot]]
ordinal = 1
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
"#,
        )
        .expect_err("duplicate binding must collide");
        assert!(matches!(
            collision,
            ContentManifestError::KernelSlotLedger {
                error: KernelSlotLedgerError::Collision {
                    first_ordinal: 0,
                    duplicate_ordinal: 1,
                    ..
                },
                ..
            }
        ));

        let rebind = parse(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
[[kernel_slot]]
ordinal = 1
rule = "struggle/spark-mechanic"
sample = "struggle/rebound"
slot = 0
"#,
        )
        .expect_err("permanent binding cannot change sample");
        assert!(matches!(
            rebind,
            ContentManifestError::KernelSlotLedger {
                error: KernelSlotLedgerError::Rebind {
                    first_ordinal: 0,
                    replacement_ordinal: 1,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn rule_and_sample_use_the_canonical_bsl_qname_reader() {
        for (field, row) in [
            (
                "rule",
                r#"
[[kernel_slot]]
ordinal = 0
rule = "not-a-qname"
sample = "struggle/spark"
slot = 0
"#,
            ),
            (
                "sample",
                r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "not-a-qname"
slot = 0
"#,
            ),
        ] {
            let error = parse(row).expect_err("symbol cannot stand in for a QName");
            assert!(matches!(
                error,
                ContentManifestError::KernelSlotLedger {
                    error: KernelSlotLedgerError::InvalidQName {
                        ordinal: 0,
                        field: actual,
                        ..
                    },
                    ..
                } if actual == field
            ));
        }
    }

    #[test]
    fn each_rule_has_its_own_continuous_append_only_slot_sequence() {
        let error = parse(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
[[kernel_slot]]
ordinal = 1
rule = "struggle/spark-mechanic"
sample = "struggle/future"
slot = 2
"#,
        )
        .expect_err("rule-local slot one cannot be skipped");
        assert!(matches!(
            error,
            ContentManifestError::KernelSlotLedger {
                error: KernelSlotLedgerError::RuleSlotSequence {
                    expected: 1,
                    actual: 2,
                    ordinal: 1,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn sample_identity_cannot_move_to_another_rule_or_slot() {
        for replacement in [
            r#"
[[kernel_slot]]
ordinal = 1
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 1
"#,
            r#"
[[kernel_slot]]
ordinal = 1
rule = "vitality/next"
sample = "struggle/spark"
slot = 0
"#,
        ] {
            let error = parse(&format!(
                r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
{replacement}
"#
            ))
            .expect_err("sample identity must remain one-to-one historically");
            assert!(matches!(
                error,
                ContentManifestError::KernelSlotLedger {
                    error: KernelSlotLedgerError::SampleCollision {
                        first_ordinal: 0,
                        replacement_ordinal: 1,
                        ..
                    },
                    ..
                }
            ));
        }
    }

    #[test]
    fn live_kernel_matching_distinguishes_missing_sample_and_slot() {
        let manifest = parse(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
[[kernel_slot]]
ordinal = 1
rule = "struggle/spark-mechanic"
sample = "struggle/future"
slot = 1
"#,
        )
        .expect("valid two-slot history");

        assert_eq!(
            manifest.match_kernel_slot("struggle/spark-mechanic", "struggle/spark", 0),
            KernelSlotReservationMatch::Exact
        );
        assert!(matches!(
            manifest.match_kernel_slot("struggle/spark-mechanic", "struggle/changed", 0),
            KernelSlotReservationMatch::SampleMismatch { .. }
        ));
        assert!(matches!(
            manifest.match_kernel_slot("struggle/spark-mechanic", "struggle/spark", 2),
            KernelSlotReservationMatch::SlotMismatch { .. }
        ));
        assert_eq!(
            manifest.match_kernel_slot("struggle/spark-mechanic", "struggle/missing", 2),
            KernelSlotReservationMatch::Missing
        );
    }
}
