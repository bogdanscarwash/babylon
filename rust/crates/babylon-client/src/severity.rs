//! The transcribed severity taxonomy (B3 wave-1 Task 4, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.2):
//! `derive_severity` reproduces `src/babylon/models/event_severity.py`'s
//! own pure rule (`:229`) verbatim for the four kind arms this crate's
//! landed content ever exercises (ALARM and PATTERN never appear on the
//! wire here — no landed `.bsl` pack emits an invariant-residual or a
//! distinguished-cell event, so this table omits both rather than
//! declaring dead rows), and [`SEVERITY_TAXONOMY`] transcribes the 12
//! rows `event_severity.py::SEVERITY_TAXONOMY` declares for the
//! `EventType`s this wave's two stories actually emit (§2.2's own table).
//!
//! **The drift guard.** `tests/unit/render/test_rust_narration_parity.py`
//! (Task 4.6) parses [`SEVERITY_TAXONOMY`] and asserts every row's
//! `(kind, proximity)` matches the Python module's own
//! `SEVERITY_TAXONOMY` — a §9b-parity-guard-shaped check, the same
//! FFI-boundary-no-import-can-cross precedent `palette.rs`'s own module
//! doc names for `test_rust_theme_parity.py`.
//!
//! RED→GREEN record: authored RED when none of `EventKind`,
//! `TerminalProximity`, `SeverityTier`, `derive_severity`, `SeverityRow`,
//! `SEVERITY_TAXONOMY`, `severity_for` existed (the test module failed to
//! compile, mirroring the `d4f353d9` "module absent" RED-commit precedent
//! — `ui/time.rs`'s own module doc); GREEN since this module landed — the
//! items below are the production taxonomy the parity guard measures.

/// The R-EC-1 generator-fact kind (`event_severity.py::EventKind`) — this
/// crate declares only the four arms its own landed content can ever
/// produce (see the module doc); `Alarm`/`Pattern` from the Python side's
/// full five-member enum are omitted, not stubbed, since a dead row would
/// be exactly the kind of undead declaration III.11 warns against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// An invariant residual — always critical.
    Alarm,
    /// A chi change at a sited atom — binary severity by `TerminalProximity`.
    Crossing,
    /// A register/ledger row above its declared salience floor.
    Flow,
    /// A verb resolved — tier is its declared salience floor.
    Act,
}

/// How close a `Crossing` sits to a terminal/endgame-axis lock
/// (`event_severity.py::TerminalProximity`). Meaningful only for
/// `EventKind::Crossing` — every other kind passes `Na`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalProximity {
    /// Void-adjacency / regime->crisis entry — derives critical.
    TerminalAdjacent,
    /// Directional movement TOWARD a terminal-adjacent configuration
    /// without entering it — derives warning.
    TerminalApproach,
    /// A reversible crossing that stays within the current qualitative
    /// level — derives informational.
    IntraLevel,
    /// Not applicable — every non-`Crossing` kind passes this.
    Na,
}

/// The three-bucket taxonomy (spec-061 FR-012) — unchanged by this
/// derivation, only how a tier is assigned changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityTier {
    Critical,
    Warning,
    Informational,
}

/// The pure derivation rule (`event_severity.py::derive_severity`, `:229`)
/// — `kind` x `terminal_proximity` -> tier, transcribed verbatim for the
/// four kind arms this crate declares.
///
/// # Errors
/// A loud `Err` — never a silently defaulted tier (III.11) — if `kind` is
/// `Crossing` and `terminal_proximity` is `Na` (the Python `ValueError`
/// arm this mirrors), or if `kind` is `Flow`/`Act` and `salience_floor` is
/// `None` or `Some(SeverityTier::Critical)` (a FLOW/ACT event may never
/// derive critical — `event_severity.py`'s own `_validate_flow_or_act_shape`).
pub fn derive_severity(
    kind: EventKind,
    terminal_proximity: TerminalProximity,
    salience_floor: Option<SeverityTier>,
) -> Result<SeverityTier, String> {
    match kind {
        EventKind::Alarm => Ok(SeverityTier::Critical),
        EventKind::Crossing => match terminal_proximity {
            TerminalProximity::TerminalAdjacent => Ok(SeverityTier::Critical),
            TerminalProximity::TerminalApproach => Ok(SeverityTier::Warning),
            TerminalProximity::IntraLevel => Ok(SeverityTier::Informational),
            TerminalProximity::Na => Err(format!(
                "CROSSING requires a real terminal_proximity, got {terminal_proximity:?}"
            )),
        },
        EventKind::Flow | EventKind::Act => match salience_floor {
            None => Err(format!("{kind:?} requires a declared salience_floor")),
            Some(SeverityTier::Critical) => {
                Err(format!("{kind:?} salience_floor may never be Critical"))
            }
            Some(floor) => Ok(floor),
        },
    }
}

