//! Fog-safe semantic Archive page and knowledge contracts.

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::sha256_of;
use minijinja::{context, Environment, UndefinedBehavior};
use postgres::types::FromSqlOwned;
use postgres::{Config, GenericClient, IsolationLevel, NoTls, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::archive_revision::emission::{ArchiveEmissionLinkV2, ArchiveEmissionManifestV2};
use crate::identity::CampaignId;
use crate::migration_manifest::SCHEMA_ADVISORY_LOCK_KEY;
use crate::postgres_diagnostic::PostgresDiagnosticV1;

/// Exact additive schema used by the semantic Archive worker.
pub const SEMANTIC_ARCHIVE_SCHEMA_V1_SQL: &str =
    include_str!("../migrations/semantic_archive_v1.sql");
/// Exact additive atom schema consumed by the semantic Archive worker
/// (ADR249 R1/R2); these bytes fold into [`archive_worker_contract_sha256_v1`].
pub const ARCHIVE_ATOM_SCHEMA_V1_SQL: &str = include_str!("../migrations/archive_atom_v1.sql");
/// Receipt-processing status view; page bytes and their source ticks stay immutable.
pub const ARCHIVE_VERIFICATION_SCHEMA_V1_SQL: &str =
    include_str!("../migrations/archive_verification_v1.sql");
const ARCHIVE_PAGE_TEMPLATE_V1: &str = include_str!("archive_page_v1.md.j2");
const MAX_ID_BYTES: usize = 128;
const MAX_TEXT_BYTES: usize = 4_096;
pub(crate) const MAX_SIGNALS: usize = 256;
pub(crate) const MAX_LINKS: usize = 256;
const MAX_KNOWLEDGE_GRANTS: usize = 65_535;
pub(crate) const MAX_PAGE_BYTES: usize = 1_048_576;
const ARCHIVE_SCHEMA_CONTRACT_ID: &str = "babylon.semantic-archive-schema.v1";
const ARCHIVE_ATOM_SCHEMA_CONTRACT_ID: &str = "babylon.archive-atom-schema.v1";
const ARCHIVE_WORKER_DOMAIN_V1: &[u8] = b"babylon.semantic-archive-worker.v1\0";
const ARCHIVE_DIRTY_BATCH_DOMAIN_V1: &[u8] = b"babylon.semantic-archive-dirty-batch.v1\0";
const ARCHIVE_KNOWLEDGE_DOMAIN_V1: &[u8] = b"babylon.semantic-archive-knowledge.v1\0";
const ARCHIVE_ATOM_DOMAIN_V1: &[u8] = b"babylon.semantic-archive-atom.v1\0";
const ARCHIVE_SCHEMA_MARKERS_SQL_V1: &str = "SELECT \
    pg_catalog.to_regclass('babylon_meta.semantic_archive_schema_v1') IS NOT NULL, \
    pg_catalog.to_regclass('babylon_meta.archive_knowledge_grant_v1') IS NOT NULL, \
    pg_catalog.to_regclass('babylon_meta.archive_receipt_consumption_v1') IS NOT NULL, \
    pg_catalog.to_regclass('babylon_meta.archive_page_v1') IS NOT NULL, \
    EXISTS (\
        SELECT 1 FROM pg_catalog.pg_constraint \
        WHERE conname = 'archive_page_v1_campaign_id_source_resolve_tick_fkey' \
          AND conrelid = pg_catalog.to_regclass('babylon_meta.archive_page_v1') \
          AND confrelid = pg_catalog.to_regclass('babylon_state.archive_dirty_receipt_v1')\
    )";
/// PER-318 upgrade: an installed base schema whose page provenance still
/// anchors at the consumption marker cannot stage a paged drain batch, so
/// the installer re-anchors the exact constraint at the durable dirty
/// receipt before reporting the schema current.
const ARCHIVE_SCHEMA_PAGE_FK_UPGRADE_SQL_V1: &str = "ALTER TABLE babylon_meta.archive_page_v1 \
    DROP CONSTRAINT archive_page_v1_campaign_id_source_resolve_tick_fkey; \
    ALTER TABLE babylon_meta.archive_page_v1 \
    ADD CONSTRAINT archive_page_v1_campaign_id_source_resolve_tick_fkey \
    FOREIGN KEY (campaign_id, source_resolve_tick) \
    REFERENCES babylon_state.archive_dirty_receipt_v1(campaign_id, resolve_tick) \
    ON DELETE CASCADE";
const ARCHIVE_ATOM_SCHEMA_MARKERS_SQL_V1: &str = "SELECT \
    pg_catalog.to_regclass('babylon_meta.archive_atom_schema_v1') IS NOT NULL, \
    pg_catalog.to_regclass('babylon_meta.archive_atom_v1') IS NOT NULL, \
    pg_catalog.to_regclass('babylon_meta.archive_page_atom_v1') IS NOT NULL, \
    pg_catalog.to_regclass('public.v_archive_page_known_v1') IS NOT NULL, \
    pg_catalog.to_regclass('public.v_archive_atom_visible') IS NOT NULL, \
    pg_catalog.to_regclass('public.v_county_card_atoms') IS NOT NULL";
/// SQL-only knowledge boundary used before any template receives values.
/// Page-subject knowledge only: seeded concept grants widen the grant table's
/// subject domain (ADR249 R3/R12) but never enter the page knowledge snapshot.
pub const ARCHIVE_KNOWLEDGE_SQL_V1: &str = "SELECT subject_kind, subject_id, grant_key, \
    granted_tick, provenance_source_id, provenance_locator \
    FROM babylon_meta.archive_knowledge_grant_v1 \
    WHERE campaign_id = $1::uuid AND granted_tick <= $2 \
      AND subject_kind IN ('county', 'place') \
    ORDER BY subject_kind, subject_id, grant_key LIMIT $3";
/// SHA-256 of the pinned strict `MiniJinja` page template.
pub const ARCHIVE_PAGE_TEMPLATE_SHA256_V1: [u8; 32] = [
    0xd7, 0x90, 0x43, 0x79, 0xcf, 0x09, 0xf4, 0x1d, 0xb6, 0xab, 0xea, 0x91, 0x46, 0x5b, 0x5f, 0xe6,
    0xe8, 0x04, 0x86, 0x7c, 0xf8, 0x76, 0xbd, 0x44, 0xa0, 0x9f, 0xe6, 0x3b, 0xa9, 0x75, 0x51, 0x08,
];

/// Closed semantic page kinds in the first Archive slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ArchiveSubjectKindV1 {
    /// United States county identified by five-digit Census FIPS.
    County,
    /// Census-designated place identified by seven-digit place GEOID.
    Place,
}

impl ArchiveSubjectKindV1 {
    /// Stable storage and page-path spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::County => "county",
            Self::Place => "place",
        }
    }
}

/// Stable semantic Archive page identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ArchivePageRefV1 {
    kind: ArchiveSubjectKindV1,
    id: String,
}

impl ArchivePageRefV1 {
    /// Construct one exact county or place identity.
    ///
    /// # Errors
    /// Refuses the wrong digit width or an unbounded identity.
    pub fn try_new(kind: ArchiveSubjectKindV1, id: String) -> Result<Self, SemanticArchiveErrorV1> {
        let expected = match kind {
            ArchiveSubjectKindV1::County => 5,
            ArchiveSubjectKindV1::Place => 7,
        };
        if id.len() != expected
            || id.len() > MAX_ID_BYTES
            || !id.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(SemanticArchiveErrorV1::InvalidIdentity);
        }
        Ok(Self { kind, id })
    }

    /// Return the closed subject kind.
    #[must_use]
    pub const fn kind(&self) -> ArchiveSubjectKindV1 {
        self.kind
    }

    /// Borrow the exact external identity.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn page_key(&self) -> String {
        format!("{}/{}", self.kind.as_str(), self.id)
    }
}

/// Known page identity and safe player-facing title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSubjectV1 {
    page_ref: ArchivePageRefV1,
    title: String,
}

impl ArchiveSubjectV1 {
    /// Construct one page subject.
    ///
    /// # Errors
    /// Refuses an invalid identity or unsafe title.
    pub fn try_new(
        kind: ArchiveSubjectKindV1,
        id: String,
        title: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&title)?;
        Ok(Self {
            page_ref: ArchivePageRefV1::try_new(kind, id)?,
            title,
        })
    }

    /// Borrow the stable page reference.
    #[must_use]
    pub const fn page_ref(&self) -> &ArchivePageRefV1 {
        &self.page_ref
    }

    /// Borrow the safe known title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// One player-visible source locator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArchiveCitationV1 {
    source_id: String,
    locator: String,
}

#[derive(Deserialize)]
struct UnvalidatedArchiveCitationV1 {
    source_id: String,
    locator: String,
}

impl<'de> Deserialize<'de> for ArchiveCitationV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let citation = UnvalidatedArchiveCitationV1::deserialize(deserializer)?;
        Self::try_new(citation.source_id, citation.locator).map_err(serde::de::Error::custom)
    }
}

impl ArchiveCitationV1 {
    /// Construct a bounded source citation.
    ///
    /// # Errors
    /// Refuses an empty, NUL-containing, or unbounded component.
    pub fn try_new(source_id: String, locator: String) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&source_id)?;
        validate_text(&locator)?;
        Ok(Self { source_id, locator })
    }

    /// Borrow the stable source identity.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Borrow the precise source locator.
    #[must_use]
    pub fn locator(&self) -> &str {
        &self.locator
    }
}

