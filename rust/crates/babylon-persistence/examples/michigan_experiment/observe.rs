//! Read receipt facts and next-opening ceilings; never evaluate mechanics here.
use std::collections::BTreeMap;

use babylon_bsl::identity_codec::StableBslValue;
use babylon_graph::{hypergraph_store::HypergraphStore, stable_element::StableElementKey};
use babylon_material_circuit::{MaterialCircuitState, SiteId};
use babylon_persistence::michigan_material::{
    MichiganMaterialCatalog, MichiganMaterialInput, MichiganMaterialProcess,
};
use babylon_tick::{
    material_replay::PreparedMaterialTick,
    material_staffing::STAFFING_COMPOSITION_ID,
    material_world::{decode_material_receipts, MaterialTickReceipts},
    replay_session::SuccessfulEvent,
};
use serde::Serialize;
use serde_json::Value;

use super::{
    artifacts::{contract, Result},
    run::{hex, Case, PERIODS},
};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Staffing {
    pub opening_employed: u64,
    pub opening_reserve: u64,
    pub previous_unretained_hours: u64,
    pub current_unretained_hours: u64,
    pub retained_hours: u64,
    pub target_employed: u64,
    pub hires: u64,
    pub separations: u64,
    pub closing_employed: u64,
    pub closing_reserve: u64,
    pub next_opening_hours: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Production {
    pub planned_batches: u64,
    pub produced_batches: u64,
    pub consumed_input_units: u64,
    pub produced_output_units: u64,
    pub used_labor_hours: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct NextOpening {
    pub period: u64,
    pub capacity_batches: u64,
    pub input_batches: u64,
    pub labor_batches: u64,
    pub material_requested_batches: u64,
    pub planned_batches: u64,
    pub limiting_bounds: Vec<&'static str>,
    pub labor_below_material_request: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ProcessRow {
    pub process: String,
    pub site_id: String,
    pub input_good: String,
    pub output_good: String,
    pub input_unit: String,
    pub output_unit: String,
    pub opening_input: u64,
    pub closing_input: u64,
    pub opening_output: u64,
    pub closing_output: u64,
    pub completed_capacity_batches: u64,
    pub completed_available_labor_hours: u64,
    pub completed_receipt: Option<Production>,
    pub staffing: Staffing,
    pub next_opening: Option<NextOpening>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Transit {
    pub lot_id: String,
    pub quantity: u64,
    pub dispatch_period: u64,
    pub arrival_period: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RouteRow {
    pub route: String,
    pub route_id: String,
    pub good: String,
    pub unit: String,
    pub travel_periods: u16,
    pub capacity: u64,
    pub ordered: u64,
    pub unshipped_before: u64,
    pub supplier_available_at_dispatch: u64,
    pub dispatch_limiting_bounds: Vec<&'static str>,
    pub dispatched: u64,
    pub arrived: u64,
    pub delivered: u64,
    pub cumulative_shipped: u64,
    pub cumulative_delivered: u64,
    pub cumulative_realized: u64,
    pub cumulative_lost: u64,
    pub in_transit: Vec<Transit>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PeriodRow {
    pub schema: &'static str,
    pub case: &'static str,
    pub period: u64,
    pub tick_content_sha256: String,
    pub graph_tick_content_sha256: String,
    pub graph_state_sha256: String,
    pub graph_world_sha256: String,
    pub prior_world_sha256: String,
    pub world_sha256: String,
    pub material_receipts_sha256: String,
    pub graph_event_section_sha256: String,
    pub processes: Vec<ProcessRow>,
    pub routes: Vec<RouteRow>,
    pub metal_input_equivalent_kg: u64,
    pub food_kg: u64,
}

impl PeriodRow {
    pub fn output(&self, process: &str) -> u64 {
        self.processes
            .iter()
            .find(|row| row.process == process)
            .expect("validated fixed five-process catalog")
            .closing_output
    }
    pub fn food_evidence(&self) -> Value {
        serde_json::json!({
            "processes": self.processes.iter().filter(|row|
                row.process == "meal-milling" || row.process == "meal-packaging").collect::<Vec<_>>(),
            "routes": self.routes.iter().filter(|row| row.route == "food-transfer").collect::<Vec<_>>(),
            "food_kg": self.food_kg
        })
    }
}

fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| contract("observation addition overflow"))
}
fn multiply(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b)
        .ok_or_else(|| contract("observation multiplication overflow"))
}
fn total(mut values: impl Iterator<Item = u64>) -> Result<u64> {
    values.try_fold(0, add)
}

pub fn stock(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    site: SiteId,
    good: &str,
) -> Result<u64> {
    let good = catalog
        .good(good)
        .ok_or_else(|| contract("unknown inventory good"))?;
    total(
        state
            .inventory
            .iter()
            .filter(|row| row.site_id == site && row.good_id == good.id())
            .map(|row| row.quantity),
    )
}

fn workforce(event: &SuccessfulEvent, period: u64) -> Result<(String, Staffing)> {
    if event.emitting_rule() != STAFFING_COMPOSITION_ID
        || event.choice_receipt().is_some()
        || event.fields().len() != 13
    {
        return Err(contract("unexpected staffing evidence contract"));
    }
    let fields: BTreeMap<_, _> = event
        .fields()
        .iter()
        .map(|(key, value)| (key.as_str(), value))
        .collect();
    let number = |key: &str| match fields.get(key) {
        Some(StableBslValue::Int(value)) => {
            u64::try_from(*value).map_err(|_| contract(format!("negative staffing field {key}")))
        }
        _ => Err(contract(format!(
            "missing or non-integer staffing field {key}"
        ))),
    };
    if number("period")? != period {
        return Err(contract("staffing period mismatch"));
    }
    let Some(StableBslValue::Node(StableElementKey::Node { local_name, .. })) =
        fields.get("subject")
    else {
        return Err(contract("missing stable staffing subject"));
    };
    Ok((
        local_name.clone(),
        Staffing {
            opening_employed: number("opening-employed")?,
            opening_reserve: number("opening-reserve")?,
            previous_unretained_hours: number("previous-unretained-hours")?,
            current_unretained_hours: number("current-unretained-hours")?,
            retained_hours: number("retained-hours")?,
            target_employed: number("target-employed")?,
            hires: number("hires")?,
            separations: number("separations")?,
            closing_employed: number("closing-employed")?,
            closing_reserve: number("closing-reserve")?,
            next_opening_hours: number("next-opening-hours")?,
        },
    ))
}

fn staffing(
    candidate: &PreparedMaterialTick<HypergraphStore>,
) -> Result<BTreeMap<String, Staffing>> {
    let mut rows = BTreeMap::new();
    for event in candidate.graph_report().successful_event_batch().events() {
        if event.event_type() == "WORKFORCE_STAFFING"
            || event.emitting_rule() == STAFFING_COMPOSITION_ID
        {
            let (key, row) = workforce(event, candidate.identity().resolve_tick())?;
            if rows.insert(key, row).is_some() {
                return Err(contract("duplicate staffing account"));
            }
        }
    }
    if rows.len() != 5 {
        return Err(contract("expected five complete staffing accounts"));
    }
    Ok(rows)
}

fn capacity(state: &MaterialCircuitState, process: &MichiganMaterialProcess) -> Result<u64> {
    state
        .capacities
        .iter()
        .find(|row| {
            row.process_id == process.id()
                && row.site_id == process.site_id()
                && row.period == state.period
        })
        .map(|row| row.available_batches)
        .ok_or_else(|| contract("missing period capacity"))
}

fn labor(state: &MaterialCircuitState, process: &MichiganMaterialProcess) -> Result<u64> {
    state
        .labor
        .iter()
        .find(|row| row.site_id == process.site_id() && row.period == state.period)
        .map(|row| row.available)
        .ok_or_else(|| contract("missing period labor"))
}

fn plan(state: &MaterialCircuitState, process: &MichiganMaterialProcess) -> u64 {
    state
        .production_commitments
        .iter()
        .find(|row| row.process_id == process.id() && row.period == state.period)
        .map_or(0, |row| row.planned_batches)
}

fn minima(values: &[(&'static str, u64)]) -> Vec<&'static str> {
    let minimum = values.iter().map(|(_, value)| *value).min();
    values
        .iter()
        .filter_map(|(key, value)| (Some(*value) == minimum).then_some(*key))
        .collect()
}

fn next_opening(
    state: &MaterialCircuitState,
    process: &MichiganMaterialProcess,
    input: u64,
    input_recipe: &MichiganMaterialInput,
    staffing: &Staffing,
) -> Result<Option<NextOpening>> {
    if state.period > PERIODS {
        return Ok(None);
    }
    let capacity_batches = capacity(state, process)?;
    let labor_hours = labor(state, process)?;
    if staffing.next_opening_hours != labor_hours
        || !staffing
            .current_unretained_hours
            .is_multiple_of(process.labor_hours_per_batch)
    {
        return Err(contract(
            "staffing request/labor does not match single-process account",
        ));
    }
    let input_batches = input / input_recipe.quantity_per_batch;
    let labor_batches = labor_hours / process.labor_hours_per_batch;
    let material_requested_batches =
        staffing.current_unretained_hours / process.labor_hours_per_batch;
    let planned_batches = plan(state, process);
    Ok(Some(NextOpening {
        period: state.period,
        capacity_batches,
        input_batches,
        labor_batches,
        material_requested_batches,
        planned_batches,
        limiting_bounds: minima(&[
            ("input", input_batches),
            ("labor", labor_batches),
            ("capacity", capacity_batches),
        ]),
        labor_below_material_request: planned_batches < material_requested_batches,
    }))
}

fn production(
    receipts: &MaterialTickReceipts,
    opening: &MaterialCircuitState,
    process: &MichiganMaterialProcess,
    input_recipe: &MichiganMaterialInput,
) -> Result<Option<Production>> {
    let receipt = receipts
        .production
        .iter()
        .find(|row| row.process_id == process.id());
    let planned = plan(opening, process);
    let Some(row) = receipt else {
        if planned != 0 {
            return Err(contract("missing completed production receipt"));
        }
        return Ok(None);
    };
    if row.site_id != process.site_id()
        || row.planned_batches != planned
        || row.produced_batches > planned
    {
        return Err(contract("production receipt/commitment mismatch"));
    }
    Ok(Some(Production {
        planned_batches: planned,
        produced_batches: row.produced_batches,
        consumed_input_units: multiply(row.produced_batches, input_recipe.quantity_per_batch)?,
        produced_output_units: multiply(row.produced_batches, process.output_quantity_per_batch)?,
        used_labor_hours: multiply(row.produced_batches, process.labor_hours_per_batch)?,
    }))
}

fn processes(
    case: &Case,
    opening: &MaterialCircuitState,
    closing: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
    mut staffing: BTreeMap<String, Staffing>,
) -> Result<Vec<ProcessRow>> {
    let catalog = &case.catalog;
    catalog
        .processes()
        .iter()
        .map(|process| {
            let [input_recipe] = process.inputs.as_slice() else {
                return Err(contract(
                    "regional comparison expects one input per process",
                ));
            };
            let seed = catalog
                .staffing()
                .pools
                .iter()
                .find(|pool| {
                    pool.site_key == process.site_key && pool.process_keys.contains(&process.key)
                })
                .ok_or_else(|| contract("unbound process workforce"))?;
            let staffing = staffing
                .remove(&seed.local_name())
                .ok_or_else(|| contract("missing process staffing receipt"))?;
            let input = catalog
                .good(&input_recipe.good_key)
                .ok_or_else(|| contract("missing input good"))?;
            let output = catalog
                .good(&process.output_good_key)
                .ok_or_else(|| contract("missing output good"))?;
            let closing_input = stock(catalog, closing, process.site_id(), &input_recipe.good_key)?;
            Ok(ProcessRow {
                process: process.key.clone(),
                site_id: hex(&process.site_id().as_bytes()),
                input_good: input_recipe.good_key.clone(),
                output_good: process.output_good_key.clone(),
                input_unit: input.unit_key.clone(),
                output_unit: output.unit_key.clone(),
                opening_input: stock(catalog, opening, process.site_id(), &input_recipe.good_key)?,
                closing_input,
                opening_output: stock(
                    catalog,
                    opening,
                    process.site_id(),
                    &process.output_good_key,
                )?,
                closing_output: stock(
                    catalog,
                    closing,
                    process.site_id(),
                    &process.output_good_key,
                )?,
                completed_capacity_batches: capacity(opening, process)?,
                completed_available_labor_hours: labor(opening, process)?,
                completed_receipt: production(receipts, opening, process, input_recipe)?,
                next_opening: next_opening(
                    closing,
                    process,
                    closing_input,
                    input_recipe,
                    &staffing,
                )?,
                staffing,
            })
        })
        .collect()
}

fn transit(state: &MaterialCircuitState, route: babylon_material_circuit::RouteId) -> Vec<Transit> {
    state
        .freight
        .iter()
        .filter(|row| row.route_id == route)
        .map(|row| Transit {
            lot_id: hex(&row.lot_id.as_bytes()),
            quantity: row.quantity,
            dispatch_period: row.dispatch_period,
            arrival_period: row.stage_arrival_period,
        })
        .collect()
}

fn regional_route_capacity(
    opening: &MaterialCircuitState,
    route: babylon_material_circuit::RouteId,
    grams_per_unit: u64,
) -> Result<(u16, u64)> {
    let stages: Vec<_> = opening
        .route_stages
        .iter()
        .filter(|stage| stage.route_id == route)
        .collect();
    let [stage] = stages.as_slice() else {
        return Err(contract(
            "regional comparison expects one timed stage per route",
        ));
    };
    let memberships: Vec<_> = opening
        .route_stage_capacities
        .iter()
        .filter(|row| row.route_id == route && row.stage_index == stage.stage_index)
        .collect();
    let [membership] = memberships.as_slice() else {
        return Err(contract(
            "regional comparison expects one mass principal per route",
        ));
    };
    let capacity_grams = opening
        .corridor_capacities
        .iter()
        .find(|row| row.corridor_id == membership.corridor_id && row.period == opening.period)
        .map(|row| row.available_grams)
        .ok_or_else(|| contract("missing corridor mass capacity"))?;
    let capacity = capacity_grams / grams_per_unit;
    Ok((stage.travel_periods, capacity))
}

fn routes(
    case: &Case,
    opening: &MaterialCircuitState,
    closing: &MaterialCircuitState,
    receipts: &MaterialTickReceipts,
    processes: &[ProcessRow],
) -> Result<Vec<RouteRow>> {
    case.catalog
        .routes()
        .iter()
        .map(|route| {
            let prior = opening
                .orders
                .iter()
                .find(|row| row.order_id == route.order_id())
                .ok_or_else(|| contract("missing opening order"))?;
            let order = closing
                .orders
                .iter()
                .find(|row| row.order_id == route.order_id())
                .ok_or_else(|| contract("missing closing order"))?;
            let good = case
                .catalog
                .good(&route.good_key)
                .ok_or_else(|| contract("missing route good"))?;
            let produced = total(
                processes
                    .iter()
                    .filter(|row| {
                        row.site_id == hex(&prior.supplier_site_id.as_bytes())
                            && row.output_good == route.good_key
                    })
                    .map(|row| {
                        row.completed_receipt
                            .as_ref()
                            .map_or(0, |receipt| receipt.produced_output_units)
                    }),
            )?;
            let supply = add(
                stock(
                    &case.catalog,
                    opening,
                    prior.supplier_site_id,
                    &route.good_key,
                )?,
                produced,
            )?;
            let unshipped = prior
                .ordered
                .checked_sub(prior.shipped)
                .ok_or_else(|| contract("order shipped exceeds ordered"))?;
            let (travel_periods, capacity) =
                regional_route_capacity(opening, route.id(), good.grams_per_unit)?;
            Ok(RouteRow {
                route: route.key.clone(),
                route_id: hex(&route.id().as_bytes()),
                good: route.good_key.clone(),
                unit: good.unit_key.clone(),
                travel_periods,
                capacity,
                ordered: order.ordered,
                unshipped_before: unshipped,
                supplier_available_at_dispatch: supply,
                dispatch_limiting_bounds: minima(&[
                    ("supplier_stock", supply),
                    ("corridor", capacity),
                    ("order", unshipped),
                ]),
                dispatched: total(
                    receipts
                        .dispatches
                        .iter()
                        .filter(|row| row.order_id == route.order_id())
                        .map(|row| row.quantity),
                )?,
                arrived: total(
                    receipts
                        .arrivals
                        .iter()
                        .filter(|row| row.order_id == route.order_id())
                        .map(|row| row.quantity),
                )?,
                delivered: total(
                    receipts
                        .deliveries
                        .iter()
                        .filter(|row| row.order_id == route.order_id())
                        .map(|row| row.quantity),
                )?,
                cumulative_shipped: order.shipped,
                cumulative_delivered: order.delivered,
                cumulative_realized: order.realized,
                cumulative_lost: order.lost,
                in_transit: transit(closing, route.id()),
            })
        })
        .collect()
}

fn conserved(case: &Case, state: &MaterialCircuitState) -> Result<(u64, u64)> {
    let mut metal = 0;
    let mut food = 0;
    for (key, weight, is_metal) in [
        ("billet", 1, true),
        ("sheet", 1, true),
        ("panel", 10, true),
        ("subassembly", 20, true),
        ("grain", 1, false),
        ("meal", 1, false),
        ("packaged-meal", 1, false),
    ] {
        let good = case
            .catalog
            .good(key)
            .ok_or_else(|| contract("missing conservation good"))?;
        let held = total(
            state
                .inventory
                .iter()
                .filter(|row| row.good_id == good.id())
                .map(|row| row.quantity),
        )?;
        let freight = total(
            state
                .freight
                .iter()
                .filter(|row| row.good_id == good.id())
                .map(|row| row.quantity),
        )?;
        let quantity = multiply(add(held, freight)?, weight)?;
        if is_metal {
            metal = add(metal, quantity)?;
        } else {
            food = add(food, quantity)?;
        }
    }
    if metal != add(600, case.spec.opening_sheet_kg)? || food != 200 {
        return Err(contract("within-case material conservation failed"));
    }
    Ok((metal, food))
}

pub fn period(
    case: &Case,
    opening: &MaterialCircuitState,
    candidate: &PreparedMaterialTick<HypergraphStore>,
) -> Result<PeriodRow> {
    let closing = candidate.material().register().state();
    let receipts = decode_material_receipts(candidate.material().receipt_bytes())
        .map_err(|error| contract(format!("material receipt decode: {error:?}")))?;
    let identity = candidate.identity();
    if receipts.resolve_tick != opening.period || closing.period != opening.period + 1 {
        return Err(contract("observation period alignment failed"));
    }
    let processes = processes(case, opening, closing, &receipts, staffing(candidate)?)?;
    let routes = routes(case, opening, closing, &receipts, &processes)?;
    let (metal, food) = conserved(case, closing)?;
    Ok(PeriodRow {
        schema: "MichiganDeliveryStockPeriodV1",
        case: case.spec.id,
        period: opening.period,
        tick_content_sha256: hex(identity.tick_content_hash().as_bytes()),
        graph_tick_content_sha256: hex(identity.graph_tick_content_hash().as_bytes()),
        graph_state_sha256: hex(&candidate.graph_report().report().after),
        graph_world_sha256: hex(&identity.graph_world_after()),
        prior_world_sha256: hex(&identity.prior_world_hash()),
        world_sha256: hex(&identity.result_world_hash()),
        material_receipts_sha256: hex(&identity.receipt_digest()),
        graph_event_section_sha256: hex(&candidate
            .graph_report()
            .successful_event_batch()
            .source_digest()),
        processes,
        routes,
        metal_input_equivalent_kg: metal,
        food_kg: food,
    })
}
