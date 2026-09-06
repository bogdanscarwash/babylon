use std::collections::BTreeSet;

use super::*;
use crate::michigan_material::{
    michigan_material_catalog_v1, michigan_material_foundation_v1, MichiganDeliveryPresetV1,
};
use babylon_material_circuit::{advance_material_circuit_v2, MaterialCircuitTransitionV2};

fn rebuild(
    bundle: &SectorBundleV1,
    rows: &MaterialCircuitStateV2,
) -> Result<SectorBundleV1, SectorBundleErrorV1> {
    SectorBundleV1::from_parts(
        bundle.owner.clone(),
        bundle.sources.clone(),
        bundle.goods.clone(),
        bundle.processes.clone(),
        bundle.labor_unit,
        rows,
    )
}

fn bundles() -> Vec<SectorBundleV1> {
    michigan_sector_bundles_v1().unwrap().to_vec()
}

fn macomb(values: &[SectorBundleV1]) -> usize {
    values
        .iter()
        .position(|bundle| bundle.owner.county_geoid == "26099")
        .unwrap()
}

fn trace(mut state: MaterialCircuitStateV2, weeks: usize) -> Vec<MaterialCircuitTransitionV2> {
    (0..weeks)
        .map(|_| {
            let result = advance_material_circuit_v2(&state).unwrap();
            state = result.state.clone();
            result
        })
        .collect()
}