/// One knowledge-grant-addressable semantic signal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSignalV1 {
    grant_key: String,
    label: String,
    value: String,
    citation: ArchiveCitationV1,
}

impl ArchiveSignalV1 {
    /// Construct one bounded signal.
    ///
    /// # Errors
    /// Refuses an unsafe grant key, label, or value.
    pub fn try_new(
        grant_key: String,
        label: String,
        value: String,
        citation: ArchiveCitationV1,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_key(&grant_key)?;
        validate_text(&label)?;
        validate_text(&value)?;
        validate_text(citation.source_id())?;
        validate_text(citation.locator())?;
        Ok(Self {
            grant_key,
            label,
            value,
            citation,
        })
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

    /// Borrow the player-facing signal value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Borrow the pinned provenance citation.
    #[must_use]
    pub const fn citation(&self) -> &ArchiveCitationV1 {
        &self.citation
    }
}

/// One outbound semantic page link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveLinkV1 {
    target: ArchivePageRefV1,
    known_label: String,
}

impl ArchiveLinkV1 {
    /// Construct one outbound link whose label may be shown only when known.
    ///
    /// # Errors
    /// Refuses an unsafe label.
    pub fn try_new(
        target: ArchivePageRefV1,
        known_label: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&known_label)?;
        Ok(Self {
            target,
            known_label,
        })
    }

    /// Borrow the exact link target identity.
    #[must_use]
    pub const fn target(&self) -> &ArchivePageRefV1 {
        &self.target
    }

    /// Borrow the label shown only when the target subject is known.
    #[must_use]
    pub fn known_label(&self) -> &str {
        &self.known_label
    }
}

/// One receipt-bound page refresh requested by the semantic worker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchivePageInputV1 {
    subject: ArchiveSubjectV1,
    verified_tick: u64,
    tick_content_hash: [u8; 32],
    decision_question: String,
    signals: Vec<ArchiveSignalV1>,
    links: Vec<ArchiveLinkV1>,
}

impl ArchivePageInputV1 {
    /// Construct one bounded dirty-subject work item.
    ///
    /// # Errors
    /// Refuses synthetic tick zero, duplicate keys, or unbounded collections.
    pub fn try_new(
        subject: ArchiveSubjectV1,
        verified_tick: u64,
        tick_content_hash: [u8; 32],
        decision_question: String,
        signals: Vec<ArchiveSignalV1>,
        links: Vec<ArchiveLinkV1>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if verified_tick == 0 || verified_tick > i64::MAX as u64 {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
        }
        validate_text(&decision_question)?;
        if signals.len() > MAX_SIGNALS || links.len() > MAX_LINKS {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        let signal_keys = signals
            .iter()
            .map(|signal| signal.grant_key.as_str())
            .collect::<BTreeSet<_>>();
        let link_targets = links
            .iter()
            .map(|link| &link.target)
            .collect::<BTreeSet<_>>();
        if signal_keys.len() != signals.len() || link_targets.len() != links.len() {
            return Err(SemanticArchiveErrorV1::DuplicateKey);
        }
        Ok(Self {
            subject,
            verified_tick,
            tick_content_hash,
            decision_question,
            signals,
            links,
        })
    }

    /// Borrow the page subject.
    #[must_use]
    pub const fn subject(&self) -> &ArchiveSubjectV1 {
        &self.subject
    }

    /// Return the receipt-stamped verified tick.
    #[must_use]
    pub const fn verified_tick(&self) -> u64 {
        self.verified_tick
    }

    /// Borrow the receipt-stamped tick content hash.
    #[must_use]
    pub const fn tick_content_hash(&self) -> &[u8; 32] {
        &self.tick_content_hash
    }

    /// Borrow the stable decision question.
    #[must_use]
    pub fn decision_question(&self) -> &str {
        &self.decision_question
    }

    /// Borrow the ordered grant-keyed signals.
    #[must_use]
    pub fn signals(&self) -> &[ArchiveSignalV1] {
        &self.signals
    }

    /// Borrow the ordered outbound links.
    #[must_use]
    pub fn links(&self) -> &[ArchiveLinkV1] {
        &self.links
    }
}

/// Exact SQL-derived knowledge grants supplied to the pure renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveKnowledgeV1 {
    grants: BTreeMap<(ArchivePageRefV1, String), ArchiveKnowledgeGrantV1>,
}

impl ArchiveKnowledgeV1 {
    /// Validate an exact SQL grant result.
    ///
    /// # Errors
    /// Refuses duplicate rows or a malformed key or citation.
    pub fn try_new(grants: Vec<ArchiveKnowledgeGrantV1>) -> Result<Self, SemanticArchiveErrorV1> {
        let mut indexed = BTreeMap::new();
        for grant in grants {
            validate_key(&grant.grant_key)?;
            validate_text(grant.citation.source_id())?;
            validate_text(grant.citation.locator())?;
            let key = (grant.page_ref.clone(), grant.grant_key.clone());
            if indexed.insert(key, grant).is_some() {
                return Err(SemanticArchiveErrorV1::DuplicateGrant);
            }
        }
        Ok(Self { grants: indexed })
    }

    pub(crate) fn knows_subject(&self, page_ref: &ArchivePageRefV1) -> bool {
        self.grant(page_ref, "subject").is_some()
    }

    pub(crate) fn knows_field(&self, page_ref: &ArchivePageRefV1, grant_key: &str) -> bool {
        self.grant(page_ref, grant_key).is_some()
    }

    pub(crate) fn grant(
        &self,
        page_ref: &ArchivePageRefV1,
        grant_key: &str,
    ) -> Option<&ArchiveKnowledgeGrantV1> {
        self.grants.get(&(page_ref.clone(), grant_key.to_owned()))
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = &ArchiveKnowledgeGrantV1> {
        self.grants.values()
    }

    /// Hash every exact, ordered knowledge-grant row in this snapshot.
    #[must_use]
    pub fn sha256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(ARCHIVE_KNOWLEDGE_DOMAIN_V1);
        hash_len(&mut hasher, self.grants.len());
        for ((page_ref, grant_key), grant) in &self.grants {
            hash_page_ref(&mut hasher, page_ref);
            hash_bytes(&mut hasher, grant_key.as_bytes());
            hasher.update(grant.granted_tick.to_be_bytes());
            hash_citation(&mut hasher, &grant.citation);
        }
        hasher.finalize().into()
    }
}

/// One rendered, searchable, citation-bearing page artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedArchivePageV1 {
    markdown: String,
    search_text: String,
    citations: Vec<ArchiveCitationV1>,
    sha256: [u8; 32],
}

impl RenderedArchivePageV1 {
    /// Borrow exact UTF-8 Markdown bytes.
    #[must_use]
    pub fn markdown(&self) -> &str {
        &self.markdown
    }

    /// Borrow the known-only search materialization.
    #[must_use]
    pub fn search_text(&self) -> &str {
        &self.search_text
    }

    /// Borrow the exact provenance citations for the known page material.
    #[must_use]
    pub fn citations(&self) -> &[ArchiveCitationV1] {
        &self.citations
    }

    /// Return SHA-256 of exact Markdown bytes.
    #[must_use]
    pub const fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
}

/// Pinned strict `MiniJinja` rendering authority.
pub struct FogSafeArchiveRendererV1 {
    environment: Environment<'static>,
}

impl FogSafeArchiveRendererV1 {
    /// Compile the one embedded template with strict undefined behavior.
    ///
    /// # Errors
    /// Returns [`SemanticArchiveErrorV1::Template`] for checked-in syntax drift.
    pub fn new() -> Result<Self, SemanticArchiveErrorV1> {
        let mut environment = Environment::empty();
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment
            .add_template("archive-page-v1", ARCHIVE_PAGE_TEMPLATE_V1)
            .map_err(|_| SemanticArchiveErrorV1::Template)?;
        Ok(Self { environment })
    }

    /// Render one known subject with SQL-derived field and link grants.
    ///
    /// # Errors
    /// Refuses an unknown subject or any strict template failure.
    pub fn render(
        &self,
        input: &ArchivePageInputV1,
        knowledge: &ArchiveKnowledgeV1,
    ) -> Result<RenderedArchivePageV1, SemanticArchiveErrorV1> {
        self.render_with_emission(input, knowledge)
            .map(|(page, _)| page)
    }

