use std::{process::Command, sync::OnceLock};

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{hypergraph_store::HypergraphStore, stable_state::StableGraphStateV1};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::{
    material_replay::{MaterialLaborV1, PreparedMaterialTickV3},
    material_staffing::StaffingCompositionV1,
    material_world::decode_material_receipts_v3,
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
            let receipt = decode_material_receipts_v3(prepared.material().receipt_bytes()).unwrap();
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

fn reverse_unordered(snapshot: &mut ProductionSnapshotV1) {
    snapshot.sites.reverse();
    snapshot.labor_accounts.reverse();
    snapshot.staffing_accounts.reverse();
    if let Some(balance) = &mut snapshot.material_balance {
        balance.rows.reverse();
    }
    snapshot.observed_contexts.reverse();
    snapshot.process_attributions.reverse();
    snapshot.routes.reverse();
    for route in &mut snapshot.routes {
        route.corridor_legs.reverse();
    }
    snapshot.freight_capacity_accounts.reverse();
    for account in &mut snapshot.freight_capacity_accounts {
        account.route_ids.reverse();
        if let Some(completed) = &mut account.completed {
            completed.reservations.reverse();
            for reservation in &mut completed.reservations {
                reservation.orders.reverse();
            }
        }
    }
    snapshot.freight.reverse();
    snapshot.provenance.reverse();
    for site in &mut snapshot.sites {
        site.inventory.reverse();
        site.inputs.reverse();
        site.labor.reverse();
        for input in &mut site.inputs {
            input.supplier_site_ids.reverse();
        }
    }
    for event in &mut snapshot.events {
        event.subject_site_ids.reverse();
    }
}

#[test]
fn published_replay_and_insertion_order_twins_have_the_same_evidence() {
    for original in published_observations() {
        let before = original.clone();
        let mut twin = original.clone();
        reverse_unordered(twin.production.as_mut().unwrap());
        assert_eq!(
            original.production_evidence_digest(),
            twin.production_evidence_digest()
        );
        let decoded: ObserverEconomySnapshotV1 =
            serde_json::from_slice(&serde_json::to_vec(original).unwrap()).unwrap();
        assert_eq!(
            original.production_evidence_digest(),
            decoded.production_evidence_digest()
        );
        assert_eq!(
            original, &before,
            "canonicalization cannot mutate the observation"
        );
    }
    let digests: std::collections::HashSet<_> = published_observations()
        .iter()
        .map(ObserverEconomySnapshotV1::production_evidence_digest)
        .collect();
    assert_eq!(
        digests.len(),
        4,
        "each published scope has a distinct identity"
    );
}

#[test]
fn nested_rows_exact_units_and_duplicate_multiplicity_are_preserved() {
    let mut original = committed();
    let site = &mut original.production.as_mut().unwrap().sites[0];
    let mut stock = site.inventory[0].clone();
    stock.unit_id.push_str("-different-unit");
    stock.quantity = u64::MAX;
    site.inventory.push(stock);
    let mut input = site.inputs[0].clone();
    input.good_id.push_str("-different-good");
    input.supplier_site_ids = vec!["supplier-z".to_owned(), "supplier-a".to_owned()];
    site.inputs.push(input);
    let mut labor = site.labor[0].clone();
    labor.unit.push_str("-different-unit");
    site.labor.push(labor);
    let mut twin = original.clone();
    reverse_unordered(twin.production.as_mut().unwrap());
    assert_eq!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );

    let site = &mut twin.production.as_mut().unwrap().sites[0];
    site.inventory.push(site.inventory[0].clone());
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest(),
        "sorting must not discard duplicate rows"
    );
}

