//! Registration + scenario-ceremony conformance for the joint Decomposition
//! (@11.0) + ControlRatio (@12.0) port train — Task 1 of
//! `docs/superpowers/plans/2026-08-17-decomposition-controlratio-port.md`
//! (frozen source: `src/babylon/engine/systems/decomposition.py`, 370
//! lines, + `src/babylon/engine/systems/control_ratio.py`, 248 lines).
//!
//! # Task 1 scope
//!
//! This task ships NO `decomposition/*`/`control-ratio/*` rule pack content
//! (that is Task 2 onward) — it registers both systems in `lib.rs`, builds
//! the shared conformance world (`content/scenarios/
//! decomposition-conformance.bscn`), carries the frozen-mirror provenance
//! below, and records THE SPIKE verdict (plan §2's RISK box) that later
//! tasks depend on.
//!
//! # Step 1 red-phase note
//!
//! `scenario_and_empty_pack_load` (below) was written and run BEFORE
//! `"decomposition"`/`"control-ratio"` were added to `lib.rs`'s
//! registered-system `HashSet`; it failed with:
//!
//! ```text
//! rule decomposition/probe rejected: E-LOAD-002: rule decomposition/probe
//! carries no anchor and its first id segment names no registered system —
//! a rule cannot land nowhere (§2.3)
//! ```
//!
//! — confirming the probe reaches the real anchor check
//! (`mod_anchors::check_anchor` against `ctx.systems`, `rule_pipeline.rs:313`)
//! rather than failing for an unrelated reason (the scenario's own
//! declarations loaded clean). Registering both systems turned it green.
//! Same deviation from the plan's literal "empty rule source" text that
//! `production_conformance.rs:75-94` already recorded: `run_once_into`'s own
//! `split_content` refuses a content set with zero `(rule …)` top-forms
//! outright, so a truly empty source cannot reach the anchor check at all;
//! this test uses two minimal, never-firing probe rules instead, one per
//! newly-registered system.
//!
//! # THE SPIKE (Step 5) — verdict
//!
//! Two throwaway spike rules were run through the real `run_once_into`
//! driver against this exact scenario, then deleted (their bodies never
//! landed in this file):
//!
//! - **(a)** a carrier-anchored rule (`:field institution/…` binding) folding
//!   `(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it
//!   social-class/population))` into `institution/la-population` — LOADED,
//!   EVALUATED, and produced the correct sum: `1000 + 20 + 77 + 200 + 10 =
//!   1307`, asserted directly against the post-tick graph before deletion.
//! - **(b)** a class-anchored rule reading `(field-of (select-max (nodes
//!   NodeType/INSTITUTION) 1) institution/decomposition-complete)` — LOADED
//!   AND EVALUATED CLEAN.
//!
//! **Neither shape refused.** The incidence-edge fallback design (seeding a
//! carrier→class edge and reading via `neighbors`) is NOT needed; Task 2
//! onward relies on both shapes directly, exactly as plan §2 specifies. This
//! is also the FIRST landed content to use `NodeType/INSTITUTION` as a
//! singleton carrier reached via `(select-max (nodes NodeType/INSTITUTION)
//! 1)`, and the FIRST to fold over `(nodes NodeType/SOCIAL_CLASS)` rather
//! than a `neighbors` query — every prior `fold`/`select-max` in the landed
//! estate reads `neighbors` (`territory.bsl:157-160`, `production.bsl:172-
//! 175`), which is capability precedent, not content precedent; this
//! scenario is the content precedent.
//!
//! # Frozen-mirror provenance
//!
//! Every state value below was printed by the frozen `DecompositionSystem`
//! (@11.0) then `ControlRatioSystem` (@12.0), one `step()` each, sharing ONE
//! `TickContext.persistent_data` dict (matching the frozen engine's own
//! single `context` threaded through every system in a tick), over a
//! fixture that mirrors `decomposition-conformance.bscn` node for node. The
//! command, from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run python \
//!     rust/crates/babylon-tick/content/scenarios/decomposition_conformance.py
//! ```
//!
//! Its output on 2026-08-17, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml, carceral: section):
//!   carceral.control_capacity = 4
//!   carceral.enforcer_fraction = 0.15
//!   carceral.proletariat_fraction = 0.85
//!   carceral.revolution_threshold = 0.5
//!   carceral.decomposition_delay = 52
//!   carceral.control_ratio_delay = 52
//!   carceral.terminal_decision_delay = 1
//!   carceral.<approaching-consumption-multiple> = 2  (bare literal, decomposition.py:155, NO defines backing)
//!
//! post-tick persistent_data (the frozen state machine §2 reformulates onto the carrier):
//!   _class_decomposition_tick = 1
//!   _decomposition_complete = True
//!   _superwage_crisis_tick = 1
//!
//! post-tick social classes:
//!   la-dying       active=False population=1000 (seed 1000) wealth=400.0 (seed 400.0)
//!   enforcer-seed  active=True population=170 (seed 20) wealth=160.0 (seed 100.0)
//!   ip-seed        active=True population=850 (seed 77) wealth=340.0 (seed 33.0)
//!   lumpen         active=True population=200 (seed 200) wealth=10.0 (seed 10.0)
//!   bourgeois      active=True population=10 (seed 10) wealth=9000.0 (seed 9000.0)
//!
//! events:
//!   superwage_crisis {'payer_id': 'C003', 'receiver_id': 'la-dying', 'desired_wages': 0.0, 'available_pool': 0.0, 'narrative_hint': 'SUPERWAGE CRISIS: Labor aristocracy wealth collapsing. Super-wages cannot sustain the privileged stratum.'}
//!   class_decomposition {'source_class': 'la-dying', 'source_population': 1000, 'source_wealth': 400.0, 'enforcer_fraction': 0.15, 'proletariat_fraction': 0.85, 'population_transferred': {'to_enforcer': 150, 'to_proletariat': 850}, 'wealth_transferred': {'to_enforcer': 60.0, 'to_proletariat': 340.0}, 'trigger_event': 'superwage_crisis', 'narrative_hint': 'CLASS DECOMPOSITION: Labor aristocracy collapses. 150 become guards/cops. 850 fall into the precariat.'}
//! ```
//!
//! **Both the early-warning `SUPERWAGE_CRISIS` and the fallback-triggered
//! `CLASS_DECOMPOSITION` fire in the SAME tick** — a real, observed frozen
//! behavior, not a design assumption: `la-dying`'s wealth (400) is both
//! `< subsistence_threshold + 2*consumption` (500 + 2*10 = 520, the
//! "approaching" early-warning gate) AND `< subsistence_threshold` (500, the
//! `la_about_to_die` fallback gate), and `DecompositionSystem.step()`
//! evaluates both gates sequentially within one call
//! (`decomposition.py:179-223`) with no tick-order separation between them.
//! `ip-seed`'s OVERWRITE (`population: 77 -> 850`, NOT `77 + 850`) versus
//! `enforcer-seed`'s ADDITIVE gain (`population: 20 -> 170 = 20 + 150`) is
//! exactly the two mutation shapes the scenario's own header names as
//! provable by this fixture's non-zero pre-seeds.
//!
//! # Why exact equality and no tolerance
//!
//! `Int * Real` and `int()` truncation are the only operations either engine
//! performs to reach the numbers above — both correctly-rounded binary64
//! (`bsl-language.rst` §4.3) — so later tasks' Rust-side pins may assert
//! exact equality against BSL-measured values, not against these printed
//! floats (ADR183: this mirror is a structure/ordering oracle, not a byte
//! oracle; no `decomposition/*`/`control-ratio/*` rule exists yet in Task 1
//! to measure BSL-side numbers from).
//!
//! # Task 2 — Pack A's p01 (LA census) + p02 (early warning)
//!
//! The four tests below are the first `decomposition/*` rules to run
//! against this world; every asserted value is measured from the BSL engine
//! itself (`content/rules/decomposition.bsl`), cross-checked by hand against
//! the frozen mirror's dump above: `la-dying`'s census carries its own
//! population (1000) and wealth (400.0) exactly, both flags fire (wealth
//! 400 < subsistence 500 < subsistence + 2*consumption 520), and every
//! other class's four census fields stay at their seeded 0 (the D127
//! hash-neutral idiom — p01 has no `when` clause). `SUPERWAGE_CRISIS` fires
//! exactly once with the flattened 3-key payload (D-records 4/5 in
//! `decomposition.bsl`'s header: `payer_id`/`narrative_hint` dropped, no
//! string payloads on `emit`), and the carrier latches
//! `superwage-crisis-known`/`-tick` (1 / tick 1) the SAME tick — matching
//! the frozen mirror's `_superwage_crisis_tick = 1`.
//!
//! **Fuel — measured, not guessed, per the E-LOAD-040 refusal readback,
//! THEN the documented `bound ≥ :fuel` / runtime `bound + 1` off-by-one
//! (`bsl-language.rst` §4.5, "authors should budget `:fuel ≥ bound + 1`"):**
//! p01's rule declared with `:fuel 1` refused at load with `E-LOAD-040:
//! rule decomposition/p01-la-census static bound 72 exceeds its declared
//! :fuel 1`; p02 (once p01's fuel was raised) refused the same way at
//! `static bound 32`. Setting `:fuel` to those EXACT computed bounds (72,
//! 32) loads clean but then `E-EVAL-040`s at runtime ("fuel meter reached
//! zero") — the documented §3.7/§4.5 boundary: the load check accepts
//! `bound == :fuel`, but the runtime meter must stay strictly positive, so
//! the true minimum is `bound + 1`. `:fuel 73` / `:fuel 33` (measured
//! bound + 1, not a round margin) is what the rules below declare.
//!
//! # Task 3 — Pack A's p03 (the carrier trigger + the frozen split)
//!
//! `p03-trigger`'s fuel, measured the same way: `:fuel 1` refused at load
//! with `E-LOAD-040: rule decomposition/p03-trigger static bound 168
//! exceeds its declared :fuel 1`. Setting `:fuel 168` (the exact bound) —
//! UNLIKE p01/p02 — loaded AND ran clean, with no `E-EVAL-040` at runtime.
//!
//! **Retraction (fix round 1, PR review): the headroom is NOT a `guard`
//! effect.** A `guard` charges its body exactly once regardless of whether
//! its condition passes — `cost(guard) = 1 + cost(cond) + Σ cost(effects)`
//! (`bound_checker.rs:259-263`) — so a `guard` creates no static/dynamic
//! gap by itself, and every `:expr` binding (including `fallback-fire` and
//! `delay-elapsed-fire`, both bound BEFORE `should-fire`) evaluates
//! eagerly and unconditionally, once per subject per tick, regardless of
//! what any later binding or guard does (`rule_pipeline.rs::
//! resolve_expr_bindings`). The ORIGINAL claim in this paragraph — that
//! the guard's tick-1-fires/tick-2-skips shape explains the headroom — is
//! WRONG and is retracted; it must not become precedent for Task 4/5 fuel
//! budgeting.
//!
//! **The real mechanism: `or` short-circuits (§4.1, `evaluator.rs:636-
//! 658`).** `should-fire`'s own body is `(and (or fallback-fire delay-
//! elapsed-fire) …)` — and because `fallback-fire`/`delay-elapsed-fire`
//! are themselves PRE-BOUND named bindings by the time `should-fire`
//! evaluates, referencing either inside this `or` is a single `Symbol`
//! variable lookup costing exactly `cost::VARIABLE_REF = 1`
//! (`fuel.rs:23`), not a re-evaluation of the whole named expression. The
//! static bound counts BOTH operands of every `or`/`and` unconditionally
//! (`bound_checker.rs`'s `sum_costs` has no short-circuit-aware branch);
//! at runtime, `fallback-fire` evaluates `#t` first and `or` returns
//! immediately (`stop_on = true` for `or`, `evaluator.rs:645-658`),
//! skipping the `delay-elapsed-fire` variable-ref read entirely. That
//! skipped read is the ENTIRE headroom: exactly 1 fuel unit. `charge`'s
//! exhaustion check is strict `amount >= *fuel` (`evaluator.rs:330-337`,
//! the meter must stay strictly positive) — consuming `bound - 1 = 167`
//! against a `168`-unit budget leaves exactly 1 unit of slack at every
//! charge point along the way, which is exactly why `:fuel 168` cleared
//! runtime here and nowhere else in this pack. This is a FRAGILE,
//! fixture-specific 1-unit margin (it depends on `fallback-fire`
//! evaluating true before `delay-elapsed-fire` is read, which depends on
//! THIS fixture's fallback-trigger vector), not a structural property of
//! `guard` or of this rule shape in general — it must not be relied on for
//! any other rule's fuel budgeting. `:fuel 169` (measured bound + 1) is
//! declared below anyway, matching the §4.5 authors-should-budget
//! convention p01/p02 already follow, rather than relying on this
//! particular fixture's short-circuit margin.
//!
//! # Task 4 addendum — p03's static bound is SCENARIO-DEPENDENT
//!
//! `:fuel 169` (measured against the 5-`SOCIAL_CLASS` primary world) was
//! insufficient once Task 4's `decomposition-delay-conformance.bscn`
//! (Task 4 carried mutation obligation 11's `la-inactive`, a SIXTH
//! `SOCIAL_CLASS` node) tried to load the same rule set: `E-LOAD-040: rule
//! decomposition/p03-trigger static bound 176 exceeds its declared :fuel
//! 169`. This is the direct, mechanical consequence of `p03`'s four
//! `(fold sum (nodes NodeType/SOCIAL_CLASS) …)` bindings — each fold's
//! static bound scales with the SOCIAL_CLASS `CardinalityCeiling` the
//! LOADING scenario itself mints (plan §2's own "which also statically
//! bounds every fold over it", Power-of-10 rule 2), so one rule's fuel
//! must cover the WORST-CASE ceiling across every scenario that loads it,
//! not just the one it was first measured against. Re-measured via the
//! same E-LOAD-040 readback against the LARGER world: bound 176, declared
//! `:fuel 177` (measured bound + 1) below — comfortably covers the
//! 5-class primary world's bound (168) and the Task 4 zero-population
//! inline world's (1 SOCIAL_CLASS node, smaller still) too.
//! `p04-enforcer-intake`/`p05-ip-intake`/`p06-la-deactivate` bind no
//! `nodes`/`fold` query at all (every carrier read is a fixed-cost
//! `select-max` over the ceiling-1 INSTITUTION singleton), so their own
//! fuel is scenario-independent — declared once, measured once, below.

