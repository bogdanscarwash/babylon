//! PER-22 county dossier producer (slice 2) for the semantic Archive worker.
//!
//! [`CountyDossierProducerV1`] turns one committed dirty receipt into a
//! bounded batch of county dossier pages. Counties enumerate from the
//! campaign's declared `babylon_meta.territory_county_map_v1` rows in GEOID
//! order; titles, place links, and place labels resolve from the checked,
//! digest-pinned Michigan spatial reference products, so the receipt batch
//! hash folds the exact pinned artifact bytes in through the page content.
//!
//! # Page semantics
//!
//! Each page carries the governed census county name as its title, the
//! established county decision question, one grant-keyed signal per committed
//! per-tick territory field that exists at the receipt's resolve tick —
//! the four public QCEW baseline integer fields, or the established
//! `territory/median-wage` and `territory/phi-hour` real fields — and one link
//! per place overlapping the county. A committed field that is missing at the
//! resolve tick emits no signal, never a fabricated value. Signal values are
//! exact decimal integers for QCEW and `%.6f` for the established real fields.
//! Public baseline citations pin the QCEW artifact and county row; established
//! real-field citations pin `campaign/{resolve_tick}/{territory local name}`.
//! Unknown place subjects
//! stay redlinks because the renderer hides labels for subjects no knowledge
//! grant covers.
//!
//! # Dirty detection
//!
//! A county is dirty when no stored page exists for it or when its semantic
//! projection — `(county_geoid, title, decision question, grant-visible
//! signals with citations, place link names)` — differs from the stored page's
//! projection. The projection folds the grant-visible rendering: each signal
//! counts only while the campaign grants that field key on the county, and a
//! place link name counts only while the campaign grants that place subject,
//! both snapshotted at the receipt tick through [`ARCHIVE_COUNTY_GRANTS_SQL_V1`].
//! A page published redacted therefore re-dirties the moment later grants
//! reveal its signals or link names, and the next sweep republishes it.
//! Receipt-stamped fields (`verified_tick`, `tick_content_hash`, and the tick
//! segment of each citation locator) never dirty a page: the projection is
//! recomputed from the stored Markdown with the pinned `archive_page_v1.md.j2`
//! shape ([`crate::ARCHIVE_PAGE_TEMPLATE_SHA256_V1`] bytes folded into the
//! hash), stripping the receipt-stamped frontmatter. Malformed stored pages
//! are treated as dirty, which safely republishes drifted content.
//!
//! # Drain bound
//!
//! The producer never truncates a dirty set and never refuses an over-bound
//! one. When more than [`ArchiveDirtyBatchV1::MAX_PAGES`] counties are dirty
//! for one receipt it selects the leading `limit` entries in geoid order and
//! reports the exact undrained tail count ([`ArchiveDirtySelectionV1`]), so
//! the sweep stages one bounded batch and the receipt stays pending until
//! the tail drains across successive sweeps. Michigan's 83 mapped counties
//! keep the production county drain a normal complete batch, leaving most of
//! the shared page budget for the place producer (PER-318).

use std::collections::{BTreeMap, BTreeSet};

use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::tick_content_hash::RefDigestV1;
use postgres::{Config, NoTls};
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

use crate::archive::{database, decode, validate_text, ARCHIVE_PAGE_TEMPLATE_SHA256_V1};
use crate::archive_foundation_grants::county_qcew_citation;
use crate::michigan_economy::{
    QCEW_ECONOMICS_ARTIFACT_SHA256_V1, QCEW_ECONOMICS_FIELD_KEYS_V1, QCEW_ECONOMICS_SOURCE_ID_V1,
};
use crate::place_producer::{
    PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256_V1, PINNED_PLACE_IDENTITY_ARTIFACT_SHA256_V1,
};
use crate::{
    michigan_spatial_reference_products_v1, representative_h3_reference_cohort_v1,
    ArchiveCitationV1, ArchiveDirtySelectionV1, ArchiveDossierProducerV1, ArchiveLinkV1,
    ArchivePageInputV1, ArchivePageRefV1, ArchiveProducerOutcomeV1, ArchiveSignalV1,
    ArchiveSubjectKindV1, ArchiveSubjectV1, CampaignId, PendingArchiveReceiptV1,
    SemanticArchiveErrorV1, SpatialReferenceProducts,
};

/// Stable decision question every county dossier page answers.
pub const COUNTY_DECISION_QUESTION_V1: &str =
    "Which neighboring place should organizers investigate next?";

/// Knowledge-grant key addressing the committed median-wage signal.
pub const COUNTY_MEDIAN_WAGE_GRANT_KEY_V1: &str = "median-wage";

/// Knowledge-grant key addressing the committed phi-hour signal.
pub const COUNTY_PHI_HOUR_GRANT_KEY_V1: &str = "phi-hour";

/// Player-facing label of the committed median-wage signal.
pub const COUNTY_MEDIAN_WAGE_LABEL_V1: &str = "Median wage";

/// Player-facing label of the committed phi-hour signal.
pub const COUNTY_PHI_HOUR_LABEL_V1: &str = "Imperial rent Φ";

/// Source identity pinning committed per-tick territory provenance.
pub const COMMITTED_TICK_SOURCE_ID_V1: &str = "committed-tick-v1";

