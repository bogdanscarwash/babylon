//! PER-22 place dossier producer (slice 3) for the semantic Archive worker.
//!
//! [`PlaceDossierProducerV1`] turns one committed dirty receipt into a bounded
//! batch of place dossier pages resolved lazily from the checked Michigan
//! spatial reference products. Nothing is materialized into `PostgreSQL`
//! reference tables: place geometry and county overlap resolve from the
//! digest-pinned products fixture at produce time, so the receipt batch hash
//! folds the exact pinned artifact bytes in through the page content.
//!
//! # Page semantics
//!
//! Every page carries the census identity (`name_lsad`) as its title, one
//! grant-keyed `identity` signal whose citation pins the exact
//! `census_place_identity_mi_2023` artifact row, the stable place decision
//! question, and one link per overlapping county slice. Cross-county places
//! keep every slice — a place overlapping three counties links to all three,
//! sorted by county GEOID, and never collapses to a dominant county. No
//! committed place-level state exists (D2 absence-maximal ruling), so the
//! page publishes no other signals.
//!
//! # Dirty detection
//!
//! A place is dirty when no stored page exists for it or when its semantic
//! projection — `(place_geoid, title, decision question, grant-visible
//! signals, county link names)` — differs from the stored page's projection.
//! The projection folds the grant-visible rendering: the `identity` signal
//! counts only while the campaign grants that field, and a county link name
//! counts only while the campaign grants that county subject, both snapshotted
//! at the receipt tick through [`ARCHIVE_PLACE_GRANTS_SQL_V1`]. A page
//! published redacted therefore re-dirties the moment later grants reveal its
//! signal or link names, and the next sweep republishes it. Receipt-stamped
//! fields (`verified_tick`, `tick_content_hash`) never dirty a page: the
//! projection is recomputed from the stored Markdown with the pinned
//! `archive_page_v1.md.j2` shape ([`crate::ARCHIVE_PAGE_TEMPLATE_SHA256_V1`]
//! bytes folded into the hash), stripping the receipt-stamped frontmatter.
//! Malformed stored pages are treated as dirty, which safely republishes
//! drifted content.
//!
//! # Drain bound
//!
//! The producer never truncates a dirty set and never refuses an over-bound
//! one. When more than [`ArchiveDirtyBatchV1::MAX_PAGES`] places are dirty
//! for one receipt it selects the leading `limit` entries in geoid order and
//! reports the exact undrained tail count ([`ArchiveDirtySelectionV1`]), so
//! the sweep stages one bounded batch, the receipt stays pending, and the
//! next sweep continues the drain where the stored-current pages leave off.
//! The full 745-place campaign therefore materializes across successive
//! sweeps instead of dead-ending on the batch bound (PER-318). The bounded
//! allowlist ([`PlaceDossierProducerV1::with_place_allowlist`]) still covers
//! small fixture drains and rerun proofs.

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::tick_content_hash::RefDigestV1;
use postgres::Config;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

use crate::archive::{validate_text, ARCHIVE_PAGE_TEMPLATE_SHA256_V1};
use crate::{
    michigan_spatial_reference_products_v1, representative_h3_reference_cohort_v1,
    ArchiveCitationV1, ArchiveDirtySelectionV1, ArchiveDossierProducerV1, ArchiveLinkV1,
    ArchivePageInputV1, ArchivePageRefV1, ArchiveProducerOutcomeV1, ArchiveSignalV1,
    ArchiveSubjectKindV1, ArchiveSubjectV1, CampaignId, PendingArchiveReceiptV1, PlaceIdentityRow,
    SemanticArchiveErrorV1, SpatialReferenceProducts,
};

/// Stable decision question every place dossier page answers.
pub const PLACE_DECISION_QUESTION_V1: &str =
    "Which overlapping county should organizers investigate next?";

/// Source identity of the census place authority contract artifact.
pub const PLACE_IDENTITY_SOURCE_ID_V1: &str = "census-place-authority-v1";

/// Grant key addressing the one place identity signal.
pub const PLACE_IDENTITY_GRANT_KEY_V1: &str = "identity";

/// Player-facing label of the place identity signal.
pub const PLACE_IDENTITY_SIGNAL_LABEL_V1: &str = "Census identity";

/// Artifact locator prefix pinning one identity row.
pub const PLACE_IDENTITY_LOCATOR_PREFIX_V1: &str =
    "census_place_identity_mi_2023.csv.gz#place_geoid=";

