//! Registration + scenario-ceremony conformance for the ImperialRent BSL
//! port train (Material Base @9.0, Checkpoint A campaign) — Task 1 of
//! `docs/superpowers/plans/2026-08-18-imperialrent-port.md` (frozen source:
//! `src/babylon/engine/systems/economic.py::ImperialRentSystem`, `step()` at
//! `:46-86`, 837 lines total).
//!
//! # Task 1 scope
//!
//! This task ships NO `imperial-rent/*` rule pack content (that is Task 2
//! onward) — it registers the system in `lib.rs`, builds the shared world-1
//! conformance scenario (`content/scenarios/imperial-rent-conformance.bscn`),
//! carries the frozen-mirror provenance below, and records THE SPIKE
//! verdict (plan §8's Task 1 Step 5) that later tasks depend on.
//!
//! Where this train's own dossier
//! (`reports/imperial-rent-bsl-surface-facts-2026-08-18.md`) and the task
//! brief disagree, the dossier governs — notably: B7 is STRUCK (no
//! `social-class/class-consciousness` field; the frozen accessor already
//! re-points to `social-class/revolutionary`, matching `solidarity.bsl`'s
//! own identical re-point), and this train's ADR number is
//! NEXT-FREE-AT-LANDING (ADR215 as of the dossier's 2026-08-18 measurement,
//! re-checked whenever an ADR actually files).
//!
//! # Step 1 red-phase note
//!
//! `scenario_and_empty_pack_load` (below) was written and run BEFORE
//! `"imperial-rent"` was added to `lib.rs`'s registered-system `HashSet`
//! (verified by temporarily reverting the registration and re-running this
//! one test); it failed with:
//!
//! ```text
//! rule imperial-rent/probe rejected: E-LOAD-002: rule imperial-rent/probe
//! carries no anchor and its first id segment names no registered system —
//! a rule cannot land nowhere (§2.3)
//! ```
//!
//! — confirming the probe reaches the real anchor check
//! (`mod_anchors::check_anchor` against `ctx.systems`, `rule_pipeline.rs:313`)
//! rather than failing for an unrelated reason (the scenario's own
//! declarations loaded clean). Registering the system turned it green. Same
//! deviation from the plan's literal "empty rule source" text that
//! `production_conformance.rs:75-94` and `decomposition_conformance.rs`'s
//! own Step 1 note already record: `run_once_into`'s own `split_content`
//! refuses a content set with zero `(rule …)` top-forms outright, so a truly
//! empty source cannot reach the anchor check at all; this test uses one
//! minimal, never-firing probe rule instead (only ONE new system this train
//! registers, unlike the Decomposition+ControlRatio train's two).
//!
//! # THE SPIKE (Step 5) — verdict
//!
//! Two throwaway spike rules were run through the real `run_once_into`
//! driver, then deleted (their bodies never landed in this file or in
//! `content/rules/`):
//!
//! - **Spike 1** (against `SCENARIO`, this file's own primary world) proved
//!   shapes (a)-(e) and (g) in one rule anchored on `social-class/active`
//!   (subject type `SOCIAL_CLASS`, inferred from the first `:field`
//!   binding's namespace, `tick.rs::subject_type_of`):
//!   - **(a)** `(for-each (neighbors self EdgeType/EXPLOITATION :out
//!     NodeType/SOCIAL_CLASS) …)` fired for `periphery-worker` (the sole
//!     EXPLOITATION source) and iterated zero times for the other four
//!     classes (D127 hash-neutral) — LOADED AND EVALUATED CLEAN.
//!   - **(b)** `(update-edge (edge-between EdgeType/EXPLOITATION self it)
//!     exploitation/value-flow (set 1))` wrote the `.bscn`-seeded
//!     `exploitation/value-flow` edge attribute — post-tick read back `1.0`.
//!   - **(c)** the same body issued BOTH `(update-node self
//!     social-class/production-value (add 1))` and `(update-node it
//!     social-class/production-value (add 1))` — both writes observed
//!     applying (confirmed by the final aggregate values below, which
//!     include their contribution).
//!   - **(d)** a nested `(guard (= (field-of it social-class/role)
//!     SocialRole/CORE_BOURGEOISIE) (emit …) (update-node it … (add 10))
//!     (update-node it … (add 10)))` — THREE effects — loaded and ran, all
//!     three firing together (the guard's predicate held for
//!     `core-bourgeoisie`, the sole EXPLOITATION target; exactly one
//!     `SURPLUS_EXTRACTION` event observed).
//!   - **(e)** the `(field-of it social-class/role)` enum-ref equality
//!     inside that guard typechecked and evaluated correctly.
//!   - **(g)** a second `for-each` over `(nodes NodeType/SOCIAL_CLASS)` (5
//!     elements) issued a repeated, `it`-independent `(update-node self
//!     social-class/production-value (set 42))` alongside a repeated
//!     `(update-node self social-class/wealth (add 1))`. Measured post-tick
//!     values (`run_once_into`, this exact rule + `SCENARIO`, 2026-08-18):
//!     `periphery-worker`/`comprador`/`labor-aristocracy`/`petty-b` all read
//!     `production-value=42.0` (the repeated `set` accepted, never refused,
//!     last write wins — all five were identical so "last" is
//!     unobservable by value alone, but no collision error occurred) and
//!     `wealth=seed+5.0` (`805.0`/`305.0`/`255.0` — the `add` DID
//!     accumulate across all 5 iterations, unlike `set`).
//!     `core-bourgeoisie` (subject-order FIRST, `tick.rs`'s "subject order
//!     outer, source order inner") reads `production-value=53.0` — its OWN
//!     `for-each (nodes …)` batch (5x `set 42`) applied FIRST (it is the
//!     lowest-id subject), THEN `periphery-worker`'s LATER `it`-targeted
//!     `(add 1)` and the guard's `(add 10)` applied ON TOP of the
//!     already-42 value (`42 + 1 + 10 = 53`) — the cross-subject
//!     accumulation-onto-a-prior-set the dossier's `structural_verbs.rs`
//!     reading predicted, now observed for real. `core-bourgeoisie`'s
//!     `wealth=10015.0` (`10000 + 10` [guard] `+ 5` [own for-each's 5x
//!     `add 1`]) confirms the same batch, no contribution lost. Exactly
//!     Task 0 Step 4(e)'s static reading, now proven against the real
//!     driver, with no surprise.
//! - **Spike 2** (against a small inline two-INSTITUTION-node fixture, NOT
//!   `SCENARIO` — this scenario mints exactly one INSTITUTION node by
//!   design) proved **(f)**: `(select-max (nodes NodeType/INSTITUTION)
//!   (field-of it institution/rent-carrier))` resolved the real carrier
//!   (`rent-carrier 1`) over a LOWER-id decoy (`rent-carrier 0`) — proving
//!   the discriminator, not ascending-id tiebreak, decides the winner. This
//!   shape is kept as the PERMANENT
//!   `carrier_discriminator_resolves_over_a_lower_id_decoy` test below
//!   (Step 6), not deleted with the rest of the spike.
//!
//! **No shape refused.** Task 2 onward may rely on all seven directly,
//! exactly as the plan's §3.1/§8 disposition specifies.
//!
//! # THE THREE FIRSTS (Step 6) — fix round 1, reviewer Important 2
//!
//! Pinned here, by name, per the brief's own Step 6 instruction (the
//! per-test doc on `carrier_discriminator_resolves_over_a_lower_id_decoy`
//! below states the THIRD of these in its own words; this is the one place
//! all three are named together):
//!
//! 1. **This is the first BSL content — of any kind, throwaway spike or
//!    landed pack — to use `update-edge`.** Spike 1 (item (b) above) is the
//!    first-ever content-adjacent `update-edge` invocation in this repo's
//!    history; the landed `content/rules/*.bsl` estate uses only
//!    `update-node` before this train. (`update-edge` itself landed in the
//!    language at T3/ADR198 — this is its first CONTENT-side use.)
//! 2. **`wages/value-flow`'s first WRITE is deferred to Task 5's `r06` —
//!    NOT claimed here.** This pack's own edges seed `wages/value-flow`
//!    (`.bscn`'s `edge-attr`, a landed idiom since `consciousness.bsl`'s
//!    `p2-wages-push` reader), but Task 1 writes only `exploitation/
//!    value-flow` (Spike 1, item (b)) — a brand-new namespace, not the
//!    reused `wages/*` one. No content pack writes `wages/value-flow` as of
//!    this commit.
//! 3. **This is the first carrier resolved by a declared discriminator
//!    (`institution/rent-carrier`) rather than a constant score.** Every
//!    landed carrier read before this train (`decomposition.bsl`'s 14
//!    `(select-max (nodes NodeType/INSTITUTION) 1)` sites) uses a CONSTANT
//!    score — D45's ascending-id tiebreak decides the winner by construction
//!    whenever more than one INSTITUTION node exists. THIS pack is the
//!    first to score by a per-node FIELD instead
//!    (`carrier_discriminator_resolves_over_a_lower_id_decoy` proves the
//!    two selection strategies diverge on a real fixture, not merely that
//!    the new syntax loads).
//!
//! # Frozen-mirror provenance
//!
//! Every state value below was printed by the frozen `ImperialRentSystem`
//! (@9.0), one tick of its five phases (`step()`'s own body, replicated
//! call-for-call — see the mirror's own fix-round-1 comment — so the
//! pre-quantization pool value is observable), over a fixture that mirrors
//! `imperial-rent-conformance.bscn` node for node and edge for edge — §9's
//! canonical mirror recipe (all three graph attributes seeded explicitly,
//! `persistent_data` left `{}`, no boundary register bound).
//!
//! **Header fact (d), fix round 1 (reviewer Important 1): the printed
//! `economy` graph attribute is QUANTIZED; BSL's own arithmetic is not.**
//! `GlobalEconomy`'s three fields each carry Pydantic's `SnapToGrid`
//! validator (`models/types.py:26-30,41-44`; `kernel/math.py:41-56`,
//! ROUND_HALF_UP to 6 decimals), and `_save_economy` (`economic.py:831-836`)
//! constructs a `GlobalEconomy(...)` on every save — a frozen-Python-ONLY
//! artifact with no BSL counterpart. The mirror below therefore prints BOTH
//! the quantized `economy` dict (what the frozen engine actually stores)
//! AND a separate RAW, pre-quantization `imperial_rent_pool` line, measured
//! (not hand-derived) from the same `tick_context` dict `_save_economy`
//! itself reads. **The RAW line, not the quantized dict's
//! `imperial_rent_pool`, is the correct oracle for a future `r09-pool-
//! decay` BSL comparison** — BSL has no Currency-quantization step, so its
//! own raw binary64 output will match the RAW print exactly and differ from
//! the quantized one by ~1.28e-7 in this world. Node `wealth`/
//! `effective_wealth`/etc. and edge `value_flow` are never quantized
//! (`graph.update_node`/`update_edge` write raw dict attributes, no
//! Pydantic validation on that path) — visible directly in the stdout below
//! as six-decimal `economy` values beside 15-digit `wealth` values.
//!
//! ```text
//! PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run python \
//!     rust/crates/babylon-tick/content/scenarios/imperial_rent_conformance.py
//! ```
//!
//! Its output on 2026-08-18, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml, economy: section):
//!   economy.extraction_efficiency = 0.8
//!   economy.comprador_cut = 0.9
//!   economy.super_wage_rate = 0.2
//!   economy.superwage_multiplier = 1.0
//!   economy.superwage_ppp_impact = 0.5
//!   economy.initial_rent_pool = 100.0
//!   economy.pool_high_threshold = 0.7
//!   economy.pool_low_threshold = 0.3
//!   economy.pool_critical_threshold = 0.1
//!   economy.min_wage_rate = 0.05
//!   economy.max_wage_rate = 0.35
//!   economy.negligible_rent = 0.01
//!   economy.trpf_coefficient = 0.0005
//!   economy.rent_pool_decay = 0.002
//!   economy.bribery_wage_delta = 0.05
//!   economy.austerity_wage_delta = -0.05
//!   economy.iron_fist_repression_delta = 0.1
//!   economy.crisis_wage_delta = -0.15
//!   economy.crisis_repression_delta = 0.2
//!   economy.bribery_tension_threshold = 0.7
//!   economy.iron_fist_tension_threshold = 0.5
//!   economy.trpf_efficiency_floor = 0.1
//!   timescale.weeks_per_year = 52
//!
//! context.persistent_data (pre-tick) = {}
//! services.boundary_register = None
//!
//! post-tick social classes:
//!   core-bourgeoisie   wealth=10045.819420118343 effective_wealth=None unearned_increment=None ppp_multiplier=None w_paid=None v_produced=None
//!   periphery-worker   wealth=493.84923076923076 effective_wealth=None unearned_increment=None ppp_multiplier=None w_paid=None v_produced=None
//!   comprador          wealth=720.0 effective_wealth=None unearned_increment=None ppp_multiplier=None w_paid=None v_produced=None
//!   labor-aristocracy  wealth=340.33134911242604 effective_wealth=356.46388875739643 unearned_increment=16.13253964497041 ppp_multiplier=1.4 w_paid=40.33134911242603 v_produced=40.0
//!   petty-b            wealth=250.0 effective_wealth=None unearned_increment=None ppp_multiplier=None w_paid=None v_produced=None
//!
//! post-tick edges (value_flow):
//!   exploitation periphery-worker -> core-bourgeoisie: value_flow=6.150769230769232
//!   tribute comprador -> core-bourgeoisie: value_flow=80.0
//!   wages core-bourgeoisie -> labor-aristocracy: value_flow=40.33134911242603
//!
//! post-tick context.persistent_data = {}
//!
//! post-tick economy (ALREADY DECAYED, see header (b)) = {'imperial_rent_pool': 185.447781, 'current_super_wage_rate': 0.25, 'current_repression_level': 0.5}
//! post-tick economy.imperial_rent_pool RAW (pre-quantization, see header (d); THE ORACLE FOR BSL's r09) = 185.4477812781065
//!
//! events:
//!   surplus_extraction {'source_id': 'periphery-worker', 'target_id': 'core-bourgeoisie', 'amount': 6.150769230769232, 'mechanism': 'imperial_rent'}
//! ```
//!
//! `petty-b` (the non-participant witness — no edge touches it) is
//! untouched: `wealth=250.0` (its seed, unchanged) and every wages-only
//! field stays `None`. `comprador`'s `wealth=720.0` is the frozen §1.6-c
//! OVERWRITE (`source.wealth = cut_amount`, `800 * 0.9 = 720`), not an
//! additive mutation — the exact shape `r03` (Task 3) transcribes.
//!
//! # Why exact equality and no tolerance
//!
//! `Int * Real` and correctly-rounded binary64 arithmetic are the only
//! operations either engine performs to reach the numbers above — **with
//! ONE named exception, fix round 1 (Important 1): the quantized
//! `imperial_rent_pool`/`current_super_wage_rate`/`current_repression_level`
//! fields inside the printed `economy` dict**, which additionally pass
//! through Pydantic's `SnapToGrid` (ROUND_HALF_UP, 6 decimals) on their way
//! out of `_save_economy` — a frozen-Python-only step, header fact (d). For
//! every OTHER number above (every node field, every edge `value_flow`),
//! `bsl-language.rst` §4.3's claim holds without qualification, so later
//! tasks' Rust-side pins may assert exact equality against BSL-measured
//! values, not against these printed floats (ADR183: this mirror is a
//! structure/ordering oracle, not a byte oracle; no `imperial-rent/*` rule
//! exists yet in Task 1 to measure BSL-side numbers from). For the pool
//! specifically, compare against the mirror's separate RAW print, not the
//! quantized dict entry.
//!
//! # Task 2 — `r00-tick-reset` + `r01-extraction` + `r02-extraction-credit`
//!
//! The nine tests below are the first `imperial-rent/*` rules to run against
//! this world; every asserted value is measured from the BSL engine itself
//! (`content/rules/imperial-rent.bsl`), cross-checked by hand against the
//! frozen mirror's dump above. **Task 2's pack ports ONLY Phase 1
//! (Extraction) — Phases 2/3 (Tribute/Wages, `r03`-`r07`) are not yet
//! landed**, so `core-bourgeoisie`'s post-tick wealth in THIS world is
//! `10000.0 + rent` alone, NOT the mirror's own cross-phase final print
//! (`10045.819420118343`, which also includes the tribute credit and the
//! wages debit) — ADR183: the mirror is a structure/ordering oracle, not a
//! byte oracle for a partial pack. The one number that IS directly
//! mirror-comparable, because no other phase touches it, is
//! `periphery-worker`'s post-tick wealth (`493.84923076923076`) and the
//! `EXPLOITATION` edge's `exploitation/value-flow` (`6.150769230769232`,
//! `RENT` below).
//!
//! **Fuel — measured, not guessed, per the E-LOAD-040 refusal readback,
//! then the documented bound+1 off-by-one (`bsl-language.rst` §4.5),
//! 2026-08-18, RE-MEASURED in fix round 1:** each rule was declared with
//! `:fuel 1`, loaded against this module's own `run()` (world 1, the
//! primary scenario), and the exact `E-LOAD-040: … static bound B …`
//! refusal read back verbatim: `r00-tick-reset` bound `9` → `:fuel 10`;
//! `r01-extraction` bound `69` → `:fuel 70`; `r02-extraction-credit` bound
//! `40` → `:fuel 41` against the ORIGINAL one-EXPLOITATION-edge fixtures —
//! **but fix round 1's Minor 6 repair added a SECOND EXPLOITATION edge to
//! `r01_skips_an_inactive_counterparty`'s fixture (the positive-exclusion
//! witness), raising the for-each's cardinality ceiling for THAT scenario
//! and moving both bounds again: `r01-extraction` bound `103` → `:fuel
//! 104`; `r02-extraction-credit` bound `74` → `:fuel 75`, re-measured
//! against the NEW two-edge fixture and re-verified across all three Task 2
//! scenarios at these final values.** `r00-tick-reset`'s bound (9, no
//! `for-each` of its own) is unaffected. All bounds stay well below the
//! plan's own "hundreds-to-thousands" forecast for query-bearing rules — a
//! real, measured fact, not a re-derivation of that forecast — though the
//! margin narrowed once a real two-edge world entered the suite, a fact
//! worth carrying into Task 3's own fuel measurement (that pack's own
//! for-each ceilings will scale the same way with TRIBUTE edge counts). The
//! full 14-test suite passes at these EXACT bound+1 values.
//!
//! **Task 3's own fuel — measured against BOTH world 1 (one TRIBUTE edge)
//! AND world 10 (two TRIBUTE edges off one comprador), 2026-08-18, the
//! worst-case ceiling per the brief's own instruction:** `r03-tribute`
//! bound `39` (single-TRIBUTE-edge scenarios: world 1,
//! `r01_skips_an_inactive_counterparty`'s and
//! `r01_does_not_emit_exactly_at_the_negligible_rent_boundary`'s own
//! dummy-TRIBUTE-edge fixtures) vs. bound `63` (two-TRIBUTE-edge scenarios:
//! world 10, `r03_skips_a_non_positive_comprador`'s two-comprador fixture)
//! → `:fuel 64` (worst case + 1). `r04-tribute-credit` bound `43`
//! (single-edge) vs. bound `77` (two-edge) → `:fuel 78`. Both re-verified
//! green across the FULL 22-test suite at these exact values (this file's
//! own dummy EXPLOITATION/TRIBUTE edges, added solely to give
//! `CardinalityCeilings` a computable entry for a type a fixture's own
//! narrative does not otherwise use, E-LOAD-045/D76, contribute to these
//! same graph-wide ceilings — see each inline fixture's own comment).
//!
//! **`r04-tribute-credit` RE-MEASURED, review fix round 1 (C1):** the C1
//! fix adds a THIRD `(edge-between …)`/`(field-of …)` accessor pair inside
//! the guard's own condition (previously only the two `update-node`
//! effects carried one each), raising the static bound. Re-measured
//! (`:fuel 1` readback) against the FULL suite, including the two new
//! fixtures this round added: bound `45` (single-edge) vs. bound `84`
//! (two-edge, world 10 and `r03_and_r04_skip_an_inactive_recipient`'s own
//! two-TRIBUTE-edge fixture) → `:fuel 85` (worst case + 1). Re-verified
//! green across the FULL 25-test suite at this exact value.
//!
//! **`r00-tick-reset` RE-MEASURED, review fix round 2 (N1):** D202's own
//! second effect (`(for-each (edges EdgeType/TRIBUTE) (update-edge it
//! tribute/value-flow (set 0)))`) raises `r00`'s static bound from its
//! Task 2 value (9, no `for-each` of its own then). Re-measured (`:fuel 1`
//! readback) against the FULL suite: bound `17` (single-TRIBUTE-edge
//! scenarios) vs. bound `22` (two-edge scenarios) → `:fuel 23` (worst
//! case + 1). Re-verified green across the FULL 25-test suite at this
//! exact value.