/// Read-only declared county enumeration used by the county dossier.
pub const ARCHIVE_COUNTY_MAP_READ_SQL_V1: &str = "SELECT territory_local_name, county_geoid \
FROM babylon_meta.territory_county_map_v1 \
WHERE campaign_id = $1::uuid ORDER BY county_geoid, territory_local_name";

/// Read-only committed per-tick territory fields used by the county dossier.
///
/// The exact observed baseline and established real fields use their stored
/// scenario-local field names (`territory_state_field_v1` persists the local
/// name, not the declared `territory/` path); every row comes from the
/// committed tick the receipt names, never from material ledgers or
/// `MAX(tick)` shortcuts.
pub const ARCHIVE_COUNTY_FIELD_READ_SQL_V1: &str = "SELECT \
    t.territory_id, f.field_name, f.value_tag, f.real_bits, f.int_value \
    FROM babylon_state.territory_state_v1 t \
    JOIN babylon_state.territory_state_field_v1 f \
      ON f.campaign_id = t.campaign_id \
     AND f.resolve_tick = t.resolve_tick \
     AND f.territory_id = t.territory_id \
    WHERE t.campaign_id = $1::uuid AND t.resolve_tick = $2 \
      AND f.field_name IN ('median-wage', 'phi-hour', 'qcew-establishments', \
          'qcew-employment', 'qcew-total-annual-wages', 'qcew-average-weekly-wage') \
    ORDER BY t.territory_id, f.position";

/// Read-only stored county-page projection used by the dirty diff.
///
/// The query returns the exact stored page rows for one campaign, ordered by
/// subject, and never joins material or raw event ledgers.
pub const ARCHIVE_COUNTY_PAGE_READ_SQL_V1: &str = "SELECT subject_id, title, markdown \
FROM babylon_meta.archive_page_v1 \
WHERE campaign_id = $1::uuid AND subject_kind = 'county' \
ORDER BY subject_id";

/// Receipt-tick grant snapshot used by the dirty diff.
///
/// The query returns the exact campaign grant rows visible at the receipt
/// tick (`granted_tick <= $2`), mirroring the renderer's knowledge snapshot
/// semantics, and never joins material or raw event ledgers. Page-subject
/// knowledge only: seeded concept grants widen the grant table's subject
/// domain (ADR249 R3/R12) but never enter the page grant snapshot, which
/// decodes through the page-domain subject kind.
pub const ARCHIVE_COUNTY_GRANTS_SQL_V1: &str = "SELECT subject_kind, subject_id, grant_key \
FROM babylon_meta.archive_knowledge_grant_v1 \
WHERE campaign_id = $1::uuid AND granted_tick <= $2 \
  AND subject_kind IN ('county', 'place') \
ORDER BY subject_kind, subject_id, grant_key";

/// Contract-pinned SHA-256 of the `dim_county` identity artifact that backs
/// the governed county names.
pub const PINNED_COUNTY_IDENTITY_ARTIFACT_SHA256_V1: [u8; 32] = [
    0x13, 0x0b, 0x76, 0x79, 0xd0, 0x44, 0x1d, 0x5c, 0x3c, 0x21, 0x83, 0xa2, 0xbe, 0xf8, 0x58, 0x07,
    0x3d, 0x30, 0x11, 0x03, 0x95, 0x50, 0xbf, 0xbf, 0x01, 0x5b, 0x38, 0x05, 0x66, 0xc7, 0x20, 0x32,
];

const COUNTY_SEMANTIC_DOMAIN_V1: &[u8] = b"babylon.county-page-semantic.v1\0";
const COUNTY_IDENTITY_PRODUCT_CODE_V1: &str = "dim_county";
const PLACE_IDENTITY_PRODUCT_CODE_V1: &str = "census_place_identity_mi_2023";
const OVERLAP_PRODUCT_CODE_V1: &str = "census_county_place_h3_land_overlap_mi_2023";
const COUNTY_SUBJECT_GRANT_KEY_V1: &str = "subject";
const MEDIAN_WAGE_FIELD_V1: &str = "median-wage";
const PHI_HOUR_FIELD_V1: &str = "phi-hour";
const REAL_VALUE_TAG_V1: i16 = 3;
const INT_VALUE_TAG_V1: i16 = 1;
const QCEW_LABELS_V1: [&str; 4] = [
    "QCEW 2024 annual-average establishments",
    "QCEW 2024 annual-average employment (jobs)",
    "QCEW 2024 total annual wages (USD)",
    "QCEW 2024 average weekly wage (USD/week)",
];

/// Format one committed real with the Python statblock's `%.6f` discipline.
///
/// Negative zero canonicalizes to positive zero before formatting so a
/// sign-only bit difference never re-publishes a page; committed state
/// hashing treats `-0.0 == 0.0` and the rendered page follows that identity.
///
/// # Errors
/// Refuses a non-finite value; committed values are canonical finite binary64
/// and any other input is a malformed committed state, never a display value.
pub fn format_county_statblock_value_v1(value: f64) -> Result<String, SemanticArchiveErrorV1> {
    if !value.is_finite() {
        return Err(SemanticArchiveErrorV1::InvalidText);
    }
    let canonical = if value == 0.0 { 0.0 } else { value };
    Ok(format!("{canonical:.6}"))
}