    pub(crate) fn render_with_emission(
        &self,
        input: &ArchivePageInputV1,
        knowledge: &ArchiveKnowledgeV1,
    ) -> Result<(RenderedArchivePageV1, ArchiveEmissionManifestV2), SemanticArchiveErrorV1> {
        let subject_grant = knowledge
            .grant(input.subject.page_ref(), "subject")
            .ok_or(SemanticArchiveErrorV1::UnknownSubject)?;
        let signals = input
            .signals
            .iter()
            .filter(|signal| knowledge.knows_field(input.subject.page_ref(), &signal.grant_key))
            .cloned()
            .collect::<Vec<_>>();
        let links = input
            .links
            .iter()
            .map(|link| {
                ArchiveEmissionLinkV2::try_new(
                    link.target.clone(),
                    knowledge
                        .knows_subject(&link.target)
                        .then(|| link.known_label.clone()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let emission = ArchiveEmissionManifestV2::try_new(
            subject_grant.citation.clone(),
            input.decision_question.clone(),
            signals,
            links,
        )?;
        let page = self.render_emission(
            &input.subject,
            input.verified_tick,
            &input.tick_content_hash,
            &emission,
        )?;
        Ok((page, emission))
    }

    pub(crate) fn render_emission(
        &self,
        subject: &ArchiveSubjectV1,
        verified_tick: u64,
        source_hash: &[u8; 32],
        emission: &ArchiveEmissionManifestV2,
    ) -> Result<RenderedArchivePageV1, SemanticArchiveErrorV1> {
        let signals = emission
            .signals()
            .iter()
            .map(TemplateSignalV1::from)
            .collect::<Vec<_>>();
        let links = emission
            .links()
            .iter()
            .map(|link| TemplateLinkV1 {
                page_key: link.target().page_key(),
                known_label: link.known_label(),
            })
            .collect::<Vec<_>>();
        let tick_content_hash = hex_digest(source_hash);
        let template = self
            .environment
            .get_template("archive-page-v1")
            .map_err(|_| SemanticArchiveErrorV1::Template)?;
        let markdown = template
            .render(context! {
                subject_key => subject.page_ref().page_key(),
                title => subject.title(),
                verified_tick => verified_tick,
                tick_content_hash => tick_content_hash,
                decision_question => emission.question(),
                signals => signals,
                links => links,
            })
            .map_err(|_| SemanticArchiveErrorV1::Template)?;
        let search_text = emission.search_text(subject);
        let citations = emission.citations();
        if markdown.len() > MAX_PAGE_BYTES || search_text.len() > MAX_PAGE_BYTES {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        let sha256 = sha256_of(markdown.as_bytes());
        Ok(RenderedArchivePageV1 {
            markdown,
            search_text,
            citations,
            sha256,
        })
    }

    /// Return the checked-in template identity.
    #[must_use]
    pub const fn template_sha256(&self) -> [u8; 32] {
        ARCHIVE_PAGE_TEMPLATE_SHA256_V1
    }
}

/// One bounded batch bound to a single committed dirty receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDirtyBatchV1 {
    resolve_tick: u64,
    tick_content_hash: [u8; 32],
    pages: Vec<ArchivePageInputV1>,
}

impl ArchiveDirtyBatchV1 {
    /// Maximum number of dirty pages consumed from one committed receipt.
    pub const MAX_PAGES: usize = 256;

    /// Borrow the ordered page inputs bound to this receipt.
    #[must_use]
    pub fn pages(&self) -> &[ArchivePageInputV1] {
        &self.pages
    }

    /// Return the bound resolve tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }

    /// Return the exact tick content hash for this receipt.
    #[must_use]
    pub const fn tick_content_hash(&self) -> &[u8; 32] {
        &self.tick_content_hash
    }

    /// Validate an ordered dirty-subject batch.
    ///
    /// # Errors
    /// Refuses tick mismatch, duplicate subjects, or more than 256 pages.
    pub fn try_new(
        resolve_tick: u64,
        tick_content_hash: [u8; 32],
        pages: Vec<ArchivePageInputV1>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if resolve_tick == 0 || resolve_tick > i64::MAX as u64 {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
        }
        if pages.len() > Self::MAX_PAGES {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        let subjects = pages
            .iter()
            .map(|page| page.subject.page_ref())
            .collect::<BTreeSet<_>>();
        if subjects.len() != pages.len() {
            return Err(SemanticArchiveErrorV1::DuplicateKey);
        }
        if pages.iter().any(|page| {
            page.verified_tick != resolve_tick || page.tick_content_hash != tick_content_hash
        }) {
            return Err(SemanticArchiveErrorV1::ReceiptMismatch);
        }
        Ok(Self {
            resolve_tick,
            tick_content_hash,
            pages,
        })
    }

    /// Hash every exact ordered input byte used to materialize this receipt.
    #[must_use]
    pub fn sha256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(ARCHIVE_DIRTY_BATCH_DOMAIN_V1);
        hasher.update(self.resolve_tick.to_be_bytes());
        hasher.update(self.tick_content_hash);
        hash_len(&mut hasher, self.pages.len());
        for page in &self.pages {
            hash_page_input(&mut hasher, page);
        }
        hasher.finalize().into()
    }
}

/// One append-only SQL knowledge grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveKnowledgeGrantV1 {
    pub(crate) page_ref: ArchivePageRefV1,
    pub(crate) grant_key: String,
    pub(crate) granted_tick: u64,
    pub(crate) citation: ArchiveCitationV1,
}

impl ArchiveKnowledgeGrantV1 {
    /// Construct one subject (`subject`) or field grant.
    ///
    /// # Errors
    /// Refuses an unsafe key or a tick outside `PostgreSQL` `BIGINT`.
    pub fn try_new(
        page_ref: ArchivePageRefV1,
        grant_key: String,
        granted_tick: u64,
        citation: ArchiveCitationV1,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_key(&grant_key)?;
        validate_text(citation.source_id())?;
        validate_text(citation.locator())?;
        if granted_tick > i64::MAX as u64 {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
        }
        Ok(Self {
            page_ref,
            grant_key,
            granted_tick,
            citation,
        })
    }
}

/// Idempotent schema-install result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveSchemaDispositionV1 {
    /// The exact additive schema committed now.
    Installed,
    /// The exact contract marker and all relations already existed.
    AlreadyCurrent,
}

/// Idempotent receipt-consumption result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMaterializeDispositionV1 {
    /// This invocation consumed and rendered the receipt.
    Applied,
    /// The exact batch, worker, and knowledge snapshot already consumed the receipt.
    AlreadyConsumed,
}

/// How one materialization pass relates to the receipt's consumption row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMaterializeModeV1 {
    /// The receipt's full dirty set drained: write the pages and claim the
    /// consumption row so `verified_tick` may advance past it.
    Consume,
    /// Dirty pages remain (PER-318 paged drain): write this bounded batch of
    /// pages without claiming, leaving the receipt pending for the next
    /// sweep. The monotonic page guard and content-addressed atoms make an
    /// exact restage a no-op, so a re-sweep never double-writes.
    Stage,
}

/// One persisted page result from an applied batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedArchivePageV1 {
    pub(crate) page_ref: ArchivePageRefV1,
    pub(crate) page: RenderedArchivePageV1,
    pub(crate) persisted: bool,
    pub(crate) atoms: ArchiveAtomMintV1,
}

impl MaterializedArchivePageV1 {
    /// Borrow the stable page identity.
    #[must_use]
    pub const fn page_ref(&self) -> &ArchivePageRefV1 {
        &self.page_ref
    }

    /// Borrow the rendered artifact.
    #[must_use]
    pub const fn page(&self) -> &RenderedArchivePageV1 {
        &self.page
    }

    /// Whether this page replaced the current materialization.
    #[must_use]
    pub const fn persisted(&self) -> bool {
        self.persisted
    }

    /// Borrow the atom mint result for this page.
    #[must_use]
    pub const fn atoms(&self) -> &ArchiveAtomMintV1 {
        &self.atoms
    }
}

/// Receipt-level worker report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveMaterializeReportV1 {
    pub(crate) disposition: ArchiveMaterializeDispositionV1,
    pub(crate) pages: Vec<MaterializedArchivePageV1>,
}

impl ArchiveMaterializeReportV1 {
    /// Return whether this invocation applied or observed an exact retry.
    #[must_use]
    pub const fn disposition(&self) -> ArchiveMaterializeDispositionV1 {
        self.disposition
    }

    /// Borrow rendered page results in caller-supplied order.
    #[must_use]
    pub fn pages(&self) -> &[MaterializedArchivePageV1] {
        &self.pages
    }
}

/// Governed evidence classification carried by every semantic atom
/// (constitutional compact: Observed, Derived, Calibrated, Designed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArchiveEvidenceClassV1 {
    /// A fact read from a pinned source.
    Observed,
    /// A deterministic measure produced from pinned facts.
    Derived,
    /// A value fitted against observation under a declared rule.
    Calibrated,
    /// A value fixed by game design.
    Designed,
}

impl ArchiveEvidenceClassV1 {
    /// Stable storage spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "Observed",
            Self::Derived => "Derived",
            Self::Calibrated => "Calibrated",
            Self::Designed => "Designed",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Observed => 1,
            Self::Derived => 2,
            Self::Calibrated => 3,
            Self::Designed => 4,
        }
    }
}

/// Closed atom subject kinds (ADR249 R1/R12): the page kinds plus glossary
/// concepts. Unlike [`ArchiveSubjectKindV1`] this domain is not a page kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ArchiveAtomSubjectKindV1 {
    /// United States county identified by five-digit Census FIPS.
    County,
    /// Census-designated place identified by seven-digit place GEOID.
    Place,
    /// Glossary concept identified by its concept key.
    Concept,
}

impl ArchiveAtomSubjectKindV1 {
    /// Stable storage spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::County => "county",
            Self::Place => "place",
            Self::Concept => "concept",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::County => 1,
            Self::Place => 2,
            Self::Concept => 3,
        }
    }
}

/// One exact atom subject identity with its per-kind id discipline.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ArchiveAtomSubjectV1 {
    kind: ArchiveAtomSubjectKindV1,
    id: String,
}

impl ArchiveAtomSubjectV1 {
    /// Construct one bounded atom subject identity.
    ///
    /// # Errors
    /// Refuses a malformed per-kind id: five-digit county, seven-digit place,
    /// or a concept key matching ``^[a-z0-9][a-z0-9-]{0,127}$`` exactly.
    pub fn try_new(
        kind: ArchiveAtomSubjectKindV1,
        id: String,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        let exact = match kind {
            ArchiveAtomSubjectKindV1::County => {
                id.len() == 5 && id.bytes().all(|byte| byte.is_ascii_digit())
            }
            ArchiveAtomSubjectKindV1::Place => {
                id.len() == 7 && id.bytes().all(|byte| byte.is_ascii_digit())
            }
            ArchiveAtomSubjectKindV1::Concept => {
                let mut bytes = id.bytes();
                matches!(bytes.next(), Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit())
                    && bytes.all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            }
        };
        if !exact || id.len() > MAX_ID_BYTES || id.is_empty() {
            return Err(SemanticArchiveErrorV1::InvalidIdentity);
        }
        Ok(Self { kind, id })
    }

