//! Foundation knowledge-grant seeding (ADR249 R3).
//!
//! At campaign foundation every public-reference subject — all 83 Michigan
//! counties and all 745 Michigan Census places from the pinned spatial
//! reference products, plus every glossary concept from the pinned fixture —
//! receives the cursory public-record grant keys at tick 0: `subject`,
//! `identity`, and `containment` for geography, `subject` and `identity`
//! for concepts. The four explicitly public QCEW 2024 baseline keys are also
//! granted for counties. Other composition and magnitude keys remain earned.

use postgres::Transaction;

use crate::archive::{
    insert_grant_row, validate_key, ArchiveAtomSubject, ArchiveAtomSubjectKind, ArchiveCitation,
    SemanticArchiveError,
};
use crate::glossary_concepts::{glossary_concepts, GlossaryConceptsError};
use crate::h3_reference_cohort::{representative_h3_reference_cohort, H3ReferenceCohortError};
use crate::identity::CampaignId;
use crate::michigan_economy::{
    michigan_economy, MichiganEconomyError, QCEW_ECONOMICS_ARTIFACT_SHA256,
    QCEW_ECONOMICS_FIELD_KEYS, QCEW_ECONOMICS_SOURCE_ID,
};
use crate::spatial_reference_products::{
    michigan_spatial_reference_products, SpatialReferenceProductsError,
};
use babylon_kernel::content_digest::sha256_of;

/// The foundation grant stamp: knowledge granted before any committed tick.
pub const FOUNDATION_GRANT_TICK: u64 = 0;

/// Source identity of the pinned `dim_county` artifact behind county grants.
pub const FOUNDATION_COUNTY_SOURCE_ID: &str = "h3-estate-contract-v1";
/// Locator prefix pinning one county identity row in `dim_county.parquet`.
pub const FOUNDATION_COUNTY_LOCATOR_PREFIX: &str = "dim_county.parquet#fips=";
/// Source identity of the pinned place identity contract artifact.
pub const FOUNDATION_PLACE_IDENTITY_SOURCE_ID: &str = "census-place-authority-v1";
/// Locator prefix pinning one place identity row.
pub const FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX: &str =
    "census_place_identity_mi_2023.csv.gz#place_geoid=";
/// Source identity of the pinned county/place overlap contract artifact.
pub const FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID: &str = "county-place-h3-overlap-v1";
/// Locator prefix pinning one place containment row in the overlap artifact.
pub const FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX: &str =
    "census_county_place_h3_land_overlap_mi_2023.parquet#place_geoid=";

/// Cursory public-record keys granted to every county at foundation.
pub const FOUNDATION_COUNTY_GRANT_KEYS: [&str; 3] = ["subject", "identity", "containment"];
/// Cursory public-record keys granted to every place at foundation.
pub const FOUNDATION_PLACE_GRANT_KEYS: [&str; 3] = ["subject", "identity", "containment"];
/// Cursory public-record keys granted to every glossary concept at foundation.
pub const FOUNDATION_CONCEPT_GRANT_KEYS: [&str; 2] = ["subject", "identity"];

/// One canonical foundation grant row (ADR249 R3): the subject address, the
/// grant key, and the pinned provenance citation. The granted tick is always
/// [`FOUNDATION_GRANT_TICK`] — knowledge granted before any committed tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundationGrantRow {
    subject: ArchiveAtomSubject,
    grant_key: String,
    citation: ArchiveCitation,
}

impl FoundationGrantRow {
    /// Validate one canonical foundation grant row.
    ///
    /// # Errors
    /// Refuses a malformed grant key; the subject and citation carry their
    /// own exact validation.
    pub fn try_new(
        subject: ArchiveAtomSubject,
        grant_key: String,
        citation: ArchiveCitation,
    ) -> Result<Self, SemanticArchiveError> {
        validate_key(&grant_key)?;
        Ok(Self {
            subject,
            grant_key,
            citation,
        })
    }

    /// Borrow the exact grant subject.
    #[must_use]
    pub const fn subject(&self) -> &ArchiveAtomSubject {
        &self.subject
    }

    /// Borrow the grant key.
    #[must_use]
    pub fn grant_key(&self) -> &str {
        &self.grant_key
    }