use babylon_bsl::evaluator::Value;
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/imperial-rent-conformance.bscn");
const RULE: &str = include_str!("../content/rules/imperial-rent.bsl");

/// World 10 — the two-TRIBUTE-edge comprador (D184(b)/D200, Task 3).
const SCENARIO_10: &str =
    include_str!("../content/scenarios/imperial-rent-multi-tribute-conformance.bscn");

// Node ids, fixed by the scenario's own declaration order (the scenario's
// own header names the same map; `decomposition_conformance.rs`/
// `territory_conformance.rs` precedent) — asserted, not merely declared,
// by `the_scenario_loads_clean_with_the_declared_census` below (fix round
// 1, reviewer Minor 7).
const CORE_BOURGEOISIE: NodeId = NodeId(0);
const PERIPHERY_WORKER: NodeId = NodeId(1);
const COMPRADOR: NodeId = NodeId(2);
const LABOR_ARISTOCRACY: NodeId = NodeId(3);
const PETTY_B: NodeId = NodeId(4);
const IMPERIAL_RENT_REGISTER: NodeId = NodeId(5);

// World 10's own node ids, fixed by
// `imperial-rent-multi-tribute-conformance.bscn`'s own declaration order.
const W10_COMPRADOR: NodeId = NodeId(0);
const W10_RECIPIENT_A: NodeId = NodeId(1);
const W10_RECIPIENT_B: NodeId = NodeId(2);
const W10_CARRIER: NodeId = NodeId(3);

/// The frozen mirror's own printed EXPLOITATION `value_flow`
/// (`6.150769230769232`, this module doc's frozen-mirror provenance block
/// above) — periphery-worker's consciousness (0.2) and wealth (500) are
/// untouched by any phase Task 2 does not port, so this number IS directly
/// mirror-comparable, bit-exact, unlike `core-bourgeoisie`'s cross-phase
/// wealth (see the Task 2 doc section above).
const RENT: f64 = 6.150769230769232;

/// The frozen mirror's own printed TRIBUTE `value_flow` on world 1
/// (`80.0`, this module doc's frozen-mirror provenance block above) —
/// comprador's wealth (800) is untouched by any OTHER phase (comprador
/// carries no EXPLOITATION/WAGES edge of its own in world 1), so this
/// number IS directly mirror-comparable, bit-exact.
const TRIBUTE: f64 = 80.0;

/// World 1's/world 10's `cut`/`tribute`, hand-derived independently in Rust
/// — the SAME operation order `r03`/`r04`'s shared bindings declare (`cut`
/// = `wealth * comprador-cut`, `tribute` = `wealth - cut`), computed on a
/// COMPLETELY SEPARATE interpreter (rustc's own f64 arithmetic, not the BSL
/// evaluator) from a fresh reading of the world's own inputs — NOT via
/// `TRIBUTE` (the mirror's own printed constant) and NOT via reading
/// `tribute/value-flow` off the graph.
fn hand_derived_cut_and_tribute(wealth: f64, comprador_cut: f64) -> (f64, f64) {
    let cut = wealth * comprador_cut;
    let tribute = wealth - cut;
    (cut, tribute)
}

/// Task 2 + Task 3 scope: `r00-tick-reset` + `r01-extraction` +
/// `r02-extraction-credit` + `r03-tribute` + `r04-tribute-credit`, run
/// against the shared world-1 scenario.
fn run() -> (HypergraphStore, CollectingSink) {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Task 2 + Task 3 rules must run");
    (graph, sink)
}

