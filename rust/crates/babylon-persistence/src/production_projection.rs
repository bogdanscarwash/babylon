//! Projection of exact committed material registers and evidence, without adjudication.

pub(crate) mod context;
mod freight;
mod labor;
pub(crate) mod material_balance;
mod merchants;
mod outbound;
pub(crate) mod staffing;

use babylon_material_circuit::{MaterialCircuitState, OrderId, ProcessId, SiteId};
use babylon_tick::material_world::{MaterialTickReceipts, MaterialWorldRegister};

use crate::michigan_economy::digest_hex;
use crate::michigan_material::{
    MichiganDeliveryPreset, MichiganMaterialCatalog, MichiganMaterialPath, MichiganMaterialRoute,
    MichiganSiteRole,
};
use crate::{
    production_observation::ProductionDeliveryEvidence,
    production_observation::ProductionDeliveryStage, production_observation::ProductionEvent,
    production_observation::ProductionFreight, production_observation::ProductionInput,
    production_observation::ProductionLabor, production_observation::ProductionPhysicalEdge,
    production_observation::ProductionProcess, production_observation::ProductionRoadSource,
    production_observation::ProductionRoute, production_observation::ProductionRouteTransport,
    production_observation::ProductionSite, production_observation::ProductionSiteRole,
    production_observation::ProductionSnapshot, production_observation::ProductionStock,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductionProjectionError {
    Content,
    State,
    History,
    Arithmetic,
}

pub(crate) fn project_material_observation(
    catalog: &MichiganMaterialCatalog,
    preset: MichiganDeliveryPreset,
    register: &MaterialWorldRegister,
    opening: Option<&MaterialWorldRegister>,
    history: &[(MaterialTickReceipts, [u8; 32])],
) -> Result<ProductionSnapshot, ProductionProjectionError> {
    let tick = register.completed_tick();
    if tick > catalog.horizon_ticks() || u64::try_from(history.len()).ok() != Some(tick) {
        return Err(ProductionProjectionError::History);
    }
    for (index, (receipt, _)) in history.iter().enumerate() {
        if usize::try_from(receipt.resolve_tick).ok() != Some(index + 1) {
            return Err(ProductionProjectionError::History);
        }
    }
    let state = register.state();
    let labor_accounts = labor::project_labor_accounts(
        state,
        opening.map(MaterialWorldRegister::state),
        history.last().map(|(receipt, _)| receipt),
    )?;
    let material_balance = material_balance::project_material_balance(
        catalog,
        state,
        opening.map(MaterialWorldRegister::state),
        history.last().map(|(receipt, _)| receipt),
    )?;
    let freight_capacity_accounts = freight::project_freight_capacity_accounts(
        catalog,
        state,
        opening.map(MaterialWorldRegister::state),
        history.last().map(|(receipt, _)| receipt),
    )?;
    let (merchant_handling_accounts, final_demand_accounts) = merchants::project_merchants(
        catalog,
        state,
        opening.map(MaterialWorldRegister::state),
        history.last().map(|(receipt, _)| receipt),
    )?;
    let sites = project_sites(catalog, state, history.last().map(|(receipt, _)| receipt))?;
    let routes = project_routes(catalog, state)?;
    let freight = project_in_transit_freight(catalog, state, &routes)?;
    let mut events = Vec::new();
    for (receipt, digest) in history {
        project_events(catalog, receipt, *digest, &mut events)?;
    }
    let physical_edges = catalog.physical_network().map_or_else(Vec::new, |network| {
        network
            .edges
            .iter()
            .map(|edge| ProductionPhysicalEdge {
                id: edge.id.clone(),
                shape_e7: edge.shape_e7.clone(),
                distance_mm: edge.distance_mm,
            })
            .collect()
    });
    let road_source = catalog.physical_network().map(|network| {
        let source = &network.source;
        ProductionRoadSource {
            pbf_sha256: source.pbf_sha256.clone(),
            pbf_bytes: source.pbf_bytes,
            pbf_url: source.pbf_url.clone(),
            replication_timestamp: source.replication_timestamp.clone(),
            footprint_sha256: source.footprint_sha256.clone(),
            buffer_degrees_e7: source.buffer_degrees_e7,
            extraction_version: source.extraction_version.clone(),
            distance_version: source.distance_version.clone(),
            routing_profile_version: source.routing_profile_version.clone(),
            graph_sha256: source.graph_sha256.clone(),
        }
    });
    Ok(ProductionSnapshot {
        scenario_label: scenario_label(preset).to_owned(),
        horizon_period: catalog.horizon_ticks(), content_authority_sha256: digest_hex(&catalog.defines_hash()),
        sites, routes, freight, events, labor_accounts, material_balance, freight_capacity_accounts,
        merchant_handling_accounts, final_demand_accounts, physical_edges, road_source,
        staffing_accounts: Vec::new(), observed_contexts: Vec::new(), process_attributions: Vec::new(),
        provenance: vec![
            format!("Designed {}-period physical circuit at {} resolution.", catalog.horizon_ticks(), catalog.geographic_scale()),
            "Recipes, stock, finite orders, workforce schedules and capacity quantities are Designed.".to_owned(),
            "Observed QCEW annual-average jobs are separate from current modeled employed and reserve people.".to_owned(),
            catalog.terminal_output_disposition().to_owned(),
            "Capacity reservations precede physical arrivals. Local transfer and end-buyer fulfillment are distinct stock movements; no payments or consumption are inferred.".to_owned(),
            format!("Captured content authority sha256:{}", digest_hex(&catalog.defines_hash())),
        ],
    })
}

fn project_in_transit_freight(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    routes: &[ProductionRoute],
) -> Result<Vec<ProductionFreight>, ProductionProjectionError> {
    let mut freight = Vec::new();
    for lot in &state.freight {
        let good = catalog
            .goods()
            .iter()
            .find(|row| row.id() == lot.good_id)
            .ok_or(ProductionProjectionError::State)?;
        if good.unit_id() != lot.unit_id
            || !routes
                .iter()
                .any(|route| route.id == digest_hex(&lot.route_id.as_bytes()))
        {
            return Err(ProductionProjectionError::State);
        }
        freight.push(ProductionFreight {
            id: digest_hex(&lot.lot_id.as_bytes()),
            route_id: digest_hex(&lot.route_id.as_bytes()),
            source_site_id: digest_hex(&lot.source_site_id.as_bytes()),
            destination_site_id: digest_hex(&lot.destination_site_id.as_bytes()),
            good_id: digest_hex(&good.id().as_bytes()),
            unit_id: digest_hex(&good.unit_id().as_bytes()),
            good: good.label.clone(),
            unit: good.unit_key.clone(),
            quantity: lot.quantity,
            dispatch_period: lot.dispatch_period,
            arrival_period: lot.stage_arrival_period,
            current_stage_index: lot.current_stage_index,
            grams_per_unit: outbound::mass(state, lot.good_id, lot.unit_id)?,
            mass_grams: lot
                .quantity
                .checked_mul(outbound::mass(state, lot.good_id, lot.unit_id)?)
                .ok_or(ProductionProjectionError::Arithmetic)?,
        });
    }
    Ok(freight)
}

fn scenario_label(preset: MichiganDeliveryPreset) -> &'static str {
    match preset {
        MichiganDeliveryPreset::Standard => "Michigan: standard delivery",
        MichiganDeliveryPreset::Delayed => "Michigan: delayed delivery",
        MichiganDeliveryPreset::SharedFreightAmple => "Michigan: shared freight — ample",
        MichiganDeliveryPreset::SharedFreightConstrained => {
            "Michigan: shared freight — constrained"
        }
        MichiganDeliveryPreset::StatewideBaseline => "Michigan statewide: baseline",
        MichiganDeliveryPreset::StatewideFreightConstraint => {
            "Michigan statewide: freight constraint"
        }
        MichiganDeliveryPreset::StatewidePackagingShortage => {
            "Michigan statewide: packaging shortage"
        }
        MichiganDeliveryPreset::StatewideBoth => {
            "Michigan statewide: freight and packaging constraints"
        }
    }
}

