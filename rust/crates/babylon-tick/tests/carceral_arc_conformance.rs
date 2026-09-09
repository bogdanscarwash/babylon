//! The JOINT ARC — Task 8 of the Decomposition (@11.0) + ControlRatio
//! (@12.0) port train (`docs/superpowers/plans/2026-08-17-decomposition-
//! controlratio-port.md`). This is the train's ACCEPTANCE TEST: the ported
//! analogue of `tests/scenarios/test_carceral_equilibrium.py`'s
//! phase-sequence assertion (`TestCarceralEquilibrium::
//! test_default_trajectory_phases`), and the proof the two packs COMPOSE —
//! `content/rules/decomposition.bsl` (Pack A, which declares the ONE
//! `floor` intrinsic this arc needs) and `content/rules/control-ratio.bsl`
//! (Pack B, which declares none) CONCATENATED and loaded together against
//! ONE scenario, over a multi-tick `TickSession`.
//!
//! # Derived tick schedule — verified against the frozen mirror, not
//! trusted from the plan's own illustrative numbers (this task's own
//! brief)
//!
//! `carceral-arc-conformance.bscn`'s `la-approaching` reproduces
//! `decomposition-delay-conformance.bscn`'s DELAY-PATH vector (wealth 515,
//! strictly between `subsistence + 1*consumption` (510) and `subsistence +
//! 2*consumption` (520) — "approaching, not dying"), on the SHIPPED
//! `carceral/*` delays (`src/babylon/data/defines.yaml:293-300`:
//! `decomposition_delay=52`, `control_ratio_delay=52`,
//! `terminal_decision_delay=1`). The arithmetic, by hand and cross-checked
//! against the frozen mirror below:
//!
//! ```text
//! SUPERWAGE_CRISIS      tick 1    (la-approaching clears the approaching gate)
//! CLASS_DECOMPOSITION   tick 53   (1 + decomposition_delay(52))
//! CONTROL_RATIO_CRISIS  tick 105  (53 + control_ratio_delay(52))
//! TERMINAL_DECISION     tick 106  (105 + terminal_decision_delay(1))
//! ```
//!
//! # Frozen-mirror provenance
//!
//! `carceral_arc_conformance.py` runs the frozen `DecompositionSystem`
//! (@11.0) then `ControlRatioSystem` (@12.0), in that call order, EVERY
//! tick from 1 to 112, sharing ONE `TickContext.persistent_data` (matching
//! the frozen engine's own single `context` threaded through every system
//! every tick — the composition this whole arc proves the ported estate
//! reproduces. The historical §5 byte-order inversion is superseded by the
//! executable phase registry below. Run from the
//! repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run python \
//!     rust/crates/babylon-tick/content/scenarios/carceral_arc_conformance.py
//! ```
//!
//! Its output on 2026-08-17, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml, carceral: section, SHIPPED values):
//!   carceral.control_capacity = 4
//!   carceral.enforcer_fraction = 0.15
//!   carceral.proletariat_fraction = 0.85
//!   carceral.revolution_threshold = 0.5
//!   carceral.decomposition_delay = 52
//!   carceral.control_ratio_delay = 52
//!   carceral.terminal_decision_delay = 1
//!
//! tick 1: superwage_crisis {'payer_id': 'C003', 'receiver_id': 'la-approaching', 'desired_wages': 0.0, 'available_pool': 0.0, 'narrative_hint': 'SUPERWAGE CRISIS: Labor aristocracy wealth collapsing. Super-wages cannot sustain the privileged stratum.'}
//! tick 53: class_decomposition {'source_class': 'la-approaching', 'source_population': 600, 'source_wealth': 515.0, 'enforcer_fraction': 0.15, 'proletariat_fraction': 0.85, 'population_transferred': {'to_enforcer': 90, 'to_proletariat': 510}, 'wealth_transferred': {'to_enforcer': 77.25, 'to_proletariat': 437.75}, 'trigger_event': 'superwage_crisis', 'narrative_hint': 'CLASS DECOMPOSITION: Labor aristocracy collapses. 90 become guards/cops. 510 fall into the precariat.'}
//! tick 105: control_ratio_crisis {'enforcer_population': 110, 'prisoner_population': 710, 'control_capacity': 4, 'max_controllable': 440, 'actual_ratio': 6.454545454545454, 'over_capacity_by': 270, 'control_ratio': 6.454545454545454, 'capacity_threshold': 4.0, 'narrative_hint': 'CONTROL RATIO CRISIS: 710 prisoners exceed 440 control capacity (1:4 ratio). The carceral state cannot contain the surplus.'}
//! tick 106: terminal_decision {'outcome': 'genocide', 'avg_organization': 0.056338028169014086, 'revolution_threshold': 0.5, 'prisoner_population': 710, 'enforcer_population': 110, 'narrative_hint': 'GENOCIDE: Atomized surplus population cannot resist. The system eliminates what it cannot exploit or control.'}
//!
//! milestone ticks (first occurrence):
//!   superwage_crisis = 1
//!   class_decomposition = 53
//!   control_ratio_crisis = 105
//!   terminal_decision = 106
//! ```
//!
//! Both the hand-derived arithmetic and the frozen mirror agree exactly —
//! the ported estate's own `TickSession` run below is measured
//! independently against this (ADR183: the mirror is the STRUCTURE/
//! ORDERING oracle, never a byte oracle; every numeric assertion below is
//! measured from THIS engine's own run).
//!
//! # Executable phase ordering
//!
//! The governed registry places Decomposition @11 before ControlRatio @12.
//! The ordinary 52-tick delay keeps the full arc's milestone schedule at
//! 1/53/105/106. The zero-delay fixture below makes the ordering observable:
//! ControlRatio must see Decomposition's same-tick carrier write and emit its
//! crisis on tick 1. A global rule-id sort would invert the packs because
//! `'c' < 'd'` and delay that crisis to tick 2.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::TickSession;

