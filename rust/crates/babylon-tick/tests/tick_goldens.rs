//! Widens the tick-hash golden surface from ONE pinned end-to-end hash
//! (`babylon-client/tests/engine_link.rs`'s post-tick hash on
//! two-classes.bscn + fundamental-theorem.bsl) to SIX: pre- and post-tick,
//! on all THREE committed content pairs `babylon-tick`/`babylon-client`
//! ship (two single-rule pairs plus the B2 demo world's two-rule pair).
//! The pre-tick value for the first pair previously appeared only in prose
//! (`docs/superpowers/plans/2026-08-11-hypergraph-storage-swap-plan.md`
//! Task 3); this is where it becomes an executable pin.
//!
//! This surface exists to catch the storage swap — or, for the third pair,
//! ANY unintentional drift in the shipped demo content or the two rule
//! packs it composes — moving a byte the single `engine_link` pin would
//! miss. A change that happened to leave the FINAL hash of ONE content
//! pair unmoved by coincidence would still move at least one of these six.
//! Mutation-proven (adversarial-panel finding FB4, this fix round):
//! tripling `us-counties-lifecycle-demo.bscn`'s county-01011 `pop-d`, and
//! a total-preserving recomposition of county-01001's `pop-d`/`pop-p`
//! (2042/5748 -> 2043/5747), both left every OTHER test in this crate and
//! `babylon-client` green — this golden is what catches either.
//!
//! Measured, never derived: every hash below was produced by running
//! `run_once` once against the committed content and reading the printed
//! value back — see the storage-swap plan, Phase A Task 3, Step 1, for the
//! first two pairs; the third pair's hashes were measured the same way at
//! this fix round's own tip (2026-08-11) and cross-confirmed against the
//! identical post-tick hash `tests/tick_loop.rs`'s own
//! `pressing_space_advances_the_tick_and_updates_the_hash_text` observes
//! through the client's independent `EngineSession` seam.

use babylon_bsl::scenario::{
    compose_declaration_preludes, load_scenario, load_scenario_with_prelude,
};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_tick::{hex, run_once, run_once_with_prelude};

const TWO_CLASSES_SCENARIO: &str = include_str!("../content/scenarios/two-classes.bscn");
const FUNDAMENTAL_THEOREM_RULE: &str = include_str!("../content/rules/fundamental-theorem.bsl");
const VITALITY_SCENARIO: &str = include_str!("../content/scenarios/vitality-conformance.bscn");
const VITALITY_RULE: &str = include_str!("../content/rules/vitality.bsl");
const DEMO_SCENARIO: &str = include_str!("../content/scenarios/us-counties-lifecycle-demo.bscn");
const DEMO_VITALITY_RULE: &str = include_str!("../content/rules/vitality.bsl");
const DEMO_LIFECYCLE_RULE: &str = include_str!("../content/rules/lifecycle.bsl");
const ORG_FOUNDATION_SCENARIO: &str =
    include_str!("../content/scenarios/organization-foundation.bscn");
const ORG_FOUNDATION_RULE: &str = include_str!("../content/rules/organization.bsl");
const TERRITORY_SCENARIO: &str = include_str!("../content/scenarios/territory-conformance.bscn");
const TERRITORY_RULE: &str = include_str!("../content/rules/territory.bsl");
const PRODUCTION_SCENARIO: &str = include_str!("../content/scenarios/production-conformance.bscn");
const PRODUCTION_RULE: &str = include_str!("../content/rules/production.bsl");
const WORLDVIEW_SCENARIO: &str = include_str!("../content/scenarios/worldview-foundation.bscn");
const WORLDVIEW_RULES: &str = include_str!("../content/rules/worldview.bsl");
const CONSCIOUSNESS_TERNARY_SCENARIO: &str =
    include_str!("../content/scenarios/consciousness-ternary-conformance.bscn");
const CONSCIOUSNESS_TERNARY_RULES: &str = include_str!("../content/rules/consciousness.bsl");
const SOLIDARITY_SCENARIO: &str = include_str!("../content/scenarios/solidarity-conformance.bscn");
const SOLIDARITY_RULE: &str = include_str!("../content/rules/solidarity.bsl");
// Train B item 4 (#591, D157): the declaration prelude the consciousness-
// ternary golden now shares its WorldView type through, rather than
// re-declaring it — see the .bscn's own header for the retirement note.
const WORLDVIEW_PRELUDE: &str = include_str!("../content/declarations/worldview.bscn");
const ORGANIZATION_PRACTICE_PRELUDE: &str =
    include_str!("../content/declarations/organization-practice.bscn");

fn practice_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE])
        .expect("the organization practice prelude composes")
}

fn practice_worldview_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE, WORLDVIEW_PRELUDE])
        .expect("the ordered organization and worldview preludes compose")
}

#[test]
fn two_classes_fundamental_theorem_hashes_are_pinned() {
    let report =
        run_once(TWO_CLASSES_SCENARIO, FUNDAMENTAL_THEOREM_RULE).expect("two-classes tick");
    assert_eq!(
        hex(&report.before),
        "5a44ab0c426eca240a0010cc70321bd0ff944d2eee2408454899a942dc85a205",
        "pre-tick hash moved — this is the SUBSTRATE'S load of two-classes.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a554e427679",
        "post-tick hash moved — the same pin babylon-client's engine_link \
         asserts on this content pair"
    );
}

#[test]
fn vitality_conformance_hashes_are_pinned() {
    let report = run_once(VITALITY_SCENARIO, VITALITY_RULE).expect("vitality tick");
    assert_eq!(
        hex(&report.before),
        "20dbc24fc6ba17067cb26eb4ce4c2792c51cb0402395dc55363a5e4e38572fea",
        "pre-tick hash moved — this is the SUBSTRATE'S load of vitality-conformance.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "4c7f95d967e2bf28cd5be91bbd439b61652d2c8d4103e8b5d7a3a8ad789baf64",
        "post-tick hash moved — the vitality pack's engine output"
    );
}

/// The B2 demo world's own golden (adversarial-panel fix FB4): before this
/// test existed, NOTHING pinned `us-counties-lifecycle-demo.bscn`'s content
/// against silent drift — tripling county-01011's `pop-d`, or a
/// total-preserving recomposition of county-01001's `pop-d`/`pop-p`, both
/// left every other test in this crate and `babylon-client` green
/// (mutation-proven, this fix round). `run_once` composes the two rule
/// packs the same way `EngineSession::start`/`us_counties_demo.rs` do —
/// concatenation order is arbitrary because `prepare_rules` compiles the
/// governed phase order — and runs tick 1.
#[test]
fn us_counties_lifecycle_demo_hashes_are_pinned() {
    let rule_src = format!("{DEMO_VITALITY_RULE}\n{DEMO_LIFECYCLE_RULE}");
    let report = run_once(DEMO_SCENARIO, &rule_src).expect("demo tick");
    assert_eq!(
        hex(&report.before),
        "c190053e6d5d6eb261f1325bf87a6347dad8bb99f4e6fb7f2e297d355ccc28ab",
        "pre-tick hash moved — this is the SUBSTRATE'S load of us-counties-lifecycle-demo.bscn \
         (twelve territories + six social classes)"
    );
    assert_eq!(
        hex(&report.after),
        "f4ea98647520ca8e5b2b74e4970626a179236b48efde144c91850c52640f2b5d",
        "post-tick hash moved — both rule packs' combined tick-1 output. Cross-confirmed against \
         the identical value babylon-client's EngineSession seam produces \
         (tests/tick_loop.rs::pressing_space_advances_the_tick_and_updates_the_hash_text)"
    );
}