/// World 10 — the two-TRIBUTE-edge comprador (D184(b)/D200, Task 3). Runs
/// the SAME shared `RULE` (the whole pack, r00-r04) against
/// `imperial-rent-multi-tribute-conformance.bscn` instead of the primary
/// world. World 10 seeds NO EXPLOITATION edge at all, so `r01`/`r02`
/// iterate zero times per node (D127 hash-neutral) — every observable
/// effect on this world traces to `r00`/`r03`/`r04` alone.
fn run_world_10() -> (HypergraphStore, CollectingSink) {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO_10, RULE, &mut graph, &mut sink)
        .expect("world 10 must run against the shared RULE pack");
    (graph, sink)
}

fn attribute(graph: &HypergraphStore, id: NodeId, field: &str) -> f64 {
    graph
        .node_attribute(id, field)
        .unwrap_or_else(|e| panic!("node {id:?} field {field}: {}", e.message))
}

fn exploitation_value_flow(graph: &HypergraphStore, from: NodeId, to: NodeId) -> f64 {
    graph
        .edge_attribute("EXPLOITATION", from, to, "exploitation/value-flow")
        .unwrap_or_else(|e| {
            panic!(
                "EXPLOITATION {from:?}->{to:?} exploitation/value-flow: {}",
                e.message
            )
        })
}

fn tribute_value_flow(graph: &HypergraphStore, from: NodeId, to: NodeId) -> f64 {
    graph
        .edge_attribute("TRIBUTE", from, to, "tribute/value-flow")
        .unwrap_or_else(|e| panic!("TRIBUTE {from:?}->{to:?} tribute/value-flow: {}", e.message))
}

/// World 1's `rent`, hand-derived independently in Rust — the SAME
/// operation order `r01`/`r02`'s shared bindings declare (`base-eff` /
/// `trpf-mult` / `eff` / `one-minus-consciousness` / `rent-uncapped` /
/// `rent`), but computed on a COMPLETELY SEPARATE interpreter (rustc's own
/// f64 arithmetic, not the BSL evaluator) from a fresh reading of the
/// world's own inputs (`extraction-efficiency 0.8`, `weeks-per-year 52`,
/// `trpf-coefficient 0.0005`, `trpf-efficiency-floor 0.1`, `tick 1`,
/// periphery-worker's `wealth 500` and `revolutionary 0.2`) — NOT via
/// `RENT` (the mirror's own printed constant) and NOT via reading
/// `exploitation/value-flow` off the graph. Shared by
/// `r01_applies_the_weekly_conversion_before_the_trpf_multiplier` (which
/// checks it against the ENGINE's observed rent) and
/// `r02_credits_only_a_core_bourgeoisie_target` (fix round 1, Important 1:
/// re-anchored here so the row regains a kill surface distinct from
/// `r01_and_r02_agree_on_the_rent`'s read-fidelity check — see that test's
/// own doc comment for the DIFFERENT claim each row now makes).
fn hand_derived_rent() -> f64 {
    let base_eff = 0.8_f64 / 52.0_f64;
    let trpf_unclamped = (1.0_f64 - 0.0_f64) - (0.0005_f64 * 1.0_f64);
    let trpf_mult = if trpf_unclamped > 0.1_f64 {
        trpf_unclamped
    } else {
        0.1_f64
    };
    let eff = base_eff * trpf_mult;
    let one_minus_c = (1.0_f64 - 0.0_f64) - 0.2_f64;
    let rent_uncapped = (eff * 500.0_f64) * one_minus_c;
    if rent_uncapped < 500.0_f64 {
        rent_uncapped
    } else {
        500.0_f64
    }
}

/// The load-smoke test, through the REAL `run_once_into` seam — proves the
/// scenario loads clean against the newly-registered `"imperial-rent"`
/// system. See the module doc's Step 1 red-phase note for the pre-
/// registration failure text this test produced before `lib.rs`'s
/// `HashSet` carried `"imperial-rent"`.
#[test]
fn scenario_and_empty_pack_load() {
    const PROBE_RULE: &str = r#"
(rule imperial-rent/probe
  :role mechanic :evidence derived :material-basis "load-only smoke: prove the scenario loads against a registered imperial-rent system"
  :fuel 8
  (bindings (binding wealth :field social-class/wealth))
  (when (< wealth 0))
  (effects
    (update-node self social-class/wealth (set wealth))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, PROBE_RULE, &mut graph, &mut sink)
        .expect("the scenario must load and run against the registered-system probe rule");
}

/// The scenario's own node/edge census, independent of any rule pack — five
/// `SOCIAL_CLASS` fixtures plus the one `INSTITUTION` carrier, three edges
/// (EXPLOITATION, TRIBUTE, WAGES).
#[test]
fn the_scenario_loads_clean_with_the_declared_census() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    assert_eq!(loaded.node_count, 6, "5 social classes + 1 carrier");
    assert_eq!(
        loaded.edge_count, 3,
        "EXPLOITATION + TRIBUTE + WAGES, one each"
    );
    assert_eq!(
        loaded.node_types.get("SOCIAL_CLASS").copied(),
        Some(5),
        "five social-class nodes"
    );
    assert_eq!(
        loaded.node_types.get("INSTITUTION").copied(),
        Some(1),
        "exactly one carrier in THIS world — the D198 discriminator vector \
         below builds its own second-INSTITUTION-node fixture separately"
    );

    // Fix round 1 (reviewer Minor 7): the six top-level `NodeId` constants
    // are asserted here, not merely declared — Tasks 2-8 key every
    // assertion off this exact map, and the one id most likely to surprise
    // a later reader is `IMPERIAL_RENT_REGISTER = NodeId(5)` (that the
    // INSTITUTION node continues the SAME ascending counter after the five
    // SOCIAL_CLASS nodes, rather than starting its own).
    for id in [
        CORE_BOURGEOISIE,
        PERIPHERY_WORKER,
        COMPRADOR,
        LABOR_ARISTOCRACY,
        PETTY_B,
        IMPERIAL_RENT_REGISTER,
    ] {
        assert!(
            graph.node_exists(id),
            "{id:?} must exist — the scenario's declaration order fixes \
             this map"
        );
    }
    assert_eq!(
        graph
            .node_attribute(CORE_BOURGEOISIE, "social-class/role")
            .expect("core-bourgeoisie has a role"),
        0.0, // CORE_BOURGEOISIE is ordinal 0, ADR195
        "NodeId(0) is core-bourgeoisie, not merely SOME social-class node"
    );
    assert_eq!(
        graph
            .node_attribute(IMPERIAL_RENT_REGISTER, "institution/rent-carrier")
            .expect("imperial-rent-register has rent-carrier"),
        1.0,
        "NodeId(5) is imperial-rent-register (rent-carrier == 1) — the \
         claim every later task's carrier read keys off"
    );
}

/// The `defenum` ordinal-parity test (Global Constraints: "`defenum` is not
/// shared across scenarios… the suite carries one ordinal-parity test
/// mirroring the mint's"), mirroring
/// `decomposition_conformance.rs::social_role_order_is_the_ruled_ordinal` —
/// THIS scenario's own `SocialRole` re-declaration must store the SAME
/// eight members in the SAME order as
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

/// THE SPIKE's item (f), kept PERMANENT (Step 6) rather than deleted with
/// the rest of the spike: `institution/rent-carrier` FIELD-GUARDED
/// resolution (D198, plan §3.1) over a naive ascending-id `select-max`
/// tiebreak. This is its own small inline fixture — `SCENARIO` above mints
/// exactly one INSTITUTION node by design (§3.1: "with exactly one
/// INSTITUTION node the constant score and the discriminator score agree
/// by construction"), so this is the world that actually exercises the
/// discriminator's REASON to exist: `decoy-carrier` is declared FIRST
/// (lower NodeId) with `rent-carrier 0`, `real-carrier` SECOND (higher
/// NodeId) with `rent-carrier 1` — a naive constant-score `select-max`
/// (D45's ascending-id-first-wins) would resolve `decoy-carrier`; the
/// discriminator score must resolve `real-carrier` instead.
#[test]
fn carrier_discriminator_resolves_over_a_lower_id_decoy() {
    const TWO_INSTITUTION_SCENARIO: &str = r#"
(scenario imperial-rent/carrier-discriminator-probe
  (defvocabulary NodeType (INSTITUTION))
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)

  (node decoy-carrier NodeType/INSTITUTION
    (institution/rent-carrier 0)
    (institution/rent-pool 0))

  (node real-carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 0)))
"#;
    const DISCRIMINATOR_RULE: &str = r#"
(rule imperial-rent/discriminator-probe
  :role mechanic :evidence derived :material-basis "THE SPIKE item (f): the field-guarded select-max discriminator over a lower-id decoy INSTITUTION node (D198, plan §3.1)"
  :fuel 64
  (bindings (binding carrier :field institution/rent-carrier))
  (when (>= carrier 0))
  (effects
    (update-node
      (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))
      institution/rent-pool
      (set 999))))
"#;
    const DECOY: NodeId = NodeId(0);
    const REAL_CARRIER: NodeId = NodeId(1);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(
        TWO_INSTITUTION_SCENARIO,
        DISCRIMINATOR_RULE,
        &mut graph,
        &mut sink,
    )
    .expect("the two-INSTITUTION-node fixture must load and run clean");

    assert_eq!(
        graph
            .node_attribute(REAL_CARRIER, "institution/rent-pool")
            .expect("real-carrier has rent-pool"),
        999.0,
        "the discriminator resolves real-carrier (rent-carrier 1), the \
         higher-id node — NOT the lower-id decoy"
    );
    assert_eq!(
        graph
            .node_attribute(DECOY, "institution/rent-pool")
            .expect("decoy-carrier has rent-pool"),
        0.0,
        "the lower-id decoy (rent-carrier 0) is untouched — a naive \
         ascending-id constant-score select-max would have written HERE \
         instead, which is exactly the D198 hazard this discriminator \
         closes"
    );
}

// ---------------------------------------------------------------------
// Task 2 — `r00-tick-reset` + `r01-extraction` + `r02-extraction-credit`
// ---------------------------------------------------------------------
//
// World 1's only EXPLOITATION edge is periphery-worker -> core-bourgeoisie;
// periphery-worker's consciousness (0.2p, B7's re-point target) makes the
// frozen `(1 - consciousness)` factor provable (a zero consciousness would
// make that term structurally invisible, 1.0 always). `RENT` (above) is the
// frozen mirror's own printed `exploitation value_flow` — the ONE number
// Task 2's partial pack can compare bit-exact against the mirror, since no
// other phase touches periphery-worker or the EXPLOITATION edge.

/// Worker wealth decremented, bourgeoisie wealth incremented — both
/// bit-exact against the frozen mirror via `.to_bits()`. `core-bourgeoisie`'s
/// post-tick wealth is `10000.0 + RENT`, NOT the mirror's own cross-phase
/// final print (`10045.819420118343`) — Task 2 ports Phase 1 alone (see the
/// module doc's own Task 2 section).
#[test]
fn r01_extracts_the_frozen_rent_from_the_active_worker() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, PERIPHERY_WORKER, "social-class/wealth").to_bits(),
        493.849_230_769_230_76_f64.to_bits(),
        "periphery-worker wealth == seed(500) - RENT, bit-exact against the \
         frozen mirror's own printed periphery-worker wealth — extraction \
         is the ONLY phase touching this node in the frozen engine too"
    );
    assert_eq!(
        attribute(&graph, CORE_BOURGEOISIE, "social-class/wealth").to_bits(),
        (10_000.0_f64 + RENT + TRIBUTE).to_bits(),
        "core-bourgeoisie wealth == seed(10000) + RENT (r01) + TRIBUTE \
         (r03, THIS task) — Task 3 lands Phase 2 (tribute), and \
         core-bourgeoisie is world 1's SOLE TRIBUTE recipient as well as \
         its sole EXPLOITATION target, so both credits land on the SAME \
         node this tick, applied in byte order (r01 first, then r03) — \
         `10_000.0 + RENT + TRIBUTE` is left-associative in Rust, matching \
         the engine's own two sequential `add`s exactly. Phase 3 (wages, \
         r06-r07) is not yet landed, so the mirror's own cross-phase final \
         print (10045.819420118343, which ALSO subtracts the wages \
         payment) is STILL not the comparand here (ADR183: the mirror is a \
         structure/ordering oracle, not a byte oracle for a partial pack)"
    );
}

