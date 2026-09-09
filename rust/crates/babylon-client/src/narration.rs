//! The narration templates (B3 wave-1 Task 4, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.2/
//! §2.3): [`NARRATION_TABLE`] transcribes one sentence per landed
//! `EventType` — the four carceral beats from the frozen systems' own
//! `narrative_hint` copy (`decomposition.py`/`control_ratio.py`, §2.2 point
//! 4), the four counties-story events from
//! `src/babylon/game/chronicle_adapter.py::_SUMMARY_BUILDERS`'s own
//! wording — plus, for the four critical rows, the transcribed `because:`
//! causal line (§2.3). Every `{slot}` binds to a named wire payload key
//! (never a pydantic field name); a key the payload does not carry renders
//! the literal text `{absent}`; a key `§2.6`'s I2 table declares a
//! structural non-computation renders its reason, never the literal
//! numeral the payload happens to hold. `{subject}` is the one reserved
//! non-wire slot name — it renders the resolved display of the row's own
//! declared `subject_key` (Minor 2), never guessed.
//!
//! **`TERMINAL_DECISION` is payload-conditional, not slot-conditional
//! (§2.2 point 4).** `control_ratio.py:221-232`'s two `narrative_hint`
//! branches (GENOCIDE / REVOLUTION) are two ENTIRELY DIFFERENT sentences,
//! neither containing a `{slot}` — a single `NarrationSpec.template`
//! cannot express a payload-conditional CHOICE of whole sentence, so that
//! one dispatch lives beside the table (`terminal_decision_template`)
//! rather than inside it. An `outcome` value that is neither `0` nor `1`
//! falls through to the generic line, same discipline as an unverified
//! `EventType`.
//!
//! **The drift guard.** `tests/unit/render/test_rust_narration_parity.py`
//! (Task 4.6) parses [`NARRATION_TABLE`] and asserts every template's
//! wire-key slots are keys `babylon.engine.event_builders.EVENT_BUILDERS`
//! (or, for the handful of slots BSL's payload-flattening/EVENT_BUILDERS'
//! own incompleteness makes unverifiable that way, the frozen system's own
//! raw `payload={...}` dict literal — a documented, cited, narrow
//! exemption, never a silent gap) actually reads for that `EventType`.
//!
//! RED→GREEN record: authored RED when none of `NarrationSpec`,
//! `NARRATION_TABLE`, `spec_for`, `render` existed (the test module failed
//! to compile, mirroring the `d4f353d9` "module absent" RED-commit
//! precedent); GREEN since this module landed — the table below is the
//! production implementation the parity guard above measures.

use babylon_bsl::evaluator::Value;

/// One declared row: the template + optional causal `because:` line for
/// one landed `EventType`, plus its own provenance (§2.2's `NarrationSpec`
/// shape). Every field is `&'static str`/`Option<&'static str>` — the
/// whole table is compile-time data, not a runtime construction.
#[derive(Debug, Clone, Copy)]
pub struct NarrationSpec {
    pub event_type: &'static str,
    /// The declared wire key naming the one `NodeRef` this beat is about
    /// (Minor 2 — never first-match-wins); `None` for a pure-aggregate
    /// payload, rendered at world scope (Minor 3).
    pub subject_key: Option<&'static str>,
    pub template: &'static str,
    pub because: Option<&'static str>,
    /// The frozen file, its line range, and the freeze tag — survives the
    /// Python engine deletion ceremony as the sole remaining provenance
    /// record (I7).
    pub source: &'static str,
}