/// One grant-keyed county signal with its pre-formatted statblock value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountySignalV1 {
    grant_key: String,
    label: String,
    value: String,
}

impl CountySignalV1 {
    /// Construct one bounded county signal.
    ///
    /// # Errors
    /// Refuses an unsafe grant key, label, or value. The grant key is
    /// revalidated as a strict key when the page input is assembled.
    pub fn try_new(
        grant_key: String,
        label: String,
        value: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&grant_key)?;
        validate_text(&label)?;
        validate_text(&value)?;
        Ok(Self {
            grant_key,
            label,
            value,
        })
    }

    /// Construct one committed-real signal with `%.6f` formatting.
    ///
    /// # Errors
    /// Refuses a non-finite committed value or unsafe text.
    pub fn from_committed_real(
        grant_key: String,
        label: String,
        value: f64,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        Self::try_new(grant_key, label, format_county_statblock_value_v1(value)?)
    }

    /// Borrow the knowledge-grant address key.
    #[must_use]
    pub fn grant_key(&self) -> &str {
        &self.grant_key
    }

    /// Borrow the player-facing signal label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Borrow the pre-formatted player-facing value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// One overlapping place link of a county dossier page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountyPlaceLinkV1 {
    place_geoid: String,
    place_name: String,
}

impl CountyPlaceLinkV1 {
    /// Construct one place link with its governed census place label.
    ///
    /// # Errors
    /// Refuses a malformed place GEOID or unsafe place label.
    pub fn try_new(
        place_geoid: String,
        place_name: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, place_geoid.clone())?;
        validate_text(&place_name)?;
        Ok(Self {
            place_geoid,
            place_name,
        })
    }

    /// Borrow the seven-digit place GEOID.
    #[must_use]
    pub fn place_geoid(&self) -> &str {
        &self.place_geoid
    }

    /// Borrow the governed census place label.
    #[must_use]
    pub fn place_name(&self) -> &str {
        &self.place_name
    }
}

/// One desired county dossier page resolved from the declared mapping, the
/// committed per-tick territory fields, and the pinned products.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountyPagePlanV1 {
    county_geoid: String,
    territory_local_name: String,
    title: String,
    signals: Vec<CountySignalV1>,
    place_links: Vec<CountyPlaceLinkV1>,
}

impl CountyPagePlanV1 {
    /// Construct one desired county page plan.
    ///
    /// # Errors
    /// Refuses a malformed county GEOID, unsafe text, or duplicate signal
    /// keys or link targets. Signals are stored sorted by grant key and links
    /// sorted by place GEOID.
    pub fn try_new(
        county_geoid: String,
        territory_local_name: String,
        title: String,
        signals: Vec<CountySignalV1>,
        place_links: Vec<CountyPlaceLinkV1>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        ArchiveSubjectV1::try_new(
            ArchiveSubjectKindV1::County,
            county_geoid.clone(),
            title.clone(),
        )?;
        validate_text(&territory_local_name)?;
        let mut signals = signals;
        signals.sort_by(|left, right| left.grant_key.cmp(&right.grant_key));
        if signals
            .windows(2)
            .any(|pair| pair[0].grant_key == pair[1].grant_key)
        {
            return Err(SemanticArchiveErrorV1::DuplicateKey);
        }
        let mut place_links = place_links;
        place_links.sort_by(|left, right| left.place_geoid.cmp(&right.place_geoid));
        if place_links
            .windows(2)
            .any(|pair| pair[0].place_geoid == pair[1].place_geoid)
        {
            return Err(SemanticArchiveErrorV1::DuplicateKey);
        }
        Ok(Self {
            county_geoid,
            territory_local_name,
            title,
            signals,
            place_links,
        })
    }

    /// Borrow the five-digit county GEOID.
    #[must_use]
    pub fn county_geoid(&self) -> &str {
        &self.county_geoid
    }

    /// Borrow the scenario-local territory name the mapping declares.
    #[must_use]
    pub fn territory_local_name(&self) -> &str {
        &self.territory_local_name
    }

    /// Borrow the governed census county title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrow the sorted grant-keyed signals.
    #[must_use]
    pub fn signals(&self) -> &[CountySignalV1] {
        &self.signals
    }

    /// Borrow the sorted overlapping place links.
    #[must_use]
    pub fn place_links(&self) -> &[CountyPlaceLinkV1] {
        &self.place_links
    }
}

/// One grant-visible signal in the county page semantic projection.
///
/// The `provenance_name` is the receipt-independent identity of the committed
/// provenance locator (`campaign/{receipt tick}/{territory local name}`); the
/// receipt tick segment is a receipt stamp and never enters the projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountySignalProjectionV1 {
    label: String,
    value: String,
    source_id: String,
    provenance_name: String,
}

impl CountySignalProjectionV1 {
    /// Construct one signal projection.
    ///
    /// # Errors
    /// Refuses unsafe text or a value that cannot round-trip through the
    /// pinned `archive_page_v1.md.j2` signal bullet delimiters.
    pub fn try_new(
        label: String,
        value: String,
        source_id: String,
        provenance_name: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&label)?;
        validate_text(&value)?;
        validate_text(&source_id)?;
        validate_text(&provenance_name)?;
        if label.contains(":** ") || value.contains(" — ") || source_id.contains("; ") {
            return Err(SemanticArchiveErrorV1::InvalidText);
        }
        Ok(Self {
            label,
            value,
            source_id,
            provenance_name,
        })
    }

    /// Borrow the player-facing signal label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Borrow the pre-formatted signal value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Borrow the citation source identity.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Borrow the receipt-independent provenance name.
    #[must_use]
    pub fn provenance_name(&self) -> &str {
        &self.provenance_name
    }
}

