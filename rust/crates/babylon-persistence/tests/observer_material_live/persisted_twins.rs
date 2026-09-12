//! PER-325: independently committed delivery twins and restart controls.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    advance_material_period, assert_material_accounts, identity_hex, install_reader_role,
    provision_observer_role, CampaignId, DisposableTarget, DurableMaterialRuntime,
    MichiganContentPreset, MichiganDeliveryPreset, NoTls, ObserverEconomyReader,
    ObserverVisibility, Uuid,
};
use babylon_kernel::content_digest::sha256_of;
use babylon_persistence::{
    observer_reader::ObserverEconomySnapshot, production_observation::ProductionSnapshot,
};
use babylon_tick::material_world::{decode_material_receipts, MaterialTickReceipts};
use postgres::Client;

// Both downstream onset periods and their following continuation are inside this
// live proof. Existing Michigan replay tests cover the complete 16-period horizon.
const PROOF_PERIODS: u64 = 8;

struct RunPair {
    preset: MichiganDeliveryPreset,
    uninterrupted: DurableMaterialRuntime,
    restarted: DurableMaterialRuntime,
    foundation_digest: [u8; 32],
    history: Vec<[ObserverEconomySnapshot; 2]>,
    restart_periods: Vec<u64>,
}

impl RunPair {
    fn create(target: &DisposableTarget, preset: MichiganDeliveryPreset, id: u128) -> Self {
        let foundation = MichiganContentPreset::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let foundation_digest = foundation.digest();
        let uninterrupted = DurableMaterialRuntime::create(
            &target.writer,
            CampaignId::from_uuid(Uuid::from_u128(id)),
            foundation,
        )
        .unwrap();
        let restarted = DurableMaterialRuntime::create(
            &target.writer,
            CampaignId::from_uuid(Uuid::from_u128(id + 1)),
            MichiganContentPreset::new_campaign(preset)
                .create_foundation(&crate::test_support::catalog())
                .unwrap(),
        )
        .unwrap();
        Self {
            preset,
            uninterrupted,
            restarted,
            foundation_digest,
            history: Vec::new(),
            restart_periods: Vec::new(),
        }
    }

    fn snapshots(&self, observer: &ObserverEconomyReader) -> [ObserverEconomySnapshot; 2] {
        [&self.uninterrupted, &self.restarted].map(|runtime| {
            observer
                .snapshot(runtime.campaign_id(), runtime.session().completed_tick())
                .unwrap()
        })
    }

    fn advance(
        &mut self,
        target: &DisposableTarget,
        observer: &ObserverEconomyReader,
        connection: &mut Client,
    ) {
        advance_material_period(&mut self.uninterrupted);
        advance_material_period(&mut self.restarted);
        self.assert_exact_continuation();
        let current = self.snapshots(observer);
        assert_eq!(current[0].production, current[1].production);
        assert_eq!(current[0].nominal_world_hash, current[1].nominal_world_hash);
        assert_eq!(current[0].tick_content_hash, current[1].tick_content_hash);
        // The envelope, unlike the content identity, includes the campaign UUID.
        assert_ne!(current[0].campaign_id, current[1].campaign_id);
        assert_ne!(current[0].envelope_digest, current[1].envelope_digest);
        let receipts = authenticated_receipts(connection, &self.uninterrupted, &current[0]);
        assert_eq!(
            receipts,
            authenticated_receipts(connection, &self.restarted, &current[1])
        );
        for (prior, next) in self.history.last().unwrap().iter().zip(&current) {
            assert_reconciled(prior, next, &receipts);
        }
        let tick = current[0].resolve_tick;
        if is_restart_boundary(self.preset, tick, &current[1], &receipts) {
            self.restarted = DurableMaterialRuntime::open(
                &target.writer,
                self.restarted.campaign_id(),
                self.foundation_digest,
            )
            .unwrap();
            self.assert_exact_continuation();
            assert_eq!(self.snapshots(observer), current);
            self.restart_periods.push(tick);
        }
        self.history.push(current);
    }

    fn assert_exact_continuation(&self) {
        assert_eq!(self.uninterrupted.tail(), self.restarted.tail());
        assert_eq!(
            self.uninterrupted.session().material().canonical_bytes(),
            self.restarted.session().material().canonical_bytes()
        );
        assert_eq!(
            self.uninterrupted.session().current_world_hash().unwrap(),
            self.restarted.session().current_world_hash().unwrap()
        );
    }