#[test]
fn scope_and_committed_identity_changes_are_bound() {
    let original = committed();
    let mut changed = serde_json::to_value(&original).unwrap();
    for field in [
        "campaign_id",
        "foundation_digest",
        "tick_content_hash",
        "envelope_digest",
        "nominal_world_hash",
    ] {
        changed[field] = Value::String("different-identity".to_owned());
        let twin: ObserverEconomySnapshotV1 = serde_json::from_value(changed.clone()).unwrap();
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest(),
            "{field}"
        );
        changed = serde_json::to_value(&original).unwrap();
    }
    let mut twin = original.clone();
    twin.resolve_tick += 1;
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
    twin = original.clone();
    twin.visibility = ObserverVisibilityV1::KnownPreview;
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
}

#[test]
fn meaningful_event_order_remains_bound() {
    let original = committed();
    let mut twin = original.clone();
    let production = twin.production.as_mut().unwrap();
    assert!(production.events.len() > 1);
    production.events.reverse();
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
}

#[test]
fn missing_production_and_foundation_absence_do_not_masquerade_as_zero() {
    let mut known = committed();
    known.visibility = ObserverVisibilityV1::KnownPreview;
    known.production = None;
    known.nominal_world_hash = None;
    assert_eq!(known.production_evidence_digest(), None);

    let foundation = &published_observations()[0];
    let mut invented_zero = foundation.clone();
    invented_zero.production.as_mut().unwrap().sites[0].produced_batches = Some(0);
    assert_ne!(
        foundation.production_evidence_digest(),
        invented_zero.production_evidence_digest()
    );
    let mut invented_labor = foundation.clone();
    invented_labor.production.as_mut().unwrap().labor_accounts[0].completed =
        Some(crate::CompletedProductionLaborV1 {
            period: 0,
            opening: 0,
            planned: 0,
            used: 0,
            unused: 0,
        });
    assert_ne!(
        foundation.production_evidence_digest(),
        invented_labor.production_evidence_digest(),
        "an absent completed account is not a zero account"
    );
    let mut invented_balance = foundation.clone();
    invented_balance
        .production
        .as_mut()
        .unwrap()
        .material_balance = Some(crate::CompletedMaterialBalanceV1 {
        period: 0,
        rows: Vec::new(),
    });
    assert_ne!(
        foundation.production_evidence_digest(),
        invented_balance.production_evidence_digest(),
        "an absent material account is not an empty completed account"
    );
    let mut invented_identity = foundation.clone();
    invented_identity.tick_content_hash = Some(String::new());
    assert_ne!(
        foundation.production_evidence_digest(),
        invented_identity.production_evidence_digest()
    );
}

/// Independent traversal through the public serde schema catches a newly added
/// scalar field that the fixed evidence encoder accidentally omits.
fn scalar_paths(value: &Value, prefix: &str, result: &mut Vec<String>) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                scalar_paths(value, &format!("{prefix}/{key}"), result);
            }
        }
        Value::Array(rows) => {
            for (index, value) in rows.iter().enumerate() {
                scalar_paths(value, &format!("{prefix}/{index}"), result);
            }
        }
        _ => result.push(prefix.to_owned()),
    }
}

fn changed_production_scalar(field: &Value, path: &str) -> Value {
    if path.ends_with("/delivery_evidence") && field.is_null() {
        let snapshot = &published_observations()[2];
        let delivery = snapshot
            .production
            .as_ref()
            .unwrap()
            .events
            .iter()
            .find_map(|event| event.delivery_evidence.as_ref())
            .unwrap();
        return serde_json::to_value(delivery).unwrap();
    }
    if path.ends_with("/delivery_evidence/stage") {
        return Value::String(
            if field.as_str() == Some("Arrival") {
                "Delivery"
            } else {
                "Arrival"
            }
            .to_owned(),
        );
    }
    match field {
        Value::String(text) => Value::String(format!("{text}\0altered")),
        Value::Number(number) => Value::from(number.as_u64().unwrap() + 1),
        Value::Null => Value::from(0),
        other => panic!("unexpected scalar {other:?}"),
    }
}

