//! Synthetic scale and accounting evidence, never real infrastructure qualification.
//! Regenerate the small fixture with `fixtures/generate_statewide_synthetic.py`.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    GoodIdV1, MaterialCircuitStateV3, MerchantRoleV3, OrderIdV1, SiteIdV1, UnitIdV1,
    MAX_MATERIAL_CIRCUIT_ROWS_V1,
};
use babylon_persistence::{
    michigan_content::{admit_michigan_content_v1, MichiganContentPresetV1},
    michigan_material::{
        MichiganDeliveryPresetV1, MichiganMaterialCatalogV1, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2,
    },
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::{
    material_replay::{MaterialReplayErrorV3, MaterialReplaySessionV3, PreparedMaterialTickV3},
    material_world::{
        decode_material_receipts_v4, MaterialTickReceiptsV4, MaterialWorldRegisterV3,
        MAX_MATERIAL_WORLD_REGISTER_BYTES_V3,
    },
    replay_session::ReplayCommitDispositionV1,
};
use std::{collections::BTreeMap, time::Instant};

type Session = MaterialReplaySessionV3<HypergraphStore>;
type Goods = BTreeMap<(GoodIdV1, UnitIdV1), u128>;

#[path = "fixtures/statewide_synthetic.rs"]
mod synthetic;

#[test]
fn physical_paths_require_all_overlapping_capacity_memberships_and_preserve_identity() {
    use babylon_persistence::michigan_material::MichiganMaterialPathV2;
    let fixture = synthetic::load();
    let qualification = serde_json::to_vec(&fixture.qualification).unwrap();
    let mut physical = fixture.physical;
    let mut overlap = physical.capacity_groups[0].clone();
    overlap.key = "second-shared-road-service".to_owned();
    overlap.label = "Second Designed road service".to_owned();
    physical.capacity_groups.push(overlap);
    let compile = |network| {
        MichiganMaterialCatalogV1::from_statewide_qualification(
            include_str!("../../../../content/scenarios/michigan/defines.toml"),
            &qualification,
            network,
            Vec::new(),
        )
    };
    let catalog = compile(physical.clone()).unwrap();
    let mut routed = 0;
    for route in catalog.routes() {
        if let MichiganMaterialPathV2::Routed { capacity_keys, .. } = &route.path {
            assert_eq!(
                capacity_keys,
                &["second-shared-road-service", "synthetic-shared-road"]
            );
            routed += 1;
        }
    }
    assert!(routed > 0);
    physical.edges.reverse();
    physical.capacity_groups.reverse();
    for group in &mut physical.capacity_groups {
        group.edge_keys.reverse();
    }
    assert_eq!(catalog, compile(physical.clone()).unwrap());
    let uncovered = physical.edges[0].id.clone();
    for group in &mut physical.capacity_groups {
        group.edge_keys.retain(|key| key != &uncovered);
    }
    assert!(
        compile(physical).is_err(),
        "a routed edge needs its capacity membership"
    );
}

fn prepare(session: &Session) -> PreparedMaterialTickV3<HypergraphStore> {
    let actions = OrderedPracticeActionBatchV1::empty(
        session.graph_session().session_identity().clone(),
        session.completed_tick() + 1,
    )
    .unwrap();
    session.prepare_advance(&actions).unwrap()
}

fn stock_and_transit(state: &MaterialCircuitStateV3) -> Goods {
    let mut totals = Goods::new();
    for row in &state.inventory {
        *totals.entry((row.good_id, row.unit_id)).or_default() += u128::from(row.quantity);
    }
    for row in &state.freight {
        *totals.entry((row.good_id, row.unit_id)).or_default() += u128::from(row.quantity);
    }
    totals
}

fn assert_goods_conserved(
    opening: &MaterialCircuitStateV3,
    closing: &MaterialCircuitStateV3,
    receipts: &MaterialTickReceiptsV4,
) {
    let mut available = stock_and_transit(opening);
    let mut accounted = stock_and_transit(closing);
    for receipt in &receipts.production {
        let output = opening
            .process_outputs
            .iter()
            .find(|row| row.process_id == receipt.process_id)
            .unwrap();
        assert_eq!(receipt.site_id, output.site_id);
        *available
            .entry((output.good_id, output.unit_id))
            .or_default() +=
            u128::from(output.quantity_per_batch) * u128::from(receipt.produced_batches);
        for input in opening
            .input_coefficients
            .iter()
            .filter(|row| row.process_id == receipt.process_id)
        {
            *accounted.entry((input.good_id, input.unit_id)).or_default() +=
                u128::from(input.quantity_per_batch) * u128::from(receipt.produced_batches);
        }
    }
    for receipt in &receipts.losses {
        let order = opening
            .orders
            .iter()
            .find(|row| row.order_id == receipt.order_id)
            .unwrap();
        *accounted.entry((order.good_id, order.unit_id)).or_default() +=
            u128::from(receipt.quantity);
    }
    for receipt in &receipts.local_fulfillments {
        *accounted
            .entry((receipt.good_id, receipt.unit_id))
            .or_default() += u128::from(receipt.quantity);
    }
    available.retain(|_, value| *value != 0);
    accounted.retain(|_, value| *value != 0);
    assert_eq!(
        available, accounted,
        "period {} must balance each native good-unit separately; internal transfers are not sinks",
        opening.period
    );
}

fn assert_order_accounts(
    opening: &MaterialCircuitStateV3,
    state: &MaterialCircuitStateV3,
    receipts: &MaterialTickReceiptsV4,
    fulfilled: &mut BTreeMap<OrderIdV1, u128>,
) {
    for receipt in &receipts.local_fulfillments {
        let order = state
            .final_demand_orders
            .iter()
            .find(|row| row.order_id == receipt.order_id)
            .unwrap();
        assert_eq!(receipt.retailer_site_id, order.retailer_site_id);
        assert_eq!(receipt.demand_principal_id, order.demand_principal_id);
        assert_eq!(
            (receipt.good_id, receipt.unit_id),
            (order.good_id, order.unit_id)
        );
        *fulfilled.entry(receipt.order_id).or_default() += u128::from(receipt.quantity);
    }
    assert_eq!(
        state.final_demand_orders.len(),
        opening.final_demand_orders.len()
    );
    for (order, prior) in state
        .final_demand_orders
        .iter()
        .zip(&opening.final_demand_orders)
    {
        assert_eq!(order.order_id, prior.order_id);
        assert_eq!(order.ordered, prior.ordered, "finite demand never renews");
        assert!(order.fulfilled <= order.ordered);
        assert_eq!(
            u128::from(order.fulfilled),
            fulfilled.get(&order.order_id).copied().unwrap_or(0),
            "the finite order is credited exactly once for each receipted handoff"
        );
    }
    for order in &state.orders {
        let transit = state
            .freight
            .iter()
            .filter(|lot| lot.order_id == order.order_id)
            .map(|lot| u128::from(lot.quantity))
            .sum::<u128>();
        assert_eq!(
            u128::from(order.shipped),
            u128::from(order.delivered) + u128::from(order.lost) + transit
        );
        assert_eq!(order.realized, order.delivered);
        assert!(order.shipped <= order.ordered);
    }
}

fn assert_opening_labor_limits(
    opening: &MaterialCircuitStateV3,
    receipts: &MaterialTickReceiptsV4,
) {
    let mut used: BTreeMap<(SiteIdV1, UnitIdV1), u128> = BTreeMap::new();
    for receipt in &receipts.production {
        let coefficient = opening
            .labor_coefficients
            .iter()
            .find(|row| row.process_id == receipt.process_id)
            .unwrap();
        *used
            .entry((receipt.site_id, coefficient.unit_id))
            .or_default() +=
            u128::from(receipt.produced_batches) * u128::from(coefficient.quantity_per_batch);
    }
    for receipt in &receipts.handling {
        let merchant = opening
            .merchants
            .iter()
            .find(|row| row.site_id == receipt.site_id)
            .unwrap();
        *used
            .entry((receipt.site_id, merchant.labor_unit_id))
            .or_default() += u128::from(receipt.used_hours);
    }
    for ((site, unit), hours) in used {
        let budget = opening
            .labor
            .iter()
            .find(|row| row.site_id == site && row.unit_id == unit && row.period == opening.period)
            .map_or(0, |row| row.available);
        assert!(
            hours <= u128::from(budget),
            "one owner shares one opening labor budget"
        );
    }
}

fn maximum_row_count(state: &MaterialCircuitStateV3) -> usize {
    [
        state.site_logistics_nodes.len(),
        state.process_outputs.len(),
        state.input_coefficients.len(),
        state.labor_coefficients.len(),
        state.freight_mass_coefficients.len(),
        state.supplier_routes.len(),
        state.route_stages.len(),
        state.route_stage_capacities.len(),
        state.inventory.len(),
        state.orders.len(),
        state.backlog.len(),
        state.freight.len(),
        state.corridor_capacities.len(),
        state.capacities.len(),
        state.labor.len(),
        state.production_commitments.len(),
        state.merchants.len(),
        state.handling_coefficients.len(),
        state.final_demand_principals.len(),
        state.final_demand_orders.len(),
    ]
    .into_iter()
    .max()
    .unwrap()
}

#[test]
fn synthetic_statewide_roster_completes_sixteen_authenticated_periods_with_native_accounting() {
    let started = Instant::now();
    let catalog = synthetic::catalog();
    assert_eq!(catalog.sites().len(), 397);
    assert_eq!(catalog.owners().len(), 397);
    assert_eq!(catalog.processes().len(), 233);
    assert_eq!(catalog.merchants().len(), 166);
    assert_eq!(catalog.final_demands().len(), 233);
    assert_eq!(catalog.staffing().pools.len(), 397);
    assert!(catalog.routes().len() > 500);
    assert!(catalog.defines_bytes().len() < MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2);
    let (mut session, foundation_bytes) = captured_session(&catalog);
    let compile_time = started.elapsed();
    let mut fulfilled = BTreeMap::new();
    let mut maximum_register_bytes = session.material().canonical_bytes().len();
    let mut maximum_receipt_bytes = 0;
    let mut maximum_rows = maximum_row_count(session.material().state());
    let mut dispatch_count = 0;
    let mut local_count = 0;
    let advancing = Instant::now();
    for period in 1..=16 {
        let prepared = prepare(&session);
        let bytes = prepared.material().receipt_bytes();
        assert_eq!(sha256_of(bytes), prepared.identity().receipt_digest());
        let receipts = decode_material_receipts_v4(bytes).unwrap();
        assert_eq!(receipts.resolve_tick, period);
        let register = prepared.material().register();
        assert_eq!(
            MaterialWorldRegisterV3::decode(register.canonical_bytes()).unwrap(),
            *register
        );
        maximum_register_bytes = maximum_register_bytes.max(register.canonical_bytes().len());
        maximum_receipt_bytes = maximum_receipt_bytes.max(bytes.len());
        maximum_rows = maximum_rows.max(maximum_row_count(register.state()));
        assert_goods_conserved(session.material().state(), register.state(), &receipts);
        assert_order_accounts(
            session.material().state(),
            register.state(),
            &receipts,
            &mut fulfilled,
        );
        assert_opening_labor_limits(session.material().state(), &receipts);
        dispatch_count += receipts.dispatches.len();
        local_count += receipts.local_transfers.len();
        session
            .commit_prepared_and_publish(&mut CollectingSink::default(), prepared, |_| {
                Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
            })
            .unwrap();
    }
    assert!(maximum_register_bytes < MAX_MATERIAL_WORLD_REGISTER_BYTES_V3);
    assert!(maximum_receipt_bytes < MAX_MATERIAL_WORLD_REGISTER_BYTES_V3);
    assert!(maximum_rows < MAX_MATERIAL_CIRCUIT_ROWS_V1);
    assert!(dispatch_count > 0 && local_count > 0 && !fulfilled.is_empty());
    let state = session.material().state();
    let unsold = unsold_merchandise(state);
    assert!(unsold.values().any(|quantity| *quantity > 0));
    let named_unsold = named_goods(&catalog, &unsold);
    let unfilled = state
        .final_demand_orders
        .iter()
        .filter(|r| r.fulfilled < r.ordered)
        .count();
    let before = session.current_world_hash().unwrap();
    let actions =
        OrderedPracticeActionBatchV1::empty(session.graph_session().session_identity().clone(), 17)
            .unwrap();
    assert!(matches!(
        session.prepare_advance(&actions),
        Err(MaterialReplayErrorV3::Horizon)
    ));
    assert_eq!(before, session.current_world_hash().unwrap());
    assert_eq!(session.completed_tick(), 16);
    eprintln!(
        "SYNTHETIC SCALE ONLY: owners=397 counties=83 processes=233 routes={} captured_bytes={} foundation_bytes={foundation_bytes} max_register_bytes={maximum_register_bytes} max_receipt_bytes={maximum_receipt_bytes} max_rows={maximum_rows} dispatches={dispatch_count} local_transfers={local_count} fulfilled_orders={} unfilled_orders={unfilled} compile_and_admit_ms={} advance_16_ms={} unsold_native_by_good={named_unsold:?}",
        catalog.routes().len(), catalog.defines_bytes().len(), fulfilled.len(),
        compile_time.as_millis(), advancing.elapsed().as_millis(),
    );
}

fn unsold_merchandise(state: &MaterialCircuitStateV3) -> Goods {
    let merchants: BTreeMap<_, _> = state
        .merchants
        .iter()
        .map(|m| (m.site_id, m.role))
        .collect();
    let mut unsold = Goods::new();
    for row in &state.inventory {
        if matches!(
            merchants.get(&row.site_id),
            Some(MerchantRoleV3::Wholesale | MerchantRoleV3::Retail)
        ) {
            *unsold.entry((row.good_id, row.unit_id)).or_default() += u128::from(row.quantity);
        }
    }
    unsold
}

fn captured_session(catalog: &MichiganMaterialCatalogV1) -> (Session, usize) {
    let preset = MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::StatewideBaseline);
    let foundation = preset.create_foundation(catalog).unwrap();
    let foundation_bytes = foundation.canonical_bytes().len();
    assert!(foundation_bytes < MAX_MATERIAL_WORLD_REGISTER_BYTES_V3);
    assert_eq!(
        foundation
            .initial_register()
            .state()
            .final_demand_principals
            .len(),
        83
    );
    let state = foundation.initial_register().state();
    assert_eq!(state.site_logistics_nodes.len(), 397);
    assert_eq!(state.process_outputs.len(), 233);
    assert_eq!(state.merchants.len(), 166);
    assert_eq!(state.supplier_routes.len(), catalog.routes().len());
    assert_eq!(state.final_demand_orders.len(), 233);
    assert_eq!(state.labor.len(), 397);
    let admitted = admit_michigan_content_v1(
        preset.id(),
        16,
        &foundation.spec().content_digest,
        &foundation.digest(),
        0,
        foundation.canonical_bytes(),
    )
    .unwrap();
    assert_eq!(admitted.digest(), foundation.digest());
    let mut corrupted = foundation.canonical_bytes().to_vec();
    *corrupted.last_mut().unwrap() ^= 1;
    assert!(admit_michigan_content_v1(
        preset.id(),
        16,
        &foundation.spec().content_digest,
        &foundation.digest(),
        0,
        &corrupted,
    )
    .is_err());
    (foundation.into_session().unwrap(), foundation_bytes)
}