use babylon_bsl::evaluator::Value;
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::{run_once_into, TickSession};

/// Fixed deterministic V1 namespace for this file's non-kernel
/// `TickSession` fixtures.
fn test_session() -> SessionId {
    SessionId::new("decomposition-conformance-test").expect("literal is non-empty")
}

const SCENARIO: &str = include_str!("../content/scenarios/decomposition-conformance.bscn");
const RULE: &str = include_str!("../content/rules/decomposition.bsl");

// Node ids, fixed by the scenario's own declaration order (the scenario's
// own header names the same map; territory-conformance.bscn/territory_
// conformance.rs precedent).
const LA_DYING: NodeId = NodeId(0);
const ENFORCER_SEED: NodeId = NodeId(1);
const IP_SEED: NodeId = NodeId(2);
const LUMPEN: NodeId = NodeId(3);
const BOURGEOIS: NodeId = NodeId(4);
const CARCERAL_REGISTER: NodeId = NodeId(5);

// Task 4's delay-path companion (`decomposition-delay-conformance.bscn`) —
// a SEPARATE world, not a modification of the fallback-trigger world above
// (Global Constraints: every pre-existing golden byte-identical). Node ids
// again fixed by declaration order.
const DELAY_SCENARIO: &str =
    include_str!("../content/scenarios/decomposition-delay-conformance.bscn");