/// The transcribed narration table (§2.2/§2.3) — one row per landed
/// `EventType` this wave's two shipped stories actually emit. One row per
/// line (the parity guard's own regex contract).
pub const NARRATION_TABLE: &[NarrationSpec] = &[
    NarrationSpec {
        event_type: "SUPERWAGE_CRISIS",
        subject_key: Some("receiver"),
        template: "SUPERWAGE CRISIS: Labor aristocracy wealth collapsing. Super-wages cannot sustain the privileged stratum.",
        because: Some("the labor aristocracy's wealth clears the approaching, not dying, gate \u{2014} super-wages can no longer sustain the privileged stratum"),
        source: "src/babylon/engine/systems/decomposition.py:189-192 @ p27-python-freeze",
    },
    NarrationSpec {
        event_type: "CLASS_DECOMPOSITION",
        subject_key: Some("source-class"),
        template: "CLASS DECOMPOSITION: Labor aristocracy collapses. {population-transferred-to-enforcer} become guards/cops. {population-transferred-to-proletariat} fall into the precariat.",
        because: Some("triggered by superwage_crisis, {decomposition-delay} ticks earlier (carceral/decomposition-delay, carceral-arc-conformance.bscn:17,137)"),
        source: "src/babylon/engine/systems/decomposition.py:361-365 @ p27-python-freeze",
    },
    NarrationSpec {
        event_type: "CONTROL_RATIO_CRISIS",
        subject_key: None,
        template: "CONTROL RATIO CRISIS: {prisoner-population} prisoners exceed {max-controllable} control capacity (1:{control-capacity} ratio). The carceral state cannot contain the surplus.",
        because: Some("triggered by class_decomposition, {control-ratio-delay} ticks earlier (carceral/control-ratio-delay, carceral-arc-conformance.bscn:18,138) \u{2014} the prisoners exceed what the enforcers can hold"),
        source: "src/babylon/engine/systems/control_ratio.py:201-205 @ p27-python-freeze",
    },
    NarrationSpec {
        event_type: "TERMINAL_DECISION",
        subject_key: None,
        template: TERMINAL_DECISION_GENOCIDE,
        because: Some("the atomized surplus population cannot resist \u{2014} average organization {avg-organization} falls short of the revolution threshold {revolution-threshold}"),
        source: "src/babylon/engine/systems/control_ratio.py:228-232 @ p27-python-freeze",
    },
    NarrationSpec {
        event_type: "LIFECYCLE_TRANSITION",
        subject_key: Some("territory-id"),
        template: "{subject} lifecycle: D={pop-d} P={pop-p} D'={pop-d-prime} (dependency ratio {dependency-ratio})",
        because: None,
        source: "src/babylon/game/chronicle_adapter.py:409-413 @ p27-python-freeze",
    },
    NarrationSpec {
        event_type: "LEGITIMATION_CRISIS",
        subject_key: Some("territory-id"),
        template: "{subject} legitimation crisis (index {legitimation-index})",
        because: None,
        source: "src/babylon/game/chronicle_adapter.py:414-417 @ p27-python-freeze",
    },
    NarrationSpec {
        event_type: "LEGITIMATION_RECOVERY",
        subject_key: Some("territory-id"),
        template: "{subject} legitimation recovers (index {legitimation-index})",
        because: None,
        source: "src/babylon/game/chronicle_adapter.py:418-421 @ p27-python-freeze",
    },
    // task-4-review.md Minor 6: this template deliberately drops the frozen
    // copy's `({cause})` parenthetical — `cause` is not on the wire at all
    // (§2.6's I2 table, `NOT_COMPUTED_PAYLOAD_KEYS` below), so a template
    // slot for it would render the honest `{absent}`/not-computed text
    // where the frozen mirror renders a real word ("starvation"/
    // "wealth_threshold"). Silence, not a fabricated placeholder, is the
    // §2.6-conformant choice.
    NarrationSpec {
        event_type: "ENTITY_DEATH",
        subject_key: Some("entity-id"),
        template: "{subject} dies: wealth {wealth} < needs {consumption-needs}",
        because: None,
        source: "src/babylon/game/chronicle_adapter.py:447-450 @ p27-python-freeze",
    },
];