    /// Adapt one page subject reference into an atom subject.
    ///
    /// # Errors
    /// Refuses a malformed identity; page references are prevalidated, so this
    /// cannot fail for refs produced by [`ArchivePageRefV1::try_new`].
    pub fn from_page_ref(page_ref: &ArchivePageRefV1) -> Result<Self, SemanticArchiveErrorV1> {
        let kind = match page_ref.kind() {
            ArchiveSubjectKindV1::County => ArchiveAtomSubjectKindV1::County,
            ArchiveSubjectKindV1::Place => ArchiveAtomSubjectKindV1::Place,
        };
        Self::try_new(kind, page_ref.id().to_owned())
    }

    /// Return the closed atom subject kind.
    #[must_use]
    pub const fn kind(&self) -> ArchiveAtomSubjectKindV1 {
        self.kind
    }

    /// Borrow the exact external identity.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// One typed, canonical atom value (ADR249 R1).
#[derive(Clone, Debug, PartialEq)]
pub enum ArchiveAtomValueV1 {
    /// Bounded UTF-8 text.
    Text(String),
    /// Canonical finite binary64; `-0.0` normalizes to `+0.0`.
    F64(f64),
    /// Exact unsigned 64-bit integer.
    U64(u64),
    /// Exact boolean.
    Bool(bool),
}

impl ArchiveAtomValueV1 {
    const fn tag(&self) -> u8 {
        match self {
            Self::Text(_) => 1,
            Self::F64(_) => 2,
            Self::U64(_) => 3,
            Self::Bool(_) => 4,
        }
    }

    fn kind_str(&self) -> &'static str {
        match self {
            Self::Text(_) => "text",
            Self::F64(_) => "f64",
            Self::U64(_) => "u64",
            Self::Bool(_) => "bool",
        }
    }
}

/// One immutable, content-addressed semantic atom (ADR249 R1). `atom_id` is
/// SHA-256 of the canonical encoding pinned by `contracts/archive_atom_v1.yaml`;
/// identical bytes re-mint to the identical id, so writer retries are
/// idempotent and atoms never mutate.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveAtomV1 {
    campaign_id: CampaignId,
    subject: ArchiveAtomSubjectV1,
    signal_key: String,
    grant_key: String,
    evidence_class: ArchiveEvidenceClassV1,
    value: ArchiveAtomValueV1,
    citation: ArchiveCitationV1,
    valid_tick: u64,
    atom_id: [u8; 32],
}

impl ArchiveAtomV1 {
    /// Validate and mint one canonical atom.
    ///
    /// # Errors
    /// Refuses a malformed subject, key, citation, or tick; refuses a
    /// non-finite `f64` value with [`SemanticArchiveErrorV1::NonFiniteValue`]
    /// so refusal vectors can name it (R1).
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        campaign_id: CampaignId,
        subject: ArchiveAtomSubjectV1,
        signal_key: String,
        grant_key: String,
        evidence_class: ArchiveEvidenceClassV1,
        value: &ArchiveAtomValueV1,
        citation: ArchiveCitationV1,
        valid_tick: u64,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_key(&signal_key)?;
        validate_key(&grant_key)?;
        validate_text(citation.source_id())?;
        validate_text(citation.locator())?;
        if valid_tick > i64::MAX as u64 {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
        }
        let canonical = match value {
            ArchiveAtomValueV1::Text(text) => {
                if text.is_empty() || text.len() > MAX_TEXT_BYTES || text.as_bytes().contains(&0) {
                    return Err(SemanticArchiveErrorV1::InvalidText);
                }
                ArchiveAtomValueV1::Text(text.clone())
            }
            ArchiveAtomValueV1::F64(number) => {
                if !number.is_finite() {
                    return Err(SemanticArchiveErrorV1::NonFiniteValue);
                }
                // Parity doctrine: -0.0 canonicalizes to +0.0 so a sign-only
                // bit difference never mints a second identity.
                ArchiveAtomValueV1::F64(if *number == 0.0 { 0.0 } else { *number })
            }
            ArchiveAtomValueV1::U64(number) => ArchiveAtomValueV1::U64(*number),
            ArchiveAtomValueV1::Bool(flag) => ArchiveAtomValueV1::Bool(*flag),
        };
        let atom_id = canonical_atom_id(
            campaign_id.as_uuid(),
            &subject,
            &signal_key,
            &grant_key,
            evidence_class,
            &canonical,
            &citation,
            valid_tick,
        );
        Ok(Self {
            campaign_id,
            subject,
            signal_key,
            grant_key,
            evidence_class,
            value: canonical,
            citation,
            valid_tick,
            atom_id,
        })
    }

    /// Borrow the exact campaign identity.
    #[must_use]
    pub const fn campaign_id(&self) -> &CampaignId {
        &self.campaign_id
    }

    /// Borrow the exact atom subject.
    #[must_use]
    pub const fn subject(&self) -> &ArchiveAtomSubjectV1 {
        &self.subject
    }

    /// Borrow the stable signal key.
    #[must_use]
    pub fn signal_key(&self) -> &str {
        &self.signal_key
    }

    /// Borrow the knowledge-grant address key.
    #[must_use]
    pub fn grant_key(&self) -> &str {
        &self.grant_key
    }

    /// Return the governed evidence class.
    #[must_use]
    pub const fn evidence_class(&self) -> ArchiveEvidenceClassV1 {
        self.evidence_class
    }

    /// Borrow the typed canonical value.
    #[must_use]
    pub const fn value(&self) -> &ArchiveAtomValueV1 {
        &self.value
    }

    /// Borrow the pinned provenance citation.
    #[must_use]
    pub const fn citation(&self) -> &ArchiveCitationV1 {
        &self.citation
    }

    /// Return the tick this atom's knowledge was valid from.
    #[must_use]
    pub const fn valid_tick(&self) -> u64 {
        self.valid_tick
    }

    /// Return the content-addressed atom identity.
    #[must_use]
    pub const fn atom_id(&self) -> [u8; 32] {
        self.atom_id
    }
}

/// Pure fog predicate (ADR249 R2, decision 2): an atom is visible exactly
/// while a grant row covers `(campaign, subject, grant_key)` with
/// `granted_tick <= atom.valid_tick` and the valid tick sits inside the
/// acknowledged-commit horizon. `granted_tick` is the grant row's tick when
/// the exact grant row exists and `None` when it does not; the horizon is
/// marker-backed by the caller (never `MAX(tick)`).
#[must_use]
pub const fn archive_atom_visible_v1(
    atom: &ArchiveAtomV1,
    granted_tick: Option<u64>,
    acknowledged_horizon_tick: u64,
) -> bool {
    match granted_tick {
        Some(tick) => tick <= atom.valid_tick && atom.valid_tick <= acknowledged_horizon_tick,
        None => false,
    }
}

/// Per-page atom mint result inside one materialized receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveAtomMintV1 {
    minted: usize,
    joined: usize,
}

impl ArchiveAtomMintV1 {
    /// Construct one exact mint report row.
    #[must_use]
    pub const fn new(minted: usize, joined: usize) -> Self {
        Self { minted, joined }
    }

    /// Exact number of canonical atoms minted for the page.
    #[must_use]
    pub const fn minted(&self) -> usize {
        self.minted
    }

    /// Exact number of composition rows asserted for the page.
    #[must_use]
    pub const fn joined(&self) -> usize {
        self.joined
    }
}

#[allow(clippy::too_many_arguments)]
fn canonical_atom_id(
    campaign_id: &uuid::Uuid,
    subject: &ArchiveAtomSubjectV1,
    signal_key: &str,
    grant_key: &str,
    evidence_class: ArchiveEvidenceClassV1,
    value: &ArchiveAtomValueV1,
    citation: &ArchiveCitationV1,
    valid_tick: u64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ARCHIVE_ATOM_DOMAIN_V1);
    hasher.update(campaign_id.as_bytes());
    hasher.update([subject.kind.tag()]);
    hash_bytes(&mut hasher, subject.id.as_bytes());
    hash_bytes(&mut hasher, signal_key.as_bytes());
    hash_bytes(&mut hasher, grant_key.as_bytes());
    hasher.update([evidence_class.tag()]);
    hasher.update([value.tag()]);
    match value {
        ArchiveAtomValueV1::Text(text) => hash_bytes(&mut hasher, text.as_bytes()),
        ArchiveAtomValueV1::F64(number) => {
            let canonical = if *number == 0.0 { 0.0 } else { *number };
            hasher.update(canonical.to_bits().to_be_bytes());
        }
        ArchiveAtomValueV1::U64(number) => hasher.update(number.to_be_bytes()),
        ArchiveAtomValueV1::Bool(flag) => hasher.update([u8::from(*flag)]),
    }
    hash_citation(&mut hasher, citation);
    hasher.update(valid_tick.to_be_bytes());
    hasher.finalize().into()
}

/// `PostgreSQL` adapter for the client-owned epistemic Archive tier.
#[derive(Clone)]
pub struct SemanticArchiveStoreV1 {
    config: Config,
}

