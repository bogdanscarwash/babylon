//! Consciousness class-surface ternary port (issue #588, ADR204 W10).
//! FIRST consumer of the `probability` deffield lane in committed content
//! (digest B.2) — pinned by name here. UNPOSITIONED idiom: ternary fields
//! are never defaulted into existence; readers optional-bind with `0.0p`
//! defaults and gate on `(> (+ r (+ l f)) 0)` — a sum of zero IS "no
//! reading" (L-ABS), never a fabricated share.
//!
//! Task 1+2 scope: the declaration surface
//! (`consciousness-ternary-conformance.bscn`), the pack's first rule
//! (`consciousness/p0-position`), and the measured readout
//! (`consciousness/p8-dominant-worldview`, Task 2) — dominant pole = argmax,
//! ties resolved in the ruled order LIBERAL >
//! REVOLUTIONARY > FASCIST at a STRICT 1e-6 (frozen
//! models/entities/consciousness.py:177-192, transcribed verbatim) — THE one
//! home for the hegemonic tie-break.
//!
//! Task 3 scope: the measured update law — the nine-rule pack's p1..p7
//! (inbox reset, org/class solidarity pushes, wage balance, agitation, the
//! ADR016 routing law RE-POINTED at the stored ternary, persisted baselines)
//! — with the dual-implementation Python generator as the conformance oracle
//! (ADR183: the frozen engine is a structure/ordering contract, not a byte
//! oracle; the port stores the ternary directly and APPLIES Δl, diverging
//! from frozen trajectories BY CONSTRUCTION — the re-point, pack D-row 1).
//! Every expected value below is the generator's repr output pinned EXACTLY
//! (no tolerance — the estate norm, lifecycle_conformance.rs's header).
//!
//! # Provenance
//!
//! The oracle is `content/scenarios/consciousness_ternary_conformance.py`,
//! which mirrors the pack's binding order operation-for-operation (the BSL
//! side is the transcription of record; reassociation is a conformance bug).
//! The command, from the repository root:
//!
//! ```text
//! uv run python \
//!     rust/crates/babylon-tick/content/scenarios/consciousness_ternary_conformance.py
//! ```
//!
//! Its output on 2026-08-15, verbatim (Task 1-3); RE-RUN on 2026-08-17 for
//! Train B item 3 (#591, D151's narrowing 3 discharged) — the wage flow
//! rides the un-narrowed WAGES-edge push (`consciousness/p2-wages-push`,
//! printed as `p2w` below, D116-ordered between `p2` and `p3`), and the
//! per-class value columns are BIT-IDENTICAL to the 2026-08-15 run (the
//! single-employer exactness the ceremony proves — only the fired counts
//! moved, +13 both ticks):
//!
//! ```text
//! --- tick 1 ---
//! predicted fired counts (guard-passed subjects per rule):
//!   consciousness/p0: 1
//!   consciousness/p1: 11
//!   consciousness/p2: 1
//!   consciousness/p2w: 13
//!   consciousness/p3: 6
//!   consciousness/p4: 3
//!   consciousness/p5: 3
//!   consciousness/p6: 11
//!   consciousness/p7: 3
//!   consciousness/p8: 11
//!   total: 63
//!
//! p0 seed result for class-emergent's tick-1 start: (0.0, 1.0, 0.0)
//!
//! node                     r                      l                      f                      agitation_out          inbox    balance                prev_w   prev_wealth dominant
//! class-exploited          0.5072                 0.382                  0.11080000000000001    0.135                  0.4      -0.05263157894736842   9.0      50.0     'REVOLUTIONARY'
//! class-bribed             0.1                    0.48                   0.42                   0.9                    0        0.09090909090909091    12.0     90.0     'LIBERAL'
//! class-unpositioned       ABSENT                 ABSENT                 ABSENT                 ABSENT                 ABSENT   ABSENT                 ABSENT   ABSENT   ABSENT
//! class-emergent           0.009                  0.982                  0.009                  0.135                  0.5      -0.1111111111111111    8.0      30.0     'LIBERAL'
//! employer                 ABSENT                 ABSENT                 ABSENT                 ABSENT                 ABSENT   ABSENT                 ABSENT   ABSENT   ABSENT
//! tv-liberal-clear         0.2                    0.5                    0.3                    0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! tv-revolutionary-clear   0.6                    0.4                    0.0                    0.0                    0        ABSENT                 ABSENT   ABSENT   'REVOLUTIONARY'
//! tv-fascist-clear         0.2                    0.3                    0.5                    0.0                    0        ABSENT                 ABSENT   ABSENT   'FASCIST'
//! tv-tie-lr                0.5                    0.5                    0.0                    0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! tv-tie-rf                0.5                    0.0                    0.5                    0.0                    0        ABSENT                 ABSENT   ABSENT   'REVOLUTIONARY'
//! tv-tie-lf                0.0                    0.5                    0.5                    0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! tv-strict-gap            0.333333               0.333333               0.333334               0.0                    0        ABSENT                 ABSENT   ABSENT   'FASCIST'
//! tv-tie-all-true          0.333333               0.333334               0.333333               0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//!
//! --- tick 2 ---
//! predicted fired counts (guard-passed subjects per rule):
//!   consciousness/p0: 0
//!   consciousness/p1: 11
//!   consciousness/p2: 1
//!   consciousness/p2w: 13
//!   consciousness/p3: 6
//!   consciousness/p4: 3
//!   consciousness/p5: 3
//!   consciousness/p6: 11
//!   consciousness/p7: 3
//!   consciousness/p8: 11
//!   total: 62
//!
//! node                     r                      l                      f                      agitation_out          inbox    balance                prev_w   prev_wealth dominant
//! class-exploited          0.51368                0.3658                 0.12052000000000002    0.12150000000000001    0.4      -0.05263157894736842   9.0      50.0     'REVOLUTIONARY'
//! class-bribed             0.1                    0.372                  0.528                  0.81                   0        0.09090909090909091    12.0     90.0     'FASCIST'
//! class-unpositioned       ABSENT                 ABSENT                 ABSENT                 ABSENT                 ABSENT   ABSENT                 ABSENT   ABSENT   ABSENT
//! class-emergent           0.0171                 0.9658                 0.0171                 0.12150000000000001    0.5      -0.1111111111111111    8.0      30.0     'LIBERAL'
//! employer                 ABSENT                 ABSENT                 ABSENT                 ABSENT                 ABSENT   ABSENT                 ABSENT   ABSENT   ABSENT
//! tv-liberal-clear         0.2                    0.5                    0.3                    0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! tv-revolutionary-clear   0.6                    0.4                    0.0                    0.0                    0        ABSENT                 ABSENT   ABSENT   'REVOLUTIONARY'
//! tv-fascist-clear         0.2                    0.3                    0.5                    0.0                    0        ABSENT                 ABSENT   ABSENT   'FASCIST'
//! tv-tie-lr                0.5                    0.5                    0.0                    0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! tv-tie-rf                0.5                    0.0                    0.5                    0.0                    0        ABSENT                 ABSENT   ABSENT   'REVOLUTIONARY'
//! tv-tie-lf                0.0                    0.5                    0.5                    0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! tv-strict-gap            0.333333               0.333333               0.333334               0.0                    0        ABSENT                 ABSENT   ABSENT   'FASCIST'
//! tv-tie-all-true          0.333333               0.333334               0.333333               0.0                    0        ABSENT                 ABSENT   ABSENT   'LIBERAL'
//! ```
//!
//! The fired-count arithmetic (the ceremony's spike, Step 3): the EXPECTED
//! hypothesis was "employer only, +1" — MEASURED reality differs, recorded
//! honestly rather than forced. `p2-wages-push`'s subject type is
//! SOCIAL_CLASS (its one `:field` binding, `social-class/active`, pins it —
//! `tick.rs::subject_type_of`), mirroring `p2-org-solidarity-push`'s own
//! `active`-gated shape exactly; EVERY social class in this world seeds
//! `active 1` (all thirteen), so the `when` guard passes thirteen times —
//! the for-each idiom fires on edgeless subjects too (`collect_pass`'s
//! `fired += 1` runs regardless of how many neighbors the for-each finds,
//! `tick.rs:715`), and only `employer`'s three WAGES edges actually push a
//! write. Tick 1: 50 (Task 3's total) + 13 = 63. Tick 2: 49 (tick-2's prior
//! total, p0 not re-firing) + 13 = 62.
//!
//! Controller ruling 2026-08-15 (Ruling A, extended — resolving the Task-3
//! NEEDS_CONTEXT): class-bribed's tick-1 dominant is LIBERAL — the vector's
//! witness is the ADR016 ROUTING (Δf = +0.12, Δr = 0, eff_sol
//! chauvinist-clamped to 0), not a one-tick flip; hegemony erodes, it
//! doesn't snap. The FASCIST flip lands at tick 2 (the
//! `tick_two_accumulation_witness` below).
//!
//! **Post-repair note (#491 T1, D183, controller adjudication 2026-08-18,
//! C1's own ceremony):** the transcript above is the FROZEN-ADJACENT
//! dual-implementation oracle's own output, transcribed verbatim and NOT
//! regenerated for this repair (the same discipline
//! `solidarity_conformance.rs` uses for its own oracle) — it still reads
//! the pre-C1 formula (raw `Δwealth` as the rent term) and its
//! `class-bribed` row (`l=0.372, f=0.528, agitation_out=0.81,
//! dominant='FASCIST'`) is now STALE. C1 re-expresses the rent term as
//! `Δwealth ÷ previous-wealth` (a proportional rate, licensed extensive ÷
//! extensive → intensive, T1's own E-TYPE-040 arm) rather than the raw
//! absolute figure; class-bribed's actual tick-2 output is now `l≈0.5976,
//! f≈0.3024, agitation_out≈0.008526315789473686, dominant='LIBERAL'` — the
//! flip this transcript's row and the ruling above describe no longer
//! happens. Every OTHER row (all other classes, all `tv-*` fixtures, every
//! fired count) is unaffected — verified in
//! `measured_update_law_matches_the_dual_implementation_exactly` and
//! `tick_two_accumulation_witness` below, both re-measured, not derived
//! from this stale transcript.
//!
//! The four spelling-spike verdicts
//! Task 1 was chartered to settle are recorded in the scenario file's own
//! header; the store-side facts they rest on:
//!
//! - Absence errors on read (III.11): an unwritten field is a loud
//!   `GraphError`, never a default `0.0` (`scenario.rs`'s "No defaults"
//!   contract, `substrate.rs:177-184`).
//! - One-home law + the Task-1 -> Task-2 handoff (controller ruling 1):
//!   `social-class/dominant-worldview`'s ONLY writer is
//!   `consciousness/p8-dominant-worldview`; p0 does NOT record dominance at
//!   positioning. Task 1 pinned the field ABSENT on the freshly-positioned
//!   class as the one-home guard (no writer existed yet); with p8 landed,
//!   that same class now reads the measured readout of its (0, 1, 0) rest
//!   state — LIBERAL — and loud absence survives only for the genuinely
//!   unread (class-unpositioned, employer).