fn project_sites(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    receipt: Option<&MaterialTickReceipts>,
) -> Result<Vec<ProductionSite>, ProductionProjectionError> {
    let mut result = Vec::new();
    for site in catalog.sites() {
        let processes = catalog
            .processes()
            .iter()
            .filter(|row| row.site_key == site.key)
            .map(|process| project_process(catalog, state, process, receipt))
            .collect::<Result<Vec<_>, _>>()?;
        let role = match site.role {
            MichiganSiteRole::Production => ProductionSiteRole::Production,
            MichiganSiteRole::Wholesale => ProductionSiteRole::Wholesale,
            MichiganSiteRole::Retail => ProductionSiteRole::Retail,
        };
        if (role == ProductionSiteRole::Production) == processes.is_empty() {
            return Err(ProductionProjectionError::State);
        }
        let baseline = catalog
            .industry_for_site(site)
            .ok_or(ProductionProjectionError::Content)?;
        result.push(ProductionSite {
            id: digest_hex(&site.id().as_bytes()),
            county_geoid: site.county_geoid.clone(),
            name: site.label.clone(),
            industry_code: site.naics.clone(),
            observed_employment: baseline.annual_avg_emplvl,
            role,
            sector_code: site.sector_code.clone(),
            processes,
            inventory: project_inventory(catalog, state, site.id())?,
        });
    }
    if result
        .iter()
        .map(|site| site.processes.len())
        .sum::<usize>()
        != state.process_outputs.len()
        || result
            .iter()
            .filter(|site| site.role != ProductionSiteRole::Production)
            .count()
            != state.merchants.len()
    {
        return Err(ProductionProjectionError::State);
    }
    Ok(result)
}