impl SemanticArchiveStoreV1 {
    /// Bind the worker to one Rust-authoritative `PostgreSQL` target.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Install the additive Archive schemas under the shared schema lock.
    ///
    /// The base semantic Archive schema and the additive atom schema each
    /// install idempotently with their own marker census; either one missing
    /// installs, and the disposition reports `Installed` unless both were
    /// already byte-current.
    ///
    /// # Errors
    /// Refuses partial markers, a wrong contract row, or database failure.
    pub fn install_schema(&self) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
        let mut client = self.connect("connect Archive revision installer")?;
        if crate::archive_revision::schema::installed(&mut client)? {
            let revisions = crate::archive_revision::schema::install(&mut client)?;
            let wakeup = crate::archive_wakeup::install(&mut client)?;
            return Ok(
                if revisions == ArchiveSchemaDispositionV1::Installed
                    || wakeup == ArchiveSchemaDispositionV1::Installed
                {
                    ArchiveSchemaDispositionV1::Installed
                } else {
                    ArchiveSchemaDispositionV1::AlreadyCurrent
                },
            );
        }
        let mut disposition = self.install_base_schema()?;
        if self.install_atom_schema()? == ArchiveSchemaDispositionV1::Installed {
            disposition = ArchiveSchemaDispositionV1::Installed;
        }
        if self.install_verification_view()? == ArchiveSchemaDispositionV1::Installed {
            disposition = ArchiveSchemaDispositionV1::Installed;
        }
        if crate::archive_revision::schema::install(&mut client)?
            == ArchiveSchemaDispositionV1::Installed
        {
            disposition = ArchiveSchemaDispositionV1::Installed;
        }
        if crate::archive_wakeup::install(&mut client)? == ArchiveSchemaDispositionV1::Installed {
            disposition = ArchiveSchemaDispositionV1::Installed;
        }
        Ok(disposition)
    }

    fn install_verification_view(
        &self,
    ) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
        let mut client = self.connect("connect Archive verification installer")?;
        let mut transaction = client
            .transaction()
            .map_err(|error| database("begin Archive verification install", &error))?;
        transaction
            .query_one(
                "SELECT pg_catalog.pg_advisory_xact_lock($1)",
                &[&SCHEMA_ADVISORY_LOCK_KEY],
            )
            .map_err(|error| database("lock Archive verification install", &error))?;
        let row = transaction
            .query_one(
                "SELECT pg_catalog.to_regclass('public.v_archive_verification_v1') IS NOT NULL",
                &[],
            )
            .map_err(|error| database("inspect Archive verification view", &error))?;
        let installed = decode::<bool>(&row, 0)?;
        if !installed {
            transaction
                .batch_execute(ARCHIVE_VERIFICATION_SCHEMA_V1_SQL)
                .map_err(|error| database("install Archive verification view", &error))?;
        }
        transaction
            .commit()
            .map_err(|error| database("commit Archive verification install", &error))?;
        Ok(if installed {
            ArchiveSchemaDispositionV1::AlreadyCurrent
        } else {
            ArchiveSchemaDispositionV1::Installed
        })
    }

    /// Install the additive base semantic Archive schema under the shared
    /// schema lock.
    ///
    /// # Errors
    /// Refuses partial markers, a wrong contract row, or database failure.
    pub fn install_base_schema(
        &self,
    ) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
        let mut client = self.connect("connect Archive schema installer")?;
        client
            .query_one(
                "SELECT pg_catalog.pg_advisory_lock($1)",
                &[&SCHEMA_ADVISORY_LOCK_KEY],
            )
            .map_err(|error| database("lock Archive schema installer", &error))?;
        let result = (|| {
            let row = client
                .query_one(ARCHIVE_SCHEMA_MARKERS_SQL_V1, &[])
                .map_err(|error| database("inspect Archive schema markers", &error))?;
            let markers = [
                decode::<bool>(&row, 0)?,
                decode::<bool>(&row, 1)?,
                decode::<bool>(&row, 2)?,
                decode::<bool>(&row, 3)?,
            ];
            let page_fk_current = decode::<bool>(&row, 4)?;
            if markers == [false; 4] && !page_fk_current {
                let mut transaction = client
                    .build_transaction()
                    .isolation_level(IsolationLevel::Serializable)
                    .start()
                    .map_err(|error| database("begin Archive schema install", &error))?;
                transaction
                    .batch_execute(
                        "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
                    )
                    .map_err(|error| database("set Archive schema install settings", &error))?;
                transaction
                    .batch_execute(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL)
                    .map_err(|error| database("install Archive schema", &error))?;
                transaction
                    .commit()
                    .map_err(|error| database("commit Archive schema", &error))?;
                Ok(ArchiveSchemaDispositionV1::Installed)
            } else if markers == [true; 4] {
                let row = client
                    .query_one(
                        "SELECT contract_id FROM babylon_meta.semantic_archive_schema_v1",
                        &[],
                    )
                    .map_err(|error| database("read Archive schema contract", &error))?;
                let contract_id: String = decode(&row, 0)?;
                if contract_id != ARCHIVE_SCHEMA_CONTRACT_ID {
                    return Err(SemanticArchiveErrorV1::SchemaMismatch);
                }
                if page_fk_current {
                    Ok(ArchiveSchemaDispositionV1::AlreadyCurrent)
                } else {
                    let mut transaction = client
                        .build_transaction()
                        .isolation_level(IsolationLevel::Serializable)
                        .start()
                        .map_err(|error| database("begin Archive schema upgrade", &error))?;
                    transaction
                        .batch_execute(ARCHIVE_SCHEMA_PAGE_FK_UPGRADE_SQL_V1)
                        .map_err(|error| {
                            database("upgrade Archive page provenance anchor", &error)
                        })?;
                    transaction
                        .commit()
                        .map_err(|error| database("commit Archive schema upgrade", &error))?;
                    Ok(ArchiveSchemaDispositionV1::Installed)
                }
            } else {
                Err(SemanticArchiveErrorV1::PartialSchema)
            }
        })();
        let unlock = client
            .query_one(
                "SELECT pg_catalog.pg_advisory_unlock($1)",
                &[&SCHEMA_ADVISORY_LOCK_KEY],
            )
            .and_then(|row| row.try_get::<_, bool>(0))
            .map_err(|error| database("unlock Archive schema installer", &error));
        match (result, unlock) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(disposition), Ok(true)) => Ok(disposition),
            (Ok(_), Ok(false)) => Err(SemanticArchiveErrorV1::SchemaMismatch),
        }
    }

    /// Install the additive Archive atom schema under the shared schema lock.
    ///
    /// The atom schema widens the knowledge-grant subject domain to concepts,
    /// so the base semantic Archive schema must already be installed. The
    /// fog-safe views land in the `public` schema (never `babylon_meta`) with
    /// guarded reader grants.
    ///
    /// # Errors
    /// Refuses partial markers, a wrong contract row, or database failure.
    pub fn install_atom_schema(
        &self,
    ) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
        let mut client = self.connect("connect Archive atom schema installer")?;
        client
            .query_one(
                "SELECT pg_catalog.pg_advisory_lock($1)",
                &[&SCHEMA_ADVISORY_LOCK_KEY],
            )
            .map_err(|error| database("lock Archive atom schema installer", &error))?;
        let result = (|| {
            let row = client
                .query_one(ARCHIVE_ATOM_SCHEMA_MARKERS_SQL_V1, &[])
                .map_err(|error| database("inspect Archive atom schema markers", &error))?;
            let markers = [
                decode::<bool>(&row, 0)?,
                decode::<bool>(&row, 1)?,
                decode::<bool>(&row, 2)?,
                decode::<bool>(&row, 3)?,
                decode::<bool>(&row, 4)?,
                decode::<bool>(&row, 5)?,
            ];
            if markers == [false; 6] {
                let mut transaction = client
                    .build_transaction()
                    .isolation_level(IsolationLevel::Serializable)
                    .start()
                    .map_err(|error| database("begin Archive atom schema install", &error))?;
                transaction
                    .batch_execute(
                        "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
                    )
                    .map_err(|error| {
                        database("set Archive atom schema install settings", &error)
                    })?;
                transaction
                    .batch_execute(ARCHIVE_ATOM_SCHEMA_V1_SQL)
                    .map_err(|error| database("install Archive atom schema", &error))?;
                transaction
                    .commit()
                    .map_err(|error| database("commit Archive atom schema install", &error))?;
                Ok(ArchiveSchemaDispositionV1::Installed)
            } else if markers == [true; 6] {
                let row = client
                    .query_one(
                        "SELECT contract_id FROM babylon_meta.archive_atom_schema_v1",
                        &[],
                    )
                    .map_err(|error| database("read Archive atom schema contract", &error))?;
                let contract_id: String = decode(&row, 0)?;
                if contract_id != ARCHIVE_ATOM_SCHEMA_CONTRACT_ID {
                    return Err(SemanticArchiveErrorV1::SchemaMismatch);
                }
                Ok(ArchiveSchemaDispositionV1::AlreadyCurrent)
            } else {
                Err(SemanticArchiveErrorV1::PartialSchema)
            }
        })();
        let unlock = client
            .query_one(
                "SELECT pg_catalog.pg_advisory_unlock($1)",
                &[&SCHEMA_ADVISORY_LOCK_KEY],
            )
            .and_then(|row| row.try_get::<_, bool>(0))
            .map_err(|error| database("unlock Archive atom schema installer", &error));
        match (result, unlock) {
            (Err(error), _) | (Ok(_), Err(error)) => Err(error),
            (Ok(disposition), Ok(true)) => Ok(disposition),
            (Ok(_), Ok(false)) => Err(SemanticArchiveErrorV1::SchemaMismatch),
        }
    }

    /// Insert one immutable subject or field knowledge grant.
    ///
    /// # Errors
    /// Exact retry succeeds; conflicting provenance or tick refuses.
    pub fn grant_knowledge(
        &self,
        campaign_id: CampaignId,
        grant: &ArchiveKnowledgeGrantV1,
    ) -> Result<(), SemanticArchiveErrorV1> {
        let mut client = self.connect("connect Archive knowledge writer")?;
        insert_grant_row_v1(
            &mut client,
            campaign_id,
            grant.page_ref.kind.as_str(),
            &grant.page_ref.id,
            &grant.grant_key,
            grant.granted_tick,
            &grant.citation,
        )
    }

    /// Materialize one committed receipt's bounded dirty page batch atomically.
    ///
    /// In [`ArchiveMaterializeModeV1::Consume`] mode the receipt's full dirty
    /// set drained: the pages write and the consumption row claims the receipt
    /// inside one serializable transaction. In [`ArchiveMaterializeModeV1::Stage`]
    /// mode dirty pages remain (the PER-318 paged drain): the same atomic
    /// transaction writes this bounded batch without claiming, so the receipt
    /// stays pending, `verified_tick` honestly stalls behind it, and nothing
    /// downstream may treat the receipt as settled. One sweep's page batch
    /// always commits atomically (ADR223); the receipt's full drain converges
    /// across successive sweeps, and an exact restage of a staged batch is a
    /// no-op through the monotonic page guard and content-addressed atoms.
    ///
    /// # Errors
    /// Refuses an absent/mismatched receipt, unknown page subject, template failure,
    /// conflicting prior batch, worker, or knowledge identity, or database failure.
    pub fn materialize_receipt(
        &self,
        campaign_id: CampaignId,
        batch: &ArchiveDirtyBatchV1,
        mode: ArchiveMaterializeModeV1,
    ) -> Result<ArchiveMaterializeReportV1, SemanticArchiveErrorV1> {
        crate::archive_revision::publication::materialize(self, campaign_id, batch, mode)
    }

    pub(crate) fn connect(
        &self,
        operation: &'static str,
    ) -> Result<postgres::Client, SemanticArchiveErrorV1> {
        self.config
            .connect(NoTls)
            .map_err(|error| database(operation, &error))
    }
}