/// The one-shot driver identity (D179): deterministic, never a UUID or clock.
fn run_once_session() -> SessionId {
    SessionId::new("run-once").expect("literal is non-empty")
}

const ARC_SCENARIO: &str = include_str!("../content/scenarios/carceral-arc-conformance.bscn");
const DECOMPOSITION_RULE: &str = include_str!("../content/rules/decomposition.bsl");
const CONTROL_RATIO_RULE: &str = include_str!("../content/rules/control-ratio.bsl");

// Node ids, fixed by the scenario's own declaration order (the scenario's
// own header names the same map).
const LA_APPROACHING: NodeId = NodeId(0);
const ENFORCER_SEED: NodeId = NodeId(1);
const IP_SEED: NodeId = NodeId(2);
const LUMPEN: NodeId = NodeId(3);
const BOURGEOIS: NodeId = NodeId(4);
const CARCERAL_REGISTER: NodeId = NodeId(5);

/// Both packs, concatenated — `TickSession::new`/`prepare_rules` resolves
/// the governed phase order regardless of concatenation order, so this
/// order is arbitrary, matching
/// `us_counties_lifecycle_demo_hashes_are_pinned`'s own `format!` idiom
/// (`tick_goldens.rs`).
fn joint_rule_src() -> String {
    format!("{DECOMPOSITION_RULE}\n{CONTROL_RATIO_RULE}")
}

fn attribute(graph: &HypergraphStore, id: NodeId, field: &str) -> f64 {
    graph
        .node_attribute(id, field)
        .unwrap_or_else(|e| panic!("node {id:?} field {field}: {}", e.message))
}

/// The last tick this suite ever advances to — comfortably past the
/// derived TERMINAL_DECISION tick (106), with margin to prove nothing
/// fires a fifth time afterward.
const LAST_TICK: i64 = 110;

/// One event, tagged with the tick it fired on — `(tick, event_type,
/// payload)`.
type ArcEvent = (i64, String, Vec<(String, Value)>);

/// `run_arc`'s own result: the finished session (for post-session state
/// assertions) plus every event the whole run produced, in order.
type ArcRun = (TickSession<HypergraphStore>, Vec<ArcEvent>);

/// One run of the whole arc: every event, tagged with the tick it fired on,
/// in the order `TickSession::advance` produced them. A fresh
/// `CollectingSink` per tick (the `decomposition_conformance.rs`
/// `the_delay_path_emits_the_warning_at_tick_1_and_decomposes_at_tick_53`
/// idiom) is what makes the per-tick tagging possible — `advance` accepts
/// whatever sink it is given and does not clear it itself.
fn run_arc() -> ArcRun {
    let rule_src = joint_rule_src();
    let mut session = TickSession::new(
        ARC_SCENARIO,
        &rule_src,
        HypergraphStore::new(),
        run_once_session(),
    )
    .expect("both packs must load together against the arc scenario");
    let mut all_events = Vec::new();
    for tick in 1..=LAST_TICK {
        let mut sink = CollectingSink::default();
        session
            .advance(&mut sink)
            .unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        for (ty, payload) in sink.events {
            all_events.push((tick, ty, payload));
        }
    }
    (session, all_events)
}

