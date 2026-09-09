//! Full-size persisted accounting on explicitly synthetic transport geometry.
//! Real road and intervention qualification remain separate acceptance evidence.

use super::{
    advance_material_period, identity_hex, install_observer_economy_schema_v1,
    install_reader_role_v1, CampaignId, DisposableTarget, DurableMaterialRuntimeV3,
    MichiganContentPresetV1, MichiganDeliveryPresetV1, ObserverEconomyReaderV1,
    ObserverVisibilityV1, Uuid,
};
use babylon_persistence::{ObserverEconomySnapshotV1, ProductionSnapshotV2};
use std::{collections::BTreeSet, time::Instant};

#[path = "../fixtures/statewide_synthetic.rs"]
mod synthetic;

fn assert_projection(snapshot: &ObserverEconomySnapshotV1, runtime: &DurableMaterialRuntimeV3) {
    assert_eq!(snapshot.resolve_tick, runtime.session().completed_tick());
    assert!(snapshot.production_evidence_digest().unwrap().is_some());
    let rows = snapshot.production.as_ref().unwrap();
    assert_eq!(rows.sites.len(), 397);
    assert_eq!(
        rows.sites
            .iter()
            .map(|site| &site.county_geoid)
            .collect::<BTreeSet<_>>()
            .len(),
        83
    );
    assert_eq!(
        rows.sites
            .iter()
            .map(|site| site.processes.len())
            .sum::<usize>(),
        233
    );
    assert_eq!(rows.merchant_handling_accounts.len(), 166);
    assert_eq!(rows.staffing_accounts.len(), 397);
    assert_eq!(rows.labor_accounts.len(), 397);
    assert!(rows
        .road_source
        .as_ref()
        .unwrap()
        .pbf_url
        .starts_with("synthetic://"));
    let mut pools = BTreeSet::new();
    for pool in &rows.staffing_accounts {
        assert!(pools.insert(&pool.pool_id));
        assert_eq!(pool.employed + pool.reserve, pool.labor_force);
    }
    for account in &rows.labor_accounts {
        if let Some(done) = &account.completed {
            assert_eq!(done.used + done.unused, done.opening);
            assert!(done.handling_used <= done.used);
        }
    }
    assert_final_orders(rows, runtime);
    if snapshot.resolve_tick == 0 {
        assert!(rows.material_balance.is_none());
        assert!(rows
            .merchant_handling_accounts
            .iter()
            .all(|row| row.completed.is_none()));
    } else {
        assert_stock_accounts(rows, snapshot.resolve_tick);
    }
}

fn assert_final_orders(rows: &ProductionSnapshotV2, runtime: &DurableMaterialRuntimeV3) {
    let mut orders = BTreeSet::new();
    let state = runtime.session().material().state();
    for account in &rows.final_demand_accounts {
        assert_eq!(account.fulfilled + account.outstanding, account.ordered);
        assert_eq!(
            account.fulfilled,
            account
                .orders
                .iter()
                .map(|order| order.fulfilled)
                .sum::<u64>()
        );
        for order in &account.orders {
            assert!(orders.insert(&order.order_id));
            let committed = state
                .final_demand_orders
                .iter()
                .find(|row| identity_hex(row.order_id.as_bytes()) == order.order_id)
                .unwrap();
            assert_eq!(order.fulfilled, committed.fulfilled);
            assert_eq!(order.ordered, committed.ordered);
        }
    }
    assert_eq!(orders.len(), 233);
}

fn assert_stock_accounts(rows: &ProductionSnapshotV2, period: u64) {
    let balance = rows.material_balance.as_ref().unwrap();
    assert_eq!(balance.period, period);
    let mut principals = BTreeSet::new();
    for row in &balance.rows {
        assert!(principals.insert((&row.site_id, &row.good_id, &row.unit_id)));
        assert_eq!(
            u128::from(row.opening)
                + u128::from(row.arrivals)
                + u128::from(row.local_received)
                + u128::from(row.produced),
            u128::from(row.consumed)
                + u128::from(row.dispatched)
                + u128::from(row.local_transferred)
                + u128::from(row.final_demand_fulfilled)
                + u128::from(row.closing),
        );
        let site = rows
            .sites
            .iter()
            .find(|site| site.id == row.site_id)
            .unwrap();
        let closing = site
            .inventory
            .iter()
            .find(|stock| stock.good_id == row.good_id && stock.unit_id == row.unit_id)
            .map_or(0, |stock| stock.quantity);
        assert_eq!(row.closing, closing);
    }
}

fn assert_same_world(left: &DurableMaterialRuntimeV3, right: &DurableMaterialRuntimeV3) {
    assert_eq!(left.tail(), right.tail());
    assert_eq!(
        left.session().material().canonical_bytes(),
        right.session().material().canonical_bytes()
    );
    assert_eq!(
        left.session().current_world_hash().unwrap(),
        right.session().current_world_hash().unwrap()
    );
}