fn project_process(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    process: &crate::michigan_material::MichiganMaterialProcess,
    receipt: Option<&MaterialTickReceipts>,
) -> Result<ProductionProcess, ProductionProjectionError> {
    let site = process.site_id();
    let output = state
        .process_outputs
        .iter()
        .find(|row| row.process_id == process.id() && row.site_id == site)
        .ok_or(ProductionProjectionError::State)?;
    let good = catalog
        .good(&process.output_good_key)
        .ok_or(ProductionProjectionError::Content)?;
    if output.good_id != good.id() || output.unit_id != good.unit_id() {
        return Err(ProductionProjectionError::State);
    }
    let latest = receipt.and_then(|receipt| {
        receipt
            .production
            .iter()
            .find(|row| row.process_id == process.id() && row.site_id == site)
    });
    let labor = state
        .labor_coefficients
        .iter()
        .filter(|row| row.process_id == process.id())
        .map(|coefficient| {
            let available = state
                .labor
                .iter()
                .find(|row| {
                    row.site_id == site
                        && row.unit_id == coefficient.unit_id
                        && row.period == state.period
                })
                .map_or(0, |row| row.available);
            ProductionLabor {
                unit: "labor-hours".to_owned(),
                available,
                quantity_per_batch: coefficient.quantity_per_batch,
            }
        })
        .collect();
    Ok(ProductionProcess {
        id: digest_hex(&process.id().as_bytes()),
        name: process.key.clone(),
        output_good_id: digest_hex(&good.id().as_bytes()),
        output_unit_id: digest_hex(&good.unit_id().as_bytes()),
        output_good: good.label.clone(),
        output_unit: good.unit_key.clone(),
        output_per_batch: output.quantity_per_batch,
        available_batches: state
            .capacities
            .iter()
            .find(|row| {
                row.process_id == process.id() && row.site_id == site && row.period == state.period
            })
            .map_or(0, |row| row.available_batches),
        planned_batches: receipt.map(|_| latest.map_or(0, |row| row.planned_batches)),
        produced_batches: receipt.map(|_| latest.map_or(0, |row| row.produced_batches)),
        inputs: project_inputs(catalog, state, site, process.id())?,
        labor,
    })
}

