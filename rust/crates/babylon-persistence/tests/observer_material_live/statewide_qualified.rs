//! Opt-in qualification of the actual captured statewide sources on owned `PostgreSQL`.
//! The ordinary reader focus excludes this full four-preset, sixteen-period run.

use super::{
    advance_material_period, identity_hex, install_reader_role, provision_observer_role,
    CampaignId, CollectingSink, DisposableTarget, DurableMaterialRuntime, MichiganContentPreset,
    MichiganDeliveryPreset, NoTls, ObserverEconomyReader, ObserverVisibility,
    OrderedPracticeActionBatch, Uuid,
};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_material_circuit::{GoodId, MaterialCircuitState, UnitId};
use babylon_persistence::{
    michigan_material::{MichiganMaterialCatalog, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES},
    production_observation::ProductionCapacityKind,
    production_observation::ProductionOutboundKind,
    production_observation::ProductionSnapshot,
    ProductionEvidenceDigest,
};
use babylon_tick::{
    material_replay::MaterialReplaySession,
    material_world::{
        decode_material_receipts, MaterialTickReceipts, MAX_MATERIAL_WORLD_REGISTER_BYTES,
    },
    replay_session::ReplayCommitDisposition,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

type Session = MaterialReplaySession<HypergraphStore>;
type Goods = BTreeMap<(GoodId, UnitId), u128>;
const SOURCE_FILES: [&str; 4] = [
    "defines.toml",
    "statewide-sources.json",
    "statewide-qualification.json.gz",
    "statewide-physical.json.gz",
];

struct SourceCopies(PathBuf);
impl SourceCopies {
    fn capture() -> Self {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../content/scenarios/michigan");
        let directory = loop {
            let id =
                super::NEXT_DISPOSABLE_TARGET.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "babylon-qualified-sources-{}-{id}",
                std::process::id()
            ));
            match std::fs::create_dir(&directory) {
                Ok(()) => break directory,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create owned source copy: {error}"),
            }
        };
        let copies = Self(directory);
        for name in SOURCE_FILES {
            let source = root.join(name);
            assert!(
                source
                    .metadata()
                    .expect("qualified canonical source must exist")
                    .len()
                    <= 67_108_864,
                "bounded source copy: {}",
                source.display()
            );
            std::fs::copy(source, copies.0.join(name)).unwrap();
        }
        copies
    }
    fn defines(&self) -> PathBuf {
        self.0.join("defines.toml")
    }
    fn change(&self) {
        std::fs::OpenOptions::new()
            .append(true)
            .open(self.defines())
            .unwrap()
            .write_all(b"\n# Changed only in the owned PostgreSQL qualification copy.\n")
            .unwrap();
    }
    fn remove(&self) {
        for name in SOURCE_FILES {
            std::fs::remove_file(self.0.join(name)).unwrap();
        }
    }
}
impl Drop for SourceCopies {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!(
                "owned qualified source cleanup {}: {error}",
                self.0.display()
            );
        }
    }
}

#[derive(Default, Debug)]
struct Measurements {
    create_with_reference: Duration,
    advance: Duration,
    reference: Duration,
    projection: Duration,
    resume: Duration,
    captured_bytes: usize,
    foundation_bytes: usize,
    maximum_register_bytes: usize,
    maximum_receipt_bytes: usize,
    maximum_family_rows: usize,
    dispatch_receipts: usize,
    arrival_receipts: usize,
    local_transfer_receipts: usize,
    final_handoff_receipts: usize,
}

#[test]
#[ignore = "requires actual qualified canonical siblings and the statewide_qualified PostgreSQL focus"]
fn actual_statewide_sources_survive_four_persisted_sixteen_period_campaigns() {
    for (index, preset) in [
        MichiganDeliveryPreset::StatewideBaseline,
        MichiganDeliveryPreset::StatewideFreightConstraint,
        MichiganDeliveryPreset::StatewidePackagingShortage,
        MichiganDeliveryPreset::StatewideBoth,
    ]
    .into_iter()
    .enumerate()
    {
        qualify_preset(preset, u128::try_from(index).unwrap());
    }
}

