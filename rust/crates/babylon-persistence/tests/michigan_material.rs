use babylon_material_circuit::{
    advance_material_circuit_v2, decode_material_circuit_state_v2,
    encode_material_circuit_state_v2, MaterialCircuitStateV2,
};
use babylon_persistence::michigan_material::{
    michigan_material_catalog_v1, michigan_material_foundation_v1, MichiganDeliveryPresetV1,
    MichiganMaterialSiteV1,
};

fn inventory(state: &MaterialCircuitStateV2, site: &str, good: &str) -> u64 {
    let catalog = michigan_material_catalog_v1().unwrap();
    let site_id = catalog.site(site).unwrap().id();
    let good_id = catalog.good(good).unwrap().id();
    state
        .inventory
        .iter()
        .find(|row| row.site_id == site_id && row.good_id == good_id)
        .map_or(0, |row| row.quantity)
}

fn assert_material_conserved(state: &MaterialCircuitStateV2) {
    let catalog = michigan_material_catalog_v1().unwrap();
    let mut metal = 0;
    let mut food = 0;
    for (good_key, scale, is_metal) in [
        ("billet", 1, true),
        ("sheet", 1, true),
        ("panel", 10, true),
        ("subassembly", 20, true),
        ("grain", 1, false),
        ("meal", 1, false),
        ("packaged-meal", 1, false),
    ] {
        let good_id = catalog.good(good_key).unwrap().id();
        let on_hand: u64 = state
            .inventory
            .iter()
            .filter(|row| row.good_id == good_id)
            .map(|row| row.quantity)
            .sum();
        let in_transit: u64 = state
            .freight
            .iter()
            .filter(|row| row.good_id == good_id)
            .map(|row| row.quantity)
            .sum();
        if is_metal {
            metal += (on_hand + in_transit) * scale;
        } else {
            food += (on_hand + in_transit) * scale;
        }
    }
    assert_eq!(
        metal, 600,
        "metal input-equivalent kg at opening {}",
        state.week
    );
    assert_eq!(food, 200, "food kg at opening {}", state.week);
}

fn assert_second_week_delivery_delay(
    standard: &MaterialCircuitStateV2,
    delayed: &MaterialCircuitStateV2,
) {
    assert_eq!(inventory(standard, "macomb-fabricated-metal", "sheet"), 80);
    assert_eq!(inventory(delayed, "macomb-fabricated-metal", "sheet"), 0);
    let transformer = michigan_material_catalog_v1()
        .unwrap()
        .processes()
        .iter()
        .find(|row| row.key == "panel-forming")
        .unwrap();
    assert_eq!(
        standard
            .production_commitments
            .iter()
            .find(|row| row.process_id == transformer.id())
            .unwrap()
            .planned_batches,
        8
    );
    assert_eq!(
        delayed
            .production_commitments
            .iter()
            .find(|row| row.process_id == transformer.id())
            .map_or(0, |row| row.planned_batches),
        0
    );
    assert_eq!(inventory(standard, "wayne-vehicle-parts", "subassembly"), 0);
}

#[test]
fn presets_share_exact_setup_except_the_single_declared_delay() {
    let standard = michigan_material_foundation_v1(MichiganDeliveryPresetV1::Standard).unwrap();
    let mut delayed = michigan_material_foundation_v1(MichiganDeliveryPresetV1::Delayed).unwrap();
    let catalog = michigan_material_catalog_v1().unwrap();
    let route = catalog
        .routes()
        .iter()
        .find(|route| route.key == "sheet-transfer")
        .unwrap();
    let changed = delayed
        .route_legs
        .iter_mut()
        .find(|row| row.route_id == route.id())
        .unwrap();
    assert_eq!(changed.travel_weeks, 3);
    changed.travel_weeks = 1;
    assert_eq!(
        encode_material_circuit_state_v2(&standard).unwrap(),
        encode_material_circuit_state_v2(&delayed).unwrap()
    );
    assert_eq!(standard.week, 1);
    assert_eq!(standard.capacities.len(), 5 * 16);
    assert_eq!(standard.labor.len(), 5 * 16);
    assert_eq!(standard.corridor_capacities.len(), 3 * 16);
    assert_eq!(catalog.terminal_output_disposition(), "on_hand_unsold");
    for site in catalog.sites() {
        let source = catalog.industry_for_site(site).unwrap();
        assert_eq!(source.area_fips, site.county_geoid);
        assert_eq!(source.industry_code, site.naics);
        assert!(source.disclosure_code.is_empty());
    }
}