    fn assert_held_history(&self, observer: &ObserverEconomyReader) {
        for tick in [0, 1, 2, 4, 5, 7, 8] {
            for (runtime, held) in [&self.uninterrupted, &self.restarted]
                .into_iter()
                .zip(&self.history[tick])
            {
                assert_eq!(
                    observer
                        .snapshot(runtime.campaign_id(), held.resolve_tick)
                        .unwrap(),
                    *held
                );
            }
        }
    }
}

fn authenticated_receipts(
    connection: &mut Client,
    runtime: &DurableMaterialRuntime,
    snapshot: &ObserverEconomySnapshot,
) -> MaterialTickReceipts {
    let tail = runtime.tail().unwrap();
    let tick = i64::try_from(tail.resolve_tick()).unwrap();
    let bytes: Vec<u8> = connection
        .query_one(
            "SELECT receipt_bytes FROM public.v_observer_material_state_v1 \
             WHERE campaign_id=$1::uuid AND resolve_tick=$2",
            &[runtime.campaign_id().as_uuid(), &tick],
        )
        .unwrap()
        .get(0);
    assert_eq!(sha256_of(&bytes), tail.receipt_digest());
    assert_eq!(snapshot.resolve_tick, tail.resolve_tick());
    assert_eq!(
        snapshot.campaign_id,
        runtime.campaign_id().as_uuid().to_string()
    );
    assert_eq!(
        snapshot.foundation_digest,
        identity_hex(tail.foundation_digest())
    );
    assert_eq!(
        snapshot.tick_content_hash,
        Some(identity_hex(*tail.tick_content_hash().as_bytes()))
    );
    assert_eq!(
        snapshot.nominal_world_hash,
        Some(identity_hex(tail.result_world_hash()))
    );
    assert!(snapshot.envelope_digest.is_some());
    assert!(snapshot.production_evidence_digest().unwrap().is_some());
    let receipts = decode_material_receipts(&bytes).unwrap();
    assert_eq!(receipts.resolve_tick, snapshot.resolve_tick);
    receipts
}

fn production(snapshot: &ObserverEconomySnapshot) -> &ProductionSnapshot {
    snapshot.production.as_ref().unwrap()
}

fn stock(rows: &ProductionSnapshot, site: &str, good: &str, unit: &str) -> u64 {
    rows.sites
        .iter()
        .find(|row| row.id == site)
        .unwrap()
        .inventory
        .iter()
        .find(|row| row.good_id == good && row.unit_id == unit)
        .map_or(0, |row| row.quantity)
}

fn assert_reconciled(
    prior: &ObserverEconomySnapshot,
    current: &ObserverEconomySnapshot,
    receipts: &MaterialTickReceipts,
) {
    assert_material_accounts(current);
    assert_eq!(prior.resolve_tick + 1, current.resolve_tick);
    let before = production(prior);
    let after = production(current);
    assert_inventory(before, after, receipts);
    assert_labor(before, after, current.resolve_tick);
    assert_freight(before, after, receipts);
}

