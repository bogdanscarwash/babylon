//! Loading proof for the Program 28 B2 demo world (Phase B, Task 7):
//! twelve real-FIPS counties (`lifecycle/dpd-circuit`) plus six social
//! classes (`vitality/subsistence-and-death`), loaded and ticked together
//! through the persistent `TickSession` seam Phase A built.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::SessionId;
use babylon_tick::TickSession;

const SCENARIO: &str = include_str!("../content/scenarios/us-counties-lifecycle-demo.bscn");
const VITALITY: &str = include_str!("../content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../content/rules/lifecycle.bsl");

#[test]
fn the_demo_scenario_loads_and_ticks_both_packs() {
    let rule_src = format!("{VITALITY}\n{LIFECYCLE}");
    let session_id = SessionId::new("us-counties-demo-test").expect("literal is non-empty");
    let mut session =
        TickSession::new(SCENARIO, &rule_src, HypergraphStore::new(), session_id).expect("load");
    let mut sink = CollectingSink::default();
    let report = session.advance(&mut sink).expect("tick 1");
    assert_ne!(report.before, report.after);
    assert_eq!(report.per_rule_fired.len(), 2);
    // Governed phase order — Vitality @1 before Lifecycle @7, regardless
    // of the rule_src concatenation order above.
    assert_eq!(report.per_rule_fired[0].0, "vitality/subsistence-and-death");
    assert_eq!(report.per_rule_fired[1].0, "lifecycle/dpd-circuit");
    // lifecycle fires unconditionally on all twelve territories; vitality
    // fires on 5 of 6 social classes (`dissolved` fails the guard) — the
    // same per-pack counts the individually-pinned conformance tests
    // assert, scaled from 4/6 territories to 12 territories (still all of
    // them; the scenario mints no territory that fails any guard) and
    // unchanged for the six untouched vitality fixture nodes.
    assert_eq!(
        report.per_rule_fired[1].1, 12,
        "all twelve territories fire"
    );
    assert_eq!(
        report.per_rule_fired[0].1, 5,
        "five of six social classes fire"
    );
    // The recovering-county archetype (county-01013/01015/01017) fires
    // LEGITIMATION_RECOVERY on tick 1 under these defconsts (matching
    // lifecycle-conformance.bscn's own documented behavior) — proves the
    // twelve territory nodes really run lifecycle, not just mint
    // successfully.
    // Event type strings are bare enum members at the sink boundary — same
    // as node-type strings (`"TERRITORY"`, never `"NodeType/TERRITORY"`) —
    // `structural_verbs::EffectExecutor::enum_member` returns the bare
    // `Atom::EnumRef { member, .. }`, not the `EventType/`-qualified
    // source spelling. The plan's own literal code block wrote
    // `"EventType/LEGITIMATION_RECOVERY"` here, which fails at runtime
    // (verified: 0 matches) — fixed to the bare member, matching every
    // other event-name assertion in this crate
    // (`lifecycle_conformance.rs`/`lifecycle_crisis_conformance.rs` both
    // compare against bare `"LEGITIMATION_RECOVERY"`/`"LEGITIMATION_CRISIS"`).
    assert!(sink
        .events
        .iter()
        .any(|(name, _)| name == "LEGITIMATION_RECOVERY"));
}
