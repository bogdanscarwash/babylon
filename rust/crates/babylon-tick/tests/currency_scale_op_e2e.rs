//! The composed-path proof for the declared-domain Currency scale operation
//! (`bsl-language.rst` §3.2 addendum, Director ruling 2026-08-11,
//! #492/ADR194): a scenario declaring a `defconst`-with-`:cap` Ratio and a
//! rule computing `Currency × Ratio` clear content, load and evaluation
//! through the SAME `run_once`/`run_once_into` the CLI driver and
//! `babylon-client`'s engine link both call — the composed-path lesson
//! `floor_intrinsic_e2e.rs` records (round 1 of ADR188 Row 2's review
//! shipped a helper only a test called; this crate's E2E tests exist so a
//! seam is proved through the REAL entry point, not a lookalike).
//!
//! **Why the money value is an inline literal, not a `:field`.** Slice 1's
//! `GraphSubstrate` attributes are plain `f64` (`scenario.rs`'s own module
//! doc: "No `Currency` attributes" — typed i128 attribute storage is a
//! declared Phase-2 trait revision) and `defconst` refuses a `Currency`
//! literal for the identical reason (`scenario.rs::load_defconst`,
//! unchanged by this addendum). A `Currency` value can currently enter a
//! rule's evaluation ONLY as a literal written directly in the rule body —
//! true before this change and after it; this test's rule reads that
//! literal exactly the way an eviction-pipeline rule will once Territory's
//! own port train lands (out of scope here — this proves the MACHINERY).
//!
//! **Why the result is `emit`ted, not `update-node`d.** The identical gap:
//! `update-node` refuses a `Currency` write (`structural_verbs.rs`'s
//! `currency_writes_are_a_loud_declared_gap_never_a_lossy_cast`), while
//! `emit`'s payload is an evaluated [`babylon_bsl::evaluator::Value`] with
//! no type restriction (`emit_collects_the_evaluated_payload`). Observing
//! the product through the event sink sidesteps the graph-storage gap
//! entirely rather than working around it.
//!
//! Territory's real `rent_spike_multiplier` is `1500.5$ × 2.0` — the exact
//! shape this test proves, with the exact numbers the port train's fixture
//! cites (§492's issue body: "tested moddable to 2.0").

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_tick::run_once_into;