/// One declared row of the transcribed taxonomy — `event_type` is the raw
/// wire string `CollectingSink` carries (never an owned/parsed `EventType`
/// enum — this crate has no import path to the Python enum, and the wire
/// string IS the ground truth here, same as every other event-type
/// comparison in this crate).
#[derive(Debug, Clone, Copy)]
pub struct SeverityRow {
    pub event_type: &'static str,
    pub kind: EventKind,
    pub proximity: TerminalProximity,
    pub salience_floor: Option<SeverityTier>,
}

/// The 12 rows `event_severity.py::SEVERITY_TAXONOMY` declares for the
/// `EventType`s this wave's two shipped stories emit (plan §2.2's own
/// table) — transcribed, not invented; `tests/unit/render/
/// test_rust_narration_parity.py` (Task 4.6) is the drift guard. One row
/// per line (the parity guard's own regex contract, matching
/// `palette.rs`'s "keep each constant on one line" precedent).
pub const SEVERITY_TAXONOMY: &[SeverityRow] = &[
    SeverityRow {
        event_type: "TERMINAL_DECISION",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::TerminalAdjacent,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "CONTROL_RATIO_CRISIS",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::TerminalAdjacent,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "CLASS_DECOMPOSITION",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::TerminalAdjacent,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "SUPERWAGE_CRISIS",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::TerminalAdjacent,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "LEGITIMATION_CRISIS",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::TerminalApproach,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "LEGITIMATION_RECOVERY",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::IntraLevel,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "MASS_AWAKENING",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::IntraLevel,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "ENTITY_DEATH",
        kind: EventKind::Crossing,
        proximity: TerminalProximity::IntraLevel,
        salience_floor: None,
    },
    SeverityRow {
        event_type: "LIFECYCLE_TRANSITION",
        kind: EventKind::Flow,
        proximity: TerminalProximity::Na,
        salience_floor: Some(SeverityTier::Informational),
    },
    SeverityRow {
        event_type: "CONSCIOUSNESS_TRANSMISSION",
        kind: EventKind::Flow,
        proximity: TerminalProximity::Na,
        salience_floor: Some(SeverityTier::Informational),
    },
    SeverityRow {
        event_type: "VALUE_TRANSFER",
        kind: EventKind::Flow,
        proximity: TerminalProximity::Na,
        salience_floor: Some(SeverityTier::Informational),
    },
    SeverityRow {
        event_type: "DISPOSSESSION_EVENT",
        kind: EventKind::Flow,
        proximity: TerminalProximity::Na,
        salience_floor: Some(SeverityTier::Informational),
    },
];

/// Resolves `event_type`'s severity tier — `event_severity.py::
/// resolve_severity`'s own loud floor: an `event_type` with no declared
/// [`SeverityRow`] resolves to `Warning`, never a quiet `Informational`
/// degrade (III.11 — silence about an unclassified event is itself a
/// defect, and `Warning` is the loud way to say so without falsely
/// claiming `Critical`).
///
/// # Panics
/// Cannot panic through any call site in this crate: [`SEVERITY_TAXONOMY`]'s
/// own rows are exercised by `the_twelve_transcribed_rows_resolve_to_the_declared_table`,
/// which would itself fail first if a row's `(kind, proximity, salience_floor)`
/// were ever malformed.
#[must_use]
pub fn severity_for(event_type: &str) -> SeverityTier {
    SEVERITY_TAXONOMY
        .iter()
        .find(|row| row.event_type == event_type)
        .map_or(SeverityTier::Warning, |row| {
            derive_severity(row.kind, row.proximity, row.salience_floor)
                .expect("SEVERITY_TAXONOMY's own rows are well-formed by construction")
        })
}

/// Looks up `event_type`'s declared [`EventKind`], if any — `ui::beats`'
/// own collapse rule keys off `Flow` specifically (§2.2 point 3: "same-tick
/// FLOW events of one type collapse"), which `severity_for`'s resolved
/// TIER alone cannot distinguish from a `Crossing::IntraLevel` row (both
/// resolve `Informational`, but only the FLOW rows collapse).
#[must_use]
pub fn kind_for(event_type: &str) -> Option<EventKind> {
    SEVERITY_TAXONOMY
        .iter()
        .find(|row| row.event_type == event_type)
        .map(|row| row.kind)
}

#[cfg(test)]
mod tests {
    use super::{derive_severity, severity_for, EventKind, SeverityTier, TerminalProximity};

    // ---- derive_severity's four rule arms (event_severity.py:252-274) ----