#[test]
#[ignore = "requires the owned PostgreSQL harness; synthetic full-roster circulation proof"]
fn statewide_synthetic_circulation_survives_sixteen_persisted_periods_and_held_history() {
    let mut target = DisposableTarget::create();
    let start = Instant::now();
    let sources = synthetic::SyntheticSources::create();
    let catalog =
        babylon_persistence::michigan_material::MichiganMaterialCatalogV1::load_for_preset(
            &sources.path("defines.toml"),
            MichiganDeliveryPresetV1::StatewideBaseline,
        )
        .unwrap();
    let foundation =
        MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::StatewideBaseline)
            .create_foundation(&catalog)
            .unwrap();
    let foundation_digest = foundation.digest();
    let campaign = CampaignId::from_uuid(Uuid::from_u128(29_701));
    let mut runtime =
        DurableMaterialRuntimeV3::create(&target.writer, campaign, foundation).unwrap();
    let reference_foundation =
        MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::StatewideBaseline)
            .create_foundation(&synthetic::catalog())
            .unwrap();
    let mut reference = reference_foundation.into_session().unwrap();
    // Saved PostgreSQL authority must outlive every current input file.
    drop(sources);
    install_reader_role_v1(&target.writer).unwrap();
    install_observer_economy_schema_v1(&target.writer).unwrap();
    let config = target.login("babylon_observer", "statewidescale");
    let observer =
        ObserverEconomyReaderV1::connect(&config, ObserverVisibilityV1::FullObserver).unwrap();
    let creation = start.elapsed();
    let mut held = Vec::new();
    let mut advance_time = std::time::Duration::ZERO;
    let mut projection_time = std::time::Duration::ZERO;
    let mut resume_time = std::time::Duration::ZERO;
    for period in 0..=16 {
        if period > 0 {
            let began = Instant::now();
            advance_material_period(&mut runtime);
            advance_reference(&mut reference);
            advance_time += began.elapsed();
            assert_eq!(
                runtime.session().material().canonical_bytes(),
                reference.material().canonical_bytes()
            );
            assert_eq!(
                runtime.session().current_world_hash().unwrap(),
                reference.current_world_hash().unwrap()
            );
        }
        let began = Instant::now();
        let snapshot = observer
            .snapshot(campaign, period)
            .unwrap_or_else(|error| panic!("statewide observation period {period}: {error:?}"));
        projection_time += began.elapsed();
        assert_projection(&snapshot, &runtime);
        if [0, 1, 2, 8, 16].contains(&period) {
            held.push(snapshot.clone());
        }
        if [1, 2, 16].contains(&period) {
            let began = Instant::now();
            let reopened =
                DurableMaterialRuntimeV3::open(&target.writer, campaign, foundation_digest)
                    .unwrap();
            resume_time += began.elapsed();
            assert_same_world(&runtime, &reopened);
            runtime = reopened;
        }
    }
    for snapshot in &held {
        assert_eq!(
            observer.snapshot(campaign, snapshot.resolve_tick).unwrap(),
            *snapshot
        );
    }
    let final_rows = held.last().unwrap().production.as_ref().unwrap();
    assert!(final_rows
        .final_demand_accounts
        .iter()
        .any(|row| row.fulfilled > 0));
    assert!(final_rows
        .final_demand_accounts
        .iter()
        .any(|row| row.retail_stock_on_hand > 0));
    assert_eq!(observer.campaigns().unwrap()[0].durable_tick, 16);
    let preview_config = target.login("babylon_reader", "statewidepreview");
    let preview =
        ObserverEconomyReaderV1::connect(&preview_config, ObserverVisibilityV1::KnownPreview)
            .unwrap();
    let restricted = preview.snapshot(campaign, 16).unwrap();
    assert!(restricted.production.is_none());
    assert!(restricted.production_evidence_digest().unwrap().is_none());
    eprintln!("synthetic statewide PostgreSQL: creation={creation:?}, advance16_with_reference={advance_time:?}, projection17={projection_time:?}, resume3={resume_time:?}");
}

fn advance_reference(
    reference: &mut babylon_tick::material_replay::MaterialReplaySessionV3<
        babylon_graph::hypergraph_store::HypergraphStore,
    >,
) {
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
    use babylon_tick::replay_session::ReplayCommitDispositionV1;
    let actions = OrderedPracticeActionBatchV1::empty(
        reference.graph_session().session_identity().clone(),
        reference.completed_tick() + 1,
    )
    .unwrap();
    let prepared = reference.prepare_advance(&actions).unwrap();
    reference
        .commit_prepared_and_publish(&mut CollectingSink::default(), prepared, |_| {
            Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
        })
        .unwrap();
}