#[test]
fn every_disclosed_production_scalar_including_catalog_provenance_is_bound() {
    let original = committed();
    let value = serde_json::to_value(original.production.as_ref().unwrap()).unwrap();
    let mut paths = Vec::new();
    scalar_paths(&value, "", &mut paths);
    assert!(paths.iter().any(|path| path.starts_with("/freight/")));
    assert!(paths
        .iter()
        .any(|path| path.starts_with("/labor_accounts/")));
    assert!(paths.iter().any(|path| path.starts_with("/provenance/")));
    assert!(paths
        .iter()
        .any(|path| path.starts_with("/staffing_accounts/")));
    assert!(paths
        .iter()
        .any(|path| path.starts_with("/material_balance/rows/")));
    for path in paths {
        let mut changed = value.clone();
        let field = changed.pointer_mut(&path).unwrap();
        *field = changed_production_scalar(field, &path);
        let mut twin = original.clone();
        twin.production = Some(serde_json::from_value(changed).unwrap());
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest(),
            "{path}"
        );
    }
}

#[test]
fn presentation_identity_does_not_alias_world_or_envelope_identity() {
    let snapshot = committed();
    let digest = snapshot.production_evidence_digest().unwrap();
    assert_eq!(digest.as_bytes().len(), 32);
    let hex = digest.to_hex();
    assert_eq!(hex.len(), 64);
    assert!(hex
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    assert_ne!(Some(&hex), snapshot.nominal_world_hash.as_ref());
    assert_ne!(Some(&hex), snapshot.envelope_digest.as_ref());
    assert_ne!(Some(&hex), snapshot.tick_content_hash.as_ref());
}

#[test]
fn digest_is_identical_in_two_fresh_processes() {
    const ENV: &str = "BABYLON_PRODUCTION_EVIDENCE_PROCESS";
    const MARKER: &str = "production-observation-evidence:";
    if let Some(order) = std::env::var_os(ENV) {
        let mut observation = contextual_observation();
        if order == "reverse" {
            reverse_unordered(observation.production.as_mut().unwrap());
        }
        println!(
            "{MARKER}{}",
            observation.production_evidence_digest().unwrap().to_hex()
        );
        return;
    }
    let executable = std::env::current_exe().unwrap();
    let run_child = |order| {
        let output = Command::new(&executable)
            .args([
                "--exact",
                "production_evidence::tests::digest_is_identical_in_two_fresh_processes",
                "--nocapture",
            ])
            .env(ENV, order)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .find_map(|line| line.strip_prefix(MARKER).map(str::to_owned))
            .unwrap()
    };
    let forward = run_child("forward");
    assert_eq!(forward, run_child("reverse"));
    assert_eq!(
        forward,
        contextual_observation()
            .production_evidence_digest()
            .unwrap()
            .to_hex()
    );
}

// Reuse the current staffed campaign reading to isolate the observed
// context presentation family; no source totals allocate modeled workers.
fn contextual_observation() -> ObserverEconomySnapshotV1 {
    let mut snapshot = published_observations()[2].clone();
    let admitted = crate::michigan_content::MichiganContentPresetV1::FourWeekStandardV6
        .admitted(&crate::test_support::catalog())
        .unwrap();
    snapshot.foundation_digest = digest_hex(&admitted.digest());
    crate::production_projection::context::attach_observed_context_v1(
        &admitted,
        ObserverVisibilityV1::FullObserver,
        snapshot.production.as_mut().unwrap(),
    )
    .unwrap();
    snapshot
}

#[test]
fn every_attribution_and_observed_context_scalar_is_bound() {
    let original = contextual_observation();
    let value = serde_json::to_value(original.production.as_ref().unwrap()).unwrap();
    let mut paths = Vec::new();
    for field in ["observed_contexts", "process_attributions"] {
        scalar_paths(&value[field], &format!("/{field}"), &mut paths);
    }
    assert!(paths.iter().any(|path| path.ends_with("/subject/scenario")));
    assert!(paths
        .iter()
        .any(|path| path.ends_with("/cohort_subject/local_name")));
    assert!(paths.iter().any(|path| path.ends_with("/source_sha256")));
    for path in paths {
        let mut changed = value.clone();
        let field = changed.pointer_mut(&path).unwrap();
        *field = if path.ends_with("/evidence_class") {
            Value::String(
                if field.as_str() == Some("Observed") {
                    "Designed"
                } else {
                    "Observed"
                }
                .to_owned(),
            )
        } else {
            match &*field {
                Value::String(text) => Value::String(format!("{text} altered")),
                Value::Number(number) => Value::from(number.as_u64().unwrap() + 1),
                Value::Null => Value::from(0),
                other => panic!("unexpected context scalar {other:?}"),
            }
        };
        let mut twin = original.clone();
        twin.production = Some(serde_json::from_value(changed).unwrap());
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest(),
            "{path}"
        );
    }
}

