//! Pure contract checks for PER-23 foundation knowledge-grant seeding.

use babylon_persistence::{
    foundation_grant_rows, foundation_grants_semantic_sha256, glossary_concepts,
    identity::CampaignId, seed_foundation_grants, ArchiveAtomSubjectKind, FoundationGrantsError,
    FOUNDATION_CONCEPT_GRANT_KEYS, FOUNDATION_COUNTY_GRANT_KEYS, FOUNDATION_COUNTY_LOCATOR_PREFIX,
    FOUNDATION_COUNTY_SOURCE_ID, FOUNDATION_GRANT_TICK,
    FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX, FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID,
    FOUNDATION_PLACE_GRANT_KEYS, FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX,
    FOUNDATION_PLACE_IDENTITY_SOURCE_ID, GLOSSARY_CONCEPTS_FIXTURE_PATH,
    PINNED_FOUNDATION_GRANTS_SEMANTIC_SHA256, PINNED_GLOSSARY_CONCEPTS_SHA256,
};

const EXPECTED_COUNTIES: usize = 83;
const EXPECTED_PLACES: usize = 745;
const EXPECTED_CONCEPTS: usize = 8;
const EXPECTED_GRANT_ROWS: usize =
    7 * EXPECTED_COUNTIES + 3 * EXPECTED_PLACES + 2 * EXPECTED_CONCEPTS;

#[test]
fn glossary_concepts_parse_to_the_pinned_corpus() {
    let concepts = glossary_concepts().expect("pinned glossary corpus parses");

    assert_eq!(concepts.concepts().len(), EXPECTED_CONCEPTS);
    let semantic = hex_lower(&concepts.semantic_sha256());
    assert_eq!(
        semantic,
        "d296f02168c66199168f732388abfeaf06d03932f784885884b82382b9454ebe"
    );
    let ids = concepts
        .concepts()
        .iter()
        .map(babylon_persistence::GlossaryConcept::concept_id)
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
            "census-identity",
            "class-composition",
            "containment",
            "employment",
            "existence",
            "identity",
            "median-wage",
            "phi-hour",
        ]
    );
    let median_wage = concepts
        .concepts()
        .iter()
        .find(|concept| concept.concept_id() == "median-wage")
        .expect("median-wage concept");
    assert_eq!(median_wage.display_label(), "Median wage");
    let phi_hour = concepts
        .concepts()
        .iter()
        .find(|concept| concept.concept_id() == "phi-hour")
        .expect("phi-hour concept");
    assert_eq!(phi_hour.display_label(), "Imperial rent Φ");
}

#[test]
fn pinned_products_cover_every_public_reference_subject() {
    let cohort = babylon_persistence::h3_reference_cohort::representative_h3_reference_cohort()
        .expect("pinned cohort");
    let products =
        babylon_persistence::spatial_reference_products::michigan_spatial_reference_products(
            cohort,
        )
        .expect("pinned products");

    let michigan_counties = products
        .counties()
        .iter()
        .filter(|county| county.county_geoid().starts_with("26") && county.county_fips() != "999")
        .count();
    assert_eq!(michigan_counties, EXPECTED_COUNTIES);
    assert_eq!(products.counties().len(), 3285);
    assert_eq!(products.places().len(), EXPECTED_PLACES);
    assert!(products
        .counties()
        .iter()
        .filter(|county| {
            county.county_geoid().starts_with("26") && county.county_fips() != "999"
        })
        .all(|county| county.county_geoid().starts_with("26")));
    assert!(products
        .places()
        .iter()
        .all(|place| place.place_geoid().starts_with("26")));
}

#[test]
fn foundation_citation_identities_are_stable() {
    assert_eq!(FOUNDATION_GRANT_TICK, 0);
    assert_eq!(FOUNDATION_COUNTY_SOURCE_ID, "h3-estate-contract-v1");
    assert_eq!(FOUNDATION_COUNTY_LOCATOR_PREFIX, "dim_county.parquet#fips=");
    assert_eq!(
        FOUNDATION_PLACE_IDENTITY_SOURCE_ID,
        "census-place-authority-v1"
    );
    assert_eq!(
        FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX,
        "census_place_identity_mi_2023.csv.gz#place_geoid="
    );
    assert_eq!(
        FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID,
        "county-place-h3-overlap-v1"
    );
    assert_eq!(
        FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX,
        "census_county_place_h3_land_overlap_mi_2023.parquet#place_geoid="
    );
    assert_eq!(
        FOUNDATION_COUNTY_GRANT_KEYS,
        ["subject", "identity", "containment"]
    );
    assert_eq!(
        FOUNDATION_PLACE_GRANT_KEYS,
        ["subject", "identity", "containment"]
    );
    assert_eq!(FOUNDATION_CONCEPT_GRANT_KEYS, ["subject", "identity"]);
    assert_eq!(
        GLOSSARY_CONCEPTS_FIXTURE_PATH,
        "contracts/fixtures/glossary_concepts_v1.jsonl"
    );
    let hex = hex_lower(&PINNED_GLOSSARY_CONCEPTS_SHA256);
    assert_eq!(
        hex,
        "f47e289dc4e7a11c595f0e42643e352e255775c77dde3a7ed35a91de8d84d85a"
    );
}