fn qualify_preset(delivery: MichiganDeliveryPreset, index: u128) {
    let began = Instant::now();
    let copies = SourceCopies::capture();
    let captured = MichiganMaterialCatalog::load_for_preset(&copies.defines(), delivery)
        .expect("New must admit the canonical qualified sources");
    let catalog = captured.with_preset(delivery).unwrap();
    let physical = catalog.physical_network().expect("actual road network");
    assert!(!physical.source.pbf_url.starts_with("synthetic://"));
    assert!(physical.source.pbf_url.starts_with("https://"));
    assert!(!physical.edges.is_empty());
    assert_eq!(physical.terminals.len(), 83);
    assert_eq!(catalog.sites().len(), 397);
    let preset = MichiganContentPreset::new_campaign(delivery);
    let foundation = preset.create_foundation(&captured).unwrap();
    let foundation_digest = foundation.digest();
    let twin = preset.create_foundation(&captured).unwrap();
    assert_eq!(foundation.canonical_bytes(), twin.canonical_bytes());
    let mut reference = twin.into_session().unwrap();
    let mut measured = Measurements {
        captured_bytes: catalog.defines_bytes().len(),
        foundation_bytes: foundation.canonical_bytes().len(),
        ..Measurements::default()
    };
    assert!(measured.captured_bytes < MAX_MICHIGAN_CAPTURED_CONTENT_BYTES);
    assert!(measured.foundation_bytes < MAX_MATERIAL_WORLD_REGISTER_BYTES);
    let mut target = DisposableTarget::create();
    let campaign = CampaignId::from_uuid(Uuid::from_u128(29_800 + index));
    let mut runtime = DurableMaterialRuntime::create(&target.writer, campaign, foundation).unwrap();
    install_reader_role(&target.writer).unwrap();
    provision_observer_role(&target.writer).unwrap();
    let config = target.login("babylon_observer", "actualstatewide");
    let observer =
        ObserverEconomyReader::connect(&config, ObserverVisibility::FullObserver).unwrap();
    let mut sql = target.writer.connect(NoTls).unwrap();
    measured.create_with_reference = began.elapsed();
    let mut held = Vec::new();
    observe(
        &observer,
        campaign,
        &runtime,
        &catalog,
        None,
        &mut measured,
        &mut held,
    );
    for period in 1..=16 {
        let opening_capacities = reference
            .material()
            .state()
            .corridor_capacities
            .iter()
            .filter(|row| row.period == period)
            .map(|row| {
                (
                    identity_hex(row.corridor_id.as_bytes()),
                    row.available_grams,
                )
            })
            .collect();
        let receipts = advance_pair(
            &mut runtime,
            &mut reference,
            &mut sql,
            campaign,
            &mut measured,
        );
        observe(
            &observer,
            campaign,
            &runtime,
            &catalog,
            Some((&receipts, &opening_capacities)),
            &mut measured,
            &mut held,
        );
        if period == 1 {
            copies.change();
        }
        if period == 2 {
            copies.remove();
        }
        if [1, 2, 16].contains(&period) {
            assert!(MichiganMaterialCatalog::load_for_preset(&copies.defines(), delivery).is_err());
            reopen_runtime(
                &mut runtime,
                &target,
                campaign,
                foundation_digest,
                &mut measured,
            );
        }
        eprintln!("actual statewide {}: period {period}/16", preset.id());
    }
    assert_history_and_preview(&observer, &mut target, campaign, &held, &mut measured);
    assert_qualified_totals(&mut sql, campaign, preset.id(), &measured);
}