use babylon_bsl::compose_declaration_preludes;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::run_once_into_with_prelude;

const SCENARIO: &str = include_str!("../content/scenarios/consciousness-ternary-conformance.bscn");
// Train B item 4 (#591, D157): the scenario stopped re-declaring
// `WorldView` itself — every caller here routes through the shared
// declaration prelude instead.
const WORLDVIEW_PRELUDE: &str = include_str!("../content/declarations/worldview.bscn");
const ORGANIZATION_PRACTICE_PRELUDE: &str =
    include_str!("../content/declarations/organization-practice.bscn");
const CONSCIOUSNESS_RULES: &str = include_str!("../content/rules/consciousness.bsl");

fn practice_worldview_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE, WORLDVIEW_PRELUDE])
        .expect("the ordered organization and worldview preludes compose")
}

// Node ids, fixed by the scenario's own declaration order (the scenario
// file's header names the same map).
const CLASS_EXPLOITED: NodeId = NodeId(0);
const CLASS_BRIBED: NodeId = NodeId(1);
const CLASS_UNPOSITIONED: NodeId = NodeId(2);
const CLASS_EMERGENT: NodeId = NodeId(3);
const EMPLOYER: NodeId = NodeId(4);
// Task 2's eight tv-* read-path fixtures (ids 6-13; org-solid is 5).
const TV_LIBERAL_CLEAR: NodeId = NodeId(6);
const TV_REVOLUTIONARY_CLEAR: NodeId = NodeId(7);
const TV_FASCIST_CLEAR: NodeId = NodeId(8);
const TV_TIE_LR: NodeId = NodeId(9);
const TV_TIE_RF: NodeId = NodeId(10);
const TV_TIE_LF: NodeId = NodeId(11);
const TV_STRICT_GAP: NodeId = NodeId(12);
const TV_TIE_ALL_TRUE: NodeId = NodeId(13);