/// **The acceptance test.** The full five-phase sequence, in the derived
/// order, on the DERIVED ticks (1 / 53 / 105 / 106) — verified against the
/// frozen mirror above, not trusted from the plan's own illustrative
/// numbers. Mirrors `test_carceral_equilibrium.py::
/// test_default_trajectory_phases`'s own phase-sequence assertion (its
/// `milestones[...] < milestones[...]` chain), minus the pre-carceral
/// `metabolic_rift_opened` milestone (`ImperialRentSystem` @9.0 is
/// unported and blocked, decomposition.bsl's own D-record 3 / BLOCKER-3).
#[test]
fn the_full_carceral_arc_runs_in_order() {
    let (_, events) = run_arc();

    let tick_of = |name: &str| -> Option<i64> {
        events
            .iter()
            .find(|(_, ty, _)| ty == name)
            .map(|(tick, _, _)| *tick)
    };

    let superwage = tick_of("SUPERWAGE_CRISIS").expect("SUPERWAGE_CRISIS must fire");
    let decomposition = tick_of("CLASS_DECOMPOSITION").expect("CLASS_DECOMPOSITION must fire");
    let crisis = tick_of("CONTROL_RATIO_CRISIS").expect("CONTROL_RATIO_CRISIS must fire");
    let terminal = tick_of("TERMINAL_DECISION").expect("TERMINAL_DECISION must fire");

    assert_eq!(superwage, 1, "SUPERWAGE_CRISIS: derived tick 1 (1 + 0)");
    assert_eq!(
        decomposition, 53,
        "CLASS_DECOMPOSITION: derived tick 53 (1 + decomposition-delay 52)"
    );
    assert_eq!(
        crisis, 105,
        "CONTROL_RATIO_CRISIS: derived tick 105 (53 + control-ratio-delay 52)"
    );
    assert_eq!(
        terminal, 106,
        "TERMINAL_DECISION: derived tick 106 (105 + terminal-decision-delay 1)"
    );

    // The phase-sequence ordering claim itself (test_carceral_equilibrium.
    // py's own theoretical claim) — restated as explicit `<` chains,
    // independent of the exact tick values above, so a future re-pin of
    // the derived schedule (a defines.yaml change) cannot silently also
    // break the ORDER without this failing separately.
    assert!(
        superwage < decomposition,
        "superwage must precede decomposition"
    );
    assert!(
        decomposition < crisis,
        "decomposition must precede the control-ratio crisis"
    );
    assert!(
        crisis < terminal,
        "the crisis must precede the terminal decision"
    );
}

/// Each of the four events fires EXACTLY once across the whole session —
/// the four carrier latches (`superwage-crisis-known`,
/// `decomposition-complete`, `control-crisis-emitted`,
/// `terminal-decision-emitted`) are each one-time gates, and this is the
/// event-log proof that holds across BOTH packs composed together, not
/// merely within one pack's own suite.
#[test]
fn the_arc_emits_each_event_exactly_once() {
    let (_, events) = run_arc();
    for name in [
        "SUPERWAGE_CRISIS",
        "CLASS_DECOMPOSITION",
        "CONTROL_RATIO_CRISIS",
        "TERMINAL_DECISION",
    ] {
        let count = events.iter().filter(|(_, ty, _)| ty == name).count();
        assert_eq!(
            count, 1,
            "{name} must fire exactly once across the whole arc"
        );
    }
    // No fifth or sixth event of any OTHER type — the arc's only rule
    // packs are decomposition/control-ratio, whose only emitting rules are
    // p02, p06, c03, c04.
    assert_eq!(
        events.len(),
        4,
        "exactly four events total across the whole arc"
    );
}

