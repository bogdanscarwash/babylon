//! The four Territory-shaped end-to-end query-lane vectors (BSL
//! query-evaluation plan, `docs/superpowers/plans/2026-08-11-bsl-query-
//! evaluation-plan.md`, ⟨PR 5⟩ Task 15).
//!
//! **This ships no Territory content.** It proves the four shapes
//! `reports/territory-port-phase1-inventory-2026-08-11.md` §6's blocker
//! table named — sink selection with a tie, population transfer against a
//! computed reference, heat spillover reading pre-tick neighbour state, and
//! `for-each`-driven organization suppression — are now expressible and
//! CORRECT through the real production entry point
//! (`babylon_tick::run_once_into`, the same seam the CLI driver and
//! `babylon-client`'s engine link both call), using a synthetic, hand-built
//! fixture (`content/scenarios/query-lane-e2e.bscn`) rather than real
//! Territory data — the same posture Metabolism's own fixtures took before
//! Metabolism had a canonical scenario.
//!
//! Each shape below runs as its OWN single-rule content set, loaded fresh
//! from the ONE shared scenario file each time (`run_once_into` loads the
//! scenario into a NEW graph per call) — so no shape's tick can observe
//! another shape's writes, and the cross-RULE pre-state gap (D-row **Q14**,
//! `bsl-language.rst`'s register — `run_once_into`/`TickSession::advance`
//! run each rule in a content set to completion before the next starts, so
//! TWO rules at the same anchor position do not share pre-state) never
//! applies here: nothing in this file ever loads more than one `(rule …)`
//! form in the same `rule_src`.
//!
//! **Why every rule declares `territory/shape`.** `run_tick`'s subject loop
//! (`tick.rs::run_tick`) walks EVERY node of a rule's derived subject type,
//! and `bind_subject` reads every declared `:field` binding for every one
//! of them BEFORE the guard runs — so a rule sharing the TERRITORY subject
//! type with three other shapes' nodes needs a way to skip the ones it does
//! not own. `territory/shape` is that discriminator (a fixture-only field,
//! not a Territory-port one); see the scenario file's own header for the
//! full reasoning, including why it is the ONLY field seeded on every
//! territory node while `heat`/`priority`/`population` are seeded only
//! where a shape's own guarded query or write path reaches.
//!
//! **A landed-code finding, recorded rather than fixed (out of this task's
//! scope).** `rule_pipeline::resolve_expr_bindings` still unconditionally
//! passes `graph: None` to the `EvalEnv` it builds for a `:expr` binding
//! (its own doc comment names this "P6", flagged during the PR #514 fix
//! round, waiting on "the same collect-then-apply repair" as `tick.rs`'s
//! guard/effects environment) — but Task 12 (that repair) landed in group 3
//! and this callsite was not updated alongside it. A `:expr` binding
//! containing a query form (`fold`/`neighbors`/`select-max`/`exists`/…)
//! therefore still fails loud today ("needs the graph substrate … this
//! EvalEnv carries none") through the REAL `run_once_into` driver, even
//! though the underlying evaluator serves every one of those heads. Every
//! rule below routes around this by writing its query forms directly
//! inline inside a `(when …)` guard or an `(effects …)` operand — both of
//! which DO receive the real, graph-carrying `env` (`tick.rs::run_tick`
//! constructs it after `resolve_expr_bindings` returns) — never inside an
//! `:expr` binding. This is a real, narrow gap the next slice touching
//! `:expr` bindings should close; Task 16's handoff record names it.
//!
//! **RIDER (Territory port train, fix round, 2026-08-12) — CLOSED.** The
//! gap this note describes is fixed: `resolve_expr_bindings` now takes a
//! `graph: Option<&dyn GraphSubstrate>` parameter, threaded alongside the
//! already-threaded `types`/`enums` registries (never alone), and
//! `tick.rs::collect_pass` passes `Some(graph)` from its own live Pass-1
//! borrow. `territory/p3-spillover`'s `inflow` binding
//! (`content/rules/territory.bsl`) is the first `:expr` body containing a
//! query form (`exists`+`fold`+`field-of`) to clear the real
//! `run_once_into` driver end to end. Filed as register row D130
//! (`docs/reference/bsl-language.rst`). This file's four vectors are
//! untouched and still valid as written — none of them needed the fix,
//! since each routes its own query forms through a guard/effect operand
//! by design, not through a `:expr` binding.
//!
//! # Provenance
//!
//! Every expected numeric value below is DERIVED in this file's own
//! comments from `src/babylon/data/defines.yaml`'s coefficients
//! (`heat_spillover_rate: 0.05`, `displacement_rate: 0.1`) and the
//! scenario's own seeded literals, using the SAME IEEE-754 binary64
//! arithmetic Rust and Python both perform on basic ops (`+ − × ÷`,
//! correctly rounded) — never copied from this crate's own printed output.
//! Where a value is not a "clean" finite decimal, its exact bit pattern is
//! pinned via `to_bits()` rather than a tolerance comparison (CLAUDE.md's
//! Tests-as-Behavioral-Contracts principle 4: basic IEEE-754 ops reproduce
//! bit-exactly across languages, so a value independently computed in
//! Python and asserted here as an exact `f64` literal is not a guess).

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/query-lane-e2e.bscn");

