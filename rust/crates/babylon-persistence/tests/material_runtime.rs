use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{hypergraph_store::HypergraphStore, state_hash::CanonicalState};
use babylon_persistence::{
    michigan_content::MichiganContentPreset, michigan_economy::michigan_observer_foundation,
    michigan_material::MichiganDeliveryPreset,
};
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::{
    material_replay::{MaterialCommitError, MaterialReplaySession},
    material_world::{decode_material_receipts, MaterialWorldRegister},
    replay_session::ReplayCommitDisposition,
};

fn session(preset: MichiganDeliveryPreset) -> MaterialReplaySession<HypergraphStore> {
    MichiganContentPreset::new_campaign(preset)
        .create_foundation(&crate::test_support::catalog())
        .unwrap()
        .into_session()
        .unwrap()
}
fn actions(session: &MaterialReplaySession<HypergraphStore>) -> OrderedPracticeActionBatch {
    OrderedPracticeActionBatch::empty(
        session.graph_session().session_identity().clone(),
        session.completed_tick() + 1,
    )
    .unwrap()
}
#[test]
fn material_commit_failure_leaves_graph_circuit_world_time_and_sink_unchanged() {
    for preset in [
        MichiganDeliveryPreset::Standard,
        MichiganDeliveryPreset::SharedFreightAmple,
        MichiganDeliveryPreset::SharedFreightConstrained,
    ] {
        let mut session = session(preset);
        let graph = session.graph_session().graph().state_hash().unwrap();
        let register = session.material().canonical_bytes().to_vec();
        let world = session.current_world_hash().unwrap();
        let candidate = session.prepare_advance(&actions(&session)).unwrap();
        let prepared_receipt_bytes = candidate.material().receipt_bytes().to_vec();
        let receipts = decode_material_receipts(&prepared_receipt_bytes).unwrap();
        let catalog = crate::test_support::catalog();
        let (sheet, meal) = if preset == MichiganDeliveryPreset::SharedFreightConstrained {
            (120, 40)
        } else {
            (320, 80)
        };
        for (key, expected) in [("sheet-transfer", sheet), ("food-transfer", meal)] {
            let order_id = catalog
                .routes()
                .iter()
                .find(|route| route.key == key)
                .unwrap()
                .order_id();
            assert_eq!(
                receipts
                    .dispatches
                    .iter()
                    .find(|row| row.order_id == order_id)
                    .unwrap()
                    .quantity,
                expected
            );
        }
        let hash = candidate.identity().tick_content_hash();
        let mut sink = CollectingSink::default();
        let refused = session.commit_prepared_and_publish(&mut sink, candidate, |_| {
            Err::<ReplayCommitDisposition, _>("refused before marker")
        });
        assert!(matches!(
            refused,
            Err(MaterialCommitError::Commit("refused before marker"))
        ));
        assert_eq!(session.completed_tick(), 0);
        assert_eq!(session.material().canonical_bytes(), register);
        assert_eq!(session.graph_session().graph().state_hash().unwrap(), graph);
        assert_eq!(session.current_world_hash().unwrap(), world);
        assert!(sink.events.is_empty());
        let retry = session.prepare_advance(&actions(&session)).unwrap();
        assert_eq!(retry.identity().tick_content_hash(), hash);
        assert_eq!(retry.material().receipt_bytes(), prepared_receipt_bytes);
        let (ack, _) = session
            .commit_prepared_and_publish(&mut sink, retry, |_| {
                Ok::<_, ()>(ReplayCommitDisposition::Committed)
            })
            .unwrap();
        assert_eq!(session.completed_tick(), 1);
        assert_eq!(session.graph_session().completed_tick(), 1);
        assert_ne!(session.graph_session().graph().state_hash().unwrap(), graph);
        assert_ne!(session.current_world_hash().unwrap(), world);
        assert_eq!(
            ack.result_world_hash(),
            session.current_world_hash().unwrap()
        );
    }
}
#[test]
fn material_transition_failure_abandons_prepared_graph_and_identity() {
    let (graph, _) = michigan_observer_foundation().unwrap();
    let mut initial = MichiganContentPreset::FourWeekStandard
        .create_foundation(&crate::test_support::catalog())
        .unwrap()
        .initial_register()
        .state()
        .clone();
    let source = initial
        .production_commitments
        .iter()
        .find(|row| row.period == 1 && row.planned_batches > 0)
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
            .push(babylon_material_circuit::InventoryRow {
                site_id: output.site_id,
                good_id: output.good_id,
                unit_id: output.unit_id,
                quantity: u64::MAX,
            });
    }
    let register = MaterialWorldRegister::try_new(0, initial).unwrap();
    let session = MaterialReplaySession::new(
        graph,
        register,
        [7; 32],
        16,
        MichiganContentPreset::FourWeekStandard
            .create_foundation(&crate::test_support::catalog())
            .unwrap()
            .labor()
            .clone(),
    )
    .unwrap();
    let bytes = session.material().canonical_bytes().to_vec();
    let hash = session.current_world_hash().unwrap();
    assert!(session.prepare_advance(&actions(&session)).is_err());
    assert_eq!(session.completed_tick(), 0);
    assert_eq!(session.graph_session().completed_tick(), 0);
    assert_eq!(session.material().canonical_bytes(), bytes);
    assert_eq!(session.current_world_hash().unwrap(), hash);
}
#[test]
fn staffed_arrival_feeds_following_commitments_through_the_full_horizon() {
    let mut fast = session(MichiganDeliveryPreset::Standard);
    let mut slow = session(MichiganDeliveryPreset::Delayed);
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
            let receipts = decode_material_receipts(candidate.material().receipt_bytes()).unwrap();
            assert_eq!(receipts.resolve_tick, tick);
            let decoded =
                MaterialWorldRegister::decode(candidate.material().register().canonical_bytes())
                    .unwrap();
            assert_eq!(&decoded, candidate.material().register());
            for produced in &receipts.production {
                let prior = session
                    .material()
                    .state()
                    .production_commitments
                    .iter()
                    .find(|row| {
                        row.period == tick
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
                    Ok::<_, ()>(ReplayCommitDisposition::Committed)
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
    requires_copy::<babylon_tick::material_replay::IdentifiedMaterialTick>();
    let session = session(MichiganDeliveryPreset::Standard);
    let candidate = session.prepare_advance(&actions(&session)).unwrap();
    let identity = *candidate.identity();
    assert_eq!(
        babylon_tick::material_replay::IdentifiedMaterialTick::decode(identity.canonical_bytes())
            .unwrap(),
        identity
    );
}

#[path = "support/material_config.rs"]
mod test_support;