/// Task 1's posture test: one tick of the pack over the
/// five-class-plus-org world (Task 2's eight tv-* fixtures ride along),
/// asserting the seed roles' post-tick states — including that absence stays
/// LOUD (the UNPOSITIONED witness) and that positioning records the ruled
/// rest state. Task-2 handoff: `dominant-worldview` is written by
/// `consciousness/p8-dominant-worldview` (the one-home law). Task-3 note:
/// with p1..p7 landed, the end-of-tick store shows the ROUTED values — p0's
/// (0, 1, 0) write on class-emergent is overwritten by p6-route in the same
/// tick (D116), so the positioning law is witnessed by the exact routed
/// vector (0.009, 0.982, 0.009), which is only consistent with a (0, 1, 0)
/// start; the full vector table lives in
/// `measured_update_law_matches_the_dual_implementation_exactly` below.
#[test]
fn unpositioned_class_gets_no_reading() {
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = run_once_into_with_prelude(
        SCENARIO,
        &practice_worldview_prelude(),
        CONSCIOUSNESS_RULES,
        &mut graph,
        &mut sink,
    )
    .expect("the consciousness ternary scenario plus the ten-rule pack must load and run");

    // p0 fired exactly once: class-emergent is the ONLY subject that is
    // active, anchored (wages-paid + value-produced present), and
    // ternary-absent (r + l + f sums to exactly zero under the declared
    // `0.0p` defaults).
    let p0_fired = report
        .per_rule_fired
        .iter()
        .find(|(id, _)| id == "consciousness/p0-position")
        .map(|(_, n)| *n);
    assert_eq!(
        p0_fired,
        Some(1),
        "p0-position fires for class-emergent and nothing else"
    );
    assert_eq!(
        report.fired, 65,
        "p0:1 (class-emergent) + p1:13 (W2 repair, adjudication (d): \
         unconditional `(when #t)`, every SOCIAL_CLASS subject — was 11, \
         positioned-only, before the repair) + p2:1 (org-solid) + \
         p2-wages-push:13 (every active \
         SOCIAL_CLASS subject — the for-each idiom fires on edgeless \
         subjects too; only employer's three WAGES edges write, Step-3 \
         spike) + p3:6 (r > 0.3 sources: class-exploited, \
         tv-revolutionary-clear, tv-tie-lr, tv-tie-rf, tv-strict-gap, \
         tv-tie-all-true) + p4:3 + p5:3 + p6:11 (every positioned class — \
         the sum-guard alone) + p7:3 (anchored) + p8:11 (the readout)"
    );

    // class-unpositioned (no anchors, no ternary seed): every pack field
    // EXCEPT the two inbox carriers stays unwritten and reads loud-absent —
    // never a fabricated share (L-ABS; the row-19 disease's death
    // certificate). p6's route gates on the zero ternary sum; p4/p5/p7 gate
    // on the anchor sentinels; p6's agitation binding is the
    // `:optional :default 0` form (pack D-record 1) — never tick-fatal on
    // absence — and the sum-guard keeps this class out regardless.
    for field in [
        "social-class/revolutionary",
        "social-class/liberal",
        "social-class/fascist",
        "social-class/agitation",
        "social-class/wage-balance",
        "social-class/previous-wages",
        "social-class/previous-wealth",
        "social-class/dominant-worldview",
    ] {
        assert!(
            graph.node_attribute(CLASS_UNPOSITIONED, field).is_err(),
            "unpositioned: {field} must error absent (III.11), never default"
        );
    }
    // W2 repair (adjudication (d)): p1-inbox-reset's guard is now
    // `(when #t)` — unconditional over every SOCIAL_CLASS subject,
    // POSITIONED OR NOT — so solidarity-inbox is no longer L-ABS-absent for
    // an unpositioned class; it is explicitly reset to 0 every tick, same
    // as a positioned class's. This is the intended discharge of the
    // false-positive/latent-defect pair the repair fixes, not a new
    // fabricated share: every reader of this field is already an
    // `:optional :default 0` binding, so an EXPLICIT 0 and an ABSENT-
    // reads-as-default-0 were always observationally identical to every
    // reader; only the store's own presence bit moves.
    assert_eq!(
        graph
            .node_attribute(CLASS_UNPOSITIONED, "social-class/solidarity-inbox")
            .expect("W2 repair: unconditional reset writes 0 even when unpositioned"),
        0.0
    );

    // class-emergent (anchors, no ternary seed): p0 positioned it at the
    // ruled unorganized rest state (0, 1, 0) THIS tick, then p5/p6 routed it
    // same-tick (D116) — the store shows the routed vector, exactly the
    // dual implementation's tick-1 row. The routed (0.009, 0.982, 0.009) is
    // only consistent with the (0, 1, 0) start: the seeding law is pinned
    // through the routing law's own arithmetic.
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/revolutionary")
            .expect("positioned and routed: revolutionary written"),
        0.009
    );
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/liberal")
            .expect("positioned and routed: liberal written"),
        0.982
    );
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/fascist")
            .expect("positioned and routed: fascist written"),
        0.009
    );
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/agitation")
            .expect("positioned: agitation produced"),
        0.135,
        "p5 produced 0.15 (wage cut 9 -> 8), p6 decayed it to 0.135"
    );
    // ONE-HOME HANDOFF (controller ruling 1, Task 1 -> Task 2): p0 does NOT
    // record dominance — `social-class/dominant-worldview`'s only writer is
    // `consciousness/p8-dominant-worldview`. The readout now reflects the
    // same tick's ROUTED ternary (D116): l = 0.982 is the unique max —
    // LIBERAL. Stored ordinal 1 (declaration order IS the storage ordinal,
    // ADR195) — pinned as a STORED VALUE directly by this suite's own
    // dominant-ordinal table
    // (`measured_update_law_matches_the_dual_implementation_exactly`'s
    // `dynamic` vectors, below in this file), not through a parity test in
    // another file (Train B item 4, #591: the prelude switch retired that
    // external parity test — `tick_goldens.rs`'s own
    // `consciousness_ternary_worldview_member_order_is_the_ruled_ordinal`
    // — as a declared test death).
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/dominant-worldview")
            .expect("p8 writes the readout for the same-tick-positioned class (D116)"),
        1.0,
        "dominant of the routed (0.009, 0.982, 0.009) is LIBERAL — the one-home readout"
    );

    // class-exploited (seeded (0.5, 0.4, 0.1)): p0 did NOT touch it — the
    // seed was p6's ROUTING INPUT, not p0's subject. The store shows the
    // seed routed through the update law, exactly the dual implementation's
    // tick-1 row (full table in the conformance test below).
    assert_eq!(
        graph
            .node_attribute(CLASS_EXPLOITED, "social-class/revolutionary")
            .expect("seeded: revolutionary present"),
        0.5072,
        "the positioned seed is not p0's subject — it is p6's input"
    );
    assert_eq!(
        graph
            .node_attribute(CLASS_EXPLOITED, "social-class/liberal")
            .expect("seeded: liberal present"),
        0.382
    );
    assert_eq!(
        graph
            .node_attribute(CLASS_EXPLOITED, "social-class/fascist")
            .expect("seeded: fascist present"),
        0.11080000000000001
    );

    // employer (active, population, NO anchors): p0 did NOT position it —
    // the -1 anchor sentinels reject it even though it IS active. An
    // anchorless class is never a consciousness subject via p0/p4/p5/p6/
    // p7/p8 — even though employer now carries a wage relation of its own
    // (the seeded WAGES edges + wages/value-flow, D151's discharge) and is
    // p2-wages-push's only WRITING subject: its for-each pushes into the
    // three classes below, never back onto itself, so none of ITS OWN
    // fields — ternary, agitation, balance, baselines, dominant — is ever
    // written by any of the ten rules. The two inbox carriers are the
    // exception (W2 repair, adjudication (d)): p1-inbox-reset is
    // unconditional now, so it fires over EVERY SOCIAL_CLASS subject,
    // employer included.
    for field in [
        "social-class/revolutionary",
        "social-class/liberal",
        "social-class/fascist",
        "social-class/agitation",
        "social-class/wage-balance",
        "social-class/previous-wages",
        "social-class/previous-wealth",
        "social-class/dominant-worldview",
    ] {
        assert!(
            graph.node_attribute(EMPLOYER, field).is_err(),
            "employer: never a subject — {field} errors absent"
        );
    }
    assert_eq!(
        graph
            .node_attribute(EMPLOYER, "social-class/solidarity-inbox")
            .expect("W2 repair: p1-inbox-reset fires unconditionally, employer included"),
        0.0,
        "employer: solidarity-inbox is explicitly reset to 0, not absent"
    );
}