fn project_inventory(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    site_id: SiteId,
) -> Result<Vec<ProductionStock>, ProductionProjectionError> {
    let mut inventory = Vec::new();
    for row in state.inventory.iter().filter(|row| row.site_id == site_id) {
        let good = catalog
            .goods()
            .iter()
            .find(|good| good.id() == row.good_id && good.unit_id() == row.unit_id)
            .ok_or(ProductionProjectionError::State)?;
        inventory.push(ProductionStock {
            good_id: digest_hex(&good.id().as_bytes()),
            unit_id: digest_hex(&good.unit_id().as_bytes()),
            good: good.label.clone(),
            unit: good.unit_key.clone(),
            quantity: row.quantity,
        });
    }
    Ok(inventory)
}

fn project_inputs(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    site_id: SiteId,
    process_id: ProcessId,
) -> Result<Vec<ProductionInput>, ProductionProjectionError> {
    let mut inputs = Vec::new();
    for coefficient in state
        .input_coefficients
        .iter()
        .filter(|row| row.process_id == process_id)
    {
        let good = catalog
            .goods()
            .iter()
            .find(|good| good.id() == coefficient.good_id && good.unit_id() == coefficient.unit_id)
            .ok_or(ProductionProjectionError::State)?;
        let on_hand = state
            .inventory
            .iter()
            .find(|row| {
                row.site_id == site_id && row.good_id == good.id() && row.unit_id == good.unit_id()
            })
            .map_or(0, |row| row.quantity);
        let suppliers = state
            .supplier_routes
            .iter()
            .filter(|row| {
                row.buyer_site_id == site_id
                    && row.good_id == good.id()
                    && row.unit_id == good.unit_id()
            })
            .map(|row| digest_hex(&row.supplier_site_id.as_bytes()))
            .collect();
        inputs.push(ProductionInput {
            good_id: digest_hex(&good.id().as_bytes()),
            unit_id: digest_hex(&good.unit_id().as_bytes()),
            good: good.label.clone(),
            unit: good.unit_key.clone(),
            quantity_per_batch: coefficient.quantity_per_batch,
            on_hand,
            supplier_site_ids: suppliers,
        });
    }
    Ok(inputs)
}

fn project_routes(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
) -> Result<Vec<ProductionRoute>, ProductionProjectionError> {
    catalog
        .routes()
        .iter()
        .map(|route| {
            let order = state
                .orders
                .iter()
                .find(|order| order.order_id == route.order_id())
                .ok_or(ProductionProjectionError::State)?;
            let good = catalog
                .good(&route.good_key)
                .ok_or(ProductionProjectionError::Content)?;
            let stages = freight::project_route_stages(state, route.id())?;
            let travel_periods = stages
                .iter()
                .try_fold(0_u64, |sum, stage| sum.checked_add(stage.travel_periods))
                .ok_or(ProductionProjectionError::Arithmetic)?;
            let (transport_kind, physical_edge_ids, distance_mm) = match &route.path {
                MichiganMaterialPath::Local if stages.is_empty() => {
                    (ProductionRouteTransport::Local, Vec::new(), None)
                }
                MichiganMaterialPath::Routed {
                    travel_periods: authored,
                    physical_edge_keys,
                    distance_mm,
                    ..
                } if travel_periods == u64::from(*authored) => (
                    ProductionRouteTransport::Staged,
                    physical_edge_keys.clone(),
                    *distance_mm,
                ),
                _ => return Err(ProductionProjectionError::State),
            };
            Ok(ProductionRoute {
                id: digest_hex(&route.id().as_bytes()),
                supplier_site_id: digest_hex(&order.supplier_site_id.as_bytes()),
                buyer_site_id: digest_hex(&order.buyer_site_id.as_bytes()),
                good_id: digest_hex(&good.id().as_bytes()),
                unit_id: digest_hex(&good.unit_id().as_bytes()),
                good: good.label.clone(),
                unit: good.unit_key.clone(),
                travel_periods,
                stages,
                transport_kind,
                physical_edge_ids,
                distance_mm,
                grams_per_unit: outbound::mass(state, good.id(), good.unit_id())?,
                ordered: order.ordered,
                shipped: order.shipped,
                delivered: order.delivered,
                lost: order.lost,
                realized: order.realized,
                backlog: order
                    .ordered
                    .checked_sub(order.shipped)
                    .ok_or(ProductionProjectionError::State)?,
            })
        })
        .collect()
}

