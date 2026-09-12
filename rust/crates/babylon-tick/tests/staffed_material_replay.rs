//! Graph-owned people and physical hours cross one real replay publication boundary.
//! This is a Designed circuit fixture, not admission of current Michigan content.

#[allow(
    dead_code,
    reason = "reuse the checked foundation loader, as replay_session does"
)]
#[path = "../../babylon-persistence/src/michigan_dynamic_hex_foundation.rs"]
mod michigan_dynamic_hex_foundation;

use babylon_bsl::canonical_ast::rules_hash_of;
use babylon_bsl::causal_contract::{EffectSignature, EvidenceClass, RuleRole};
use babylon_bsl::evaluator::Value;
use babylon_bsl::identity_codec::StableBslValue;
use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_element::StableElementKey;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::GraphSubstrate;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionId};
use babylon_kernel::tick_content_hash::RefDigest;
use babylon_kernel::{content_digest::sha256_of, content_digest::ContentDigest};
use babylon_material_circuit::{
    BacklogRow, CapacityRow, CorridorCapacity, CorridorId, FreightMassCoefficient, GoodId,
    InputOutputCoefficient, InventoryRow, LaborCapacityRow, LaborCoefficient, LogisticsNodeId,
    MaterialCircuitState, OrderAccessMode, OrderId, OrderRow, ProcessId, ProcessOutput, RouteId,
    RouteStage, RouteStageCapacity, SiteId, SiteLogisticsNode, StaffingPolicy, StaffingPoolBinding,
    StaffingPoolId, StaffingWorkSource, SupplierRoute, SupplierTransport, UnitId,
};
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::h3_runtime::MichiganDynamicHexValueBits;
use babylon_tick::material_replay::{
    IdentifiedMaterialTick, MaterialBaseError, MaterialCommitError, MaterialReplayError,
    MaterialReplaySession, PreparedMaterialTick,
};
use babylon_tick::material_staffing::{
    StaffingComposition, StaffingNodeBinding, EMPLOYED_POPULATION, PREVIOUS_UNRETAINED_HOURS,
    RESERVE_POPULATION, STAFFING_COMPOSITION_ID, STAFFING_FIELDS,
};
use babylon_tick::material_state::{
    DynamicHexStateRow, MaterialState, MaterialStateRows, MaterialStateRowsInput,
    OrganizationStateRow, TerritoryStateRow, WorldRegisterRow,
};
use babylon_tick::material_world::{
    decode_material_receipts, MaterialTickReceipts, MaterialWorldRegister,
};
use babylon_tick::replay_session::{ReplayCommitDisposition, ReplayTickError, ReplayTickSession};

type Session = MaterialReplaySession<HypergraphStore>;
type Candidate = PreparedMaterialTick<HypergraphStore>;

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
    (social-class/previous-unretained-labor-hours 160)
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

fn subject() -> StableElementKey {
    StableElementKey::Node {
        scenario: "staffing/replay".to_owned(),
        local_name: "workers".to_owned(),
    }
}

fn site(value: u8) -> SiteId {
    SiteId::from_bytes([value; 32])
}

fn good(value: u8) -> GoodId {
    GoodId::from_bytes([value; 32])
}

fn unit(value: u8) -> UnitId {
    UnitId::from_bytes([value; 32])
}

fn process() -> ProcessId {
    ProcessId::from_bytes([1; 32])
}

fn labor(period: u64, available: u64) -> LaborCapacityRow {
    LaborCapacityRow {
        site_id: site(1),
        unit_id: unit(1),
        period,
        available,
    }
}