/// The weekly conversion (÷ weeks-per-year) applied BEFORE the TRPF
/// multiplier — hand-derived independently in Rust (a genuine cross-check
/// against a DIFFERENT computation path, not a restatement of the BSL
/// rule's own arithmetic) and asserted bit-exact against the engine's own
/// observed rent. This is `r01`'s own mutation-vector-2 witness: swapping
/// the TRPF `if` comparison picks the WRONG branch at tick 1 (`0.9995` vs.
/// the `0.1` floor are far apart, so the two branches are NOT degenerate
/// here) and this hand-derived value would then disagree with the engine.
#[test]
fn r01_applies_the_weekly_conversion_before_the_trpf_multiplier() {
    let (graph, _) = run();
    let expected_rent = hand_derived_rent();
    let observed_rent = exploitation_value_flow(&graph, PERIPHERY_WORKER, CORE_BOURGEOISIE);
    assert_eq!(
        observed_rent.to_bits(),
        expected_rent.to_bits(),
        "independently hand-derived (base-eff / trpf-mult / eff / rent, the \
         SAME operation order the rule declares) must agree bit-exact with \
         the engine's own observed rent — this is the vector that flips if \
         the TRPF `if` comparison is swapped, since 0.9995 (unclamped) and \
         0.1 (the floor) are far apart at tick 1"
    );
    assert_eq!(
        observed_rent.to_bits(),
        RENT.to_bits(),
        "and both agree with the frozen mirror's own printed value"
    );
}

/// The EXPLOITATION edge's `exploitation/value-flow` attribute — the
/// self-anchored `update-edge` write (D182), bit-exact against the frozen
/// mirror.
#[test]
fn r01_writes_the_exploitation_value_flow() {
    let (graph, _) = run();
    assert_eq!(
        exploitation_value_flow(&graph, PERIPHERY_WORKER, CORE_BOURGEOISIE).to_bits(),
        RENT.to_bits(),
        "exploitation/value-flow must equal RENT bit-exact"
    );
}

/// `r01` emits exactly one SURPLUS_EXTRACTION, payload key-by-key —
/// `source`/`target` NodeRefs (BLOCKER-5b's rename from the frozen
/// `source_id`/`target_id` strings), `amount` — and **no** `mechanism` key
/// (BLOCKER-5: there is no `Str` payload value).
#[test]
fn r01_emits_surplus_extraction_above_the_negligible_floor() {
    let (_, sink) = run();
    let extractions: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "SURPLUS_EXTRACTION")
        .collect();
    assert_eq!(
        extractions.len(),
        1,
        "exactly one SURPLUS_EXTRACTION this tick — RENT (6.15...) is well \
         above negligible-rent (0.01)"
    );
    let (_, payload) = extractions[0];
    assert_eq!(
        payload.len(),
        3,
        "exactly three payload keys — no `mechanism`"
    );
    assert_eq!(
        payload[0],
        ("source".to_owned(), Value::NodeRef(PERIPHERY_WORKER))
    );
    assert_eq!(
        payload[1],
        ("target".to_owned(), Value::NodeRef(CORE_BOURGEOISIE))
    );
    assert_eq!(payload[2], ("amount".to_owned(), Value::Real(RENT)));
}

/// `r01`'s nested guard on the EXPLOITATION target's own `active` field
/// (`economic.py:280`): an INACTIVE counterparty gets no wealth transfer,
/// no edge write, and no emit — proven on a dedicated inline fixture (world
/// 1's own core-bourgeoisie is always active, so this shape needs its own
/// small world). REVISED fix round 1 (review Important 2 + Minor 6): the
/// worker now carries a SECOND EXPLOITATION edge, to an ACTIVE
/// PETTY_BOURGEOISIE target, so this one fixture proves THREE things at
/// once instead of one under-specified claim:
/// (1) the inactive-target edge's seeded `exploitation/value-flow` is now a
///     NON-ZERO sentinel (`7`, not `0`) — Important 2's fix: with the OLD
///     zero seed, `r02`'s `it`-active conjunct had NO sentinel at all
///     (deleting it would still credit 0.0, an unobservable no-op); the
///     mutation evidence below (drop the conjunct → the carrier reads 7,
///     not 0) is what proves it killable now.
/// (2) the petty-target edge IS processed by r01 (its own gate checks only
///     `active`, never `role`) — a genuine non-zero wealth transfer, which
///     r02 then correctly EXCLUDES from the carrier credit because its
///     role fails the CORE_BOURGEOISIE gate. This is Minor 6's POSITIVE
///     EXCLUSION WITNESS: `r02_credits_only_a_core_bourgeoisie_target`'s
///     "only" claim previously rested on mutation vector 3 alone (no
///     Task-2 world contained a real non-CORE_BOURGEOISIE active target);
///     this fixture supplies one, assertion-checkable with no mutation at
///     all.
/// (3) `r01`'s own SURPLUS_EXTRACTION emit still fires for the edge that
///     legitimately transfers wealth (petty-target's), so "no events at
///     all" is no longer the right claim — exactly one event, targeting
///     petty-target, is.
#[test]
fn r01_skips_an_inactive_counterparty() {
    const INACTIVE_TARGET_SCENARIO: &str = r#"
(scenario imperial-rent/inactive-counterparty-probe
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  ; TRIBUTE added Task 3 — the shared RULE now includes r03/r04, which
  ; reference EdgeType/TRIBUTE and tribute/value-flow regardless of whether
  ; this fixture's own topology carries any TRIBUTE edge (it carries none —
  ; r03/r04 iterate zero times per node here, D127 hash-neutral).
  (defvocabulary EdgeType (EXPLOITATION TRIBUTE))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int intensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/revolutionary probability intensive)
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)
  (deffield institution/rent-tribute-inflow real extensive)
  (deffield exploitation/value-flow real intensive)
  (deffield tribute/value-flow real intensive)

  (defconst economy/extraction-efficiency 0.8c)
  (defconst economy/trpf-coefficient 0.0005c)
  (defconst economy/trpf-efficiency-floor 0.1c)
  (defconst economy/negligible-rent 0.01c)
  (defconst economy/comprador-cut 0.9c)
  (defconst timescale/weeks-per-year 52)

  (node worker NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PERIPHERY_PROLETARIAT)
    (social-class/active 1)
    (social-class/wealth 500)
    (social-class/revolutionary 0.2p))

  (node inactive-target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 0)
    (social-class/wealth 10000)
    (social-class/revolutionary 0.0p))

  (node carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 100)
    (institution/rent-tribute-inflow 0))

  (node petty-target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 2000)
    (social-class/revolutionary 0.0p))

  ; `dummy-comprador`/`dummy-recipient` + the ONE TRIBUTE edge between them
  ; are NOT part of this fixture's own EXPLOITATION-focused narrative —
  ; they exist SOLELY to give EdgeType/TRIBUTE a computable static-fuel
  ; ceiling (E-LOAD-045, D76/§2.9: the shared RULE pack's r03/r04 reference
  ; EdgeType/TRIBUTE regardless of this fixture's own topology). `wealth 0`
  ; makes r03's own `when` gate (`wealth > 0`) exclude it entirely; a
  ; non-CORE_BOURGEOISIE role on the recipient additionally excludes it
  ; from r04's own gate — doubly hash-neutral, non-empty in the graph's
  ; edge-type census.
  (node dummy-comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 0)
    (social-class/revolutionary 0.0p))

  (node dummy-recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (edge EdgeType/EXPLOITATION worker inactive-target 1)
  (edge-attr EdgeType/EXPLOITATION worker inactive-target exploitation/value-flow 7)
  (edge EdgeType/EXPLOITATION worker petty-target 1)
  (edge-attr EdgeType/EXPLOITATION worker petty-target exploitation/value-flow 0)
  (edge EdgeType/TRIBUTE dummy-comprador dummy-recipient 1)
  (edge-attr EdgeType/TRIBUTE dummy-comprador dummy-recipient tribute/value-flow 0))
"#;
    const WORKER: NodeId = NodeId(0);
    const INACTIVE_TARGET: NodeId = NodeId(1);
    const CARRIER: NodeId = NodeId(2);
    const PETTY_TARGET: NodeId = NodeId(3);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(INACTIVE_TARGET_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the inactive-counterparty fixture must load and run clean");

    // worker's revolutionary (0.2p) and wealth (500) match world 1's
    // periphery-worker exactly, so the SAME hand-derived rent applies.
    let expected_rent = hand_derived_rent();

    assert_eq!(
        graph
            .edge_attribute(
                "EXPLOITATION",
                WORKER,
                INACTIVE_TARGET,
                "exploitation/value-flow"
            )
            .expect("the seeded edge attribute reads back"),
        7.0,
        "exploitation/value-flow stays at its seeded 7 (a NON-ZERO \
         sentinel, fix round 1 Important 2) — r01 never wrote it; the \
         value's SURVIVAL is now observable, not merely a default"
    );
    assert_eq!(
        attribute(&graph, INACTIVE_TARGET, "social-class/wealth"),
        10_000.0,
        "inactive-target wealth untouched — no add"
    );

    assert_eq!(
        attribute(&graph, WORKER, "social-class/wealth").to_bits(),
        (500.0_f64 - expected_rent).to_bits(),
        "worker wealth == seed - rent, exactly ONCE — of the worker's two \
         EXPLOITATION edges, only petty-target's fires this tick \
         (inactive-target's stays blocked by the it-active guard)"
    );
    assert_eq!(
        graph
            .edge_attribute(
                "EXPLOITATION",
                WORKER,
                PETTY_TARGET,
                "exploitation/value-flow"
            )
            .expect("the petty-target edge attribute reads back")
            .to_bits(),
        expected_rent.to_bits(),
        "fix round 1, Minor 6 — the positive exclusion witness's first \
         half: r01 DOES write a real, non-zero rent onto the petty-target \
         edge (r01's own gate checks only `active`, never `role`) — a \
         genuine wealth transfer, not a no-op"
    );
    assert_eq!(
        attribute(&graph, PETTY_TARGET, "social-class/wealth").to_bits(),
        (2_000.0_f64 + expected_rent).to_bits(),
        "petty-target wealth == seed + rent — the transfer landed for real"
    );

    let extractions: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "SURPLUS_EXTRACTION")
        .collect();
    assert_eq!(
        extractions.len(),
        1,
        "exactly one SURPLUS_EXTRACTION — petty-target's edge legitimately \
         fires (rent > negligible-rent); inactive-target's stays blocked \
         entirely, before the emit gate is ever reached"
    );
    assert_eq!(
        extractions[0].1[1],
        ("target".to_owned(), Value::NodeRef(PETTY_TARGET)),
        "the one emitted event targets petty-target, not inactive-target"
    );

    assert_eq!(
        attribute(&graph, CARRIER, "institution/rent-tribute-inflow"),
        0.0,
        "r02 credits NEITHER edge — inactive-target is blocked by r02's \
         own it-active conjunct (now independently killable: dropping it \
         alone credits the seeded 7, see the mutation evidence in the \
         commit body); petty-target is blocked by r02's role conjunct — \
         fix round 1 Minor 6's positive exclusion witness's second half: \
         petty-target's edge carries a REAL non-zero rent (asserted \
         above), so this 0 means 'excluded', not 'nothing happened'"
    );
}