/// Hash the exact schema and template inputs used by the idempotent worker.
#[must_use]
pub fn archive_worker_contract_sha256_v1() -> [u8; 32] {
    let mut bytes = Vec::with_capacity(
        ARCHIVE_WORKER_DOMAIN_V1.len()
            + SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.len()
            + ARCHIVE_ATOM_SCHEMA_V1_SQL.len()
            + ARCHIVE_PAGE_TEMPLATE_SHA256_V1.len(),
    );
    bytes.extend_from_slice(ARCHIVE_WORKER_DOMAIN_V1);
    bytes.extend_from_slice(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.as_bytes());
    bytes.extend_from_slice(ARCHIVE_ATOM_SCHEMA_V1_SQL.as_bytes());
    bytes.extend_from_slice(&ARCHIVE_PAGE_TEMPLATE_SHA256_V1);
    sha256_of(&bytes)
}

/// Insert one immutable knowledge-grant row by exact subject kind and id.
///
/// Page subjects validate through [`ArchivePageRefV1`]; concept subjects
/// validate through [`ArchiveAtomSubjectV1`] (ADR249 R12) because concepts
/// are grant subjects without being page kinds. The insert is idempotent:
/// an exact retry succeeds and any drifted row refuses `GrantConflict`.
pub(crate) fn insert_grant_row_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    subject_kind: &str,
    subject_id: &str,
    grant_key: &str,
    granted_tick: u64,
    citation: &ArchiveCitationV1,
) -> Result<(), SemanticArchiveErrorV1> {
    validate_key(grant_key)?;
    validate_text(citation.source_id())?;
    validate_text(citation.locator())?;
    match subject_kind {
        "county" => {
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, subject_id.to_owned())?;
        }
        "place" => {
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, subject_id.to_owned())?;
        }
        "concept" => {
            ArchiveAtomSubjectV1::try_new(
                ArchiveAtomSubjectKindV1::Concept,
                subject_id.to_owned(),
            )?;
        }
        _ => return Err(SemanticArchiveErrorV1::InvalidIdentity),
    }
    let granted_tick =
        i64::try_from(granted_tick).map_err(|_| SemanticArchiveErrorV1::InvalidVerifiedTick)?;
    let affected = client
        .execute(
            "INSERT INTO babylon_meta.archive_knowledge_grant_v1 \
             (campaign_id, subject_kind, subject_id, grant_key, granted_tick, \
              provenance_source_id, provenance_locator) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (campaign_id, subject_kind, subject_id, grant_key) DO NOTHING",
            &[
                campaign_id.as_uuid(),
                &subject_kind,
                &subject_id,
                &grant_key,
                &granted_tick,
                &citation.source_id,
                &citation.locator,
            ],
        )
        .map_err(|error| database("insert Archive knowledge grant", &error))?;
    if affected == 1 {
        return Ok(());
    }
    let row = client
        .query_one(
            "SELECT granted_tick, provenance_source_id, provenance_locator \
             FROM babylon_meta.archive_knowledge_grant_v1 \
             WHERE campaign_id = $1::uuid AND subject_kind = $2 \
               AND subject_id = $3 AND grant_key = $4",
            &[
                campaign_id.as_uuid(),
                &subject_kind,
                &subject_id,
                &grant_key,
            ],
        )
        .map_err(|error| database("reconcile Archive knowledge grant", &error))?;
    let exact = decode::<i64>(&row, 0)? == granted_tick
        && decode::<String>(&row, 1)? == citation.source_id
        && decode::<String>(&row, 2)? == citation.locator;
    if exact {
        Ok(())
    } else {
        Err(SemanticArchiveErrorV1::GrantConflict)
    }
}

/// Claim the receipt for one materialize pass.
///
/// Stage mode never claims: an existing consumption row means the receipt
/// settled earlier, and the exact retry reconciles the stored claim digests
/// before reporting `AlreadyConsumed`; a differing row refuses with the same
/// conflict Consume mode raises. Consume mode inserts the exact conflict row;
/// an identical prior claim reconciles as already consumed, a differing one
/// refuses.
///
/// # Errors
/// Returns a database failure or a conflicting prior claim.
pub(crate) fn read_knowledge(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<ArchiveKnowledgeV1, SemanticArchiveErrorV1> {
    let rows = client
        .query(
            ARCHIVE_KNOWLEDGE_SQL_V1,
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &i64::try_from(MAX_KNOWLEDGE_GRANTS + 1)
                    .map_err(|_| SemanticArchiveErrorV1::CollectionBound)?,
            ],
        )
        .map_err(|error| database("read Archive knowledge grants", &error))?;
    if rows.len() > MAX_KNOWLEDGE_GRANTS {
        return Err(SemanticArchiveErrorV1::CollectionBound);
    }
    let mut grants = Vec::with_capacity(rows.len());
    for row in rows {
        let kind = decode_subject_kind(&decode::<String>(&row, 0)?)?;
        let page_ref = ArchivePageRefV1::try_new(kind, decode(&row, 1)?)?;
        let grant_key: String = decode(&row, 2)?;
        let granted_tick = u64::try_from(decode::<i64>(&row, 3)?)
            .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
        let citation = ArchiveCitationV1::try_new(decode(&row, 4)?, decode(&row, 5)?)?;
        grants.push(ArchiveKnowledgeGrantV1::try_new(
            page_ref,
            grant_key,
            granted_tick,
            citation,
        )?);
    }
    ArchiveKnowledgeV1::try_new(grants)
}

/// Mint the canonical atom set one known page asserts (ADR249 R1): the
/// subject atom, one atom per known signal, and one link atom per known link
/// target. Ungranted signals and unknown link targets mint nothing, matching
/// the renderer's known-only material.
pub(crate) fn mint_page_atoms(
    campaign_id: CampaignId,
    resolve_tick: u64,
    input: &ArchivePageInputV1,
    knowledge: &ArchiveKnowledgeV1,
) -> Result<Vec<ArchiveAtomV1>, SemanticArchiveErrorV1> {
    let page_ref = input.subject.page_ref();
    let subject = ArchiveAtomSubjectV1::from_page_ref(page_ref)?;
    let mut atoms = Vec::with_capacity(input.signals.len() + input.links.len() + 1);
    if let Some(grant) = knowledge.grant(page_ref, "subject") {
        atoms.push(ArchiveAtomV1::try_new(
            campaign_id,
            subject.clone(),
            "subject".to_owned(),
            "subject".to_owned(),
            ArchiveEvidenceClassV1::Observed,
            &ArchiveAtomValueV1::Text(input.subject.title().to_owned()),
            grant.citation.clone(),
            resolve_tick,
        )?);
    }
    for signal in &input.signals {
        if !knowledge.knows_field(page_ref, signal.grant_key()) {
            continue;
        }
        let evidence_class = signal_evidence_class(page_ref, signal, knowledge)?;
        atoms.push(ArchiveAtomV1::try_new(
            campaign_id,
            subject.clone(),
            signal.grant_key().to_owned(),
            signal.grant_key().to_owned(),
            evidence_class,
            &ArchiveAtomValueV1::Text(signal.value().to_owned()),
            signal.citation().clone(),
            resolve_tick,
        )?);
    }
    for link in &input.links {
        let Some(grant) = knowledge.grant(link.target(), "subject") else {
            continue;
        };
        atoms.push(ArchiveAtomV1::try_new(
            campaign_id,
            subject.clone(),
            "link".to_owned(),
            "subject".to_owned(),
            ArchiveEvidenceClassV1::Observed,
            &ArchiveAtomValueV1::Text(link.target().page_key()),
            grant.citation.clone(),
            resolve_tick,
        )?);
    }
    Ok(atoms)
}