// Node ids, fixed by the scenario's own declaration order (scenario.rs's
// "declaration order is the id order" contract) — see the scenario file's
// header comment for the full id map.
const T0: NodeId = NodeId(0);
const T1: NodeId = NodeId(1);
const T2: NodeId = NodeId(2);
const T3: NodeId = NodeId(3);
#[allow(dead_code)] // named for documentation symmetry with the id map above
const SOURCE_B: NodeId = NodeId(4);
const SINK_A: NodeId = NodeId(5);
const SINK_B: NodeId = NodeId(6);
const ISOLATED_C: NodeId = NodeId(7);
#[allow(dead_code)] // named for documentation symmetry with the id map above
const PENAL_COLONY_D: NodeId = NodeId(8);
const TENANT_1: NodeId = NodeId(9);
const TENANT_2: NodeId = NodeId(10);
const NON_TENANT: NodeId = NodeId(11);

// ============================================================ Shape A

/// Heat spillover: `(fold sum (neighbors self EdgeType/ADJACENCY :any
/// NodeType/TERRITORY) (field-of it territory/heat))`, over the 4-territory
/// chain `t0 - t1 - t2 - t3` — Territory blocker table row 3
/// (`_process_spillover`, `territory.py:269-316`).
///
/// **The pre-state proof.** All four subjects fire under ONE rule, in
/// ascending id order (`t0, t1, t2, t3`). Under the §4.2 chapter C4 law
/// (Task 12, landed group 3), every firing's fold reads the SAME pre-tick
/// heat values — so `t1`'s spillover reads `t0`'s ORIGINAL 0.25, never the
/// 0.275 `t0`'s OWN firing (earlier in subject order) just wrote. If
/// `run_tick` still mutated in place (the pre-Task-12 divergence, D-row Q1),
/// `t1` would read `t0`'s ALREADY-SPILLED heat instead, and every assertion
/// below would need a different, order-dependent number. This is the
/// end-to-end analogue of `tick.rs`'s own unit-level pre-state tests, run
/// through the REAL scenario/rule/driver seam rather than a hand-built
/// `EvalEnv`.
///
/// # Derivation
///
/// `rate = 5.0 / 100.0` (the `0.05c` defconst's own conversion contract) —
/// not exact in binary64; every number below carries the real rounding.
///
/// Seeded heats (each an exact binary64 value — `1/4`, `1/2`, `1/8`, `3/8`):
/// `heat(t0)=0.25, heat(t1)=0.5, heat(t2)=0.125, heat(t3)=0.375`.
///
/// `neighbors(tk, ADJACENCY, :any)` ascending-id order (`nodes_materializes_
/// in_ascending_id_order`'s own guarantee) gives, per subject:
/// - `t0`: `{t1}` → `inflow = 0.5`
/// - `t1`: `{t0, t2}` → `inflow = 0.25 + 0.125 = 0.375` (exact)
/// - `t2`: `{t1, t3}` → `inflow = 0.5 + 0.375 = 0.875` (exact)
/// - `t3`: `{t2}` → `inflow = 0.125`
///
/// `new_heat = heat + inflow * rate`, each term computed independently in
/// Python (`repr()`/`struct.pack('>d', …)`, `python3` at the time of
/// writing) and pinned here as the exact bits:
/// - `t0`: `0.25 + 0.5*0.05 = 0.275` (`0x3fd199999999999a`)
/// - `t1`: `0.5 + 0.375*0.05 = 0.51875` (`0x3fe099999999999a`)
/// - `t2`: `0.125 + 0.875*0.05 = 0.16875` (`0x3fc599999999999a`)
/// - `t3`: `0.375 + 0.125*0.05 = 0.38125` (`0x3fd8666666666666`)
///
/// All four are comfortably under the frozen system's `min(1.0, …)` ceiling
/// (`territory.py:315`), so this vector never needs to express that clamp —
/// a deliberate fixture choice, not an omission (this task ships no
/// Territory content, and the clamp needs no query-lane capability this
/// plan serves).
const RULE_SPILLOVER: &str = r#"
(rule territory/spillover-e2e
  :role mechanic :evidence derived :material-basis "heat spillover via a pull-side fold reading pre-tick neighbour heat — Territory blocker table row 3 (_process_spillover, territory.py:269-316); proves the section-4.2 chapter-C4 pre-state law end to end through the real run_once_into seam"
  :fuel 256
  (bindings
    (binding shape :field territory/shape)
    (binding rate :const territory/heat-spillover-rate))
  (when (= shape 0))
  (effects
    (update-node self territory/heat
      (add (* (fold sum (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY)
                    (field-of it territory/heat))
              rate)))))