fn assert_qualified_totals(
    sql: &mut postgres::Client,
    campaign: CampaignId,
    preset: &str,
    measured: &Measurements,
) {
    let committed: i64 = sql
        .query_one(
            "SELECT count(*) FROM babylon_state.material_tick_v3 WHERE campaign_id=$1::uuid",
            &[campaign.as_uuid()],
        )
        .unwrap()
        .get(0);
    assert_eq!(committed, 16);
    assert!(measured.dispatch_receipts > 0 && measured.arrival_receipts > 0);
    assert!(measured.local_transfer_receipts > 0 && measured.final_handoff_receipts > 0);
    assert!(measured.maximum_family_rows < 65_536);
    assert!(measured.maximum_register_bytes < MAX_MATERIAL_WORLD_REGISTER_BYTES);
    assert!(measured.maximum_receipt_bytes < MAX_MATERIAL_WORLD_REGISTER_BYTES);
    eprintln!("actual statewide PostgreSQL {preset}: {measured:?}");
}

fn reopen_runtime(
    runtime: &mut DurableMaterialRuntime,
    target: &DisposableTarget,
    campaign: CampaignId,
    foundation_digest: [u8; 32],
    measured: &mut Measurements,
) {
    let started = Instant::now();
    let reopened =
        DurableMaterialRuntime::open(&target.writer, campaign, foundation_digest).unwrap();
    measured.resume += started.elapsed();
    assert_eq!(runtime.tail(), reopened.tail());
    assert_eq!(
        runtime.session().material().canonical_bytes(),
        reopened.session().material().canonical_bytes()
    );
    assert_eq!(
        runtime.session().current_world_hash().unwrap(),
        reopened.session().current_world_hash().unwrap()
    );
    *runtime = reopened;
}

fn advance_pair(
    runtime: &mut DurableMaterialRuntime,
    reference: &mut Session,
    sql: &mut postgres::Client,
    campaign: CampaignId,
    measured: &mut Measurements,
) -> MaterialTickReceipts {
    let began = Instant::now();
    let actions = OrderedPracticeActionBatch::empty(
        reference.graph_session().session_identity().clone(),
        reference.completed_tick() + 1,
    )
    .unwrap();
    let prepared = reference.prepare_advance(&actions).unwrap();
    let bytes = prepared.material().receipt_bytes();
    let receipts = decode_material_receipts(bytes).unwrap();
    assert_goods_conserved(
        reference.material().state(),
        prepared.material().register().state(),
        &receipts,
    );
    measured.maximum_receipt_bytes = measured.maximum_receipt_bytes.max(bytes.len());
    measured.dispatch_receipts += receipts.dispatches.len();
    measured.arrival_receipts += receipts.arrivals.len();
    measured.local_transfer_receipts += receipts.local_transfers.len();
    measured.final_handoff_receipts += receipts.local_fulfillments.len();
    measured.reference += began.elapsed();
    let began = Instant::now();
    advance_material_period(runtime);
    measured.advance += began.elapsed();
    let row = sql.query_one("SELECT register_bytes,receipt_bytes FROM babylon_state.material_tick_v3 WHERE campaign_id=$1::uuid AND resolve_tick=$2", &[campaign.as_uuid(), &i64::try_from(receipts.resolve_tick).unwrap()]).unwrap();
    assert_eq!(
        row.get::<_, Vec<u8>>(0),
        prepared.material().register().canonical_bytes()
    );
    assert_eq!(row.get::<_, Vec<u8>>(1), bytes);
    reference
        .commit_prepared_and_publish(&mut CollectingSink::default(), prepared, |_| {
            Ok::<_, ()>(ReplayCommitDisposition::Committed)
        })
        .unwrap();
    assert_eq!(
        runtime.session().material().canonical_bytes(),
        reference.material().canonical_bytes()
    );
    assert_eq!(
        runtime.session().current_world_hash().unwrap(),
        reference.current_world_hash().unwrap()
    );
    receipts
}