#[test]
fn delivery_delay_changes_following_week_output_with_food_causally_disconnected() {
    let catalog = michigan_material_catalog_v1().unwrap();
    let food_sites: Vec<_> = catalog
        .sites()
        .iter()
        .filter(|site| site.naics == "311")
        .map(MichiganMaterialSiteV1::id)
        .collect();
    let food_route = catalog
        .routes()
        .iter()
        .find(|route| route.key == "food-transfer")
        .unwrap();
    let mut standard = michigan_material_foundation_v1(MichiganDeliveryPresetV1::Standard).unwrap();
    let mut delayed = michigan_material_foundation_v1(MichiganDeliveryPresetV1::Delayed).unwrap();
    let mut first_standard_output = None;
    let mut first_delayed_output = None;
    for week in 1..=MichiganDeliveryPresetV1::Standard.horizon_ticks() {
        let a = advance_material_circuit_v2(&standard).unwrap();
        let b = advance_material_circuit_v2(&delayed).unwrap();
        assert_eq!(
            a.production
                .iter()
                .filter(|row| food_sites.contains(&row.site_id))
                .collect::<Vec<_>>(),
            b.production
                .iter()
                .filter(|row| food_sites.contains(&row.site_id))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            a.state
                .inventory
                .iter()
                .filter(|row| food_sites.contains(&row.site_id))
                .collect::<Vec<_>>(),
            b.state
                .inventory
                .iter()
                .filter(|row| food_sites.contains(&row.site_id))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            a.state
                .freight
                .iter()
                .filter(|row| row.route_id == food_route.id())
                .collect::<Vec<_>>(),
            b.state
                .freight
                .iter()
                .filter(|row| row.route_id == food_route.id())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            a.state
                .orders
                .iter()
                .find(|row| row.order_id == food_route.order_id()),
            b.state
                .orders
                .iter()
                .find(|row| row.order_id == food_route.order_id())
        );
        standard = a.state;
        delayed = b.state;
        assert_material_conserved(&standard);
        assert_material_conserved(&delayed);
        if inventory(&standard, "wayne-vehicle-parts", "subassembly") > 0 {
            first_standard_output.get_or_insert(week);
        }
        if inventory(&delayed, "wayne-vehicle-parts", "subassembly") > 0 {
            first_delayed_output.get_or_insert(week);
        }
        if week == 2 {
            assert_second_week_delivery_delay(&standard, &delayed);
        }
    }
    assert_eq!(first_standard_output, Some(5));
    assert_eq!(first_delayed_output, Some(7));
    for state in [&standard, &delayed] {
        assert_eq!(inventory(state, "wayne-vehicle-parts", "subassembly"), 30);
        assert_eq!(inventory(state, "oakland-food", "packaged-meal"), 200);
        assert!(state.freight.is_empty());
        assert!(state
            .orders
            .iter()
            .all(|order| order.ordered == order.delivered
                && order.realized == order.delivered
                && order.lost == 0));
        assert_eq!(state.week, 17);
    }
}

#[test]
fn every_dispatch_transit_arrival_restart_reproduces_exact_continuation() {
    for preset in [
        MichiganDeliveryPresetV1::Standard,
        MichiganDeliveryPresetV1::Delayed,
    ] {
        let mut state = michigan_material_foundation_v1(preset).unwrap();
        for _ in 0..preset.horizon_ticks() {
            let bytes = encode_material_circuit_state_v2(&state).unwrap();
            let reopened = decode_material_circuit_state_v2(&bytes).unwrap();
            let expected = advance_material_circuit_v2(&state).unwrap();
            let actual = advance_material_circuit_v2(&reopened).unwrap();
            assert_eq!(actual, expected);
            state = actual.state;
        }
    }
}