/// Receipt-stamp-free, grant-visible semantic projection of one county page.
///
/// This is the single projection the dirty diff hashes: the desired side
/// builds it from the plan plus the receipt-tick grant snapshot, and the
/// stored side parses it back out of the rendered Markdown. Place names are
/// present only while the campaign grants the place subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountyPageProjectionV1 {
    title: String,
    question: String,
    signals: Vec<CountySignalProjectionV1>,
    places: Vec<(String, Option<String>)>,
}

impl CountyPageProjectionV1 {
    /// Construct one county page projection.
    ///
    /// # Errors
    /// Refuses unsafe text, a malformed place GEOID, a repeated signal label,
    /// or a repeated place link.
    pub fn try_new(
        title: String,
        question: String,
        signals: Vec<CountySignalProjectionV1>,
        places: Vec<(String, Option<String>)>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&title)?;
        validate_text(&question)?;
        let mut signal_labels = BTreeSet::new();
        for signal in &signals {
            if !signal_labels.insert(signal.label.clone()) {
                return Err(SemanticArchiveErrorV1::DuplicateKey);
            }
        }
        let mut unique = BTreeSet::new();
        for (geoid, name) in &places {
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, geoid.clone())?;
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
            places,
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
    pub fn signals(&self) -> &[CountySignalProjectionV1] {
        &self.signals
    }

    /// Borrow the sorted place links as `(geoid, known name)`.
    #[must_use]
    pub fn places(&self) -> &[(String, Option<String>)] {
        &self.places
    }
}

/// Receipt-tick snapshot of campaign knowledge grants for county pages.
///
/// The index answers exactly the two grant questions the county page
/// projection depends on: whether a county field (one committed signal) is
/// granted, and whether a place subject (one link name) is granted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountyGrantIndexV1 {
    grants: BTreeMap<ArchivePageRefV1, BTreeSet<String>>,
}

impl CountyGrantIndexV1 {
    /// Index exact SQL grant rows decoded from [`ARCHIVE_COUNTY_GRANTS_SQL_V1`].
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
        self.knows_field(page_ref, COUNTY_SUBJECT_GRANT_KEY_V1)
    }
}

/// Build the grant-visible desired projection for one county page plan.
///
/// Each committed signal appears only while the snapshot grants that field
/// key on the county; each place link carries its governed name only while
/// the snapshot grants that place subject.
///
/// # Errors
/// Refuses any unsafe projected component.
pub fn desired_county_projection_v1(
    plan: &CountyPagePlanV1,
    grants: &CountyGrantIndexV1,
) -> Result<CountyPageProjectionV1, SemanticArchiveErrorV1> {
    let county_ref =
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, plan.county_geoid().to_owned())?;
    let signals = plan
        .signals()
        .iter()
        .filter(|signal| grants.knows_field(&county_ref, signal.grant_key()))
        .map(|signal| {
            let (source_id, provenance_name) =
                if QCEW_ECONOMICS_FIELD_KEYS_V1.contains(&signal.grant_key()) {
                    let citation = county_qcew_citation(plan.county_geoid());
                    (
                        citation.source_id().to_owned(),
                        citation.locator().to_owned(),
                    )
                } else {
                    (
                        COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
                        plan.territory_local_name().to_owned(),
                    )
                };
            CountySignalProjectionV1::try_new(
                signal.label().to_owned(),
                signal.value().to_owned(),
                source_id,
                provenance_name,
            )
        })
        .collect::<Result<Vec<_>, SemanticArchiveErrorV1>>()?;
    let places = plan
        .place_links()
        .iter()
        .map(|link| {
            let place_ref = ArchivePageRefV1::try_new(
                ArchiveSubjectKindV1::Place,
                link.place_geoid().to_owned(),
            )?;
            Ok((
                link.place_geoid().to_owned(),
                grants
                    .knows_subject(&place_ref)
                    .then(|| link.place_name().to_owned()),
            ))
        })
        .collect::<Result<Vec<_>, SemanticArchiveErrorV1>>()?;
    CountyPageProjectionV1::try_new(
        plan.title().to_owned(),
        COUNTY_DECISION_QUESTION_V1.to_owned(),
        signals,
        places,
    )
}

