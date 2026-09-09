//! Pure contracts for the bounded PER-278 reference-product bundle.

use babylon_persistence::{
    michigan_spatial_reference_products_v1, representative_h3_reference_cohort_v1,
    CountyH3LandAreaRow, CountyPlaceH3LandAreaRow, H3CountRow, ReferenceProduct,
};

#[test]
fn checked_bundle_has_exact_governed_products_and_counts() {
    let cohort = cohort();
    let bundle = michigan_spatial_reference_products_v1(&cohort)
        .expect("checked PER-278 fixture must validate");

    assert_eq!(bundle.ref_digest(), cohort.receipt().ref_digest());
    assert_eq!(bundle.products().len(), 7);
    assert_eq!(bundle.counties().len(), 3_285);
    assert_eq!(bundle.places().len(), 745);
    assert_eq!(bundle.land_fractions().len(), 45_572);
    assert_eq!(bundle.population_counts().len(), 22_509);
    assert_eq!(bundle.workplace_counts().len(), 11_833);
    assert_eq!(bundle.county_land_areas().len(), 31_881);
    assert_eq!(bundle.county_place_land_areas().len(), 4_813);

    let codes = bundle
        .products()
        .iter()
        .map(ReferenceProduct::code)
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            "census_county_h3_land_overlap_mi_2023",
            "census_county_place_h3_land_overlap_mi_2023",
            "census_place_identity_mi_2023",
            "dim_county",
            "h3_res7_land_mask",
            "h3_res7_population",
            "h3_res7_workplace",
        ]
    );
}

#[test]
fn checked_bundle_preserves_measure_and_absence_law() {
    let cohort = cohort();
    let bundle = michigan_spatial_reference_products_v1(&cohort).unwrap();

    assert_eq!(
        bundle
            .county_land_areas()
            .iter()
            .map(CountyH3LandAreaRow::land_area_m2)
            .sum::<u64>(),
        146_426_246_267
    );
    assert_eq!(
        bundle
            .county_place_land_areas()
            .iter()
            .map(CountyPlaceH3LandAreaRow::place_land_area_m2)
            .sum::<u64>(),
        7_689_548_061
    );
    assert_eq!(
        bundle
            .population_counts()
            .iter()
            .map(H3CountRow::count)
            .sum::<u64>(),
        10_066_869
    );
    assert_eq!(
        bundle
            .workplace_counts()
            .iter()
            .map(H3CountRow::count)
            .sum::<u64>(),
        3_931_809
    );
    assert!(bundle
        .land_fractions()
        .iter()
        .any(|row| row.parts_per_million() == 0));
    assert!(bundle.population_counts().iter().all(|row| row.count() > 0));
    assert!(bundle.workplace_counts().iter().all(|row| row.count() > 0));
}

fn cohort() -> babylon_persistence::H3ReferenceCohort {
    representative_h3_reference_cohort_v1()
        .expect("the sole checked-in source fixture must validate")
        .clone()
}