/// `TERMINAL_DECISION`'s two whole-sentence variants — see the module doc.
/// Neither contains a `{slot}`.
const TERMINAL_DECISION_GENOCIDE: &str = "GENOCIDE: Atomized surplus population cannot resist. \
     The system eliminates what it cannot exploit or control.";
const TERMINAL_DECISION_REVOLUTION: &str = "REVOLUTION: Organized prisoners and radicalized \
     guards unite. The carceral apparatus turns against capital.";

/// Looks up `event_type`'s declared row, if any.
#[must_use]
pub fn spec_for(event_type: &str) -> Option<&'static NarrationSpec> {
    NARRATION_TABLE
        .iter()
        .find(|spec| spec.event_type == event_type)
}

/// One payload-only key §2.6/I2 declares a structural non-computation —
/// the SAME two `SUPERWAGE_CRISIS` rows `projection.rs::MATERIAL_NOT_COMPUTED`
/// declares (a bare structural zero, `decomposition.bsl:264-265`), mirrored
/// here because narration slots resolve against the EVENT PAYLOAD, not a
/// graph field, so they need their own declared table even though the
/// underlying fact is the same. `ENTITY_DEATH.cause` is declared too for
/// the same parity, though no shipped template names it (§2.6: "the
/// narration templates for those slots carry no `{slot}` at all").
struct NotComputedPayloadKey {
    event_type: &'static str,
    key: &'static str,
    reason: &'static str,
}

const NOT_COMPUTED_PAYLOAD_KEYS: &[NotComputedPayloadKey] = &[
    NotComputedPayloadKey {
        event_type: "SUPERWAGE_CRISIS",
        key: "desired-wages",
        reason:
            "a bare structural zero \u{2014} this port's real dollar figures do not compute here",
    },
    NotComputedPayloadKey {
        event_type: "SUPERWAGE_CRISIS",
        key: "available-pool",
        reason:
            "a bare structural zero \u{2014} this port's real dollar figures do not compute here",
    },
    NotComputedPayloadKey {
        event_type: "ENTITY_DEATH",
        key: "cause",
        reason: "not on the wire at all \u{2014} the discriminant is re-derivable, not carried",
    },
];

fn not_computed_reason(event_type: &str, key: &str) -> Option<&'static str> {
    NOT_COMPUTED_PAYLOAD_KEYS
        .iter()
        .find(|row| row.event_type == event_type && row.key == key)
        .map(|row| row.reason)
}

/// **C1 (review round 1) — the two rows whose Python source interpolates a
/// bare `int` with no format spec.** `decomposition.py:361-365`'s
/// `enforcer_pop_gain`/`proletariat_pop` and `control_ratio.py:201-205`'s
/// `prisoner_pop`/`max_controllable`/`control_capacity` are genuine Python
/// `int`s, rendered with no decimal point. On the RUST wire these slots are
/// still `Value::Real` — every `field-of` read returns `Real` regardless of
/// the field's declared logical type (`babylon-bsl/src/tick.rs::
/// bind_field_value`; proved live at `decomposition_conformance.rs:745-756`,
/// `Value::Real(150.0)`) — so rendering them through the general `:.2f`-style
/// path (correct for the OTHER five rows' genuinely continuous floats, e.g.
/// `pop-d`/`legitimation-index`) produced "90.00 become guards/cops." in
/// production against the frozen mirror's own bare "90". This table names
/// exactly the slots that need the bare-integer rendering instead — a
/// per-slot declaration, not a global `Real` rule, because a global
/// `r.fract() == 0.0` rule would ALSO strip the `.2f` decimals off a
/// coincidentally-whole `pop-d`/`legitimation-index` value on some future
/// tick, silently breaking the five rows this fix must not touch.
struct IntegralSlot {
    event_type: &'static str,
    key: &'static str,
}