/// `r02` credits `rent-tribute-inflow` AND `rent-pool` by exactly the
/// HAND-DERIVED rent — both on the D198 discriminator-scored carrier, both
/// reset/seeded by `r00`/Task 1 before this tick's credit. RE-ANCHORED
/// mid-Task-2, fix round 1 (review Important 1): the earlier form compared
/// against `RENT` (the mirror's own printed constant, algebraically
/// identical to `exploitation/value-flow` given
/// `r01_writes_the_exploitation_value_flow`'s own pin) — so this row's kill
/// set was PROVABLY a subset of `r01_and_r02_agree_on_the_rent`'s (both
/// executed r02-internal mutation vectors, 3 and 5, flip BOTH tests
/// together; see that test's own doc comment for the measured evidence).
/// Anchoring on `hand_derived_rent()` instead — a value computed by NEITHER
/// r01 NOR r02, on a wholly separate interpreter — makes this row an
/// END-TO-END correctness claim ("is the credited amount the mathematically
/// TRUE rent") rather than a restatement of r02's own read-fidelity (which
/// `r01_and_r02_agree_on_the_rent` already owns). The two rows' claims are
/// now DISTINCT even though r02-internal-corruption vectors still flip both
/// together (expected: breaking r02's credit disagrees with EVERY
/// correct-value representation at once) — the distinctness is proven by
/// mutation vector 1 (r01-only: drop the `(1-consciousness)` factor),
/// which flips THIS row (the credited amount is now provably wrong) while
/// `r01_and_r02_agree_on_the_rent` stays GREEN (r02 still faithfully reads
/// whatever — even wrong — value r01 published).
#[test]
fn r02_credits_only_a_core_bourgeoisie_target() {
    let (graph, _) = run();
    let expected_rent = hand_derived_rent();
    let (_expected_cut, expected_tribute) = hand_derived_cut_and_tribute(800.0, 0.9);
    assert_eq!(
        attribute(
            &graph,
            IMPERIAL_RENT_REGISTER,
            "institution/rent-tribute-inflow"
        )
        .to_bits(),
        (expected_rent + expected_tribute).to_bits(),
        "rent-tribute-inflow: r00 reset it to 0, r02 added the hand-derived \
         rent (EXPLOITATION), r04 (THIS task) added the hand-derived \
         tribute (TRIBUTE) — core-bourgeoisie is world 1's SOLE target of \
         BOTH edge types, so its carrier credit now carries both \
         contributions, applied in byte order (r02 before r04) — \
         `expected_rent + expected_tribute` is left-associative, matching \
         the engine's own sequential `add`s exactly (0 + rent, then + \
         tribute)"
    );
    assert_eq!(
        attribute(&graph, IMPERIAL_RENT_REGISTER, "institution/rent-pool").to_bits(),
        (100.0_f64 + expected_rent + expected_tribute).to_bits(),
        "rent-pool: seeded 100, r02 added rent, r04 (THIS task) added \
         tribute — r00 does NOT reset this field (D181: it is the \
         persistent GlobalEconomy field)"
    );
}

/// The non-participant witness — no edge touches `petty-b`, so every rule
/// in this pack must leave it exactly as seeded.
#[test]
fn the_petty_bourgeois_witness_is_untouched() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, PETTY_B, "social-class/wealth"),
        250.0,
        "petty-b wealth == its seed, unchanged — no edge touches it"
    );
}

/// D201's copies-agree row (§8a) — REVISED mid-Task-2 (D116/D197 ledger row
/// 7): r02 is NOT an independent re-derivation of `rent` (an earlier draft
/// was, and `r01_and_r02_agree_on_the_rent`'s own FP-rounding failure below
/// is what caught the deeper bug — r01 runs first and mutates the worker's
/// own wealth, so a fresh `:field wealth` re-read in r02 silently read
/// POST-r01 state). r02 instead reads r01's SAME-TICK `exploitation/
/// value-flow` write. This row therefore asserts the FAITHFUL-READ
/// invariant — `Δ(rent-tribute-inflow)` (r02's carrier credit) equals the
/// EXPLOITATION edge's own recorded `exploitation/value-flow` (r01's exact,
/// unrounded `set`) — bit-exact, deliberately NOT via `Δ(core-bourgeoisie
/// wealth)`: `(10000.0 + RENT) - 10000.0 = 6.150769230769583`, which
/// differs from RENT (`6.150769230769232`) by `3.51e-13` — measured, not
/// assumed: this was the ORIGINAL form of this test, and it failed on
/// exactly this gap even after the r02 fix landed. This is ORDINARY
/// binary64 absorption in the addition (`10000.0 + RENT` cannot represent
/// all of RENT's low-order bits at that magnitude), revealed exactly by
/// the subsequent exact (Sterbenz) subtraction — `≈395 ULP of RENT` (whose
/// own ULP near `2^2` is `2^-50`), or `≈0.19 ULP of the sum` (whose ULP
/// near `2^13` is `2^-39`) — NOT catastrophic cancellation, which would
/// require the two operands to nearly cancel; `10000.0 + RENT` do not.
/// `r01_extracts_the_frozen_rent_from_the_active_worker` already covers
/// `core-bourgeoisie`'s absolute wealth bit-exact.
#[test]
fn r01_and_r02_agree_on_the_rent() {
    let (graph, _) = run();
    let edge_rent = exploitation_value_flow(&graph, PERIPHERY_WORKER, CORE_BOURGEOISIE);
    // Task 3 addendum: core-bourgeoisie is world 1's SOLE target of BOTH
    // EXPLOITATION (r02's credit) and TRIBUTE (r04's credit, THIS task) —
    // both land on the SAME rent-tribute-inflow field this tick, so this
    // row's own comparand must now include the TRIBUTE edge's published
    // value too. Measured, not assumed: `(RENT + TRIBUTE) - TRIBUTE !=
    // RENT` bit-exact at these magnitudes (ordinary binary64 rounding, the
    // SAME class this test's own doc already names below for the wealth
    // round-trip) — subtracting TRIBUTE back out to isolate r02's own
    // contribution is NOT safe, so this row instead asserts the FULL
    // relationship: both edges' own published values sum EXACTLY to the
    // observed carrier total. r04's OWN isolated read-fidelity claim,
    // independent of r02's contribution entirely, is
    // `r03_and_r04_agree_on_the_tribute` (below), asserted on world 10 —
    // ZERO EXPLOITATION edges exist there, so r02 contributes nothing to
    // isolate against.
    let edge_tribute = tribute_value_flow(&graph, COMPRADOR, CORE_BOURGEOISIE);
    // r00 resets rent-tribute-inflow to 0 every tick (D116 ledger row 3), so
    // this direct read already IS the tick's delta — no subtraction needed
    // (fix round 1, Minor 7: the earlier `- 0.0_f64` was a no-op dressed as
    // a delta computation).
    let inflow_credited = attribute(
        &graph,
        IMPERIAL_RENT_REGISTER,
        "institution/rent-tribute-inflow",
    );
    assert_eq!(
        inflow_credited.to_bits(),
        (edge_rent + edge_tribute).to_bits(),
        "r02's AND r04's combined carrier credit (rent-tribute-inflow, read \
         from r01's published exploitation/value-flow PLUS r03's published \
         tribute/value-flow, THIS task) must equal the sum of those two \
         edge attributes bit-exact — D201's duplication ledger, revised: \
         this guards BOTH r02's and r04's READ PATHS (the qnames each \
         reads, and that neither applies extra scaling), not independent \
         formulas' coincidence. A mutation perturbing EITHER r02's or r04's \
         read (Step 7 vectors 5 and the new Task 3 vector) must flip THIS \
         row while every single-rule row stays green"
    );
}

/// D196's CONVERSE witness: on world 1's own single-EXPLOITATION-edge
/// worker, `wealth - rent >= 0` holds WITHOUT a clamp — proven directly (not
/// merely by non-negativity, which would be trivially true, but by exact
/// algebraic agreement: `wealth_post == wealth_seed - RENT`, i.e. no clamp
/// intervened at all). The DISCRIMINATING two-edge case, where the
/// invariant does NOT hold and the dropped clamp becomes observable
/// (D184(a)), is world 8 — measured in Task 6, not here.
#[test]
fn r01_never_drives_a_single_edge_worker_negative() {
    let (graph, _) = run();
    let wealth_post = attribute(&graph, PERIPHERY_WORKER, "social-class/wealth");
    assert_eq!(
        wealth_post.to_bits(),
        (500.0_f64 - RENT).to_bits(),
        "wealth_post == wealth_seed - RENT exactly — the algebraic \
         signature of `sub` with NO clamp; on this one-edge world, \
         wealth_seed - RENT is itself >= 0 (500 - 6.15... > 0), matching \
         D196's reachability proof that the frozen clamp is dead here"
    );
    assert!(
        wealth_post >= 0.0,
        "the reachability invariant itself: rent <= wealth on a one-edge \
         world, so no clamp is needed to keep wealth non-negative"
    );
}

/// The `>` (strict) negligible-rent emit gate's own boundary witness
/// (Step 7 vector 4): a dedicated fixture where `rent` computes to EXACTLY
/// `negligible-rent` (both `1.0` — chosen for exact binary64
/// representability, so no rounding ambiguity survives the two independent
/// literal parses) — `>` must NOT emit here; mutating the gate to `>=`
/// must.
#[test]
fn r01_does_not_emit_exactly_at_the_negligible_rent_boundary() {
    const BOUNDARY_SCENARIO: &str = r#"
(scenario imperial-rent/negligible-rent-boundary-probe
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  ; TRIBUTE added Task 3 — the shared RULE now includes r03/r04, which
  ; reference EdgeType/TRIBUTE and tribute/value-flow regardless of whether
  ; this fixture's own topology carries any TRIBUTE edge (it carries none —
  ; r03/r04 iterate zero times per node here, D127 hash-neutral).
  (defvocabulary EdgeType (EXPLOITATION TRIBUTE))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int intensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/revolutionary probability intensive)
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)
  (deffield institution/rent-tribute-inflow real extensive)
  (deffield exploitation/value-flow real intensive)
  (deffield tribute/value-flow real intensive)

  ; extraction-efficiency=1.0, weeks-per-year=1, trpf-coefficient=0.0,
  ; trpf-efficiency-floor=0.0 => eff = 1.0 exactly (no floating-point
  ; rounding: 1.0/1 = 1.0, 1.0 - 0.0*1 = 1.0). worker wealth=1 (Int),
  ; revolutionary=0.0p => rent = 1.0 * 1 * 1.0 = 1.0 exactly. negligible-rent
  ; = 1.0c, the SAME literal spelling ("1.0"), so both parse to the
  ; identical f64 bit pattern — the boundary is exact, not approximate.
  (defconst economy/extraction-efficiency 1.0c)
  (defconst economy/trpf-coefficient 0.0c)
  (defconst economy/trpf-efficiency-floor 0.0c)
  (defconst economy/negligible-rent 1.0c)
  (defconst economy/comprador-cut 0.9c)
  (defconst timescale/weeks-per-year 1)

  (node worker NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PERIPHERY_PROLETARIAT)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (node target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 10000)
    (social-class/revolutionary 0.0p))

  (node carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 100)
    (institution/rent-tribute-inflow 0))

  ; `dummy-comprador`/`dummy-recipient` + the ONE TRIBUTE edge between them
  ; give EdgeType/TRIBUTE a computable static-fuel ceiling (E-LOAD-045,
  ; D76/§2.9) — NOT sourced from `worker` (whose wealth=1 is load-bearing
  ; for THIS test's exact rent boundary and must stay untouched by r03's
  ; own OVERWRITE). `wealth 0` excludes it from r03's own `when` gate.
  (node dummy-comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 0)
    (social-class/revolutionary 0.0p))

  (node dummy-recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (edge EdgeType/EXPLOITATION worker target 1)
  (edge-attr EdgeType/EXPLOITATION worker target exploitation/value-flow 0)
  (edge EdgeType/TRIBUTE dummy-comprador dummy-recipient 1)
  (edge-attr EdgeType/TRIBUTE dummy-comprador dummy-recipient tribute/value-flow 0))
"#;
    const WORKER: NodeId = NodeId(0);
    const TARGET: NodeId = NodeId(1);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(BOUNDARY_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the negligible-rent boundary fixture must load and run clean");

    let rent = graph
        .edge_attribute("EXPLOITATION", WORKER, TARGET, "exploitation/value-flow")
        .expect("the edge attribute reads back");
    assert_eq!(
        rent.to_bits(),
        1.0_f64.to_bits(),
        "rent must land EXACTLY at the negligible-rent boundary (1.0) for \
         this to be a real boundary witness"
    );
    assert!(
        sink.events.iter().all(|(ty, _)| ty != "SURPLUS_EXTRACTION"),
        "the strict `>` gate must NOT emit when rent == negligible-rent \
         exactly — mutating `>` to `>=` must flip this assertion"
    );
}