/// Contract-pinned SHA-256 of `census_place_identity_mi_2023.csv.gz`
/// (`contracts/census_place_authority_v1.yaml`).
pub const PINNED_PLACE_IDENTITY_ARTIFACT_SHA256_V1: [u8; 32] = [
    0xcb, 0x86, 0x4b, 0x4f, 0x6f, 0x43, 0x90, 0x2b, 0xb8, 0x21, 0xe8, 0x4f, 0xe9, 0xa4, 0x05, 0x5a,
    0x90, 0x39, 0xe0, 0xa7, 0x4d, 0x8b, 0x83, 0x99, 0xf2, 0x09, 0xae, 0x6e, 0xd2, 0x6a, 0x8b, 0xe7,
];

/// Contract-pinned SHA-256 of
/// `census_county_place_h3_land_overlap_mi_2023.parquet`
/// (`contracts/county_place_h3_overlap_v1.yaml`).
pub const PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256_V1: [u8; 32] = [
    0xfc, 0xb7, 0xba, 0xaf, 0x63, 0xa5, 0x42, 0x2a, 0xcc, 0xce, 0x87, 0x09, 0x99, 0x7d, 0xe8, 0xe4,
    0x09, 0x93, 0x6f, 0x71, 0x31, 0xfa, 0x0e, 0xf6, 0xb0, 0xa2, 0x87, 0x62, 0xfd, 0xfe, 0xe4, 0x2f,
];

/// Read-only stored place-page projection used by the dirty diff.
///
/// The query returns the exact stored page rows for one campaign, ordered by
/// subject, and never joins material or raw event ledgers.
pub const ARCHIVE_PLACE_PAGE_READ_SQL_V1: &str = "SELECT subject_id, title, markdown \
FROM babylon_meta.archive_page_v1 \
WHERE campaign_id = $1::uuid AND subject_kind = 'place' \
ORDER BY subject_id";

/// Receipt-tick grant snapshot used by the dirty diff.
///
/// The query returns the exact campaign grant rows visible at the receipt
/// tick (`granted_tick <= $2`), mirroring the renderer's knowledge snapshot
/// semantics, and never joins material or raw event ledgers. Page-subject
/// knowledge only: seeded concept grants widen the grant table's subject
/// domain (ADR249 R3/R12) but never enter the page grant snapshot, which
/// decodes through the page-domain subject kind.
pub const ARCHIVE_PLACE_GRANTS_SQL_V1: &str = "SELECT subject_kind, subject_id, grant_key \
FROM babylon_meta.archive_knowledge_grant_v1 \
WHERE campaign_id = $1::uuid AND granted_tick <= $2 \
  AND subject_kind IN ('county', 'place') \
ORDER BY subject_kind, subject_id, grant_key";

/// Grant key that establishes knowledge of a page subject.
const PLACE_SUBJECT_GRANT_KEY_V1: &str = "subject";

const PLACE_SEMANTIC_DOMAIN_V1: &[u8] = b"babylon.place-page-semantic.v1\0";
const PLACE_PRODUCT_CODE_V1: &str = "census_place_identity_mi_2023";
const OVERLAP_PRODUCT_CODE_V1: &str = "census_county_place_h3_land_overlap_mi_2023";

/// One overlapping county slice of a place dossier page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceCountySliceV1 {
    county_geoid: String,
    county_name: String,
}

impl PlaceCountySliceV1 {
    /// Construct one county slice with its governed census county name.
    ///
    /// # Errors
    /// Refuses a malformed county GEOID or unsafe county name.
    pub fn try_new(
        county_geoid: String,
        county_name: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, county_geoid.clone())?;
        ArchiveSubjectV1::try_new(
            ArchiveSubjectKindV1::County,
            county_geoid.clone(),
            county_name.clone(),
        )?;
        Ok(Self {
            county_geoid,
            county_name,
        })
    }

    /// Borrow the five-digit county GEOID.
    #[must_use]
    pub fn county_geoid(&self) -> &str {
        &self.county_geoid
    }

    /// Borrow the governed census county name.
    #[must_use]
    pub fn county_name(&self) -> &str {
        &self.county_name
    }
}

/// One desired place dossier page resolved from the pinned products.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacePagePlanV1 {
    place_geoid: String,
    title: String,
    county_links: Vec<PlaceCountySliceV1>,
}