const INTEGRAL_SLOTS: &[IntegralSlot] = &[
    IntegralSlot {
        event_type: "CLASS_DECOMPOSITION",
        key: "population-transferred-to-enforcer",
    },
    IntegralSlot {
        event_type: "CLASS_DECOMPOSITION",
        key: "population-transferred-to-proletariat",
    },
    IntegralSlot {
        event_type: "CONTROL_RATIO_CRISIS",
        key: "prisoner-population",
    },
    IntegralSlot {
        event_type: "CONTROL_RATIO_CRISIS",
        key: "max-controllable",
    },
    IntegralSlot {
        event_type: "CONTROL_RATIO_CRISIS",
        key: "control-capacity",
    },
];

fn is_integral_slot(event_type: &str, key: &str) -> bool {
    INTEGRAL_SLOTS
        .iter()
        .any(|s| s.event_type == event_type && s.key == key)
}

/// Renders one `Value` for embedding in a slot — `Int`/`Real` are the only
/// variants any shipped template's payload slots ever carry; the remaining
/// arms are handled exhaustively (Rust's own type-safety requirement) with
/// an honest debug fallback, never a fabricated numeral.
fn format_value(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Real(r) => format!("{r:.2}"),
        Value::Bool(b) => b.to_string(),
        Value::Enum { member, .. } => member.clone(),
        Value::NodeRef(id) => format!("node #{}", id.0),
        Value::Mass(_)
        | Value::Currency(_)
        | Value::Ratio { .. }
        | Value::HyperedgeRef(_)
        | Value::EdgeRef(_) => {
            format!("{value:?}")
        }
    }
}

/// Renders a value known (per [`INTEGRAL_SLOTS`]) to be a whole-number
/// count on the frozen source's own side, dropping `format_value`'s `.2f`
/// tail — `Value::Real` rounds to the nearest whole number (the wire
/// values here are always exact integral floats by construction, `floor()`'d
/// before storage, so `{:.0}` never actually rounds anything away).
fn format_integral(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Real(r) => {
            // task-4-review.md's own deferred minor: "integral by
            // construction" is proven only for CLASS_DECOMPOSITION's two
            // slots (`decomposition.bsl`'s explicit `floor`); uncited for
            // CONTROL_RATIO_CRISIS's three (`control-ratio.bsl` never
            // calls `floor`). A debug-only guard, not a release-behavior
            // change (per the review's own suggested remedy): if this ever
            // fires, `{:.0}` below would otherwise silently round a
            // genuinely fractional value away.
            debug_assert!(
                r.fract().abs() < f64::EPSILON,
                "format_integral called on a non-integral Real {r} \u{2014} the {{:.0}} render \
                 would silently round it"
            );
            format!("{r:.0}")
        }
        other => format_value(other),
    }
}

/// Resolves one `{key}` slot's substitution text (§2.2/§2.6/Minor 2):
/// `{subject}` is the reserved non-wire name for the caller-resolved
/// subject display; a declared `NotComputed` key renders its reason
/// (never the literal numeral the payload happens to hold); a key the
/// payload does not carry renders the literal text `{absent}` — this is
/// also how the two delay slots (I3) render until Task 5's `Story` catalog
/// wires them: `decomposition-delay`/`control-ratio-delay` are never
/// payload keys (they are declared SCENARIO constants, not wire data), so
/// they fall through this same honest-absence path rather than a baked
/// literal; otherwise the payload's own value, formatted (bare integer for
/// a declared [`IntegralSlot`], `.2f` otherwise).
fn render_slot(
    event_type: &str,
    key: &str,
    payload: &[(String, Value)],
    subject_display: &str,
) -> String {
    if key == "subject" {
        return subject_display.to_owned();
    }
    if let Some(reason) = not_computed_reason(event_type, key) {
        return format!("not computed by this port \u{2014} {reason}");
    }
    match payload.iter().find(|(k, _)| k == key) {
        Some((_, value)) if is_integral_slot(event_type, key) => format_integral(value),
        Some((_, value)) => format_value(value),
        None => "{absent}".to_owned(),
    }
}