#[test]
fn four_nonempty_bundles_own_five_processes_without_merging_wayne_resources() {
    let bundles = michigan_sector_bundles_v1().unwrap();
    assert_eq!(bundles.len(), 4);
    assert_eq!(
        bundles
            .iter()
            .map(|bundle| bundle.processes.len())
            .sum::<usize>(),
        5
    );
    assert_eq!(
        bundles
            .iter()
            .map(|bundle| bundle.owner.subject.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        4
    );
    let wayne = bundles
        .iter()
        .find(|bundle| bundle.owner.county_geoid == "26163")
        .unwrap();
    assert_eq!(wayne.processes.len(), 2);
    assert_eq!(wayne.rows.site_logistics_nodes.len(), 2);
    for week in 1..=HORIZON_TICKS {
        let budgets: BTreeSet<_> = wayne
            .rows
            .labor
            .iter()
            .filter(|row| row.week == week)
            .map(|row| row.available)
            .collect();
        assert_eq!(budgets, BTreeSet::from([16, 80]));
    }
    for bundle in bundles {
        assert_eq!(bundle.owner.sector_code, "31-33");
        assert_eq!(
            bundle.production_evidence_class(),
            crate::ArchiveEvidenceClassV1::Designed
        );
        assert_eq!(bundle.horizon_ticks(), 16);
        assert_eq!(
            SectorBundleV1::decode(bundle.canonical_bytes(), bundle.sha256()).unwrap(),
            *bundle
        );
    }
}

#[test]
fn compiled_rows_equal_both_existing_physical_foundations_byte_for_byte() {
    for preset in [
        MichiganDeliveryPresetV1::Standard,
        MichiganDeliveryPresetV1::Delayed,
    ] {
        let actual = compile_sector_bundles_v1(&bundles(), preset).unwrap();
        let legacy = michigan_material_foundation_v1(preset).unwrap();
        assert_eq!(actual, legacy);
        assert_eq!(
            encode_material_circuit_state_v2(&actual).unwrap(),
            encode_material_circuit_state_v2(&legacy).unwrap()
        );
    }
}

#[test]
fn row_and_bundle_insertion_order_cannot_change_canonical_identity_or_execution() {
    let mut reordered = bundles();
    for bundle in &mut reordered {
        let original = bundle.clone();
        let mut rows = original.rows.clone();
        rows.process_outputs.reverse();
        rows.input_coefficients.reverse();
        rows.labor_coefficients.reverse();
        rows.site_logistics_nodes.reverse();
        rows.inventory.reverse();
        rows.capacities.reverse();
        rows.labor.reverse();
        rows.production_commitments.reverse();
        let mut goods = original.goods.clone();
        goods.reverse();
        let mut processes = original.processes.clone();
        processes.reverse();
        *bundle = SectorBundleV1::from_parts(
            original.owner.clone(),
            original.sources.clone(),
            goods,
            processes,
            original.labor_unit,
            &rows,
        )
        .unwrap();
        assert_eq!(*bundle, original);
    }
    reordered.reverse();
    let expected =
        compile_sector_bundles_v1(&bundles(), MichiganDeliveryPresetV1::Standard).unwrap();
    let actual = compile_sector_bundles_v1(&reordered, MichiganDeliveryPresetV1::Standard).unwrap();
    assert_eq!(expected, actual);
    assert_eq!(trace(expected, 4), trace(actual, 4));
}

#[test]
fn codec_refuses_wrong_expected_digest_unknown_versions_truncation_and_trailing_bytes() {
    let original = bundles().remove(0);
    let bytes = original.canonical_bytes();
    assert_eq!(&bytes[..BUNDLE_DOMAIN.len()], b"babylon.sector-bundle.v1\0");
    assert_eq!(
        &bytes[BUNDLE_DOMAIN.len()..BUNDLE_DOMAIN.len() + 2],
        &[0, 1]
    );
    assert_eq!(
        SectorBundleV1::decode(bytes, [0; 32]),
        Err(SectorBundleErrorV1::Digest)
    );
    let mut changed = bytes.to_vec();
    changed[BUNDLE_DOMAIN.len() + 1] = 2;
    assert_eq!(
        SectorBundleV1::decode(&changed, sha256_of(&changed)),
        Err(SectorBundleErrorV1::WireVersion)
    );
    changed[0] ^= 1;
    assert_eq!(
        SectorBundleV1::decode(&changed, sha256_of(&changed)),
        Err(SectorBundleErrorV1::WireDomain)
    );
    for length in [0, BUNDLE_DOMAIN.len(), bytes.len() - 1] {
        let truncated = &bytes[..length];
        assert_eq!(
            SectorBundleV1::decode(truncated, sha256_of(truncated)),
            Err(SectorBundleErrorV1::WireTruncated)
        );
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert_eq!(
        SectorBundleV1::decode(&trailing, sha256_of(&trailing)),
        Err(SectorBundleErrorV1::WireTrailing)
    );
    let mut noncanonical = original;
    noncanonical.goods.reverse();
    let bytes = codec::encode(&noncanonical).unwrap();
    assert_eq!(
        SectorBundleV1::decode(&bytes, sha256_of(&bytes)),
        Err(SectorBundleErrorV1::WireNoncanonical)
    );
}

#[test]
fn source_and_owner_mismatches_refuse_without_replacing_observed_identity() {
    let original = bundles().remove(0);
    let mut changed = original.clone();
    changed.sources.county_source_sha256[0] ^= 1;
    assert_eq!(
        rebuild(&changed, &changed.rows),
        Err(SectorBundleErrorV1::Source)
    );
    changed = original.clone();
    changed.owner.sector_code = "99".to_owned();
    assert_eq!(
        rebuild(&changed, &changed.rows),
        Err(SectorBundleErrorV1::Owner)
    );
    changed = original;
    changed.processes[0].industry_code = "999".to_owned();
    assert_eq!(
        rebuild(&changed, &changed.rows),
        Err(SectorBundleErrorV1::ProcessOwnership)
    );
}

#[test]
fn incomplete_and_duplicate_bundle_ownership_refuse() {
    let mut values = bundles();
    assert_eq!(
        compile_sector_bundles_v1(&values[..3], MichiganDeliveryPresetV1::Standard),
        Err(SectorBundleErrorV1::Coverage)
    );
    values[1] = values[0].clone();
    assert_eq!(
        compile_sector_bundles_v1(&values, MichiganDeliveryPresetV1::Standard),
        Err(SectorBundleErrorV1::ProcessOwnership)
    );
    let mut empty = bundles().remove(0);
    empty.processes.clear();
    assert_eq!(
        rebuild(&empty, &empty.rows),
        Err(SectorBundleErrorV1::Bound)
    );
}

#[test]
fn missing_input_and_weekly_resource_rows_are_not_silent_zeroes() {
    let values = bundles();
    let original = &values[macomb(&values)];
    let mut rows = original.rows.clone();
    let input = rows.input_coefficients[0].good_id;
    rows.inventory.retain(|row| row.good_id != input);
    assert_eq!(rebuild(original, &rows), Err(SectorBundleErrorV1::GoodUnit));
    let mut rows = original.rows.clone();
    rows.labor.retain(|row| row.week != 16);
    assert_eq!(rebuild(original, &rows), Err(SectorBundleErrorV1::Resource));
    let mut rows = original.rows.clone();
    rows.capacities.retain(|row| row.week != 16);
    assert_eq!(rebuild(original, &rows), Err(SectorBundleErrorV1::Resource));
}

#[test]
fn rewriting_a_good_unit_everywhere_cannot_relabel_its_physical_principal() {
    let values = bundles();
    let index = macomb(&values);
    let mut changed = values[index].clone();
    let input = changed.rows.input_coefficients[0].good_id;
    let new_unit = changed.rows.process_outputs[0].unit_id;
    changed
        .goods
        .iter_mut()
        .find(|row| row.good_id == input)
        .unwrap()
        .unit_id = new_unit;
    changed.rows.input_coefficients[0].unit_id = new_unit;
    for row in changed
        .rows
        .inventory
        .iter_mut()
        .filter(|row| row.good_id == input)
    {
        row.unit_id = new_unit;
    }
    assert_eq!(
        rebuild(&changed, &changed.rows),
        Err(SectorBundleErrorV1::GoodUnit)
    );
}

fn produced(transition: &MaterialCircuitTransitionV2, process: ProcessIdV1) -> u64 {
    transition
        .production
        .iter()
        .find(|row| row.process_id == process)
        .map_or(0, |row| row.produced_batches)
}

fn assert_food_equal(left: &MaterialCircuitTransitionV2, right: &MaterialCircuitTransitionV2) {
    let catalog = michigan_material_catalog_v1().unwrap();
    let sites: BTreeSet<_> = catalog
        .sites()
        .iter()
        .filter(|site| site.naics == "311")
        .map(crate::michigan_material::MichiganMaterialSiteV1::id)
        .collect();
    let processes: BTreeSet<_> = catalog
        .processes()
        .iter()
        .filter(|process| sites.contains(&process.site_id()))
        .map(crate::michigan_material::MichiganMaterialProcessV1::id)
        .collect();
    assert_eq!(
        left.production
            .iter()
            .filter(|row| processes.contains(&row.process_id))
            .collect::<Vec<_>>(),
        right
            .production
            .iter()
            .filter(|row| processes.contains(&row.process_id))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left.state
            .inventory
            .iter()
            .filter(|row| sites.contains(&row.site_id))
            .collect::<Vec<_>>(),
        right
            .state
            .inventory
            .iter()
            .filter(|row| sites.contains(&row.site_id))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left.state
            .labor
            .iter()
            .filter(|row| sites.contains(&row.site_id))
            .collect::<Vec<_>>(),
        right
            .state
            .labor
            .iter()
            .filter(|row| sites.contains(&row.site_id))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left.state
            .orders
            .iter()
            .filter(|row| sites.contains(&row.buyer_site_id))
            .collect::<Vec<_>>(),
        right
            .state
            .orders
            .iter()
            .filter(|row| sites.contains(&row.buyer_site_id))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left.state
            .freight
            .iter()
            .filter(|row| sites.contains(&row.source_site_id))
            .collect::<Vec<_>>(),
        right
            .state
            .freight
            .iter()
            .filter(|row| sites.contains(&row.source_site_id))
            .collect::<Vec<_>>()
    );
}

#[test]
fn bundle_recipe_and_labor_coefficients_change_actual_production_not_just_metadata() {
    let originals = bundles();
    let index = macomb(&originals);
    let process = originals[index].processes[0].process_id;
    let baseline = trace(
        compile_sector_bundles_v1(&originals, MichiganDeliveryPresetV1::Standard).unwrap(),
        8,
    );
    assert_eq!(produced(&baseline[2], process), 8);
    for change_labor in [false, true] {
        let mut changed = originals.clone();
        let mut rows = changed[index].rows.clone();
        if change_labor {
            rows.labor_coefficients[0].quantity_per_batch = 4;
        } else {
            rows.input_coefficients[0].quantity_per_batch = 20;
        }
        changed[index] = rebuild(&changed[index], &rows).unwrap();
        assert_ne!(changed[index].sha256(), originals[index].sha256());
        // These private causal mutants are not newly admitted content. The old
        // content pin must reject them even though their exact quantities type-check.
        assert_eq!(
            SectorBundleV1::decode(changed[index].canonical_bytes(), originals[index].sha256()),
            Err(SectorBundleErrorV1::Digest)
        );
        assert_eq!(changed[index].owner, originals[index].owner);
        assert_eq!(changed[index].sources, originals[index].sources);
        let changed = trace(
            compile_sector_bundles_v1(&changed, MichiganDeliveryPresetV1::Standard).unwrap(),
            8,
        );
        assert_eq!(produced(&changed[2], process), 4);
        assert_ne!(
            encode_material_circuit_state_v2(&changed[2].state).unwrap(),
            encode_material_circuit_state_v2(&baseline[2].state).unwrap()
        );
        for (left, right) in baseline.iter().zip(&changed) {
            assert_food_equal(left, right);
        }
    }
}

#[test]
fn delay_twins_keep_bundle_identity_and_capacity_and_change_following_week_output() {
    let values = bundles();
    let standard = compile_sector_bundles_v1(&values, MichiganDeliveryPresetV1::Standard).unwrap();
    let delayed = compile_sector_bundles_v1(&values, MichiganDeliveryPresetV1::Delayed).unwrap();
    assert_eq!(standard.capacities, delayed.capacities);
    assert_eq!(standard.labor, delayed.labor);
    let mut normalized = delayed.clone();
    normalized.route_legs = standard.route_legs.clone();
    assert_eq!(normalized, standard);
    let standard = trace(standard, 16);
    let delayed = trace(delayed, 16);
    let process = values[macomb(&values)].processes[0].process_id;
    assert_eq!(produced(&standard[1], process), 0);
    assert_eq!(produced(&standard[2], process), 8);
    assert_eq!(produced(&delayed[2], process), 0);
    assert_eq!(produced(&delayed[4], process), 8);
    for (left, right) in standard.iter().zip(&delayed) {
        assert_food_equal(left, right);
    }
}

#[test]
fn exact_bundle_and_register_decode_preserve_dispatch_transit_arrival_continuation() {
    let original = bundles();
    let decoded: Vec<_> = original
        .iter()
        .map(|bundle| SectorBundleV1::decode(bundle.canonical_bytes(), bundle.sha256()).unwrap())
        .collect();
    let opening = compile_sector_bundles_v1(&decoded, MichiganDeliveryPresetV1::Delayed).unwrap();
    let history = trace(opening, 5);
    for frame in &history[..4] {
        let bytes = encode_material_circuit_state_v2(&frame.state).unwrap();
        let restored = decode_material_circuit_state_v2(&bytes).unwrap();
        assert_eq!(
            advance_material_circuit_v2(&restored).unwrap(),
            advance_material_circuit_v2(&frame.state).unwrap()
        );
    }
}