impl PlacePagePlanV1 {
    /// Construct one desired place page plan.
    ///
    /// # Errors
    /// Refuses a malformed place GEOID, an unsafe title, or duplicate county
    /// slices. County slices are stored sorted by county GEOID.
    pub fn try_new(
        place_geoid: String,
        title: String,
        county_links: Vec<PlaceCountySliceV1>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        ArchiveSubjectV1::try_new(
            ArchiveSubjectKindV1::Place,
            place_geoid.clone(),
            title.clone(),
        )?;
        let mut county_links = county_links;
        county_links.sort_by(|left, right| left.county_geoid.cmp(&right.county_geoid));
        if county_links
            .windows(2)
            .any(|pair| pair[0].county_geoid == pair[1].county_geoid)
        {
            return Err(SemanticArchiveErrorV1::DuplicateKey);
        }
        Ok(Self {
            place_geoid,
            title,
            county_links,
        })
    }

    /// Borrow the seven-digit place GEOID.
    #[must_use]
    pub fn place_geoid(&self) -> &str {
        &self.place_geoid
    }

    /// Borrow the census `name_lsad` page title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrow the sorted per-county slices.
    #[must_use]
    pub fn county_links(&self) -> &[PlaceCountySliceV1] {
        &self.county_links
    }
}

/// One grant-visible signal in the place page semantic projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceSignalProjectionV1 {
    label: String,
    value: String,
    source_id: String,
    locator: String,
}

impl PlaceSignalProjectionV1 {
    /// Construct one signal projection.
    ///
    /// # Errors
    /// Refuses unsafe text or a value that cannot round-trip through the
    /// pinned `archive_page_v1.md.j2` signal bullet delimiters.
    pub fn try_new(
        label: String,
        value: String,
        source_id: String,
        locator: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&label)?;
        validate_text(&value)?;
        validate_text(&source_id)?;
        validate_text(&locator)?;
        if label.contains(":** ") || value.contains(" — ") || source_id.contains("; ") {
            return Err(SemanticArchiveErrorV1::InvalidText);
        }
        Ok(Self {
            label,
            value,
            source_id,
            locator,
        })
    }

    /// Borrow the player-facing signal label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Borrow the signal value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Borrow the citation source identity.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Borrow the artifact locator.
    #[must_use]
    pub fn locator(&self) -> &str {
        &self.locator
    }
}

/// Receipt-stamp-free, grant-visible semantic projection of one place page.
///
/// This is the single projection the dirty diff hashes: the desired side
/// builds it from the plan plus the receipt-tick grant snapshot, and the
/// stored side parses it back out of the rendered Markdown. County names are
/// present only while the campaign grants the county subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacePageProjectionV1 {
    title: String,
    question: String,
    signals: Vec<PlaceSignalProjectionV1>,
    counties: Vec<(String, Option<String>)>,
}

impl PlacePageProjectionV1 {
    /// Construct one place page projection.
    ///
    /// # Errors
    /// Refuses unsafe text, a malformed county GEOID, or a repeated county.
    pub fn try_new(
        title: String,
        question: String,
        signals: Vec<PlaceSignalProjectionV1>,
        counties: Vec<(String, Option<String>)>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&title)?;
        validate_text(&question)?;
        let mut unique = BTreeSet::new();
        for (geoid, name) in &counties {
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, geoid.clone())?;
            if let Some(name) = name {
                validate_text(name)?;
            }
            if !unique.insert(geoid.clone()) {
                return Err(SemanticArchiveErrorV1::DuplicateKey);
            }
        }
        Ok(Self {
            title,
            question,
            signals,
            counties,
        })
    }

    /// Borrow the page title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrow the decision question.
    #[must_use]
    pub fn question(&self) -> &str {
        &self.question
    }

    /// Borrow the grant-visible signals in rendered order.
    #[must_use]
    pub fn signals(&self) -> &[PlaceSignalProjectionV1] {
        &self.signals
    }

    /// Borrow the sorted county links as `(geoid, known name)`.
    #[must_use]
    pub fn counties(&self) -> &[(String, Option<String>)] {
        &self.counties
    }
}

/// Receipt-tick snapshot of campaign knowledge grants for place pages.
///
/// The index answers exactly the two grant questions the place page
/// projection depends on: whether a place field (the `identity` signal) is
/// granted, and whether a county subject (the link name) is granted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaceGrantIndexV1 {
    grants: BTreeMap<ArchivePageRefV1, BTreeSet<String>>,
}

