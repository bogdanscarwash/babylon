use std::sync::OnceLock;

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{hypergraph_store::HypergraphStore, stable_state::StableGraphStateV1};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::{
    material_replay::{MaterialLaborV1, PreparedMaterialTickV3},
    material_staffing::StaffingCompositionV1,
    material_world::decode_material_receipts_v4,
    replay_session::ReplayCommitDispositionV1,
};
use serde_json::Value;

use super::*;
use crate::{
    material_envelope::CommittedMaterialTickEnvelopeV3,
    michigan_content::MichiganContentPresetV1,
    michigan_economy::digest_hex,
    michigan_material::MichiganDeliveryPresetV1,
    production_projection::{
        project_material_observation_v1, staffing::project_staffing_accounts_v1,
    },
    runtime::prepare_committed_tick_v2,
    CampaignId,
};

/// Exercises the existing engine, canonical envelope, publication and projector.
/// The commit callback is an in-memory sink; this is not live-Postgres evidence.
fn published_observations() -> &'static [ObserverEconomySnapshotV1] {
    static OBSERVATIONS: OnceLock<Vec<ObserverEconomySnapshotV1>> = OnceLock::new();
    OBSERVATIONS.get_or_init(|| {
        let preset = MichiganDeliveryPresetV1::Standard;
        let foundation = MichiganContentPresetV1::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let foundation_digest = foundation.digest();
        let MaterialLaborV1::Staffed(composition) = foundation.labor().clone() else {
            panic!("current Michigan foundation is staffed");
        };
        let mut session = foundation.into_session().unwrap();
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(293));
        let mut observation = ObserverEconomySnapshotV1 {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 0,
            foundation_digest: digest_hex(&foundation_digest),
            nominal_world_hash: None,
            tick_content_hash: None,
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: vec![],
            production: Some(
                project_material_observation_v1(
                    &crate::test_support::catalog(),
                    preset,
                    session.material(),
                    None,
                    &[],
                )
                .unwrap(),
            ),
        };
        observation.production.as_mut().unwrap().staffing_accounts = project_staffing_accounts_v1(
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
            let actions = OrderedPracticeActionBatchV1::empty(
                session.graph_session().session_identity().clone(),
                tick,
            )
            .unwrap();
            let opening = session.material().clone();
            let opening_graph = session.graph_session().stable_graph_state().unwrap();
            let prepared = session.prepare_advance(&actions).unwrap();
            let staffing = prepared_staffing(&composition, &opening_graph, &prepared);
            let identity = *prepared.identity();
            let receipt = decode_material_receipts_v4(prepared.material().receipt_bytes()).unwrap();
            let families = prepare_committed_tick_v2(prepared.graph_report())
                .unwrap()
                .into_material_families(identity.tick_content_hash())
                .unwrap();
            let envelope = CommittedMaterialTickEnvelopeV3::compose(
                campaign,
                &identity,
                families,
                prepared.material().register().canonical_bytes(),
                prepared.material().receipt_bytes(),
            )
            .unwrap();
            let (ack, _) = session
                .commit_prepared_and_publish(&mut sink, prepared, |_| {
                    Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
                })
                .unwrap();
            history.push((receipt, ack.receipt_digest()));
            observation.resolve_tick = ack.resolve_tick();
            observation.tick_content_hash = Some(digest_hex(ack.tick_content_hash().as_bytes()));
            observation.envelope_digest = Some(digest_hex(&envelope.digest()));
            observation.nominal_world_hash = Some(digest_hex(&ack.result_world_hash()));
            observation.production = Some(
                project_material_observation_v1(
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
    composition: &StaffingCompositionV1,
    opening: &StableGraphStateV1,
    prepared: &PreparedMaterialTickV3<HypergraphStore>,
) -> Vec<crate::ProductionStaffingAccountV1> {
    let report = prepared.graph_report();
    let events = report
        .successful_event_batch()
        .events()
        .iter()
        .map(|event| crate::stored_tick::StoredEventV2 {
            emitting_rule: event.emitting_rule().to_owned(),
            choice_receipt_ordinal: event
                .choice_receipt()
                .map(babylon_tick::choice_receipt::ChoiceReceiptRefV1::encounter_ordinal),
            event_type: event.event_type().to_owned(),
            fields: event.fields().to_vec(),
        })
        .collect::<Vec<_>>();
    project_staffing_accounts_v1(
        composition,
        report.result_stable_graph(),
        prepared.material().register(),
        Some(opening),
        &events,
    )
    .unwrap()
}

fn committed() -> ObserverEconomySnapshotV1 {
    published_observations()[1].clone()
}

fn digest(snapshot: &ObserverEconomySnapshotV1) -> ProductionEvidenceDigestV6 {
    snapshot.production_evidence_digest().unwrap().unwrap()
}

/// Add the disclosure families absent from the small regional engine fixture.
/// All state/receipt projections are tested at their authenticated seams; this
/// fixture tests that the public evidence encoder binds every disclosed field.
fn full_disclosure() -> ObserverEconomySnapshotV1 {
    use crate::*;
    let mut observation = committed();
    let production = observation.production.as_mut().unwrap();
    let site = production.sites[0].id.clone();
    let stock = production.sites[0].inventory[0].clone();
    production.physical_edges = vec![
        ProductionPhysicalEdgeV2 {
            id: "edge-a".to_owned(),
            shape_e7: vec![[-830_000_000, 420_000_000], [-830_001_000, 420_001_000]],
            distance_mm: 17_000,
        },
        ProductionPhysicalEdgeV2 {
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
    production.road_source = Some(ProductionRoadSourceV2 {
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
        .push(ProductionMerchantHandlingAccountV2 {
            site_id: site.clone(),
            capacity_id: "handling-capacity".to_owned(),
            labor_unit_id: "labor".to_owned(),
            coefficients: vec![ProductionHandlingCoefficientV2 {
                good_id: stock.good_id.clone(),
                unit_id: stock.unit_id.clone(),
                grams_per_unit: 10,
                hours_per_unit: 2,
            }],
            completed: Some(CompletedProductionMerchantHandlingV2 {
                period: 1,
                needed_hours: 12,
                used_hours: 6,
                handled_grams: 30,
                orders: vec![ProductionMerchantHandlingOrderV2 {
                    order_id: "final-order".to_owned(),
                    kind: ProductionOutboundKindV2::LocalFinalDemand,
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
        .push(ProductionFinalDemandAccountV2 {
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
            orders: vec![ProductionFinalDemandOrderV2 {
                order_id: "final-order".to_owned(),
                retailer_site_id: site,
                ordered: 10,
                fulfilled: 3,
                outstanding: 7,
            }],
            completed: Some(CompletedProductionFinalDemandV2 {
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
        |rows: &mut ProductionSnapshotV2| {
            rows.routes[0].physical_edge_ids.swap(0, 1);
        },
        |rows: &mut ProductionSnapshotV2| {
            rows.routes[0].physical_edge_ids.pop();
        },
        |rows: &mut ProductionSnapshotV2| {
            rows.physical_edges[0].shape_e7.reverse();
        },
        |rows: &mut ProductionSnapshotV2| {
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
        let Ok(changed) = serde_json::from_value::<ObserverEconomySnapshotV1>(changed) else {
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
        |row: &mut ObserverEconomySnapshotV1| {
            row.campaign_id.push('x');
        },
        |row: &mut ObserverEconomySnapshotV1| {
            row.resolve_tick += 1;
        },
        |row: &mut ObserverEconomySnapshotV1| {
            row.foundation_digest.push('x');
        },
        |row: &mut ObserverEconomySnapshotV1| {
            row.tick_content_hash = None;
        },
        |row: &mut ObserverEconomySnapshotV1| {
            row.envelope_digest = None;
        },
        |row: &mut ObserverEconomySnapshotV1| {
            row.nominal_world_hash = None;
        },
        |row: &mut ObserverEconomySnapshotV1| {
            row.production.as_mut().unwrap().labor_accounts[0].completed = None;
        },
    ] {
        let mut changed = before.clone();
        mutate(&mut changed);
        assert_ne!(digest(&before), digest(&changed));
    }
    assert_ne!(digest(&published_observations()[0]), digest(&before));
    let mut preview = before;
    preview.visibility = ObserverVisibilityV1::KnownPreview;
    assert_eq!(
        preview.production_evidence_digest(),
        Err(ProductionEvidenceErrorV6::InvalidIdentity)
    );
    preview.production = None;
    assert_eq!(preview.production_evidence_digest(), Ok(None));
}

#[test]
fn duplicate_principals_and_row_bounds_refuse_instead_of_acquiring_a_digest() {
    let before = full_disclosure();
    for mutate in [
        |rows: &mut ProductionSnapshotV2| {
            rows.sites.push(rows.sites[0].clone());
        },
        |rows: &mut ProductionSnapshotV2| {
            rows.physical_edges.push(rows.physical_edges[0].clone());
        },
        |rows: &mut ProductionSnapshotV2| {
            rows.freight_capacity_accounts
                .push(rows.freight_capacity_accounts[0].clone());
        },
        |rows: &mut ProductionSnapshotV2| {
            rows.merchant_handling_accounts
                .push(rows.merchant_handling_accounts[0].clone());
        },
        |rows: &mut ProductionSnapshotV2| {
            rows.final_demand_accounts
                .push(rows.final_demand_accounts[0].clone());
        },
    ] {
        let mut changed = before.clone();
        mutate(changed.production.as_mut().unwrap());
        assert_eq!(
            changed.production_evidence_digest(),
            Err(ProductionEvidenceErrorV6::InvalidIdentity)
        );
    }
    let mut bounded = before;
    bounded.production.as_mut().unwrap().routes =
        vec![bounded.production.as_ref().unwrap().routes[0].clone(); MAX_ROWS + 1];
    assert_eq!(
        bounded.production_evidence_digest(),
        Err(ProductionEvidenceErrorV6::Bound)
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