    /// Borrow the pinned provenance citation.
    #[must_use]
    pub const fn citation(&self) -> &ArchiveCitation {
        &self.citation
    }
}

/// Build the canonical foundation grant-row set from the pinned artifacts
/// only: the 83 Michigan counties and 745 Michigan places of the pinned
/// spatial reference products and the eight glossary concepts of the pinned
/// fixture. No scenario or foundation argument enters (ADR249 R3).
///
/// # Errors
/// Refuses pinned-fixture drift or a malformed derived identity.
pub fn foundation_grant_rows() -> Result<Vec<FoundationGrantRow>, FoundationGrantsError> {
    let cohort = representative_h3_reference_cohort()?;
    let products = michigan_spatial_reference_products(cohort)?;
    let concepts = glossary_concepts()?;
    let mut rows = Vec::with_capacity(2_832);
    for county in products
        .counties()
        .iter()
        .filter(|county| michigan_public_reference_county(county))
    {
        let subject = ArchiveAtomSubject::try_new(
            ArchiveAtomSubjectKind::County,
            county.county_geoid().to_owned(),
        )?;
        let citation = county_citation(county.county_geoid());
        for key in FOUNDATION_COUNTY_GRANT_KEYS {
            rows.push(FoundationGrantRow::try_new(
                subject.clone(),
                key.to_owned(),
                citation.clone(),
            )?);
        }
    }
    for county in michigan_economy()?.counties() {
        let subject = ArchiveAtomSubject::try_new(
            ArchiveAtomSubjectKind::County,
            county.county_geoid.clone(),
        )?;
        for key in QCEW_ECONOMICS_FIELD_KEYS {
            rows.push(FoundationGrantRow::try_new(
                subject.clone(),
                key.to_owned(),
                county_qcew_citation(&county.county_geoid),
            )?);
        }
    }
    for place in products.places() {
        let subject = ArchiveAtomSubject::try_new(
            ArchiveAtomSubjectKind::Place,
            place.place_geoid().to_owned(),
        )?;
        let identity = place_identity_citation(place.place_geoid());
        let containment = place_containment_citation(place.place_geoid());
        for key in FOUNDATION_PLACE_GRANT_KEYS {
            let citation = if key == "containment" {
                &containment
            } else {
                &identity
            };
            rows.push(FoundationGrantRow::try_new(
                subject.clone(),
                key.to_owned(),
                citation.clone(),
            )?);
        }
    }
    for concept in concepts.concepts() {
        let subject = ArchiveAtomSubject::try_new(
            ArchiveAtomSubjectKind::Concept,
            concept.concept_id().to_owned(),
        )?;
        for key in FOUNDATION_CONCEPT_GRANT_KEYS {
            rows.push(FoundationGrantRow::try_new(
                subject.clone(),
                key.to_owned(),
                concept.citation().clone(),
            )?);
        }
    }
    Ok(rows)
}

fn michigan_public_reference_county(
    county: &crate::spatial_reference_products::CountyIdentityRow,
) -> bool {
    county.county_geoid().starts_with(MICHIGAN_GEOID_PREFIX)
        && county.county_fips() != STATEWIDE_RESIDUAL_COUNTY_FIPS
}

