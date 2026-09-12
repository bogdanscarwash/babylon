use std::sync::OnceLock;

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{hypergraph_store::HypergraphStore, stable_state::StableGraphState};
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::{
    material_replay::PreparedMaterialTick, material_staffing::StaffingComposition,
    material_world::decode_material_receipts, replay_session::ReplayCommitDisposition,
};
use serde_json::Value;

use super::*;
use crate::{
    identity::CampaignId,
    material_envelope::CommittedMaterialTickEnvelope,
    michigan_content::MichiganContentPreset,
    michigan_economy::digest_hex,
    michigan_material::MichiganDeliveryPreset,
    production_projection::{project_material_observation, staffing::project_staffing_accounts},
    runtime::prepare_committed_tick,
};

/// Exercises the existing engine, canonical envelope, publication and projector.
/// The commit callback is an in-memory sink; this is not live-Postgres evidence.
fn published_observations() -> &'static [ObserverEconomySnapshot] {
    static OBSERVATIONS: OnceLock<Vec<ObserverEconomySnapshot>> = OnceLock::new();
    OBSERVATIONS.get_or_init(|| {
        let preset = MichiganDeliveryPreset::Standard;
        let foundation = MichiganContentPreset::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let foundation_digest = foundation.digest();
        let composition = foundation.labor().clone();
        let mut session = foundation.into_session().unwrap();
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(293));
        let mut observation = ObserverEconomySnapshot {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 0,
            foundation_digest: digest_hex(&foundation_digest),
            nominal_world_hash: None,
            tick_content_hash: None,
            envelope_digest: None,
            visibility: ObserverVisibility::FullObserver,
            counties: vec![],
            production: Some(
                project_material_observation(
                    &crate::test_support::catalog(),
                    preset,
                    session.material(),
                    None,
                    &[],
                )
                .unwrap(),
            ),
        };
        observation.production.as_mut().unwrap().staffing_accounts = project_staffing_accounts(
            &composition,
            &session.graph_session().stable_graph_state().unwrap(),
            session.material(),
            None,
            &[],
        )
        .unwrap();
        let mut result = vec![observation.clone()];
        let mut history = Vec::new();
        let mut sink = CollectingSink::default();
        for tick in 1..=3 {
            let actions = OrderedPracticeActionBatch::empty(
                session.graph_session().session_identity().clone(),
                tick,
            )
            .unwrap();
            let opening = session.material().clone();
            let opening_graph = session.graph_session().stable_graph_state().unwrap();
            let prepared = session.prepare_advance(&actions).unwrap();
            let staffing = prepared_staffing(&composition, &opening_graph, &prepared);
            let identity = *prepared.identity();
            let receipt = decode_material_receipts(prepared.material().receipt_bytes()).unwrap();
            let families = prepare_committed_tick(prepared.graph_report())
                .unwrap()
                .into_material_families(identity.tick_content_hash())
                .unwrap();
            let envelope = CommittedMaterialTickEnvelope::compose(
                campaign,
                &identity,
                families,
                prepared.material().register().canonical_bytes(),
                prepared.material().receipt_bytes(),
            )
            .unwrap();
            let (ack, _) = session
                .commit_prepared_and_publish(&mut sink, prepared, |_| {
                    Ok::<_, ()>(ReplayCommitDisposition::Committed)
                })
                .unwrap();
            history.push((receipt, ack.receipt_digest()));
            observation.resolve_tick = ack.resolve_tick();
            observation.tick_content_hash = Some(digest_hex(ack.tick_content_hash().as_bytes()));
            observation.envelope_digest = Some(digest_hex(&envelope.digest()));
            observation.nominal_world_hash = Some(digest_hex(&ack.result_world_hash()));
            observation.production = Some(
                project_material_observation(
                    &crate::test_support::catalog(),
                    preset,
                    session.material(),
                    Some(&opening),
                    &history,
                )
                .unwrap(),
            );
            observation.production.as_mut().unwrap().staffing_accounts = staffing;
            result.push(observation.clone());
        }
        result
    })
}

