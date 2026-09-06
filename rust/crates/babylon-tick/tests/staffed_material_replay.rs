//! Graph-owned people and physical hours cross one real replay publication boundary.
//! This is a Designed circuit fixture, not admission of current Michigan content.

#[allow(
    dead_code,
    reason = "reuse the checked foundation loader, as replay_session does"
)]
#[path = "../../babylon-persistence/src/michigan_dynamic_hex_foundation.rs"]
mod michigan_dynamic_hex_foundation;

use babylon_bsl::causal_contract::{EffectSignature, EvidenceClass, RuleRole};
use babylon_bsl::evaluator::Value;
use babylon_bsl::identity_codec::StableBslValueV1;
use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_element::StableElementKeyV1;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::GraphSubstrate;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::{sha256_of, ContentDigest};
use babylon_material_circuit::{
    BacklogRowV1, CapacityRowV1, CorridorCapacityV2, CorridorIdV2, GoodIdV1,
    InputOutputCoefficientV1, InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1,
    LogisticsNodeIdV2, MaterialCircuitStateV2, OrderAccessModeV1, OrderIdV1, OrderRowV2,
    ProcessIdV1, ProcessOutputV1, RouteIdV2, RouteLegV2, SiteIdV1, SiteLogisticsNodeV2,
    StaffingPolicyV1, StaffingPoolBindingV1, StaffingPoolIdV1, SupplierRouteV2, UnitIdV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::h3_runtime::MichiganDynamicHexValueBitsV1;
use babylon_tick::material_replay::{
    IdentifiedMaterialTickV3, MaterialBaseErrorV1, MaterialCommitErrorV3, MaterialLaborV1,
    MaterialReplayErrorV3, MaterialReplaySessionV3, PreparedMaterialTickV3,
};
use babylon_tick::material_staffing::{
    StaffingCompositionV1, StaffingNodeBindingV1, EMPLOYED_POPULATION, PREVIOUS_UNRETAINED_HOURS,
    RESERVE_POPULATION, STAFFING_COMPOSITION_ID_V1, STAFFING_FIELDS_V1,
};
use babylon_tick::material_state::{
    DynamicHexStateRowV1, MaterialStateRowsInputV1, MaterialStateRowsV1, MaterialStateV1,
    OrganizationStateRowV1, TerritoryStateRowV1, WorldRegisterRowV1,
};
use babylon_tick::material_world::{
    decode_material_receipts_v3, MaterialTickReceiptsV3, MaterialWorldRegisterV2,
};
use babylon_tick::replay_session::{ReplayCommitDispositionV1, ReplayTickError, ReplayTickSession};

type Session = MaterialReplaySessionV3<HypergraphStore>;
type Candidate = PreparedMaterialTickV3<HypergraphStore>;

const SCENARIO: &str = r"
(scenario staffing/replay
  (deffield social-class/employed-population int extensive)
  (deffield social-class/reserve-population int extensive)
  (deffield social-class/previous-unretained-labor-hours int extensive)
  (deffield social-class/seen-employed int extensive)
  (deffield social-class/probability probability intensive)
  (node workers NodeType/SOCIAL_CLASS
    (social-class/employed-population 1)
    (social-class/reserve-population 0)
    (social-class/previous-unretained-labor-hours 40)
    (social-class/seen-employed 1)
    (social-class/probability 0.9p)))
";

// The common after-metabolism boundary orders this explicit later rule after
// the native g4-workforce-staffing composition by the governed rule-ID bytes.
const WITNESS: &str = r#"
(rule zz-staffing/witness
  :role mechanic :evidence designed
  :material-basis "fixture observes the graph workforce after native staffing"
  :fuel 64
  (anchor :after metabolism)
  (bindings (binding employed :field social-class/employed-population))
  (when #t)
  (effects
    (update-node self social-class/seen-employed (set employed))
    (emit EventType/STAFFING_WITNESS (subject self) (employed employed))))
"#;

const FAILURE: &str = r#"
(rule zzz-staffing/failure
  :role mechanic :evidence designed
  :material-basis "fixture invalid write after staffing must abort the whole candidate"
  :fuel 64
  (anchor :after metabolism)
  (bindings
    (binding requested :field social-class/previous-unretained-labor-hours)
    (binding probability :field social-class/probability))
  (when (> requested 0))
  (effects
    (emit EventType/STAFFING_ABORT)
    (update-node self social-class/probability (add 0.4i))))
"#;

fn subject() -> StableElementKeyV1 {
    StableElementKeyV1::Node {
        scenario: "staffing/replay".to_owned(),
        local_name: "workers".to_owned(),
    }
}

fn site(value: u8) -> SiteIdV1 {
    SiteIdV1::from_bytes([value; 32])
}

fn good(value: u8) -> GoodIdV1 {
    GoodIdV1::from_bytes([value; 32])
}

fn unit(value: u8) -> UnitIdV1 {
    UnitIdV1::from_bytes([value; 32])
}

fn process() -> ProcessIdV1 {
    ProcessIdV1::from_bytes([1; 32])
}

fn labor(week: u64, available: u64) -> LaborCapacityRowV1 {
    LaborCapacityRowV1 {
        site_id: site(1),
        unit_id: unit(1),
        week,
        available,
    }
}

fn opening() -> MaterialCircuitStateV2 {
    let mut state = MaterialCircuitStateV2 {
        week: 1,
        site_logistics_nodes: [1, 2]
            .map(|id| SiteLogisticsNodeV2 {
                site_id: site(id),
                node_id: LogisticsNodeIdV2::from_bytes([id; 32]),
            })
            .to_vec(),
        process_outputs: vec![ProcessOutputV1 {
            process_id: process(),
            site_id: site(1),
            good_id: good(2),
            unit_id: unit(2),
            quantity_per_batch: 5,
        }],
        input_coefficients: vec![InputOutputCoefficientV1 {
            process_id: process(),
            good_id: good(1),
            unit_id: unit(2),
            quantity_per_batch: 2,
        }],
        labor_coefficients: vec![LaborCoefficientV1 {
            process_id: process(),
            unit_id: unit(1),
            quantity_per_batch: 40,
        }],
        supplier_routes: Vec::new(),
        route_legs: Vec::new(),
        inventory: vec![InventoryRowV1 {
            site_id: site(2),
            good_id: good(1),
            unit_id: unit(2),
            quantity: 4,
        }],
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        corridor_capacities: Vec::new(),
        capacities: (1..=8)
            .map(|week| CapacityRowV1 {
                process_id: process(),
                site_id: site(1),
                week,
                available_batches: 1,
            })
            .collect(),
        labor: vec![labor(1, 40)],
        production_commitments: Vec::new(),
    };
    install_freight(&mut state);
    state
}

fn install_freight(state: &mut MaterialCircuitStateV2) {
    let route = RouteIdV2::from_bytes([1; 32]);
    let corridor = CorridorIdV2::from_bytes([1; 32]);
    let order = OrderIdV1::from_bytes([1; 32]);
    state.supplier_routes.push(SupplierRouteV2 {
        buyer_site_id: site(1),
        supplier_site_id: site(2),
        good_id: good(1),
        unit_id: unit(2),
        route_id: route,
    });
    state.route_legs.push(RouteLegV2 {
        route_id: route,
        leg_index: 0,
        corridor_id: corridor,
        from_node_id: LogisticsNodeIdV2::from_bytes([2; 32]),
        to_node_id: LogisticsNodeIdV2::from_bytes([1; 32]),
        travel_weeks: 2,
        loss_ppm: 0,
    });
    state.orders.push(OrderRowV2 {
        order_id: order,
        access_mode: OrderAccessModeV1::CommoditySale,
        buyer_site_id: site(1),
        supplier_site_id: site(2),
        good_id: good(1),
        unit_id: unit(2),
        ordered: 4,
        shipped: 0,
        lost: 0,
        delivered: 0,
        realized: 0,
    });
    state.backlog.push(BacklogRowV1 {
        order_id: order,
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacityV2 {
        corridor_id: corridor,
        unit_id: unit(2),
        week: 1,
        available: 4,
    });
}

fn staffed_labor() -> MaterialLaborV1 {
    let pool = StaffingPoolBindingV1::try_new(
        StaffingPoolIdV1::from_bytes([1; 32]),
        site(1),
        unit(1),
        1,
        StaffingPolicyV1::one_week(40).unwrap(),
        vec![process()],
    )
    .unwrap();
    let composition =
        StaffingCompositionV1::try_new(vec![
            StaffingNodeBindingV1::try_new(subject(), pool).unwrap()
        ])
        .unwrap();
    MaterialLaborV1::Staffed(composition)
}

fn try_session(rules: &str, labor: MaterialLaborV1) -> Result<Session, MaterialReplayErrorV3> {
    let foundation = michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation_v1().unwrap();
    let (_, parsed) = split_content(rules).unwrap();
    let forms = parsed.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    let graph = ReplayTickSession::new(
        SCENARIO,
        None,
        rules,
        HypergraphStore::new(),
        ReplaySessionIdV1::try_from("staffing/replay-session").unwrap(),
        ReplaySeed::new(40),
        ContentDigest {
            defines_hash: [40; 32],
            rules_hash: rules_hash_of(&forms).unwrap(),
        },
        RefDigestV1::from_bytes(foundation.reference_bundle_digest()),
        MaterialStateV1::try_new(foundation).unwrap(),
    )
    .map_err(MaterialReplayErrorV3::Graph)?;
    MaterialReplaySessionV3::new(
        graph,
        MaterialWorldRegisterV2::try_new(0, opening()).unwrap(),
        sha256_of(b"staffed-replay-fixture-foundation"),
        7,
        labor,
    )
}

fn session(rules: &str) -> Session {
    try_session(rules, staffed_labor()).unwrap()
}

fn prepare(session: &Session) -> Candidate {
    let actions = OrderedPracticeActionBatchV1::empty(
        session.graph_session().session_identity().clone(),
        session.completed_tick() + 1,
    )
    .unwrap();
    session.prepare_advance(&actions).unwrap()
}

fn commit(
    session: &mut Session,
    sink: &mut CollectingSink,
    candidate: Candidate,
) -> IdentifiedMaterialTickV3 {
    session
        .commit_prepared_and_publish(sink, candidate, |_| {
            Ok::<_, &'static str>(ReplayCommitDispositionV1::Committed)
        })
        .unwrap()
        .0
}

fn advance(session: &mut Session, sink: &mut CollectingSink) -> MaterialTickReceiptsV3 {
    let candidate = prepare(session);
    let receipts = decode_material_receipts_v3(candidate.material().receipt_bytes()).unwrap();
    commit(session, sink, candidate);
    receipts
}

fn assert_stock(session: &Session, field: &str, expected: f64) {
    let graph = session.graph_session().graph();
    let nodes = graph.nodes("SOCIAL_CLASS");
    assert_eq!(nodes.len(), 1);
    assert_eq!(
        graph.node_attribute(nodes[0], field).unwrap().to_bits(),
        expected.to_bits(),
        "{field}"
    );
}

fn assert_people(session: &Session, employed: f64, reserve: f64, previous: f64) {
    assert_eq!((employed + reserve).to_bits(), 1.0_f64.to_bits());
    assert_stock(session, EMPLOYED_POPULATION, employed);
    assert_stock(session, RESERVE_POPULATION, reserve);
    assert_stock(session, PREVIOUS_UNRETAINED_HOURS, previous);
}

fn staffing_field(candidate: &Candidate, name: &str) -> i64 {
    let events = candidate.graph_report().successful_event_batch().events();
    let event = events
        .iter()
        .find(|event| event.emitting_rule() == STAFFING_COMPOSITION_ID_V1)
        .unwrap();
    let (_, value) = event
        .fields()
        .iter()
        .find(|(field, _)| field == name)
        .unwrap();
    let StableBslValueV1::Int(value) = value else {
        panic!("staffing receipt field {name} must be exact int");
    };
    *value
}

#[derive(Debug, PartialEq)]
struct LiveState {
    graph: Vec<u8>,
    graph_material: MaterialStateV1,
    registers: Vec<u8>,
    physical: Vec<u8>,
    world_hash: [u8; 32],
    tick: u64,
    events: Vec<(String, Vec<(String, Value)>)>,
}

fn live(session: &Session, sink: &CollectingSink) -> LiveState {
    let graph = session.graph_session();
    // These fixture rules change only social-class attributes. Compare the
    // complete live H3 owner to a separately constructed checked foundation;
    // it deliberately offers no unchecked Clone or arbitrary-state constructor.
    let foundation = michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation_v1().unwrap();
    let graph_material = MaterialStateV1::try_new(foundation).unwrap();
    assert_eq!(graph.material_state(), &graph_material);
    LiveState {
        graph: graph.graph().encode_state().unwrap().as_bytes().to_vec(),
        graph_material,
        registers: graph.world_registers().unwrap().canonical_bytes().to_vec(),
        physical: session.material().canonical_bytes().to_vec(),
        world_hash: session.current_world_hash().unwrap(),
        tick: session.completed_tick(),
        events: sink.events.clone(),
    }
}

fn owned_checkpoint_rows(rows: &MaterialStateRowsV1) -> MaterialStateRowsV1 {
    let owned = MaterialStateRowsV1::try_from_rows(MaterialStateRowsInputV1 {
        world_registers: rows.world_registers().rows().iter().map(|row| {
            WorldRegisterRowV1::try_new(row.qname().to_owned(), row.value().clone()).unwrap()
        }).collect(),
        territories: rows.territories().rows().iter().map(|row| {
            TerritoryStateRowV1::try_new(row.territory_id().clone(), row.ordered_fields().to_vec()).unwrap()
        }).collect(),
        dynamic_hexes: rows.dynamic_hexes().rows().iter().map(|row| {
            let [c, v, s, k, biocapacity_stock, energy_stock, raw_material_stock,
                internet_access_pct, surveillance_coupling] = row.value_bits();
            DynamicHexStateRowV1::try_new(row.cell_id(), MichiganDynamicHexValueBitsV1 {
                c, v, s, k, biocapacity_stock, energy_stock, raw_material_stock,
                internet_access_pct, surveillance_coupling,
            }).unwrap()
        }).collect(),
        organizations: rows.organizations().rows().iter().map(|row| {
            OrganizationStateRowV1::try_new(
                row.organization_id().clone(), row.organization_kind().clone(),
                row.ordered_territory_ids().to_vec(), row.ordered_fields().to_vec(),
            ).unwrap()
        }).collect(),
    }).unwrap();
    assert_eq!(owned.canonical_bytes(), rows.canonical_bytes());
    owned
}

#[test]
fn one_empty_week_holds_then_releases_and_real_arrival_rehires_for_next_week() {
    let mut session = session("");
    let mut sink = CollectingSink::default();
    let first = prepare(&session);
    assert_eq!(staffing_field(&first, "current-unretained-hours"), 0);
    assert_eq!(staffing_field(&first, "retained-hours"), 40);
    assert_eq!(staffing_field(&first, "separations"), 0);
    let receipts = decode_material_receipts_v3(first.material().receipt_bytes()).unwrap();
    assert_eq!(
        (
            receipts.dispatches[0].quantity,
            receipts.dispatches[0].final_arrival_week
        ),
        (4, 3)
    );
    commit(&mut session, &mut sink, first);
    assert_people(&session, 1.0, 0.0, 0.0);
    assert_eq!(session.material().state().labor, vec![labor(2, 40)]);

    let second = prepare(&session);
    assert_eq!(staffing_field(&second, "separations"), 1);
    assert_eq!(staffing_field(&second, "retained-hours"), 0);
    commit(&mut session, &mut sink, second);
    assert_people(&session, 0.0, 1.0, 0.0);
    assert_eq!(session.material().state().labor, vec![labor(3, 0)]);
    assert!(session.material().state().production_commitments.is_empty());

    let arrival = prepare(&session);
    assert_eq!(staffing_field(&arrival, "current-unretained-hours"), 40);
    assert_eq!(staffing_field(&arrival, "hires"), 1);
    let receipts = decode_material_receipts_v3(arrival.material().receipt_bytes()).unwrap();
    assert_eq!(receipts.arrivals.len(), 1);
    assert_eq!(receipts.arrivals[0].quantity, 4);
    assert!(receipts.production.is_empty());
    assert_eq!(
        arrival.material().register().state().production_commitments[0].planned_batches,
        1
    );
    commit(&mut session, &mut sink, arrival);
    assert_people(&session, 1.0, 0.0, 40.0);
    assert_eq!(session.material().state().labor, vec![labor(4, 40)]);

    let fourth = advance(&mut session, &mut sink);
    let fifth = advance(&mut session, &mut sink);
    assert_eq!(fourth.production[0].produced_batches, 1);
    assert_eq!(fifth.production[0].produced_batches, 1);
    assert_eq!(
        session.material().state().inventory,
        vec![
            InventoryRowV1 {
                site_id: site(1),
                good_id: good(1),
                unit_id: unit(2),
                quantity: 0
            },
            InventoryRowV1 {
                site_id: site(1),
                good_id: good(2),
                unit_id: unit(2),
                quantity: 10
            },
            InventoryRowV1 {
                site_id: site(2),
                good_id: good(1),
                unit_id: unit(2),
                quantity: 0
            },
        ]
    );
    let order = &session.material().state().orders[0];
    assert_eq!(
        (order.shipped, order.delivered, order.realized, order.lost),
        (4, 4, 4, 0)
    );
    assert!(session.material().state().freight.is_empty());
    assert_eq!(
        session.material().state().backlog,
        vec![BacklogRowV1 {
            order_id: OrderIdV1::from_bytes([1; 32]),
            quantity: 0,
        }]
    );
    assert_people(&session, 1.0, 0.0, 0.0);
    advance(&mut session, &mut sink);
    assert_people(&session, 0.0, 1.0, 0.0);
    assert_eq!(session.material().state().labor, vec![labor(7, 0)]);
}

#[test]
fn prepared_and_failed_commit_publish_nothing_and_retry_has_identical_joint_identity() {
    let mut session = session("");
    let mut sink = CollectingSink::default();
    advance(&mut session, &mut sink);
    let before = live(&session, &sink);
    let candidate = prepare(&session);
    assert_eq!(staffing_field(&candidate, "separations"), 1);
    assert_eq!(live(&session, &sink), before);
    let expected = *candidate.identity();
    let expected_graph = candidate.graph_report().result_stable_graph().clone();
    let expected_physical = candidate.material().register().clone();
    let error = session.commit_prepared_and_publish(&mut sink, candidate, |identity| {
        assert_eq!(identity, &expected);
        Err::<ReplayCommitDispositionV1, _>("durable commit refused")
    });
    assert!(matches!(
        error,
        Err(MaterialCommitErrorV3::Commit("durable commit refused"))
    ));
    assert_eq!(live(&session, &sink), before);

    let retry = prepare(&session);
    assert_eq!(retry.identity(), &expected);
    assert_eq!(retry.graph_report().result_stable_graph(), &expected_graph);
    assert_eq!(retry.material().register(), &expected_physical);
    let identity = commit(&mut session, &mut sink, retry);
    assert_eq!(identity, expected);
    assert_eq!(session.material(), &expected_physical);
    assert_eq!(
        session.graph_session().stable_graph_state().unwrap(),
        expected_graph
    );
    assert_eq!(session.completed_tick(), 2);
    assert_eq!(session.graph_session().completed_tick(), 2);
    assert_eq!(sink.events.len(), before.events.len() + 1);
    assert_eq!(&sink.events[..before.events.len()], before.events);
    assert_people(&session, 0.0, 1.0, 0.0);
}

#[test]
fn successful_acknowledgement_publishes_stable_staffing_and_identity_free_audit_evidence() {
    let mut session = session("");
    let mut sink = CollectingSink::default();
    advance(&mut session, &mut sink);
    let candidate = prepare(&session);
    let report = candidate.graph_report();
    let events = report.successful_event_batch().events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type(), "WORKFORCE_STAFFING");
    assert_eq!(events[0].emitting_rule(), STAFFING_COMPOSITION_ID_V1);
    assert_eq!(events[0].choice_receipt(), None);
    assert!(events[0]
        .fields()
        .contains(&("subject".to_owned(), StableBslValueV1::Node(subject()))));
    for (name, expected) in [
        ("week", 2),
        ("opening-employed", 1),
        ("opening-reserve", 0),
        ("closing-employed", 0),
        ("closing-reserve", 1),
        ("separations", 1),
        ("hires", 0),
        ("next-opening-hours", 0),
    ] {
        assert_eq!(staffing_field(&candidate, name), expected);
    }
    let audit = &report.report().audit_receipts;
    assert_eq!(audit.len(), 4);
    assert!(audit
        .iter()
        .all(|row| row.rule_id == STAFFING_COMPOSITION_ID_V1
            && row.role == RuleRole::Mechanic
            && row.evidence == EvidenceClass::Designed));
    assert_eq!(
        audit.iter().map(|row| row.ordinal).collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_eq!(
        audit[0].effect,
        EffectSignature::Event("EventType/WORKFORCE_STAFFING".to_owned())
    );
    assert_eq!(
        audit[1..].iter().map(|row| &row.effect).collect::<Vec<_>>(),
        [
            &EffectSignature::NodeField(EMPLOYED_POPULATION.to_owned()),
            &EffectSignature::NodeField(RESERVE_POPULATION.to_owned()),
            &EffectSignature::NodeField(PREVIOUS_UNRETAINED_HOURS.to_owned()),
        ]
    );
    assert_eq!(
        candidate.identity().graph_tick_content_hash(),
        report.tick_content_hash()
    );
    assert_eq!(
        candidate.identity().receipt_digest(),
        sha256_of(candidate.material().receipt_bytes())
    );
    let expected_event = report.report().committed_events[0].clone();
    commit(&mut session, &mut sink, candidate);
    assert_eq!(
        sink.events.last().unwrap(),
        &(
            expected_event.event_type().to_owned(),
            expected_event.payload().to_vec(),
        )
    );
    assert_people(&session, 0.0, 1.0, 0.0);
    assert_eq!(session.material().state().labor, vec![labor(3, 0)]);
}

#[test]
fn later_anchored_rule_reads_this_candidates_staffing_before_graph_finalization() {
    let mut session = session(WITNESS);
    let mut sink = CollectingSink::default();
    advance(&mut session, &mut sink);
    assert_stock(&session, "social-class/seen-employed", 1.0);
    let candidate = prepare(&session);
    let events = candidate.graph_report().successful_event_batch().events();
    assert_eq!(
        events
            .iter()
            .map(|event| event.emitting_rule())
            .collect::<Vec<_>>(),
        [STAFFING_COMPOSITION_ID_V1, "zz-staffing/witness"]
    );
    assert!(events[1].fields().contains(&(
        "employed".to_owned(),
        StableBslValueV1::RealBits(0.0_f64.to_bits()),
    )));
    // Preparation has exposed the changed stock only to the detached later rule.
    assert_stock(&session, "social-class/seen-employed", 1.0);
    assert_people(&session, 1.0, 0.0, 0.0);
    commit(&mut session, &mut sink, candidate);
    assert_stock(&session, "social-class/seen-employed", 0.0);
    assert_people(&session, 0.0, 1.0, 0.0);
}

#[test]
fn failing_later_rule_aborts_staffing_physical_close_events_and_both_clocks() {
    let mut session = session(&format!("{WITNESS}\n{FAILURE}"));
    let mut sink = CollectingSink::default();
    advance(&mut session, &mut sink);
    advance(&mut session, &mut sink);
    assert_people(&session, 0.0, 1.0, 0.0);
    assert_eq!(session.material().state().freight[0].quantity, 4);
    let before = live(&session, &sink);
    let actions =
        OrderedPracticeActionBatchV1::empty(session.graph_session().session_identity().clone(), 3)
            .unwrap();
    for _ in 0..2 {
        let result = session.prepare_advance(&actions);
        let Err(MaterialReplayErrorV3::Graph(ReplayTickError::Execution { message })) = result
        else {
            panic!("the later domain-invalid write must abort replay execution");
        };
        assert!(message.contains("zzz-staffing/failure"), "{message}");
        assert_eq!(live(&session, &sink), before);
    }
    assert_people(&session, 0.0, 1.0, 0.0);
    assert_stock(&session, "social-class/seen-employed", 0.0);
    assert_eq!(session.material().state().labor, vec![labor(3, 0)]);
}

#[test]
fn same_version_checkpoint_restores_retention_and_replays_arrival_with_identical_evidence() {
    let mut uninterrupted = session(WITNESS);
    let mut original_sink = CollectingSink::default();
    advance(&mut uninterrupted, &mut original_sink);
    let second = prepare(&uninterrupted);
    let graph = second.graph_report().result_stable_graph().clone();
    let graph_material = owned_checkpoint_rows(second.graph_report().material_state_rows());
    let registers = second
        .graph_report()
        .result_registers()
        .canonical_bytes()
        .to_vec();
    let physical = second.material().register().canonical_bytes().to_vec();
    commit(&mut uninterrupted, &mut original_sink, second);

    let mut restored = session(WITNESS);
    restored
        .restore_full_checkpoint(&graph, &graph_material, &registers, &physical)
        .unwrap();
    assert_people(&restored, 0.0, 1.0, 0.0);
    assert_eq!(
        restored.current_world_hash().unwrap(),
        uninterrupted.current_world_hash().unwrap()
    );
    let mut restored_sink = CollectingSink::default();
    let original_event_count = original_sink.events.len();
    for _ in 3..=5 {
        let next = prepare(&uninterrupted);
        let replay = prepare(&restored);
        assert_eq!(replay.identity(), next.identity());
        assert_eq!(
            replay.graph_report().successful_event_batch(),
            next.graph_report().successful_event_batch()
        );
        assert_eq!(
            replay.graph_report().report().audit_receipts,
            next.graph_report().report().audit_receipts
        );
        assert_eq!(
            replay.material().receipt_bytes(),
            next.material().receipt_bytes()
        );
        commit(&mut uninterrupted, &mut original_sink, next);
        commit(&mut restored, &mut restored_sink, replay);
    }
    assert_eq!(restored.material(), uninterrupted.material());
    assert_eq!(
        restored.graph_session().stable_graph_state().unwrap(),
        uninterrupted.graph_session().stable_graph_state().unwrap()
    );
    assert_eq!(
        restored_sink.events,
        original_sink.events[original_event_count..]
    );
}

#[test]
fn staffed_admission_refuses_every_native_field_in_early_late_and_untaken_writes() {
    for field in STAFFING_FIELDS_V1 {
        for (anchor, guard) in [
            ("before vitality", "#t"),
            ("after metabolism", "#t"),
            ("after metabolism", "#f"),
        ] {
            let source = format!(
                r#"
(rule zz-staffing/foreign-writer
  :role mechanic :evidence designed
  :material-basis "fixture exercises declared ownership even when the branch is untaken"
  :fuel 64
  (anchor :{anchor})
  (bindings (binding employed :field social-class/employed-population))
  (when #t)
  (effects (guard {guard} (update-node self {field} (set 9)))))
"#
            );
            let error = try_session(&source, staffed_labor())
                .err()
                .unwrap_or_else(|| {
                    panic!("Staffed accepted {anchor} write to {field}, guard {guard}")
                });
            let MaterialReplayErrorV3::Graph(ReplayTickError::MaterialBase(
                MaterialBaseErrorV1::StaffingFieldOwner {
                    rule_id,
                    field: actual,
                },
            )) = &error
            else {
                panic!("Staffed must refuse {anchor} write to {field}, guard {guard}: {error:?}");
            };
            assert_eq!(rule_id, "zz-staffing/foreign-writer");
            assert_eq!(actual, field);
            // Ownership is specific to Staffed admission, not a new global BSL ban.
            let mut scheduled = try_session(&source, MaterialLaborV1::Scheduled).unwrap();
            let mut sink = CollectingSink::default();
            advance(&mut scheduled, &mut sink);
            let original = if field == EMPLOYED_POPULATION {
                1.0
            } else if field == RESERVE_POPULATION {
                0.0
            } else {
                40.0
            };
            assert_stock(
                &scheduled,
                field,
                if guard == "#t" { 9.0 } else { original },
            );
            assert!(sink.events.is_empty());
        }
    }
}

#[test]
fn removing_the_staffed_subject_is_refused_by_the_existing_shape_verb_loader() {
    let source = r#"
(rule zz-staffing/remove-owner
  :role mechanic :evidence designed
  :material-basis "fixture preserves the existing graph-shape loading boundary"
  :fuel 64
  (anchor :after metabolism)
  (bindings (binding employed :field social-class/employed-population))
  (when #t)
  (effects (remove-node self)))
"#;
    for labor in [staffed_labor(), MaterialLaborV1::Scheduled] {
        let result = try_session(source, labor);
        let Err(MaterialReplayErrorV3::Graph(ReplayTickError::Preparation { message })) = result
        else {
            panic!("the existing shape-verb gate must refuse before a session is created");
        };
        assert!(message.contains("remove-node"), "{message}");
        assert!(message.contains("graph-shape verbs"), "{message}");
    }
}