fn assert_inventory(
    before: &ProductionSnapshot,
    after: &ProductionSnapshot,
    receipts: &MaterialTickReceipts,
) {
    let catalog = crate::test_support::catalog();
    let stock_key = |site_key: &str, good_key: &str| {
        let good = catalog.good(good_key).unwrap();
        (
            identity_hex(catalog.site(site_key).unwrap().id().as_bytes()),
            identity_hex(good.id().as_bytes()),
            identity_hex(good.unit_id().as_bytes()),
        )
    };
    let mut dispatches = BTreeMap::new();
    for receipt in &receipts.dispatches {
        let route = catalog
            .routes()
            .iter()
            .find(|route| route.id() == receipt.route_id)
            .unwrap();
        assert_eq!(receipt.order_id, route.order_id());
        *dispatches
            .entry(stock_key(&route.supplier_site_key, &route.good_key))
            .or_insert(0_u128) += u128::from(receipt.quantity);
    }
    let mut arrivals = BTreeMap::new();
    for receipt in &receipts.arrivals {
        let route = catalog
            .routes()
            .iter()
            .find(|route| route.order_id() == receipt.order_id)
            .unwrap();
        *arrivals
            .entry(stock_key(&route.buyer_site_key, &route.good_key))
            .or_insert(0_u128) += u128::from(receipt.quantity);
    }
    for row in &after.material_balance.as_ref().unwrap().rows {
        let key = (
            row.site_id.clone(),
            row.good_id.clone(),
            row.unit_id.clone(),
        );
        assert_eq!(
            u128::from(row.dispatched),
            dispatches.remove(&key).unwrap_or(0)
        );
        assert_eq!(u128::from(row.arrivals), arrivals.remove(&key).unwrap_or(0));
        assert_eq!(
            row.opening,
            stock(before, &row.site_id, &row.good_id, &row.unit_id)
        );
        let site = after
            .sites
            .iter()
            .find(|site| site.id == row.site_id)
            .unwrap();
        let produced: u128 = site
            .processes
            .iter()
            .filter(|process| {
                process.output_good_id == row.good_id && process.output_unit_id == row.unit_id
            })
            .map(|process| {
                u128::from(process.produced_batches.unwrap()) * u128::from(process.output_per_batch)
            })
            .sum();
        let consumed: u128 = site
            .processes
            .iter()
            .flat_map(|process| process.inputs.iter().map(move |input| (process, input)))
            .filter(|(_, input)| input.good_id == row.good_id && input.unit_id == row.unit_id)
            .map(|(process, input)| {
                u128::from(process.produced_batches.unwrap()) * u128::from(input.quantity_per_batch)
            })
            .sum();
        assert_eq!(u128::from(row.produced), produced);
        assert_eq!(u128::from(row.consumed), consumed);
    }
    assert!(
        dispatches.is_empty(),
        "every dispatch debits its source stock"
    );
    assert!(
        arrivals.is_empty(),
        "every arrival credits its destination stock"
    );
    for site in &after.sites {
        for process in &site.processes {
            let receipt = receipts.production.iter().find(|row| {
                identity_hex(row.site_id.as_bytes()) == site.id
                    && identity_hex(row.process_id.as_bytes()) == process.id
            });
            assert_eq!(
                process.produced_batches,
                Some(receipt.map_or(0, |row| row.produced_batches))
            );
            assert_eq!(
                process.planned_batches,
                Some(receipt.map_or(0, |row| row.planned_batches))
            );
        }
    }
}

fn assert_labor(before: &ProductionSnapshot, after: &ProductionSnapshot, tick: u64) {
    for labor in &after.labor_accounts {
        let completed = labor.completed.as_ref().unwrap();
        let previous = before
            .labor_accounts
            .iter()
            .find(|row| row.site_id == labor.site_id && row.unit_id == labor.unit_id)
            .unwrap();
        let site = after
            .sites
            .iter()
            .find(|site| site.id == labor.site_id)
            .unwrap();
        let production_hours: u128 = site
            .processes
            .iter()
            .map(|process| {
                assert_eq!(process.labor.len(), 1);
                u128::from(process.produced_batches.unwrap())
                    * u128::from(process.labor[0].quantity_per_batch)
            })
            .sum();
        assert_eq!(completed.period, tick);
        assert_eq!(completed.opening, previous.next_opening_available);
        assert_eq!(
            u128::from(completed.used) + u128::from(completed.unused),
            u128::from(completed.opening)
        );
        assert!(completed.used <= completed.planned && completed.planned <= completed.opening);
        assert_eq!(
            u128::from(completed.used),
            production_hours + u128::from(completed.handling_used)
        );
    }
    for pool in &after.staffing_accounts {
        assert_eq!(pool.hours_per_person, 160);
        assert_eq!(pool.employed + pool.reserve, pool.labor_force);
        assert_eq!(pool.next_opening_hours, pool.employed * 160);
    }
}