fn prepared_staffing(
    composition: &StaffingComposition,
    opening: &StableGraphState,
    prepared: &PreparedMaterialTick<HypergraphStore>,
) -> Vec<crate::production_observation::ProductionStaffingAccount> {
    let report = prepared.graph_report();
    let events = report
        .successful_event_batch()
        .events()
        .iter()
        .map(|event| crate::stored_tick::StoredEvent {
            emitting_rule: event.emitting_rule().to_owned(),
            choice_receipt_ordinal: event
                .choice_receipt()
                .map(babylon_tick::choice_receipt::ChoiceReceiptRef::encounter_ordinal),
            event_type: event.event_type().to_owned(),
            fields: event.fields().to_vec(),
        })
        .collect::<Vec<_>>();
    project_staffing_accounts(
        composition,
        report.result_stable_graph(),
        prepared.material().register(),
        Some(opening),
        &events,
    )
    .unwrap()
}

fn committed() -> ObserverEconomySnapshot {
    published_observations()[1].clone()
}

fn digest(snapshot: &ObserverEconomySnapshot) -> ProductionEvidenceDigest {
    snapshot.production_evidence_digest().unwrap().unwrap()
}

/// Add the disclosure families absent from the small regional engine fixture.
/// All state/receipt projections are tested at their authenticated seams; this
/// fixture tests that the public evidence encoder binds every disclosed field.
fn full_disclosure() -> ObserverEconomySnapshot {
    use crate::production_observation::{
        CompletedProductionFinalDemand, CompletedProductionMerchantHandling,
        ProductionFinalDemandAccount, ProductionFinalDemandOrder, ProductionHandlingCoefficient,
        ProductionMerchantHandlingAccount, ProductionMerchantHandlingOrder, ProductionOutboundKind,
        ProductionPhysicalEdge, ProductionRoadSource,
    };
    let mut observation = committed();
    let production = observation.production.as_mut().unwrap();
    let site = production.sites[0].id.clone();
    let stock = production.sites[0].inventory[0].clone();
    production.physical_edges = vec![
        ProductionPhysicalEdge {
            id: "edge-a".to_owned(),
            shape_e7: vec![[-830_000_000, 420_000_000], [-830_001_000, 420_001_000]],
            distance_mm: 17_000,
        },
        ProductionPhysicalEdge {
            id: "edge-b".to_owned(),
            shape_e7: vec![[-830_001_000, 420_001_000], [-830_003_000, 420_003_000]],
            distance_mm: 33_000,
        },
    ];
    production.routes[0].physical_edge_ids = vec![
        "edge-a".to_owned(),
        "edge-b".to_owned(),
        "edge-a".to_owned(),
    ];
    production.routes[0].distance_mm = Some(67_000);
    production.road_source = Some(ProductionRoadSource {
        pbf_sha256: "pbf-sha".to_owned(),
        pbf_bytes: 100,
        pbf_url: "https://example.org/roads.pbf".to_owned(),
        replication_timestamp: "2026-09-09T00:00:00Z".to_owned(),
        footprint_sha256: "footprint-sha".to_owned(),
        buffer_degrees_e7: 200_000,
        extraction_version: "extract-v1".to_owned(),
        distance_version: "integer-v1".to_owned(),
        routing_profile_version: "michigan-freight-routing-v1".to_owned(),
        graph_sha256: "graph-sha".to_owned(),
    });
    production
        .merchant_handling_accounts
        .push(ProductionMerchantHandlingAccount {
            site_id: site.clone(),
            capacity_id: "handling-capacity".to_owned(),
            labor_unit_id: "labor".to_owned(),
            coefficients: vec![ProductionHandlingCoefficient {
                good_id: stock.good_id.clone(),
                unit_id: stock.unit_id.clone(),
                grams_per_unit: 10,
                hours_per_unit: 2,
            }],
            completed: Some(CompletedProductionMerchantHandling {
                period: 1,
                needed_hours: 12,
                used_hours: 6,
                handled_grams: 30,
                orders: vec![ProductionMerchantHandlingOrder {
                    order_id: "final-order".to_owned(),
                    kind: ProductionOutboundKind::LocalFinalDemand,
                    good_id: stock.good_id.clone(),
                    unit_id: stock.unit_id.clone(),
                    requested: 10,
                    feasible_quantity: 6,
                    handled_quantity: 3,
                    needed_hours: 12,
                    used_hours: 6,
                    remaining_unshipped: 7,
                }],
            }),
        });
    production
        .final_demand_accounts
        .push(ProductionFinalDemandAccount {
            demand_principal_id: "county-demand".to_owned(),
            county_geoid: "26163".to_owned(),
            good_id: stock.good_id.clone(),
            unit_id: stock.unit_id.clone(),
            good: stock.good,
            unit: stock.unit,
            ordered: 10,
            fulfilled: 3,
            outstanding: 7,
            retail_stock_on_hand: 7,
            retailer_site_ids: vec![site.clone()],
            orders: vec![ProductionFinalDemandOrder {
                order_id: "final-order".to_owned(),
                retailer_site_id: site,
                ordered: 10,
                fulfilled: 3,
                outstanding: 7,
            }],
            completed: Some(CompletedProductionFinalDemand {
                period: 1,
                opening_fulfilled: 0,
                newly_fulfilled: 3,
                closing_fulfilled: 3,
            }),
        });
    observation
}