/// Substitutes every `{key}` in `template` via [`render_slot`]. UTF-8 safe
/// (operates on `str` slices via `split_once`, never raw byte casts —
/// several templates carry a non-ASCII em dash). Loop bound: `template`'s
/// own byte length — each iteration consumes at least one `{...}` pair, so
/// the loop cannot outlive that many passes (Power-of-10 rule 2).
fn substitute(
    event_type: &str,
    template: &str,
    payload: &[(String, Value)],
    subject_display: &str,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    for _ in 0..template.len() {
        let Some((literal, after_open)) = rest.split_once('{') else {
            result.push_str(rest);
            return result;
        };
        result.push_str(literal);
        let Some((key, tail)) = after_open.split_once('}') else {
            // An unmatched `{` — render it literally rather than dropping it.
            result.push('{');
            result.push_str(after_open);
            return result;
        };
        result.push_str(&render_slot(event_type, key, payload, subject_display));
        rest = tail;
    }
    result
}

/// The two rendered lines a beat card ever carries — `because` is `None`
/// for every row that declares no causal line (never `Some(String::new())`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedBeat {
    pub headline: String,
    pub because: Option<String>,
}

/// `TERMINAL_DECISION`'s payload-conditional template choice (see the
/// module doc) — `None` when `outcome` is missing or neither `0` nor `1`,
/// the same "no verified copy" discipline an unrecognized `EventType`
/// falls through under.
fn terminal_decision_template(payload: &[(String, Value)]) -> Option<&'static str> {
    match payload.iter().find(|(k, _)| k == "outcome").map(|(_, v)| v) {
        Some(Value::Int(1)) => Some(TERMINAL_DECISION_REVOLUTION),
        Some(Value::Int(0)) => Some(TERMINAL_DECISION_GENOCIDE),
        _ => None,
    }
}