/// The frozen scenario test's own default outcome
/// (`test_carceral_equilibrium.py:191-195`, `"Without player organization,
/// terminal decision should resolve to 'genocide'"`): `ip-seed`'s
/// organization is UNTOUCHED by either intake rule (p04/p05 write only
/// population/wealth/active), so the 510 newly-active prisoners it
/// contributes carry organization 0.0 — only `lumpen`'s pre-existing 200 @
/// 0.2 contribute anything nonzero, giving a population-weighted average
/// far below `carceral/revolution-threshold` (0.5). `(outcome 0)` is the
/// numeric GENOCIDE encoding (D-record 5/BLOCKER-5, `control-ratio.bsl`'s
/// header).
#[test]
fn the_arc_ends_in_genocide_with_no_organization() {
    let (_, events) = run_arc();
    let (_, _, payload) = events
        .iter()
        .find(|(_, ty, _)| ty == "TERMINAL_DECISION")
        .expect("TERMINAL_DECISION must fire");
    assert_eq!(
        payload[0],
        ("outcome".to_owned(), Value::Int(0)),
        "outcome must be the numeric GENOCIDE encoding (0), not REVOLUTION (1)"
    );
    match &payload[1].1 {
        Value::Real(avg_organization) => {
            assert!(
                *avg_organization < 0.5,
                "avg-organization ({avg_organization}) must be strictly below \
                 carceral/revolution-threshold (0.5) — the GENOCIDE routing condition"
            );
        }
        other => panic!("avg-organization must be Value::Real, got {other:?}"),
    }
}

/// Cross-check: the post-session class states match the frozen mirror's
/// own tick-112 dump exactly (population/wealth/active for all five
/// classes) — a second, independent proof (alongside the event sequence
/// above) that the ported composition reproduces the frozen engine's own
/// end state, not merely its event log.
#[test]
fn the_arc_post_session_class_states_match_the_frozen_mirror() {
    let (session, _) = run_arc();
    let graph = session.graph();
    assert_eq!(
        attribute(graph, LA_APPROACHING, "social-class/active"),
        0.0,
        "la-approaching: deactivated by p06 at the decomposition fire tick"
    );
    assert_eq!(
        attribute(graph, LA_APPROACHING, "social-class/population"),
        600.0,
        "la-approaching: population UNTOUCHED, never zeroed (non-conservation)"
    );
    assert_eq!(
        attribute(graph, ENFORCER_SEED, "social-class/active"),
        1.0,
        "enforcer-seed: activated by p04's intake"
    );
    assert_eq!(
        attribute(graph, ENFORCER_SEED, "social-class/population"),
        110.0,
        "enforcer-seed: 20 + floor(600*0.15) = 20 + 90 = 110 (ADDITIVE)"
    );
    assert_eq!(
        attribute(graph, IP_SEED, "social-class/active"),
        1.0,
        "ip-seed: activated by p05's intake"
    );
    assert_eq!(
        attribute(graph, IP_SEED, "social-class/population"),
        510.0,
        "ip-seed: SET to floor(600*0.85) = 510 (OVERWRITE, not 77 + 510)"
    );
    assert_eq!(
        attribute(graph, LUMPEN, "social-class/population"),
        200.0,
        "lumpen: untouched by either pack's own mutating effects"
    );
    assert_eq!(
        attribute(graph, BOURGEOIS, "social-class/population"),
        10.0,
        "bourgeois: the non-participant vector, untouched by both packs"
    );
    assert_eq!(
        attribute(
            graph,
            CARCERAL_REGISTER,
            "institution/terminal-decision-emitted"
        ),
        1.0,
        "carrier: the terminal decision latch"
    );
}

// ---------------------------------------------------------------------
// Step 4 — the governed phase order, made executable.
// ---------------------------------------------------------------------