fn order_route(
    catalog: &MichiganMaterialCatalog,
    id: OrderId,
) -> Result<&MichiganMaterialRoute, ProductionProjectionError> {
    catalog
        .routes()
        .iter()
        .find(|route| route.order_id() == id)
        .ok_or(ProductionProjectionError::State)
}

fn project_events(
    catalog: &MichiganMaterialCatalog,
    receipts: &MaterialTickReceipts,
    digest: [u8; 32],
    events: &mut Vec<ProductionEvent>,
) -> Result<(), ProductionProjectionError> {
    let receipt_digest = digest_hex(&digest);
    let mut emit = |kind: &str, subjects: Vec<String>, description: String, delivery_evidence| {
        events.push(ProductionEvent {
            id: format!("{receipt_digest}:{}", events.len()),
            period: receipts.resolve_tick,
            subject_site_ids: subjects,
            kind: kind.to_owned(),
            description,
            receipt_digest: receipt_digest.clone(),
            delivery_evidence,
        });
    };
    for production in &receipts.production {
        emit_production_event(catalog, production, &mut emit)?;
    }
    for dispatch in &receipts.dispatches {
        let route = order_route(catalog, dispatch.order_id)?;
        if route.id() != dispatch.route_id {
            return Err(ProductionProjectionError::State);
        }
        emit_route_event(
            catalog,
            route,
            "dispatch",
            dispatch.quantity,
            Some(dispatch.final_arrival_period),
            None,
            &mut emit,
        )?;
    }
    emit_local_trade_events(catalog, receipts, &mut emit)?;
    for loss in &receipts.losses {
        emit_route_event(
            catalog,
            order_route(catalog, loss.order_id)?,
            "freight loss",
            loss.quantity,
            None,
            None,
            &mut emit,
        )?;
    }
    for arrival in &receipts.arrivals {
        emit_route_event(
            catalog,
            order_route(catalog, arrival.order_id)?,
            "arrival",
            arrival.quantity,
            None,
            Some(ProductionDeliveryStage::Arrival),
            &mut emit,
        )?;
    }
    for delivery in &receipts.deliveries {
        emit_route_event(
            catalog,
            order_route(catalog, delivery.order_id)?,
            "delivery",
            delivery.quantity,
            None,
            Some(ProductionDeliveryStage::Delivery),
            &mut emit,
        )?;
    }
    for realization in &receipts.realizations {
        emit_route_event(
            catalog,
            order_route(catalog, realization.order_id)?,
            "quantity realization",
            realization.quantity,
            None,
            Some(ProductionDeliveryStage::QuantityRealization),
            &mut emit,
        )?;
    }
    Ok(())
}

fn emit_local_trade_events(
    catalog: &MichiganMaterialCatalog,
    receipts: &MaterialTickReceipts,
    emit: &mut impl FnMut(&str, Vec<String>, String, Option<ProductionDeliveryEvidence>),
) -> Result<(), ProductionProjectionError> {
    for transfer in &receipts.local_transfers {
        let route = order_route(catalog, transfer.order_id)?;
        let supplier = catalog
            .site(&route.supplier_site_key)
            .ok_or(ProductionProjectionError::Content)?;
        let buyer = catalog
            .site(&route.buyer_site_key)
            .ok_or(ProductionProjectionError::Content)?;
        let good = catalog
            .good(&route.good_key)
            .ok_or(ProductionProjectionError::Content)?;
        if route.path != MichiganMaterialPath::Local
            || transfer.supplier_site_id != supplier.id()
            || transfer.buyer_site_id != buyer.id()
            || transfer.good_id != good.id()
            || transfer.unit_id != good.unit_id()
        {
            return Err(ProductionProjectionError::State);
        }
        emit_route_event(
            catalog,
            route,
            "local transfer",
            transfer.quantity,
            None,
            None,
            emit,
        )?;
    }
    for fulfillment in &receipts.local_fulfillments {
        let order = catalog
            .final_demands()
            .iter()
            .find(|order| order.order_id() == fulfillment.order_id)
            .ok_or(ProductionProjectionError::State)?;
        let retailer = catalog
            .site(&order.retailer_site_key)
            .ok_or(ProductionProjectionError::Content)?;
        let good = catalog
            .good(&order.good_key)
            .ok_or(ProductionProjectionError::Content)?;
        if fulfillment.retailer_site_id != retailer.id()
            || fulfillment.demand_principal_id != order.principal_id()
            || fulfillment.good_id != good.id()
            || fulfillment.unit_id != good.unit_id()
        {
            return Err(ProductionProjectionError::State);
        }
        emit("end-buyer fulfillment", vec![digest_hex(&retailer.id().as_bytes())],
            format!("{}: {} {} {} delivered to local end buyers; consumption and payment are not inferred.",
                retailer.label, fulfillment.quantity, good.unit_key, good.label), None);
    }
    Ok(())
}