#[test]
fn context_order_is_irrelevant_but_multiplicity_and_unknown_values_remain_distinct() {
    let original = contextual_observation();
    let mut twin = original.clone();
    reverse_unordered(twin.production.as_mut().unwrap());
    assert_eq!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
    let decoded: ObserverEconomySnapshotV1 =
        serde_json::from_slice(&serde_json::to_vec(&original).unwrap()).unwrap();
    assert_eq!(
        decoded.production_evidence_digest(),
        original.production_evidence_digest()
    );
    let rows = twin.production.as_mut().unwrap();
    rows.observed_contexts
        .push(rows.observed_contexts[0].clone());
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
    twin = original.clone();
    let rows = twin.production.as_mut().unwrap();
    rows.process_attributions
        .push(rows.process_attributions[0].clone());
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
    twin = original.clone();
    twin.production.as_mut().unwrap().observed_contexts[0].annual_avg_emplvl = None;
    let absent_digest = twin.production_evidence_digest();
    assert_ne!(original.production_evidence_digest(), absent_digest);
    twin.production.as_mut().unwrap().observed_contexts[0].annual_avg_emplvl = Some(0);
    assert_ne!(twin.production_evidence_digest(), absent_digest);
}

#[test]
fn every_typed_delivery_field_and_stage_is_bound_without_merging_receipts() {
    let original = published_observations()[2].clone();
    let rows = original.production.as_ref().unwrap();
    let stages: std::collections::BTreeSet<_> = rows
        .events
        .iter()
        .filter_map(|event| event.delivery_evidence.as_ref().map(|row| row.stage))
        .collect();
    assert_eq!(
        stages,
        [
            crate::ProductionDeliveryStageV1::Arrival,
            crate::ProductionDeliveryStageV1::Delivery,
            crate::ProductionDeliveryStageV1::QuantityRealization
        ]
        .into_iter()
        .collect()
    );
    let value = serde_json::to_value(rows).unwrap();
    let mut paths = Vec::new();
    scalar_paths(&value["events"], "/events", &mut paths);
    for path in paths
        .iter()
        .filter(|path| path.contains("/delivery_evidence"))
    {
        let mut changed = value.clone();
        let field = changed.pointer_mut(path).unwrap();
        *field = changed_production_scalar(field, path);
        let mut twin = original.clone();
        twin.production = Some(serde_json::from_value(changed).unwrap());
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest(),
            "{path}"
        );
    }
    let mut duplicated = original.clone();
    let rows = duplicated.production.as_mut().unwrap();
    rows.events.push(
        rows.events
            .iter()
            .find(|event| event.delivery_evidence.is_some())
            .unwrap()
            .clone(),
    );
    assert_ne!(
        original.production_evidence_digest(),
        duplicated.production_evidence_digest()
    );
    let mut absent = original.clone();
    let event = absent
        .production
        .as_mut()
        .unwrap()
        .events
        .iter_mut()
        .find(|event| event.delivery_evidence.is_some())
        .unwrap();
    event.delivery_evidence = None;
    assert_ne!(
        original.production_evidence_digest(),
        absent.production_evidence_digest()
    );
}