fn observe(
    observer: &ObserverEconomyReader,
    campaign: CampaignId,
    runtime: &DurableMaterialRuntime,
    catalog: &MichiganMaterialCatalog,
    completed: Option<(&MaterialTickReceipts, &BTreeMap<String, u64>)>,
    measured: &mut Measurements,
    held: &mut Vec<(u64, ProductionEvidenceDigest)>,
) {
    let began = Instant::now();
    let snapshot = observer
        .snapshot(campaign, runtime.session().completed_tick())
        .unwrap();
    measured.projection += began.elapsed();
    assert_eq!(snapshot.resolve_tick, runtime.session().completed_tick());
    if snapshot.resolve_tick == 0 {
        assert!(snapshot.nominal_world_hash.is_none());
    } else {
        assert_eq!(
            snapshot.nominal_world_hash.as_deref(),
            Some(identity_hex(runtime.session().current_world_hash().unwrap()).as_str())
        );
    }
    let evidence = snapshot.production_evidence_digest().unwrap().unwrap();
    if [0, 1, 2, 3, 8, 16].contains(&snapshot.resolve_tick) {
        held.push((snapshot.resolve_tick, evidence));
    }
    let rows = snapshot.production.as_ref().unwrap();
    assert_eq!(
        rows.content_authority_sha256,
        identity_hex(catalog.defines_hash())
    );
    assert_eq!(rows.sites.len(), catalog.sites().len());
    assert_eq!(rows.staffing_accounts.len(), catalog.staffing().pools.len());
    assert_eq!(
        rows.merchant_handling_accounts.len(),
        catalog.merchants().len()
    );
    assert_eq!(
        rows.physical_edges.len(),
        catalog.physical_network().unwrap().edges.len()
    );
    assert_eq!(
        rows.road_source.as_ref().unwrap().pbf_sha256,
        catalog.physical_network().unwrap().source.pbf_sha256
    );
    assert_workforce(rows, snapshot.resolve_tick);
    assert_stock(rows, runtime.session().material().state());
    assert_final_demand(
        rows,
        runtime.session().material().state(),
        completed.map(|r| r.0),
    );
    assert_capacity(rows, snapshot.resolve_tick, completed);
    assert_merchant(rows, completed.map(|r| r.0));
    assert_qualified_witness(rows, catalog, snapshot.resolve_tick);
    let register = runtime.session().material();
    measured.maximum_register_bytes = measured
        .maximum_register_bytes
        .max(register.canonical_bytes().len());
    measured.maximum_family_rows = measured
        .maximum_family_rows
        .max(maximum_rows(register.state()));
}

fn witnessed_output(
    rows: &ProductionSnapshot,
    catalog: &MichiganMaterialCatalog,
    key: &str,
    unit: &str,
) -> u64 {
    let process = catalog.processes().iter().find(|p| p.key == key).unwrap();
    let id = identity_hex(process.id().as_bytes());
    let observed = rows
        .sites
        .iter()
        .flat_map(|site| &site.processes)
        .find(|process| process.id == id)
        .unwrap();
    assert_eq!(observed.output_unit, unit);
    observed
        .produced_batches
        .unwrap()
        .checked_mul(observed.output_per_batch)
        .unwrap()
}