/// Hash the exact receipt-stamp-free county page projection.
///
/// The projection covers the county GEOID, title, decision question, the
/// pinned page template identity ([`ARCHIVE_PAGE_TEMPLATE_SHA256_V1`]), the
/// ordered grant-visible signals with their full citation identities
/// (source id and provenance name; the locator's receipt tick is a receipt
/// stamp and deliberately never enters the hash), and the sorted place links
/// including whether each name is visible. `verified_tick` and
/// `tick_content_hash` deliberately never enter the hash, so a later receipt
/// alone never re-publishes an unchanged page.
#[must_use]
pub fn county_page_semantic_sha256_v1(
    county_geoid: &str,
    projection: &CountyPageProjectionV1,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(COUNTY_SEMANTIC_DOMAIN_V1);
    hash_text(&mut hasher, county_geoid);
    hash_text(&mut hasher, projection.title());
    hash_text(&mut hasher, projection.question());
    hasher.update(ARCHIVE_PAGE_TEMPLATE_SHA256_V1);
    hash_len(&mut hasher, projection.signals().len());
    for signal in projection.signals() {
        hash_text(&mut hasher, signal.label());
        hash_text(&mut hasher, signal.value());
        hash_text(&mut hasher, signal.source_id());
        hash_text(&mut hasher, signal.provenance_name());
    }
    hash_len(&mut hasher, projection.places().len());
    for (geoid, name) in projection.places() {
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

/// Parse the semantic projection out of one stored rendered county page.
///
/// The parser is coupled to the pinned `archive_page_v1.md.j2` template
/// ([`crate::ARCHIVE_PAGE_TEMPLATE_SHA256_V1`]) and returns `None` for any
/// stored page whose frontmatter subject, title, question, signal-bullet, or
/// related-link shape drifted; callers treat `None` as dirty. Each signal
/// citation locator must name the same receipt tick as the frontmatter
/// `verified_tick`: the locator's tick is a receipt stamp, and a disagreeing
/// locator means the stored page drifted.
#[must_use]
pub fn parse_stored_county_page_v1(
    county_geoid: &str,
    title: &str,
    markdown: &str,
) -> Option<CountyPageProjectionV1> {
    let mut lines = markdown.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut subject_exact = false;
    let mut verified_tick: Option<u64> = None;
    for line in lines.by_ref() {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("subject: ") {
            subject_exact = value == format!("county/{county_geoid}");
        }
        if let Some(value) = line.strip_prefix("verified_tick: ") {
            verified_tick = value.parse::<u64>().ok();
        }
    }
    if !subject_exact {
        return None;
    }
    let verified_tick = verified_tick?;
    let stored_title = lines.next()?.strip_prefix("# ")?.to_owned();
    if stored_title != title {
        return None;
    }
    let mut question = None;
    let mut signals = Vec::new();
    let mut places = Vec::new();
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
            signals.push(parse_signal_bullet(line, verified_tick)?);
            continue;
        }
        if in_related {
            if let Some(entry) = line.strip_prefix("- [") {
                let (label, key) = parse_subject_scheme_link(entry)?;
                let place = key.strip_prefix("place/")?;
                if place.len() != 7 || !place.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                places.push((place.to_owned(), label));
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
    CountyPageProjectionV1::try_new(stored_title, question?, signals, places).ok()
}

/// Parse one pinned-template signal bullet with its committed provenance.
fn parse_signal_bullet(line: &str, verified_tick: u64) -> Option<CountySignalProjectionV1> {
    let rest = line.strip_prefix("- **")?;
    let (label, rest) = rest.split_once(":** ")?;
    let (value, citation) = rest.split_once(" — ")?;
    let (source_id, locator) = citation.split_once("; ")?;
    let provenance_name = if source_id == QCEW_ECONOMICS_SOURCE_ID_V1 {
        let county = locator.strip_prefix("qcew_county_economics_mi_2024.csv.gz#county_geoid=")?;
        let (geoid, digest) = county.split_once("&sha256=")?;
        if geoid.len() != 5
            || !geoid.starts_with("26")
            || !geoid.bytes().all(|byte| byte.is_ascii_digit())
            || digest != QCEW_ECONOMICS_ARTIFACT_SHA256_V1
        {
            return None;
        }
        locator.to_owned()
    } else {
        parse_committed_locator(locator, verified_tick)?
    };
    CountySignalProjectionV1::try_new(
        label.to_owned(),
        value.to_owned(),
        source_id.to_owned(),
        provenance_name,
    )
    .ok()
}

/// Parse the pinned `campaign/{receipt tick}/{territory local name}` locator.
fn parse_committed_locator(locator: &str, verified_tick: u64) -> Option<String> {
    let rest = locator.strip_prefix("campaign/")?;
    let (tick, name) = rest.split_once('/')?;
    if tick.parse::<u64>().ok()? != verified_tick || name.is_empty() || name.contains('/') {
        return None;
    }
    Some(name.to_owned())
}

/// Keep only the desired county pages whose subject the grant snapshot knows.
///
/// The fog-safe renderer refuses an unknown subject, so an ungranted county
/// must leave the batch before construction: it simply produces no page this
/// sweep. When its subject grant arrives later the county has no stored page,
/// so it is dirty and publishes then — the grant gate never fabricates pages.
#[must_use]
pub fn filter_granted_county_plans_v1<'a>(
    desired: &'a [CountyPagePlanV1],
    grants: &CountyGrantIndexV1,
) -> Vec<&'a CountyPagePlanV1> {
    desired
        .iter()
        .filter(|plan| {
            let county_ref = ArchivePageRefV1::try_new(
                ArchiveSubjectKindV1::County,
                plan.county_geoid().to_owned(),
            );
            county_ref.is_ok_and(|page_ref| grants.knows_subject(&page_ref))
        })
        .collect()
}

/// Select the dirty desired pages, sorted by county GEOID, bounded by `limit`.
///
/// A desired page is dirty when no stored projection exists for its subject
/// or when the grant-visible projection hash differs. The selection never
/// truncates silently and never refuses: it takes the leading `limit` dirty
/// entries and reports the exact undrained tail count, so an over-bound dirty
/// set pages across successive sweeps — stored-current pages drop out of the
/// dirty set automatically, which is what advances the head each sweep.
///
/// # Errors
/// Returns any projection refusal.
pub fn select_dirty_county_pages_v1<'a>(
    desired: &'a [CountyPagePlanV1],
    stored: &BTreeMap<String, CountyPageProjectionV1>,
    grants: &CountyGrantIndexV1,
    limit: usize,
) -> Result<ArchiveDirtySelectionV1<&'a CountyPagePlanV1>, SemanticArchiveErrorV1> {
    let mut dirty = Vec::new();
    for plan in desired {
        let projection = desired_county_projection_v1(plan, grants)?;
        let is_dirty = stored.get(plan.county_geoid()).is_none_or(|page| {
            county_page_semantic_sha256_v1(plan.county_geoid(), page)
                != county_page_semantic_sha256_v1(plan.county_geoid(), &projection)
        });
        if is_dirty {
            dirty.push(plan);
        }
    }
    let remaining = dirty.len().saturating_sub(limit);
    dirty.truncate(limit);
    Ok(ArchiveDirtySelectionV1::new(dirty, remaining))
}