"#;

#[test]
fn shape_a_heat_spillover_reads_pre_tick_neighbour_state() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_SPILLOVER, &mut graph, &mut sink)
        .expect("the spillover rule must load and run through run_once_into");
    assert_eq!(
        report.fired, 4,
        "exactly the four shape=0 chain territories fire"
    );

    let heat = |id: NodeId| graph.node_attribute(id, "territory/heat").unwrap();
    assert_eq!(
        heat(T0).to_bits(),
        0.275_f64.to_bits(),
        "t0: 0.25 + 0.5*0.05 = 0.275, read from t1's PRE-tick heat"
    );
    assert_eq!(
        heat(T1).to_bits(),
        0.51875_f64.to_bits(),
        "t1: 0.5 + (0.25+0.125)*0.05 = 0.51875, reading t0's and t2's PRE-tick \
         heat — NOT t0's already-updated 0.275"
    );
    assert_eq!(
        heat(T2).to_bits(),
        0.16875_f64.to_bits(),
        "t2: 0.125 + (0.5+0.375)*0.05 = 0.16875"
    );
    assert_eq!(
        heat(T3).to_bits(),
        0.38125_f64.to_bits(),
        "t3: 0.375 + 0.125*0.05 = 0.38125, reading t2's PRE-tick heat"
    );
}

// ============================================================ Shape B

/// Priority sink selection with a tie, plus the population transfer it
/// feeds: `(select-max (neighbors self EdgeType/ADJACENCY :out
/// NodeType/TERRITORY) (field-of it territory/priority))`, guarded by
/// `exists` (Task 6's Territory `_find_sink_node` shape), then `update-node`
/// against the COMPUTED reference — Territory blocker table rows 1-2
/// (`_find_sink_node`, `territory.py:139-194`; the population transfer,
/// `territory.py:259-267`).
///
/// `source-b` has two ADJACENCY `:out` neighbours, `sink-a` and `sink-b`,
/// scored EQUAL (`territory/priority = 5` on both). §2.7's tiebreak (D45)
/// says the FIRST element in ascending id byte order wins for `select-max`
/// — `sink-a` (id 5) was declared before `sink-b` (id 6), so `sink-a` must
/// be the one written. The frozen `_find_sink_node`
/// (`territory.py:166-193`) carries its OWN mode-ordered tiebreak
/// (`_PRIORITY_BY_MODE`, a fixed type-priority list scanned in order) — a
/// DIFFERENT tiebreak rule from this language's "lowest id wins"; the
/// Territory port's own D-record owes a comparison between the two, and
/// this vector is the evidence that comparison will need.
///
/// # Derivation
///
/// `rate = 1.0 / 10.0` (the `0.1c` defconst). `source-b`'s own population is
/// seeded `100` (an exact `int`). `transfer = 100.0 * 0.1`; computed
/// independently in Python: `100 * 0.1 == 10.0` exactly (the rounding in
/// `0.1`'s own binary64 representation happens to cancel against `100`'s
/// power-of-2 factor here) — `sink_a.population = 0 + 10.0 = 10.0`.
///
/// `sink-b`'s `territory/population` is never seeded in the scenario, so a
/// post-tick read of it failing with "never written" is stronger proof that
/// it was NOT selected than a numeric "still 0" comparison would be — a
/// bug that wrote the wrong constant to the wrong node would still read as
/// "still 0" if `sink-b` happened to start at 0, but a bug that wrote to
/// `sink-b` at all is caught unconditionally by the read failing to fail.
const RULE_SINK_SELECTION: &str = r#"
(rule territory/sink-selection-tiebreak-e2e
  :role mechanic :evidence derived :material-basis "priority sink selection with the section-2.7 language-level tiebreak, guarded by exists (Task 6), feeding update-node against the computed reference — Territory blocker table rows 1-2 (_find_sink_node, territory.py:139-194; the population transfer, territory.py:259-267)"
  :fuel 256
  (bindings
    (binding shape :field territory/shape)
    (binding rate :const territory/displacement-rate))
  (when (= shape 1))
  (effects
    (update-node
      (if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY) #t)
          (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                      (field-of it territory/priority))
          self)
      territory/population
      (add (* (field-of self territory/population) rate)))))