fn assert_qualified_witness(
    rows: &ProductionSnapshot,
    catalog: &MichiganMaterialCatalog,
    period: u64,
) {
    let freight = matches!(
        catalog.preset(),
        MichiganDeliveryPreset::StatewideFreightConstraint | MichiganDeliveryPreset::StatewideBoth
    );
    let packaging = matches!(
        catalog.preset(),
        MichiganDeliveryPreset::StatewidePackagingShortage | MichiganDeliveryPreset::StatewideBoth
    );
    if period == 1 {
        let bridge = catalog
            .corridors()
            .iter()
            .find(|c| c.key == "mackinac-bridge-freight")
            .unwrap();
        let bridge_id = identity_hex(bridge.id().as_bytes());
        let account = rows
            .freight_capacity_accounts
            .iter()
            .find(|c| c.corridor_id == bridge_id)
            .unwrap();
        assert_eq!(account.route_ids.len(), 16);
        let reservations = &account.completed.as_ref().unwrap().reservations;
        assert_eq!(reservations.len(), 1);
        assert_eq!(
            reservations[0].opening_available_grams,
            if freight { 1_000_000 } else { 100_000_000 }
        );
        assert_eq!(
            reservations[0].newly_reserved_grams,
            if freight { 709_000 } else { 2_207_000 }
        );
        assert_eq!(
            witnessed_output(rows, catalog, "26097-31-33-prepared_food", "kg"),
            if packaging { 800 } else { 1_600 }
        );
    }
    if period == 2 {
        let site = rows
            .sites
            .iter()
            .find(|s| s.county_geoid == "26033" && s.sector_code == "31-33")
            .unwrap();
        let workforce = rows
            .staffing_accounts
            .iter()
            .find(|a| a.site_id == site.id)
            .unwrap();
        assert_eq!(
            (workforce.employed, workforce.reserve),
            if freight { (1, 11) } else { (3, 9) }
        );
    }
    if period == 3 {
        let food = match (freight, packaging) {
            (true, _) => 100,
            (false, true) => 400,
            (false, false) => 300,
        };
        assert_eq!(
            witnessed_output(rows, catalog, "26097-31-33-prepared_food", "kg"),
            food
        );
        assert_eq!(
            witnessed_output(rows, catalog, "26033-31-33-household_wares", "item"),
            if freight { 1 } else { 8 }
        );
    }
}

fn assert_workforce(rows: &ProductionSnapshot, period: u64) {
    let mut pools = BTreeSet::new();
    let mut labor = BTreeSet::new();
    for account in &rows.staffing_accounts {
        assert!(pools.insert(&account.pool_id));
        assert_eq!(
            u128::from(account.employed) + u128::from(account.reserve),
            u128::from(account.labor_force)
        );
        if let Some(done) = &account.completed {
            assert_eq!(done.period, period);
            assert_eq!(
                u128::from(done.opening_employed) + u128::from(done.hires),
                u128::from(account.employed) + u128::from(done.separations)
            );
            assert_eq!(
                u128::from(done.opening_reserve) + u128::from(done.separations),
                u128::from(account.reserve) + u128::from(done.hires)
            );
        } else {
            assert_eq!(period, 0);
        }
    }
    for account in &rows.labor_accounts {
        assert!(labor.insert((&account.site_id, &account.unit_id)));
        if let Some(done) = &account.completed {
            assert_eq!(
                u128::from(done.used) + u128::from(done.unused),
                u128::from(done.opening)
            );
            assert!(done.handling_used <= done.used);
        } else {
            assert_eq!(period, 0);
        }
    }
    assert_eq!(labor.len(), pools.len());
}

fn assert_stock(rows: &ProductionSnapshot, state: &MaterialCircuitState) {
    if let Some(balance) = &rows.material_balance {
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
                    + u128::from(row.closing)
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
    } else {
        assert_eq!(state.period, 1);
    }
    for route in &rows.routes {
        let transit = rows
            .freight
            .iter()
            .filter(|lot| lot.route_id == route.id)
            .map(|lot| u128::from(lot.quantity))
            .sum::<u128>();
        assert_eq!(
            u128::from(route.shipped),
            u128::from(route.delivered) + u128::from(route.lost) + transit
        );
        assert_eq!(route.realized, route.delivered);
        assert!(route.shipped <= route.ordered);
    }
    assert_eq!(rows.freight.len(), state.freight.len());
    for lot in &rows.freight {
        assert!(lot.arrival_period > lot.dispatch_period);
        assert_eq!(
            u128::from(lot.mass_grams),
            u128::from(lot.quantity) * u128::from(lot.grams_per_unit)
        );
    }
}