/// Recompute the canonical semantic digest of the foundation grant-row set:
/// rows sorted by (subject kind, subject id, grant key), then the domain, the
/// u64 big-endian row count, and per row the length-prefixed subject kind,
/// subject id, grant key, citation source id, and citation locator, each in
/// UTF-8, followed by the u64 big-endian foundation grant tick.
///
/// # Panics
/// Panics only when a row count or field length exceeds u64, which the pinned
/// artifact bounds make unrepresentable.
#[must_use]
pub fn foundation_grants_semantic_sha256(rows: &[FoundationGrantRow]) -> [u8; 32] {
    let mut canonical = rows.to_vec();
    canonical.sort_by(|left, right| {
        left.subject
            .kind()
            .as_str()
            .cmp(right.subject.kind().as_str())
            .then_with(|| left.subject.id().cmp(right.subject.id()))
            .then_with(|| left.grant_key.cmp(&right.grant_key))
    });
    let mut bytes =
        Vec::with_capacity(FOUNDATION_GRANTS_SEMANTIC_DOMAIN.len() + 8 + canonical.len() * 64);
    bytes.extend_from_slice(FOUNDATION_GRANTS_SEMANTIC_DOMAIN);
    bytes.extend_from_slice(
        &u64::try_from(canonical.len())
            .expect("foundation grant rows fit u64")
            .to_be_bytes(),
    );
    for row in &canonical {
        for field in [
            row.subject.kind().as_str(),
            row.subject.id(),
            row.grant_key.as_str(),
            row.citation.source_id(),
            row.citation.locator(),
        ] {
            bytes.extend_from_slice(
                &u64::try_from(field.len())
                    .expect("foundation grant field fits u64")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(field.as_bytes());
        }
        bytes.extend_from_slice(&FOUNDATION_GRANT_TICK.to_be_bytes());
    }
    sha256_of(&bytes)
}

/// Census GEOID prefix of Michigan; only Michigan counties are
/// public-reference subjects in the v1 game world.
pub const MICHIGAN_GEOID_PREFIX: &str = "26";

/// The `999` county FIPS is a statewide residual pseudo-county in the pinned
/// `dim_county` artifact, not a public-reference county subject.
pub const STATEWIDE_RESIDUAL_COUNTY_FIPS: &str = "999";

/// Canonical semantic domain (with trailing NUL) of the foundation grant-row
/// digest pinned by `contracts/archive_foundation_grants_v1.yaml`.
pub const FOUNDATION_GRANTS_SEMANTIC_DOMAIN: &[u8] = b"babylon.archive-foundation-grants.v1\0";

/// Pinned SHA-256 of the canonical foundation grant-row encoding (ADR249 R3).
/// The independent Python verifier recomputes the identical digest over the
/// pinned spatial-products fixture and the glossary fixture; the value is
/// pinned here from failing-test output (never hand-edited) and mirrored by
/// `contracts/archive_foundation_grants_v1.yaml`.
pub const PINNED_FOUNDATION_GRANTS_SEMANTIC_SHA256: [u8; 32] = [
    0x9b, 0xb3, 0xcd, 0xda, 0x37, 0xdc, 0x3d, 0xee, 0x59, 0x24, 0x2f, 0xcf, 0xe0, 0x8f, 0x6c, 0x3f,
    0x5b, 0xac, 0x01, 0xff, 0x43, 0x39, 0x88, 0x38, 0x3d, 0x45, 0x9f, 0xa0, 0xdf, 0x9d, 0xfc, 0x65,
];

/// Closed refusal taxonomy for foundation grant seeding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FoundationGrantsError {
    /// The pinned H3 reference cohort fixture drifted.
    Cohort(H3ReferenceCohortError),
    /// The pinned spatial reference products fixture drifted.
    Spatial(SpatialReferenceProductsError),
    /// The pinned glossary concept fixture drifted.
    Concepts(GlossaryConceptsError),
    /// The pinned public QCEW county economics artifact drifted.
    Economics(MichiganEconomyError),
    /// One grant row refused validation, conflicted, or hit a database error.
    Archive(SemanticArchiveError),
}

impl From<MichiganEconomyError> for FoundationGrantsError {
    fn from(error: MichiganEconomyError) -> Self {
        Self::Economics(error)
    }
}

impl From<H3ReferenceCohortError> for FoundationGrantsError {
    fn from(error: H3ReferenceCohortError) -> Self {
        Self::Cohort(error)
    }
}

impl From<SpatialReferenceProductsError> for FoundationGrantsError {
    fn from(error: SpatialReferenceProductsError) -> Self {
        Self::Spatial(error)
    }
}

impl From<GlossaryConceptsError> for FoundationGrantsError {
    fn from(error: GlossaryConceptsError) -> Self {
        Self::Concepts(error)
    }
}

impl From<SemanticArchiveError> for FoundationGrantsError {
    fn from(error: SemanticArchiveError) -> Self {
        Self::Archive(error)
    }
}

impl std::fmt::Display for FoundationGrantsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "foundation grants refusal: {self:?}")
    }
}

impl std::error::Error for FoundationGrantsError {}

/// Exact foundation-seed census for one campaign.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoundationGrantReport {
    counties: usize,
    places: usize,
    concepts: usize,
    grant_rows: usize,
}

