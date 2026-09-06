use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{hypergraph_store::HypergraphStore, state_hash::CanonicalState};
use babylon_kernel::sha256_of;
use babylon_persistence::{
    michigan_economy::michigan_observer_foundation_v1,
    michigan_material::{michigan_material_foundation_v1, MichiganDeliveryPresetV1},
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::{
    material_replay::{MaterialCommitErrorV3, MaterialReplaySessionV3},
    material_world::{decode_material_receipts_v3, MaterialWorldRegisterV2},
    replay_session::ReplayCommitDispositionV1,
};

fn session(preset: MichiganDeliveryPresetV1) -> MaterialReplaySessionV3<HypergraphStore> {
    let (graph, _) = michigan_observer_foundation_v1().unwrap();
    let material =
        MaterialWorldRegisterV2::try_new(0, michigan_material_foundation_v1(preset).unwrap())
            .unwrap();
    MaterialReplaySessionV3::new(
        graph,
        material,
        sha256_of(preset.id().as_bytes()),
        preset.horizon_ticks(),
    )
    .unwrap()
}
fn actions(session: &MaterialReplaySessionV3<HypergraphStore>) -> OrderedPracticeActionBatchV1 {
    OrderedPracticeActionBatchV1::empty(
        session.graph_session().session_identity().clone(),
        session.completed_tick() + 1,
    )
    .unwrap()
}
#[test]
fn material_commit_failure_leaves_graph_circuit_world_time_and_sink_unchanged() {
    let mut session = session(MichiganDeliveryPresetV1::Standard);
    let graph = session.graph_session().graph().state_hash().unwrap();
    let register = session.material().canonical_bytes().to_vec();
    let world = session.current_world_hash().unwrap();
    let candidate = session.prepare_advance(&actions(&session)).unwrap();
    let hash = candidate.identity().tick_content_hash();
    let mut sink = CollectingSink::default();
    let refused = session.commit_prepared_and_publish(&mut sink, candidate, |_| {
        Err::<ReplayCommitDispositionV1, _>("refused before marker")
    });
    assert!(matches!(
        refused,
        Err(MaterialCommitErrorV3::Commit("refused before marker"))
    ));
    assert_eq!(session.completed_tick(), 0);
    assert_eq!(session.material().canonical_bytes(), register);
    assert_eq!(session.graph_session().graph().state_hash().unwrap(), graph);
    assert_eq!(session.current_world_hash().unwrap(), world);
    assert!(sink.events.is_empty());
    let retry = session.prepare_advance(&actions(&session)).unwrap();
    assert_eq!(retry.identity().tick_content_hash(), hash);
    let (ack, _) = session
        .commit_prepared_and_publish(&mut sink, retry, |_| {
            Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
        })
        .unwrap();
    assert_eq!(session.completed_tick(), 1);
    assert_eq!(session.graph_session().completed_tick(), 1);
    assert_eq!(session.graph_session().graph().state_hash().unwrap(), graph);
    assert_ne!(session.current_world_hash().unwrap(), world);
    assert_eq!(
        ack.result_world_hash(),
        session.current_world_hash().unwrap()
    );
}
#[test]
fn material_transition_failure_abandons_prepared_graph_and_identity() {
    let (graph, _) = michigan_observer_foundation_v1().unwrap();
    let mut initial = michigan_material_foundation_v1(MichiganDeliveryPresetV1::Standard).unwrap();
    let source = initial
        .production_commitments
        .iter()
        .find(|row| row.week == 1 && row.planned_batches > 0)
        .unwrap();
    let output = initial
        .process_outputs
        .iter()
        .find(|row| row.process_id == source.process_id)
        .unwrap()
        .clone();
    if let Some(stock) = initial.inventory.iter_mut().find(|row| {
        row.site_id == output.site_id
            && row.good_id == output.good_id
            && row.unit_id == output.unit_id
    }) {
        stock.quantity = u64::MAX;
    } else {
        initial
            .inventory
            .push(babylon_material_circuit::InventoryRowV1 {
                site_id: output.site_id,
                good_id: output.good_id,
                unit_id: output.unit_id,
                quantity: u64::MAX,
            });
    }
    let register = MaterialWorldRegisterV2::try_new(0, initial).unwrap();
    let session = MaterialReplaySessionV3::new(graph, register, [7; 32], 16).unwrap();
    let bytes = session.material().canonical_bytes().to_vec();
    let hash = session.current_world_hash().unwrap();
    assert!(session.prepare_advance(&actions(&session)).is_err());
    assert_eq!(session.completed_tick(), 0);
    assert_eq!(session.graph_session().completed_tick(), 0);
    assert_eq!(session.material().canonical_bytes(), bytes);
    assert_eq!(session.current_world_hash().unwrap(), hash);
}
#[test]
fn arrival_feeds_following_commitments_and_delay_changes_only_circuit_world() {
    let mut fast = session(MichiganDeliveryPresetV1::Standard);
    let mut slow = session(MichiganDeliveryPresetV1::Delayed);
    assert_eq!(
        fast.graph_session().graph().state_hash().unwrap(),
        slow.graph_session().graph().state_hash().unwrap()
    );
    let mut first_fast_delivery = None;
    let mut first_slow_delivery = None;
    for tick in 1..=16 {
        for (session, first) in [
            (&mut fast, &mut first_fast_delivery),
            (&mut slow, &mut first_slow_delivery),
        ] {
            let candidate = session.prepare_advance(&actions(session)).unwrap();
            let receipts =
                decode_material_receipts_v3(candidate.material().receipt_bytes()).unwrap();
            assert_eq!(receipts.resolve_tick, tick);
            let decoded =
                MaterialWorldRegisterV2::decode(candidate.material().register().canonical_bytes())
                    .unwrap();
            assert_eq!(&decoded, candidate.material().register());
            for produced in &receipts.production {
                let prior = session
                    .material()
                    .state()
                    .production_commitments
                    .iter()
                    .find(|row| {
                        row.week == tick
                            && row.process_id == produced.process_id
                            && row.site_id == produced.site_id
                    });
                assert_eq!(
                    prior.map_or(0, |row| row.planned_batches),
                    produced.planned_batches
                );
                assert!(produced.produced_batches <= produced.planned_batches);
            }
            if !receipts.deliveries.is_empty() {
                first.get_or_insert(tick);
            }
            session
                .commit_prepared_and_publish(&mut CollectingSink::default(), candidate, |_| {
                    Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
                })
                .unwrap();
        }
    }
    assert!(first_fast_delivery.is_some());
    assert!(first_slow_delivery.is_some());
    assert!(fast.prepare_advance(&actions(&fast)).is_err());
    assert!(slow.prepare_advance(&actions(&slow)).is_err());
}

#[test]
fn committed_identity_can_publish_without_heap_allocation() {
    fn requires_copy<T: Copy>() {}
    requires_copy::<babylon_tick::material_replay::IdentifiedMaterialTickV3>();
    let session = session(MichiganDeliveryPresetV1::Standard);
    let candidate = session.prepare_advance(&actions(&session)).unwrap();
    let identity = *candidate.identity();
    assert_eq!(
        babylon_tick::material_replay::IdentifiedMaterialTickV3::decode(identity.canonical_bytes())
            .unwrap(),
        identity
    );
}
