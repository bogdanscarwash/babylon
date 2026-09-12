//! Capture/regional regression evidence before the statewide compiler is introduced.

#[test]
fn captured_authority_contains_normalized_content_instead_of_only_numeric_defines() {
    let source = include_str!("../../../../../content/scenarios/michigan/defines.toml");
    let catalog = MichiganMaterialCatalog::from_defines_toml(source).unwrap();
    let stored: serde_json::Value = serde_json::from_slice(catalog.defines_bytes()).unwrap();
    assert_eq!(stored["schema"], "MichiganCapturedContentV2");
    assert_eq!(stored["normalized"]["sites"].as_array().unwrap().len(), 5);
    assert_eq!(
        stored["normalized"]["processes"].as_array().unwrap().len(),
        5
    );
}

use super::*;
use std::collections::BTreeMap;

#[test]
fn normalized_permutations_and_preset_round_trips_preserve_complete_authority() {
    let original = MichiganMaterialCatalog::from_defines_toml(include_str!(
        "../../../../../content/scenarios/michigan/defines.toml"
    ))
    .unwrap();
    let mut capture = original.capture.clone();
    capture.normalized.sites.reverse();
    capture.normalized.processes.reverse();
    capture.normalized.goods.reverse();
    capture.normalized.owners.reverse();
    capture.normalized.industry.reverse();
    capture.normalized.staffing.pools.reverse();
    capture.interventions.reverse();
    let reordered = MichiganMaterialCatalog::capture(capture).unwrap();
    assert_eq!(original.defines_bytes(), reordered.defines_bytes());
    let constrained = original
        .with_preset(MichiganDeliveryPreset::SharedFreightConstrained)
        .unwrap();
    let restored =
        MichiganMaterialCatalog::from_stored_defines(constrained.defines_bytes()).unwrap();
    for catalog in [&original, &reordered, &constrained, &restored] {
        assert_eq!(catalog.defines_hash(), sha256_of(catalog.defines_bytes()));
    }
    assert_ne!(original.defines_hash(), constrained.defines_hash());
    assert_eq!(constrained, restored);
    assert_eq!(
        original.graph_scenario_source(),
        constrained.graph_scenario_source()
    );
    assert_eq!(
        original,
        constrained
            .with_preset(MichiganDeliveryPreset::Standard)
            .unwrap()
    );
    assert!(!restored.observed_defines().is_empty());
    assert!(restored
        .graph_scenario_source()
        .contains("business-26163-31-33"));
    assert!(restored
        .with_preset(MichiganDeliveryPreset::StatewideBoth)
        .is_err());
}