#[test]
fn material_balance_row_multiplicity_and_presence_are_bound() {
    let original = committed();
    let mut twin = original.clone();
    let balance = twin
        .production
        .as_mut()
        .unwrap()
        .material_balance
        .as_mut()
        .unwrap();
    balance.rows.push(balance.rows[0].clone());
    assert_ne!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
    let mut absent = original.clone();
    absent.production.as_mut().unwrap().material_balance = None;
    assert_ne!(
        original.production_evidence_digest(),
        absent.production_evidence_digest()
    );
}

fn fixture_wire(vector: &Value) -> Vec<u8> {
    let hex = vector["canonical_hex"].as_str().unwrap();
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn historical_v3_wire_vector_keeps_its_exact_bytes_without_a_live_encoder() {
    let vector: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/fixtures/production_evidence_v3.json"
    )))
    .unwrap();
    let wire = fixture_wire(&vector);
    assert_eq!(vector["schema_version"], 3);
    assert_eq!(wire.len(), 442);
    assert_eq!(vector["byte_length"], 442);
    assert!(wire.starts_with(b"babylon.production-observation-evidence.v3\0\0\0\0\x03"));
    let expected = "4e5e6efd36f6e9ec5e052cf95815e4f4f33dc6a8a49eb15df5448fb1ecc140fe";
    assert_eq!(vector["sha256"], expected);
    assert_eq!(digest_hex(&Sha256::digest(&wire)), expected);
    assert!(vector["snapshot"]["production"]
        .get("staffing_accounts")
        .is_none());
    assert!(
        serde_json::from_value::<ObserverEconomySnapshotV1>(vector["snapshot"].clone()).is_err()
    );
}

#[test]
fn historical_v4_wire_vectors_keep_exact_bytes_without_a_live_encoder() {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/fixtures/production_evidence_v4.json"
    )))
    .unwrap();
    assert_eq!(fixture["schema_version"], 4);
    let vectors = fixture["vectors"].as_array().unwrap();
    assert_eq!(vectors.len(), 2);
    for vector in vectors {
        assert!(
            serde_json::from_value::<ObserverEconomySnapshotV1>(vector["snapshot"].clone())
                .is_err()
        );
        let wire = fixture_wire(vector);
        assert!(wire.starts_with(b"babylon.production-observation-evidence.v4\0\0\0\0\x04"));
        assert_eq!(vector["byte_length"], wire.len());
        let expected = vector["sha256"].as_str().unwrap();
        assert_eq!(digest_hex(&Sha256::digest(&wire)), expected);
    }
}

#[test]
fn staffing_shape_is_required_and_unknown_fields_are_refused() {
    let original = serde_json::to_value(staffing_observation()).unwrap();
    let mut missing = original.clone();
    missing["production"]
        .as_object_mut()
        .unwrap()
        .remove("staffing_accounts");
    assert!(serde_json::from_value::<ObserverEconomySnapshotV1>(missing).is_err());
    for pointer in [
        "/production/staffing_accounts/0",
        "/production/staffing_accounts/0/subject",
        "/production/staffing_accounts/0/completed",
    ] {
        let mut changed = original.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unsupported".into(), Value::Bool(true));
        assert!(
            serde_json::from_value::<ObserverEconomySnapshotV1>(changed).is_err(),
            "{pointer}"
        );
    }
}

fn staffing_observation() -> ObserverEconomySnapshotV1 {
    use crate::production_observation::{
        CompletedProductionStaffingV1, ProductionStaffingAccountV1, ProductionStaffingSubjectV1,
    };
    let mut value = committed();
    value.production.as_mut().unwrap().staffing_accounts = vec![ProductionStaffingAccountV1 {
        pool_id: "pool-a".into(),
        site_id: "site-a".into(),
        unit_id: "labor-hour".into(),
        subject: ProductionStaffingSubjectV1 {
            scenario: "fixture/workforce".into(),
            local_name: "workers-a".into(),
        },
        hours_per_person: 40,
        labor_force: 4,
        employed: 2,
        reserve: 2,
        previous_unretained_hours: 80,
        next_opening_period: 2,
        next_opening_hours: 80,
        completed: Some(CompletedProductionStaffingV1 {
            period: 1,
            opening_employed: 4,
            opening_reserve: 0,
            previous_unretained_hours: 40,
            current_unretained_hours: 80,
            retained_hours: 80,
            target_employed: 2,
            hires: 0,
            separations: 2,
        }),
    }];
    value
}