"#;

#[test]
fn shape_b_priority_sink_selection_with_a_tie() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_SINK_SELECTION, &mut graph, &mut sink)
        .expect("the sink-selection rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only source-b (shape=1) fires");

    let sink_a_population = graph
        .node_attribute(SINK_A, "territory/population")
        .unwrap();
    assert_eq!(
        sink_a_population, 10.0,
        "sink-a (the lower-id, tied-priority sink) received the transfer: \
         0 + 100*0.1 = 10.0"
    );

    let sink_b_result = graph.node_attribute(SINK_B, "territory/population");
    assert!(
        sink_b_result.is_err(),
        "sink-b was never selected, so its never-seeded population field \
         must still read as unwritten (III.11) — a write to the wrong node \
         would make this read succeed: {sink_b_result:?}"
    );
}

// ============================================================ Shape C

/// The `exists`-guarded fallback over a territory with NO `ADJACENCY`
/// neighbour — the requirement the plan's own intro derives from
/// `_process_eviction_pipeline`'s `sink_id is None` case ("exists earns its
/// place, and slice 1 ships it"): without the guard, `select-max` over the
/// empty query is `E-EVAL-021` (§4.4/D45), a TICK ABORT, where the frozen
/// engine just skips the transfer.
///
/// `isolated-c` (shape=3) carries zero `ADJACENCY` edges. Its rule reuses
/// the SAME `if`/`exists`/`select-max`/fallback shape Shape B's rule does,
/// scored the same way — the only difference is the topology it runs
/// against — so this vector is proof that the identical, reusable pattern
/// takes the FALLBACK branch (`self`) rather than erroring, and that the
/// selected reference really is the fallback: `update-node`'s target
/// resolves to `self`, so `isolated-c`'s OWN `territory/priority` (never
/// seeded) ends the tick holding the literal the effect sets — `1` — which
/// could only happen if the target really was `isolated-c` itself.
const RULE_FALLBACK: &str = r#"
(rule territory/fallback-no-sink-e2e
  :role mechanic :evidence derived :material-basis "the exists-guarded selection's fallback branch, never E-EVAL-021, when a territory has no ADJACENCY neighbour — the plan intro's exists requirement, over _process_eviction_pipeline's sink_id-is-None case"
  :fuel 128
  (bindings
    (binding shape :field territory/shape))
  (when (= shape 3))
  (effects
    (update-node
      (if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY) #t)
          (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                      (field-of it territory/priority))
          self)
      territory/priority
      (set 1))))
"#;

#[test]
fn shape_c_empty_neighbourhood_takes_the_fallback_branch_never_e_eval_021() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_FALLBACK, &mut graph, &mut sink).expect(
        "an empty ADJACENCY neighbourhood must take the fallback branch, \
         never raise E-EVAL-021 — a returned Err here would be exactly \
         that failure",
    );
    assert_eq!(report.fired, 1, "only isolated-c (shape=3) fires");

    let priority = graph
        .node_attribute(ISOLATED_C, "territory/priority")
        .unwrap();
    assert_eq!(
        priority, 1.0,
        "the update-node target resolved to self (isolated-c) via the \
         fallback branch — the ONLY way isolated-c's own never-seeded \
         priority field ends the tick holding the literal the effect sets"
    );
}