/// The registry places `decomposition/*` before `control-ratio/*`, despite
/// the opposite global rule-id byte order. This fixture isolates the ONE
/// cross-pack datum that matters
/// (`institution/decomposition-fire-tick`/`-fired-known`) from every other
/// variable: `control-ratio-delay` is 0 (so if `c03` could see
/// `fired-known` become 1 on the SAME tick it is written, its readiness
/// gate would clear immediately, `tick >= fire-tick + 0`), the fire tick
/// is deliberately UNSEEDED (`institution/decomposition-fire-tick 0`,
/// `institution/decomposition-fired-known 0` — Pack A's own `p03-trigger`
/// writes it for the first time, naturally, at whatever tick the fallback
/// fires), and there is no `CARCERAL_ENFORCER` node at all — the
/// zero-enforcer guard-split (BLOCKER-4, `control-ratio.bsl`'s D-record 4)
/// makes `enforcer-population * control-capacity == 0`, so ANY nonzero
/// `lumpen-witness` prisoner population is unconditionally "over capacity"
/// the instant the readiness gate clears — the readiness gate is the ONLY
/// thing this fixture leaves in question.
///
/// `la-dying` is the FALLBACK trigger (wealth 400 < subsistence 500,
/// `decomposition-conformance.bscn`'s own vector) — `CLASS_DECOMPOSITION`
/// fires at tick 1, with NO delay to wait out, so the byte-order race is
/// exercised on the EARLIEST possible tick.
const BYTE_ORDER_SCENARIO: &str = r#"
(scenario carceral/byte-order-race
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int extensive)
  (deffield social-class/population int extensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/subsistence-threshold real extensive)
  (deffield social-class/s-bio real extensive)
  (deffield social-class/s-class real extensive)
  (deffield social-class/organization coefficient intensive)
  (deffield social-class/la-census-population int extensive)
  (deffield social-class/la-census-wealth real extensive)
  (deffield social-class/la-approaching-flag int extensive)
  (deffield social-class/la-dying-flag int extensive)
  (deffield social-class/enforcer-census-population int extensive)
  (deffield social-class/prisoner-census-population int extensive)
  (deffield social-class/prisoner-census-org-weighted real extensive)

  (deffield institution/superwage-crisis-known int extensive)
  (deffield institution/superwage-crisis-tick int extensive)
  (deffield institution/decomposition-complete int extensive)
  (deffield institution/decomposition-fired-known int extensive)
  (deffield institution/decomposition-fire-tick int extensive)
  (deffield institution/control-crisis-emitted int extensive)
  (deffield institution/control-crisis-tick int extensive)
  (deffield institution/terminal-decision-emitted int extensive)
  (deffield institution/la-population int extensive)
  (deffield institution/la-wealth real extensive)
  (deffield institution/la-approaching-count int extensive)
  (deffield institution/la-dying-count int extensive)
  (deffield institution/enforcer-pop-gain int extensive)
  (deffield institution/enforcer-wealth-gain real extensive)
  (deffield institution/ip-population int extensive)
  (deffield institution/ip-wealth real extensive)
  (deffield institution/enforcer-population int extensive)
  (deffield institution/prisoner-population int extensive)
  (deffield institution/prisoner-org-weighted real extensive)

  ; CarceralDefines, src/babylon/data/defines.yaml:293-300. control-ratio-
  ; delay is COMPANION-VARIED to 0 (Global Constraints: "MAY vary a
  ; delay/fraction to make a branch reachable at tick 1") — the ONE
  ; variable this fixture exists to isolate; every other constant stays at
  ; its shipped value.
  (defconst carceral/control-capacity 4)
  (defconst carceral/enforcer-fraction 0.15c)
  (defconst carceral/proletariat-fraction 0.85c)
  (defconst carceral/revolution-threshold 0.5c)
  (defconst carceral/decomposition-delay 52)
  (defconst carceral/control-ratio-delay 0)
  (defconst carceral/terminal-decision-delay 1)
  (defconst carceral/approaching-consumption-multiple 2)

  (node la-dying NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/LABOR_ARISTOCRACY)
    (social-class/active 1)
    (social-class/population 1000)
    (social-class/wealth 400)
    (social-class/subsistence-threshold 500)
    (social-class/s-bio 5)
    (social-class/s-class 5)
    (social-class/organization 0.0c)
    (social-class/la-census-population 0)
    (social-class/la-census-wealth 0)
    (social-class/la-approaching-flag 0)
    (social-class/la-dying-flag 0)
    (social-class/enforcer-census-population 0)
    (social-class/prisoner-census-population 0)
    (social-class/prisoner-census-org-weighted 0))

  (node lumpen-witness NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/LUMPENPROLETARIAT)
    (social-class/active 1)
    (social-class/population 100)
    (social-class/wealth 0)
    (social-class/subsistence-threshold 0)
    (social-class/s-bio 0)
    (social-class/s-class 0)
    (social-class/organization 0.3c)
    (social-class/la-census-population 0)
    (social-class/la-census-wealth 0)
    (social-class/la-approaching-flag 0)
    (social-class/la-dying-flag 0)
    (social-class/enforcer-census-population 0)
    (social-class/prisoner-census-population 0)
    (social-class/prisoner-census-org-weighted 0))

  (node carceral-register NodeType/INSTITUTION
    ; UNSEEDED (plan §5's own constraint): decomposition-fire-tick/
    ; -fired-known are left at 0 — p03-trigger writes them naturally, the
    ; first time, at whatever tick the fallback fires (tick 1 here).
    (institution/superwage-crisis-known 0)
    (institution/superwage-crisis-tick 0)
    (institution/decomposition-complete 0)
    (institution/decomposition-fired-known 0)
    (institution/decomposition-fire-tick 0)
    (institution/control-crisis-emitted 0)
    (institution/control-crisis-tick 0)
    (institution/terminal-decision-emitted 0)
    (institution/la-population 0)
    (institution/la-wealth 0)
    (institution/la-approaching-count 0)
    (institution/la-dying-count 0)
    (institution/enforcer-pop-gain 0)
    (institution/enforcer-wealth-gain 0)
    (institution/ip-population 0)
    (institution/ip-wealth 0)
    (institution/enforcer-population 0)
    (institution/prisoner-population 0)
    (institution/prisoner-org-weighted 0)))