    #[test]
    fn alarm_always_derives_critical() {
        assert_eq!(
            derive_severity(EventKind::Alarm, TerminalProximity::Na, None),
            Ok(SeverityTier::Critical)
        );
    }

    #[test]
    fn crossing_terminal_adjacent_derives_critical() {
        assert_eq!(
            derive_severity(
                EventKind::Crossing,
                TerminalProximity::TerminalAdjacent,
                None
            ),
            Ok(SeverityTier::Critical)
        );
    }

    #[test]
    fn crossing_terminal_approach_derives_warning() {
        assert_eq!(
            derive_severity(
                EventKind::Crossing,
                TerminalProximity::TerminalApproach,
                None
            ),
            Ok(SeverityTier::Warning)
        );
    }

    #[test]
    fn crossing_intra_level_derives_informational() {
        assert_eq!(
            derive_severity(EventKind::Crossing, TerminalProximity::IntraLevel, None),
            Ok(SeverityTier::Informational)
        );
    }

    #[test]
    fn flow_derives_its_declared_salience_floor_never_critical() {
        assert_eq!(
            derive_severity(
                EventKind::Flow,
                TerminalProximity::Na,
                Some(SeverityTier::Warning)
            ),
            Ok(SeverityTier::Warning)
        );
        assert!(
            derive_severity(
                EventKind::Flow,
                TerminalProximity::Na,
                Some(SeverityTier::Critical)
            )
            .is_err(),
            "a FLOW/ACT floor of Critical is the exact fabrication the taxonomy forbids \
             (event_severity.py's own _validate_flow_or_act_shape)"
        );
    }

    #[test]
    fn act_derives_its_declared_salience_floor_never_critical() {
        assert_eq!(
            derive_severity(
                EventKind::Act,
                TerminalProximity::Na,
                Some(SeverityTier::Informational)
            ),
            Ok(SeverityTier::Informational)
        );
    }

    /// `event_severity.py:229`'s own `ValueError` arm: a CROSSING row with
    /// `terminal_proximity = NA` is a loud error, never a default tier.
    #[test]
    fn crossing_with_na_proximity_is_a_loud_error_not_a_default() {
        let result = derive_severity(EventKind::Crossing, TerminalProximity::Na, None);
        assert!(
            result.is_err(),
            "CROSSING with NA proximity must be a loud Err, never a silently \
             defaulted tier — got {result:?}"
        );
    }

    // ---- SEVERITY_TAXONOMY's 12 transcribed rows resolve to §2.2's table ----

    #[test]
    fn the_twelve_transcribed_rows_resolve_to_the_declared_table() {
        let expected: &[(&str, SeverityTier)] = &[
            ("TERMINAL_DECISION", SeverityTier::Critical),
            ("CONTROL_RATIO_CRISIS", SeverityTier::Critical),
            ("CLASS_DECOMPOSITION", SeverityTier::Critical),
            ("SUPERWAGE_CRISIS", SeverityTier::Critical),
            ("LEGITIMATION_CRISIS", SeverityTier::Warning),
            ("LEGITIMATION_RECOVERY", SeverityTier::Informational),
            ("MASS_AWAKENING", SeverityTier::Informational),
            ("ENTITY_DEATH", SeverityTier::Informational),
            ("LIFECYCLE_TRANSITION", SeverityTier::Informational),
            ("CONSCIOUSNESS_TRANSMISSION", SeverityTier::Informational),
            ("VALUE_TRANSFER", SeverityTier::Informational),
            ("DISPOSSESSION_EVENT", SeverityTier::Informational),
        ];
        assert_eq!(
            expected.len(),
            12,
            "this test's own table must name 12 rows"
        );
        for (event_type, tier) in expected {
            assert_eq!(
                severity_for(event_type),
                *tier,
                "{event_type} must resolve to {tier:?}"
            );
        }
    }

    #[test]
    fn flow_and_act_events_never_resolve_critical() {
        for event_type in [
            "LIFECYCLE_TRANSITION",
            "CONSCIOUSNESS_TRANSMISSION",
            "VALUE_TRANSFER",
            "DISPOSSESSION_EVENT",
        ] {
            assert_ne!(
                severity_for(event_type),
                SeverityTier::Critical,
                "{event_type} is FLOW — it must never resolve critical"
            );
        }
    }

    #[test]
    fn an_unclassified_event_type_resolves_to_the_loud_warning_floor() {
        assert_eq!(
            severity_for("SOME_FUTURE_EVENT_TYPE_NOT_IN_THE_TABLE"),
            SeverityTier::Warning,
            "event_severity.py::resolve_severity's own loud floor: unclassified -> \
             warning, never a quiet informational degrade"
        );
    }
}
