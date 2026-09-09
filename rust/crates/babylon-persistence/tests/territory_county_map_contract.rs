use babylon_persistence::{
    extract_declared_territory_county_map_v1, TerritoryCountyMapErrorV1, TerritoryCountyMapRowV1,
    TERRITORY_COUNTY_MAP_FIELD_V1, TERRITORY_COUNTY_MAP_SCHEMA_CONTRACT_ID,
    TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL,
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
fn schema_contract_keeps_declared_mapping_out_of_material_state() {
    assert!(TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL
        .contains("CREATE TABLE babylon_meta.territory_county_map_v1"));
    assert!(TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL
        .contains("PRIMARY KEY (campaign_id, territory_local_name)"));
    assert!(TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL
        .contains("REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE"));
    assert!(TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL.contains("county_geoid ~ '^[0-9]{5}$'"));
    assert!(TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL
        .contains("REVOKE ALL ON TABLE babylon_meta.territory_county_map_v1 FROM PUBLIC"));
    assert!(!TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL.contains("IF NOT EXISTS"));
    assert!(!TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL.contains("CREATE TABLE babylon_state."));
    // The declared mapping is reference data: it must not point into the
    // tick-authority schema or the spatial-product rows that a separate
    // governed installer may not have installed yet.
    assert!(!TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL.contains("babylon_state."));
    assert!(!TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL.contains("county_identity"));
    assert!(TERRITORY_COUNTY_MAP_SCHEMA_CONTRACT_ID.len() <= 256);
    assert_eq!(
        TERRITORY_COUNTY_MAP_SCHEMA_CONTRACT_ID,
        "babylon.territory-county-map-schema.v1"
    );
}

#[test]
fn declared_int_field_seeds_as_county_map_rows_with_zero_padded_geoids() {
    let rows = extract_declared_territory_county_map_v1(DECLARED_SCENARIO, None)
        .expect("declared scenario extracts");
    assert_eq!(
        rows,
        [
            TerritoryCountyMapRowV1::try_new("alcona".to_owned(), "01001".to_owned())
                .expect("zero-padded geoid"),
            TerritoryCountyMapRowV1::try_new("wayne".to_owned(), "26163".to_owned())
                .expect("exact geoid"),
        ]
    );
    assert_eq!(TERRITORY_COUNTY_MAP_FIELD_V1, "territory/county-fips");
}

#[test]
fn scenario_without_the_field_declaration_extracts_no_rows() {
    let rows = extract_declared_territory_county_map_v1(
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
    let error = extract_declared_territory_county_map_v1(
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
        TerritoryCountyMapErrorV1::MissingCountyFips {
            node: "wayne".to_owned()
        }
    );
}

#[test]
fn duplicate_county_geoid_across_nodes_refuses() {
    let error = extract_declared_territory_county_map_v1(
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
        TerritoryCountyMapErrorV1::DuplicateCountyGeoid {
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
        let error = extract_declared_territory_county_map_v1(&source, None)
            .expect_err("out-of-range county fips refuses");
        assert!(
            matches!(
                error,
                TerritoryCountyMapErrorV1::CountyFipsOutOfRange { .. }
            ),
            "unexpected error: {error:?}"
        );
    }
}

#[test]
fn non_int_field_declaration_refuses() {
    let error = extract_declared_territory_county_map_v1(
        r"
(scenario contract/wrong-type
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips real intensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 21.0r)))
",
        None,
    )
    .expect_err("non-int county-fips declaration refuses");
    assert_eq!(error, TerritoryCountyMapErrorV1::FieldDeclRefused);
}

#[test]
fn non_extensive_int_declaration_refuses() {
    // The declaration contract is BOTH axes: `int` type AND `extensive`
    // kind. An intensive int is as refused as a real.
    let error = extract_declared_territory_county_map_v1(
        r"
(scenario contract/wrong-kind
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int intensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 26163)))
",
        None,
    )
    .expect_err("non-extensive county-fips declaration refuses");
    assert_eq!(error, TerritoryCountyMapErrorV1::FieldDeclRefused);
}

#[test]
fn prelude_declared_field_extracts_rows() {
    // Campaigns that declare the field in a declaration prelude (the
    // session-hydration path) extract exactly like in-scenario declarations.
    let rows = extract_declared_territory_county_map_v1(
        PRELUDE_DECLARED_SCENARIO,
        Some(COUNTY_FIPS_PRELUDE),
    )
    .expect("prelude-declared scenario extracts");
    assert_eq!(
        rows,
        [
            TerritoryCountyMapRowV1::try_new("wayne".to_owned(), "26163".to_owned())
                .expect("exact geoid")
        ]
    );
}

#[test]
fn prelude_only_declaration_without_the_prelude_refuses() {
    // Honest failure shape: re-reading the scenario WITHOUT its prelude
    // cannot resolve the field, so the load itself refuses.
    let error = extract_declared_territory_county_map_v1(PRELUDE_DECLARED_SCENARIO, None)
        .expect_err("prelude-only declaration without the prelude refuses");
    assert_eq!(error, TerritoryCountyMapErrorV1::ScenarioLoad);
}

#[test]
fn row_validation_pins_the_exact_geoid_shape() {
    assert_eq!(
        TerritoryCountyMapRowV1::try_new("wayne".to_owned(), "2616".to_owned()),
        Err(TerritoryCountyMapErrorV1::InvalidCountyGeoid)
    );
    assert_eq!(
        TerritoryCountyMapRowV1::try_new("wayne".to_owned(), "261633".to_owned()),
        Err(TerritoryCountyMapErrorV1::InvalidCountyGeoid)
    );
    assert_eq!(
        TerritoryCountyMapRowV1::try_new("wayne".to_owned(), "2616a".to_owned()),
        Err(TerritoryCountyMapErrorV1::InvalidCountyGeoid)
    );
    assert_eq!(
        TerritoryCountyMapRowV1::try_new(String::new(), "26163".to_owned()),
        Err(TerritoryCountyMapErrorV1::InvalidTerritoryLocalName)
    );
}
