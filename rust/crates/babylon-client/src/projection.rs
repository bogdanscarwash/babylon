//! The admin/player seam (B3 wave-1 Task 3, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.6):
//! every panel and lens computation reads a graph field through
//! [`Projector::read`] instead of calling `graph.node_attribute` directly,
//! so a later fog filter (the `apply_fog` port, gated on #593) is a
//! projector swap at this ONE call site, not a rewrite across the eleven
//! places that used to call `node_attribute` on their own
//! (`b3-charter/data-seam.md` §1 named the defect: "there is no seam to
//! insert a fog filter at; the raw read and the render are the same call
//! site").
//!
//! **Four provenances, not two.** Collapsing "the engine wrote this" and
//! "the engine never wrote this" into a single `Option<f64>` is the shape
//! every call site already had. What was missing is the THIRD class III.11
//! actually requires: a field a rule pack DECLARED it will never compute
//! (a structural zero, a discriminant that never reaches the wire) is not
//! an honest absence — reading it back as `Absent` (implying it might show
//! up later) would understate the claim, and reading it back as its
//! literal stored value (often a bare `0.0`) would be the exact fabrication
//! III.11 forbids. [`Provenance::NotComputed`] is that third class. The
//! fourth, [`Provenance::Redacted`], is DECLARED DEAD until #593 (I9) — no
//! player exists yet for a fog filter to protect, so nothing in this crate
//! constructs it outside this file; `tests/projection.rs`'s
//! `redacted_is_declared_dead_until_593` is the sentinel, exempting only
//! this file, the same whole-file-exemption shape
//! `tests/unit/render/test_rust_theme_parity.py` already uses for
//! `palette.rs`'s own `Color::srgb_u8` literals.

use babylon_graph::substrate::{GraphSubstrate, NodeId};

/// The four provenances a [`Reading`] can carry (§2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// The engine computed this field and wrote it — render the number.
    Material,
    /// The engine never wrote this field on this node. Render `reason`,
    /// never a fabricated `0.0` — the field may carry real data on a
    /// LATER tick, or on a different node; this says nothing about that,
    /// only that this read, right now, found nothing.
    Absent {
        /// Why nothing was found — never a numeral (III.11).
        reason: &'static str,
    },
    /// The port DECLARED it will never compute this field at all (a bare
    /// structural zero, a discriminant that never reaches the wire, or a
    /// latch before its companion flag flips) — a STRONGER claim than
    /// `Absent`: this is never coming, by design, not merely "not yet."
    /// Render `reason`, never the literal value the field happens to hold.
    NotComputed {
        /// Why this port declares the field uncomputed — never a numeral.
        reason: &'static str,
    },
    /// Declared-dead until #593 (I9). ADR182 R2 rules redaction-with-remedy
    /// a genuinely different fact than absence (a fog filter withholding a
    /// value that DOES exist), so the variant stays for the day a player
    /// view needs it — but wave 1 has no player and no fog filter, so
    /// nothing constructs this today. Its match arm exists only because
    /// the compiler requires exhaustiveness.
    Redacted {
        /// What would resolve the redaction (e.g. a capability the player
        /// could acquire) — never a numeral. Unused until #593.
        remedy: &'static str,
    },
}

/// One field read through the seam: the raw value (present only for
/// `Material`) and its provenance. `value` is `None` for every
/// non-`Material` provenance — `NotComputed`/`Absent`/`Redacted` never
/// smuggle a numeral through `value` either (§2.6: "never a numeral, never
/// a dash that could read as a value").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    /// `Some(_)` under `Material`, `None` under every other provenance.
    pub value: Option<f64>,
    /// How `value` was arrived at.
    pub provenance: Provenance,
}

impl Reading {
    /// Renders this reading for a text panel at `precision` decimal
    /// places. `Material` renders the raw numeral; every other provenance
    /// renders its declared reason and contains **no digit** — the render
    /// is the one place `Provenance`'s discipline becomes visible on
    /// screen (III.11), and `tests/projection.rs` asserts the digit-free
    /// property directly on this method's output.
    ///
    /// # Panics
    /// If `provenance` is `Material` but `value` is `None` — cannot happen
    /// through [`Projector::read`], the only constructor this crate uses.
    #[must_use]
    pub fn render(&self, precision: usize) -> String {
        match self.provenance {
            Provenance::Material => {
                let value = self
                    .value
                    .expect("Material provenance always carries a value");
                format!("{value:.precision$}")
            }
            Provenance::Absent { reason } => format!("absent \u{2014} {reason}"),
            Provenance::NotComputed { reason } => {
                format!("not computed by this port \u{2014} {reason}")
            }
            Provenance::Redacted { remedy } => format!("redacted \u{2014} {remedy}"),
        }
    }
}

/// One field this projector renders through [`Provenance::NotComputed`]
/// unconditionally — checked BEFORE any graph read, so a field a port
/// structurally never computes renders its reason even where
/// `graph.node_attribute` would itself resolve `Absent` (the field is
/// genuinely never written by any rule in the two wave-1 packs that
/// declare it uncomputed).
#[derive(Debug, Clone, Copy)]
struct NotComputedField {
    /// The declared key, matching §2.6's I2 table's own `Key` column
    /// spelling (`<EVENT_TYPE>.<payload-key>` for a payload-only field that
    /// never becomes a graph attribute at all — these three rows are
    /// exactly that shape; no rule in `decomposition.bsl`/`vitality.bsl`
    /// ever calls `update-node` for them).
    field: &'static str,
    /// Why this port declares the field uncomputed — authored digit-free
    /// on purpose (`Reading::render`'s own contract).
    reason: &'static str,
}