/// The organization foundation hash anchor. The scenario seeds two
/// organizations, four class-territory pairs, and explicit membership,
/// presence, command, tenancy, attributed adjacency, and solidarity
/// relations. The rule pack spends a bounded practice action, relays rooted
/// capacity through throughput, alternate-capacity, inventory-buffer, and
/// reproduction-dependence fields, and provokes command response. It does not
/// replenish the action or write membership or care without their governed
/// intent, attribution, stock, labor, and routing paths.
///
/// The before hash pins the expanded relational world, including the typed
/// material embedding on each PRESENCE relation. The after hash pins its
/// first material tick. Structural assertions in
/// `organization_practice_conformance.rs` explain the behavior summarized
/// by these bytes.
#[test]
fn organization_foundation_hashes_are_pinned() {
    let report = run_once_with_prelude(
        ORG_FOUNDATION_SCENARIO,
        &practice_prelude(),
        ORG_FOUNDATION_RULE,
    )
    .expect("organization-foundation tick");
    assert_eq!(
        hex(&report.before),
        "9ae0172b0ce63a1d3f5a9e82fce433bb06c516b4fff2ec5056ec588a21b9228f",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         organization-foundation.bscn (the org estate's first entry into \
         the Rust byte gate, spec §11)"
    );
    assert_eq!(
        hex(&report.after),
        "75f8639bc52ee7353f0b7e62f7891f47185a005de57332bddec797d8aaf3da2b",
        "post-tick hash moved — this pins the first material practice, \
         fresh-work propagation, unchanged membership, and command writes"
    );
    assert_eq!(
        hex(&report.world_before),
        "bc5d675cf9e0b2203beede6e861a4c263e5c80da66d3f4835f84c4ceddd33fe4",
        "pre-tick nominal world hash moved — this pins graph state, completed \
         time, allocator cursors, and the governed schedule"
    );
    assert_eq!(
        hex(&report.world_after),
        "21e0abce8e9dbaa83e55a3d2e5dc73235dc172e1cca3b05556beebd1eba7087a",
        "post-tick nominal world hash moved — this pins post-tick graph state, \
         completed time, allocator cursors, and the governed schedule"
    );
    assert_eq!(
        report.fired, 12,
        "the first tick must execute 4 territory resets + 2 presence \
         attributions + 1 rooted practice + \
         1 circulation relay + 3 capacity applications + 1 command response"
    );
}

/// The Territory port's own composition golden (P27 PR B, Task 8): all
/// FIVE `territory/*` rules against the twelve-territory/three-class
/// conformance world in one tick — the port train's entry into the Rust
/// byte gate. `territory_conformance.rs`'s own suite already pins every
/// STRUCTURAL claim this hash summarizes (the latch set, the sink
/// tiebreak, the suppression set, the camp's same-tick decay); this
/// golden exists to catch ANY unintentional drift a structural assertion
/// happens not to cover — the same class of blind spot
/// `us_counties_lifecycle_demo_hashes_are_pinned`'s own header names.
#[test]
fn territory_conformance_hashes_are_pinned() {
    // Re-pinned once (fix round, 2026-08-12) after the fixture-change batch
    // (MINOR-4: sink-reservation seeded nonzero; MINOR-7: the D123
    // directed-walk witness edge; NIT-10: sink-reservation declared before
    // sink-penal) — this golden is new in this PR, so the re-pin is a
    // measurement, not a ceremony (III.13 baseline ceremonies apply to
    // `tests/baselines/**`, not this crate's own goldens).
    let report = run_once(TERRITORY_SCENARIO, TERRITORY_RULE).expect("territory tick");
    assert_eq!(
        hex(&report.before),
        "3794b114d302a8466889795573ecf3f87547af5c200e1ead11c4fc9fcac88ad6",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         territory-conformance.bscn (twelve territories + three social \
         classes + eight edges)"
    );
    assert_eq!(
        hex(&report.after),
        "510091298354429a755e6b851c9db356b2b1d7c35e74d092447535a7883e1af8",
        "post-tick hash moved — all five phase rules' combined tick-1 output"
    );
    assert_eq!(
        report.fired, 30,
        "12 (p1) + 4 (p2) + 12 (p3) + 1 (p4-camp-decay) + 1 (p4-penal-suppression) = 30"
    );
}

/// The Production port's own composition golden (P27, issue #565, Task 5;
/// RE-PINNED, fix round, adversarial verification): all FIVE
/// `production/*` rules against the nine-social-class/five-territory
/// conformance world in one tick — the port train's entry into the Rust
/// byte gate. `production_conformance.rs`'s own suite already pins every
/// STRUCTURAL claim this hash summarizes (the wealth ledger, the employer
/// accumulation, the idle-worker hash-neutral vector, the extraction-
/// intensity broadcast — now matching the frozen mirror's single-territory
/// attribution exactly, the fix round's own discharge of D136); this golden
/// exists to catch ANY unintentional drift a structural assertion happens
/// not to cover — the same class of blind spot
/// `territory_conformance_hashes_are_pinned`'s own header names. The
/// re-pin (both hashes AND the fired count moved: the pack gained a fifth
/// rule, `production/p0-production-total-reset`, and the fixture gained a
/// fifth territory/ninth social class, `t-tight`/`worker-tight`, MINOR-2)
/// is a measurement, not a ceremony (III.13 baseline ceremonies apply to
/// `tests/baselines/**`, not this crate's own goldens) — this golden is new
/// enough in this repo's history that its own prior value was never load-
/// bearing outside this crate. The five OTHER pre-existing pins above are
/// untouched by this content pair's own load/rules.
#[test]
fn production_conformance_hashes_are_pinned() {
    let report = run_once(PRODUCTION_SCENARIO, PRODUCTION_RULE).expect("production tick");
    assert_eq!(
        hex(&report.before),
        "83192431e51d9be36aea347cec0861ebe352e47ee8f9bce4f39840f3e581ad4b",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         production-conformance.bscn (nine social classes + five \
         territories + twelve edges)"
    );
    assert_eq!(
        hex(&report.after),
        "1538162e443afd4b1dcc020bec886e616c91bc680dffce50e52d48df4af8f1eb",
        "post-tick hash moved — all five rules' combined tick-1 output"
    );
    assert_eq!(
        report.fired, 17,
        "5 (p0) + 3 (p1) + 3 (p2) + 1 (p3) + 5 (p4) = 17 — the fix round's own predicted \
         arithmetic, verified rather than trusted"
    );
}

/// The WorldView mint's own golden (ADR204 W10, ceremony ADR206): the
/// substrate LOAD of the mint scenario, byte-pinned. What this pin
/// guards is the world's graph facts — the canonical state hash covers
/// nodes/attributes/edges/hyperedges/edge attributes ONLY, so the
/// `defenum` declaration itself does not move these bytes; the ruled
/// member ORDER (REVOLUTIONARY=0 / LIBERAL=1 / FASCIST=2 — declaration
/// order IS the storage ordinal, ADR195) is guarded by the explicit
/// registry assertion in `worldview_member_order_is_the_ruled_ordinal`
/// below, not by this hash. The pack's one rule is a never-firing load
/// probe (the rule pipeline refuses a zero-rule content set; the idiom
/// is production_conformance.rs's own): the guard is false for every
/// legal population, so `fired == 0` and `before == after` are the mint
/// stage's honest expectations (the measured-ternary consumers land
/// with the port train), NOT a bug — exactly the emit-only logic the
/// organization golden's own header spells out, one step further.
#[test]
fn worldview_foundation_hashes_are_pinned() {
    let report = run_once(WORLDVIEW_SCENARIO, WORLDVIEW_RULES).expect("worldview-foundation tick");
    assert_eq!(
        hex(&report.before),
        "098ef6bd62ebc072de94d370242430d84b1b8cf2223b3b190b359ed6e871edbf",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         worldview-foundation.bscn (the mint world's graph-fact pin)"
    );
    assert_eq!(
        hex(&report.after),
        "098ef6bd62ebc072de94d370242430d84b1b8cf2223b3b190b359ed6e871edbf",
        "post-tick hash moved — the probe rule never fires, so this \
         equals `before` by construction; a divergence here means the \
         tick mutated state without a firing rule, which is its own bug"
    );
    assert_eq!(
        report.fired, 0,
        "the worldview mint pack's load probe never fires (its guard is \
         false for every legal population)"
    );
}

/// The ruled ordinal order, guarded EXPLICITLY (task-review finding,
/// plan amended 2026-08-15): the canonical state hash covers graph
/// facts only, so the `defenum` declaration never moves the byte pin
/// above — THIS registry assertion, not the hash, is what guards
/// REVOLUTIONARY=0 / LIBERAL=1 / FASCIST=2. Declaration order IS the
/// storage ordinal (ADR195); a reordered, renamed, or dropped member
/// fails here loudly.
#[test]
fn worldview_member_order_is_the_ruled_ordinal() {
    let mut graph = HypergraphStore::new();
    let loaded =
        load_scenario(WORLDVIEW_SCENARIO, &mut graph).expect("worldview-foundation loads clean");
    let ty = loaded
        .enums
        .resolve("WorldView")
        .expect("the WorldView defenum is declared");
    assert_eq!(loaded.enums.ordinal(ty, "REVOLUTIONARY"), Some(0));
    assert_eq!(loaded.enums.ordinal(ty, "LIBERAL"), Some(1));
    assert_eq!(loaded.enums.ordinal(ty, "FASCIST"), Some(2));
}

/// The consciousness class-surface ternary port's own composition golden
/// (ADR204 W10, issue #588, Tasks 1-2): the FIRST committed `probability`-lane
/// deffields, the pack's first rule (`consciousness/p0-position`), and the
/// measured readout (`consciousness/p8-dominant-worldview`), against the
/// thirteen-class-plus-org conformance world in one tick — the
/// port train's entry into the Rust byte gate. The
/// `consciousness_ternary_conformance.rs` posture test already pins every
/// STRUCTURAL claim this hash summarizes (the UNPOSITIONED witness's loud
/// absence, the exact (0, 1, 0) positioning, the untouched positioned
/// seeds); this golden exists to catch ANY unintentional drift a structural
/// assertion happens not to cover — the same class of blind spot
/// `territory_conformance_hashes_are_pinned`'s own header names. RE-PINNED
/// four times in this train (Task-1 fix round 1, controller rulings 1-3:
/// the dominant write came out, the WAGES edge machinery was replaced by
/// the class-side wages-received field, the micros seeds went raw; Task-1
/// fix round 2: agitation re-typed to the verbatim-f64 int lane with zero
/// seeds; Task 2: the p8-dominant-worldview read path landed and the world
/// gained the eight tv-* read-path fixtures, moving both hashes and the
/// fired count 1 -> 12; Task 3: the measured update law p1..p7 landed —
/// the routed ternaries, the produced agitation, the inboxes (p1's reset
/// writes 0 onto the eight tv-* fixtures' absent solidarity-inbox: the
/// controller-flagged hash-visible machinery write), the wage balances, the
/// persisted baselines — moving the post-tick hash and the fired count
/// 12 -> 50; the pre-tick hash is unchanged) — this golden was minted in
/// this train, so the re-pins are measurements, not ceremonies (III.13
/// baseline ceremonies apply to `tests/baselines/**`, not this crate's own
/// goldens). Every structural claim the post-tick hash summarizes is pinned
/// exactly in `consciousness_ternary_conformance.rs` against the
/// dual-implementation oracle.
///
/// FIFTH RE-PIN — Train B item 3 (issue #591, D151's discharge): WAGES
/// edges + `wages/value-flow` seeding restored the frozen fold-sum via
/// push-over-pull; `wages-received` retired; BOTH hashes re-pinned
/// (attribute-set change, zero value drift — proven by
/// `consciousness_ternary_conformance.rs` passing with only the two field
/// re-points); fired 50 -> 63. The attribute-set delta is exactly the
/// brief's own predicted shape, verified rather than assumed: pre-tick
/// state differs ONLY by −3 `wages-received` node attributes, +3 WAGES
/// edges, +3 `wages/value-flow` edge attributes (+0 `wages-inbox` —
/// unseeded); post-tick state differs ONLY by −`wages-received`,
/// +`wages-inbox` on the reset/pushed classes. The fired-count spike (Step
/// 3, measured against the EXPECTED "employer only, +1" hypothesis): the
/// mirrored `active`-gated shape does not discriminate on WAGES-edge
/// presence, so all thirteen SOCIAL_CLASS subjects fire (not one) — the
/// measured arithmetic is 50 + 13 = 63 (tick 1), 49 + 13 = 62 (tick 2, p0
/// not re-firing) — recorded honestly per the house pattern this file's own
/// `production_conformance_hashes_are_pinned` header sets.
///
/// LOAD-MECHANISM CHANGE ONLY, HASHES UNCHANGED — Train B item 4 (issue
/// #591, D157): the `.bscn`'s own `(defenum WorldView …)` re-declaration
/// came out; the WorldView type now arrives from the shared declaration
/// prelude (`content/declarations/worldview.bscn`) via
/// `run_once_with_prelude`, and the scenario's `deffield … enum WorldView`
/// resolves against the SAME registry contents it always did
/// (`EnumRegistry::declare`'s identical-recognition arm is not even reached
/// here — the `.bscn` no longer re-declares at all). `defenum` declarations
/// are unhashed and the graph content is byte-identical, so BOTH hashes and
/// `fired` (63) below are UNCHANGED from the fifth re-pin — verified, not
/// assumed (this is exactly the byte-neutrality proof the brief demanded;
/// see `consciousness_ternary_conformance.rs` for the companion
/// value-level proof).
///
/// TASK 5 PRACTICE PRELUDE — the scenario now composes the organization
/// practice declarations before the WorldView prelude. Its already-active
/// `org-solid` witness also seeds the promoted `organization/action-budget`
/// field at 1. That explicit graph field moves both hashes below; the
/// conformance suite proves that rule firings and material results stay
/// unchanged.
#[test]
fn consciousness_ternary_foundation_hashes_are_pinned() {
    let report = run_once_with_prelude(
        CONSCIOUSNESS_TERNARY_SCENARIO,
        &practice_worldview_prelude(),
        CONSCIOUSNESS_TERNARY_RULES,
    )
    .expect("consciousness-ternary tick");
    assert_eq!(
        hex(&report.before),
        "bd881846b193228281d5be66a7083ded2aab2e9790b1fc86a9103a04f5629363",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         consciousness-ternary-conformance.bscn (thirteen social classes + one \
         organization + three SOLIDARITY edges + three WAGES edges — the \
         Task 5 practice-prelude measurement)"
    );
    assert_eq!(
        hex(&report.after),
        "a59bf3721c35face89c02ec5a7e8bf79d3bf994927bb7d63e3e6f5cacf3ccd17",
        "post-tick hash moved — the ten-rule pack's combined tick-1 output \
         (p0's positioning, p1..p7's measured update law plus the new \
         p2-wages-push, p8's readout) — attribute-set change only, zero \
         value drift (Train B item 3's re-pin). RE-PINNED (#491 T1, D183, \
         controller adjudication 2026-08-18: C1/p5-agitation's proportional \
         rent term, licensed by D181, plus D182/D183's product/division \
         licensing that unblocked p6-route's load path): class-bribed's \
         agitation/l/f VALUES moved (not the attribute set) — see \
         consciousness_ternary_conformance.rs's re-measured assertions and \
         its module-header post-repair note for the exact numbers and the \
         ternary-routing consequence (the tick-2 FASCIST flip no longer \
         happens). RE-PINNED AGAIN 2026-08-21 in the \
         worktree-sweep integration merge (bsl-hygiene-knockout × #491): \
         W2's unconditional p1-inbox-reset guard (adjudication (d), the \
         SIXTH re-pin on the hygiene train) composes with T1's value moves \
         — the two never-positioned nodes gain explicit solidarity-inbox/ \
         wages-inbox 0 writes on top of the #491 text. Measured on the \
         merged tree; both parent pins (52ffb5e3…, 4d983944…) superseded."
    );
    assert_eq!(
        report.fired, 65,
        "p0:1 + p1:13 (W2 repair, adjudication (d): unconditional \
         `(when #t)`, every SOCIAL_CLASS subject — was 11, positioned-only, \
         before the repair) + p2:1 + \
         p2-wages-push:13 (every active SOCIAL_CLASS subject — the mirrored \
         active gate does not discriminate on WAGES-edge presence, so the \
         for-each idiom fires on edgeless subjects too; only employer's \
         three WAGES edges write) + p3:6 (r > 0.3 sources) + p4:3 + p5:3 + \
         p6:11 + p7:3 + p8:11 — the per-rule breakdown is pinned in \
         consciousness_ternary_conformance.rs"
    );
}

/// The prelude's own ordinal, guarded EXPLICITLY — final whole-branch
/// review item 2 (#591): the declared test death below rests on a claim
/// ("byte-identical to the mint's declaration") that only a comment
/// enforced. `worldview_member_order_is_the_ruled_ordinal` above loads
/// ONLY `WORLDVIEW_SCENARIO` (the mint) — it never loads
/// `WORLDVIEW_PRELUDE`, which is what `consciousness_ternary_foundation_
/// hashes_are_pinned` above ACTUALLY consumes via `run_once_with_prelude`.
/// This is the same failure mode Task 3's F1 caught one PR earlier (a
/// byte-identity claim guarded only by a comment), recurring unnoticed
/// inside the same train. This test closes it by asserting the three
/// ordinals as declared by `WORLDVIEW_PRELUDE` itself, loaded through the
/// real loader (`load_scenario_with_prelude`) against a minimal probe
/// scenario — not by prose about what the mint's line happens to match.
#[test]
fn worldview_prelude_member_order_is_the_ruled_ordinal() {
    let mut graph = HypergraphStore::new();
    let loaded =
        load_scenario_with_prelude(WORLDVIEW_PRELUDE, "(scenario t/ordinal-probe)", &mut graph)
            .expect("the worldview prelude loads clean against an empty probe scenario");
    let ty = loaded
        .enums
        .resolve("WorldView")
        .expect("the WorldView defenum is declared by the prelude");
    assert_eq!(loaded.enums.ordinal(ty, "REVOLUTIONARY"), Some(0));
    assert_eq!(loaded.enums.ordinal(ty, "LIBERAL"), Some(1));
    assert_eq!(loaded.enums.ordinal(ty, "FASCIST"), Some(2));
}

// The `consciousness_ternary_worldview_member_order_is_the_ruled_ordinal`
// test that lived here (the ternary port's own re-declaration guard) is a
// DECLARED TEST DEATH — Train B item 4 (#591, D157): the prelude
// composition (above) makes the re-declaration it guarded IMPOSSIBLE (the
// `.bscn` no longer declares `WorldView` at all), so the assertion it made
// has no subject left to guard. `worldview_prelude_member_order_is_the_
// ruled_ordinal` (immediately above) is the ordinal home for what this
// golden's tick actually reads — the prelude — asserted executably rather
// than by a comment's claim that the mint's line happens to match it.

/// The Solidarity port train's own composition golden (issue #557 umbrella,
/// Task 4): the ONE `solidarity/p0-transmit` rule against the
/// twenty-two-social-class conformance world in one tick — the port
/// train's entry into the Rust byte gate. `solidarity_conformance.rs`'s own
/// suite already pins every STRUCTURAL claim this hash summarizes (every
/// witness target's post-tick value, the three skip gates, the
/// multi-inbound last-write-wins divergence from frozen D-record 2, the
/// clamp, the exact-0.6 boundary, the nine ordered CONSCIOUSNESS_TRANSMISSION
/// / MASS_AWAKENING events) against the dual-implementation oracle
/// (`content/scenarios/solidarity_conformance.py`, Task 4); this golden
/// exists to catch ANY unintentional drift a structural assertion happens
/// not to cover — the same class of blind spot
/// `territory_conformance_hashes_are_pinned`'s own header names. Measured,
/// never derived (`tick_goldens.rs`'s own doctrine, lines 21-23 above): run
/// once, `hex(&report.before)`/`hex(&report.after)` read back and pasted
/// here verbatim. New in this train, so this is a measurement, not a
/// ceremony (III.13 baseline ceremonies apply to `tests/baselines/**`, not
/// this crate's own goldens); it touches none of the 8 existing pins above.
#[test]
fn solidarity_conformance_hashes_are_pinned() {
    let report = run_once(SOLIDARITY_SCENARIO, SOLIDARITY_RULE).expect("solidarity tick");
    assert_eq!(
        hex(&report.before),
        "20124f5ca91da3cb30fba41bc373175fdf3b06dc82f3c3b162da172951bb29de",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         solidarity-conformance.bscn (twenty-two social classes + twelve \
         SOLIDARITY edges)"
    );
    assert_eq!(
        hex(&report.after),
        "62212dab6bdc255f334eca1ff2260e2ad939776f15aee744ae771ab1be30b3d1",
        "post-tick hash moved — the one-rule pack's tick-1 output (fourteen \
         subjects fire, nine transmit-or-awaken events, one multi-inbound \
         last-write-wins divergence from frozen). RE-PINNED (#491 T1 S1, \
         Director sitting 2026-08-18: repair-now+ceremony): p0-transmit's \
         write is now a kind-coherent convex combination, algebraically \
         identical to `target + delta` but not bit-identical for the \
         multi-inbound witness's non-power-of-2 strength (0.3) — a 1-ULP \
         rounding-order drift on exactly two attribute bytes \
         (multi-target's stored value and one CONSCIOUSNESS_TRANSMISSION \
         payload field), predicted in \
         reports/kind-straddle-repair-options-2026-08-18.md §2.1 and \
         confirmed by solidarity_conformance.rs's own updated assertions. \
         report.before is UNCHANGED (no new graph content, only the rule \
         text moved)."
    );
    assert_eq!(
        report.fired, 14,
        "14 of 22 witness nodes have active=1 and revolutionary > \
         solidarity/activation-threshold (0.3) — solidarity_conformance.rs's own \
         the_conformance_world_loads_with_the_declared_census pins the same count"
    );
}

const DECOMPOSITION_SCENARIO: &str =
    include_str!("../content/scenarios/decomposition-conformance.bscn");
const DECOMPOSITION_DELAY_SCENARIO: &str =
    include_str!("../content/scenarios/decomposition-delay-conformance.bscn");
const DECOMPOSITION_RULE: &str = include_str!("../content/rules/decomposition.bsl");

/// The Decomposition port's own composition golden (Checkpoint A campaign,
/// #591 family, Task 4 — closes Pack A): all SIX `decomposition/*` rules
/// against the fallback-trigger conformance world (`la-dying`'s wealth 400
/// already below subsistence 500 at tick 1, so `CLASS_DECOMPOSITION` fires
/// the SAME tick as the SUPERWAGE_CRISIS early warning —
/// `decomposition_conformance.rs`'s own module doc explains why the frozen
/// mirror shows both firing together). `decomposition_conformance.rs`'s own
/// suite already pins every STRUCTURAL claim this hash summarizes (the
/// additive enforcer intake, the overwrite IP intake, the non-conserving LA
/// deactivation, the flattened CLASS_DECOMPOSITION payload); this golden
/// exists to catch ANY unintentional drift a structural assertion happens
/// not to cover — the same class of blind spot
/// `territory_conformance_hashes_are_pinned`'s own header names.
#[test]
fn decomposition_conformance_hashes_are_pinned() {
    let report = run_once(DECOMPOSITION_SCENARIO, DECOMPOSITION_RULE).expect("decomposition tick");
    assert_eq!(
        hex(&report.before),
        "4001e15449fbf467624417f3c4a9cca22e27bdea3320c81669808c5940a7eb8a",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         decomposition-conformance.bscn (five social classes + one carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "6bcc49d18b1e2494adf96bada45425616b955373293494d314ecdf20679d9b0f",
        "post-tick hash moved — all six rules' combined tick-1 output: p01's \
         census, p02's SUPERWAGE_CRISIS latch, p03's fallback-triggered fire, \
         p04/p05's intake, p06's LA deactivation"
    );
    assert_eq!(
        report.fired, 10,
        "p01:5 (every SOCIAL_CLASS subject, D127 unconditional) + p02:1 \
         (la-dying only) + p03:1 (the one carrier) + p04:1 (enforcer-seed) \
         + p05:1 (ip-seed) + p06:1 (la-dying)"
    );
}

/// The Decomposition port's delay-path companion golden (Task 4 Step 4):
/// ONE tick against `decomposition-delay-conformance.bscn` — the early
/// warning fires, but `CLASS_DECOMPOSITION` does not (the 52-tick delay has
/// not elapsed at tick 1), so p04-p06 never match their `fire-tick == tick`
/// gate this tick. `decomposition_conformance.rs`'s own
/// `the_delay_path_emits_the_warning_at_tick_1_and_decomposes_at_tick_53`/
/// `the_delay_path_does_not_decompose_at_tick_52` pin the full multi-tick
/// lifecycle structurally; this golden pins tick 1 alone against silent
/// drift, the same role every other pair in this file plays.
#[test]
fn decomposition_delay_conformance_hashes_are_pinned() {
    let report = run_once(DECOMPOSITION_DELAY_SCENARIO, DECOMPOSITION_RULE)
        .expect("decomposition-delay tick");
    assert_eq!(
        hex(&report.before),
        "40f0facb177fb535af415f99f70244663cc0ffe4fc26352efc91d308301f5e1e",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         decomposition-delay-conformance.bscn (six social classes + one \
         carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "0eaf7f1459559645510efd57c71739f3ef8813409f3944b9eba51492d141748b",
        "post-tick hash moved — the pack's tick-1 output: only p01's census \
         and p02's SUPERWAGE_CRISIS latch write anything (p03 folds but does \
         not fire; p04-p06 never match fire-tick == tick this tick)"
    );
    assert_eq!(
        report.fired, 8,
        "p01:6 (every SOCIAL_CLASS subject, D127 unconditional) + p02:1 \
         (la-approaching only) + p03:1 (the one carrier) + p04:0 + p05:0 + \
         p06:0 (should-fire is false — the delay has not elapsed)"
    );
}

// ---------------------------------------------------------------------
// Task 8 — the remaining golden pins: the four Pack B scenarios plus the
// joint five-phase arc (Checkpoint A campaign, #591 family). The 11
// pre-existing pins above (verified by direct count, `tick_goldens.rs`'s
// own `fn .*hashes_are_pinned` tally — 9 pre-Pack-A + Pack A's 2) stay
// byte-identical; none of Task 8's own edits touch any content pair above
// this comment.
// ---------------------------------------------------------------------

const CONTROL_RATIO_SCENARIO: &str =
    include_str!("../content/scenarios/control-ratio-conformance.bscn");
const CONTROL_RATIO_REVOLUTION_SCENARIO: &str =
    include_str!("../content/scenarios/control-ratio-revolution-conformance.bscn");
const CONTROL_RATIO_WITHIN_CAPACITY_SCENARIO: &str =
    include_str!("../content/scenarios/control-ratio-within-capacity-conformance.bscn");
const CONTROL_RATIO_ZERO_ENFORCER_SCENARIO: &str =
    include_str!("../content/scenarios/control-ratio-zero-enforcer-conformance.bscn");
const CONTROL_RATIO_RULE: &str = include_str!("../content/rules/control-ratio.bsl");

/// The ControlRatio Pack B primary golden (Task 8, closes the remaining
/// Pack B pins): all FOUR `control-ratio/*` rules against the PRIMARY
/// (genocide) conformance world in one tick — this pack's own entry into
/// the Rust byte gate, alongside `decomposition_conformance_hashes_are_
/// pinned`'s own role for Pack A. `control_ratio_conformance.rs`'s own
/// suite already pins every STRUCTURAL claim this hash summarizes (the
/// two-role prisoner census, the `<=` boundary, the guard-split emit, the
/// terminal-decision routing); this golden exists to catch ANY
/// unintentional drift a structural assertion happens not to cover — the
/// same class of blind spot `territory_conformance_hashes_are_pinned`'s
/// own header names.
#[test]
fn control_ratio_conformance_hashes_are_pinned() {
    let report = run_once(CONTROL_RATIO_SCENARIO, CONTROL_RATIO_RULE).expect("control-ratio tick");
    assert_eq!(
        hex(&report.before),
        "54f7a559a3c047561979994bd058460a3bd12ba361511117bb5227a32f4ad583",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         control-ratio-conformance.bscn (six social classes + one carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "cececdab38bc6ba483baf60ee4df32cb4043073ce18fdd54ce9c866c922b6e5b",
        "post-tick hash moved — all four rules' combined tick-1 output: c01's \
         census, c02's carrier publication, c03's CONTROL_RATIO_CRISIS, c04's \
         GENOCIDE TERMINAL_DECISION"
    );
    assert_eq!(
        report.fired, 9,
        "c01:6 (every SOCIAL_CLASS subject, D127 unconditional) + c02:1 \
         (the one carrier) + c03:1 (the crisis) + c04:1 (the terminal decision)"
    );
}

/// The ControlRatio Pack B revolution companion golden — identical
/// structure to the primary world, organization 0.2 -> 0.6, routing
/// REVOLUTION instead of GENOCIDE.
#[test]
fn control_ratio_revolution_conformance_hashes_are_pinned() {
    let report = run_once(CONTROL_RATIO_REVOLUTION_SCENARIO, CONTROL_RATIO_RULE)
        .expect("control-ratio-revolution tick");
    assert_eq!(
        hex(&report.before),
        "af67a81e16e480adfc621e8617eb1edef99921a45e67b5544451d8f10edc4c1f",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         control-ratio-revolution-conformance.bscn (six social classes + one carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "0ebd2a90c4868a84dd8547c5c37a99fd44cd612f2cbc53c06163847e7c34cb0a",
        "post-tick hash moved — all four rules' combined tick-1 output, \
         routing REVOLUTION (organization 0.6 >= revolution-threshold 0.5)"
    );
    assert_eq!(
        report.fired, 9,
        "c01:6 + c02:1 + c03:1 (the crisis) + c04:1 (the terminal decision)"
    );
}

/// The ControlRatio Pack B within-capacity companion golden — the `<=`
/// boundary (prisoner population 40 == enforcer population 10 *
/// control-capacity 4 exactly): NO crisis, so c03/c04 never fire this tick.
#[test]
fn control_ratio_within_capacity_conformance_hashes_are_pinned() {
    let report = run_once(CONTROL_RATIO_WITHIN_CAPACITY_SCENARIO, CONTROL_RATIO_RULE)
        .expect("control-ratio-within-capacity tick");
    assert_eq!(
        hex(&report.before),
        "f4c8d6b0a12047e713ec3d995cb70f519a4136dadb852192116d237ecdb0834a",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         control-ratio-within-capacity-conformance.bscn (three social classes + one carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "67aa4f7bfcc2ad807331354ea786001a6dc46a7ea5a7514c87ad963f90860470",
        "post-tick hash moved — c01/c02's own tick-1 output; c03's `when` \
         (prisoner-population > max-controllable) is false at the exact `<=` \
         boundary, so c03/c04 never fire"
    );
    assert_eq!(
        report.fired, 4,
        "c01:3 (every SOCIAL_CLASS subject) + c02:1 (the one carrier) + \
         c03:0 + c04:0 (within capacity — no crisis, no terminal decision)"
    );
}

/// The ControlRatio Pack B zero-enforcer companion golden — BLOCKER-4's
/// guard-split branch: a REAL, active, zero-population CARCERAL_ENFORCER
/// class (not an absent one), so `max-controllable` is 0 and ANY nonzero
/// prisoner population clears the `<=` boundary; the crisis payload omits
/// `actual-ratio`/`control-ratio` entirely (loud absence, not `float("inf")`).
#[test]
fn control_ratio_zero_enforcer_conformance_hashes_are_pinned() {
    let report = run_once(CONTROL_RATIO_ZERO_ENFORCER_SCENARIO, CONTROL_RATIO_RULE)
        .expect("control-ratio-zero-enforcer tick");
    assert_eq!(
        hex(&report.before),
        "62f02edb2de87305b34ec7efd5b0a638929300a60ac8473aace3e9b86ccad100",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         control-ratio-zero-enforcer-conformance.bscn (three social classes + one carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "897c1939b9f798026ddc41d9732b0b676a0b628f00b8a845a1c8261d5f725204",
        "post-tick hash moved — all four rules' combined tick-1 output, the \
         zero-enforcer guard-split branch (BLOCKER-4), routing GENOCIDE \
         (organization 0.4 < revolution-threshold 0.5)"
    );
    assert_eq!(
        report.fired, 6,
        "c01:3 (every SOCIAL_CLASS subject) + c02:1 + c03:1 (the crisis, \
         zero-enforcer branch) + c04:1 (the terminal decision) — three \
         SOCIAL_CLASS nodes here, not six (unlike the primary/revolution \
         worlds), so c01's own count is half theirs"
    );
}

const VITALITY_ATTRITION_SCENARIO: &str =
    include_str!("../content/scenarios/vitality-attrition-conformance.bscn");
const VITALITY_ATTRITION_RULE: &str = include_str!("../content/rules/vitality-attrition.bsl");

/// The K=16 wealth-mass carrier's own golden (#491 T4, Phase 1 — "the
/// carrier, inert"; ADR194 R1). `vitality_attrition_conformance.rs`'s own
/// posture suite already pins every STRUCTURAL claim this hash summarizes
/// (the exact-1.0 mass sums, the absence fence, cut monotonicity/
/// positivity, η/τ's ruled values, the Currency-lane round-trip); this
/// golden exists to catch ANY unintentional drift a structural assertion
/// happens not to cover — the same class of blind spot
/// `territory_conformance_hashes_are_pinned`'s own header names.
///
/// **POST-T6 (2026-08-21, #491 Phase 3b): the `before == after` narrative
/// below is HISTORICAL — T6's `vitality/subsistence-mortality` writes state
/// (the population decrement), so after ≠ before now and the pins/messages
/// below were re-measured. The T4/T5-era reasoning is kept verbatim because
/// it explains why the earlier pins had the values they did.**
///
/// `before == after` here is not a bug, and **stays true post-T5** for a
/// DIFFERENT reason than T4's own probe gave it: `vitality-attrition.bsl`
/// no longer carries a never-firing probe (T5, Phase 3a, replaced it with
/// `vitality/subsistence-clearing`, the dual measure `clearing`/
/// `failing_certain`/`straddle_band`) — the rule fires for FOUR of the six
/// classes now (`report.fired == 4`, updated below from T4's `0`), but its
/// only effect is `emit`, which never touches graph state (III.11: no
/// `update-node`/`update-edge`/`update-hyperedge`/`add-*`/`remove-*` verb
/// appears in the rule at all — `vitality_attrition_conformance.rs`'s own
/// T5 suite reads the emitted events, not post-tick graph state, for
/// exactly this reason). So this tick still moves no STATE even though it
/// is no longer a load-only smoke — T5.7's own framing, "a binding and a
/// condition, no effect." What this pin actually guards is the substrate LOAD:
/// six social classes, the Currency-lane re-seed of `wealth`/`s-bio`/
/// `s-class` (T3, OQ-J — this is the FIRST conformance world in this crate
/// to declare a `currency` node attribute, so `CanonicalState` section
/// `0x06` (D189/D190) materializes for it; this pin is that section's
/// first real-content byte measurement, not derived from any pre-existing
/// pin above), the sixteen-mass carrier (five explicit 16-value vectors
/// plus one class carrying none at all), the fifteen grid-cut defconsts,
/// and η/τ. Measured, never derived: `run_once` against the committed
/// content, `hex(&report.before)`/`hex(&report.after)` read back and
/// pasted here verbatim (`tick_goldens.rs`'s own doctrine, lines 21-23
/// above). New in this train, so this is a measurement, not a ceremony
/// (III.13 baseline ceremonies apply to `tests/baselines/**`, not this
/// crate's own goldens); the SIXTEEN pre-existing pins above (verified by
/// direct count against this checkout's BASE, `ec3e1867` — sixteen
/// `fn .*hashes_are_pinned` tests, eighteen `#[test]` functions total
/// counting the two ordinal guards) stay byte-identical, proven by running
/// this crate's full suite both before and after this pin's own addition.
#[test]
fn vitality_attrition_carrier_hashes_are_pinned() {
    let report = run_once(VITALITY_ATTRITION_SCENARIO, VITALITY_ATTRITION_RULE)
        .expect("vitality-attrition carrier tick");
    assert_eq!(
        hex(&report.before),
        "82c15036a47d5779d84a76b510596967e1b50acb26e68c9203c87567645bc2e5",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         vitality-attrition-conformance.bscn. RE-MEASURED at T6 (2026-08-21, \
         #491 Phase 3b): the world gained the `calibration` node and the \
         `vitality/kappa` defconst (the T6.3 derivation, D198) — seven \
         social classes now, the Currency-lane re-seed, the K=16 mass \
         carrier, the fifteen cuts, η and τ; section 0x06's real-content \
         byte measurement stands. Prior pin d93402d6…, superseded."
    );
    assert_eq!(
        hex(&report.after),
        "05d3e664adb5023f6159d1a99b953a7d1ca838fca6a75fef76ead2d767e0ea00",
        "post-tick hash — T6 (2026-08-21): the pack now WRITES state by \
         design (`vitality/subsistence-mortality`'s population decrement, \
         the T6.6 re-measure, no ceremony — III.13 covers tests/baselines/**, \
         not this crate's goldens), so `after` no longer equals `before`; \
         the T5-only emit-only pin this replaces was \
         d93402d6… (= before), superseded. The exact writes are pinned by \
         `vitality_attrition_conformance.rs`'s T6 suite (deaths, remaining \
         population, never wealth)."
    );
    assert_eq!(
        report.fired, 11,
        "5 (the measure: core, bourgeoisie, hermit, last-worker, \
         calibration) + 6 (the mortality rule's guard is active × \
         population only, so remnant fires too, its effects inner-guarded \
         away; dissolved passes neither guard) — measured on the merged T6 \
         world 2026-08-21"
    );
}

const CARCERAL_ARC_SCENARIO: &str =
    include_str!("../content/scenarios/carceral-arc-conformance.bscn");

/// The joint carceral arc's own composition golden (Task 8, the train's
/// acceptance test) — TICK 1 ALONE, the same single-tick convention every
/// other pair in this file follows: `carceral_arc_conformance.rs`'s own
/// multi-tick `TickSession` suite already pins the FULL five-phase
/// sequence (ticks 1/53/105/106) structurally; this golden exists to catch
/// ANY unintentional tick-1 drift a structural assertion happens not to
/// cover — the same class of blind spot `territory_conformance_hashes_are_
/// pinned`'s own header names, applied to the concatenation of BOTH packs
/// for the first time in this crate's own golden surface.
#[test]
fn carceral_arc_conformance_hashes_are_pinned() {
    let rule_src = format!("{DECOMPOSITION_RULE}\n{CONTROL_RATIO_RULE}");
    let report = run_once(CARCERAL_ARC_SCENARIO, &rule_src).expect("carceral-arc tick");
    assert_eq!(
        hex(&report.before),
        "504a4515c4e6d4d4c369a535c58a21ab98e8ee37ba852819c7b4893473881e74",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         carceral-arc-conformance.bscn (five social classes + one carrier)"
    );
    assert_eq!(
        hex(&report.after),
        "04b2a84623e25fdf7fd7761e3c591baa8b42aa96300c76b02caca59e0c74b3d6",
        "post-tick hash moved — BOTH packs' combined tick-1 output: c01-c04's \
         readiness gate stays closed (decomposition-fired-known unwritten \
         until p03 runs, later in byte order), p01's census, p02's \
         SUPERWAGE_CRISIS latch, p03's carrier fold (should-fire false at \
         tick 1 — the delay path)"
    );
    assert_eq!(
        report.fired, 13,
        "c01:5 (every SOCIAL_CLASS subject) + c02:1 (the carrier) + c03:0 \
         (readiness gate closed) + c04:0 + p01:5 (every SOCIAL_CLASS \
         subject) + p02:1 (la-approaching's early warning) + p03:1 (the \
         carrier fold, always fires, should-fire false) + p04:0 + p05:0 + \
         p06:0 (fire-tick never reaches tick 1 on the delay path)"
    );
}

// ---------------------------------------------------------------------
// The Community port train's eight content-world pins (issue #667, Task
// 10 Step 6) — one per world (1, 2, 3, 4, 5, 5b, 5c, 6), each pinning
// `before` and `after` in one test as the landed pins do (the arc's
// `after` is its tick-3 value). Every one carries the triad: what it
// summarizes that the conformance suites already pin; that it is
// MEASURED, never derived — new in this train, so this is a measurement,
// not a ceremony (III.13 applies to tests/baselines/**, not this crate's
// own goldens); and that it touches none of the sixteen prior pins.
// ---------------------------------------------------------------------

const COMMUNITY_W1: &str = include_str!("../content/scenarios/community-conformance.bscn");
const COMMUNITY_W2: &str = include_str!("../content/scenarios/community-floor-conformance.bscn");
const COMMUNITY_W3: &str =
    include_str!("../content/scenarios/community-degenerate-conformance.bscn");
const COMMUNITY_W4: &str =
    include_str!("../content/scenarios/community-cost-modifier-conformance.bscn");
const COMMUNITY_W5: &str =
    include_str!("../content/scenarios/community-decay-arc-conformance.bscn");
const COMMUNITY_W5B: &str =
    include_str!("../content/scenarios/community-solidarity-seam-conformance.bscn");
const COMMUNITY_W5C: &str =
    include_str!("../content/scenarios/community-carrier-collision-conformance.bscn");
const COMMUNITY_W6: &str = include_str!("../content/scenarios/community-empty-conformance.bscn");
const COMMUNITY_PACK: &str = include_str!("../content/rules/community.bsl");
const COMMUNITY_TIE: &str = include_str!("../content/scenarios/community-tie-conformance.bscn");
const COMMUNITY_SOLIDARITY: &str = include_str!("../content/rules/solidarity.bsl");
const COMMUNITY_CONTROL_RATIO: &str = include_str!("../content/rules/control-ratio.bsl");

/// World 1 — summarizes `community_conformance.rs`'s census/weights/
/// normalization/floor pins (already bit-exact against the mirror).
/// Measured, never derived; touches none of the sixteen prior pins.
#[test]
fn community_world_1_hashes_are_pinned() {
    let report = run_once_with_prelude(COMMUNITY_W1, &practice_prelude(), COMMUNITY_PACK)
        .expect("world 1 tick");
    assert_eq!(
        hex(&report.before),
        "855f6f9b92a47b909f7d470aa84556c9ac48a319fff0905c142ee58199a392ba",
        "pre-tick hash moved — the substrate's load of community-conformance.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "b40cb0de99238850396698f731b06c707d33c23b8df14e173e6c40ab12a7e17a",
        "post-tick hash moved — the full c00-c11 pack over world 1"
    );
}

/// World 2 — summarizes the floor-binding pins (Ruling 3's ordering
/// executed, the proportionality read). Measured, never derived; touches
/// none of the sixteen prior pins.
#[test]
fn community_world_2_hashes_are_pinned() {
    let report = run_once_with_prelude(COMMUNITY_W2, &practice_prelude(), COMMUNITY_PACK)
        .expect("world 2 tick");
    assert_eq!(
        hex(&report.before),
        "74c94d50d41bbe816d3f0de17956162d9698aad36a39f2ecf006b169e17eeb6b",
        "pre-tick hash moved — the substrate's load of community-floor-conformance.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "d5c93a6f1fb5622172d4c63ce896235f7c65f5ff9a5ab1d3ed64567089735cd0",
        "post-tick hash moved — the floor world after one tick"
    );
}

/// World 3 — summarizes the degenerate + skip-gate pins. Measured, never
/// derived; touches none of the sixteen prior pins.
#[test]
fn community_world_3_hashes_are_pinned() {
    let report = run_once_with_prelude(COMMUNITY_W3, &practice_prelude(), COMMUNITY_PACK)
        .expect("world 3 tick");
    assert_eq!(
        hex(&report.before),
        "cbc85aab2f12b2858ae215e896fe72e2343471b7e40f3e79b4360f321b40c83d",
        "pre-tick hash moved — the substrate's load of community-degenerate-conformance.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "cb370284f6763feac318e79bb3fc71568e47376862ff809f934e474094686461",
        "post-tick hash moved — the degenerate world after one tick"
    );
}

/// World 4 — summarizes the cost-modifier pins (the product, the exact
/// 1.0, the honest-null inactive). Measured, never derived; touches none
/// of the sixteen prior pins.
#[test]
fn community_world_4_hashes_are_pinned() {
    let report = run_once_with_prelude(COMMUNITY_W4, &practice_prelude(), COMMUNITY_PACK)
        .expect("world 4 tick");
    assert_eq!(
        hex(&report.before),
        "3a021222a3c3d9a5606305feed76e5d20dd0e1a14dd712309ca0f0578e0107b7",
        "pre-tick hash moved — the substrate's load of community-cost-modifier-conformance.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "e11bec9553aa50479ca03c408e8a58028bae2cfaad4033e3131960b862eb4f9d",
        "post-tick hash moved — the cost-modifier world after one tick"
    );
}

/// World 5 — the arc's tick-3 `after` (the three-tick decay chain the arc
/// mirror oracles). Measured, never derived; touches none of the sixteen
/// prior pins.
#[test]
fn community_world_5_arc_hashes_are_pinned() {
    let mut session = babylon_tick::TickSession::new_with_prelude(
        COMMUNITY_W5,
        &practice_prelude(),
        COMMUNITY_PACK,
        HypergraphStore::new(),
        babylon_kernel::SessionId::new("community-decay-arc").expect("literal"),
    )
    .expect("world 5 session");
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let mut last = None;
    for _ in 0..3 {
        last = Some(session.advance(&mut sink).expect("tick"));
    }
    let report = last.expect("three ticks ran");
    assert_eq!(
        hex(&report.before),
        "f0d645424b4c87e50df595577d4a16ac9fa34f9b223a85a65135af9f1e467ae3",
        "tick-3 pre-hash moved — the arc world's state entering tick 3"
    );
    assert_eq!(
        hex(&report.after),
        "df9462f954d9ccbbdc0d28c8631dcb95bab2f511f3255e36399b832968509a28",
        "tick-3 post-hash moved — the decay arc's third link"
    );
}

/// World 5b — summarizes the seam pins (SOLIDARITY strengths byte-
/// identical under the co-load; the community half actually ran).
/// Measured, never derived; touches none of the sixteen prior pins.
#[test]
fn community_world_5b_seam_hashes_are_pinned() {
    let rules = format!("{COMMUNITY_SOLIDARITY}\n{COMMUNITY_PACK}");
    let report =
        run_once_with_prelude(COMMUNITY_W5B, &practice_prelude(), &rules).expect("world 5b tick");
    assert_eq!(
        hex(&report.before),
        "43c36eece2e8250410e7730467b2cb48b5e67d8c279391bf29ffd6c2d77acf3b",
        "pre-tick hash moved — the seam world's load"
    );
    assert_eq!(
        hex(&report.after),
        "fa6297254a13c5c9ff9786e7599ce84e2e95ae09ed109f6d8ff7c98f9f147c52",
        "post-tick hash moved — solidarity + community co-loaded, one tick"
    );
}

/// World 5c — summarizes the carrier-collision pins (per-rule-id fired
/// arithmetic, the decay-applied-once read). Measured, never derived;
/// touches none of the sixteen prior pins.
#[test]
fn community_world_5c_collision_hashes_are_pinned() {
    let rules = format!("{COMMUNITY_CONTROL_RATIO}\n{COMMUNITY_PACK}");
    let report =
        run_once_with_prelude(COMMUNITY_W5C, &practice_prelude(), &rules).expect("world 5c tick");
    assert_eq!(
        hex(&report.before),
        "1ad177425e34ed4f972d8f530cec9cfe3122ab6bd3c14902ee282cfd71106d4f",
        "pre-tick hash moved — the collision world's load"
    );
    assert_eq!(
        hex(&report.after),
        "d018323568b2d910d643fd1fb79a30315b19d8629401213b12c8a12e602c4b14",
        "post-tick hash moved — control-ratio + community over ONE carrier"
    );
}

/// World 6 — summarizes the all-inactive lanes-skipped pins (and the
/// recorded decay divergence, D205). Measured, never derived; touches
/// none of the sixteen prior pins.
#[test]
fn community_world_6_hashes_are_pinned() {
    let report = run_once_with_prelude(COMMUNITY_W6, &practice_prelude(), COMMUNITY_PACK)
        .expect("world 6 tick");
    assert_eq!(
        hex(&report.before),
        "0d10526ca3a6a14eb1e2bac27c63c0e4716d4f9189fc990c07bbd9226d70d946",
        "pre-tick hash moved — the substrate's load of community-empty-conformance.bscn"
    );
    assert_eq!(
        hex(&report.after),
        "d808eb25330cc3f680d45ae9ee458c515c2abb207b08eb18b88427d821fe3c8e",
        "post-tick hash moved — the all-inactive world after one tick"
    );
}

/// The tie world (the §8a row-1 copies-agree home) — summarizes the
/// DG-2 readout pins (the class-surface and community-surface tie-breaks
/// agreeing on LIBERAL). Measured, never derived; touches none of the
/// sixteen prior pins. Ninth content world — the plan's "8 pins" count
/// predates this world's addition at the DG-2 landing.
#[test]
fn community_tie_world_hashes_are_pinned() {
    let rules = format!("{CONSCIOUSNESS_TERNARY_RULES}\n{COMMUNITY_PACK}");
    let report = run_once_with_prelude(COMMUNITY_TIE, &practice_worldview_prelude(), &rules)
        .expect("tie world tick");
    assert_eq!(
        hex(&report.before),
        "347892760ad11bb974b95ce59b8559e7b3862befd0b14444cacf457964119e83",
        "pre-tick hash moved — the tie world's load"
    );
    assert_eq!(
        hex(&report.after),
        "ed8eb9c6933b6d0a5383c0d8261d00523995a86db685af83c4ec8fe527b7c247",
        "post-tick hash moved — both packs over the tie world"
    );
}