const DELAY_LA_APPROACHING: NodeId = NodeId(0);
const DELAY_LA_INACTIVE: NodeId = NodeId(1);
const DELAY_CARCERAL_REGISTER: NodeId = NodeId(6);

/// Task 2 scope: Pack A's p01 (LA census) + p02 (early warning) only.
fn run() -> (HypergraphStore, CollectingSink) {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the Pack A rules must run");
    (graph, sink)
}

fn attribute(graph: &HypergraphStore, id: NodeId, field: &str) -> f64 {
    graph
        .node_attribute(id, field)
        .unwrap_or_else(|e| panic!("node {id:?} field {field}: {}", e.message))
}

/// p01 publishes the LA census ONLY for the active LA — every other class
/// (a different role, or an inactive one) writes zero to all four census
/// fields, the D127 hash-neutral idiom (no `when` skip).
#[test]
fn p01_publishes_the_la_census_only_for_the_active_la() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/la-census-population"),
        1000.0,
        "la-dying: la-census-population"
    );
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/la-census-wealth"),
        400.0,
        "la-dying: la-census-wealth"
    );
    for (id, name) in [
        (ENFORCER_SEED, "enforcer-seed"),
        (IP_SEED, "ip-seed"),
        (LUMPEN, "lumpen"),
        (BOURGEOIS, "bourgeois"),
    ] {
        for field in [
            "social-class/la-census-population",
            "social-class/la-census-wealth",
            "social-class/la-approaching-flag",
            "social-class/la-dying-flag",
        ] {
            assert_eq!(
                attribute(&graph, id, field),
                0.0,
                "{name}: {field} must be 0"
            );
        }
    }
}