/// Build the exact receipt-bound page input for one desired county page.
///
/// Public baseline signals cite their pinned QCEW county row. Other signals
/// cite their committed tick provenance.
///
/// # Errors
/// Refuses any unsafe page component.
pub fn county_page_input_v1(
    plan: &CountyPagePlanV1,
    resolve_tick: u64,
    tick_content_hash: [u8; 32],
) -> Result<ArchivePageInputV1, SemanticArchiveErrorV1> {
    let subject = ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::County,
        plan.county_geoid.clone(),
        plan.title.clone(),
    )?;
    let signals = plan
        .signals
        .iter()
        .map(|signal| {
            let citation = if QCEW_ECONOMICS_FIELD_KEYS_V1.contains(&signal.grant_key()) {
                county_qcew_citation(plan.county_geoid())
            } else {
                ArchiveCitationV1::try_new(
                    COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
                    format!("campaign/{resolve_tick}/{}", plan.territory_local_name),
                )?
            };
            ArchiveSignalV1::try_new(
                signal.grant_key.clone(),
                signal.label.clone(),
                signal.value.clone(),
                citation,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let links = plan
        .place_links
        .iter()
        .map(|link| {
            ArchiveLinkV1::try_new(
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, link.place_geoid.clone())?,
                link.place_name.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    ArchivePageInputV1::try_new(
        subject,
        resolve_tick,
        tick_content_hash,
        COUNTY_DECISION_QUESTION_V1.to_owned(),
        signals,
        links,
    )
}

/// Committed per-tick signal sources for one territory, absence-maximal.
///
/// Public baseline integers retain exact units and magnitude. Every field
/// missing at the resolve tick stays `None` and emits no signal.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CommittedTerritoryFieldsV1 {
    median_wage: Option<f64>,
    phi_hour: Option<f64>,
    qcew: [Option<i64>; 4],
}

impl CommittedTerritoryFieldsV1 {
    /// Construct the two committed field values, absence-maximal.
    ///
    /// # Errors
    /// Refuses a non-finite value; committed values are canonical finite
    /// binary64 and any other input is malformed committed state.
    pub fn try_new(
        median_wage: Option<f64>,
        phi_hour: Option<f64>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        for value in [median_wage, phi_hour].into_iter().flatten() {
            if !value.is_finite() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
        }
        Ok(Self {
            median_wage,
            phi_hour,
            qcew: [None; 4],
        })
    }

    /// Construct exact public QCEW fields in `QCEW_ECONOMICS_FIELD_KEYS_V1` order.
    /// # Errors
    /// Refuses negative public-record counts or whole-USD wages.
    pub fn try_from_qcew(values: [Option<i64>; 4]) -> Result<Self, SemanticArchiveErrorV1> {
        if values.iter().flatten().any(|value| *value < 0) {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        Ok(Self {
            qcew: values,
            ..Self::default()
        })
    }

    fn insert_stored(
        &mut self,
        field: &str,
        tag: i16,
        real_bits: Option<i64>,
        integer: Option<i64>,
    ) -> Result<(), SemanticArchiveErrorV1> {
        if let Some(index) = QCEW_ECONOMICS_FIELD_KEYS_V1
            .iter()
            .position(|key| *key == field)
        {
            let value = integer
                .filter(|value| *value >= 0)
                .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
            if tag != INT_VALUE_TAG_V1 || real_bits.is_some() || self.qcew[index].is_some() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            self.qcew[index] = Some(value);
        } else {
            let value = f64::from_bits(
                real_bits
                    .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?
                    .cast_unsigned(),
            );
            if tag != REAL_VALUE_TAG_V1 || integer.is_some() || !value.is_finite() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            let target = match field {
                MEDIAN_WAGE_FIELD_V1 => &mut self.median_wage,
                PHI_HOUR_FIELD_V1 => &mut self.phi_hour,
                _ => return Err(SemanticArchiveErrorV1::StoredPageMismatch),
            };
            if target.is_some() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            *target = Some(value);
        }
        Ok(())
    }

    /// Borrow the committed median-wage value, absent when not committed.
    #[must_use]
    pub fn median_wage(&self) -> Option<f64> {
        self.median_wage
    }

    /// Borrow the committed phi-hour value, absent when not committed.
    #[must_use]
    pub fn phi_hour(&self) -> Option<f64> {
        self.phi_hour
    }
}

/// Build exact grant-keyed committed signals, absence-maximal.
///
/// A committed field that is missing emits no signal, never a fabricated
/// value. Public baseline integers remain exact; established real fields use
/// `%.6f`. This is the exact signal construction [`CountyDossierProducerV1`]
/// runs inside `desired_pages`, factored pure so the language-neutral parity
/// vectors exercise it without a database.
///
/// # Errors
/// Refuses a non-finite committed value or unsafe text.
pub fn county_committed_signals_v1(
    fields: &CommittedTerritoryFieldsV1,
) -> Result<Vec<CountySignalV1>, SemanticArchiveErrorV1> {
    let mut signals = Vec::new();
    for ((key, label), value) in QCEW_ECONOMICS_FIELD_KEYS_V1
        .into_iter()
        .zip(QCEW_LABELS_V1)
        .zip(fields.qcew)
    {
        if let Some(value) = value {
            signals.push(CountySignalV1::try_new(
                key.to_owned(),
                label.to_owned(),
                value.to_string(),
            )?);
        }
    }
    if let Some(value) = fields.median_wage {
        signals.push(CountySignalV1::from_committed_real(
            COUNTY_MEDIAN_WAGE_GRANT_KEY_V1.to_owned(),
            COUNTY_MEDIAN_WAGE_LABEL_V1.to_owned(),
            value,
        )?);
    }
    if let Some(value) = fields.phi_hour {
        signals.push(CountySignalV1::from_committed_real(
            COUNTY_PHI_HOUR_GRANT_KEY_V1.to_owned(),
            COUNTY_PHI_HOUR_LABEL_V1.to_owned(),
            value,
        )?);
    }
    Ok(signals)
}

/// Production county dossier producer over the checked reference products.
pub struct CountyDossierProducerV1 {
    config: Config,
    products: SpatialReferenceProducts,
}

impl CountyDossierProducerV1 {
    /// Load the checked reference products and bind the committed-state readers.
    ///
    /// # Errors
    /// Refuses loudly when the embedded reference products, their governing
    /// H3 cohort, or any contract-pinned artifact digest diverges.
    pub fn try_new(config: &Config) -> Result<Self, SemanticArchiveErrorV1> {
        let cohort = representative_h3_reference_cohort_v1()
            .map_err(|_| SemanticArchiveErrorV1::ArtifactDigest)?;
        let products = michigan_spatial_reference_products_v1(cohort)
            .map_err(|_| SemanticArchiveErrorV1::ArtifactDigest)?;
        verify_pinned_artifact_digests(&products)?;
        Ok(Self {
            config: config.clone(),
            products,
        })
    }

    /// Resolve every desired county page from the declared mapping and the
    /// committed per-tick territory fields at `resolve_tick`.
    ///
    /// The result is sorted by county GEOID. A committed field that is absent
    /// at the resolve tick emits no signal; a county whose governed identity
    /// is missing from the pinned products refuses loudly rather than
    /// fabricating a title.
    ///
    /// # Errors
    /// Returns any database, decode, or artifact-identity failure.
    pub fn desired_pages(
        &self,
        campaign_id: CampaignId,
        resolve_tick: u64,
    ) -> Result<Vec<CountyPagePlanV1>, SemanticArchiveErrorV1> {
        let county_names = self
            .products
            .counties()
            .iter()
            .map(|county| (county.county_geoid(), county.county_name()))
            .collect::<BTreeMap<_, _>>();
        let place_names = self
            .products
            .places()
            .iter()
            .map(|place| (place.place_geoid(), place.name_lsad()))
            .collect::<BTreeMap<_, _>>();
        let overlaps = self.products.county_place_land_areas().iter().fold(
            BTreeMap::<&str, BTreeSet<&str>>::new(),
            |mut overlaps, row| {
                overlaps
                    .entry(row.county_geoid())
                    .or_default()
                    .insert(row.place_geoid());
                overlaps
            },
        );
        let mapping = self.read_county_mapping(campaign_id)?;
        let committed = self.read_committed_fields(campaign_id, resolve_tick)?;
        mapping
            .into_iter()
            .map(|(county_geoid, territory_local_name)| {
                let title = (*county_names
                    .get(county_geoid.as_str())
                    .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?)
                .to_owned();
                let signals = match committed.get(&territory_local_name) {
                    Some(fields) => county_committed_signals_v1(fields)?,
                    None => Vec::new(),
                };
                let place_links = overlaps
                    .get(county_geoid.as_str())
                    .into_iter()
                    .flatten()
                    .map(|place_geoid| {
                        let place_name = place_names
                            .get(place_geoid)
                            .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
                        CountyPlaceLinkV1::try_new(
                            (*place_geoid).to_owned(),
                            (*place_name).to_owned(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                CountyPagePlanV1::try_new(
                    county_geoid,
                    territory_local_name,
                    title,
                    signals,
                    place_links,
                )
            })
            .collect()
    }

    /// Read the campaign's declared county mapping in GEOID order.
    ///
    /// # Errors
    /// Returns any database or decode failure from the read-only query.
    fn read_county_mapping(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<(String, String)>, SemanticArchiveErrorV1> {
        let mut client = self
            .config
            .connect(NoTls)
            .map_err(|error| database("connect county mapping reader", &error))?;
        let rows = client
            .query(ARCHIVE_COUNTY_MAP_READ_SQL_V1, &[campaign_id.as_uuid()])
            .map_err(|error| database("read county mapping", &error))?;
        rows.iter()
            .map(|row| {
                let local: String = decode(row, 0)?;
                let geoid: String = decode(row, 1)?;
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, geoid.clone())?;
                validate_text(&local)?;
                Ok((geoid, local))
            })
            .collect()
    }

    /// Read the committed per-tick territory fields at one exact tick.
    ///
    /// Only real-typed rows of the two D2-committed fields enter the
    /// projection; a territory missing at the tick contributes nothing.
    ///
    /// # Errors
    /// Returns any database or decode failure, and refuses a committed field
    /// whose stored tag or bits are malformed.
    fn read_committed_fields(
        &self,
        campaign_id: CampaignId,
        resolve_tick: u64,
    ) -> Result<BTreeMap<String, CommittedTerritoryFieldsV1>, SemanticArchiveErrorV1> {
        let resolve_tick =
            i64::try_from(resolve_tick).map_err(|_| SemanticArchiveErrorV1::InvalidVerifiedTick)?;
        let mut client = self
            .config
            .connect(NoTls)
            .map_err(|error| database("connect committed territory field reader", &error))?;
        let rows = client
            .query(
                ARCHIVE_COUNTY_FIELD_READ_SQL_V1,
                &[campaign_id.as_uuid(), &resolve_tick],
            )
            .map_err(|error| database("read committed territory fields", &error))?;
        let mut committed: BTreeMap<String, CommittedTerritoryFieldsV1> = BTreeMap::new();
        for row in &rows {
            let key: Vec<u8> = decode(row, 0)?;
            let Ok(StableElementKeyV1::Node { local_name, .. }) =
                StableElementKeyV1::from_canonical_bytes(&key)
            else {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            };
            let field_name: String = decode(row, 1)?;
            let value_tag: i16 = decode(row, 2)?;
            let real_bits: Option<i64> = decode(row, 3)?;
            let integer: Option<i64> = decode(row, 4)?;
            let fields = committed.entry(local_name).or_default();
            fields.insert_stored(&field_name, value_tag, real_bits, integer)?;
        }
        Ok(committed)
    }
}

impl ArchiveDossierProducerV1 for CountyDossierProducerV1 {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &crate::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let campaign = CampaignId::from_uuid(campaign_id);
        let desired = self.desired_pages(campaign, receipt.resolve_tick())?;
        crate::archive_revision::publication::select_dirty_pages(
            &self.config,
            campaign,
            receipt,
            knowledge,
            &desired,
            page_budget,
            county_page_input_v1,
        )
    }
    fn cutover_subjects(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &crate::ArchiveKnowledgeV1,
    ) -> Result<Vec<ArchivePageRefV1>, SemanticArchiveErrorV1> {
        let campaign = CampaignId::from_uuid(campaign_id);
        let desired = self.desired_pages(campaign, receipt.resolve_tick())?;
        let mut subjects = std::collections::BTreeSet::new();
        for plan in &desired {
            let page =
                county_page_input_v1(plan, receipt.resolve_tick(), *receipt.tick_content_hash())?;
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
            COUNTY_IDENTITY_PRODUCT_CODE_V1,
            PINNED_COUNTY_IDENTITY_ARTIFACT_SHA256_V1,
        ),
        (
            PLACE_IDENTITY_PRODUCT_CODE_V1,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_qcew_ints_never_round_through_binary64() {
        let mut fields = CommittedTerritoryFieldsV1::default();
        fields
            .insert_stored("qcew-total-annual-wages", 1, None, Some(i64::MAX))
            .expect("exact integer");
        let signals = county_committed_signals_v1(&fields).expect("signals");
        assert_eq!(signals[0].value(), "9223372036854775807");
    }

    #[test]
    fn stored_qcew_wrong_type_missing_negative_or_duplicate_refuses() {
        for (tag, real, integer) in [
            (3, Some(1_f64.to_bits().cast_signed()), None),
            (1, None, None),
            (1, None, Some(-1)),
            (1, Some(0), Some(1)),
        ] {
            assert!(CommittedTerritoryFieldsV1::default()
                .insert_stored("qcew-employment", tag, real, integer)
                .is_err());
        }
        let mut fields = CommittedTerritoryFieldsV1::default();
        fields
            .insert_stored("qcew-employment", 1, None, Some(0))
            .expect("zero is observed");
        assert!(fields
            .insert_stored("qcew-employment", 1, None, Some(0))
            .is_err());
        assert_eq!(
            county_committed_signals_v1(&fields).expect("signals")[0].value(),
            "0"
        );
    }
}