// ---------------------------------------------------------------------
// Task 3 — `r03-tribute` + `r04-tribute-credit`
// ---------------------------------------------------------------------
//
// World 1's only TRIBUTE edge is comprador -> core-bourgeoisie; comprador's
// wealth (800, world 1's own seed) makes the frozen §1.6-c OVERWRITE
// (`source.wealth = cut_amount`, not `sub`) provable — a zero-wealth
// comprador couldn't distinguish `set` from `sub` (both would leave 0).
// `TRIBUTE` (above) is the frozen mirror's own printed `tribute value_flow`
// on world 1 — the ONE tribute number directly mirror-comparable bit-exact,
// since comprador carries no other edge. World 10
// (`imperial-rent-multi-tribute-conformance.bscn`) supplies the
// TWO-TRIBUTE-edge fixture D184(b)/D200 need — see that scenario's own
// header for the full frozen-vs-ported numeric derivation, and this
// module's own Python mirror extension (`imperial_rent_conformance.py`'s
// `run_world_10`) for the measured frozen-sequential oracle, re-pasted
// verbatim below in `the_two_tribute_edges_apply_the_rule_scoped_cut_once`.

/// The §1.6-c OVERWRITE, verbatim: comprador's post-tick wealth equals
/// `wealth * comprador-cut` EXACTLY (`800 * 0.9 = 720`), NOT
/// `wealth - (wealth * comprador-cut)` (`800 - 720 = 80`, the defect vector
/// a `sub`-shaped mutation would produce — mutation vector 1, Step 4).
/// Hand-derived independently (a genuine cross-check, not a restatement of
/// the BSL rule's own arithmetic).
#[test]
fn r03_overwrites_the_comprador_wealth_with_the_cut() {
    let (graph, _) = run();
    let (expected_cut, _expected_tribute) = hand_derived_cut_and_tribute(800.0, 0.9);
    let wealth_post = attribute(&graph, COMPRADOR, "social-class/wealth");
    assert_eq!(
        wealth_post.to_bits(),
        expected_cut.to_bits(),
        "comprador wealth == wealth_seed * comprador-cut EXACTLY (the \
         OVERWRITE, `set`) — NOT wealth_seed - (wealth_seed * \
         comprador-cut) (the `800 - 720 = 80` defect vector a `sub`-shaped \
         mutation would produce, §1.6-c)"
    );
    assert_ne!(
        wealth_post.to_bits(),
        (800.0_f64 - expected_cut).to_bits(),
        "sanity: the OVERWRITE value (720) and the defect-vector value (80) \
         are NOT the same bit pattern, so this row genuinely distinguishes \
         them"
    );
}

/// The recipient's wealth genuinely increased by `tribute` — an `add`, not
/// a `set` (mutation vector 2, Step 4: swapping the recipient's `(add
/// tribute)` for `(set tribute)` would leave core-bourgeoisie's wealth at
/// `tribute` alone, discarding both its seed and r01's rent credit, which
/// this bit-exact total assertion catches immediately).
#[test]
fn r03_transfers_the_remainder_to_the_recipient() {
    let (graph, _) = run();
    let (_expected_cut, expected_tribute) = hand_derived_cut_and_tribute(800.0, 0.9);
    assert_eq!(
        expected_tribute.to_bits(),
        TRIBUTE.to_bits(),
        "sanity: the hand-derived tribute agrees with the frozen mirror's \
         own printed world-1 tribute value_flow"
    );
    assert_eq!(
        attribute(&graph, CORE_BOURGEOISIE, "social-class/wealth").to_bits(),
        (10_000.0_f64 + RENT + expected_tribute).to_bits(),
        "core-bourgeoisie wealth carries the tribute ADD on top of its \
         seed and r01's rent ADD — matching \
         r01_extracts_the_frozen_rent_from_the_active_worker's own updated \
         total; THIS row's own distinct claim is the mutation vector \
         (`(add tribute)` -> `(set tribute)` on the recipient), not a \
         restatement of that other row's arithmetic"
    );
}

/// The TRIBUTE edge's `tribute/value-flow` attribute — the self-anchored
/// `update-edge` write (D182), bit-exact against the frozen mirror.
#[test]
fn r03_writes_the_tribute_value_flow() {
    let (graph, _) = run();
    assert_eq!(
        tribute_value_flow(&graph, COMPRADOR, CORE_BOURGEOISIE).to_bits(),
        TRIBUTE.to_bits(),
        "tribute/value-flow must equal TRIBUTE bit-exact"
    );
}

/// Phase 2 carries no emit in the frozen engine — `r03` adds ZERO new
/// events on top of r01's own SURPLUS_EXTRACTION. Asserts the TOTAL event
/// count is unchanged from Task 2's own count (1), not merely that no
/// TRIBUTE-flavoured event exists (there is no such event type to begin
/// with — this is the strongest form of the claim available).
#[test]
fn r03_emits_nothing() {
    let (_, sink) = run();
    assert_eq!(
        sink.events.len(),
        1,
        "exactly one event this tick (r01's own SURPLUS_EXTRACTION) — r03 \
         (and r04) add none"
    );
}

/// The `wealth > 0` gate, verbatim (`economic.py:377-378`,
/// `if comprador_wealth <= 0: continue`) — a non-positive comprador pays no
/// tribute, writes no edge attribute, and credits nothing. Mirrors
/// `r01_skips_an_inactive_counterparty`'s own positive-exclusion-witness
/// shape: a SECOND, wealth-positive comprador on the SAME fixture proves
/// r03 is not simply a no-op rule. **Review fix round 2 (N1): the
/// zero-wealth comprador's TRIBUTE edge is seeded a POSITIVE sentinel
/// (`13`) — the seeded-positive-flow / false-positive-credit case N1
/// names directly** (round 1's `-13` shape sidestepped exactly this case,
/// which is why the re-review caught it). The claim under test is no
/// longer "the seed survives" (D202's `r00` reset overwrites EVERY
/// TRIBUTE edge's `tribute/value-flow` to `0` every tick, including this
/// one, before r03/r04 ever run) — it is that the reset actually FIRES:
/// the edge reads `0.0` post-tick (not the seeded `13`), and the carrier
/// credits ONLY the real, active comprador's tribute, never the wiped
/// sentinel.
#[test]
fn r03_skips_a_non_positive_comprador() {
    const NON_POSITIVE_COMPRADOR_SCENARIO: &str = r#"
(scenario imperial-rent/non-positive-comprador-probe
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defvocabulary EdgeType (EXPLOITATION TRIBUTE))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int intensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/revolutionary probability intensive)
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)
  (deffield institution/rent-tribute-inflow real extensive)
  (deffield exploitation/value-flow real intensive)
  (deffield tribute/value-flow real intensive)

  (defconst economy/extraction-efficiency 0.8c)
  (defconst economy/trpf-coefficient 0.0005c)
  (defconst economy/trpf-efficiency-floor 0.1c)
  (defconst economy/negligible-rent 0.01c)
  (defconst economy/comprador-cut 0.9c)
  (defconst timescale/weeks-per-year 52)

  (node zero-comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 0)
    (social-class/revolutionary 0.0p))

  (node positive-comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 400)
    (social-class/revolutionary 0.0p))

  (node recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 9000)
    (social-class/revolutionary 0.0p))

  (node carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 100)
    (institution/rent-tribute-inflow 0))

  ; `dummy-worker`/`dummy-target` + the ONE EXPLOITATION edge between them
  ; give EdgeType/EXPLOITATION a computable static-fuel ceiling
  ; (E-LOAD-045, D76/§2.9) — this fixture's own narrative is Phase 2 only.
  ; `dummy-worker`'s `active 0` excludes it from r01's/r02's own `when`.
  (node dummy-worker NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PERIPHERY_PROLETARIAT)
    (social-class/active 0)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (node dummy-target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  ; Review fix round 2 (N1): the sentinel is POSITIVE (13) again — the
  ; seeded-positive-flow / false-positive-credit case N1 names directly.
  ; `r00`'s D202 reset (review fix round 2) zeroes EVERY TRIBUTE edge's
  ; tribute/value-flow every tick, BEFORE r03/r04 run — this edge's seed
  ; must therefore read 0.0 post-tick (the reset fired), not merely
  ; "survive unwritten" (round 1's now-superseded -13 framing).
  (edge EdgeType/TRIBUTE zero-comprador recipient 1)
  (edge-attr EdgeType/TRIBUTE zero-comprador recipient tribute/value-flow 13)
  (edge EdgeType/TRIBUTE positive-comprador recipient 1)
  (edge-attr EdgeType/TRIBUTE positive-comprador recipient tribute/value-flow 0)
  (edge EdgeType/EXPLOITATION dummy-worker dummy-target 1)
  (edge-attr EdgeType/EXPLOITATION dummy-worker dummy-target exploitation/value-flow 0))
"#;
    const ZERO_COMPRADOR: NodeId = NodeId(0);
    const POSITIVE_COMPRADOR: NodeId = NodeId(1);
    const RECIPIENT: NodeId = NodeId(2);
    const CARRIER: NodeId = NodeId(3);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(NON_POSITIVE_COMPRADOR_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the non-positive-comprador fixture must load and run clean");

    let (expected_cut, expected_tribute) = hand_derived_cut_and_tribute(400.0, 0.9);

    assert_eq!(
        attribute(&graph, ZERO_COMPRADOR, "social-class/wealth"),
        0.0,
        "zero-comprador wealth untouched — no set, since wealth > 0 fails \
         the rule's own `when` gate"
    );
    assert_eq!(
        graph
            .edge_attribute("TRIBUTE", ZERO_COMPRADOR, RECIPIENT, "tribute/value-flow")
            .expect("the seeded edge attribute reads back"),
        0.0,
        "tribute/value-flow reads 0.0 post-tick, NOT the seeded 13 — r00's \
         D202 reset zeroed EVERY TRIBUTE edge before r03/r04 ran; r03 \
         itself never wrote it either (wealth > 0 excluded the subject), \
         so the reset is the ONLY thing that could have moved it off 13"
    );

    assert_eq!(
        attribute(&graph, POSITIVE_COMPRADOR, "social-class/wealth").to_bits(),
        expected_cut.to_bits(),
        "positive-comprador (wealth 400) IS processed normally — the \
         positive-exclusion witness's first half"
    );
    assert_eq!(
        graph
            .edge_attribute(
                "TRIBUTE",
                POSITIVE_COMPRADOR,
                RECIPIENT,
                "tribute/value-flow"
            )
            .expect("the positive-comprador edge attribute reads back")
            .to_bits(),
        expected_tribute.to_bits(),
        "positive-comprador's tribute edge carries a real, non-zero value"
    );

    assert_eq!(
        attribute(&graph, RECIPIENT, "social-class/wealth").to_bits(),
        (9_000.0_f64 + expected_tribute).to_bits(),
        "recipient wealth == seed + ONLY positive-comprador's tribute — \
         the positive-exclusion witness's second half: zero-comprador's \
         edge carries a real non-zero SEED (13) that r00's reset wipes, \
         so this total excluding it means 'excluded', not 'nothing \
         happened'"
    );

    assert_eq!(
        attribute(&graph, CARRIER, "institution/rent-tribute-inflow").to_bits(),
        expected_tribute.to_bits(),
        "r04 credits only positive-comprador's edge — N1's false-positive- \
         credit case, closed: zero-comprador's edge is seeded a POSITIVE \
         13 (a value that WOULD pass r04's C1 `tribute/value-flow > 0` \
         gate if it survived), but r00's D202 reset zeroes it BEFORE r03/ \
         r04 ever run, so r04 never sees anything but 0.0 there — the \
         credit total is exactly positive-comprador's real tribute, with \
         NO contribution from the wiped sentinel"
    );
}

/// **I1 (review fix round 1), REVISED review fix round 2 (D202's own
/// side effect):** proves r03's `it`-active conjunct is independently
/// killable — no existing fixture seeded an inactive TRIBUTE endpoint
/// before I1. `inactive-recipient`'s edge is seeded `77` (distinguishable
/// from this fixture's own real tribute value, `50`), but round 2's D202
/// (`r00` now zeroes EVERY TRIBUTE edge every tick, unconditionally,
/// BEFORE r03/r04 run) means the seed is ALREADY GONE by the time either
/// rule evaluates it — the observable post-tick value is `0.0`, not the
/// seed. r03's `it`-active guard is still what keeps it AT `0.0` (blocking
/// the write r03 would otherwise make for an active target); r04's OWN
/// `it`-active conjunct, once D202 exists, is PROVEN UNREACHABLE-TO-KILL
/// by this or any value-based fixture — `tribute/value-flow` can no
/// longer carry a stale positive value on ANY edge, active recipient or
/// not, so `> 0` alone already excludes an inactive target regardless of
/// the it-active conjunct's presence (the SAME reachability-proof class
/// D196 already uses elsewhere in this file for a dead frozen clamp; kept
/// for defense-in-depth and symmetry with r02's own it-active conjunct,
/// per D202's own updated D-row). `active-recipient`'s own TRIBUTE edge
/// is the positive-exclusion witness: a genuine transfer happens there,
/// proving this is not a no-op rule.
#[test]
fn r03_and_r04_skip_an_inactive_recipient() {
    const INACTIVE_RECIPIENT_SCENARIO: &str = r#"
(scenario imperial-rent/inactive-recipient-probe
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defvocabulary EdgeType (EXPLOITATION TRIBUTE))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int intensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/revolutionary probability intensive)
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)
  (deffield institution/rent-tribute-inflow real extensive)
  (deffield exploitation/value-flow real intensive)
  (deffield tribute/value-flow real intensive)

  (defconst economy/extraction-efficiency 0.8c)
  (defconst economy/trpf-coefficient 0.0005c)
  (defconst economy/trpf-efficiency-floor 0.1c)
  (defconst economy/negligible-rent 0.01c)
  (defconst economy/comprador-cut 0.9c)
  (defconst timescale/weeks-per-year 52)

  (node comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 500)
    (social-class/revolutionary 0.0p))

  (node inactive-recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 0)
    (social-class/wealth 8000)
    (social-class/revolutionary 0.0p))

  (node active-recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 3000)
    (social-class/revolutionary 0.0p))

  (node carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 100)
    (institution/rent-tribute-inflow 0))

  ; `dummy-worker`/`dummy-target` + the ONE EXPLOITATION edge give
  ; EdgeType/EXPLOITATION a computable static-fuel ceiling (E-LOAD-045,
  ; D76/§2.9) — this fixture's own narrative is Phase 2 only.
  (node dummy-worker NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PERIPHERY_PROLETARIAT)
    (social-class/active 0)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (node dummy-target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (edge EdgeType/TRIBUTE comprador inactive-recipient 1)
  (edge-attr EdgeType/TRIBUTE comprador inactive-recipient tribute/value-flow 77)
  (edge EdgeType/TRIBUTE comprador active-recipient 1)
  (edge-attr EdgeType/TRIBUTE comprador active-recipient tribute/value-flow 0)
  (edge EdgeType/EXPLOITATION dummy-worker dummy-target 1)
  (edge-attr EdgeType/EXPLOITATION dummy-worker dummy-target exploitation/value-flow 0))