/// Task 2's read-path vectors: `consciousness/p8-dominant-worldview` over the
/// eight tv-* fixture classes — the measured readout (argmax, then the ruled
/// tie order LIBERAL > REVOLUTIONARY > FASCIST within a STRICT 1e-6 of the
/// max; frozen `models/entities/consciousness.py:177-192`, transcribed
/// verbatim) proven on every arm: three clear maxima, the three pairwise
/// ties, the strictness boundary (a decimal-1e-6 gap is NOT a tie — strict
/// `<` excludes the boundary in decimal and f64-verbatim alike, and the
/// frozen ground truth's `dominant_tendency` returns fascist for that seed;
/// controller ruling 2026-08-15), and the true all-equal tie. The tv classes
/// seed ONLY population / active=1 / the ternary / agitation 0 — no anchors,
/// no edges. Task-3 note: p1's inbox-reset writes 0 onto their (absent)
/// solidarity-inbox (harmless machinery write, hash-covered) and p6-route
/// fires on them (positioned — the sum-guard alone; agitation rides the
/// `:optional :default 0` read) as a bit-exact no-op —
/// EXCEPT tv-tie-all-true, whose 0.999999 simplex defect the closure's
/// remainder branch heals by +1e-6 to l (lawful A-001 behavior) before p8
/// reads it same-tick: its dominant is the readout of the HEALED ternary
/// (still LIBERAL). Every expected dominant below is unchanged by Task 3.
#[test]
fn dominant_worldview_readout_vectors() {
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = run_once_into_with_prelude(
        SCENARIO,
        &practice_worldview_prelude(),
        CONSCIOUSNESS_RULES,
        &mut graph,
        &mut sink,
    )
    .expect("the consciousness ternary scenario plus the p0+p8 pack must load and run");

    // p8 fires exactly eleven times: class-exploited, class-bribed, and
    // class-emergent (positioned by p0 THIS tick — D116 same-tick
    // visibility) plus the eight tv-* fixtures; never class-unpositioned or
    // employer (ternary sum 0 — the UNPOSITIONED guard), never org-solid
    // (not a SOCIAL_CLASS subject).
    let p8_fired = report
        .per_rule_fired
        .iter()
        .find(|(id, _)| id == "consciousness/p8-dominant-worldview")
        .map(|(_, n)| *n);
    assert_eq!(
        p8_fired,
        Some(11),
        "p8 reads every positioned class and nothing else"
    );

    // Stored ordinals: REVOLUTIONARY = 0, LIBERAL = 1, FASCIST = 2 —
    // declaration order IS the storage ordinal (ADR195). THIS table pins
    // those ordinals ITSELF, as stored values read straight off the graph
    // below (the OrgKind read-back pattern — scenario.rs's
    // an_enum_field_seeds_by_member_ref_and_stores_the_declared_ordinal) —
    // not through a parity test in another file. The prelude's own
    // `(defenum WorldView …)` (content/declarations/worldview.bscn) is
    // byte-identical to `worldview-foundation.bscn`'s, whose declaration
    // order `tick_goldens.rs`'s mint-side
    // `worldview_member_order_is_the_ruled_ordinal` independently guards
    // (Train B item 4, #591: the ternary-scenario-side parity test this
    // comment used to cite is a declared test death — the re-declaration it
    // guarded no longer exists).
    const REVOLUTIONARY: f64 = 0.0;
    const LIBERAL: f64 = 1.0;
    const FASCIST: f64 = 2.0;
    let cases = [
        (TV_LIBERAL_CLEAR, LIBERAL, "(0.2, 0.5, 0.3): clear l max"),
        (
            TV_REVOLUTIONARY_CLEAR,
            REVOLUTIONARY,
            "(0.6, 0.4, 0.0): clear r max",
        ),
        (TV_FASCIST_CLEAR, FASCIST, "(0.2, 0.3, 0.5): clear f max"),
        (
            TV_TIE_LR,
            LIBERAL,
            "(0.5, 0.5, 0.0): l == r at the max, ruled L > R",
        ),
        (
            TV_TIE_RF,
            REVOLUTIONARY,
            "(0.5, 0.0, 0.5): r == f at the max, ruled R > F",
        ),
        (
            TV_TIE_LF,
            LIBERAL,
            "(0.0, 0.5, 0.5): l == f at the max, ruled L > F",
        ),
        (
            TV_STRICT_GAP,
            FASCIST,
            "(0.333333, 0.333333, 0.333334): a decimal-1e-6 gap is NOT a tie \
             (strict `<` excludes the boundary; f64 lands the gap just ABOVE \
             fl(1e-6)) — the unique f max wins",
        ),
        (
            TV_TIE_ALL_TRUE,
            LIBERAL,
            "(0.333333, 0.333333, 0.333333): the true all-equal tie — dl = 0 < 1e-6, \
             LIBERAL takes the ruled order's first slot",
        ),
    ];
    for (id, ordinal, why) in cases {
        assert_eq!(
            graph
                .node_attribute(id, "social-class/dominant-worldview")
                .expect("tv classes are positioned seeds — p8 writes the readout"),
            ordinal,
            "{why}"
        );
    }
}