fn emit_production_event(
    catalog: &MichiganMaterialCatalog,
    production: &babylon_material_circuit::ProductionReceipt,
    emit: &mut impl FnMut(&str, Vec<String>, String, Option<ProductionDeliveryEvidence>),
) -> Result<(), ProductionProjectionError> {
    let process = catalog
        .processes()
        .iter()
        .find(|process| {
            process.id() == production.process_id && process.site_id() == production.site_id
        })
        .ok_or(ProductionProjectionError::State)?;
    let site = catalog
        .site(&process.site_key)
        .ok_or(ProductionProjectionError::Content)?;
    let good = catalog
        .good(&process.output_good_key)
        .ok_or(ProductionProjectionError::Content)?;
    let quantity = production
        .produced_batches
        .checked_mul(process.output_quantity_per_batch)
        .ok_or(ProductionProjectionError::State)?;
    emit(
        "production",
        vec![digest_hex(&site.id().as_bytes())],
        format!(
            "{}: {} of {} planned batches; {quantity} {} {} produced.",
            site.label,
            production.produced_batches,
            production.planned_batches,
            good.unit_key,
            good.label
        ),
        None,
    );
    Ok(())
}

fn emit_route_event(
    catalog: &MichiganMaterialCatalog,
    route: &MichiganMaterialRoute,
    kind: &str,
    quantity: u64,
    arrival: Option<u64>,
    stage: Option<ProductionDeliveryStage>,
    emit: &mut impl FnMut(&str, Vec<String>, String, Option<ProductionDeliveryEvidence>),
) -> Result<(), ProductionProjectionError> {
    let supplier = catalog
        .site(&route.supplier_site_key)
        .ok_or(ProductionProjectionError::Content)?;
    let buyer = catalog
        .site(&route.buyer_site_key)
        .ok_or(ProductionProjectionError::Content)?;
    let good = catalog
        .good(&route.good_key)
        .ok_or(ProductionProjectionError::Content)?;
    let suffix = arrival.map_or_else(String::new, |period| format!(" Arrival period {period}."));
    emit(
        kind,
        vec![
            digest_hex(&supplier.id().as_bytes()),
            digest_hex(&buyer.id().as_bytes()),
        ],
        format!(
            "{} -> {}: {quantity} {} {} {kind}.{suffix}",
            supplier.label, buyer.label, good.unit_key, good.label
        ),
        stage.map(|stage| ProductionDeliveryEvidence {
            stage,
            order_id: digest_hex(&route.order_id().as_bytes()),
            route_id: digest_hex(&route.id().as_bytes()),
            good_id: digest_hex(&good.id().as_bytes()),
            unit_id: digest_hex(&good.unit_id().as_bytes()),
            quantity,
        }),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::michigan_content::MichiganContentPreset;
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_kernel::content_digest::sha256_of;
    use babylon_practice_contract::OrderedPracticeActionBatch;
    use babylon_tick::material_world::decode_material_receipts;
    use babylon_tick::replay_session::ReplayCommitDisposition;

    #[test]
    fn projection_uses_exact_committed_state_and_refuses_future_history() {
        let preset = MichiganDeliveryPreset::Standard;
        let session = MichiganContentPreset::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap()
            .into_session()
            .unwrap();
        let opening = session.material().clone();
        let initial = project_material_observation(
            &crate::test_support::catalog(),
            preset,
            &opening,
            None,
            &[],
        )
        .unwrap();
        assert!(initial.provenance[0].starts_with("Designed 16-period physical circuit"));
        assert!(initial.freight.is_empty());
        assert!(initial.events.is_empty());
        assert!(initial.material_balance.is_none());
        assert_eq!(initial.labor_accounts.len(), 5);
        assert!(initial
            .labor_accounts
            .iter()
            .all(|row| { row.completed.is_none() && row.next_opening_period == 1 }));
        assert!(initial.sites.iter().all(|site| site
            .processes
            .iter()
            .all(|process| process.produced_batches.is_none())));
        let actions = OrderedPracticeActionBatch::empty(
            session.graph_session().session_identity().clone(),
            1,
        )
        .unwrap();
        let prepared = session.prepare_advance(&actions).unwrap();
        let next = prepared.material();
        let receipt = decode_material_receipts(next.receipt_bytes()).unwrap();
        let history = vec![(receipt, sha256_of(next.receipt_bytes()))];
        let snapshot = project_material_observation(
            &crate::test_support::catalog(),
            preset,
            next.register(),
            Some(&opening),
            &history,
        )
        .unwrap();
        let starved = snapshot
            .sites
            .iter()
            .find(|site| site.industry_code == "332")
            .unwrap();
        assert_eq!(starved.processes[0].planned_batches, Some(0));
        assert_eq!(starved.processes[0].produced_batches, Some(0));
        assert!(!history[0]
            .0
            .production
            .iter()
            .any(|row| digest_hex(&row.site_id.as_bytes()) == starved.id));
        assert!(!snapshot.events.iter().any(
            |event| event.kind == "production" && event.subject_site_ids.contains(&starved.id)
        ));
        assert_eq!(
            snapshot
                .freight
                .iter()
                .map(|lot| lot.quantity)
                .collect::<Vec<_>>(),
            next.register()
                .state()
                .freight
                .iter()
                .map(|lot| lot.quantity)
                .collect::<Vec<_>>()
        );
        assert!(snapshot.events.iter().all(|event| event.period == 1));
        assert_eq!(
            project_material_observation(
                &crate::test_support::catalog(),
                preset,
                &opening,
                None,
                &history
            ),
            Err(ProductionProjectionError::History)
        );
        assert_eq!(
            project_material_observation(
                &crate::test_support::catalog(),
                preset,
                next.register(),
                Some(&opening),
                &[]
            ),
            Err(ProductionProjectionError::History)
        );
    }

    #[test]
    fn projection_provenance_uses_the_saved_campaign_horizon() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/scenarios/michigan/defines.toml"
        ));
        let catalog = MichiganMaterialCatalog::from_defines_toml(
            &source.replace("HORIZON_PERIODS = 16", "HORIZON_PERIODS = 8"),
        )
        .unwrap();
        let foundation = MichiganContentPreset::FourWeekStandard
            .create_foundation(&catalog)
            .unwrap();
        let stored = crate::sector_bundle::foundation::decode_stored_bundle_defines(
            foundation
                .graph_foundation()
                .content_bundle()
                .defines_bytes(),
            foundation.graph_foundation().content_digest().defines_hash,
        )
        .unwrap();
        let snapshot = project_material_observation(
            stored.catalog(),
            MichiganDeliveryPreset::Standard,
            foundation.initial_register(),
            None,
            &[],
        )
        .unwrap();
        assert_eq!(snapshot.horizon_period, 8);
        assert!(snapshot.provenance[0].starts_with("Designed 8-period physical circuit"));
    }

    fn period_three(preset: MichiganDeliveryPreset) -> ProductionSnapshot {
        let mut session = MichiganContentPreset::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap()
            .into_session()
            .unwrap();
        let mut history = Vec::new();
        let mut opening = None;
        for tick in 1..=3 {
            let actions = OrderedPracticeActionBatch::empty(
                session.graph_session().session_identity().clone(),
                tick,
            )
            .unwrap();
            let next = session.prepare_advance(&actions).unwrap();
            history.push((
                decode_material_receipts(next.material().receipt_bytes()).unwrap(),
                sha256_of(next.material().receipt_bytes()),
            ));
            opening = Some(session.material().clone());
            session
                .commit_prepared_and_publish(&mut CollectingSink::default(), next, |_| {
                    Ok::<_, ()>(ReplayCommitDisposition::Committed)
                })
                .unwrap();
        }
        project_material_observation(
            &crate::test_support::catalog().with_preset(preset).unwrap(),
            preset,
            session.material(),
            opening.as_ref(),
            &history,
        )
        .unwrap()
    }

    #[test]
    fn physical_projection_preserves_good_identity_and_delivery_delay_causality() {
        let standard = period_three(MichiganDeliveryPreset::Standard);
        let delayed = period_three(MichiganDeliveryPreset::Delayed);
        let macomb = |snapshot: &ProductionSnapshot| {
            snapshot
                .sites
                .iter()
                .find(|site| site.county_geoid == "26099")
                .unwrap()
                .processes[0]
                .produced_batches
        };
        // The first 32 rolling batches consume 320 of the 600 billet units.
        // Standard freight delivers those 320 sheets in period 2: 32 panel
        // batches at 10 sheets each fill the 8-per-week × 4-week capacity.
        // The three-period delayed route has not arrived by period 3.
        assert_eq!(macomb(&standard), Some(32));
        assert_eq!(macomb(&delayed), Some(0));
        for site in standard
            .sites
            .iter()
            .filter(|site| site.industry_code == "311")
        {
            assert_eq!(
                site,
                delayed
                    .sites
                    .iter()
                    .find(|other| other.id == site.id)
                    .unwrap()
            );
        }
        for route in &standard.routes {
            let supplier = standard
                .sites
                .iter()
                .find(|site| site.id == route.supplier_site_id)
                .unwrap();
            let buyer = standard
                .sites
                .iter()
                .find(|site| site.id == route.buyer_site_id)
                .unwrap();
            assert!(supplier
                .processes
                .iter()
                .any(|process| process.output_good_id == route.good_id
                    && process.output_unit_id == route.unit_id));
            assert!(buyer
                .processes
                .iter()
                .flat_map(|process| &process.inputs)
                .any(|input| input.good_id == route.good_id && input.unit_id == route.unit_id));
            assert!(standard
                .freight
                .iter()
                .filter(|lot| lot.route_id == route.id)
                .all(|lot| lot.good_id == route.good_id && lot.unit_id == route.unit_id));
        }
    }

    #[test]
    fn delivery_delay_changes_staffed_time_budgets_and_preserves_unaffected_food() {
        let standard = period_three(MichiganDeliveryPreset::Standard);
        let delayed = period_three(MichiganDeliveryPreset::Delayed);
        for account in &standard.labor_accounts {
            let twin = delayed
                .labor_accounts
                .iter()
                .find(|row| row.site_id == account.site_id && row.unit_id == account.unit_id)
                .unwrap();
            let a = account.completed.as_ref().unwrap();
            let b = twin.completed.as_ref().unwrap();
            assert_eq!(a.period, 3);
            assert_eq!(a.used.checked_add(a.unused), Some(a.opening));
            assert_eq!(b.used.checked_add(b.unused), Some(b.opening));
            let site = standard
                .sites
                .iter()
                .find(|site| site.id == account.site_id)
                .unwrap();
            if site.industry_code == "332" {
                // Four workers × 40 hours × four weeks supply 640 hours;
                // the 32 panel batches each consume 20 hours in period 3.
                assert_eq!(a.used, 640);
                assert!(a.used > b.used);
                assert_eq!(b.used, 0);
                assert_eq!((a.opening, b.opening), (640, 0));
                assert_eq!((a.unused, b.unused), (0, 0));
            } else if site.industry_code == "311" {
                assert_eq!(account, twin);
            }
        }
    }
}

#[cfg(test)]
mod events_tests;