// ============================================================ Shape D

/// `PENAL_COLONY` organization suppression via `for-each` writing the
/// SOURCE node — Territory blocker table row 4 (`_suppress_organization`,
/// `territory.py:353-378`): `(for-each (neighbors self EdgeType/TENANCY :in
/// NodeType/SOCIAL_CLASS) (update-node it social-class/organization (set
/// 0)))`.
///
/// `penal-colony-d` (shape=4) has two `TENANCY :in` neighbours, `tenant-1`
/// and `tenant-2`, both seeded `social-class/organization = 1`; `non-tenant`
/// carries no `TENANCY` edge to it at all — the frozen law
/// `test_social_class_without_tenancy_edge_is_untouched` transcribed
/// directly: every tenant zeroed, the non-tenant left alone.
const RULE_PENAL_COLONY: &str = r#"
(rule territory/penal-colony-suppression-e2e
  :role mechanic :evidence derived :material-basis "PENAL_COLONY organization suppression via for-each writing the source node's TENANCY :in neighbours — Territory blocker table row 4 (_suppress_organization, territory.py:353-378)"
  :fuel 128
  (bindings
    (binding shape :field territory/shape))
  (when (= shape 4))
  (effects
    (for-each (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)
      (update-node it social-class/organization (set 0)))))
"#;

#[test]
fn shape_d_penal_colony_suppression_writes_only_tenant_classes() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_PENAL_COLONY, &mut graph, &mut sink)
        .expect("the penal-colony rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only penal-colony-d (shape=4) fires");

    let organization = |id: NodeId| {
        graph
            .node_attribute(id, "social-class/organization")
            .unwrap()
    };
    assert_eq!(organization(TENANT_1), 0.0, "tenant-1: zeroed via TENANCY");
    assert_eq!(organization(TENANT_2), 0.0, "tenant-2: zeroed via TENANCY");
    assert_eq!(
        organization(NON_TENANT),
        1.0,
        "non-tenant carries no TENANCY edge to penal-colony-d and must be \
         left exactly as seeded — test_social_class_without_tenancy_edge_\
         is_untouched, transcribed"
    );
}

// ============================================================ Determinism

/// §6.2 family 8: every shape's tick, run twice in one process against
/// independently-loaded graphs, must produce byte-identical `TickReport`s —
/// not merely equal final attributes, the full report (hash, fired count).
/// "Once in a fresh process" is proven by running this same binary via
/// `cargo test` a second time (a fresh process by construction); nothing in
/// these rules or this scenario reads process-local state (no wall clock, no
/// `HashMap` iteration on a result path — Constraint 2), so the two legs
/// are not expected to differ and do not.
#[test]
fn every_shape_is_deterministic_across_two_independent_runs() {
    for (rule, label) in [
        (RULE_SPILLOVER, "spillover"),
        (RULE_SINK_SELECTION, "sink-selection"),
        (RULE_FALLBACK, "fallback"),
        (RULE_PENAL_COLONY, "penal-colony"),
    ] {
        let mut graph_a = HypergraphStore::new();
        let mut sink_a = CollectingSink::default();
        let report_a = run_once_into(SCENARIO, rule, &mut graph_a, &mut sink_a)
            .unwrap_or_else(|e| panic!("{label}: first run: {e}"));

        let mut graph_b = HypergraphStore::new();
        let mut sink_b = CollectingSink::default();
        let report_b = run_once_into(SCENARIO, rule, &mut graph_b, &mut sink_b)
            .unwrap_or_else(|e| panic!("{label}: second run: {e}"));

        assert_eq!(report_a.before, report_b.before, "{label}: before hash");
        assert_eq!(report_a.after, report_b.after, "{label}: after hash");
        assert_eq!(report_a.fired, report_b.fired, "{label}: fired count");
        assert_eq!(
            report_a.per_rule_fired, report_b.per_rule_fired,
            "{label}: per_rule_fired"
        );
    }
}
