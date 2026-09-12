use super::*;
use crate::michigan_material::{MichiganDeliveryPreset, MichiganMaterialCatalog};
use babylon_material_circuit::{
    advance_staffing, close_material_period, MaterialCircuitTransition, StaffingPoolState,
    StaffingState,
};
fn selected(preset: MichiganDeliveryPreset) -> MichiganMaterialCatalog {
    crate::test_support::catalog().with_preset(preset).unwrap()
}
fn state(c: &MichiganMaterialCatalog) -> MaterialCircuitState {
    compile_sector_bundles(&michigan_sector_bundles(c).unwrap(), c.preset(), c).unwrap()
}
fn trace(c: &MichiganMaterialCatalog, periods: usize) -> Vec<MaterialCircuitTransition> {
    let mut state = state(c);
    let authority = staffing::StoredStaffing::authored(c).unwrap();
    let composition = authority.composition().unwrap();
    let bindings: Vec<_> = composition
        .bindings()
        .iter()
        .map(|b| b.pool().clone())
        .collect();
    let pools=composition.bindings().iter().map(|b|{let seed=authority.design().pools.iter().find(|s|matches!(b.subject(),StableElementKey::Node{local_name,..} if *local_name==s.local_name())).unwrap();StaffingPoolState::try_new(b.pool().clone(),seed.employed,seed.reserve,seed.previous_unretained_hours).unwrap()}).collect();
    let mut staffing = StaffingState::try_new(1, pools).unwrap();
    (0..periods)
        .map(|_| {
            let closed = close_material_period(&state).unwrap();
            let requests = closed.staffing_requests(&bindings).unwrap();
            let staffed = advance_staffing(&staffing, &requests).unwrap();
            let result = closed
                .finish_with_labor(staffed.next_labor().to_vec())
                .unwrap();
            staffing = staffed.into_state();
            state = result.state.clone();
            result
        })
        .collect()
}
#[test]
fn regional_owners_preserve_five_sites_and_separate_wayne_resources() {
    let c = selected(MichiganDeliveryPreset::Standard);
    let bundles = michigan_sector_bundles(&c).unwrap();
    assert_eq!(bundles.len(), 4);
    assert_eq!(bundles.iter().map(|b| b.processes.len()).sum::<usize>(), 5);
    let wayne = bundles
        .iter()
        .find(|b| b.owner.county_geoid == "26163")
        .unwrap();
    assert_eq!(wayne.rows.site_logistics_nodes.len(), 2);
    assert_eq!(wayne.rows.labor.len(), 2);
    assert_eq!(
        wayne
            .rows
            .labor
            .iter()
            .map(|r| r.available)
            .collect::<std::collections::BTreeSet<_>>(),
        [640, 3200].into()
    );
    for b in bundles {
        assert_eq!(
            SectorBundle::decode(b.canonical_bytes(), b.sha256()).unwrap(),
            b
        );
    }
}
#[test]
fn regional_shared_freight_preserves_dispatch_and_period_three_production() {
    for (preset, sheet, meal, panels, packaged) in [
        (MichiganDeliveryPreset::SharedFreightAmple, 320, 80, 32, 80),
        (
            MichiganDeliveryPreset::SharedFreightConstrained,
            120,
            40,
            12,
            40,
        ),
    ] {
        let c = selected(preset);
        let history = trace(&c, 3);
        let dispatched = |key: &str| {
            let order = c.routes().iter().find(|r| r.key == key).unwrap().order_id();
            history[0]
                .dispatches
                .iter()
                .filter(|r| r.order_id == order)
                .map(|r| r.quantity)
                .sum::<u64>()
        };
        assert_eq!(dispatched("sheet-transfer"), sheet);
        assert_eq!(dispatched("food-transfer"), meal);
        for (key, quantity) in [("panel-forming", panels), ("meal-packaging", packaged)] {
            let p = c.processes().iter().find(|p| p.key == key).unwrap();
            let batches = history[2]
                .production
                .iter()
                .find(|r| r.process_id == p.id())
                .unwrap()
                .produced_batches;
            assert_eq!(batches * p.output_quantity_per_batch, quantity);
        }
        let initial = state(&c);
        assert_eq!(initial.corridor_capacities.len(), 32);
        assert_eq!(
            initial
                .corridor_capacities
                .iter()
                .map(|r| (r.corridor_id, r.period))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            32
        );
    }
}
#[test]
fn shared_competition_retains_proportional_flooring_and_unused_gram_capacity() {
    let text = include_str!("../../../../../content/scenarios/michigan/defines.toml")
        .replace("ORDERED_UNITS = 200", "ORDERED_UNITS = 80");
    let c = MichiganMaterialCatalog::from_defines_toml(&text)
        .unwrap()
        .with_preset(MichiganDeliveryPreset::SharedFreightConstrained)
        .unwrap();
    let history = trace(&c, 1);
    for (key, quantity) in [("sheet-transfer", 141), ("food-transfer", 18)] {
        let order = c.routes().iter().find(|r| r.key == key).unwrap().order_id();
        assert_eq!(
            history[0]
                .dispatches
                .iter()
                .filter(|r| r.order_id == order)
                .map(|r| r.quantity)
                .sum::<u64>(),
            quantity
        );
    }
    let capacity = c
        .corridors()
        .iter()
        .find(|r| r.key == "shared-freight")
        .unwrap();
    assert_eq!(capacity.capacity_grams_per_period - (141 + 18) * 1000, 1000);
}
#[test]
fn delayed_regression_changes_only_timed_stages() {
    let standard = selected(MichiganDeliveryPreset::Standard);
    let delayed = selected(MichiganDeliveryPreset::Delayed);
    let a = state(&standard);
    let mut b = state(&delayed);
    b.route_stages.clone_from(&a.route_stages);
    assert_eq!(a, b);
    let a = trace(&standard, 5);
    let b = trace(&delayed, 5);
    let panel = standard
        .processes()
        .iter()
        .find(|p| p.key == "panel-forming")
        .unwrap()
        .id();
    let batches = |t: &MaterialCircuitTransition| {
        t.production
            .iter()
            .find(|r| r.process_id == panel)
            .map_or(0, |receipt| receipt.produced_batches)
    };
    assert_eq!(batches(&a[2]), 32);
    assert_eq!(batches(&b[2]), 0);
    assert_eq!(batches(&b[4]), 32);
}
#[test]
fn bundle_and_row_permutations_preserve_identity_and_changed_authority_refuses() {
    let c = selected(MichiganDeliveryPreset::Standard);
    let original = michigan_sector_bundles(&c).unwrap();
    let mut changed = original.clone();
    for b in &mut changed {
        let mut rows = b.rows.clone();
        rows.process_outputs.reverse();
        rows.inventory.reverse();
        rows.input_coefficients.reverse();
        rows.site_logistics_nodes.reverse();
        rows.freight_mass_coefficients.reverse();
        let mut goods = b.goods.clone();
        goods.reverse();
        let rebuilt = SectorBundle::from_parts(
            b.owner.clone(),
            b.sources.clone(),
            goods,
            b.processes.clone(),
            b.labor_unit,
            &rows,
        )
        .unwrap();
        assert_eq!(*b, rebuilt);
    }
    changed.reverse();
    assert_eq!(
        compile_sector_bundles(&original, c.preset(), &c).unwrap(),
        compile_sector_bundles(&changed, c.preset(), &c).unwrap()
    );
    changed[0].sources.county_source_sha256[0] ^= 1;
    assert_eq!(
        compile_sector_bundles(&changed, c.preset(), &c),
        Err(SectorBundleError::Source)
    );
    assert!(compile_sector_bundles(&original[..3], c.preset(), &c).is_err());
}
#[test]
fn current_bundle_codec_refuses_version_digest_truncation_and_extra_bytes() {
    let c = selected(MichiganDeliveryPreset::Standard);
    let b = michigan_sector_bundles(&c).unwrap().remove(0);
    let original = b.canonical_bytes();
    assert_eq!(
        &original[..BUNDLE_DOMAIN.len()],
        b"babylon.sector-bundle.v2\0"
    );
    assert_eq!(
        SectorBundle::decode(original, [0; 32]),
        Err(SectorBundleError::Digest)
    );
    let mut changed = original.to_vec();
    changed[BUNDLE_DOMAIN.len() + 1] = 1;
    assert_eq!(
        SectorBundle::decode(&changed, sha256_of(&changed)),
        Err(SectorBundleError::WireVersion)
    );
    for n in [0, BUNDLE_DOMAIN.len(), original.len() - 1] {
        let short = &original[..n];
        assert_eq!(
            SectorBundle::decode(short, sha256_of(short)),
            Err(SectorBundleError::WireTruncated)
        );
    }
    let mut changed = original.to_vec();
    changed.push(0);
    assert_eq!(
        SectorBundle::decode(&changed, sha256_of(&changed)),
        Err(SectorBundleError::WireTrailing)
    );
}