/// Task 3's measured-update-law conformance (extended by Train B item 3,
/// #591, with the un-narrowed wage flow): one tick of the ten-rule pack,
/// every positioned class's outputs asserted against the dual-implementation
/// generator's repr floats EXACTLY (no tolerance). The seven per-class
/// outputs: the routed ternary (r, l, f), the decayed agitation store, the
/// solidarity inbox, the wage balance, and the dominant readout — plus the
/// persisted baselines (previous-wages, previous-wealth) and the per-rule
/// fired counts. The generator's verbatim output is in the module header.
#[test]
fn measured_update_law_matches_the_dual_implementation_exactly() {
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = run_once_into_with_prelude(
        SCENARIO,
        &practice_worldview_prelude(),
        CONSCIOUSNESS_RULES,
        &mut graph,
        &mut sink,
    )
    .expect("the consciousness ternary scenario plus the ten-rule pack must load and run");

    // Per-rule fired counts (guard-passed subjects). p3's six include four
    // tv-* classes whose for-each iterates an empty neighbor set — a
    // guard-passed subject fires even when its bounded iteration is empty.
    // p2-wages-push's thirteen is the starkest instance of the same law:
    // its subject type is SOCIAL_CLASS (its one :field binding pins it) and
    // EVERY social class in this world seeds active 1, so the guard passes
    // thirteen times — only employer's three WAGES edges actually push a
    // write (the Step-3 spike, module header).
    let fired_of = |rule: &str| {
        report
            .per_rule_fired
            .iter()
            .find(|(id, _)| id == rule)
            .map(|(_, n)| *n)
    };
    let expected_fired = [
        ("consciousness/p0-position", 1),
        // W2 repair (adjudication (d)): p1's guard is now `(when #t)` —
        // unconditional, every SOCIAL_CLASS subject, matching
        // p2-wages-push's own thirteen below (same subject type, same
        // total population).
        ("consciousness/p1-inbox-reset", 13),
        ("consciousness/p2-org-solidarity-push", 1),
        ("consciousness/p2-wages-push", 13),
        ("consciousness/p3-class-solidarity-push", 6),
        ("consciousness/p4-wage-balance", 3),
        ("consciousness/p5-agitation", 3),
        ("consciousness/p6-route", 11),
        ("consciousness/p7-persist-baselines", 3),
        ("consciousness/p8-dominant-worldview", 11),
    ];
    for (rule, count) in expected_fired {
        assert_eq!(fired_of(rule), Some(count), "{rule} fired count");
    }

    const REVOLUTIONARY: f64 = 0.0;
    const LIBERAL: f64 = 1.0;
    // (node, r, l, f, agitation, inbox, balance, prev_wages, prev_wealth,
    //  dominant ordinal) — the generator's tick-1 repr output, verbatim.
    let dynamic = [
        (
            CLASS_EXPLOITED,
            0.5072,
            0.382,
            0.11080000000000001,
            0.135,
            0.4,
            -0.05263157894736842,
            9.0,
            50.0,
            REVOLUTIONARY,
        ),
        (
            CLASS_BRIBED,
            0.1,
            0.5987368421052631,
            0.30126315789473684,
            0.009473684210526316,
            0.0,
            0.09090909090909091,
            12.0,
            90.0,
            // Ruling A (pre-#491-T1-C1): tick-1 dominant was LIBERAL — the
            // witness was the ROUTING (Δf +0.12, Δr 0, eff_sol
            // chauvinist-clamped to 0); the FASCIST flip landed at tick 2.
            // RE-MEASURED (#491 T1, D183, controller adjudication
            // 2026-08-18, C1's own ceremony): the rent term now reads
            // Δwealth ÷ previous-wealth (a proportional rate, 5/95 ≈
            // 5.26%) instead of the frozen's raw Δwealth = 5 (an absolute
            // Currency figure) — agitation drops ~95x (0.9 -> ~0.0095) and
            // the fascist-routing pressure weakens correspondingly (Δf
            // shrinks from +0.12 to ~+0.0013). Tick-1 stays LIBERAL, same
            // ordinal as before; the MAGNITUDE is what moved. See
            // `tick_two_accumulation_witness` below: the tick-2 FASCIST
            // flip this comment used to describe no longer happens.
            LIBERAL,
        ),
        (
            CLASS_EMERGENT,
            0.009,
            0.982,
            0.009,
            0.135,
            0.5,
            -0.1111111111111111,
            8.0,
            30.0,
            LIBERAL,
        ),
    ];
    for (id, r, l, f, agitation, inbox, balance, prev_w, prev_wealth, dominant) in dynamic {
        let read = |field: &str| {
            graph
                .node_attribute(id, field)
                .unwrap_or_else(|e| panic!("{id:?} {field}: {}", e.message))
        };
        assert_eq!(read("social-class/revolutionary"), r, "{id:?} r");
        assert_eq!(read("social-class/liberal"), l, "{id:?} l");
        assert_eq!(read("social-class/fascist"), f, "{id:?} f");
        assert_eq!(
            read("social-class/agitation"),
            agitation,
            "{id:?} agitation"
        );
        assert_eq!(
            read("social-class/solidarity-inbox"),
            inbox,
            "{id:?} solidarity-inbox"
        );
        assert_eq!(
            read("social-class/wage-balance"),
            balance,
            "{id:?} wage-balance"
        );
        assert_eq!(
            read("social-class/previous-wages"),
            prev_w,
            "{id:?} previous-wages (p7 persisted this tick's pushed wages-inbox)"
        );
        assert_eq!(
            read("social-class/previous-wealth"),
            prev_wealth,
            "{id:?} previous-wealth"
        );
        assert_eq!(
            read("social-class/dominant-worldview"),
            dominant,
            "{id:?} dominant-worldview"
        );
    }

    // The percolation gate, exactly: class-emergent's inbox is 0.5 — the
    // class-exploited 0.5p push (source r 0.5 > 0.3 at p3-time) landed and
    // the class-bribed 0.9p push (source r 0.1 <= 0.3) did NOT leak.
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/solidarity-inbox")
            .expect("emergent inbox written"),
        0.5,
        "0.5p landed, 0.9p gate-blocked: the ADR087 percolation gate"
    );

    // The tv-* read-path fixtures: untouched by p2..p7's value laws (no
    // anchors; agitation seeded 0 makes routing a bit-exact no-op) EXCEPT
    // p1's inbox-reset (0 written onto the absent field) and the closure's
    // remainder branch on tv-tie-all-true (l healed +1e-6). Wage-balance and
    // the persisted baselines stay ABSENT on all eight (p4/p7 never fire).
    let tv_seeds = [
        (TV_LIBERAL_CLEAR, 0.2, 0.5, 0.3),
        (TV_REVOLUTIONARY_CLEAR, 0.6, 0.4, 0.0),
        (TV_FASCIST_CLEAR, 0.2, 0.3, 0.5),
        (TV_TIE_LR, 0.5, 0.5, 0.0),
        (TV_TIE_RF, 0.5, 0.0, 0.5),
        (TV_TIE_LF, 0.0, 0.5, 0.5),
        (TV_STRICT_GAP, 0.333333, 0.333333, 0.333334),
        // The closure heal: l = 0.333333 + (1 - 0.999999) — lawful A-001
        // remainder-to-liberal, pinned bit-exactly.
        (TV_TIE_ALL_TRUE, 0.333333, 0.333334, 0.333333),
    ];
    for (id, r, l, f) in tv_seeds {
        let read = |field: &str| {
            graph
                .node_attribute(id, field)
                .unwrap_or_else(|e| panic!("{id:?} {field}: {}", e.message))
        };
        assert_eq!(read("social-class/revolutionary"), r, "{id:?} r");
        assert_eq!(read("social-class/liberal"), l, "{id:?} l");
        assert_eq!(read("social-class/fascist"), f, "{id:?} f");
        assert_eq!(
            read("social-class/agitation"),
            0.0,
            "{id:?} agitation: seeded 0, decayed 0 — bit-neutral"
        );
        assert_eq!(
            read("social-class/solidarity-inbox"),
            0.0,
            "{id:?} solidarity-inbox: p1's reset write (the hash-visible machinery write)"
        );
        for field in [
            "social-class/wage-balance",
            "social-class/previous-wages",
            "social-class/previous-wealth",
        ] {
            assert!(
                graph.node_attribute(id, field).is_err(),
                "{id:?} {field}: no anchors — p4/p7 never fire, stays absent"
            );
        }
    }
}