impl PlaceGrantIndexV1 {
    /// Index exact SQL grant rows decoded from
    /// [`ARCHIVE_PLACE_GRANTS_SQL_V1`].
    ///
    /// # Errors
    /// Refuses a malformed page identity or grant key.
    pub fn try_from_rows(
        rows: impl IntoIterator<Item = (ArchiveSubjectKindV1, String, String)>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        let mut grants: BTreeMap<ArchivePageRefV1, BTreeSet<String>> = BTreeMap::new();
        for (kind, id, grant_key) in rows {
            let page_ref = ArchivePageRefV1::try_new(kind, id)?;
            validate_text(&grant_key)?;
            grants.entry(page_ref).or_default().insert(grant_key);
        }
        Ok(Self { grants })
    }

    /// Return whether the snapshot grants one field key on one page.
    #[must_use]
    pub fn knows_field(&self, page_ref: &ArchivePageRefV1, grant_key: &str) -> bool {
        self.grants
            .get(page_ref)
            .is_some_and(|keys| keys.contains(grant_key))
    }

    /// Return whether the snapshot grants knowledge of one page subject.
    #[must_use]
    pub fn knows_subject(&self, page_ref: &ArchivePageRefV1) -> bool {
        self.knows_field(page_ref, PLACE_SUBJECT_GRANT_KEY_V1)
    }
}

/// Build the grant-visible desired projection for one place page plan.
///
/// The `identity` signal appears only while the snapshot grants that field on
/// the place; each county link carries its governed name only while the
/// snapshot grants that county subject.
///
/// # Errors
/// Refuses any unsafe projected component.
pub fn desired_place_projection_v1(
    plan: &PlacePagePlanV1,
    grants: &PlaceGrantIndexV1,
) -> Result<PlacePageProjectionV1, SemanticArchiveErrorV1> {
    let place_ref =
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, plan.place_geoid().to_owned())?;
    let signals = if grants.knows_field(&place_ref, PLACE_IDENTITY_GRANT_KEY_V1) {
        vec![PlaceSignalProjectionV1::try_new(
            PLACE_IDENTITY_SIGNAL_LABEL_V1.to_owned(),
            plan.title().to_owned(),
            PLACE_IDENTITY_SOURCE_ID_V1.to_owned(),
            format!("{PLACE_IDENTITY_LOCATOR_PREFIX_V1}{}", plan.place_geoid()),
        )?]
    } else {
        Vec::new()
    };
    let counties = plan
        .county_links()
        .iter()
        .map(|slice| {
            let county_ref = ArchivePageRefV1::try_new(
                ArchiveSubjectKindV1::County,
                slice.county_geoid().to_owned(),
            )?;
            Ok((
                slice.county_geoid().to_owned(),
                grants
                    .knows_subject(&county_ref)
                    .then(|| slice.county_name().to_owned()),
            ))
        })
        .collect::<Result<Vec<_>, SemanticArchiveErrorV1>>()?;
    PlacePageProjectionV1::try_new(
        plan.title().to_owned(),
        PLACE_DECISION_QUESTION_V1.to_owned(),
        signals,
        counties,
    )
}

/// Hash the exact receipt-stamp-free place page projection.
///
/// The projection covers the place GEOID, title, decision question, the
/// pinned page template identity ([`ARCHIVE_PAGE_TEMPLATE_SHA256_V1`]), the
/// ordered grant-visible signals with their full citations, and the sorted
/// county links including whether each name is visible. `verified_tick` and
/// `tick_content_hash` deliberately never enter the hash, so a later receipt
/// alone never re-publishes an unchanged page.
#[must_use]
pub fn place_page_semantic_sha256_v1(
    place_geoid: &str,
    projection: &PlacePageProjectionV1,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(PLACE_SEMANTIC_DOMAIN_V1);
    hash_text(&mut hasher, place_geoid);
    hash_text(&mut hasher, projection.title());
    hash_text(&mut hasher, projection.question());
    hasher.update(ARCHIVE_PAGE_TEMPLATE_SHA256_V1);
    hash_len(&mut hasher, projection.signals().len());
    for signal in projection.signals() {
        hash_text(&mut hasher, signal.label());
        hash_text(&mut hasher, signal.value());
        hash_text(&mut hasher, signal.source_id());
        hash_text(&mut hasher, signal.locator());
    }
    hash_len(&mut hasher, projection.counties().len());
    for (geoid, name) in projection.counties() {
        hash_text(&mut hasher, geoid);
        match name {
            Some(name) => {
                hasher.update([1]);
                hash_text(&mut hasher, name);
            }
            None => hasher.update([0]),
        }
    }
    hasher.finalize().into()
}