fn named_goods(
    catalog: &MichiganMaterialCatalogV1,
    quantities: &Goods,
) -> Vec<(String, String, u128)> {
    catalog
        .goods()
        .iter()
        .filter_map(|good| {
            let quantity = quantities
                .get(&(good.id(), good.unit_id()))
                .copied()
                .unwrap_or(0);
            (quantity > 0).then(|| (good.key.clone(), good.unit_key.clone(), quantity))
        })
        .collect()
}

#[test]
fn new_reads_pinned_siblings_and_saved_open_survives_changed_or_missing_source_files() {
    use babylon_persistence::michigan_material::MichiganMaterialErrorV1;
    use babylon_persistence::MichiganDefinesErrorV1;
    let sources = synthetic::SyntheticSources::create();
    let defines_path = sources.path("defines.toml");
    let delivery = MichiganDeliveryPresetV1::StatewideBoth;
    let preset = MichiganContentPresetV1::new_campaign(delivery);
    let catalog = MichiganMaterialCatalogV1::load_for_preset(&defines_path, delivery).unwrap();
    let selected = catalog.with_preset(delivery).unwrap();
    assert_eq!(selected.sites().len(), 397);
    assert_eq!(
        selected
            .corridors()
            .iter()
            .find(|row| row.key == "synthetic-shared-road")
            .unwrap()
            .capacity_grams_per_period,
        50_000_000
    );
    assert!(selected.processes().iter().any(|process| {
        process.output_good_key == "prepared_food"
            && process
                .inputs
                .iter()
                .any(|input| input.good_key == "paper_packaging" && input.opening_quantity == 0)
    }));
    let foundation = preset.create_foundation(&catalog).unwrap();
    for name in [
        "statewide-qualification.json.gz",
        "statewide-physical.json.gz",
    ] {
        let path = sources.path(name);
        let original = std::fs::read(&path).unwrap();
        let mut changed = original.clone();
        *changed.last_mut().unwrap() ^= 1;
        std::fs::write(&path, changed).unwrap();
        assert!(matches!(
            MichiganMaterialCatalogV1::load_for_preset(&defines_path, delivery),
            Err(MichiganDefinesErrorV1::Material(
                MichiganMaterialErrorV1::ArtifactDigest
            ))
        ));
        std::fs::write(path, original).unwrap();
    }
    // All current source files become unavailable. Open only receives the
    // admitted campaign header and its captured foundation, including overrides.
    for name in [
        "defines.toml",
        "statewide-sources.json",
        "statewide-qualification.json.gz",
        "statewide-physical.json.gz",
    ] {
        std::fs::remove_file(sources.path(name)).unwrap();
    }
    assert!(MichiganMaterialCatalogV1::load_for_preset(&defines_path, delivery).is_err());
    let opened = admit_michigan_content_v1(
        preset.id(),
        16,
        &foundation.spec().content_digest,
        &foundation.digest(),
        0,
        foundation.canonical_bytes(),
    )
    .unwrap();
    assert_eq!(opened.preset(), preset);
    assert_eq!(opened.digest(), foundation.digest());
}