#[test]
fn presentation_multisets_permute_without_changing_evidence_identity() {
    let before = full_disclosure();
    let mut permuted = before.clone();
    let rows = permuted.production.as_mut().unwrap();
    rows.sites.reverse();
    for site in &mut rows.sites {
        site.inventory.reverse();
        site.processes.reverse();
        for process in &mut site.processes {
            process.inputs.reverse();
            process.labor.reverse();
            for input in &mut process.inputs {
                input.supplier_site_ids.reverse();
            }
        }
    }
    rows.routes.reverse();
    for route in &mut rows.routes {
        route.stages.reverse();
        for stage in &mut route.stages {
            stage.capacity_ids.reverse();
        }
    }
    rows.freight.reverse();
    rows.physical_edges.reverse();
    rows.labor_accounts.reverse();
    rows.staffing_accounts.reverse();
    rows.freight_capacity_accounts.reverse();
    for account in &mut rows.freight_capacity_accounts {
        account.route_ids.reverse();
        account.merchant_site_ids.reverse();
        if let Some(completed) = &mut account.completed {
            completed.reservations.reverse();
            for reservation in &mut completed.reservations {
                reservation.orders.reverse();
            }
        }
    }
    rows.material_balance.as_mut().unwrap().rows.reverse();
    rows.provenance.reverse();
    assert_eq!(digest(&before), digest(&permuted));
}

#[test]
fn physical_path_repetition_vertex_order_and_event_sequence_remain_semantic() {
    let before = full_disclosure();
    for mutation in [
        |rows: &mut ProductionSnapshot| {
            rows.routes[0].physical_edge_ids.swap(0, 1);
        },
        |rows: &mut ProductionSnapshot| {
            rows.routes[0].physical_edge_ids.pop();
        },
        |rows: &mut ProductionSnapshot| {
            rows.physical_edges[0].shape_e7.reverse();
        },
        |rows: &mut ProductionSnapshot| {
            rows.events.reverse();
        },
    ] {
        let mut changed = before.clone();
        mutation(changed.production.as_mut().unwrap());
        assert_ne!(digest(&before), digest(&changed));
    }
}

fn scalar_changes(value: &Value, pointer: &str, changes: &mut Vec<(String, Value)>) {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                scalar_changes(
                    value,
                    &format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                    changes,
                );
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                scalar_changes(value, &format!("{pointer}/{index}"), changes);
            }
        }
        Value::String(value) => {
            let replacement = match value.as_str() {
                "Local" => "Staged".to_owned(),
                "Staged" => "Local".to_owned(),
                "Delivery" => "LocalFinalDemand".to_owned(),
                "LocalFinalDemand" => "Delivery".to_owned(),
                "Transport" => "MerchantHandling".to_owned(),
                "MerchantHandling" => "Transport".to_owned(),
                "Production" => "Retail".to_owned(),
                "Observed" => "Designed".to_owned(),
                "Designed" => "Observed".to_owned(),
                _ => format!("{value} changed"),
            };
            changes.push((pointer.to_owned(), Value::String(replacement)));
        }
        Value::Number(number) => {
            let replacement = number.as_i64().map_or_else(
                || Value::from(number.as_u64().unwrap() - 1),
                |n| Value::from(n.checked_add(1).unwrap()),
            );
            changes.push((pointer.to_owned(), replacement));
        }
        Value::Bool(value) => changes.push((pointer.to_owned(), Value::Bool(!value))),
        Value::Null => {}
    }
}

