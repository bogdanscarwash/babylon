use babylon_persistence::{
    extract_declared_territory_county_map, TerritoryCountyMapError, TerritoryCountyMapRow,
    TERRITORY_COUNTY_MAP_FIELD,
};

const DECLARED_SCENARIO: &str = r"
(scenario contract/territory-county-map
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 26163))
  (node alcona NodeType/TERRITORY (territory/county-fips 1001)))
";

const PRELUDE_DECLARED_SCENARIO: &str = r"
(scenario contract/prelude-declared
  (defvocabulary NodeType (TERRITORY))
  (node wayne NodeType/TERRITORY (territory/county-fips 26163)))
";

const COUNTY_FIPS_PRELUDE: &str = r"
(deffield territory/county-fips int extensive)
";

#[test]
fn declared_int_field_seeds_as_county_map_rows_with_zero_padded_geoids() {
    let rows = extract_declared_territory_county_map(DECLARED_SCENARIO, None)
        .expect("declared scenario extracts");
    assert_eq!(
        rows,
        [
            TerritoryCountyMapRow::try_new("alcona".to_owned(), "01001".to_owned())
                .expect("zero-padded geoid"),
            TerritoryCountyMapRow::try_new("wayne".to_owned(), "26163".to_owned())
                .expect("exact geoid"),
        ]
    );
    assert_eq!(TERRITORY_COUNTY_MAP_FIELD, "territory/county-fips");
}

#[test]
fn scenario_without_the_field_declaration_extracts_no_rows() {
    let rows = extract_declared_territory_county_map(
        r"
(scenario contract/undeclared
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/dist-year int extensive)
  (node wayne NodeType/TERRITORY (territory/dist-year 2010)))
",
        None,
    )
    .expect("undeclared scenario extracts empty");
    assert!(rows.is_empty());
}

#[test]
fn territory_node_missing_the_declared_field_refuses() {
    let error = extract_declared_territory_county_map(
        r"
(scenario contract/missing-seed
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY))
",
        None,
    )
    .expect_err("missing county-fips seed refuses");
    assert_eq!(
        error,
        TerritoryCountyMapError::MissingCountyFips {
            node: "wayne".to_owned()
        }
    );
}

#[test]
fn duplicate_county_geoid_across_nodes_refuses() {
    let error = extract_declared_territory_county_map(
        r"
(scenario contract/duplicate-geoid
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 26163))
  (node clone NodeType/TERRITORY (territory/county-fips 26163)))
",
        None,
    )
    .expect_err("duplicate county geoid refuses");
    assert_eq!(
        error,
        TerritoryCountyMapError::DuplicateCountyGeoid {
            geoid: "26163".to_owned(),
            first_node: "wayne".to_owned(),
            second_node: "clone".to_owned(),
        }
    );
}

#[test]
fn county_fips_outside_the_five_digit_domain_refuses() {
    for (value, node) in [("100000", "too-large"), ("-1", "negative")] {
        let source = format!(
            "
(scenario contract/out-of-range-{node}
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY (territory/county-fips {value})))
"
        );
        let error = extract_declared_territory_county_map(&source, None)
            .expect_err("out-of-range county fips refuses");
        assert!(
            matches!(error, TerritoryCountyMapError::CountyFipsOutOfRange { .. }),
            "unexpected error: {error:?}"
        );
    }
}

#[test]
fn non_int_field_declaration_refuses() {
    let error = extract_declared_territory_county_map(
        r"
(scenario contract/wrong-type
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips real intensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 21.0r)))
",
        None,
    )
    .expect_err("non-int county-fips declaration refuses");
    assert_eq!(error, TerritoryCountyMapError::FieldDeclRefused);
}

#[test]
fn non_extensive_int_declaration_refuses() {
    // The declaration contract is BOTH axes: `int` type AND `extensive`
    // kind. An intensive int is as refused as a real.
    let error = extract_declared_territory_county_map(
        r"
(scenario contract/wrong-kind
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int intensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 26163)))
",
        None,
    )
    .expect_err("non-extensive county-fips declaration refuses");
    assert_eq!(error, TerritoryCountyMapError::FieldDeclRefused);
}

#[test]
fn prelude_declared_field_extracts_rows() {
    // Campaigns that declare the field in a declaration prelude (the
    // session-hydration path) extract exactly like in-scenario declarations.
    let rows =
        extract_declared_territory_county_map(PRELUDE_DECLARED_SCENARIO, Some(COUNTY_FIPS_PRELUDE))
            .expect("prelude-declared scenario extracts");
    assert_eq!(
        rows,
        [
            TerritoryCountyMapRow::try_new("wayne".to_owned(), "26163".to_owned())
                .expect("exact geoid")
        ]
    );
}

#[test]
fn prelude_only_declaration_without_the_prelude_refuses() {
    // Honest failure shape: re-reading the scenario WITHOUT its prelude
    // cannot resolve the field, so the load itself refuses.
    let error = extract_declared_territory_county_map(PRELUDE_DECLARED_SCENARIO, None)
        .expect_err("prelude-only declaration without the prelude refuses");
    assert_eq!(error, TerritoryCountyMapError::ScenarioLoad);
}

#[test]
fn row_validation_pins_the_exact_geoid_shape() {
    assert_eq!(
        TerritoryCountyMapRow::try_new("wayne".to_owned(), "2616".to_owned()),
        Err(TerritoryCountyMapError::InvalidCountyGeoid)
    );
    assert_eq!(
        TerritoryCountyMapRow::try_new("wayne".to_owned(), "261633".to_owned()),
        Err(TerritoryCountyMapError::InvalidCountyGeoid)
    );
    assert_eq!(
        TerritoryCountyMapRow::try_new("wayne".to_owned(), "2616a".to_owned()),
        Err(TerritoryCountyMapError::InvalidCountyGeoid)
    );
    assert_eq!(
        TerritoryCountyMapRow::try_new(String::new(), "26163".to_owned()),
        Err(TerritoryCountyMapError::InvalidTerritoryLocalName)
    );
}