/// Reparse one `Related`-section subject-scheme link bullet (after the `- `
/// prefix) into its label and page key. The known form is
/// `[{label}](subject:{kind}/{id})`; the bare fog form is
/// `[](subject:{kind}/{id})` and carries no label bytes, so its label is
/// `None`. Any shape drift returns `None` so a stored page that no longer
/// matches the pinned renderer refuses the dirty-diff read.
fn parse_subject_scheme_link(entry: &str) -> Option<(Option<String>, &str)> {
    let (label, target) = entry.split_once("](subject:")?;
    let key = target.strip_suffix(')')?;
    let label = if label.is_empty() {
        None
    } else {
        Some(label.to_owned())
    };
    Some((label, key))
}

/// Parse the semantic projection out of one stored rendered place page.
///
/// The parser is coupled to the pinned `archive_page_v1.md.j2` template
/// ([`crate::ARCHIVE_PAGE_TEMPLATE_SHA256_V1`]) and returns `None` for any
/// stored page whose frontmatter subject, title, question, signal-bullet, or
/// related-link shape drifted; callers treat `None` as dirty.
#[must_use]
pub fn parse_stored_place_page_v1(
    place_geoid: &str,
    title: &str,
    markdown: &str,
) -> Option<PlacePageProjectionV1> {
    let mut lines = markdown.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut subject_exact = false;
    for line in lines.by_ref() {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("subject: ") {
            subject_exact = value == format!("place/{place_geoid}");
        }
    }
    if !subject_exact {
        return None;
    }
    let stored_title = lines.next()?.strip_prefix("# ")?.to_owned();
    if stored_title != title {
        return None;
    }
    let mut question = None;
    let mut signals = Vec::new();
    let mut counties = Vec::new();
    let mut in_signals = false;
    let mut in_related = false;
    for line in lines {
        if let Some(rest) = line.strip_prefix("## ") {
            in_signals = rest == "Signals";
            in_related = rest == "Related";
            continue;
        }
        if in_signals {
            if line.is_empty() {
                continue;
            }
            signals.push(parse_signal_bullet(line)?);
            continue;
        }
        if in_related {
            if let Some(entry) = line.strip_prefix("- [") {
                let (label, key) = parse_subject_scheme_link(entry)?;
                let county = key.strip_prefix("county/")?;
                if county.len() != 5 || !county.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                counties.push((county.to_owned(), label));
            }
            continue;
        }
        if line.starts_with("# ") || line.starts_with("- ") || line.is_empty() {
            continue;
        }
        if question.is_none() {
            question = Some(line.to_owned());
            continue;
        }
        return None;
    }
    PlacePageProjectionV1::try_new(stored_title, question?, signals, counties).ok()
}

/// Parse one pinned-template signal bullet.
fn parse_signal_bullet(line: &str) -> Option<PlaceSignalProjectionV1> {
    let rest = line.strip_prefix("- **")?;
    let (label, rest) = rest.split_once(":** ")?;
    let (value, citation) = rest.split_once(" — ")?;
    let (source_id, locator) = citation.split_once("; ")?;
    PlaceSignalProjectionV1::try_new(
        label.to_owned(),
        value.to_owned(),
        source_id.to_owned(),
        locator.to_owned(),
    )
    .ok()
}