/// Controller-ruled witness (Ruling A extended, 2026-08-15): a TWO-tick run
/// pinning the accumulation law. p7-persist-baselines ran at tick 1, so
/// tick-2's wage/wealth increments are ZERO (the pushed wage flow
/// unchanged, the persisted baselines now equal it — the zero increment IS
/// the persist machinery's differential witness) and the tick-1 decayed
/// agitation routes again.
///
/// **Re-measured (#491 T1, D183, controller adjudication 2026-08-18, C1's
/// own ceremony): class-bribed's dominant NO LONGER FLIPS to FASCIST at
/// tick 2.** Before C1, the rent term read the frozen's raw absolute
/// `Δwealth` (5 out of 90) as an imperial-rent-decline proxy; after C1 it
/// reads `Δwealth ÷ previous-wealth` (≈5.26%), the proportional rate —
/// scale-invariant across classes of different wealth, matching how
/// imperial rent Φ behaves as a ratio everywhere else in this engine. The
/// SAME nominal $5 loss now barely registers for a $90-wealth class:
/// agitation drops ~95× (0.9 → ~0.0095 at tick 1), the fascist-routing
/// pressure weakens correspondingly, and class-bribed stays LIBERAL
/// through both ticks instead of eroding to FASCIST. This is the intended
/// material consequence of C1, not a regression: relative, not absolute,
/// material loss now drives the routing.
#[test]
fn tick_two_accumulation_witness() {
    let mut session = babylon_tick::TickSession::new_with_prelude(
        SCENARIO,
        &practice_worldview_prelude(),
        CONSCIOUSNESS_RULES,
        HypergraphStore::new(),
        SessionId::new("consciousness-ternary-conformance-test").expect("literal is non-empty"),
    )
    .expect("the consciousness ternary scenario plus the ten-rule pack must load into a session");
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    session.advance(&mut sink).expect("tick 1");
    let report2 = session.advance(&mut sink).expect("tick 2");
    let graph = session.graph();

    // Tick 2 fires every rule but p0 (class-emergent was positioned at tick
    // 1 — the seeding law fires once, never re-fires).
    let p0_fired = report2
        .per_rule_fired
        .iter()
        .find(|(id, _)| id == "consciousness/p0-position")
        .map(|(_, n)| *n);
    assert_eq!(p0_fired, Some(0), "p0 has no tick-2 subject");
    assert_eq!(
        report2.fired, 64,
        "tick-2 total: 49 (Task 3's tick-2 total, 50 minus p0's one firing) \
         + 13 (p2-wages-push, unchanged tick to tick — social-class/active \
         is never written) + 2 (W2 repair, adjudication (d): p1-inbox-reset \
         is unconditional now — 13 firings every tick, not 11 — and Task \
         3's bundled 49 still carries p1's OLD tick-2 count internally)"
    );

    const REVOLUTIONARY: f64 = 0.0;
    const LIBERAL: f64 = 1.0;
    // FASCIST unused after the D183 re-measurement (below): class-bribed
    // no longer flips at tick 2 — see the "non-flip, named" comment.
    // (node, r, l, f, agitation-out, dominant) — the generator's tick-2 repr
    // output, verbatim (module header). The tick-1 -> tick-2 agitation
    // ratio is exactly (1 - decay) = 0.9: the zero-increment witness.
    let tick_two = [
        (
            CLASS_EXPLOITED,
            0.51368,
            0.3658,
            0.12052000000000002,
            0.12150000000000001,
            REVOLUTIONARY,
        ),
        (
            CLASS_BRIBED,
            0.1,
            0.5976,
            0.3024,
            0.008526315789473686,
            LIBERAL,
        ),
        (
            CLASS_EMERGENT,
            0.0171,
            0.9658,
            0.0171,
            0.12150000000000001,
            LIBERAL,
        ),
    ];
    for (id, r, l, f, agitation, dominant) in tick_two {
        let read = |field: &str| {
            graph
                .node_attribute(id, field)
                .unwrap_or_else(|e| panic!("{id:?} {field}: {}", e.message))
        };
        assert_eq!(read("social-class/revolutionary"), r, "{id:?} tick-2 r");
        assert_eq!(read("social-class/liberal"), l, "{id:?} tick-2 l");
        assert_eq!(read("social-class/fascist"), f, "{id:?} tick-2 f");
        assert_eq!(
            read("social-class/agitation"),
            agitation,
            "{id:?} tick-2 agitation: tick-1's store x 0.9 exactly (zero fresh increment)"
        );
        assert_eq!(
            read("social-class/dominant-worldview"),
            dominant,
            "{id:?} tick-2 dominant"
        );
    }

    // The non-flip, named (re-measured, #491 T1 D183, 2026-08-18): the
    // ADR016 fascist-routing vector's readout stays LIBERAL at tick 2 —
    // two ticks of chauvinist-clamped routing (eff_sol 0 both ticks: inbox
    // 0, positive balance 1/11) still erode l (0.6 -> 0.5987 -> 0.5976
    // under f 0.3 -> 0.3013 -> 0.3024), but the erosion is now two orders
    // of magnitude gentler than the frozen absolute-currency reading
    // produced, because C1's proportional rent term makes the SAME $5
    // wealth decline barely register against a $90 class wealth. Under
    // the pre-C1 formula this same test asserted FASCIST here — the flip
    // no longer happens within these two ticks under the corrected,
    // scale-invariant reading; hegemony still erodes, but far more slowly
    // for a wealthy class than a poor one loses the same nominal amount.
    assert_eq!(
        graph
            .node_attribute(CLASS_BRIBED, "social-class/dominant-worldview")
            .expect("bribed dominant at tick 2"),
        LIBERAL,
        "no flip: proportional material loss, not absolute, now drives routing"
    );

    // The solidarity pushes fire again at tick 2 (p3 gates on the source's
    // CURRENT r — class-exploited's tick-1 routed 0.5072 > 0.3), so the
    // inboxes re-accumulate on p1's reset: 0.4 and 0.5 again, and
    // class-bribed's 0.9p stays gate-blocked (its r is still 0.1).
    assert_eq!(
        graph
            .node_attribute(CLASS_EXPLOITED, "social-class/solidarity-inbox")
            .expect("exploited inbox at tick 2"),
        0.4
    );
    assert_eq!(
        graph
            .node_attribute(CLASS_EMERGENT, "social-class/solidarity-inbox")
            .expect("emergent inbox at tick 2"),
        0.5,
        "0.5p again, 0.9p still gate-blocked"
    );

    // The persist machinery, pinned directly: previous-wages now equals the
    // pushed wage flow (9/12/8, still bit-identical to the retired
    // wages-received values — the single-employer exactness the ceremony
    // proves), so tick-2's wage-change was exactly zero — the differential
    // witness for the zero increments above.
    for (id, wage_flow) in [
        (CLASS_EXPLOITED, 9.0),
        (CLASS_BRIBED, 12.0),
        (CLASS_EMERGENT, 8.0),
    ] {
        assert_eq!(
            graph
                .node_attribute(id, "social-class/previous-wages")
                .expect("persisted baseline present"),
            wage_flow,
            "{id:?} previous-wages == the pushed wage flow after tick 1's p7"
        );
    }

    // The non-subjects stay non-subjects across both ticks — except the
    // two inbox carriers (W2 repair, adjudication (d)): p1-inbox-reset is
    // `(when #t)` now, so it fires unconditionally over every SOCIAL_CLASS
    // subject including these two never-positioned, never-routed nodes.
    for id in [CLASS_UNPOSITIONED, EMPLOYER] {
        for field in [
            "social-class/revolutionary",
            "social-class/agitation",
            "social-class/dominant-worldview",
        ] {
            assert!(
                graph.node_attribute(id, field).is_err(),
                "{id:?} {field}: still absent after two ticks"
            );
        }
        assert_eq!(
            graph
                .node_attribute(id, "social-class/solidarity-inbox")
                .expect("W2 repair: unconditional reset writes 0 every tick, non-subject or not"),
            0.0,
            "{id:?} solidarity-inbox: explicitly reset to 0, not absent, after two ticks"
        );
    }
}