#[test]
fn every_staffing_scalar_is_bound() {
    let original = staffing_observation();
    let value = serde_json::to_value(original.production.as_ref().unwrap()).unwrap();
    let mut paths = Vec::new();
    scalar_paths(
        &value["staffing_accounts"],
        "/staffing_accounts",
        &mut paths,
    );
    assert_eq!(paths.len(), 21);
    for path in paths {
        let mut changed = value.clone();
        let field = changed.pointer_mut(&path).unwrap();
        *field = changed_production_scalar(field, &path);
        let mut twin = original.clone();
        twin.production = Some(serde_json::from_value(changed).unwrap());
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest(),
            "{path}"
        );
    }
}

#[test]
fn staffing_order_multiplicity_and_completed_absence_are_distinct() {
    let mut original = staffing_observation();
    let rows = &mut original.production.as_mut().unwrap().staffing_accounts;
    let mut second = rows[0].clone();
    second.pool_id = "pool-b".into();
    second.completed = None;
    rows.push(second);
    let mut reordered = original.clone();
    reordered
        .production
        .as_mut()
        .unwrap()
        .staffing_accounts
        .reverse();
    assert_eq!(
        original.production_evidence_digest(),
        reordered.production_evidence_digest()
    );
    let mut duplicated = original.clone();
    let rows = &mut duplicated.production.as_mut().unwrap().staffing_accounts;
    rows.push(rows[0].clone());
    assert_ne!(
        original.production_evidence_digest(),
        duplicated.production_evidence_digest()
    );
    let mut absent = original.clone();
    absent.production.as_mut().unwrap().staffing_accounts[0].completed = None;
    assert_ne!(
        original.production_evidence_digest(),
        absent.production_evidence_digest()
    );
    let mut zero = absent.clone();
    zero.production.as_mut().unwrap().staffing_accounts[0].completed = Some(
        crate::production_observation::CompletedProductionStaffingV1 {
            period: 0,
            opening_employed: 0,
            opening_reserve: 0,
            previous_unretained_hours: 0,
            current_unretained_hours: 0,
            retained_hours: 0,
            target_employed: 0,
            hires: 0,
            separations: 0,
        },
    );
    assert_ne!(
        zero.production_evidence_digest(),
        absent.production_evidence_digest()
    );
}

/// This V5 vector extends the independently authored historical foundation
/// vector. Its expected hash was calculated with an independent Python encoder
/// using length-prefixed UTF-8 and unsigned big-endian integers, including >2^53.
fn freight_evidence_vector() -> ObserverEconomySnapshotV1 {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/fixtures/production_evidence_v4.json"
    )))
    .unwrap();
    let mut value = fixture["vectors"][1]["snapshot"].clone();
    value["production"]["freight_capacity_accounts"] = serde_json::json!([{
        "corridor_id": "c", "corridor_label": "Regional freight", "unit_id": "u", "unit": "kg",
        "route_ids": ["r2", "r1"], "next_opening_period": 2, "next_opening_available": 40,
        "completed": {"period": 1, "reservations": [
            {"reservation_period": 2, "opening_available": 160, "newly_reserved": 120, "remaining_available": 40,
             "orders": [{"order_id": "a", "route_id": "r1", "good_id": "g1", "unit_id": "u", "requested": 600, "dispatched": 120, "remaining_unshipped": 480}]},
            {"reservation_period": 1, "opening_available": 9_007_199_254_740_993_u64, "newly_reserved": 160, "remaining_available": 9_007_199_254_740_833_u64,
             "orders": [
                {"order_id": "z", "route_id": "r2", "good_id": "g2", "unit_id": "u", "requested": 200, "dispatched": 40, "remaining_unshipped": 160},
                {"order_id": "a", "route_id": "r1", "good_id": "g1", "unit_id": "u", "requested": 600, "dispatched": 120, "remaining_unshipped": 480}
             ]}
        ]}
    }]);
    serde_json::from_value(value).unwrap()
}