fn merchant_fixture() -> MichiganMaterialCatalog {
    let original = MichiganMaterialCatalog::from_defines_toml(include_str!(
        "../../../../../content/scenarios/michigan/defines.toml"
    ))
    .unwrap();
    let mut c = original.capture.normalized;
    let mut second = c
        .processes
        .iter()
        .find(|p| p.key == "panel-forming")
        .unwrap()
        .clone();
    second.key = "second-panel-process".to_owned();
    second.inputs.push(MichiganMaterialInput {
        good_key: "meal".to_owned(),
        quantity_per_batch: 1,
        opening_quantity: 5,
    });
    c.staffing
        .pools
        .iter_mut()
        .find(|p| p.site_key == second.site_key)
        .unwrap()
        .process_keys
        .push(second.key.clone());
    c.processes.push(second);
    append_fixture_merchants(&mut c);
    let supplier = c
        .processes
        .iter()
        .find(|p| p.key == "sheet-rolling")
        .unwrap()
        .site_key
        .clone();
    for (key, supplier, buyer) in [
        ("fixture-purchase", supplier, "fixture-42".to_owned()),
        (
            "fixture-resale",
            "fixture-42".to_owned(),
            "fixture-44-45".to_owned(),
        ),
    ] {
        c.routes.push(MichiganMaterialRoute {
            key: key.to_owned(),
            supplier_site_key: supplier,
            buyer_site_key: buyer,
            good_key: "sheet".to_owned(),
            ordered_quantity: 20,
            path: MichiganMaterialPath::Local,
        });
    }
    c.final_demands.push(MichiganFinalDemand {
        key: "fixture-final".to_owned(),
        retailer_site_key: "fixture-44-45".to_owned(),
        county_geoid: "26163".to_owned(),
        good_key: "sheet".to_owned(),
        ordered_quantity: 20,
    });
    MichiganMaterialCatalog::from_normalized(
        original.capture.defines,
        c,
        MichiganDeliveryPreset::Standard,
        Vec::new(),
    )
    .unwrap()
}
#[test]
fn normalized_multi_input_owners_share_inventory_and_labor_and_merchants_have_no_fake_road() {
    let c = merchant_fixture();
    let bundles = crate::sector_bundle::michigan_sector_bundles(&c).unwrap();
    assert_eq!(bundles.len(), 6);
    let state = crate::sector_bundle::compile_sector_bundles(&bundles, c.preset(), &c).unwrap();
    let p = c
        .processes()
        .iter()
        .find(|p| p.key == "second-panel-process")
        .unwrap();
    assert_eq!(
        state
            .input_coefficients
            .iter()
            .filter(|r| r.process_id == p.id())
            .count(),
        2
    );
    assert_eq!(
        state
            .labor
            .iter()
            .filter(|r| r.site_id == p.site_id())
            .count(),
        1
    );
    assert_eq!(
        state
            .inventory
            .iter()
            .filter(|r| r.site_id == p.site_id() && r.good_id == c.good("sheet").unwrap().id())
            .count(),
        1
    );
    assert_eq!(state.merchants.len(), 2);
    assert_eq!(state.final_demand_orders.len(), 1);
    assert_eq!(state.supplier_routes.len(), 5);
    assert_eq!(state.route_stages.len(), 3);
    assert!(state.freight.is_empty());
    for route in state
        .supplier_routes
        .iter()
        .filter(|r| r.transport_kind == babylon_material_circuit::SupplierTransport::Local)
    {
        assert!(!state
            .route_stages
            .iter()
            .any(|s| s.route_id == route.route_id));
    }
    let restored = MichiganMaterialCatalog::from_stored_defines(c.defines_bytes()).unwrap();
    assert_eq!(restored, c);
}
#[test]
fn observed_suppression_remains_absent_and_zero_employment_can_recover_from_reserve() {
    let c = merchant_fixture();
    let mut capture = c.capture;
    let row = &mut capture.normalized.industry[0];
    row.disclosure_code = "N".to_owned();
    row.annual_avg_emplvl = None;
    row.total_annual_wages = None;
    row.annual_avg_wkly_wage = None;
    let pool = &mut capture.normalized.staffing.pools[0];
    pool.employed = 0;
    pool.reserve = 5;
    pool.previous_unretained_hours = 0;
    let accepted = MichiganMaterialCatalog::capture(capture.clone()).unwrap();
    assert_eq!(
        accepted.capture.normalized.industry[0].annual_avg_emplvl,
        None
    );
    capture.normalized.industry[0].annual_avg_emplvl = Some(0);
    assert!(MichiganMaterialCatalog::capture(capture).is_err());
}
#[test]
fn complete_statewide_workforce_graph_stays_inside_the_source_bound() {
    let seeds = (0..397)
        .map(|n| MichiganWorkforceSeed {
            key: format!("owner-{:05}-31-33", 26001 + n),
            site_key: format!("owner-{n}"),
            process_keys: Vec::new(),
            merchant_handling: true,
            employed: 20,
            reserve: 4,
            previous_unretained_hours: 3200,
        })
        .collect::<Vec<_>>();
    let source = crate::michigan_cohorts::michigan_staffed_scenario(&seeds).unwrap();
    assert!(source.len() < 1_048_576);
    eprintln!("397-pool observed graph source: {} bytes", source.len());
}

fn append_fixture_merchants(c: &mut MichiganNormalizedContent) {
    for (sector, role) in [
        ("42", MichiganSiteRole::Wholesale),
        ("44-45", MichiganSiteRole::Retail),
    ] {
        let source =
            regional::owner_source("26163", sector, MICHIGAN_INDUSTRY_BASELINE_SHA256).unwrap();
        let key = format!("fixture-{sector}");
        c.industry.push(MichiganIndustryBaselineRow {
            area_fips: "26163".to_owned(),
            area_title: "Wayne County, Michigan".to_owned(),
            industry_code: sector.to_owned(),
            industry_title: source.sector_title.clone(),
            own_code: "5".to_owned(),
            agglvl_code: "74".to_owned(),
            disclosure_code: source.disclosure_code.clone(),
            annual_avg_estabs_count: source.annual_avg_estabs_count,
            annual_avg_emplvl: source.annual_avg_emplvl,
            total_annual_wages: source.total_annual_wages,
            annual_avg_wkly_wage: source.annual_avg_wkly_wage,
            source_file: source.county_source_file.clone(),
            source_sha256: source.county_source_sha256.clone(),
        });
        c.owners.push(source);
        c.sites.push(MichiganMaterialSite {
            key: key.clone(),
            label: format!("Synthetic local {sector} fixture"),
            county_geoid: "26163".to_owned(),
            naics: sector.to_owned(),
            sector_code: sector.to_owned(),
            role,
        });
        c.staffing.pools.push(MichiganWorkforceSeed {
            key: key.clone(),
            site_key: key.clone(),
            process_keys: Vec::new(),
            merchant_handling: true,
            employed: 1,
            reserve: 0,
            previous_unretained_hours: 160,
        });
        let capacity_key = format!("handling-{sector}");
        c.corridors.push(MichiganMaterialCorridor {
            key: capacity_key.clone(),
            label: format!("Synthetic {sector} handling"),
            capacity_grams_per_period: 1_000_000,
        });
        c.merchants.push(MichiganMerchant {
            site_key: key,
            capacity_key,
            handling_hours_per_unit: BTreeMap::from([("sheet".to_owned(), 1)]),
        });
    }
}