fn assert_freight(
    before: &ProductionSnapshot,
    after: &ProductionSnapshot,
    receipts: &MaterialTickReceipts,
) {
    let mut lots = BTreeSet::new();
    assert!(after.freight.iter().all(|lot| lots.insert(&lot.id)));
    for route in &after.routes {
        let previous = before.routes.iter().find(|row| row.id == route.id).unwrap();
        let catalog = crate::test_support::catalog();
        let source = catalog
            .routes()
            .iter()
            .find(|row| identity_hex(row.id().as_bytes()) == route.id)
            .unwrap();
        let dispatched: u128 = receipts
            .dispatches
            .iter()
            .filter(|row| row.route_id == source.id())
            .map(|row| u128::from(row.quantity))
            .sum();
        let delivered: u128 = receipts
            .deliveries
            .iter()
            .filter(|row| row.order_id == source.order_id())
            .map(|row| u128::from(row.quantity))
            .sum();
        assert_eq!(
            u128::from(route.shipped),
            u128::from(previous.shipped) + dispatched
        );
        assert_eq!(
            u128::from(route.delivered),
            u128::from(previous.delivered) + delivered
        );
        assert_eq!(route.lost, 0);
        let in_transit: u128 = after
            .freight
            .iter()
            .filter(|lot| lot.route_id == route.id)
            .map(|lot| u128::from(lot.quantity))
            .sum();
        assert!(route.shipped <= route.ordered);
        assert_eq!(
            u128::from(route.shipped),
            u128::from(route.delivered) + u128::from(route.lost) + in_transit
        );
        assert_eq!(route.realized, route.delivered);
    }
}

fn is_restart_boundary(
    preset: MichiganDeliveryPreset,
    tick: u64,
    snapshot: &ObserverEconomySnapshot,
    receipts: &MaterialTickReceipts,
) -> bool {
    let catalog = crate::test_support::catalog();
    let sheet = catalog
        .routes()
        .iter()
        .find(|route| route.key == "sheet-transfer")
        .unwrap();
    let arrival = if preset == MichiganDeliveryPreset::Delayed {
        4
    } else {
        2
    };
    if tick == 1 {
        assert!(receipts
            .dispatches
            .iter()
            .any(|row| row.route_id == sheet.id() && row.quantity > 0));
        assert!(production(snapshot)
            .freight
            .iter()
            .any(|lot| lot.route_id == identity_hex(sheet.id().as_bytes())
                && lot.dispatch_period == tick
                && lot.arrival_period == arrival));
    } else if tick == arrival {
        assert!(receipts
            .arrivals
            .iter()
            .any(|row| row.order_id == sheet.order_id() && row.quantity > 0));
    } else if tick == 2 {
        assert_eq!(preset, MichiganDeliveryPreset::Delayed);
        assert!(production(snapshot)
            .freight
            .iter()
            .any(|lot| lot.route_id == identity_hex(sheet.id().as_bytes())
                && lot.dispatch_period == 1
                && lot.arrival_period == 4));
    } else {
        return false;
    }
    true
}