"#;
    const COMPRADOR: NodeId = NodeId(0);
    const INACTIVE_RECIPIENT: NodeId = NodeId(1);
    const ACTIVE_RECIPIENT: NodeId = NodeId(2);
    const CARRIER: NodeId = NodeId(3);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(INACTIVE_RECIPIENT_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the inactive-recipient fixture must load and run clean");

    let (expected_cut, expected_tribute) = hand_derived_cut_and_tribute(500.0, 0.9);

    assert_eq!(
        attribute(&graph, COMPRADOR, "social-class/wealth").to_bits(),
        expected_cut.to_bits(),
        "comprador wealth == cut — r03 fired (for active-recipient's edge)"
    );
    assert_eq!(
        attribute(&graph, INACTIVE_RECIPIENT, "social-class/wealth"),
        8_000.0,
        "inactive-recipient wealth untouched — r03's it-active guard \
         blocked the whole per-edge effect body for this edge"
    );
    assert_eq!(
        graph
            .edge_attribute(
                "TRIBUTE",
                COMPRADOR,
                INACTIVE_RECIPIENT,
                "tribute/value-flow"
            )
            .expect("the seeded edge attribute reads back"),
        0.0,
        "tribute/value-flow reads 0.0, NOT the seeded 77 — r00's D202 \
         reset zeroed EVERY TRIBUTE edge before r03/r04 ran; r03's own \
         it-active guard additionally never wrote a REAL value here \
         either (against a wealth-500 comprador, whose own real tribute \
         is 50)"
    );
    assert_eq!(
        attribute(&graph, ACTIVE_RECIPIENT, "social-class/wealth").to_bits(),
        (3_000.0_f64 + expected_tribute).to_bits(),
        "active-recipient wealth == seed + tribute — the positive-exclusion \
         witness: r03 IS a real, firing rule, not a no-op"
    );
    assert_eq!(
        graph
            .edge_attribute("TRIBUTE", COMPRADOR, ACTIVE_RECIPIENT, "tribute/value-flow")
            .expect("the active-recipient edge attribute reads back")
            .to_bits(),
        expected_tribute.to_bits(),
        "active-recipient's edge carries the real tribute"
    );
    assert_eq!(
        attribute(&graph, CARRIER, "institution/rent-tribute-inflow").to_bits(),
        expected_tribute.to_bits(),
        "r04 credits ONLY active-recipient's edge (50) — inactive- \
         recipient's edge reads 0.0 post-D202-reset, so it contributes \
         nothing regardless of r04's own it-active conjunct's presence. \
         Dropping r03's it-active conjunct WOULD move this total to 127.0 \
         (50 + 77, since r03 would then write inactive-recipient's real \
         tribute for real) — the kill vector this row exists to catch for \
         r03. Dropping r04's OWN it-active conjunct does NOT move this \
         total at all (confirmed empirically, fix round 2) — D202 already \
         guarantees `tribute/value-flow > 0` alone excludes an inactive \
         target, the reachability-proof class D196 documents elsewhere"
    );
}

/// r04's END-TO-END correctness claim (hand-derived, a SEPARATE computation
/// path from `r03_and_r04_agree_on_the_tribute` below) — asserted on
/// WORLD 10, where comprador's TWO TRIBUTE edges are the SOLE traffic
/// touching `institution/rent-*` (zero EXPLOITATION edges exist, so r02
/// contributes nothing to isolate against, unlike world 1). Both edges
/// publish the SAME `tribute` (80.0, D200's repeated-derivation, NOT
/// D184(b)'s frozen 72.0 second-edge value) — `Δ(rent-tribute-inflow) ==
/// Δ(rent-pool) == 2 * tribute`.
#[test]
fn r04_credits_the_pool_and_the_tribute_inflow() {
    let (graph, _) = run_world_10();
    let (_expected_cut, expected_tribute) = hand_derived_cut_and_tribute(800.0, 0.9);
    let expected_total = expected_tribute + expected_tribute;
    assert_eq!(
        attribute(&graph, W10_CARRIER, "institution/rent-tribute-inflow").to_bits(),
        expected_total.to_bits(),
        "rent-tribute-inflow: r00 reset it to 0, r04 added the SAME \
         hand-derived tribute TWICE (once per TRIBUTE edge — BOTH \
         recipients are CORE_BOURGEOISIE) — 2 * 80.0 = 160.0, NOT the \
         frozen sequential 80.0 + 72.0 = 152.0"
    );
    assert_eq!(
        attribute(&graph, W10_CARRIER, "institution/rent-pool").to_bits(),
        (100.0_f64 + expected_total).to_bits(),
        "rent-pool: seeded 100, r04 added the SAME tribute twice — r00 \
         does NOT reset this field"
    );
}

/// **C1 (review fix round 1): the comprador-cut = 0 divergence, the
/// original bug's own regression vector.** `economy/comprador-cut = 0` is
/// LEGAL (defines.yaml:72's own domain is unbounded-below-1, and D191's
/// own `defconst` table already ships 0.9 as one point in that domain, not
/// its only legal value) — cut = wealth * 0 = 0, so tribute = wealth - 0 =
/// wealth: the FULL wealth transfers, and r03's own `(set cut)` writes
/// comprador's post-tick wealth to EXACTLY 0. Under the PRE-fix gate
/// (`(> wealth 0)` reading self's CURRENT field, evaluated on r04's own
/// firing AFTER r03 already overwrote it to 0), r04 would have WRONGLY
/// skipped the credit — 0 is never `> 0` — even though a real, positive,
/// FULL-wealth tribute transfer happened this exact tick. The C1 fix
/// (per-edge `tribute/value-flow > 0`) reads r03's own published 600.0
/// instead, crediting correctly. This fixture is the frozen-faithful proof:
/// the frozen engine's OWN gate (`economic.py:377-378`) is evaluated
/// ONCE, pre-transfer, on the true wealth (600), never re-derived
/// post-transfer — a comprador-cut of 0 does not change whether the
/// frozen engine credits, only how much of the wealth moves.
#[test]
fn r04_credits_the_full_transfer_when_comprador_cut_is_zero() {
    const ZERO_CUT_SCENARIO: &str = r#"
(scenario imperial-rent/zero-comprador-cut-probe
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defvocabulary EdgeType (EXPLOITATION TRIBUTE))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int intensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/revolutionary probability intensive)
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)
  (deffield institution/rent-tribute-inflow real extensive)
  (deffield exploitation/value-flow real intensive)
  (deffield tribute/value-flow real intensive)

  (defconst economy/extraction-efficiency 0.8c)
  (defconst economy/trpf-coefficient 0.0005c)
  (defconst economy/trpf-efficiency-floor 0.1c)
  (defconst economy/negligible-rent 0.01c)
  ; The divergence-triggering value — legal (defines.yaml:72's domain),
  ; not the shipped 0.9.
  (defconst economy/comprador-cut 0c)
  (defconst timescale/weeks-per-year 52)

  (node full-transfer-comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 600)
    (social-class/revolutionary 0.0p))

  (node full-transfer-recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 3000)
    (social-class/revolutionary 0.0p))

  (node carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 100)
    (institution/rent-tribute-inflow 0))

  ; `dummy-worker`/`dummy-target` + the ONE EXPLOITATION edge give
  ; EdgeType/EXPLOITATION a computable static-fuel ceiling (E-LOAD-045,
  ; D76/§2.9) — this fixture's own narrative is Phase 2 only.
  (node dummy-worker NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PERIPHERY_PROLETARIAT)
    (social-class/active 0)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (node dummy-target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (edge EdgeType/TRIBUTE full-transfer-comprador full-transfer-recipient 1)
  (edge-attr EdgeType/TRIBUTE full-transfer-comprador full-transfer-recipient tribute/value-flow 0)
  (edge EdgeType/EXPLOITATION dummy-worker dummy-target 1)
  (edge-attr EdgeType/EXPLOITATION dummy-worker dummy-target exploitation/value-flow 0))
"#;
    const COMPRADOR: NodeId = NodeId(0);
    const RECIPIENT: NodeId = NodeId(1);
    const CARRIER: NodeId = NodeId(2);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(ZERO_CUT_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the zero-comprador-cut fixture must load and run clean");

    assert_eq!(
        attribute(&graph, COMPRADOR, "social-class/wealth"),
        0.0,
        "comprador wealth == cut == wealth_seed * 0 == 0 — the FULL wealth \
         was cut away, the r03 OVERWRITE at its most extreme"
    );
    assert_eq!(
        graph
            .edge_attribute("TRIBUTE", COMPRADOR, RECIPIENT, "tribute/value-flow")
            .expect("the edge attribute reads back"),
        600.0,
        "tribute == wealth_seed - cut == 600 - 0 == the FULL wealth — r03 \
         wrote it correctly regardless of r04's own gate"
    );
    assert_eq!(
        attribute(&graph, RECIPIENT, "social-class/wealth"),
        3_600.0,
        "recipient wealth == seed(3000) + the full 600 transfer"
    );
    assert_eq!(
        attribute(&graph, CARRIER, "institution/rent-tribute-inflow"),
        600.0,
        "r04 CREDITS the full 600 — under the PRE-fix self-level \
         `wealth > 0` gate (re-reading comprador's wealth AFTER r03's own \
         `(set cut)` already zeroed it), this assertion would read 0.0 \
         instead (the C1 defect, confirmed red against the pre-fix gate \
         before this fix landed, reverted for this commit — see the task \
         report). The C1-fixed per-edge `tribute/value-flow > 0` gate \
         reads r03's own published 600.0 and credits correctly, matching \
         the frozen engine's ONE pre-transfer wealth check exactly"
    );
    assert_eq!(
        attribute(&graph, CARRIER, "institution/rent-pool"),
        700.0,
        "rent-pool: seeded 100 + the full 600 credit"
    );
}

