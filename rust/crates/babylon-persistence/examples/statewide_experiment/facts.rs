//! Copy committed register/receipt facts into bounded operator rows; no allocation or planning.
use super::{
    checked_sum, hex, quantity, refused,
    report::{CompletedPeriod, FinalDemand, NativeQuantity, Owner, Production, Route},
    Result,
};
use babylon_bsl::identity_codec::StableBslValue;
use babylon_graph::{hypergraph_store::HypergraphStore, stable_element::StableElementKey};
use babylon_material_circuit::{MaterialCircuitState, OrderId};
use babylon_persistence::michigan_material::MichiganMaterialCatalog;
use babylon_tick::{
    material_replay::PreparedMaterialTick,
    material_staffing::STAFFING_COMPOSITION_ID,
    material_world::{decode_material_receipts, MaterialTickReceipts},
    replay_session::SuccessfulEvent,
};
use std::collections::BTreeMap;

pub fn period(
    catalog: &MichiganMaterialCatalog,
    opening: &MaterialCircuitState,
    candidate: &PreparedMaterialTick<HypergraphStore>,
    capacity_key: &str,
) -> Result<CompletedPeriod> {
    let register = candidate.material().register();
    let closing = register.state();
    let receipts = decode_material_receipts(candidate.material().receipt_bytes())?;
    if receipts.resolve_tick != opening.period
        || Some(closing.period) != opening.period.checked_add(1)
    {
        return Err(refused("operator receipt/state period mismatch"));
    }
    let routes = routes(catalog, closing, &receipts)?;
    let capacity = catalog
        .corridors()
        .iter()
        .find(|row| row.key == capacity_key)
        .ok_or_else(|| refused("selected capacity disappeared"))?;
    let opening_grams = opening
        .corridor_capacities
        .iter()
        .find(|row| row.period == opening.period && row.corridor_id == capacity.id())
        .ok_or_else(|| refused("missing selected period capacity"))?
        .available_grams;
    let mut reserved = 0_u64;
    for route in catalog.routes().iter().filter(|route| {
        opening.route_stage_capacities.iter().any(|membership| {
            membership.route_id == route.id() && membership.corridor_id == capacity.id()
        })
    }) {
        let good = catalog
            .good(&route.good_key)
            .ok_or_else(|| refused("route good missing"))?;
        let grams = quantity(routes[&route.key].dispatched, good.grams_per_unit)?;
        reserved = reserved
            .checked_add(grams)
            .ok_or_else(|| refused("selected reservation overflow"))?;
    }
    if reserved > opening_grams {
        return Err(refused("committed dispatch exceeds selected capacity"));
    }
    let identity = candidate.identity();
    Ok(CompletedPeriod {
        period: receipts.resolve_tick,
        tick_content_sha256: hex(identity.tick_content_hash().as_bytes()),
        prior_world_sha256: hex(&identity.prior_world_hash()),
        world_sha256: hex(&identity.result_world_hash()),
        material_state_sha256: hex(&register.digest()),
        material_receipts_sha256: hex(&identity.receipt_digest()),
        staffing_events_sha256: hex(&candidate
            .graph_report()
            .successful_event_batch()
            .source_digest()),
        register_bytes: register.canonical_bytes().len(),
        receipt_bytes: candidate.material().receipt_bytes().len(),
        maximum_family_rows: maximum_rows(closing),
        selected_capacity_opening_grams: opening_grams,
        selected_capacity_reserved_grams: reserved,
        processes: processes(catalog, opening, &receipts)?,
        owners: owners(catalog, closing, &receipts, candidate)?,
        routes,
        final_demand: final_demand(catalog, closing, &receipts)?,
    })
}
fn processes(
    catalog: &MichiganMaterialCatalog,
    opening: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
) -> Result<BTreeMap<String, Production>> {
    catalog
        .processes()
        .iter()
        .map(|process| {
            let planned = opening
                .production_commitments
                .iter()
                .find(|row| row.process_id == process.id() && row.period == opening.period)
                .map_or(0, |row| row.planned_batches);
            let receipt = receipts
                .production
                .iter()
                .find(|row| row.process_id == process.id());
            let produced = match receipt {
                Some(row) if row.site_id == process.site_id() && row.planned_batches == planned => {
                    row.produced_batches
                }
                None if planned == 0 => 0,
                _ => {
                    return Err(refused(
                        "production receipt does not match opening commitment",
                    ))
                }
            };
            let consumed = process
                .inputs
                .iter()
                .map(|input| {
                    let good = catalog
                        .good(&input.good_key)
                        .ok_or_else(|| refused("input good missing"))?;
                    Ok((
                        input.good_key.clone(),
                        NativeQuantity {
                            unit: good.unit_key.clone(),
                            quantity: quantity(produced, input.quantity_per_batch)?,
                        },
                    ))
                })
                .collect::<Result<_>>()?;
            Ok((
                process.key.clone(),
                Production {
                    planned_batches: planned,
                    produced_batches: produced,
                    output_units: quantity(produced, process.output_quantity_per_batch)?,
                    consumed_inputs: consumed,
                    used_labor_hours: quantity(produced, process.labor_hours_per_batch)?,
                },
            ))
        })
        .collect()
}
fn staffing_event(event: &SuccessfulEvent, period: u64) -> Result<(String, BTreeMap<String, u64>)> {
    if event.event_type() != "WORKFORCE_STAFFING"
        || event.emitting_rule() != STAFFING_COMPOSITION_ID
        || event.choice_receipt().is_some()
        || event.fields().len() != 13
    {
        return Err(refused("unexpected staffing event family"));
    }
    let mut subject = None;
    let mut numbers = BTreeMap::new();
    for (name, value) in event.fields() {
        if name == "subject" {
            let StableBslValue::Node(StableElementKey::Node { local_name, .. }) = value else {
                return Err(refused("staffing subject is not a stable node"));
            };
            subject = Some(local_name.clone());
        } else {
            let StableBslValue::Int(value) = value else {
                return Err(refused("staffing quantity is not an integer"));
            };
            if numbers
                .insert(name.clone(), u64::try_from(*value)?)
                .is_some()
            {
                return Err(refused("duplicate staffing field"));
            }
        }
    }
    if numbers.get("period") != Some(&period) || numbers.len() != 12 {
        return Err(refused("staffing period or field coverage mismatch"));
    }
    Ok((
        subject.ok_or_else(|| refused("staffing subject missing"))?,
        numbers,
    ))
}
fn owners(
    catalog: &MichiganMaterialCatalog,
    closing: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
    candidate: &PreparedMaterialTick<HypergraphStore>,
) -> Result<BTreeMap<String, Owner>> {
    let mut workforce = BTreeMap::new();
    for event in candidate.graph_report().successful_event_batch().events() {
        if event.event_type() == "WORKFORCE_STAFFING"
            || event.emitting_rule() == STAFFING_COMPOSITION_ID
        {
            let (key, fields) = staffing_event(event, receipts.resolve_tick)?;
            if workforce.insert(key, fields).is_some() {
                return Err(refused("duplicate workforce owner"));
            }
        }
    }
    let mut result = BTreeMap::new();
    for site in catalog.sites() {
        let seed = catalog
            .staffing()
            .pools
            .iter()
            .find(|pool| pool.site_key == site.key)
            .ok_or_else(|| refused("owner has no workforce pool"))?;
        let staffing = workforce
            .remove(&seed.local_name())
            .ok_or_else(|| refused("owner staffing event missing"))?;
        let mut inventory = BTreeMap::new();
        for row in closing
            .inventory
            .iter()
            .filter(|row| row.site_id == site.id())
        {
            let good = catalog
                .goods()
                .iter()
                .find(|good| good.id() == row.good_id && good.unit_id() == row.unit_id)
                .ok_or_else(|| refused("inventory identity missing"))?;
            if inventory
                .insert(
                    good.key.clone(),
                    NativeQuantity {
                        unit: good.unit_key.clone(),
                        quantity: row.quantity,
                    },
                )
                .is_some()
            {
                return Err(refused("duplicate owner inventory principal"));
            }
        }
        result.insert(
            site.key.clone(),
            Owner {
                staffing,
                closing_inventory: inventory,
                handling_needed_hours: checked_sum(
                    receipts
                        .handling
                        .iter()
                        .filter(|row| row.site_id == site.id())
                        .map(|row| row.needed_hours),
                )?,
                handling_used_hours: checked_sum(
                    receipts
                        .handling
                        .iter()
                        .filter(|row| row.site_id == site.id())
                        .map(|row| row.used_hours),
                )?,
            },
        );
    }
    if !workforce.is_empty() {
        return Err(refused("unknown workforce event owner"));
    }
    Ok(result)
}
fn movement(receipts: impl Iterator<Item = (OrderId, u64)>, order: OrderId) -> Result<u64> {
    checked_sum(receipts.filter_map(|(id, quantity)| (id == order).then_some(quantity)))
}
fn routes(
    catalog: &MichiganMaterialCatalog,
    closing: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
) -> Result<BTreeMap<String, Route>> {
    catalog
        .routes()
        .iter()
        .map(|route| {
            let id = route.order_id();
            let order = closing
                .orders
                .iter()
                .find(|row| row.order_id == id)
                .ok_or_else(|| refused("closing delivery order missing"))?;
            Ok((
                route.key.clone(),
                Route {
                    dispatched: movement(
                        receipts.dispatches.iter().map(|r| (r.order_id, r.quantity)),
                        id,
                    )?,
                    arrived: movement(
                        receipts.arrivals.iter().map(|r| (r.order_id, r.quantity)),
                        id,
                    )?,
                    local_transferred: movement(
                        receipts
                            .local_transfers
                            .iter()
                            .map(|r| (r.order_id, r.quantity)),
                        id,
                    )?,
                    delivered: movement(
                        receipts.deliveries.iter().map(|r| (r.order_id, r.quantity)),
                        id,
                    )?,
                    cumulative_shipped: order.shipped,
                    cumulative_delivered: order.delivered,
                    cumulative_realized: order.realized,
                    cumulative_lost: order.lost,
                    outstanding: order
                        .ordered
                        .checked_sub(order.shipped)
                        .ok_or_else(|| refused("delivery order overshipped"))?,
                    in_transit: checked_sum(
                        closing
                            .freight
                            .iter()
                            .filter(|lot| lot.order_id == id)
                            .map(|lot| lot.quantity),
                    )?,
                },
            ))
        })
        .collect()
}
fn final_demand(
    catalog: &MichiganMaterialCatalog,
    closing: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
) -> Result<BTreeMap<String, FinalDemand>> {
    catalog
        .final_demands()
        .iter()
        .map(|demand| {
            let order = closing
                .final_demand_orders
                .iter()
                .find(|row| row.order_id == demand.order_id())
                .ok_or_else(|| refused("final demand order missing"))?;
            let good = catalog
                .good(&demand.good_key)
                .ok_or_else(|| refused("final demand good missing"))?;
            Ok((
                demand.key.clone(),
                FinalDemand {
                    retailer: demand.retailer_site_key.clone(),
                    county_geoid: demand.county_geoid.clone(),
                    good: good.key.clone(),
                    unit: good.unit_key.clone(),
                    ordered: order.ordered,
                    fulfilled_this_period: movement(
                        receipts
                            .local_fulfillments
                            .iter()
                            .map(|r| (r.order_id, r.quantity)),
                        order.order_id,
                    )?,
                    cumulative_fulfilled: order.fulfilled,
                    outstanding: order
                        .ordered
                        .checked_sub(order.fulfilled)
                        .ok_or_else(|| refused("final demand overfulfilled"))?,
                },
            ))
        })
        .collect()
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
    .unwrap_or(0)
}