/// §2.6's I2 table, the three members [`Projector::material`] can decide
/// UNCONDITIONALLY (independent of any node's live state) —
/// `decomposition.bsl:264-265` (`SUPERWAGE_CRISIS`'s two payload-only
/// amounts) and `vitality.bsl:90-95` (`ENTITY_DEATH`'s cause, which the
/// pack's own comment records as re-derivable but not carried). The
/// table's fourth row — an `institution/*-tick` latch before its companion
/// flag flips — is CONDITIONAL on that flag's live value; it is the
/// countdown pane's own concern (plan §2.4/§3.3, a later task) and is not
/// reproduced here.
const MATERIAL_NOT_COMPUTED: &[NotComputedField] = &[
    NotComputedField {
        field: "SUPERWAGE_CRISIS.desired-wages",
        reason: "a bare structural zero \u{2014} this port's real dollar figures do not \
                 compute here",
    },
    NotComputedField {
        field: "SUPERWAGE_CRISIS.available-pool",
        reason: "a bare structural zero \u{2014} this port's real dollar figures do not \
                 compute here",
    },
    NotComputedField {
        field: "ENTITY_DEATH.cause",
        reason: "not on the wire at all \u{2014} the discriminant is re-derivable, not carried",
    },
];

/// The seam every panel and lens computation reads a graph field through
/// (§2.6). [`Projector::material`] is wave 1's only mode: an unfogged
/// projection over material truth, declared on screen by the admin banner
/// (`crate::ui::admin::BANNER_TEXT`) — legitimate because a no-player
/// observatory has no epistemic state to protect
/// (`b3-charter/data-seam.md` §1). A later player-facing mode (the
/// `apply_fog` port, gated on #593) is a second constructor over the same
/// `read` call site, not a rewrite of the eleven places this module
/// replaces.
#[derive(Debug, Clone, Copy)]
pub struct Projector {
    not_computed: &'static [NotComputedField],
}

impl Projector {
    /// Wave 1's one projector: material truth, no fog. Constructs
    /// `Material`/`Absent`/`NotComputed` only — never `Redacted` (I9).
    #[must_use]
    pub fn material() -> Self {
        Self {
            not_computed: MATERIAL_NOT_COMPUTED,
        }
    }

    /// Reads `field` off `id` through the seam. Checks the declared
    /// non-computed table first (a field this port structurally never
    /// computes renders `NotComputed` even where `graph.node_attribute`
    /// would itself resolve `Absent`); otherwise defers to the graph
    /// directly — `Ok` is `Material`, `Err` is `Absent`. Never a fabricated
    /// `Some(0.0)`: an unwritten field always carries `value: None`.
    #[must_use]
    pub fn read(&self, graph: &dyn GraphSubstrate, id: NodeId, field: &str) -> Reading {
        if let Some(nc) = self.not_computed.iter().find(|f| f.field == field) {
            return Reading {
                value: None,
                provenance: Provenance::NotComputed { reason: nc.reason },
            };
        }
        match graph.node_attribute(id, field) {
            Ok(value) => Reading {
                value: Some(value),
                provenance: Provenance::Material,
            },
            Err(_) => Reading {
                value: None,
                provenance: Provenance::Absent {
                    reason: "the engine never wrote this field",
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_graph::hypergraph_store::HypergraphStore;

    #[test]
    fn material_read_returns_material_provenance_with_the_written_value() {
        let mut graph = HypergraphStore::new();
        let id = graph.add_node("TERRITORY").expect("add territory");
        graph
            .update_node(id, "territory/pop-d", 2042.0)
            .expect("stamp pop-d");

        let reading = Projector::material().read(&graph, id, "territory/pop-d");
        assert_eq!(reading.value, Some(2042.0));
        assert_eq!(reading.provenance, Provenance::Material);
    }

    #[test]
    fn an_unwritten_field_returns_absent_never_a_fabricated_zero() {
        let mut graph = HypergraphStore::new();
        let id = graph.add_node("TERRITORY").expect("add territory");
        // Nothing ever stamps territory/pop-p on this node.

        let reading = Projector::material().read(&graph, id, "territory/pop-p");
        assert_eq!(
            reading.value, None,
            "an unwritten field must read back as None, never Some(0.0)"
        );
        assert!(matches!(reading.provenance, Provenance::Absent { .. }));
    }

    #[test]
    fn a_declared_not_computed_key_renders_its_reason_with_no_digit() {
        let graph = HypergraphStore::new();
        // `SUPERWAGE_CRISIS.desired-wages` is unconditionally declared —
        // no node needs to exist for the not-computed table to intercept
        // it, proving the check runs BEFORE any graph read.
        let reading =
            Projector::material().read(&graph, NodeId(0), "SUPERWAGE_CRISIS.desired-wages");
        assert_eq!(reading.value, None);
        match reading.provenance {
            Provenance::NotComputed { reason } => {
                assert!(
                    !reason.chars().any(|c| c.is_ascii_digit()),
                    "the declared reason must contain no digit, got {reason:?}"
                );
            }
            other => panic!("expected NotComputed, got {other:?}"),
        }

        let rendered = reading.render(0);
        assert!(
            !rendered.chars().any(|c| c.is_ascii_digit()),
            "the rendered string must contain no digit (III.11), got {rendered:?}"
        );
        assert!(rendered.contains("not computed by this port"));
    }

    #[test]
    fn material_render_shows_the_numeral_at_the_requested_precision() {
        let reading = Reading {
            value: Some(2042.4),
            provenance: Provenance::Material,
        };
        assert_eq!(reading.render(0), "2042");
        assert_eq!(reading.render(1), "2042.4");
    }
}