#[test]
fn v5_independent_digest_vector_binds_shared_capacity_and_future_reservations() {
    let original = freight_evidence_vector();
    assert_eq!(
        original.production_evidence_digest().unwrap().to_hex(),
        "a830b341a08dad4e4efe6ea360f7725831c191a802c1e71f30ca03d4021b7e62"
    );
    let mut twin = original.clone();
    reverse_unordered(twin.production.as_mut().unwrap());
    assert_eq!(
        original.production_evidence_digest(),
        twin.production_evidence_digest()
    );
    let value = serde_json::to_value(original.production.as_ref().unwrap()).unwrap();
    let mut paths = Vec::new();
    scalar_paths(
        &value["freight_capacity_accounts"],
        "/freight_capacity_accounts",
        &mut paths,
    );
    assert!(paths
        .iter()
        .any(|path| path.ends_with("/reservation_period")));
    assert!(paths
        .iter()
        .any(|path| path.ends_with("/remaining_unshipped")));
    for path in paths {
        let mut changed = value.clone();
        let field = changed.pointer_mut(&path).unwrap();
        *field = changed_production_scalar(field, &path);
        let mut twin = original.clone();
        twin.production = Some(serde_json::from_value(changed).unwrap());
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest(),
            "{path}"
        );
    }
}

#[test]
fn freight_account_multiplicity_and_completed_zero_are_hash_distinct() {
    let original = freight_evidence_vector();
    for duplicate in [0, 1, 2, 3] {
        let mut twin = original.clone();
        let accounts = &mut twin.production.as_mut().unwrap().freight_capacity_accounts;
        match duplicate {
            0 => accounts.push(accounts[0].clone()),
            1 => {
                let duplicate_route = accounts[0].route_ids[0].clone();
                accounts[0].route_ids.push(duplicate_route);
            }
            2 => {
                let rows = &mut accounts[0].completed.as_mut().unwrap().reservations;
                rows.push(rows[0].clone());
            }
            _ => {
                let rows = &mut accounts[0].completed.as_mut().unwrap().reservations[0].orders;
                rows.push(rows[0].clone());
            }
        }
        assert_ne!(
            original.production_evidence_digest(),
            twin.production_evidence_digest()
        );
    }
    let mut absent = original.clone();
    absent
        .production
        .as_mut()
        .unwrap()
        .freight_capacity_accounts[0]
        .completed = None;
    let mut zero = absent.clone();
    zero.production.as_mut().unwrap().freight_capacity_accounts[0].completed =
        Some(crate::CompletedProductionFreightCapacityV1 {
            period: 1,
            reservations: vec![crate::ProductionFreightReservationV1 {
                reservation_period: 1,
                opening_available: 0,
                newly_reserved: 0,
                remaining_available: 0,
                orders: vec![],
            }],
        });
    assert_ne!(
        absent.production_evidence_digest(),
        zero.production_evidence_digest()
    );
}

#[test]
fn freight_account_shape_is_required_and_unknown_fields_are_refused() {
    let original = serde_json::to_value(freight_evidence_vector()).unwrap();
    let mut missing = original.clone();
    missing["production"]
        .as_object_mut()
        .unwrap()
        .remove("freight_capacity_accounts");
    assert!(serde_json::from_value::<ObserverEconomySnapshotV1>(missing).is_err());
    for pointer in [
        "/production/freight_capacity_accounts/0",
        "/production/freight_capacity_accounts/0/completed",
        "/production/freight_capacity_accounts/0/completed/reservations/0",
        "/production/freight_capacity_accounts/0/completed/reservations/0/orders/0",
    ] {
        let mut changed = original.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unsupported".into(), Value::Bool(true));
        assert!(
            serde_json::from_value::<ObserverEconomySnapshotV1>(changed).is_err(),
            "{pointer}"
        );
    }
}