/// **I2 (review fix round 1): r04 reads the edge in the `self -> it`
/// direction, never reversed — the fuel-neutral, VALUE-level drift killer
/// `r03_and_r04_agree_on_the_tribute` itself was missing.** A dedicated
/// fixture with BOTH a real forward TRIBUTE edge (comprador -> recipient,
/// which r03 writes for real) AND a decoy REVERSE edge (recipient ->
/// comprador, seeded `999.0` — a value r03/r04 could never produce here)
/// lets a `self`/`it` swap inside r04's `edge-between` resolve to a REAL,
/// DIFFERENT edge instead of erroring — on world 10 (no reverse edges
/// exist there) the same swap instead trips a load-time fuel-bound error
/// (confirmed, not a value disagreement — see the task report), which is
/// exactly the gap this fixture closes. `recipient`'s own wealth is seeded
/// `0` so it can never itself be a meaningful TRIBUTE source (r03's own
/// `wealth > 0` gate excludes it), keeping this fixture's only moving part
/// r04's own directionality. **Since fix round 2 (D202), the decoy edge's
/// `999.0` seed is wiped to `0.0` by r00's blanket TRIBUTE-edge value-flow
/// reset each tick** — the decoy remains a real, different edge a swapped
/// `edge-between` resolves to; only its observed value changed.
///
/// Mutation evidence (run red in round 1, reverted; mechanics restated
/// for the D202 reset by the round-2 re-review): swapping all three of
/// r04's `(edge-between EdgeType/TRIBUTE self it)` occurrences to
/// `(edge-between EdgeType/TRIBUTE it self)` makes this row observe the
/// decoy edge's value credited instead of the real `30.0` — at round 1
/// that read `999.0` vs `30.0`; under D202's reset it reads `0.0` vs
/// `30.0`. A genuine VALUE mismatch either way, not a fuel trip (verified
/// against a temporarily-raised fuel to isolate the value-level effect
/// from the mutation's own small fuel-bound shift).
#[test]
fn r04_reads_the_edge_self_to_it_not_reversed() {
    const REVERSED_EDGE_DECOY_SCENARIO: &str = r#"
(scenario imperial-rent/reversed-edge-decoy-probe
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defvocabulary EdgeType (EXPLOITATION TRIBUTE))
  (defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))

  (deffield social-class/role enum SocialRole)
  (deffield social-class/active int intensive)
  (deffield social-class/wealth real extensive)
  (deffield social-class/revolutionary probability intensive)
  (deffield institution/rent-carrier int extensive)
  (deffield institution/rent-pool real extensive)
  (deffield institution/rent-tribute-inflow real extensive)
  (deffield exploitation/value-flow real intensive)
  (deffield tribute/value-flow real intensive)

  (defconst economy/extraction-efficiency 0.8c)
  (defconst economy/trpf-coefficient 0.0005c)
  (defconst economy/trpf-efficiency-floor 0.1c)
  (defconst economy/negligible-rent 0.01c)
  (defconst economy/comprador-cut 0.9c)
  (defconst timescale/weeks-per-year 52)

  (node comprador NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/COMPRADOR_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 300)
    (social-class/revolutionary 0.0p))

  ; wealth 0 — recipient can NEVER itself be a meaningful TRIBUTE source
  ; (r03's own wealth > 0 gate excludes it), so the reverse decoy edge
  ; below stays undisturbed regardless of recipient's own r03 firing.
  (node recipient NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/CORE_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 0)
    (social-class/revolutionary 0.0p))

  (node carrier NodeType/INSTITUTION
    (institution/rent-carrier 1)
    (institution/rent-pool 100)
    (institution/rent-tribute-inflow 0))

  ; `dummy-worker`/`dummy-target` + the ONE EXPLOITATION edge give
  ; EdgeType/EXPLOITATION a computable static-fuel ceiling (E-LOAD-045,
  ; D76/§2.9) — this fixture's own narrative is Phase 2 only.
  (node dummy-worker NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PERIPHERY_PROLETARIAT)
    (social-class/active 0)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (node dummy-target NodeType/SOCIAL_CLASS
    (social-class/role SocialRole/PETTY_BOURGEOISIE)
    (social-class/active 1)
    (social-class/wealth 1)
    (social-class/revolutionary 0.0p))

  (edge EdgeType/TRIBUTE comprador recipient 1)
  (edge-attr EdgeType/TRIBUTE comprador recipient tribute/value-flow 0)
  ; The decoy REVERSE edge — a self/it-swapped r04 would resolve THIS edge
  ; instead of the real forward one. 999.0 is impossible for r03/r04 to
  ; produce here (comprador's own real tribute is 30.0).
  (edge EdgeType/TRIBUTE recipient comprador 1)
  (edge-attr EdgeType/TRIBUTE recipient comprador tribute/value-flow 999)
  (edge EdgeType/EXPLOITATION dummy-worker dummy-target 1)
  (edge-attr EdgeType/EXPLOITATION dummy-worker dummy-target exploitation/value-flow 0))
"#;
    const CARRIER: NodeId = NodeId(2);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(REVERSED_EDGE_DECOY_SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the reversed-edge-decoy fixture must load and run clean");

    let (_expected_cut, expected_tribute) = hand_derived_cut_and_tribute(300.0, 0.9);
    assert_eq!(
        attribute(&graph, CARRIER, "institution/rent-tribute-inflow").to_bits(),
        expected_tribute.to_bits(),
        "r04 credits the REAL forward tribute (30.0) — NOT the decoy \
         reverse edge's seeded 999.0. A self/it swap inside r04's own \
         edge-between calls would resolve the WRONG (reverse) edge and \
         credit 999.0 instead — the fuel-neutral, value-level drift this \
         row exists to catch (I2, review fix round 1)"
    );
}

/// r04's READ-FIDELITY claim (edge-read, D201's duplication ledger, §8a) —
/// the producer/consumer correction this file's own header MANDATES for
/// this row (D116/D197 ledger row 7's forward note): asserts r04's carrier
/// credit equals the SUM of both TRIBUTE edges' own published
/// `tribute/value-flow` (r03's exact `set`s), bit-exact — NOT via a
/// wealth-delta comparison (r01_and_r02_agree_on_the_rent's own doc
/// explains why that class of comparison is unsafe in general). Asserted
/// on world 10 for the SAME isolation reason as the row above.
#[test]
fn r03_and_r04_agree_on_the_tribute() {
    let (graph, _) = run_world_10();
    let tribute_a = tribute_value_flow(&graph, W10_COMPRADOR, W10_RECIPIENT_A);
    let tribute_b = tribute_value_flow(&graph, W10_COMPRADOR, W10_RECIPIENT_B);
    let inflow_credited = attribute(&graph, W10_CARRIER, "institution/rent-tribute-inflow");
    assert_eq!(
        inflow_credited.to_bits(),
        (tribute_a + tribute_b).to_bits(),
        "r04's carrier credit (rent-tribute-inflow, read from r03's \
         published tribute/value-flow on BOTH edges) must equal the sum of \
         those two edge attributes bit-exact — this guards r04's READ PATH \
         specifically (the qname it reads off each edge, and that it \
         applies no extra scaling), isolated from r02 (world 10 seeds no \
         EXPLOITATION edge at all). A mutation perturbing r04's own \
         tribute transcription (Step 4's third vector) must flip THIS row \
         while the single-rule rows above (world 10's own \
         `r04_credits_the_pool_and_the_tribute_inflow`) stay green — the \
         SAME distinction r01_and_r02_agree_on_the_rent draws relative to \
         r02_credits_only_a_core_bourgeoisie_target"
    );
}

/// **D184(b) + D200, measured together (world 10's own sole purpose).**
/// Frozen sequential (Python mirror, `imperial_rent_conformance.py::
/// run_world_10`, stdout pasted verbatim below): comprador wealth
/// `800.0 -> 720.0 -> 648.0` (the SECOND TRIBUTE edge's `cut_amount` is
/// computed off the ALREADY-OVERWRITTEN 720.0 balance,
/// `economic.py:375`'s per-edge SOURCE re-read). Ported (this row's own
/// measurement): comprador wealth `800.0 -> 720.0`, written TWICE —
/// `r03-tribute`'s `cut` is ONE rule-scoped binding, computed once from
/// `self`'s pre-state wealth, shared by BOTH `for-each` iterations (D200:
/// the repeated `set` on the same field is accepted, last-write-wins, and
/// idempotent here because both writes carry the identical value).
///
/// Frozen mirror stdout, 2026-08-18, verbatim (`PYTHONPATH="$PWD/src"
/// UV_FROZEN=1 uv run python
/// rust/crates/babylon-tick/content/scenarios/imperial_rent_conformance.py`,
/// the world-10 section):
///
/// ```text
/// ======================================================================
/// WORLD 10 — imperial-rent-multi-tribute-conformance.bscn (Task 3)
/// ======================================================================
/// comprador seed wealth = 800.0, economy.comprador_cut = 0.9
///
/// post-tick social classes:
///   comprador    wealth=648.0
///   recipient-a  wealth=5080.0
///   recipient-b  wealth=2072.0
///
/// post-tick edges (value_flow), declaration/query_edges order:
///   tribute comprador -> recipient-a: value_flow=80.0
///   tribute comprador -> recipient-b: value_flow=72.0
/// ```
#[test]
fn the_two_tribute_edges_apply_the_rule_scoped_cut_once() {
    let (graph, _) = run_world_10();
    let (expected_cut, expected_tribute) = hand_derived_cut_and_tribute(800.0, 0.9);

    // The PORTED number: 800 -> 720, written twice (D200's last-write-wins
    // on an identical value).
    let ported_comprador_wealth = attribute(&graph, W10_COMPRADOR, "social-class/wealth");
    assert_eq!(
        ported_comprador_wealth.to_bits(),
        expected_cut.to_bits(),
        "PORTED comprador wealth == 800 -> 720 (cut applied once, written \
         twice, D200) — NOT the FROZEN SEQUENTIAL 800 -> 720 -> 648 \
         (economic.py:375's per-edge source re-read, D184(b), measured by \
         the frozen mirror's own `run_world_10`, this test's own doc \
         comment)"
    );
    assert_ne!(
        ported_comprador_wealth.to_bits(),
        648.0_f64.to_bits(),
        "sanity: the ported value (720) is NOT the frozen sequential value \
         (648) — this is the divergence D184(b) predicts, now measured on \
         the real Rust driver"
    );

    // Both recipients receive the SAME tribute (the rule-scoped `tribute`
    // binding does not vary per edge) — 80.0 for BOTH, not 80.0/72.0.
    let tribute_a = tribute_value_flow(&graph, W10_COMPRADOR, W10_RECIPIENT_A);
    let tribute_b = tribute_value_flow(&graph, W10_COMPRADOR, W10_RECIPIENT_B);
    assert_eq!(
        tribute_a.to_bits(),
        expected_tribute.to_bits(),
        "recipient-a's tribute == 80.0 (the rule-scoped value)"
    );
    assert_eq!(
        tribute_b.to_bits(),
        expected_tribute.to_bits(),
        "recipient-b's tribute ALSO == 80.0, the SAME rule-scoped value — \
         NOT the frozen sequential 72.0 (the second edge's own \
         re-derived, smaller, cut_amount-driven remainder)"
    );
    assert_eq!(
        attribute(&graph, W10_RECIPIENT_A, "social-class/wealth").to_bits(),
        (5_000.0_f64 + expected_tribute).to_bits(),
        "recipient-a wealth == seed + 80.0"
    );
    assert_eq!(
        attribute(&graph, W10_RECIPIENT_B, "social-class/wealth").to_bits(),
        (2_000.0_f64 + expected_tribute).to_bits(),
        "recipient-b wealth == seed + 80.0 (the PORTED value) — the frozen \
         mirror's own recipient-b would instead read seed + 72.0 \
         (2072.0), per the doc comment's pasted stdout above"
    );
}