fn signal_evidence_class(
    page_ref: &ArchivePageRefV1,
    signal: &ArchiveSignalV1,
    knowledge: &ArchiveKnowledgeV1,
) -> Result<ArchiveEvidenceClassV1, SemanticArchiveErrorV1> {
    use crate::archive_foundation_grants::county_qcew_citation;
    use crate::michigan_economy::{michigan_economy_v1, QCEW_ECONOMICS_FIELD_KEYS_V1};

    if signal.grant_key() == "identity" {
        return Ok(ArchiveEvidenceClassV1::Observed);
    }
    let Some(index) = QCEW_ECONOMICS_FIELD_KEYS_V1
        .iter()
        .position(|key| *key == signal.grant_key())
        .filter(|_| page_ref.kind() == ArchiveSubjectKindV1::County)
    else {
        return Ok(ArchiveEvidenceClassV1::Derived);
    };
    let citation = county_qcew_citation(page_ref.id());
    if signal.citation() != &citation
        || knowledge
            .grant(page_ref, signal.grant_key())
            .is_none_or(|grant| grant.citation != citation)
    {
        return Ok(ArchiveEvidenceClassV1::Derived);
    }
    let economy = michigan_economy_v1().map_err(|_| SemanticArchiveErrorV1::ArtifactDigest)?;
    let Some(county) = economy
        .counties()
        .iter()
        .find(|county| county.county_geoid == page_ref.id())
    else {
        return Ok(ArchiveEvidenceClassV1::Derived);
    };
    let observed = [
        county.annual_avg_estabs_count,
        county.annual_avg_emplvl,
        county.total_annual_wages,
        county.annual_avg_wkly_wage,
    ][index];
    Ok(if signal.value() == observed.to_string() {
        ArchiveEvidenceClassV1::Observed
    } else {
        ArchiveEvidenceClassV1::Derived
    })
}

/// Persist minted atoms idempotently and re-assert the page composition with
/// contiguous positions inside the same guarded upsert window.
///
/// # Errors
/// Refuses a non-finite stored value, an out-of-range integer, or database failure.
pub(crate) fn persist_atom_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    atoms: &[ArchiveAtomV1],
) -> Result<ArchiveAtomMintV1, SemanticArchiveErrorV1> {
    let mut minted = 0usize;
    for atom in atoms {
        let (text_col, f64_col, u64_col, bool_col): (
            Option<&str>,
            Option<f64>,
            Option<i64>,
            Option<bool>,
        ) = match atom.value() {
            ArchiveAtomValueV1::Text(text) => (Some(text), None, None, None),
            ArchiveAtomValueV1::F64(number) => (None, Some(*number), None, None),
            ArchiveAtomValueV1::U64(number) => {
                let exact =
                    i64::try_from(*number).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?;
                (None, None, Some(exact), None)
            }
            ArchiveAtomValueV1::Bool(flag) => (None, None, None, Some(*flag)),
        };
        let affected = client
            .execute(
                "INSERT INTO babylon_meta.archive_atom_v1 \
                 (atom_id, campaign_id, subject_kind, subject_id, signal_key, grant_key, \
                  evidence_class, value_kind, value_text, value_f64, value_u64, value_bool, \
                  provenance_source_id, provenance_locator, valid_tick) \
                 VALUES ($1::bytea, $2::uuid, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, \
                         $13, $14, $15) \
                 ON CONFLICT (atom_id) DO NOTHING",
                &[
                    &&atom.atom_id()[..],
                    campaign_id.as_uuid(),
                    &atom.subject().kind().as_str(),
                    &atom.subject().id(),
                    &atom.signal_key(),
                    &atom.grant_key(),
                    &atom.evidence_class().as_str(),
                    &atom.value().kind_str(),
                    &text_col,
                    &f64_col,
                    &u64_col,
                    &bool_col,
                    &atom.citation().source_id(),
                    &atom.citation().locator(),
                    &i64::try_from(atom.valid_tick())
                        .map_err(|_| SemanticArchiveErrorV1::InvalidVerifiedTick)?,
                ],
            )
            .map_err(|error| database("insert semantic Archive atom", &error))?;
        if affected == 1 {
            minted += 1;
        }
    }
    Ok(ArchiveAtomMintV1::new(minted, atoms.len()))
}

pub(crate) fn decode_subject_kind(
    value: &str,
) -> Result<ArchiveSubjectKindV1, SemanticArchiveErrorV1> {
    match value {
        "county" => Ok(ArchiveSubjectKindV1::County),
        "place" => Ok(ArchiveSubjectKindV1::Place),
        _ => Err(SemanticArchiveErrorV1::StoredPageMismatch),
    }
}

fn decode_atom_subject_kind(
    value: &str,
) -> Result<ArchiveAtomSubjectKindV1, SemanticArchiveErrorV1> {
    match value {
        "county" => Ok(ArchiveAtomSubjectKindV1::County),
        "place" => Ok(ArchiveAtomSubjectKindV1::Place),
        "concept" => Ok(ArchiveAtomSubjectKindV1::Concept),
        _ => Err(SemanticArchiveErrorV1::StoredPageMismatch),
    }
}

fn decode_evidence_class(value: &str) -> Result<ArchiveEvidenceClassV1, SemanticArchiveErrorV1> {
    match value {
        "Observed" => Ok(ArchiveEvidenceClassV1::Observed),
        "Derived" => Ok(ArchiveEvidenceClassV1::Derived),
        "Calibrated" => Ok(ArchiveEvidenceClassV1::Calibrated),
        "Designed" => Ok(ArchiveEvidenceClassV1::Designed),
        _ => Err(SemanticArchiveErrorV1::StoredPageMismatch),
    }
}

/// Decode one stored atom row with read-time revalidation: every field is
/// revalidated and the canonical identity is recomputed against the stored
/// `atom_id`, so any stored drift refuses with `StoredPageMismatch`.
pub(crate) fn decode_stored_atom(row: &Row) -> Result<ArchiveAtomV1, SemanticArchiveErrorV1> {
    let campaign_id = CampaignId::from_uuid(decode(row, 0)?);
    let kind = decode_atom_subject_kind(&decode::<String>(row, 1)?)?;
    let subject = ArchiveAtomSubjectV1::try_new(kind, decode(row, 2)?)?;
    let signal_key: String = decode(row, 3)?;
    let grant_key: String = decode(row, 4)?;
    let evidence_class = decode_evidence_class(&decode::<String>(row, 5)?)?;
    let value_kind: String = decode(row, 6)?;
    let value = match value_kind.as_str() {
        "text" => ArchiveAtomValueV1::Text(decode(row, 7)?),
        "f64" => {
            let number: f64 = decode(row, 8)?;
            if !number.is_finite() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            ArchiveAtomValueV1::F64(if number == 0.0 { 0.0 } else { number })
        }
        "u64" => {
            let number = u64::try_from(decode::<i64>(row, 9)?)
                .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
            ArchiveAtomValueV1::U64(number)
        }
        "bool" => ArchiveAtomValueV1::Bool(decode(row, 10)?),
        _ => return Err(SemanticArchiveErrorV1::StoredPageMismatch),
    };
    let citation = ArchiveCitationV1::try_new(decode(row, 11)?, decode(row, 12)?)?;
    let valid_tick = u64::try_from(decode::<i64>(row, 13)?)
        .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
    let stored_atom_id = decode_digest(row, 14)?;
    let atom = ArchiveAtomV1::try_new(
        campaign_id,
        subject,
        signal_key,
        grant_key,
        evidence_class,
        &value,
        citation,
        valid_tick,
    )?;
    if atom.atom_id() != stored_atom_id {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    Ok(atom)
}

pub(crate) fn decode<T: FromSqlOwned>(
    row: &Row,
    index: usize,
) -> Result<T, SemanticArchiveErrorV1> {
    row.try_get(index)
        .map_err(|error| database("decode semantic Archive row", &error))
}

pub(crate) fn decode_digest(row: &Row, index: usize) -> Result<[u8; 32], SemanticArchiveErrorV1> {
    let bytes: Vec<u8> = decode(row, index)?;
    bytes
        .try_into()
        .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)
}

pub(crate) fn database(operation: &'static str, error: &postgres::Error) -> SemanticArchiveErrorV1 {
    SemanticArchiveErrorV1::Database {
        operation,
        diagnostic: PostgresDiagnosticV1::capture(error),
    }
}

#[derive(Serialize)]
struct TemplateSignalV1<'a> {
    label: &'a str,
    value: &'a str,
    citation: &'a ArchiveCitationV1,
}

impl<'a> From<&'a ArchiveSignalV1> for TemplateSignalV1<'a> {
    fn from(signal: &'a ArchiveSignalV1) -> Self {
        Self {
            label: &signal.label,
            value: &signal.value,
            citation: &signal.citation,
        }
    }
}

#[derive(Serialize)]
struct TemplateLinkV1<'a> {
    page_key: String,
    known_label: Option<&'a str>,
}