fn opening() -> MaterialCircuitState {
    let mut state = MaterialCircuitState {
        period: 1,
        site_logistics_nodes: [1, 2]
            .map(|id| SiteLogisticsNode {
                site_id: site(id),
                node_id: LogisticsNodeId::from_bytes([id; 32]),
            })
            .to_vec(),
        process_outputs: vec![ProcessOutput {
            process_id: process(),
            site_id: site(1),
            good_id: good(2),
            unit_id: unit(2),
            quantity_per_batch: 5,
        }],
        input_coefficients: vec![InputOutputCoefficient {
            process_id: process(),
            good_id: good(1),
            unit_id: unit(2),
            quantity_per_batch: 2,
        }],
        labor_coefficients: vec![LaborCoefficient {
            process_id: process(),
            unit_id: unit(1),
            quantity_per_batch: 40,
        }],
        supplier_routes: Vec::new(),
        route_stages: Vec::new(),
        route_stage_capacities: Vec::new(),
        freight_mass_coefficients: [1, 2]
            .map(|id| FreightMassCoefficient {
                good_id: good(id),
                unit_id: unit(2),
                grams_per_unit: 1000,
            })
            .to_vec(),
        inventory: vec![InventoryRow {
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
            .map(|period| CapacityRow {
                process_id: process(),
                site_id: site(1),
                period,
                available_batches: 1,
            })
            .collect(),
        labor: vec![labor(1, 160)],
        production_commitments: Vec::new(),
        merchants: Vec::new(),
        handling_coefficients: Vec::new(),
        final_demand_principals: Vec::new(),
        final_demand_orders: Vec::new(),
    };
    install_freight(&mut state);
    state
}

fn install_freight(state: &mut MaterialCircuitState) {
    let route = RouteId::from_bytes([1; 32]);
    let corridor = CorridorId::from_bytes([1; 32]);
    let order = OrderId::from_bytes([1; 32]);
    state.supplier_routes.push(SupplierRoute {
        transport_kind: SupplierTransport::Staged,
        buyer_site_id: site(1),
        supplier_site_id: site(2),
        good_id: good(1),
        unit_id: unit(2),
        route_id: route,
    });
    state.route_stages.push(RouteStage {
        route_id: route,
        stage_index: 0,
        from_node_id: LogisticsNodeId::from_bytes([2; 32]),
        to_node_id: LogisticsNodeId::from_bytes([1; 32]),
        travel_periods: 2,
        loss_ppm: 0,
    });
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: route,
        stage_index: 0,
        corridor_id: corridor,
    });
    state.orders.push(OrderRow {
        order_id: order,
        access_mode: OrderAccessMode::CommoditySale,
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
    state.backlog.push(BacklogRow {
        order_id: order,
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: corridor,
        period: 1,
        available_grams: 4000,
    });
}

fn staffed_labor() -> StaffingComposition {
    let pool = StaffingPoolBinding::try_new(
        StaffingPoolId::from_bytes([1; 32]),
        site(1),
        unit(1),
        1,
        StaffingPolicy::one_period(160).unwrap(),
        vec![StaffingWorkSource::Production(process())],
    )
    .unwrap();
    StaffingComposition::try_new(vec![StaffingNodeBinding::try_new(subject(), pool).unwrap()])
        .unwrap()
}

fn try_session(rules: &str, labor: StaffingComposition) -> Result<Session, MaterialReplayError> {
    let foundation = michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation().unwrap();
    let (_, parsed) = split_content(rules).unwrap();
    let forms = parsed.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    let graph = ReplayTickSession::new(
        SCENARIO,
        None,
        rules,
        HypergraphStore::new(),
        ReplaySessionId::try_from("staffing/replay-session").unwrap(),
        ReplaySeed::new(40),
        ContentDigest {
            defines_hash: [40; 32],
            rules_hash: rules_hash_of(&forms).unwrap(),
        },
        RefDigest::from_bytes(foundation.reference_bundle_digest()),
        MaterialState::try_new(foundation).unwrap(),
    )
    .map_err(MaterialReplayError::Graph)?;
    MaterialReplaySession::new(
        graph,
        MaterialWorldRegister::try_new(0, opening()).unwrap(),
        sha256_of(b"staffed-replay-fixture-foundation"),
        7,
        labor,
    )
}

fn session(rules: &str) -> Session {
    try_session(rules, staffed_labor()).unwrap()
}

fn prepare(session: &Session) -> Candidate {
    let actions = OrderedPracticeActionBatch::empty(
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
) -> IdentifiedMaterialTick {
    session
        .commit_prepared_and_publish(sink, candidate, |_| {
            Ok::<_, &'static str>(ReplayCommitDisposition::Committed)
        })
        .unwrap()
        .0
}

fn advance(session: &mut Session, sink: &mut CollectingSink) -> MaterialTickReceipts {
    let candidate = prepare(session);
    let receipts = decode_material_receipts(candidate.material().receipt_bytes()).unwrap();
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
        .find(|event| event.emitting_rule() == STAFFING_COMPOSITION_ID)
        .unwrap();
    let (_, value) = event
        .fields()
        .iter()
        .find(|(field, _)| field == name)
        .unwrap();
    let StableBslValue::Int(value) = value else {
        panic!("staffing receipt field {name} must be exact int");
    };
    *value
}

#[derive(Debug, PartialEq)]
struct LiveState {
    graph: Vec<u8>,
    graph_material: MaterialState,
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
    let foundation = michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation().unwrap();
    let graph_material = MaterialState::try_new(foundation).unwrap();
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

fn owned_checkpoint_rows(rows: &MaterialStateRows) -> MaterialStateRows {
    let owned = MaterialStateRows::try_from_rows(MaterialStateRowsInput {
        world_registers: rows.world_registers().rows().iter().map(|row| {
            WorldRegisterRow::try_new(row.qname().to_owned(), row.value().clone()).unwrap()
        }).collect(),
        territories: rows.territories().rows().iter().map(|row| {
            TerritoryStateRow::try_new(row.territory_id().clone(), row.ordered_fields().to_vec()).unwrap()
        }).collect(),
        dynamic_hexes: rows.dynamic_hexes().rows().iter().map(|row| {
            let [c, v, s, k, biocapacity_stock, energy_stock, raw_material_stock,
                internet_access_pct, surveillance_coupling] = row.value_bits();
            DynamicHexStateRow::try_new(row.cell_id(), MichiganDynamicHexValueBits {
                c, v, s, k, biocapacity_stock, energy_stock, raw_material_stock,
                internet_access_pct, surveillance_coupling,
            }).unwrap()
        }).collect(),
        organizations: rows.organizations().rows().iter().map(|row| {
            OrganizationStateRow::try_new(
                row.organization_id().clone(), row.organization_kind().clone(),
                row.ordered_territory_ids().to_vec(), row.ordered_fields().to_vec(),
            ).unwrap()
        }).collect(),
    }).unwrap();
    assert_eq!(owned.canonical_bytes(), rows.canonical_bytes());
    owned
}

#[test]
fn one_empty_period_holds_then_releases_and_real_arrival_rehires_for_next_period() {
    let mut session = session("");
    let mut sink = CollectingSink::default();
    let first = prepare(&session);
    assert_eq!(staffing_field(&first, "current-unretained-hours"), 0);
    assert_eq!(staffing_field(&first, "retained-hours"), 160);
    assert_eq!(staffing_field(&first, "separations"), 0);
    let receipts = decode_material_receipts(first.material().receipt_bytes()).unwrap();
    assert_eq!(
        (
            receipts.dispatches[0].quantity,
            receipts.dispatches[0].final_arrival_period
        ),
        (4, 3)
    );
    commit(&mut session, &mut sink, first);
    assert_people(&session, 1.0, 0.0, 0.0);
    assert_eq!(session.material().state().labor, vec![labor(2, 160)]);

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
    let receipts = decode_material_receipts(arrival.material().receipt_bytes()).unwrap();
    assert_eq!(receipts.arrivals.len(), 1);
    assert_eq!(receipts.arrivals[0].quantity, 4);
    assert!(receipts.production.is_empty());
    assert_eq!(
        arrival.material().register().state().production_commitments[0].planned_batches,
        1
    );
    commit(&mut session, &mut sink, arrival);
    assert_people(&session, 1.0, 0.0, 40.0);
    assert_eq!(session.material().state().labor, vec![labor(4, 160)]);

    let fourth = advance(&mut session, &mut sink);
    let fifth = advance(&mut session, &mut sink);
    assert_eq!(fourth.production[0].produced_batches, 1);
    assert_eq!(fifth.production[0].produced_batches, 1);
    assert_eq!(
        session.material().state().inventory,
        vec![
            InventoryRow {
                site_id: site(1),
                good_id: good(1),
                unit_id: unit(2),
                quantity: 0
            },
            InventoryRow {
                site_id: site(1),
                good_id: good(2),
                unit_id: unit(2),
                quantity: 10
            },
            InventoryRow {
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
        vec![BacklogRow {
            order_id: OrderId::from_bytes([1; 32]),
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
        Err::<ReplayCommitDisposition, _>("durable commit refused")
    });
    assert!(matches!(
        error,
        Err(MaterialCommitError::Commit("durable commit refused"))
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
    assert_eq!(events[0].emitting_rule(), STAFFING_COMPOSITION_ID);
    assert_eq!(events[0].choice_receipt(), None);
    assert!(events[0]
        .fields()
        .contains(&("subject".to_owned(), StableBslValue::Node(subject()))));
    for (name, expected) in [
        ("period", 2),
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
        .all(|row| row.rule_id == STAFFING_COMPOSITION_ID
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
        [STAFFING_COMPOSITION_ID, "zz-staffing/witness"]
    );
    assert!(events[1].fields().contains(&(
        "employed".to_owned(),
        StableBslValue::RealBits(0.0_f64.to_bits()),
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
        OrderedPracticeActionBatch::empty(session.graph_session().session_identity().clone(), 3)
            .unwrap();
    for _ in 0..2 {
        let result = session.prepare_advance(&actions);
        let Err(MaterialReplayError::Graph(ReplayTickError::Execution { message })) = result else {
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
    for field in STAFFING_FIELDS {
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
            let MaterialReplayError::Graph(ReplayTickError::MaterialBase(
                MaterialBaseError::StaffingFieldOwner {
                    rule_id,
                    field: actual,
                },
            )) = &error
            else {
                panic!("Staffed must refuse {anchor} write to {field}, guard {guard}: {error:?}");
            };
            assert_eq!(rule_id, "zz-staffing/foreign-writer");
            assert_eq!(actual, field);
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
    {
        let labor = staffed_labor();
        let result = try_session(source, labor);
        let Err(MaterialReplayError::Graph(ReplayTickError::Preparation { message })) = result
        else {
            panic!("the existing shape-verb gate must refuse before a session is created");
        };
        assert!(message.contains("remove-node"), "{message}");
        assert!(message.contains("graph-shape verbs"), "{message}");
    }
}