/// Renders one beat: `event_type` + its raw payload + the caller-resolved
/// `subject_display` (the declared `subject_key`'s `NodeRef` resolved to a
/// FIPS/entity display string, or `"world"` for a subject-less row — the
/// caller's job, since only it holds the roster this crate's narration
/// layer stays pure without). An unverified `EventType` (or, for
/// `TERMINAL_DECISION`, an unverified `outcome`) falls through to the
/// generic `<EVENT_TYPE> @ <subject>` line — never dropped, never guessed.
#[must_use]
pub fn render(
    event_type: &str,
    payload: &[(String, Value)],
    subject_display: &str,
) -> RenderedBeat {
    let template = if event_type == "TERMINAL_DECISION" {
        terminal_decision_template(payload)
    } else {
        spec_for(event_type).map(|spec| spec.template)
    };
    let Some(template) = template else {
        return RenderedBeat {
            headline: format!("{event_type} @ {subject_display}"),
            because: None,
        };
    };
    let spec = spec_for(event_type);
    RenderedBeat {
        headline: substitute(event_type, template, payload, subject_display),
        because: spec
            .and_then(|s| s.because)
            .map(|because| substitute(event_type, because, payload, subject_display)),
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use babylon_bsl::evaluator::Value;

    /// The frozen mirror's own tick-53 payload
    /// (`carceral_arc_conformance.rs:60`), flattened to the wire keys
    /// `decomposition.bsl:377-386` actually emits, and stamped in the
    /// **production wire shape** (C1, review round 1): every value below
    /// is `Value::Real`, never `Value::Int` — `field-of` reads (which is
    /// how `p06-la-deactivate`'s emit reaches all four transfer amounts,
    /// `decomposition.bsl:366-372`) always return `Real` regardless of the
    /// field's declared logical type
    /// (`babylon-bsl/src/tick.rs::bind_field_value`), proved live at
    /// `decomposition_conformance.rs:745-756` (`Value::Real(150.0)`). A
    /// fixture stamping `Value::Int` here would be exactly the
    /// fixture-shape defect class CLAUDE.md names: a green test over a
    /// payload shape production never emits.
    fn class_decomposition_payload() -> Vec<(String, Value)> {
        vec![
            (
                "source-class".to_owned(),
                Value::NodeRef(babylon_graph::substrate::NodeId(0)),
            ),
            ("source-population".to_owned(), Value::Real(600.0)),
            ("source-wealth".to_owned(), Value::Real(515.0)),
            ("enforcer-fraction".to_owned(), Value::Real(0.15)),
            ("proletariat-fraction".to_owned(), Value::Real(0.85)),
            (
                "population-transferred-to-enforcer".to_owned(),
                Value::Real(90.0),
            ),
            (
                "population-transferred-to-proletariat".to_owned(),
                Value::Real(510.0),
            ),
            (
                "wealth-transferred-to-enforcer".to_owned(),
                Value::Real(77.25),
            ),
            (
                "wealth-transferred-to-proletariat".to_owned(),
                Value::Real(437.75),
            ),
        ]
    }

    /// `control-ratio.bsl:319-328`'s `c03-crisis` emit, production wire
    /// shape: `enforcer-population`/`prisoner-population` are `:field`
    /// reads off the carrier (`Real`, same `bind_field_value` fact as
    /// above); `control-capacity` is a bare `:const` literal (`Int` — a
    /// `:const` binding reads the defconst value directly, never through
    /// `field-of`); `max-controllable` is `:expr (* enforcer-population
    /// control-capacity)`, a `Real \u{d7} Int` product (`Real`).
    fn control_ratio_crisis_payload() -> Vec<(String, Value)> {
        vec![
            ("enforcer-population".to_owned(), Value::Real(110.0)),
            ("prisoner-population".to_owned(), Value::Real(710.0)),
            ("control-capacity".to_owned(), Value::Int(4)),
            ("max-controllable".to_owned(), Value::Real(440.0)),
            (
                "actual-ratio".to_owned(),
                Value::Real(6.454_545_454_545_454),
            ),
            ("over-capacity-by".to_owned(), Value::Real(270.0)),
            (
                "control-ratio".to_owned(),
                Value::Real(6.454_545_454_545_454),
            ),
            ("capacity-threshold".to_owned(), Value::Real(4.0)),
        ]
    }

    /// `decomposition.bsl:262-265`'s `p02-superwage-warning` emit,
    /// production wire shape: `receiver` is the LA node's own `self`
    /// `NodeRef`; `desired-wages`/`available-pool` are the two §2.6/I2
    /// declared `NotComputed` structural zeros (bare `0.0c` literals).
    fn superwage_crisis_payload() -> Vec<(String, Value)> {
        vec![
            (
                "receiver".to_owned(),
                Value::NodeRef(babylon_graph::substrate::NodeId(0)),
            ),
            ("desired-wages".to_owned(), Value::Real(0.0)),
            ("available-pool".to_owned(), Value::Real(0.0)),
        ]
    }

    #[test]
    fn class_decomposition_renders_the_frozen_mirrors_exact_copy() {
        let rendered = render(
            "CLASS_DECOMPOSITION",
            &class_decomposition_payload(),
            "world",
        );
        assert_eq!(
            rendered.headline,
            "CLASS DECOMPOSITION: Labor aristocracy collapses. 90 become guards/cops. \
             510 fall into the precariat."
        );
    }

    /// `control-ratio.bsl:364-378`'s `c04-terminal` emit — `outcome` is a
    /// bare integer literal (`(outcome 1)`/`(outcome 0)`, never a
    /// `field-of` read, so genuinely `Value::Int`, confirmed at
    /// `carceral_arc_conformance.rs:290`); `prisoner-population`/
    /// `enforcer-population` are `:field` reads off the carrier (`Real`).
    fn terminal_decision_payload(outcome: i64) -> Vec<(String, Value)> {
        vec![
            ("outcome".to_owned(), Value::Int(outcome)),
            (
                "avg-organization".to_owned(),
                Value::Real(0.056_338_028_169_014_086),
            ),
            ("revolution-threshold".to_owned(), Value::Real(0.5)),
            ("prisoner-population".to_owned(), Value::Real(710.0)),
            ("enforcer-population".to_owned(), Value::Real(110.0)),
        ]
    }

    #[test]
    fn terminal_decision_outcome_zero_renders_the_genocide_copy() {
        let rendered = render("TERMINAL_DECISION", &terminal_decision_payload(0), "world");
        assert!(
            rendered.headline.contains("GENOCIDE"),
            "outcome 0 must render the GENOCIDE copy, got {:?}",
            rendered.headline
        );
        assert!(!rendered.headline.contains("REVOLUTION"));
    }

    #[test]
    fn terminal_decision_outcome_one_renders_the_revolution_copy() {
        let rendered = render("TERMINAL_DECISION", &terminal_decision_payload(1), "world");
        assert!(
            rendered.headline.contains("REVOLUTION"),
            "outcome 1 must render the REVOLUTION copy, got {:?}",
            rendered.headline
        );
        assert!(!rendered.headline.contains("GENOCIDE"));
    }

    #[test]
    fn a_payload_missing_a_slot_key_renders_the_literal_absent_marker_for_that_slot_only() {
        let mut payload = class_decomposition_payload();
        payload.retain(|(k, _)| k != "population-transferred-to-enforcer");
        let rendered = render("CLASS_DECOMPOSITION", &payload, "world");
        assert_eq!(
            rendered.headline,
            "CLASS DECOMPOSITION: Labor aristocracy collapses. {absent} become guards/cops. \
             510 fall into the precariat.",
            "only the missing slot renders {{absent}} — the rest of the sentence is unchanged"
        );
    }

    /// §2.6/I2's declared `SUPERWAGE_CRISIS.desired-wages` row — even
    /// though no SHIPPED template references this slot (the frozen
    /// `narrative_hint` never mentions the dollar figures either), the
    /// renderer's own `NotComputed` discipline must hold for it directly.
    #[test]
    fn a_declared_not_computed_key_renders_its_reason_and_no_numeral() {
        let payload = vec![
            (
                "receiver".to_owned(),
                Value::NodeRef(babylon_graph::substrate::NodeId(0)),
            ),
            ("desired-wages".to_owned(), Value::Real(0.0)),
            ("available-pool".to_owned(), Value::Real(0.0)),
        ];
        let rendered = super::substitute(
            "SUPERWAGE_CRISIS",
            "wages: {desired-wages}",
            &payload,
            "world",
        );
        assert!(
            !rendered.chars().any(|c| c.is_ascii_digit()),
            "a NotComputed slot must render its reason, never the literal 0.0 the \
             payload happens to hold — got {rendered:?}"
        );
        assert!(rendered.contains("not computed by this port"));
    }

    #[test]
    fn an_unknown_event_type_falls_through_to_the_generic_line() {
        let rendered = render("SOME_UNVERIFIED_EVENT_TYPE", &[], "world");
        assert_eq!(rendered.headline, "SOME_UNVERIFIED_EVENT_TYPE @ world");
        assert!(rendered.because.is_none());
    }

    // ---- §2.3's because: line, the four critical rows ----

    /// `{absent}` is not a "leftover brace" — it is `render_slot`'s own
    /// honest substitution result (used by `decomposition-delay`/
    /// `control-ratio-delay`, I3, until Task 5's `Story` catalog wires
    /// them). Scrubbing every `{absent}` occurrence first means what
    /// survives is a genuine unbound `{slot-name}`, which `substitute`
    /// should never produce for a well-formed template.
    fn assert_no_unbound_slots(rendered: &str) {
        let scrubbed = rendered.replace("{absent}", "");
        assert!(
            !scrubbed.contains('{') && !scrubbed.contains('}'),
            "every slot must resolve to a real value or the honest {{absent}} marker — no \
             other leftover {{brace}} placeholder is allowed, got {rendered:?}"
        );
    }

    /// I4 (review round 1): the predecessor of this test asserted only
    /// `spec.because.is_some()` — a struct-field presence check that never
    /// rendered anything, so a leftover unbound `{brace}` in any of the
    /// three non-`TERMINAL_DECISION` rows would have shipped green. This
    /// version renders all four critical rows against production-shaped
    /// payloads and checks their actual output.
    #[test]
    fn each_critical_row_renders_its_transcribed_because_line_with_slots_bound() {
        let superwage = render("SUPERWAGE_CRISIS", &superwage_crisis_payload(), "world");
        let because = superwage
            .because
            .expect("SUPERWAGE_CRISIS must render a because: line");
        assert_eq!(
            because,
            "the labor aristocracy's wealth clears the approaching, not dying, gate \u{2014} \
             super-wages can no longer sustain the privileged stratum",
            "SUPERWAGE_CRISIS's because: line carries no slots — it must render byte-identical"
        );

        let class_decomposition = render(
            "CLASS_DECOMPOSITION",
            &class_decomposition_payload(),
            "world",
        );
        let because = class_decomposition
            .because
            .expect("CLASS_DECOMPOSITION must render a because: line");
        assert!(
            because.starts_with("triggered by superwage_crisis, "),
            "got {because:?}"
        );
        assert!(
            because.contains("{absent} ticks earlier"),
            "decomposition-delay is not a payload key — I3 renders it through the honest \
             {{absent}} fallback until Task 5's Story catalog wires it, got {because:?}"
        );
        assert_no_unbound_slots(&because);

        let control_ratio_crisis = render(
            "CONTROL_RATIO_CRISIS",
            &control_ratio_crisis_payload(),
            "world",
        );
        let because = control_ratio_crisis
            .because
            .expect("CONTROL_RATIO_CRISIS must render a because: line");
        assert!(
            because.starts_with("triggered by class_decomposition, "),
            "got {because:?}"
        );
        assert!(
            because.contains("{absent} ticks earlier"),
            "control-ratio-delay is not a payload key — I3 renders it through the honest \
             {{absent}} fallback until Task 5's Story catalog wires it, got {because:?}"
        );
        assert_no_unbound_slots(&because);

        let terminal_decision = render("TERMINAL_DECISION", &terminal_decision_payload(0), "world");
        let because = terminal_decision
            .because
            .expect("TERMINAL_DECISION must render a because: line");
        assert!(
            because.contains("0.5"),
            "revolution-threshold must be bound to the real payload value, got {because:?}"
        );
        assert_no_unbound_slots(&because);
    }

    #[test]
    fn terminal_decisions_because_line_binds_its_threshold_slots() {
        let rendered = render("TERMINAL_DECISION", &terminal_decision_payload(0), "world");
        let because = rendered
            .because
            .expect("TERMINAL_DECISION must render a because: line");
        assert!(
            because.contains("0.5"),
            "the because: line must bind {{revolution-threshold}} to the real payload \
             value, got {because:?}"
        );
        assert!(
            !because.contains('{') && !because.contains('}'),
            "every slot in the because: line must have been bound — no leftover \
             {{brace}} placeholders, got {because:?}"
        );
    }

    /// A beat whose row declares no `because:` (every non-critical row)
    /// renders no second line at all — never an empty one.
    #[test]
    fn a_beat_with_no_because_row_renders_no_second_line_at_all() {
        let payload = vec![
            (
                "territory-id".to_owned(),
                Value::NodeRef(babylon_graph::substrate::NodeId(0)),
            ),
            ("legitimation-index".to_owned(), Value::Real(0.62)),
        ];
        let rendered = render("LEGITIMATION_RECOVERY", &payload, "01013");
        assert!(
            rendered.because.is_none(),
            "LEGITIMATION_RECOVERY declares no because: row — rendering must be None, \
             never Some(String::new())"
        );
    }
}