/// p01's two flags: `la-dying`'s wealth (400) is below both subsistence
/// (500, the dying gate) and subsistence + 2*consumption (520, the
/// approaching gate) — `wealth < subsistence` implies
/// `wealth < subsistence + 2*consumption` since consumption >= 0.
#[test]
fn p01_flags_the_dying_la() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/la-dying-flag"),
        1.0,
        "la-dying: la-dying-flag"
    );
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/la-approaching-flag"),
        1.0,
        "la-dying: la-approaching-flag"
    );
}

/// p02 emits exactly one SUPERWAGE_CRISIS, with the flattened payload
/// (`payer_id`/`narrative_hint` dropped, D-record 4/5) — `receiver` is a
/// NodeRef to the LA subject itself.
#[test]
fn p02_emits_superwage_crisis_once_with_the_receiver_ref() {
    let (_, sink) = run();
    let crises: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "SUPERWAGE_CRISIS")
        .collect();
    assert_eq!(crises.len(), 1, "exactly one SUPERWAGE_CRISIS this tick");
    let (_, payload) = crises[0];
    assert_eq!(
        payload[0],
        ("receiver".to_owned(), Value::NodeRef(LA_DYING))
    );
    assert_eq!(payload[1], ("desired-wages".to_owned(), Value::Real(0.0)));
    assert_eq!(payload[2], ("available-pool".to_owned(), Value::Real(0.0)));
}

/// p02 latches the carrier's `superwage-crisis-known`/`-tick` the SAME tick
/// it emits — decomposition.py:196-197's `persistent["_superwage_crisis_
/// tick"] = tick`.
#[test]
fn p02_latches_the_crisis_tick_on_the_carrier() {
    let (graph, _) = run();
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/superwage-crisis-known"
        ),
        1.0,
        "carrier: superwage-crisis-known"
    );
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/superwage-crisis-tick"
        ),
        1.0,
        "carrier: superwage-crisis-tick == tick 1 (run_once_into always runs tick 1)"
    );
}

/// The load-smoke test, through the REAL `run_once_into` seam — proves the
/// scenario loads clean against BOTH newly-registered systems.
#[test]
fn scenario_and_empty_pack_load() {
    const PROBE_RULE: &str = r#"
(rule decomposition/probe
  :role mechanic :evidence derived :material-basis "load-only smoke: prove the scenario loads against a registered decomposition system"
  :fuel 8
  (bindings (binding wealth :field social-class/wealth))
  (when (< wealth 0))
  (effects
    (update-node self social-class/wealth (set wealth))))

(rule control-ratio/probe
  :role mechanic :evidence derived :material-basis "load-only smoke: prove the scenario loads against a registered control-ratio system"
  :fuel 8
  (bindings (binding pop :field social-class/population))
  (when (< pop 0))
  (effects
    (update-node self social-class/population (set pop))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    run_once_into(SCENARIO, PROBE_RULE, &mut graph, &mut sink)
        .expect("the scenario must load and run against BOTH registered-system probe rules");
}

/// The scenario's own node census, independent of any rule pack — five
/// `SOCIAL_CLASS` fixtures plus the one `INSTITUTION` carrier, no edges
/// (the census is type-scoped via `nodes`, plan §2's own "why no edges are
/// needed" note).
#[test]
fn the_scenario_loads_clean_with_the_declared_census() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    assert_eq!(loaded.node_count, 6, "5 social classes + 1 carrier");
    assert_eq!(
        loaded.edge_count, 0,
        "the census is type-scoped, no edges needed"
    );
    assert_eq!(
        loaded.node_types.get("SOCIAL_CLASS").copied(),
        Some(5),
        "five social-class nodes"
    );
    assert_eq!(
        loaded.node_types.get("INSTITUTION").copied(),
        Some(1),
        "exactly one carrier — this is what gives NodeType/INSTITUTION ceiling 1"
    );
}

/// The `defenum` ordinal-parity test (class-surface plan amendment 7),
/// mirroring `tick_goldens.rs::worldview_member_order_is_the_ruled_ordinal`
/// — THIS scenario's own `SocialRole` re-declaration (not shared across
/// scenarios) must store the SAME eight members in the SAME order as
/// `src/babylon/models/enums/social.py:34-41` (ADR195: declaration order IS
/// the storage ordinal).
#[test]
fn social_role_order_is_the_ruled_ordinal() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    let ty = loaded
        .enums
        .resolve("SocialRole")
        .expect("the SocialRole defenum is declared");
    let members = [
        "CORE_BOURGEOISIE",
        "PERIPHERY_PROLETARIAT",
        "LABOR_ARISTOCRACY",
        "PETTY_BOURGEOISIE",
        "LUMPENPROLETARIAT",
        "COMPRADOR_BOURGEOISIE",
        "INTERNAL_PROLETARIAT",
        "CARCERAL_ENFORCER",
    ];
    for (ordinal, member) in members.iter().enumerate() {
        assert_eq!(
            loaded.enums.ordinal(ty, member),
            Some(ordinal as u32),
            "SocialRole::{member} must sit at ordinal {ordinal} (ADR195)"
        );
    }
}