#[test]
fn every_disclosed_scalar_is_bound_or_refused_including_new_accounting_families() {
    let before = full_disclosure();
    let expected = digest(&before);
    let json = serde_json::to_value(&before).unwrap();
    let mut mutations = Vec::new();
    scalar_changes(&json["production"], "/production", &mut mutations);
    let mut checked = 0;
    for (pointer, replacement) in mutations {
        let mut changed = json.clone();
        *changed.pointer_mut(&pointer).unwrap() = replacement;
        // An unknown enum is rejected at the disclosure boundary before hashing.
        let Ok(changed) = serde_json::from_value::<ObserverEconomySnapshot>(changed) else {
            continue;
        };
        assert_ne!(
            changed.production_evidence_digest(),
            Ok(Some(expected)),
            "unbound scalar {pointer}"
        );
        checked += 1;
    }
    assert!(
        checked > 300,
        "the actual committed accounts and new DTO families were exercised"
    );
}

#[test]
fn scope_completed_zero_and_absent_preview_are_distinct() {
    let before = committed();
    for mutate in [
        |row: &mut ObserverEconomySnapshot| {
            row.campaign_id.push('x');
        },
        |row: &mut ObserverEconomySnapshot| {
            row.resolve_tick += 1;
        },
        |row: &mut ObserverEconomySnapshot| {
            row.foundation_digest.push('x');
        },
        |row: &mut ObserverEconomySnapshot| {
            row.tick_content_hash = None;
        },
        |row: &mut ObserverEconomySnapshot| {
            row.envelope_digest = None;
        },
        |row: &mut ObserverEconomySnapshot| {
            row.nominal_world_hash = None;
        },
        |row: &mut ObserverEconomySnapshot| {
            row.production.as_mut().unwrap().labor_accounts[0].completed = None;
        },
    ] {
        let mut changed = before.clone();
        mutate(&mut changed);
        assert_ne!(digest(&before), digest(&changed));
    }
    assert_ne!(digest(&published_observations()[0]), digest(&before));
    let mut preview = before;
    preview.visibility = ObserverVisibility::KnownPreview;
    assert_eq!(
        preview.production_evidence_digest(),
        Err(ProductionEvidenceError::InvalidIdentity)
    );
    preview.production = None;
    assert_eq!(preview.production_evidence_digest(), Ok(None));
}

#[test]
fn duplicate_principals_and_row_bounds_refuse_instead_of_acquiring_a_digest() {
    let before = full_disclosure();
    for mutate in [
        |rows: &mut ProductionSnapshot| {
            rows.sites.push(rows.sites[0].clone());
        },
        |rows: &mut ProductionSnapshot| {
            rows.physical_edges.push(rows.physical_edges[0].clone());
        },
        |rows: &mut ProductionSnapshot| {
            rows.freight_capacity_accounts
                .push(rows.freight_capacity_accounts[0].clone());
        },
        |rows: &mut ProductionSnapshot| {
            rows.merchant_handling_accounts
                .push(rows.merchant_handling_accounts[0].clone());
        },
        |rows: &mut ProductionSnapshot| {
            rows.final_demand_accounts
                .push(rows.final_demand_accounts[0].clone());
        },
    ] {
        let mut changed = before.clone();
        mutate(changed.production.as_mut().unwrap());
        assert_eq!(
            changed.production_evidence_digest(),
            Err(ProductionEvidenceError::InvalidIdentity)
        );
    }
    let mut bounded = before;
    bounded.production.as_mut().unwrap().routes =
        vec![bounded.production.as_ref().unwrap().routes[0].clone(); MAX_ROWS + 1];
    assert_eq!(
        bounded.production_evidence_digest(),
        Err(ProductionEvidenceError::Bound)
    );
}

#[test]
fn native_requests_larger_than_u64_grams_are_hashed_without_narrowing() {
    let mut observation = committed();
    let expected = digest(&observation);
    let request = &mut observation
        .production
        .as_mut()
        .unwrap()
        .freight_capacity_accounts[0]
        .completed
        .as_mut()
        .unwrap()
        .reservations[0]
        .orders[0];
    request.requested_grams = u128::from(u64::MAX) * u128::from(u64::MAX);
    assert_ne!(digest(&observation), expected);
    let mut writer = EvidenceWriter {
        hash: Sha256::new(),
        remaining: 2,
        bound: false,
    };
    assert!(writer.write_all(b"abc").is_err());
    assert!(writer.bound);
}