/// Select the dirty desired pages, sorted by place GEOID, bounded by `limit`.
///
/// A desired page is dirty when no stored projection exists for its subject
/// or when the grant-visible projection hash differs. The selection never
/// truncates and never refuses: it takes the leading `limit` dirty entries
/// and reports the exact undrained tail count, so an over-bound dirty set
/// pages across successive sweeps — stored-current pages drop out of the
/// dirty set automatically, which is what advances the head each sweep.
///
/// # Errors
/// Returns any projection refusal.
pub fn select_dirty_place_pages_v1<'a>(
    desired: &'a [PlacePagePlanV1],
    stored: &BTreeMap<String, PlacePageProjectionV1>,
    grants: &PlaceGrantIndexV1,
    limit: usize,
) -> Result<ArchiveDirtySelectionV1<&'a PlacePagePlanV1>, SemanticArchiveErrorV1> {
    let mut dirty = Vec::new();
    for plan in desired {
        let projection = desired_place_projection_v1(plan, grants)?;
        let is_dirty = stored.get(plan.place_geoid()).is_none_or(|page| {
            place_page_semantic_sha256_v1(plan.place_geoid(), page)
                != place_page_semantic_sha256_v1(plan.place_geoid(), &projection)
        });
        if is_dirty {
            dirty.push(plan);
        }
    }
    let remaining = dirty.len().saturating_sub(limit);
    dirty.truncate(limit);
    Ok(ArchiveDirtySelectionV1::new(dirty, remaining))
}