// ---------------------------------------------------------------------
// Task 3 — Pack A's p03 (the carrier trigger + the frozen transfer split)
// ---------------------------------------------------------------------
//
// `p03-trigger` folds p01's four SAME-TICK census-contribution fields onto
// the carrier (D116), reads p02's SAME-TICK `superwage-crisis-known`/`-tick`
// latch, evaluates the frozen fallback-or-delay decision
// (`decomposition.py:169-208`), and — gated on `decomposition-complete == 0`
// and `la-population > 0` (`decomposition.py:129-130`, `290-291`) — writes
// the trigger latches and the four frozen transfer amounts
// (`decomposition.py:296-299`). This world is the FALLBACK-TRIGGER vector
// (`la-dying`'s wealth 400 < subsistence 500), so `should-decompose` is
// `True` at tick 1 with no 52-tick delay to wait out — matching the frozen
// mirror's own `_class_decomposition_tick = 1`.

/// `p03` folds p01's four census-contribution fields onto the carrier every
/// tick, unconditionally — the D127 hash-neutral idiom applied to the
/// carrier side: the census must stay fresh, not merely be written the one
/// tick the trigger also fires.
#[test]
fn p03_folds_the_la_census_into_the_carrier() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, CARCERAL_REGISTER, "institution/la-population"),
        1000.0,
        "carrier: la-population"
    );
    assert_eq!(
        attribute(&graph, CARCERAL_REGISTER, "institution/la-wealth"),
        400.0,
        "carrier: la-wealth"
    );
    assert_eq!(
        attribute(&graph, CARCERAL_REGISTER, "institution/la-dying-count"),
        1.0,
        "carrier: la-dying-count"
    );
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/la-approaching-count"
        ),
        1.0,
        "carrier: la-approaching-count"
    );
}

/// The fallback trigger (`la-dying-count > 0`) fires with NO delay — this
/// world's `la-dying` is already below subsistence at tick 1
/// (`decomposition.py:158-159`'s `la_about_to_die` vector) — matching the
/// frozen mirror's `_class_decomposition_tick = 1`, `_decomposition_complete
/// = True`.
#[test]
fn p03_fires_on_the_fallback_trigger_without_any_delay() {
    let (graph, _) = run();
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/decomposition-fire-tick"
        ),
        1.0,
        "carrier: decomposition-fire-tick == tick 1"
    );
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/decomposition-fired-known"
        ),
        1.0,
        "carrier: decomposition-fired-known"
    );
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/decomposition-complete"
        ),
        1.0,
        "carrier: decomposition-complete"
    );
}

/// The frozen split arithmetic (`decomposition.py:296-299`), each amount
/// asserted BIT-EXACT against the mirror's `repr` output (the `.to_bits()`
/// idiom, `production_conformance.rs:226-229`) — never a copied float,
/// ADR183: measured from THIS engine. `enforcer_pop_gain`/`proletariat_pop`
/// each floor INDEPENDENTLY (D-record 9's non-conservation defect,
/// transcribed verbatim); neither wealth amount is `int()`-demoted.
#[test]
fn p03_computes_the_frozen_splits() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, CARCERAL_REGISTER, "institution/enforcer-pop-gain").to_bits(),
        150.0_f64.to_bits(),
        "floor(1000 * 0.15) = 150"
    );
    assert_eq!(
        attribute(&graph, CARCERAL_REGISTER, "institution/ip-population").to_bits(),
        850.0_f64.to_bits(),
        "floor(1000 * 0.85) = 850"
    );
    assert_eq!(
        attribute(
            &graph,
            CARCERAL_REGISTER,
            "institution/enforcer-wealth-gain"
        )
        .to_bits(),
        60.0_f64.to_bits(),
        "400.0 * 0.15 = 60.0, exact in binary64, NOT int()-demoted"
    );
    assert_eq!(
        attribute(&graph, CARCERAL_REGISTER, "institution/ip-wealth").to_bits(),
        340.0_f64.to_bits(),
        "400.0 * 0.85 = 340.0, exact in binary64, NOT int()-demoted"
    );
}

/// `decomposition-complete == 0` gates `p03`'s trigger writes (not its
/// census fold) — a second tick over a world with no tick-2 input change
/// must NOT move `decomposition-fire-tick` off tick 1, mirroring the frozen
/// `if persistent.get("_decomposition_complete"): return` early exit
/// (`decomposition.py:129-130`). `TickSession::advance` idiom,
/// `production_conformance.rs::p0_reset_keeps_extraction_intensity_stable_
/// across_two_ticks`'s own precedent. THIS test, not a dedicated mutation,
/// is what proves the `decomposition-complete == 0` conjunct is
/// load-bearing: removing it would re-fire the guard at tick 2 and move
/// `decomposition-fire-tick` to 2, which is exactly the failure this
/// assertion catches.
#[test]
fn p03_is_idempotent_across_two_ticks() {
    let mut session = TickSession::new(SCENARIO, RULE, HypergraphStore::new(), test_session())
        .expect("the pack must load into a session");
    let mut sink = CollectingSink::default();
    session.advance(&mut sink).expect("tick 1");
    let fire_tick_after_1 = attribute(
        session.graph(),
        CARCERAL_REGISTER,
        "institution/decomposition-fire-tick",
    );
    assert_eq!(fire_tick_after_1, 1.0, "tick 1: fires, fire-tick == 1");
    session.advance(&mut sink).expect("tick 2");
    let fire_tick_after_2 = attribute(
        session.graph(),
        CARCERAL_REGISTER,
        "institution/decomposition-fire-tick",
    );
    assert_eq!(
        fire_tick_after_2, 1.0,
        "tick 2: decomposition-complete == 1 already, so the guard does not \
         re-execute — fire-tick stays pinned at tick 1, never overwritten to 2"
    );
}