impl FoundationGrantReport {
    /// Number of county subjects granted.
    #[must_use]
    pub const fn counties(&self) -> usize {
        self.counties
    }

    /// Number of place subjects granted.
    #[must_use]
    pub const fn places(&self) -> usize {
        self.places
    }

    /// Number of glossary concept subjects granted.
    #[must_use]
    pub const fn concepts(&self) -> usize {
        self.concepts
    }

    /// Total immutable grant rows asserted.
    #[must_use]
    pub const fn grant_rows(&self) -> usize {
        self.grant_rows
    }
}

fn county_citation(county_geoid: &str) -> ArchiveCitation {
    ArchiveCitation::try_new(
        FOUNDATION_COUNTY_SOURCE_ID.to_owned(),
        format!("{FOUNDATION_COUNTY_LOCATOR_PREFIX}{county_geoid}"),
    )
    .expect("bounded county locator")
}

pub(crate) fn county_qcew_citation(county_geoid: &str) -> ArchiveCitation {
    ArchiveCitation::try_new(
        QCEW_ECONOMICS_SOURCE_ID.to_owned(),
        format!("qcew_county_economics_mi_2024.csv.gz#county_geoid={county_geoid}&sha256={QCEW_ECONOMICS_ARTIFACT_SHA256}"),
    ).expect("bounded county economics locator")
}

fn place_identity_citation(place_geoid: &str) -> ArchiveCitation {
    ArchiveCitation::try_new(
        FOUNDATION_PLACE_IDENTITY_SOURCE_ID.to_owned(),
        format!("{FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX}{place_geoid}"),
    )
    .expect("bounded place identity locator")
}

fn place_containment_citation(place_geoid: &str) -> ArchiveCitation {
    ArchiveCitation::try_new(
        FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID.to_owned(),
        format!("{FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX}{place_geoid}"),
    )
    .expect("bounded place containment locator")
}

/// Seed the foundation knowledge grants for one campaign.
///
/// The canonical row set comes from [`foundation_grant_rows`], so the
/// seeded census is exactly the digest-pinned grant-row set. Every insert is
/// idempotent with an exact-reconcile refusal, so an exact retry reports the
/// same census and a drifted row refuses loudly. The whole seed runs inside
/// the caller's open transaction, preserving whole-tick atomic publication
/// (ADR223).
///
/// # Errors
/// Refuses pinned-fixture drift, a malformed subject, a conflicting prior
/// grant row, or database failure.
pub fn seed_foundation_grants(
    client: &mut Transaction<'_>,
    campaign_id: CampaignId,
) -> Result<FoundationGrantReport, FoundationGrantsError> {
    crate::current_schema::require_current_schema(client)
        .map_err(SemanticArchiveError::CurrentSchema)?;
    let mut transaction = client
        .transaction()
        .map_err(|error| crate::archive::database("begin foundation grant savepoint", &error))?;
    let rows = foundation_grant_rows()?;
    let mut grant_rows = 0usize;
    for row in &rows {
        insert_grant_row(
            &mut transaction,
            campaign_id,
            row.subject().kind().as_str(),
            row.subject().id(),
            row.grant_key(),
            FOUNDATION_GRANT_TICK,
            row.citation(),
        )?;
        grant_rows += 1;
    }
    let counties = rows
        .iter()
        .filter(|row| row.subject().kind() == ArchiveAtomSubjectKind::County)
        .count()
        / (FOUNDATION_COUNTY_GRANT_KEYS.len() + QCEW_ECONOMICS_FIELD_KEYS.len());
    let places = rows
        .iter()
        .filter(|row| row.subject().kind() == ArchiveAtomSubjectKind::Place)
        .count()
        / FOUNDATION_PLACE_GRANT_KEYS.len();
    let concepts = rows
        .iter()
        .filter(|row| row.subject().kind() == ArchiveAtomSubjectKind::Concept)
        .count()
        / FOUNDATION_CONCEPT_GRANT_KEYS.len();
    transaction
        .commit()
        .map_err(|error| crate::archive::database("commit foundation grant savepoint", &error))?;
    Ok(FoundationGrantReport {
        counties,
        places,
        concepts,
        grant_rows,
    })
}