const SCENARIO: &str = r#"
(scenario scale-op-e2e/one-territory
  (deffield territory/population int extensive)
  (defconst territory/rent-spike-multiplier 2.0r :cap 10r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;

const RULE: &str = r#"
(rule vitality/scale-op-e2e-rent-spike
  :role mechanic :evidence derived :material-basis "prove Currency x declared-domain Ratio clears content, load and evaluation (#492/ADR194)"
  :fuel 64
  (bindings
    (binding population :field territory/population)
    (binding multiplier :const territory/rent-spike-multiplier)
    (binding spiked-rent :expr (* 1500.5$ multiplier)))
  (when (> population 0))
  (effects
    (emit EventType/RUPTURE (spiked-rent spiked-rent))))
"#;

#[test]
fn a_rule_scaling_currency_by_a_declared_domain_ratio_runs_through_run_once() {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("Currency x declared-domain Ratio must clear the whole seam");

    assert_eq!(report.fired, 1);

    assert_eq!(sink.events.len(), 1);
    let (event_type, payload) = &sink.events[0];
    assert_eq!(event_type, "RUPTURE");
    let (name, value) = &payload[0];
    assert_eq!(name, "spiked-rent");

    // 1500.5$ x 2.0 = 3001.0$ exactly — a fixed-point x grid-quantized
    // multiply, no binary64 rounding ambiguity at these values.
    assert_eq!(
        *value,
        Value::Currency(babylon_kernel::Currency::from_micro_units(3_001_000_000)),
        "1500.5$ * 2.0r must equal exactly 3001.0$"
    );
}

#[test]
fn the_declared_call_is_deterministic_across_two_full_runs() {
    let mut graph_a = MemoryGraph::new();
    let mut sink_a = CollectingSink::default();
    let a = run_once_into(SCENARIO, RULE, &mut graph_a, &mut sink_a).expect("first run");

    let mut graph_b = MemoryGraph::new();
    let mut sink_b = CollectingSink::default();
    let b = run_once_into(SCENARIO, RULE, &mut graph_b, &mut sink_b).expect("second run");

    assert_eq!(a.after, b.after);
    assert_eq!(sink_a.events, sink_b.events);
}

/// The declared `:cap` fails the WHOLE tick, loudly, when the scenario
/// itself is inconsistent (`E-LOAD-052`) — proved through the full
/// `run_once` seam, not just `scenario::tests`' narrower call into
/// `load_scenario` alone (this is the same "prove it through the real
/// entry point" argument the module doc makes for the whole file).
///
/// **This does NOT reach `E-EVAL-041`, and that absence is itself
/// recorded rather than papered over.** `load_scenario` runs before
/// `load_rule_form`/`run_tick` in `run_once_into`, so an inconsistent
/// `(value, cap)` pair is refused before a rule ever evaluates it — through
/// `defconst`, the ONLY producer of a capped `Value::Ratio` today, the two
/// checks can never observe different verdicts. `E-EVAL-041`'s own defense-
/// in-depth case (a `Value::Ratio` whose `cap` and `value` disagree despite
/// having passed load — e.g. a future non-`defconst` source) is unit-tested
/// directly in `evaluator.rs`, where it CAN be constructed.
#[test]
fn a_ratio_over_its_declared_cap_fails_the_whole_tick_loudly() {
    const OVER_CAP_SCENARIO: &str = r#"
(scenario scale-op-e2e/over-cap
  (deffield territory/population int extensive)
  (defconst territory/rent-spike-multiplier 12r :cap 10r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(OVER_CAP_SCENARIO, RULE, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("exceeds its own :cap"),
        "unexpected message: {err}"
    );
}

/// The mirror success case: a bare (uncapped) declared-domain Ratio — the
/// exact shape `rent_spike_multiplier`'s real `(0, ∞)` domain has — still
/// clears the whole seam and multiplies cleanly for a value an ordinary
/// `Coefficient` could never have carried (well above `1.0`).
#[test]
fn an_uncapped_declared_domain_ratio_also_clears_the_whole_seam() {
    const UNCAPPED_SCENARIO: &str = r#"
(scenario scale-op-e2e/uncapped
  (deffield territory/population int extensive)
  (defconst territory/rent-spike-multiplier 5r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(UNCAPPED_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("an uncapped declared-domain Ratio must still clear the seam");
    assert_eq!(report.fired, 1);
    let (_, payload) = &sink.events[0];
    let (_, value) = &payload[0];
    assert_eq!(
        *value,
        Value::Currency(babylon_kernel::Currency::from_micro_units(7_502_500_000)),
        "1500.5$ * 5.0r must equal exactly 7502.5$"
    );
}

// ---- :floor (DEFECT 1 fix round, adversarial verification of #500) ----
//
// Metabolism's `entropy_factor` is the exemplar consumer the addendum
// itself names for `:floor`: declared domain `(1.0, 3.0]` — the floor is
// the whole thermodynamic point ("extraction costs more than it yields"),
// not decoration. The worked example, byte for byte:
// `(defconst metabolism/entropy-factor 1.5r :floor 1r :cap 3r)`.

const FLOOR_SCENARIO: &str = r#"
(scenario scale-op-e2e/entropy-factor
  (deffield territory/population int extensive)
  (defconst metabolism/entropy-factor 1.5r :floor 1r :cap 3r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;

const FLOOR_RULE: &str = r#"
(rule vitality/scale-op-e2e-entropy-factor
  :role mechanic :evidence derived :material-basis "prove Currency x a floored-AND-capped declared-domain Ratio (#492/ADR194) — entropy_factor's exact shape"
  :fuel 64
  (bindings
    (binding population :field territory/population)
    (binding factor :const metabolism/entropy-factor)
    (binding scaled-extraction :expr (* 1000$ factor)))
  (when (> population 0))
  (effects
    (emit EventType/RUPTURE (scaled-extraction scaled-extraction))))
"#;

/// `entropy_factor`'s own worked example clears the whole seam: `1000$ ×
/// 1.5 = 1500.0$` exactly, with BOTH bounds declared and satisfied.
#[test]
fn a_rule_scaling_currency_by_a_floored_and_capped_ratio_runs_through_run_once() {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(FLOOR_SCENARIO, FLOOR_RULE, &mut graph, &mut sink)
        .expect("Currency x a floored-and-capped Ratio must clear the whole seam");
    assert_eq!(report.fired, 1);
    let (_, payload) = &sink.events[0];
    let (_, value) = &payload[0];
    assert_eq!(
        *value,
        Value::Currency(babylon_kernel::Currency::from_micro_units(1_500_000_000)),
        "1000$ * 1.5r must equal exactly 1500.0$"
    );
}

/// The defect this train fixes, proved at the entry point: a modded
/// `entropy_factor 0.5r` — BELOW the floor `> 1.0` exists to forbid,
/// exactly the violation the ceiling-only machinery could not catch — now
/// fails the whole tick loudly at load, never evaluating silently.
#[test]
fn a_modded_entropy_factor_below_its_floor_fails_the_whole_tick_loudly() {
    const BELOW_FLOOR_SCENARIO: &str = r#"
(scenario scale-op-e2e/entropy-factor-modded-low
  (deffield territory/population int extensive)
  (defconst metabolism/entropy-factor 0.5r :floor 1r :cap 3r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(BELOW_FLOOR_SCENARIO, FLOOR_RULE, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("does not exceed its own :floor"),
        "unexpected message: {err}"
    );
}

/// The exact boundary `entropy_factor`'s `> 1.0` (not `>= 1.0`) exists to
/// forbid: a value AT the floor is still refused, even though it is not
/// BELOW it — the exclusive endpoint, proved through the full seam.
#[test]
fn a_modded_entropy_factor_exactly_at_its_floor_fails_the_whole_tick_loudly() {
    const AT_FLOOR_SCENARIO: &str = r#"
(scenario scale-op-e2e/entropy-factor-at-floor
  (deffield territory/population int extensive)
  (defconst metabolism/entropy-factor 1r :floor 1r :cap 3r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(AT_FLOOR_SCENARIO, FLOOR_RULE, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("does not exceed its own :floor"),
        "unexpected message: {err}"
    );
}

// ---- the zero-endpoint disposition (DEFECT 2 fix round, adversarial
// verification of #500) ----
//
// `Ratio` cannot hold `0` — that is structural, not an oversight: the
// ruling's own text says "positive coefficient", the reader's `E-LEX-027`
// refuses a `0r` literal before it is even a token, and `Ratio::new(0.0)`
// itself refuses at the kernel layer (`scalars.rs`, `Ratio`'s law is
// `𝔾 ∩ (0, ∞)`, open at zero). Lifecycle's `early_mortality_modifier`/
// `carceral_transition_modifier` are `ge=0.0` in the frozen engine, and the
// frozen engine actively PRODUCES `0.0` for them (`mobility.py:187-188` —
// the mortality channel OFF, semantically meaningful, not an error case).
// This addendum's Ratio lane carries these two consumers' domain `(0, 10]`
// INTERIOR only — the open zero endpoint is not representable as a Ratio
// value, full stop, by the same law that makes the whole construct "a
// POSITIVE coefficient" rather than a general-purpose bounded scalar.
//
// "Zero-scaling is the multiply's ABSENCE, not a scale" (the fix-round
// instruction's own framing): the general MECHANISM for expressing "this
// channel is off, contribute nothing" already exists in the language and
// needs no new construct — `guard` (§2.8) gates whether an EFFECT fires at
// all, on a signal SEPARATE from the Ratio-typed magnitude (never "the
// Ratio equals zero", which cannot be written). This proves that mechanism
// clears the whole seam, empirically, not just by static reasoning about
// the grammar: a `guard`-gated `Currency × Ratio` effect, keyed off an
// ordinary Bool `:const`, fires when the flag is true and is skipped
// (never evaluating the multiply at all) when it is false — exactly "the
// multiply's absence".
//
// **What this does NOT settle, and says so plainly**: WHICH signal a real
// Lifecycle port gates on (a dedicated activation const? a doctrine tag? a
// field read?) is that port's own content-modeling decision, not
// machinery — the same way this whole train ports no consumer. This proves
// only that the MECHANISM is available and typechecks/evaluates cleanly
// through the real entry point; it does not pre-choose Lifecycle's answer.

const GATED_SCENARIO: &str = r#"
(scenario scale-op-e2e/gated-modifier
  (deffield territory/population int extensive)
  (defconst lifecycle/early-mortality-modifier 1.24r)

  (node core NodeType/TERRITORY
    (territory/population 1)))
"#;

const GATED_RULE_TEMPLATE: &str = r#"
(rule vitality/scale-op-e2e-gated-modifier
  :role mechanic :evidence derived :material-basis "prove a guard-gated Currency x Ratio effect is the content-layer answer to a zero-valued frozen-engine modifier — the multiply's absence, never a Ratio of zero (#492/ADR194)"
  :fuel 64
  (bindings
    (binding population :field territory/population)
    (binding channel-active :const lifecycle/channel-active)
    (binding modifier :const lifecycle/early-mortality-modifier))
  (when (> population 0))
  (effects
    (guard channel-active
      (emit EventType/RUPTURE (scaled-mortality (* 1000$ modifier))))))
"#;

/// The channel-ACTIVE case: the guard's condition is `#t`, so the
/// `Currency × Ratio` effect fires and the exact product is observed —
/// `1000$ × 1.24 = 1240.0$`.
#[test]
fn a_guard_gated_ratio_multiply_fires_when_the_channel_is_active() {
    let scenario = GATED_SCENARIO.replace(
        "(node core",
        "(defconst lifecycle/channel-active #t)\n\n  (node core",
    );
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(&scenario, GATED_RULE_TEMPLATE, &mut graph, &mut sink)
        .expect("a guard-gated Currency x Ratio effect must clear the whole seam");
    assert_eq!(report.fired, 1);
    assert_eq!(
        sink.events.len(),
        1,
        "the guard must have let the effect through"
    );
    let (_, payload) = &sink.events[0];
    let (_, value) = &payload[0];
    assert_eq!(
        *value,
        Value::Currency(babylon_kernel::Currency::from_micro_units(1_240_000_000)),
        "1000$ * 1.24r must equal exactly 1240.0$"
    );
}

/// The channel-OFF case — the frozen engine's `early_mortality_modifier =
/// 0.0` behavior, expressed WITHOUT ever writing `0r` (which cannot exist):
/// the guard's condition is `#f`, so the `Currency × Ratio` multiply is
/// never evaluated and no event is emitted — "the multiply's absence",
/// proved by an EMPTY event list. The multiply sits INLINE in the guarded
/// `emit`, which is what makes that claim true: `run_tick` resolves
/// `:expr` BINDINGS before any guard (the #498 lesson), so a
/// binding-shaped multiply would run — and fuel-charge, and surface any
/// eval error — under a false guard. Guarded effect bodies are genuinely
/// lazy; bindings are not.
#[test]
fn a_guard_gated_ratio_multiply_never_evaluates_when_the_channel_is_off() {
    let scenario = GATED_SCENARIO.replace(
        "(node core",
        "(defconst lifecycle/channel-active #f)\n\n  (node core",
    );
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(&scenario, GATED_RULE_TEMPLATE, &mut graph, &mut sink)
        .expect("a false guard must skip its effect, not fail the tick");
    assert_eq!(
        report.fired, 1,
        "the rule still ran — only the guarded effect was skipped"
    );
    assert!(
        sink.events.is_empty(),
        "the multiply must never evaluate when the channel is off: {:?}",
        sink.events
    );
}