// ---------------------------------------------------------------------
// Task 4 — Pack A's p04 (enforcer intake) + p05 (IP intake) + p06 (LA
// deactivate), closing Pack A. All three key off `decomposition-fire-tick
// == tick`, p03's SAME-TICK write (D116); this fallback-trigger world
// fires at tick 1, so `run_once_into` (a single tick) is enough to observe
// every write.
// ---------------------------------------------------------------------

/// p04 is ADDITIVE (decomposition.py:327-332): `enforcer-seed`'s pre-seeded
/// population (20) and wealth (100.0) — NOT zero — plus p03's computed
/// gains (150 / 60.0) give 170 / 160.0. `active` flips 0 -> 1.
#[test]
fn p04_adds_to_the_seeded_enforcer() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, ENFORCER_SEED, "social-class/population"),
        170.0,
        "enforcer-seed: population 20 + 150 (ADDITIVE, not a bare 150)"
    );
    assert_eq!(
        attribute(&graph, ENFORCER_SEED, "social-class/wealth"),
        160.0,
        "enforcer-seed: wealth 100.0 + 60.0 (ADDITIVE)"
    );
    assert_eq!(
        attribute(&graph, ENFORCER_SEED, "social-class/active"),
        1.0,
        "enforcer-seed: active flips 0 -> 1"
    );
}

/// p05 is OVERWRITE (decomposition.py:335-336): `ip-seed`'s pre-seeded
/// population (77) and wealth (33.0) are REPLACED by p03's computed
/// amounts (850 / 340.0), never accumulated onto (77 + 850 = 927 would be
/// the additive-mistake value this test rules out).
#[test]
fn p05_overwrites_the_seeded_internal_proletariat() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, IP_SEED, "social-class/population"),
        850.0,
        "ip-seed: population SET to 850 exactly, not 77 + 850 = 927"
    );
    assert_eq!(
        attribute(&graph, IP_SEED, "social-class/wealth"),
        340.0,
        "ip-seed: wealth SET to 340.0 exactly, not 33.0 + 340.0"
    );
    assert_eq!(
        attribute(&graph, IP_SEED, "social-class/active"),
        1.0,
        "ip-seed: active flips 0 -> 1"
    );
}

/// p06 deactivates `la-dying` WITHOUT zeroing it — decomposition.py:339's
/// `graph.update_node(la_id, active=False)` carries no population/wealth
/// keys at all, the non-conservation vector this port transcribes
/// verbatim.
#[test]
fn p06_deactivates_the_la_without_zeroing_it() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/active"),
        0.0,
        "la-dying: active flips 1 -> 0"
    );
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/population"),
        1000.0,
        "la-dying: population UNTOUCHED at its seed, 1000 — never zeroed"
    );
    assert_eq!(
        attribute(&graph, LA_DYING, "social-class/wealth"),
        400.0,
        "la-dying: wealth UNTOUCHED at its seed, 400.0 — never zeroed"
    );
}

/// p06's CLASS_DECOMPOSITION payload, asserted key-by-key against the
/// flattened shape (D-records 4/5 in decomposition.bsl's header): the two
/// frozen nested dicts become four flat keys, `trigger_event`/
/// `narrative_hint` are both dropped (no string payloads on `emit`).
#[test]
fn p06_emits_class_decomposition_with_the_flattened_payload() {
    let (_, sink) = run();
    let decompositions: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "CLASS_DECOMPOSITION")
        .collect();
    assert_eq!(
        decompositions.len(),
        1,
        "exactly one CLASS_DECOMPOSITION this tick"
    );
    let (_, payload) = decompositions[0];
    assert_eq!(payload.len(), 9, "nine flattened keys, no dict, no string");
    assert_eq!(
        payload[0],
        ("source-class".to_owned(), Value::NodeRef(LA_DYING))
    );
    assert_eq!(
        payload[1],
        ("source-population".to_owned(), Value::Real(1000.0))
    );
    assert_eq!(payload[2], ("source-wealth".to_owned(), Value::Real(400.0)));
    assert_eq!(
        payload[3],
        ("enforcer-fraction".to_owned(), Value::Real(0.15))
    );
    assert_eq!(
        payload[4],
        ("proletariat-fraction".to_owned(), Value::Real(0.85))
    );
    assert_eq!(
        payload[5],
        (
            "population-transferred-to-enforcer".to_owned(),
            Value::Real(150.0)
        )
    );
    assert_eq!(
        payload[6],
        (
            "population-transferred-to-proletariat".to_owned(),
            Value::Real(850.0)
        )
    );
    assert_eq!(
        payload[7],
        (
            "wealth-transferred-to-enforcer".to_owned(),
            Value::Real(60.0)
        )
    );
    assert_eq!(
        payload[8],
        (
            "wealth-transferred-to-proletariat".to_owned(),
            Value::Real(340.0)
        )
    );
}

/// The non-participant vector: `bourgeois` is neither LA, CARCERAL_ENFORCER
/// nor INTERNAL_PROLETARIAT — no rule in the whole pack's `when` clause
/// ever matches its role, so its population/wealth/active stay exactly at
/// their seeded values through all six rules.
#[test]
fn the_bourgeois_class_is_untouched_by_the_whole_pack() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, BOURGEOIS, "social-class/population"),
        10.0
    );
    assert_eq!(attribute(&graph, BOURGEOIS, "social-class/wealth"), 9000.0);
    assert_eq!(attribute(&graph, BOURGEOIS, "social-class/active"), 1.0);
}