fn assert_final_demand(
    rows: &ProductionSnapshot,
    state: &MaterialCircuitState,
    receipts: Option<&MaterialTickReceipts>,
) {
    let mut orders = BTreeSet::new();
    for account in &rows.final_demand_accounts {
        assert_eq!(
            u128::from(account.fulfilled) + u128::from(account.outstanding),
            u128::from(account.ordered)
        );
        let mut newly = 0_u128;
        for order in &account.orders {
            assert!(orders.insert(&order.order_id));
            let actual = state
                .final_demand_orders
                .iter()
                .find(|row| identity_hex(row.order_id.as_bytes()) == order.order_id)
                .unwrap();
            assert_eq!(
                (order.ordered, order.fulfilled),
                (actual.ordered, actual.fulfilled)
            );
            if let Some(receipts) = receipts {
                newly += receipts
                    .local_fulfillments
                    .iter()
                    .filter(|row| row.order_id == actual.order_id)
                    .map(|row| u128::from(row.quantity))
                    .sum::<u128>();
            }
        }
        if let Some(done) = &account.completed {
            assert_eq!(u128::from(done.newly_fulfilled), newly);
            assert_eq!(
                u128::from(done.opening_fulfilled) + newly,
                u128::from(account.fulfilled)
            );
            assert_eq!(done.closing_fulfilled, account.fulfilled);
        } else {
            assert!(receipts.is_none());
        }
        let unsold = rows
            .sites
            .iter()
            .filter(|site| account.retailer_site_ids.contains(&site.id))
            .flat_map(|site| &site.inventory)
            .filter(|row| row.good_id == account.good_id && row.unit_id == account.unit_id)
            .map(|row| u128::from(row.quantity))
            .sum::<u128>();
        assert_eq!(u128::from(account.retail_stock_on_hand), unsold);
    }
    assert_eq!(orders.len(), state.final_demand_orders.len());
}

fn outbound_quantity(
    receipts: &MaterialTickReceipts,
    kind: ProductionOutboundKind,
    id: &str,
) -> u128 {
    match kind {
        ProductionOutboundKind::Delivery => receipts
            .dispatches
            .iter()
            .map(|r| (r.order_id, r.quantity))
            .chain(
                receipts
                    .local_transfers
                    .iter()
                    .map(|r| (r.order_id, r.quantity)),
            )
            .filter(|(order, _)| identity_hex(order.as_bytes()) == id)
            .map(|(_, quantity)| u128::from(quantity))
            .sum(),
        ProductionOutboundKind::LocalFinalDemand => receipts
            .local_fulfillments
            .iter()
            .filter(|r| identity_hex(r.order_id.as_bytes()) == id)
            .map(|r| u128::from(r.quantity))
            .sum(),
    }
}

fn assert_capacity(
    rows: &ProductionSnapshot,
    period: u64,
    completed: Option<(&MaterialTickReceipts, &BTreeMap<String, u64>)>,
) {
    let mut principals = BTreeSet::new();
    for account in &rows.freight_capacity_accounts {
        assert!(principals.insert(&account.corridor_id));
        let Some(done) = &account.completed else {
            assert!(completed.is_none());
            continue;
        };
        let (receipts, opening) = completed.unwrap();
        assert_eq!(done.period, period);
        let mut periods = BTreeSet::new();
        for reservation in &done.reservations {
            assert!(periods.insert(reservation.reservation_period));
            assert_eq!(
                reservation.reservation_period, period,
                "current road journeys reserve one departure period"
            );
            assert_eq!(
                reservation.opening_available_grams,
                opening[&account.corridor_id]
            );
            assert_eq!(
                u128::from(reservation.newly_reserved_grams)
                    + u128::from(reservation.remaining_available_grams),
                u128::from(reservation.opening_available_grams)
            );
            let mut orders = BTreeSet::new();
            let mut reserved = 0_u128;
            for order in &reservation.orders {
                assert!(orders.insert((order.kind, &order.order_id)));
                let actual = outbound_quantity(receipts, order.kind, &order.order_id);
                assert_eq!(u128::from(order.dispatched), actual);
                assert_eq!(
                    u128::from(order.reserved_grams),
                    actual * u128::from(order.grams_per_unit)
                );
                assert_eq!(
                    u128::from(order.requested),
                    actual + u128::from(order.remaining_unshipped)
                );
                if account.kind == ProductionCapacityKind::Transport {
                    let route = rows
                        .routes
                        .iter()
                        .find(|r| Some(&r.id) == order.route_id.as_ref())
                        .unwrap();
                    assert!(route
                        .stages
                        .iter()
                        .any(|stage| stage.capacity_ids.contains(&account.corridor_id)));
                }
                reserved += u128::from(order.reserved_grams);
            }
            assert_eq!(reserved, u128::from(reservation.newly_reserved_grams));
        }
    }
}