fn assert_food_disconnected(standard: &ProductionSnapshot, delayed: &ProductionSnapshot) {
    let food: BTreeSet<_> = standard
        .sites
        .iter()
        .filter(|site| site.industry_code == "311")
        .map(|site| site.id.as_str())
        .collect();
    assert_eq!(food.len(), 2);
    assert_eq!(
        standard
            .sites
            .iter()
            .filter(|site| food.contains(site.id.as_str()))
            .collect::<Vec<_>>(),
        delayed
            .sites
            .iter()
            .filter(|site| food.contains(site.id.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        standard
            .labor_accounts
            .iter()
            .filter(|row| food.contains(row.site_id.as_str()))
            .collect::<Vec<_>>(),
        delayed
            .labor_accounts
            .iter()
            .filter(|row| food.contains(row.site_id.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        standard
            .staffing_accounts
            .iter()
            .filter(|row| food.contains(row.site_id.as_str()))
            .collect::<Vec<_>>(),
        delayed
            .staffing_accounts
            .iter()
            .filter(|row| food.contains(row.site_id.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        standard
            .freight
            .iter()
            .filter(|lot| food.contains(lot.source_site_id.as_str()))
            .collect::<Vec<_>>(),
        delayed
            .freight
            .iter()
            .filter(|lot| food.contains(lot.source_site_id.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        standard
            .routes
            .iter()
            .filter(|route| food.contains(route.supplier_site_id.as_str()))
            .collect::<Vec<_>>(),
        delayed
            .routes
            .iter()
            .filter(|route| food.contains(route.supplier_site_id.as_str()))
            .collect::<Vec<_>>()
    );
}

fn subassembly_stock(snapshot: &ObserverEconomySnapshot) -> u64 {
    let catalog = crate::test_support::catalog();
    stock(
        production(snapshot),
        &identity_hex(catalog.site("wayne-vehicle-parts").unwrap().id().as_bytes()),
        &identity_hex(catalog.good("subassembly").unwrap().id().as_bytes()),
        &identity_hex(catalog.good("subassembly").unwrap().unit_id().as_bytes()),
    )
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL harness; serial persisted twin proof"]
fn persisted_delivery_twins_reconcile_and_restart_at_dispatch_transit_and_arrival() {
    let mut target = DisposableTarget::create();
    let mut standard = RunPair::create(&target, MichiganDeliveryPreset::Standard, 325_001);
    let mut delayed = RunPair::create(&target, MichiganDeliveryPreset::Delayed, 325_003);
    let initial = standard.uninterrupted.session().material().state();
    let mut normalized = delayed.uninterrupted.session().material().state().clone();
    let catalog = crate::test_support::catalog();
    let sheet = catalog
        .routes()
        .iter()
        .find(|route| route.key == "sheet-transfer")
        .unwrap();
    let delayed_leg = normalized
        .route_stages
        .iter_mut()
        .find(|leg| leg.route_id == sheet.id())
        .unwrap();
    assert_eq!(delayed_leg.travel_periods, 3);
    delayed_leg.travel_periods = 1;
    assert_eq!(*initial, normalized);
    let capacities = initial.capacities.clone();
    assert_eq!(
        initial.labor.iter().map(|row| row.available).sum::<u64>(),
        4960
    );
    install_reader_role(&target.writer).unwrap();
    provision_observer_role(&target.writer).unwrap();
    let observer_config = target.login("babylon_observer", "persistedtwins");
    let observer =
        ObserverEconomyReader::connect(&observer_config, ObserverVisibility::FullObserver).unwrap();
    let mut connection = observer_config.connect(NoTls).unwrap();
    for pair in [&mut standard, &mut delayed] {
        pair.history.push(pair.snapshots(&observer));
    }
    let mut first = [None, None];
    for tick in 1..=PROOF_PERIODS {
        standard.advance(&target, &observer, &mut connection);
        delayed.advance(&target, &observer, &mut connection);
        for (index, pair) in [&standard, &delayed].into_iter().enumerate() {
            assert_eq!(
                pair.uninterrupted
                    .session()
                    .material()
                    .state()
                    .capacities
                    .iter()
                    .collect::<Vec<_>>(),
                capacities
                    .iter()
                    .filter(|row| row.period > tick)
                    .collect::<Vec<_>>()
            );
            if subassembly_stock(&pair.history.last().unwrap()[0]) > 0 {
                first[index].get_or_insert(tick);
            }
        }
        assert_food_disconnected(
            production(&standard.history.last().unwrap()[0]),
            production(&delayed.history.last().unwrap()[0]),
        );
    }
    assert_eq!(first, [Some(5), Some(7)]);
    assert_eq!(standard.restart_periods, [1, 2]);
    assert_eq!(delayed.restart_periods, [1, 2, 4]);
    for pair in [&standard, &delayed] {
        assert!(subassembly_stock(&pair.history.last().unwrap()[0]) > 0);
        pair.assert_held_history(&observer);
        eprintln!(
            "PER-325 {:?}: first downstream output period {}, restart periods {:?}, final world {}",
            pair.preset,
            first[usize::from(pair.preset == MichiganDeliveryPreset::Delayed)].unwrap(),
            pair.restart_periods,
            pair.history.last().unwrap()[0]
                .nominal_world_hash
                .as_deref()
                .unwrap()
        );
    }
    assert_eq!(observer.campaigns().unwrap().len(), 4);
    assert!(observer
        .campaigns()
        .unwrap()
        .iter()
        .all(|campaign| campaign.durable_tick == PROOF_PERIODS));
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL harness; independent clone ownership"]
fn shared_freight_competition_is_committed_restart_safe_and_scope_confined() {
    let mut target = DisposableTarget::create();
    let mut ample = RunPair::create(&target, MichiganDeliveryPreset::SharedFreightAmple, 331_001);
    let mut constrained = RunPair::create(
        &target,
        MichiganDeliveryPreset::SharedFreightConstrained,
        331_003,
    );
    install_reader_role(&target.writer).unwrap();
    provision_observer_role(&target.writer).unwrap();
    let observer_config = target.login("babylon_observer", "sharedfreight");
    let observer =
        ObserverEconomyReader::connect(&observer_config, ObserverVisibility::FullObserver).unwrap();
    let preview = ObserverEconomyReader::connect(
        &target.login("babylon_reader", "sharedpreview"),
        ObserverVisibility::KnownPreview,
    )
    .unwrap();
    let mut connection = observer_config.connect(NoTls).unwrap();
    for pair in [&mut ample, &mut constrained] {
        let initial = pair.snapshots(&observer);
        let accounts = &production(&initial[0]).freight_capacity_accounts;
        assert_eq!(accounts.len(), 2);
        assert!(accounts.iter().all(|a| a.completed.is_none()));
        assert_eq!(
            accounts.iter().filter(|a| a.route_ids.len() == 2).count(),
            1
        );
        pair.history.push(initial);
    }
    for tick in 1..=PROOF_PERIODS {
        for pair in [&mut ample, &mut constrained] {
            pair.advance(&target, &observer, &mut connection);
            let snapshot = &pair.history.last().unwrap()[0];
            assert_shared_reservations(snapshot, pair.preset, tick);
            super::assert_known_material_absence(
                &preview
                    .snapshot(pair.uninterrupted.campaign_id(), tick)
                    .unwrap(),
            );
        }
    }
    let catalog = crate::test_support::catalog();
    for (pair, first_sheet, first_meal, panels, meals, panel_people, meal_people) in [
        (&ample, 320, 80, 32, 80, (4, 0), (2, 0)),
        (&constrained, 120, 40, 12, 40, (2, 2), (1, 1)),
    ] {
        let first = production(&pair.history[1][0]);
        let shared = first
            .freight_capacity_accounts
            .iter()
            .find(|a| a.route_ids.len() == 2)
            .unwrap();
        for (key, expected) in [("sheet", first_sheet), ("meal", first_meal)] {
            let good_id = identity_hex(catalog.good(key).unwrap().id().as_bytes());
            assert_eq!(
                shared.completed.as_ref().unwrap().reservations[0]
                    .orders
                    .iter()
                    .find(|order| order.good_id == good_id)
                    .unwrap()
                    .dispatched,
                expected
            );
        }
        let third = production(&pair.history[3][0]);
        for (key, output, people) in [
            ("macomb-fabricated-metal", panels, panel_people),
            ("oakland-food", meals, meal_people),
        ] {
            let id = identity_hex(catalog.site(key).unwrap().id().as_bytes());
            let site = third.sites.iter().find(|site| site.id == id).unwrap();
            assert_eq!(site.processes.len(), 1);
            let process = &site.processes[0];
            assert_eq!(
                process.produced_batches.unwrap() * process.output_per_batch,
                output
            );
            let staffing = third
                .staffing_accounts
                .iter()
                .find(|account| account.site_id == id)
                .unwrap();
            assert_eq!((staffing.employed, staffing.reserve), people);
        }
    }
    for pair in [&ample, &constrained] {
        assert_eq!(pair.restart_periods, [1, 2]);
        pair.assert_held_history(&observer);
    }
}

fn assert_shared_reservations(
    snapshot: &ObserverEconomySnapshot,
    preset: MichiganDeliveryPreset,
    tick: u64,
) {
    let accounts = &production(snapshot).freight_capacity_accounts;
    for account in accounts {
        let completed = account.completed.as_ref().unwrap();
        assert_eq!(completed.period, tick);
        for reservation in &completed.reservations {
            assert_eq!(
                reservation.opening_available_grams,
                reservation.newly_reserved_grams + reservation.remaining_available_grams
            );
            assert_eq!(
                reservation.newly_reserved_grams,
                reservation
                    .orders
                    .iter()
                    .map(|order| {
                        assert_eq!(
                            order.reserved_grams,
                            order.dispatched * order.grams_per_unit
                        );
                        order.reserved_grams
                    })
                    .sum::<u64>()
            );
        }
    }
    if tick == 1 {
        let shared = accounts.iter().find(|a| a.route_ids.len() == 2).unwrap();
        let reservation = &shared.completed.as_ref().unwrap().reservations[0];
        let expected = if preset == MichiganDeliveryPreset::SharedFreightAmple {
            (800_000, 400_000)
        } else {
            (160_000, 160_000)
        };
        assert_eq!(
            (
                reservation.opening_available_grams,
                reservation.newly_reserved_grams
            ),
            expected
        );
    }
}