// ---------------------------------------------------------------------
// Task 4 — the delay-path companion (`decomposition-delay-conformance.
// bscn`): the early warning fires at tick 1, `CLASS_DECOMPOSITION` waits
// out the shipped 52-tick `carceral/decomposition-delay`, firing at tick
// 53 — not tick 1 (unlike the fallback-trigger primary world) and not
// tick 52 (the exact `>=` boundary, decomposition.py:207).
// ---------------------------------------------------------------------

/// The full delay-path lifecycle: `SUPERWAGE_CRISIS` fires exactly once,
/// at tick 1; `CLASS_DECOMPOSITION` fires exactly once, at tick 53; tick 54
/// (one past firing) proves the fire-tick pins and does not re-fire —
/// `p03_is_idempotent_across_two_ticks`'s own precedent, one tick later
/// here since this world's OWN trigger tick is 53, not 1.
#[test]
fn the_delay_path_emits_the_warning_at_tick_1_and_decomposes_at_tick_53() {
    let mut session =
        TickSession::new(DELAY_SCENARIO, RULE, HypergraphStore::new(), test_session())
            .expect("the delay scenario must load into a session");

    let mut sink1 = CollectingSink::default();
    session.advance(&mut sink1).expect("tick 1");
    let crises: Vec<_> = sink1
        .events
        .iter()
        .filter(|(ty, _)| ty == "SUPERWAGE_CRISIS")
        .collect();
    assert_eq!(crises.len(), 1, "exactly one SUPERWAGE_CRISIS at tick 1");
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_CARCERAL_REGISTER,
            "institution/superwage-crisis-tick"
        ),
        1.0,
        "carrier: superwage-crisis-tick == 1"
    );
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_CARCERAL_REGISTER,
            "institution/decomposition-complete"
        ),
        0.0,
        "tick 1: the delay path, not the fallback — must NOT fire yet"
    );

    let mut filler_sink = CollectingSink::default();
    for tick in 2..=52 {
        session.advance(&mut filler_sink).expect("filler tick");
        assert_eq!(
            attribute(
                session.graph(),
                DELAY_CARCERAL_REGISTER,
                "institution/decomposition-complete"
            ),
            0.0,
            "tick {tick}: still waiting out the 52-tick delay"
        );
    }

    let mut sink53 = CollectingSink::default();
    session.advance(&mut sink53).expect("tick 53");
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_CARCERAL_REGISTER,
            "institution/decomposition-fire-tick"
        ),
        53.0,
        "tick 53: 53 >= 1 + 52 — the delay has elapsed"
    );
    let decompositions: Vec<_> = sink53
        .events
        .iter()
        .filter(|(ty, _)| ty == "CLASS_DECOMPOSITION")
        .collect();
    assert_eq!(decompositions.len(), 1, "exactly one CLASS_DECOMPOSITION");

    let mut sink54 = CollectingSink::default();
    session.advance(&mut sink54).expect("tick 54");
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_CARCERAL_REGISTER,
            "institution/decomposition-fire-tick"
        ),
        53.0,
        "tick 54: pinned at 53, decomposition-complete == 1 blocks re-fire"
    );
}

/// The exact boundary (decomposition.py:207's `>=`), isolated in its own
/// session: `tick >= superwage_tick + delay` with `superwage_tick == 1`
/// and `delay == 52` makes the threshold tick 53 — at tick 52 the world
/// must NOT have decomposed yet. Mutation-provable against an off-by-one
/// that fires one tick early (a `delay - 1` slip, or `>=` -> `>` fired the
/// OTHER direction, one tick late — either way this exact-tick snapshot
/// catches it where the lifecycle test's tick-53 check alone would not).
#[test]
fn the_delay_path_does_not_decompose_at_tick_52() {
    let mut session =
        TickSession::new(DELAY_SCENARIO, RULE, HypergraphStore::new(), test_session())
            .expect("the delay scenario must load into a session");
    let mut sink = CollectingSink::default();
    for _ in 1..=52 {
        session.advance(&mut sink).expect("tick");
    }
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_CARCERAL_REGISTER,
            "institution/decomposition-complete"
        ),
        0.0,
        "tick 52: 52 >= 1 + 52 is false — must not have fired"
    );
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_CARCERAL_REGISTER,
            "institution/decomposition-fire-tick"
        ),
        0.0,
        "tick 52: fire-tick still at its seeded 0, never written"
    );
    let decompositions: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "CLASS_DECOMPOSITION")
        .collect();
    assert_eq!(
        decompositions.len(),
        0,
        "tick 52: no CLASS_DECOMPOSITION anywhere across the session"
    );
}

/// Task 4 carried mutation obligation 11 (decomposition.bsl's header):
/// `la-inactive` (active 0, wealth 1 < its own subsistence 500) must
/// contribute exactly 0 to all four of its own census fields, every tick —
/// proving p01's `(= active 1)` conjunct load-bearing (an inactive LA
/// would otherwise flag `dying`, since its wealth sits well below
/// subsistence). `production-conformance.bscn:161-166`'s `worker-la-idle`
/// precedent, applied to the census pack.
#[test]
fn p01_la_inactive_contributes_nothing_to_the_census() {
    let mut session =
        TickSession::new(DELAY_SCENARIO, RULE, HypergraphStore::new(), test_session())
            .expect("the delay scenario must load into a session");
    let mut sink = CollectingSink::default();
    session.advance(&mut sink).expect("tick 1");
    for field in [
        "social-class/la-census-population",
        "social-class/la-census-wealth",
        "social-class/la-approaching-flag",
        "social-class/la-dying-flag",
    ] {
        assert_eq!(
            attribute(session.graph(), DELAY_LA_INACTIVE, field),
            0.0,
            "la-inactive: {field} must stay 0 — the active gate, not the \
             wealth-below-subsistence shape, is what keeps it silent"
        );
    }
    // la-approaching's own dying-flag must independently stay 0 all
    // session — it is the delay-path vector, never the fallback.
    assert_eq!(
        attribute(
            session.graph(),
            DELAY_LA_APPROACHING,
            "social-class/la-dying-flag"
        ),
        0.0,
        "la-approaching: never dying — wealth (515) stays above subsistence (500)"
    );
}