fn assert_merchant(rows: &ProductionSnapshot, receipts: Option<&MaterialTickReceipts>) {
    let mut sites = BTreeSet::new();
    for account in &rows.merchant_handling_accounts {
        assert!(sites.insert(&account.site_id));
        let Some(done) = &account.completed else {
            assert!(receipts.is_none());
            continue;
        };
        let receipts = receipts.unwrap();
        let matching: Vec<_> = receipts
            .handling
            .iter()
            .filter(|row| identity_hex(row.site_id.as_bytes()) == account.site_id)
            .collect();
        assert_eq!(
            u128::from(done.needed_hours),
            matching
                .iter()
                .map(|row| u128::from(row.needed_hours))
                .sum::<u128>()
        );
        assert_eq!(
            u128::from(done.used_hours),
            matching
                .iter()
                .map(|row| u128::from(row.used_hours))
                .sum::<u128>()
        );
        let labor = rows
            .labor_accounts
            .iter()
            .find(|row| row.site_id == account.site_id && row.unit_id == account.labor_unit_id)
            .unwrap()
            .completed
            .as_ref()
            .unwrap();
        assert_eq!(labor.handling_used, done.used_hours);
        assert_eq!(labor.handling_needed, done.needed_hours);
        for order in &done.orders {
            assert_eq!(
                u128::from(order.handled_quantity),
                outbound_quantity(receipts, order.kind, &order.order_id)
            );
            assert!(order.handled_quantity <= order.feasible_quantity);
        }
    }
}

fn inventory_and_transit(state: &MaterialCircuitState) -> Goods {
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
    opening: &MaterialCircuitState,
    closing: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
) {
    let mut available = inventory_and_transit(opening);
    let mut accounted = inventory_and_transit(closing);
    for receipt in &receipts.production {
        let output = opening
            .process_outputs
            .iter()
            .find(|row| row.process_id == receipt.process_id)
            .unwrap();
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
    available.retain(|_, quantity| *quantity > 0);
    accounted.retain(|_, quantity| *quantity > 0);
    assert_eq!(
        available, accounted,
        "native stock + transit + output = closing + inputs + loss + receipted final handoff"
    );
}

fn assert_history_and_preview(
    observer: &ObserverEconomyReader,
    target: &mut DisposableTarget,
    campaign: CampaignId,
    held: &[(u64, ProductionEvidenceDigest)],
    measured: &mut Measurements,
) {
    let began = Instant::now();
    for (period, digest) in held {
        assert_eq!(
            observer
                .snapshot(campaign, *period)
                .unwrap()
                .production_evidence_digest()
                .unwrap(),
            Some(*digest)
        );
    }
    measured.projection += began.elapsed();
    assert_eq!(observer.campaigns().unwrap()[0].durable_tick, 16);
    let preview_config = target.login("babylon_reader", "actualpreview");
    let preview =
        ObserverEconomyReader::connect(&preview_config, ObserverVisibility::KnownPreview).unwrap();
    let snapshot = preview.snapshot(campaign, 16).unwrap();
    assert!(snapshot.production.is_none());
    assert!(snapshot.production_evidence_digest().unwrap().is_none());
}

fn maximum_rows(state: &MaterialCircuitState) -> usize {
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