/// Build the exact receipt-bound page input for one desired place page.
///
/// # Errors
/// Refuses any unsafe page component.
pub fn place_page_input_v1(
    plan: &PlacePagePlanV1,
    resolve_tick: u64,
    tick_content_hash: [u8; 32],
) -> Result<ArchivePageInputV1, SemanticArchiveErrorV1> {
    let subject = ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::Place,
        plan.place_geoid.clone(),
        plan.title.clone(),
    )?;
    let signal = ArchiveSignalV1::try_new(
        PLACE_IDENTITY_GRANT_KEY_V1.to_owned(),
        PLACE_IDENTITY_SIGNAL_LABEL_V1.to_owned(),
        plan.title.clone(),
        ArchiveCitationV1::try_new(
            PLACE_IDENTITY_SOURCE_ID_V1.to_owned(),
            format!("{PLACE_IDENTITY_LOCATOR_PREFIX_V1}{}", plan.place_geoid),
        )?,
    )?;
    let links = plan
        .county_links()
        .iter()
        .map(|slice| {
            ArchiveLinkV1::try_new(
                ArchivePageRefV1::try_new(
                    ArchiveSubjectKindV1::County,
                    slice.county_geoid.clone(),
                )?,
                slice.county_name.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    ArchivePageInputV1::try_new(
        subject,
        resolve_tick,
        tick_content_hash,
        PLACE_DECISION_QUESTION_V1.to_owned(),
        vec![signal],
        links,
    )
}

/// Production place dossier producer over the checked reference products.
pub struct PlaceDossierProducerV1 {
    config: Config,
    products: SpatialReferenceProducts,
    allowlist: Option<Vec<String>>,
}

impl PlaceDossierProducerV1 {
    /// Load the checked reference products and bind the stored-page reader.
    ///
    /// # Errors
    /// Refuses loudly when the embedded reference products, their governing
    /// H3 cohort, or either contract-pinned artifact digest diverges.
    pub fn try_new(config: &Config) -> Result<Self, SemanticArchiveErrorV1> {
        let cohort = representative_h3_reference_cohort_v1()
            .map_err(|_| SemanticArchiveErrorV1::ArtifactDigest)?;
        let products = michigan_spatial_reference_products_v1(cohort)
            .map_err(|_| SemanticArchiveErrorV1::ArtifactDigest)?;
        verify_pinned_artifact_digests(&products)?;
        Ok(Self {
            config: config.clone(),
            products,
            allowlist: None,
        })
    }

    /// Bind a test-facing producer over a sorted, unique place GEOID subset.
    ///
    /// The allowlist must be sorted ascending, unique, and seven-digit
    /// numeric, and every member must be one of the pinned fixture places;
    /// anything else refuses with
    /// [`SemanticArchiveErrorV1::InvalidIdentity`]. Only allowlisted places
    /// enumerate, which keeps small drains single-sweep for live proofs.
    /// Production keeps the full fixture through [`PlaceDossierProducerV1::try_new`];
    /// larger dirty sets page across sweeps under the shared page budget.
    ///
    /// # Errors
    /// Refuses unsorted, duplicated, malformed, or unknown GEOIDs, or any
    /// pinned product divergence.
    pub fn with_place_allowlist(
        config: &Config,
        geoids: &[String],
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if geoids.is_empty()
            || geoids.windows(2).any(|pair| pair[0] >= pair[1])
            || geoids
                .iter()
                .any(|geoid| geoid.len() != 7 || !geoid.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return Err(SemanticArchiveErrorV1::InvalidIdentity);
        }
        let mut producer = Self::try_new(config)?;
        let known: BTreeSet<&str> = producer
            .products
            .places()
            .iter()
            .map(PlaceIdentityRow::place_geoid)
            .collect();
        if geoids.iter().any(|geoid| !known.contains(geoid.as_str())) {
            return Err(SemanticArchiveErrorV1::InvalidIdentity);
        }
        producer.allowlist = Some(geoids.to_vec());
        Ok(producer)
    }

    /// Resolve every desired place page from the pinned products.
    ///
    /// The result is sorted by place GEOID. Each place keeps every
    /// overlapping county slice; no county membership is collapsed. An
    /// allowlist-bound producer enumerates only its allowlisted places.
    ///
    /// # Errors
    /// Refuses unsafe product text; the checked fixture already pinned every
    /// row count, subject, and measure during construction.
    pub fn desired_pages(&self) -> Result<Vec<PlacePagePlanV1>, SemanticArchiveErrorV1> {
        let county_names = self
            .products
            .counties()
            .iter()
            .map(|county| (county.county_geoid(), county.county_name()))
            .collect::<BTreeMap<_, _>>();
        let slices = self.products.county_place_land_areas().iter().fold(
            BTreeMap::<&str, BTreeSet<&str>>::new(),
            |mut slices, row| {
                slices
                    .entry(row.place_geoid())
                    .or_default()
                    .insert(row.county_geoid());
                slices
            },
        );
        self.products
            .places()
            .iter()
            .filter(|place| {
                self.allowlist
                    .as_ref()
                    .is_none_or(|allowlist| allowlist.contains(&place.place_geoid().to_owned()))
            })
            .map(|place| {
                let county_links = slices
                    .get(place.place_geoid())
                    .into_iter()
                    .flatten()
                    .map(|county_geoid| {
                        let county_name = county_names
                            .get(county_geoid)
                            .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
                        PlaceCountySliceV1::try_new(
                            (*county_geoid).to_owned(),
                            (*county_name).to_owned(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                PlacePagePlanV1::try_new(
                    place.place_geoid().to_owned(),
                    place.name_lsad().to_owned(),
                    county_links,
                )
            })
            .collect()
    }
}

impl ArchiveDossierProducerV1 for PlaceDossierProducerV1 {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &crate::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let campaign = CampaignId::from_uuid(campaign_id);
        let desired = self.desired_pages()?;
        crate::archive_revision::publication::select_dirty_pages(
            &self.config,
            campaign,
            receipt,
            knowledge,
            &desired,
            page_budget,
            place_page_input_v1,
        )
    }
    fn cutover_subjects(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &crate::ArchiveKnowledgeV1,
    ) -> Result<Vec<ArchivePageRefV1>, SemanticArchiveErrorV1> {
        let desired = self.desired_pages()?;
        let mut subjects = std::collections::BTreeSet::new();
        for plan in &desired {
            let page =
                place_page_input_v1(plan, receipt.resolve_tick(), *receipt.tick_content_hash())?;
            if knowledge.knows_subject(page.subject().page_ref()) {
                subjects.insert(page.subject().page_ref().clone());
            }
        }
        Ok(subjects.into_iter().collect())
    }
}

fn verify_pinned_artifact_digests(
    products: &SpatialReferenceProducts,
) -> Result<(), SemanticArchiveErrorV1> {
    for (code, pinned) in [
        (
            PLACE_PRODUCT_CODE_V1,
            PINNED_PLACE_IDENTITY_ARTIFACT_SHA256_V1,
        ),
        (
            OVERLAP_PRODUCT_CODE_V1,
            PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256_V1,
        ),
    ] {
        let product = products
            .products()
            .iter()
            .find(|product| product.code() == code)
            .ok_or(SemanticArchiveErrorV1::ArtifactDigest)?;
        if product.artifact_sha256() != RefDigestV1::from_bytes(pinned) {
            return Err(SemanticArchiveErrorV1::ArtifactDigest);
        }
    }
    Ok(())
}

fn hash_text(hasher: &mut Sha256, text: &str) {
    hash_len(hasher, text.len());
    hasher.update(text.as_bytes());
}

fn hash_len(hasher: &mut Sha256, len: usize) {
    hasher.update(u64::try_from(len).unwrap_or(u64::MAX).to_be_bytes());
}