// ---------------------------------------------------------------------
// Task 4 carried mutation obligation 10 (decomposition.bsl's header):
// `p03-trigger`'s `(> la-population 0)` gate. Isolated in its own inline
// scenario — the fold sums EVERY SOCIAL_CLASS node in a world, and both
// scenarios above always carry an active, non-zero-population LA, so this
// property cannot be witnessed inside either of them.
// ---------------------------------------------------------------------

/// A single LA node, seeded with population 0 (so its own census
/// contribution is 0 regardless of role/active — p01's `census-population`
/// binding reads `population` verbatim) and a DIRECTLY SEEDED carrier
/// crisis latch (`superwage-crisis-known 1`, `superwage-crisis-tick 0`),
/// bypassing p02 entirely — a population-0 LA's own approaching/dying
/// flags can never go non-zero (both gate on `population > 0`), so p02
/// would never emit `SUPERWAGE_CRISIS` from this world on its own; the
/// carrier's own delay gate would otherwise stay permanently unreachable
/// without this seed. `carceral/decomposition-delay` is companion-varied
/// to 1 (a "vary a delay/fraction to make a branch reachable at tick 1"
/// companion, the dispossession precedent the plan's Global Constraints
/// name) — the seeded tick 0 plus delay 1 makes `delay-elapsed-fire`
/// (`tick >= superwage-crisis-tick + decomposition-delay`) TRUE at tick 1
/// (`1 >= 0 + 1`). With `decomposition-complete == 0` and the OR-group
/// true, `(> la-population 0)` is the ONLY thing standing between this
/// world and an incorrect fire on a population-0 LA — exactly the
/// property Task 3's own mutation evidence found unwitnessed.
#[test]
fn p03_gate_blocks_zero_population_la_even_when_the_delay_has_elapsed() {
    const ZERO_POP_SCENARIO: &str = r#"
(scenario decomposition/zero-population-gate
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int extensive)
  (deffield social-class/population int extensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/subsistence-threshold real extensive)
  (deffield social-class/s-bio real extensive)
  (deffield social-class/s-class real extensive)
  (deffield social-class/la-census-population int extensive)
  (deffield social-class/la-census-wealth real extensive)
  (deffield social-class/la-approaching-flag int extensive)
  (deffield social-class/la-dying-flag int extensive)

  (deffield institution/superwage-crisis-known int extensive)
  (deffield institution/superwage-crisis-tick int extensive)
  (deffield institution/decomposition-complete int extensive)
  (deffield institution/decomposition-fired-known int extensive)
  (deffield institution/decomposition-fire-tick int extensive)
  (deffield institution/la-population int extensive)
  (deffield institution/la-wealth real extensive)
  (deffield institution/la-approaching-count int extensive)
  (deffield institution/la-dying-count int extensive)
  (deffield institution/enforcer-pop-gain int extensive)
  (deffield institution/enforcer-wealth-gain real extensive)
  (deffield institution/ip-population int extensive)
  (deffield institution/ip-wealth real extensive)

  (defconst carceral/enforcer-fraction 0.15c)
  (defconst carceral/proletariat-fraction 0.85c)
  ; Companion-varied (Global Constraints: "MAY vary a delay/fraction to
  ; make a branch reachable at tick 1") — paired with the carrier's own
  ; SEEDED superwage-crisis-tick 0 below, this makes delay-elapsed-fire
  ; true from tick 1 (1 >= 0 + 1) without waiting out a real delay window.
  (defconst carceral/decomposition-delay 1)
  (defconst carceral/approaching-consumption-multiple 2)

  (node la-zero-pop NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/LABOR_ARISTOCRACY)
    (social-class/active 1)
    (social-class/population 0)
    (social-class/wealth 999)
    (social-class/subsistence-threshold 500)
    (social-class/s-bio 5)
    (social-class/s-class 5)
    (social-class/la-census-population 0)
    (social-class/la-census-wealth 0)
    (social-class/la-approaching-flag 0)
    (social-class/la-dying-flag 0))

  (node carceral-register NodeType/INSTITUTION
    (institution/superwage-crisis-known 1)
    (institution/superwage-crisis-tick 0)
    (institution/decomposition-complete 0)
    (institution/decomposition-fired-known 0)
    (institution/decomposition-fire-tick 0)
    (institution/la-population 0)
    (institution/la-wealth 0)
    (institution/la-approaching-count 0)
    (institution/la-dying-count 0)
    (institution/enforcer-pop-gain 0)
    (institution/enforcer-wealth-gain 0)
    (institution/ip-population 0)
    (institution/ip-wealth 0)))
"#;
    const ZERO_POP_CARCERAL_REGISTER: NodeId = NodeId(1);
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(ZERO_POP_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the zero-population world must run");
    assert_eq!(
        attribute(
            &graph,
            ZERO_POP_CARCERAL_REGISTER,
            "institution/la-population"
        ),
        0.0,
        "the fold over the one zero-population LA must itself be 0"
    );
    assert_eq!(
        attribute(
            &graph,
            ZERO_POP_CARCERAL_REGISTER,
            "institution/decomposition-fire-tick"
        ),
        0.0,
        "must NOT fire — (> la-population 0) blocks it even though the \
         delay has elapsed and decomposition-complete == 0"
    );
    assert_eq!(
        attribute(
            &graph,
            ZERO_POP_CARCERAL_REGISTER,
            "institution/decomposition-complete"
        ),
        0.0,
        "must NOT be marked complete"
    );
}