/// Stable closed refusal taxonomy for semantic Archive inputs and rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticArchiveErrorV1 {
    /// County or place identity was malformed.
    InvalidIdentity,
    /// Human-readable or grant-key text was unsafe or unbounded.
    InvalidText,
    /// Tick zero or a value outside `PostgreSQL` `BIGINT` was supplied.
    InvalidVerifiedTick,
    /// A fixed collection ceiling was exceeded.
    CollectionBound,
    /// One work item repeated a signal or link identity.
    DuplicateKey,
    /// One SQL grant row was repeated.
    DuplicateGrant,
    /// SQL did not grant knowledge of the page subject.
    UnknownSubject,
    /// A page or batch did not match its committed dirty receipt.
    ReceiptMismatch,
    /// No marker-backed dirty receipt exists for the requested tick.
    MissingCommittedReceipt,
    /// A different batch, worker, or knowledge snapshot consumed the receipt.
    ReceiptConflict,
    /// An existing knowledge grant differs from the immutable retry.
    GrantConflict,
    /// A history cursor belongs to another scope or unfinished composition.
    ArchiveCursorMismatch,
    /// A worker attempted to pass an earlier pending receipt or unsealed cutover.
    ArchiveOrderViolation,
    /// A producer cannot prove the complete cutover subject domain.
    ArchiveCoverageUnavailable,
    /// Only part of the additive Archive schema exists.
    PartialSchema,
    /// The Archive schema marker or unlock result was not exact.
    SchemaMismatch,
    /// A stored page, digest, kind, tick, or provenance row was malformed.
    StoredPageMismatch,
    /// A pinned reference-artifact digest diverged from its contract-pinned value.
    ArtifactDigest,
    /// One numeric atom value was NaN or infinite at mint (ADR249 R1); the
    /// schema's SQL CHECK backs this refusal in storage.
    NonFiniteValue,
    /// The pinned strict template failed to compile or render.
    Template,
    /// The dirty place set exceeded one receipt page bound, so nothing was
    /// selected and the receipt stays pending.
    ///
    /// Defense-only after PER-318: producers page the head batch and report
    /// the undrained tail instead of refusing, so this variant is unreachable
    /// in normal operation and remains as the typed taxonomy backstop.
    PlaceDrainOverflow {
        /// Exact number of dirty place pages observed.
        dirty: usize,
        /// The one-receipt page bound that the dirty set exceeded.
        limit: usize,
    },
    /// The dirty county set exceeded one receipt page bound, so nothing was
    /// selected and the receipt stays pending.
    ///
    /// Defense-only after PER-318: producers page the head batch and report
    /// the undrained tail instead of refusing, so this variant is unreachable
    /// in normal operation and remains as the typed taxonomy backstop.
    CountyDrainOverflow {
        /// Exact number of dirty county pages observed.
        dirty: usize,
        /// The one-receipt page bound that the dirty set exceeded.
        limit: usize,
    },
    /// Cooperative stop refused a new publication or rolled back uncommitted work.
    WorkerCanceled,
    /// One database operation failed with a bounded secret-safe driver diagnostic.
    Database {
        /// Stable operation identity.
        operation: &'static str,
        /// Secret-safe `PostgreSQL` classification, SQLSTATE, and message.
        diagnostic: PostgresDiagnosticV1,
    },
}

impl std::fmt::Display for SemanticArchiveErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "semantic Archive refusal: {self:?}")
    }
}

impl std::error::Error for SemanticArchiveErrorV1 {}

pub(crate) fn validate_text(value: &str) -> Result<(), SemanticArchiveErrorV1> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.as_bytes().contains(&0) {
        return Err(SemanticArchiveErrorV1::InvalidText);
    }
    Ok(())
}

pub(crate) fn validate_key(value: &str) -> Result<(), SemanticArchiveErrorV1> {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return Err(SemanticArchiveErrorV1::InvalidText);
    };
    if value.len() > MAX_ID_BYTES
        || !(first.is_ascii_lowercase() || first.is_ascii_digit())
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(SemanticArchiveErrorV1::InvalidText);
    }
    Ok(())
}

fn hash_page_input(hasher: &mut Sha256, input: &ArchivePageInputV1) {
    hash_page_ref(hasher, input.subject.page_ref());
    hash_bytes(hasher, input.subject.title.as_bytes());
    hasher.update(input.verified_tick.to_be_bytes());
    hasher.update(input.tick_content_hash);
    hash_bytes(hasher, input.decision_question.as_bytes());
    hash_len(hasher, input.signals.len());
    for signal in &input.signals {
        hash_bytes(hasher, signal.grant_key.as_bytes());
        hash_bytes(hasher, signal.label.as_bytes());
        hash_bytes(hasher, signal.value.as_bytes());
        hash_citation(hasher, &signal.citation);
    }
    hash_len(hasher, input.links.len());
    for link in &input.links {
        hash_page_ref(hasher, &link.target);
        hash_bytes(hasher, link.known_label.as_bytes());
    }
}

fn hash_page_ref(hasher: &mut Sha256, page_ref: &ArchivePageRefV1) {
    hasher.update([match page_ref.kind {
        ArchiveSubjectKindV1::County => 1,
        ArchiveSubjectKindV1::Place => 2,
    }]);
    hash_bytes(hasher, page_ref.id.as_bytes());
}

fn hash_citation(hasher: &mut Sha256, citation: &ArchiveCitationV1) {
    hash_bytes(hasher, citation.source_id.as_bytes());
    hash_bytes(hasher, citation.locator.as_bytes());
}

fn hash_len(hasher: &mut Sha256, len: usize) {
    hasher.update(u64::try_from(len).unwrap_or(u64::MAX).to_be_bytes());
}

fn hash_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hash_len(hasher, bytes.len());
    hasher.update(bytes);
}

pub(crate) fn hex_digest(digest: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public_qcew_page() -> (CampaignId, ArchivePageInputV1, ArchiveKnowledgeV1) {
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(319));
        let county = crate::michigan_economy::michigan_economy_v1()
            .unwrap()
            .counties()
            .iter()
            .find(|county| county.county_geoid == "26163")
            .unwrap();
        let fields = crate::CommittedTerritoryFieldsV1::try_from_qcew([
            Some(i64::try_from(county.annual_avg_estabs_count).unwrap()),
            Some(i64::try_from(county.annual_avg_emplvl).unwrap()),
            Some(i64::try_from(county.total_annual_wages).unwrap()),
            Some(i64::try_from(county.annual_avg_wkly_wage).unwrap()),
        ])
        .unwrap();
        let plan = crate::CountyPagePlanV1::try_new(
            county.county_geoid.clone(),
            "county-26163".to_owned(),
            "Wayne County".to_owned(),
            crate::county_committed_signals_v1(&fields).unwrap(),
            Vec::new(),
        )
        .unwrap();
        let page = crate::county_page_input_v1(&plan, 1, [1; 32]).unwrap();
        let grants = crate::foundation_grant_rows_v1()
            .unwrap()
            .into_iter()
            .filter(|row| {
                row.subject().kind() == ArchiveAtomSubjectKindV1::County
                    && row.subject().id() == county.county_geoid
            })
            .map(|row| {
                ArchiveKnowledgeGrantV1::try_new(
                    page.subject().page_ref().clone(),
                    row.grant_key().to_owned(),
                    0,
                    row.citation().clone(),
                )
                .unwrap()
            })
            .collect();
        (campaign, page, ArchiveKnowledgeV1::try_new(grants).unwrap())
    }

    #[test]
    fn pinned_public_qcew_atoms_retain_observed_classification() {
        let (campaign, page, grants) = public_qcew_page();
        let atoms = mint_page_atoms(campaign, 1, &page, &grants).unwrap();
        let qcew: Vec<_> = atoms
            .iter()
            .filter(|atom| atom.signal_key() != "subject")
            .collect();
        assert_eq!(qcew.len(), 4);
        for atom in qcew {
            assert_eq!(atom.evidence_class(), ArchiveEvidenceClassV1::Observed);
        }
    }

    #[test]
    fn qcew_names_cannot_reclassify_unpinned_or_changed_values_as_observed() {
        let (campaign, page, grants) = public_qcew_page();
        for field in ["source", "county", "digest", "value", "grant"] {
            let mut changed = page.clone();
            let mut changed_grants = grants.clone();
            let signal = &mut changed.signals[0];
            match field {
                "source" => signal.citation.source_id = "committed-tick-v1".to_owned(),
                "county" => {
                    signal.citation.locator = signal.citation.locator.replace("26163", "26125");
                }
                "digest" => signal.citation.locator.push('0'),
                "value" => signal.value = "1".to_owned(),
                "grant" => {
                    changed_grants
                        .grants
                        .get_mut(&(
                            changed.subject.page_ref().clone(),
                            signal.grant_key().to_owned(),
                        ))
                        .unwrap()
                        .citation
                        .source_id = "uncited-local-claim".to_owned();
                }
                _ => unreachable!(),
            }
            let key = signal.grant_key().to_owned();
            let atoms = mint_page_atoms(campaign, 1, &changed, &changed_grants).unwrap();
            let atom = atoms.iter().find(|atom| atom.signal_key() == key).unwrap();
            assert_eq!(
                atom.evidence_class(),
                ArchiveEvidenceClassV1::Derived,
                "{field}"
            );
        }
    }

    #[test]
    fn strict_mode_refuses_missing_template_names() {
        let mut environment = Environment::empty();
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment
            .add_template("missing", "{{ absent }}")
            .expect("test template compiles");
        assert!(environment
            .get_template("missing")
            .expect("test template exists")
            .render(context! {})
            .is_err());
    }
}