#[test]
fn magnitude_grant_keys_stay_ungranted_at_foundation() {
    for key in ["median-wage", "phi-hour", "class-composition", "employment"] {
        assert!(!FOUNDATION_COUNTY_GRANT_KEYS.contains(&key));
        assert!(!FOUNDATION_PLACE_GRANT_KEYS.contains(&key));
        assert!(!FOUNDATION_CONCEPT_GRANT_KEYS.contains(&key));
    }
}

#[test]
fn canonical_grant_rows_cover_exactly_the_public_reference_subjects() {
    let rows = foundation_grant_rows().expect("canonical grant rows build");
    assert_eq!(rows.len(), EXPECTED_GRANT_ROWS);
    let counties = rows
        .iter()
        .filter(|row| row.subject().kind() == ArchiveAtomSubjectKind::County)
        .count();
    let places = rows
        .iter()
        .filter(|row| row.subject().kind() == ArchiveAtomSubjectKind::Place)
        .count();
    let concepts = rows
        .iter()
        .filter(|row| row.subject().kind() == ArchiveAtomSubjectKind::Concept)
        .count();
    assert_eq!(counties, 7 * EXPECTED_COUNTIES);
    assert_eq!(places, 3 * EXPECTED_PLACES);
    assert_eq!(concepts, 2 * EXPECTED_CONCEPTS);
    for row in &rows {
        match row.subject().kind() {
            ArchiveAtomSubjectKind::County => {
                if row.grant_key().starts_with("qcew-") {
                    assert!([
                        "qcew-establishments",
                        "qcew-employment",
                        "qcew-total-annual-wages",
                        "qcew-average-weekly-wage"
                    ]
                    .contains(&row.grant_key()));
                    assert_eq!(row.citation().source_id(), "qcew-county-economics-v1");
                    assert_eq!(
                        row.citation().locator(),
                        format!(
                            "qcew_county_economics_mi_2024.csv.gz#county_geoid={}&sha256=116affb2998c6c0259d5bf14840f99f835d7e0733aa0b4f4c60a257b2723cd16",
                            row.subject().id()
                        )
                    );
                    continue;
                }
                assert_eq!(row.citation().source_id(), FOUNDATION_COUNTY_SOURCE_ID);
                assert_eq!(
                    row.citation().locator(),
                    format!("{FOUNDATION_COUNTY_LOCATOR_PREFIX}{}", row.subject().id())
                );
            }
            ArchiveAtomSubjectKind::Place => {
                let prefix = if row.grant_key() == "containment" {
                    FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX
                } else {
                    FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX
                };
                let source = if row.grant_key() == "containment" {
                    FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID
                } else {
                    FOUNDATION_PLACE_IDENTITY_SOURCE_ID
                };
                assert_eq!(row.citation().source_id(), source);
                assert_eq!(
                    row.citation().locator(),
                    format!("{prefix}{}", row.subject().id())
                );
            }
            ArchiveAtomSubjectKind::Concept => {
                assert_eq!(row.citation().source_id(), "glossary-concepts-v1");
            }
        }
    }
}

#[test]
fn canonical_grant_rows_recompute_the_pinned_semantic_digest() {
    let rows = foundation_grant_rows().expect("canonical grant rows build");
    let digest = foundation_grants_semantic_sha256(&rows);
    assert_eq!(
        digest, PINNED_FOUNDATION_GRANTS_SEMANTIC_SHA256,
        "the recomputed canonical grant-row digest must equal the contract pin"
    );
    assert_ne!(
        digest, [0x00; 32],
        "the digest pin must be harvested from failing-test output, not left as a placeholder"
    );
}

#[test]
fn seeding_needs_no_foundation_argument() {
    // The grant census derives entirely from pinned global artifacts, never
    // from scenario-authored grant lists (ADR249 R3), so the seeder takes no
    // foundation input: the signature binds one client and one campaign.
    let bound: fn(
        &mut postgres::Transaction<'_>,
        CampaignId,
    )
        -> Result<babylon_persistence::FoundationGrantReport, FoundationGrantsError> =
        seed_foundation_grants;
    let _ = bound;
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
