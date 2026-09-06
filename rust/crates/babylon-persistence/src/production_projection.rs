//! Projection of exact committed material registers and evidence, without adjudication.

pub(crate) mod context;
mod labor;
pub(crate) mod material_balance;

use babylon_material_circuit::{MaterialCircuitStateV2, OrderIdV1, ProcessIdV1, SiteIdV1};
use babylon_tick::material_world::{MaterialTickReceiptsV3, MaterialWorldRegisterV2};

use crate::michigan_economy::digest_hex;
use crate::michigan_material::{
    michigan_material_catalog_v1, MichiganDeliveryPresetV1, MichiganMaterialCatalogV1,
    MichiganMaterialRouteV1, MICHIGAN_INDUSTRY_BASELINE_SHA256_V1,
    MICHIGAN_MATERIAL_SCENARIO_SHA256_V1,
};
use crate::{
    ProductionDeliveryEvidenceV1, ProductionDeliveryStageV1, ProductionEventV1,
    ProductionFreightV1, ProductionInputV1, ProductionLaborV1, ProductionRouteV1, ProductionSiteV1,
    ProductionSnapshotV1, ProductionStockV1,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductionProjectionErrorV1 {
    Content,
    State,
    History,
    Arithmetic,
}

pub(crate) fn project_material_observation_v1(
    preset: MichiganDeliveryPresetV1,
    register: &MaterialWorldRegisterV2,
    opening: Option<&MaterialWorldRegisterV2>,
    history: &[(MaterialTickReceiptsV3, [u8; 32])],
) -> Result<ProductionSnapshotV1, ProductionProjectionErrorV1> {
    let catalog =
        michigan_material_catalog_v1().map_err(|_| ProductionProjectionErrorV1::Content)?;
    let tick = register.completed_tick();
    if tick > preset.horizon_ticks() || u64::try_from(history.len()).ok() != Some(tick) {
        return Err(ProductionProjectionErrorV1::History);
    }
    for (index, (receipt, _)) in history.iter().enumerate() {
        if usize::try_from(receipt.resolve_tick).ok() != Some(index + 1) {
            return Err(ProductionProjectionErrorV1::History);
        }
    }
    let state = register.state();
    let labor_accounts = labor::project_labor_accounts(
        state,
        opening.map(MaterialWorldRegisterV2::state),
        history.last().map(|(receipt, _)| receipt),
    )?;
    let material_balance = material_balance::project_material_balance(
        state,
        opening.map(MaterialWorldRegisterV2::state),
        history.last().map(|(receipt, _)| receipt),
    )?;
    let sites = project_sites(catalog, state, history.last().map(|(receipt, _)| receipt))?;
    let routes = project_routes(catalog, state)?;
    let mut freight = Vec::new();
    for lot in &state.freight {
        let good = catalog
            .goods()
            .iter()
            .find(|row| row.id() == lot.good_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if good.unit_id() != lot.unit_id
            || !routes
                .iter()
                .any(|route| route.id == digest_hex(&lot.route_id.as_bytes()))
        {
            return Err(ProductionProjectionErrorV1::State);
        }
        freight.push(ProductionFreightV1 {
            id: digest_hex(&lot.lot_id.as_bytes()),
            route_id: digest_hex(&lot.route_id.as_bytes()),
            source_site_id: digest_hex(&lot.source_site_id.as_bytes()),
            destination_site_id: digest_hex(&lot.destination_site_id.as_bytes()),
            good_id: digest_hex(&good.id().as_bytes()),
            unit_id: digest_hex(&good.unit_id().as_bytes()),
            good: good.label.clone(),
            unit: good.unit_key.clone(),
            quantity: lot.quantity,
            dispatch_week: lot.dispatch_week,
            arrival_week: lot.leg_arrival_week,
        });
    }
    let mut events = Vec::new();
    for (receipt, digest) in history {
        project_events(catalog, receipt, *digest, &mut events)?;
    }
    Ok(ProductionSnapshotV1 {
        scenario_label: match preset {
            MichiganDeliveryPresetV1::Standard => "Michigan: standard delivery",
            MichiganDeliveryPresetV1::Delayed => "Michigan: delayed sheet delivery",
        }.to_owned(),
        horizon_week: preset.horizon_ticks(), sites, routes, freight, events, labor_accounts, material_balance,
        observed_contexts: Vec::new(), process_attributions: Vec::new(),
        provenance: vec![
            "Designed 16-week physical demonstration: county-industry aggregates; no factory locations.".to_owned(),
            "Recipes, opening stock, orders, labor-hours, capacity and route delays are Designed.".to_owned(),
            "QCEW 2024 private-industry employment is observed annual-average jobs; model labor-hours are separate.".to_owned(),
            "Terminal subassemblies and packaged meal remain unsold on hand. Quantity realization is delivery evidence, never payment.".to_owned(),
            format!("Designed content sha256:{MICHIGAN_MATERIAL_SCENARIO_SHA256_V1}"),
            format!("Observed industry artifact sha256:{MICHIGAN_INDUSTRY_BASELINE_SHA256_V1}; {}", catalog.source_url()),
        ],
    })
}

fn project_sites(
    catalog: &MichiganMaterialCatalogV1,
    state: &MaterialCircuitStateV2,
    receipt: Option<&MaterialTickReceiptsV3>,
) -> Result<Vec<ProductionSiteV1>, ProductionProjectionErrorV1> {
    let mut result = Vec::new();
    for site in catalog.sites() {
        let process = catalog
            .processes()
            .iter()
            .find(|row| row.site_key == site.key)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        let output = state
            .process_outputs
            .iter()
            .find(|row| row.process_id == process.id() && row.site_id == site.id())
            .ok_or(ProductionProjectionErrorV1::State)?;
        let good = catalog
            .good(&process.output_good_key)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        if output.good_id != good.id() || output.unit_id != good.unit_id() {
            return Err(ProductionProjectionErrorV1::State);
        }
        let inventory = project_inventory(catalog, state, site.id())?;
        let inputs = project_inputs(catalog, state, site.id(), process.id())?;
        let labor = state
            .labor_coefficients
            .iter()
            .filter(|row| row.process_id == process.id())
            .map(|coefficient| {
                let available = state
                    .labor
                    .iter()
                    .find(|row| {
                        row.site_id == site.id()
                            && row.unit_id == coefficient.unit_id
                            && row.week == state.week
                    })
                    .map_or(0, |row| row.available);
                ProductionLaborV1 {
                    unit: "Designed labor-hours".to_owned(),
                    available,
                    quantity_per_batch: coefficient.quantity_per_batch,
                }
            })
            .collect();
        let latest = receipt.and_then(|receipt| {
            receipt
                .production
                .iter()
                .find(|row| row.process_id == process.id() && row.site_id == site.id())
        });
        let baseline = catalog
            .industry_for_site(site)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        result.push(ProductionSiteV1 {
            id: digest_hex(&site.id().as_bytes()),
            county_geoid: site.county_geoid.clone(),
            name: site.label.clone(),
            industry_code: site.naics.clone(),
            observed_employment: Some(baseline.annual_avg_emplvl),
            output_good_id: digest_hex(&good.id().as_bytes()),
            output_unit_id: digest_hex(&good.unit_id().as_bytes()),
            output_good: good.label.clone(),
            output_unit: good.unit_key.clone(),
            output_per_batch: output.quantity_per_batch,
            available_batches: state
                .capacities
                .iter()
                .find(|row| {
                    row.process_id == process.id()
                        && row.site_id == site.id()
                        && row.week == state.week
                })
                .map_or(0, |row| row.available_batches),
            // The complete committed family omits zero commitments. This is a
            // derived zero reading, never a fabricated receipt or event.
            planned_batches: receipt.map(|_| latest.map_or(0, |row| row.planned_batches)),
            produced_batches: receipt.map(|_| latest.map_or(0, |row| row.produced_batches)),
            inventory,
            inputs,
            labor,
        });
    }
    Ok(result)
}

fn project_inventory(
    catalog: &MichiganMaterialCatalogV1,
    state: &MaterialCircuitStateV2,
    site_id: SiteIdV1,
) -> Result<Vec<ProductionStockV1>, ProductionProjectionErrorV1> {
    let mut inventory = Vec::new();
    for row in state.inventory.iter().filter(|row| row.site_id == site_id) {
        let good = catalog
            .goods()
            .iter()
            .find(|good| good.id() == row.good_id && good.unit_id() == row.unit_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        inventory.push(ProductionStockV1 {
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
    catalog: &MichiganMaterialCatalogV1,
    state: &MaterialCircuitStateV2,
    site_id: SiteIdV1,
    process_id: ProcessIdV1,
) -> Result<Vec<ProductionInputV1>, ProductionProjectionErrorV1> {
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
            .ok_or(ProductionProjectionErrorV1::State)?;
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
        inputs.push(ProductionInputV1 {
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
    catalog: &MichiganMaterialCatalogV1,
    state: &MaterialCircuitStateV2,
) -> Result<Vec<ProductionRouteV1>, ProductionProjectionErrorV1> {
    catalog
        .routes()
        .iter()
        .map(|route| {
            let order = state
                .orders
                .iter()
                .find(|order| order.order_id == route.order_id())
                .ok_or(ProductionProjectionErrorV1::State)?;
            let good = catalog
                .good(&route.good_key)
                .ok_or(ProductionProjectionErrorV1::Content)?;
            let travel_weeks = state
                .route_legs
                .iter()
                .filter(|leg| leg.route_id == route.id())
                .try_fold(0_u64, |sum, leg| {
                    sum.checked_add(u64::from(leg.travel_weeks))
                })
                .ok_or(ProductionProjectionErrorV1::State)?;
            let backlog = state
                .backlog
                .iter()
                .find(|row| row.order_id == order.order_id)
                .map_or(0, |row| row.quantity);
            Ok(ProductionRouteV1 {
                id: digest_hex(&route.id().as_bytes()),
                supplier_site_id: digest_hex(&order.supplier_site_id.as_bytes()),
                buyer_site_id: digest_hex(&order.buyer_site_id.as_bytes()),
                good_id: digest_hex(&good.id().as_bytes()),
                unit_id: digest_hex(&good.unit_id().as_bytes()),
                good: good.label.clone(),
                unit: good.unit_key.clone(),
                travel_weeks,
                ordered: order.ordered,
                shipped: order.shipped,
                delivered: order.delivered,
                lost: order.lost,
                realized: order.realized,
                backlog,
            })
        })
        .collect()
}

fn order_route(
    catalog: &MichiganMaterialCatalogV1,
    id: OrderIdV1,
) -> Result<&MichiganMaterialRouteV1, ProductionProjectionErrorV1> {
    catalog
        .routes()
        .iter()
        .find(|route| route.order_id() == id)
        .ok_or(ProductionProjectionErrorV1::State)
}

fn project_events(
    catalog: &MichiganMaterialCatalogV1,
    receipts: &MaterialTickReceiptsV3,
    digest: [u8; 32],
    events: &mut Vec<ProductionEventV1>,
) -> Result<(), ProductionProjectionErrorV1> {
    let receipt_digest = digest_hex(&digest);
    let mut emit = |kind: &str, subjects: Vec<String>, description: String, delivery_evidence| {
        events.push(ProductionEventV1 {
            id: format!("{receipt_digest}:{}", events.len()),
            week: receipts.resolve_tick,
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
            return Err(ProductionProjectionErrorV1::State);
        }
        emit_route_event(
            catalog,
            route,
            "dispatch",
            dispatch.quantity,
            Some(dispatch.final_arrival_week),
            None,
            &mut emit,
        )?;
    }
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
            Some(ProductionDeliveryStageV1::Arrival),
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
            Some(ProductionDeliveryStageV1::Delivery),
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
            Some(ProductionDeliveryStageV1::QuantityRealization),
            &mut emit,
        )?;
    }
    Ok(())
}

fn emit_production_event(
    catalog: &MichiganMaterialCatalogV1,
    production: &babylon_material_circuit::ProductionReceiptV1,
    emit: &mut impl FnMut(&str, Vec<String>, String, Option<ProductionDeliveryEvidenceV1>),
) -> Result<(), ProductionProjectionErrorV1> {
    let process = catalog
        .processes()
        .iter()
        .find(|process| {
            process.id() == production.process_id && process.site_id() == production.site_id
        })
        .ok_or(ProductionProjectionErrorV1::State)?;
    let site = catalog
        .site(&process.site_key)
        .ok_or(ProductionProjectionErrorV1::Content)?;
    let good = catalog
        .good(&process.output_good_key)
        .ok_or(ProductionProjectionErrorV1::Content)?;
    let quantity = production
        .produced_batches
        .checked_mul(process.output_quantity_per_batch)
        .ok_or(ProductionProjectionErrorV1::State)?;
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
    catalog: &MichiganMaterialCatalogV1,
    route: &MichiganMaterialRouteV1,
    kind: &str,
    quantity: u64,
    arrival: Option<u64>,
    stage: Option<ProductionDeliveryStageV1>,
    emit: &mut impl FnMut(&str, Vec<String>, String, Option<ProductionDeliveryEvidenceV1>),
) -> Result<(), ProductionProjectionErrorV1> {
    let supplier = catalog
        .site(&route.supplier_site_key)
        .ok_or(ProductionProjectionErrorV1::Content)?;
    let buyer = catalog
        .site(&route.buyer_site_key)
        .ok_or(ProductionProjectionErrorV1::Content)?;
    let good = catalog
        .good(&route.good_key)
        .ok_or(ProductionProjectionErrorV1::Content)?;
    let suffix = arrival.map_or_else(String::new, |week| format!(" Arrival week {week}."));
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
        stage.map(|stage| ProductionDeliveryEvidenceV1 {
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
    use crate::michigan_material::michigan_material_foundation_v1;
    use babylon_kernel::sha256_of;
    use babylon_tick::material_world::decode_material_receipts_v3;

    #[test]
    fn projection_uses_exact_committed_state_and_refuses_future_history() {
        let preset = MichiganDeliveryPresetV1::Standard;
        let opening =
            MaterialWorldRegisterV2::try_new(0, michigan_material_foundation_v1(preset).unwrap())
                .unwrap();
        let initial = project_material_observation_v1(preset, &opening, None, &[]).unwrap();
        assert!(initial.freight.is_empty());
        assert!(initial.events.is_empty());
        assert!(initial.material_balance.is_none());
        assert_eq!(initial.labor_accounts.len(), 5);
        assert!(initial
            .labor_accounts
            .iter()
            .all(|row| { row.completed.is_none() && row.next_opening_week == 1 }));
        assert!(initial
            .sites
            .iter()
            .all(|site| site.produced_batches.is_none()));
        let next = opening.prepare_next().unwrap();
        let receipt = decode_material_receipts_v3(next.receipt_bytes()).unwrap();
        let history = vec![(receipt, sha256_of(next.receipt_bytes()))];
        let snapshot =
            project_material_observation_v1(preset, next.register(), Some(&opening), &history)
                .unwrap();
        let starved = snapshot
            .sites
            .iter()
            .find(|site| site.industry_code == "332")
            .unwrap();
        assert_eq!(starved.planned_batches, Some(0));
        assert_eq!(starved.produced_batches, Some(0));
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
        assert!(snapshot.events.iter().all(|event| event.week == 1));
        assert_eq!(
            project_material_observation_v1(preset, &opening, None, &history),
            Err(ProductionProjectionErrorV1::History)
        );
        assert_eq!(
            project_material_observation_v1(preset, next.register(), Some(&opening), &[]),
            Err(ProductionProjectionErrorV1::History)
        );
    }

    fn week_three(preset: MichiganDeliveryPresetV1) -> ProductionSnapshotV1 {
        let mut register =
            MaterialWorldRegisterV2::try_new(0, michigan_material_foundation_v1(preset).unwrap())
                .unwrap();
        let mut history = Vec::new();
        let mut opening = None;
        for _ in 0..3 {
            let next = register.prepare_next().unwrap();
            history.push((
                decode_material_receipts_v3(next.receipt_bytes()).unwrap(),
                sha256_of(next.receipt_bytes()),
            ));
            opening = Some(std::mem::replace(&mut register, next.register().clone()));
        }
        project_material_observation_v1(preset, &register, opening.as_ref(), &history).unwrap()
    }

    #[test]
    fn physical_projection_preserves_good_identity_and_delivery_delay_causality() {
        let standard = week_three(MichiganDeliveryPresetV1::Standard);
        let delayed = week_three(MichiganDeliveryPresetV1::Delayed);
        let macomb = |snapshot: &ProductionSnapshotV1| {
            snapshot
                .sites
                .iter()
                .find(|site| site.county_geoid == "26099")
                .unwrap()
                .produced_batches
        };
        assert_eq!(macomb(&standard), Some(8));
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
            assert_eq!(route.good_id, supplier.output_good_id);
            assert_eq!(route.unit_id, supplier.output_unit_id);
            assert!(buyer
                .inputs
                .iter()
                .any(|input| input.good_id == route.good_id && input.unit_id == route.unit_id));
            assert!(standard
                .freight
                .iter()
                .filter(|lot| lot.route_id == route.id)
                .all(|lot| lot.good_id == route.good_id && lot.unit_id == route.unit_id));
        }
    }

    #[test]
    fn delivery_delay_changes_used_time_with_equal_budgets_and_unaffected_food() {
        let standard = week_three(MichiganDeliveryPresetV1::Standard);
        let delayed = week_three(MichiganDeliveryPresetV1::Delayed);
        for account in &standard.labor_accounts {
            let twin = delayed
                .labor_accounts
                .iter()
                .find(|row| row.site_id == account.site_id && row.unit_id == account.unit_id)
                .unwrap();
            let a = account.completed.as_ref().unwrap();
            let b = twin.completed.as_ref().unwrap();
            assert_eq!(a.week, 3);
            assert_eq!(a.opening, b.opening);
            assert_eq!(account.next_opening_available, twin.next_opening_available);
            assert_eq!(a.used.checked_add(a.unused), Some(a.opening));
            assert_eq!(b.used.checked_add(b.unused), Some(b.opening));
            let site = standard
                .sites
                .iter()
                .find(|site| site.id == account.site_id)
                .unwrap();
            if site.industry_code == "332" {
                assert_eq!(a.used, 8 * site.labor[0].quantity_per_batch);
                assert!(a.used > b.used);
                assert_eq!(b.used, 0);
                assert!(a.unused < b.unused);
            } else if site.industry_code == "311" {
                assert_eq!(account, twin);
            }
        }
    }
}

#[cfg(test)]
mod events_tests;