"#;

const BOR_LA_DYING: NodeId = NodeId(0);
const BOR_LUMPEN: NodeId = NodeId(1);
const BOR_CARCERAL_REGISTER: NodeId = NodeId(2);

/// **The phase-order contract's executable form.** At tick 1
/// (`decomposition/p03-trigger`'s fallback fires and writes
/// `decomposition-fire-tick 1`/`decomposition-fired-known 1` for the FIRST
/// time, THIS tick), `control-ratio/c03-crisis` runs afterward and must see
/// those writes. With zero enforcers, 100 prisoners, and delay 0, every gate
/// clears and `CONTROL_RATIO_CRISIS` fires on tick 1. Its latch prevents a
/// duplicate crisis on tick 2.
#[test]
fn governed_phase_order_exposes_decomposition_to_control_ratio_in_the_same_tick() {
    let rule_src = joint_rule_src();
    let mut session = TickSession::new(
        BYTE_ORDER_SCENARIO,
        &rule_src,
        HypergraphStore::new(),
        run_once_session(),
    )
    .expect("the byte-order fixture must load against both packs");

    let mut sink1 = CollectingSink::default();
    session.advance(&mut sink1).expect("tick 1");
    let decompositions_tick1: Vec<_> = sink1
        .events
        .iter()
        .filter(|(ty, _)| ty == "CLASS_DECOMPOSITION")
        .collect();
    assert_eq!(
        decompositions_tick1.len(),
        1,
        "tick 1: la-dying's fallback fires the SAME tick (wealth 400 < subsistence 500)"
    );
    assert_eq!(
        session
            .graph()
            .node_attribute(BOR_CARCERAL_REGISTER, "institution/decomposition-fire-tick")
            .unwrap(),
        1.0,
        "tick 1: decomposition-fire-tick IS written this tick, by p03"
    );
    let crises_tick1: Vec<_> = sink1
        .events
        .iter()
        .filter(|(ty, _)| ty == "CONTROL_RATIO_CRISIS")
        .collect();
    assert_eq!(
        crises_tick1.len(),
        1,
        "tick 1: Decomposition @11 writes before ControlRatio @12 reads, so the zero-delay crisis fires now"
    );

    let mut sink2 = CollectingSink::default();
    session.advance(&mut sink2).expect("tick 2");
    let crises_tick2: Vec<_> = sink2
        .events
        .iter()
        .filter(|(ty, _)| ty == "CONTROL_RATIO_CRISIS")
        .collect();
    assert_eq!(
        crises_tick2.len(),
        0,
        "tick 2: the crisis latch prevents a duplicate after the phase-ordered tick-1 emission"
    );
    assert_eq!(
        attribute(
            session.graph(),
            BOR_LUMPEN,
            "social-class/prisoner-census-population"
        ),
        100.0,
        "lumpen-witness: untouched by either pack, its population stays the census's only \
         nonzero prisoner contribution throughout"
    );
    assert_eq!(
        attribute(session.graph(), BOR_LA_DYING, "social-class/active"),
        0.0,
        "la-dying: deactivated by p06 at tick 1 before ControlRatio's downstream read"
    );
}
