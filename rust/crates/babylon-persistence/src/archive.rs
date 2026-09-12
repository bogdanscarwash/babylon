//! Fog-safe semantic Archive page and knowledge contracts.

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::content_digest::sha256_of;
use minijinja::{context, Environment, UndefinedBehavior};
use postgres::types::FromSqlOwned;
use postgres::{Config, GenericClient, IsolationLevel, NoTls, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::archive_revision::emission::{ArchiveEmissionLink, ArchiveEmissionManifest};
use crate::identity::CampaignId;
use crate::postgres_diagnostic::PostgresDiagnostic;

/// Current Archive schema installed atomically with the material runtime schema.
pub const CURRENT_ARCHIVE_SCHEMA_SQL: &str = include_str!("../migrations/current_archive.sql");
const ARCHIVE_PAGE_TEMPLATE: &str = include_str!("archive_page_v1.md.j2");
const MAX_ID_BYTES: usize = 128;
const MAX_TEXT_BYTES: usize = 4_096;
pub(crate) const MAX_SIGNALS: usize = 256;
pub(crate) const MAX_LINKS: usize = 256;
const MAX_KNOWLEDGE_GRANTS: usize = 65_535;
pub(crate) const MAX_PAGE_BYTES: usize = 1_048_576;
const ARCHIVE_WORKER_DOMAIN: &[u8] = b"babylon.semantic-archive-worker.v1\0";
const ARCHIVE_DIRTY_BATCH_DOMAIN: &[u8] = b"babylon.semantic-archive-dirty-batch.v1\0";
const ARCHIVE_KNOWLEDGE_DOMAIN: &[u8] = b"babylon.semantic-archive-knowledge.v1\0";
const ARCHIVE_ATOM_DOMAIN: &[u8] = b"babylon.semantic-archive-atom.v1\0";
/// SQL-only knowledge boundary used before any template receives values.
/// Page-subject knowledge only: seeded concept grants widen the grant table's
/// subject domain (ADR249 R3/R12) but never enter the page knowledge snapshot.
pub const ARCHIVE_KNOWLEDGE_SQL: &str = "SELECT subject_kind, subject_id, grant_key, \
    granted_tick, provenance_source_id, provenance_locator \
    FROM babylon_meta.archive_knowledge_grant_v1 \
    WHERE campaign_id = $1::uuid AND granted_tick <= $2 \
      AND subject_kind IN ('county', 'place') \
    ORDER BY subject_kind, subject_id, grant_key LIMIT $3";
/// SHA-256 of the pinned strict `MiniJinja` page template.
pub const ARCHIVE_PAGE_TEMPLATE_SHA256: [u8; 32] = [
    0xd7, 0x90, 0x43, 0x79, 0xcf, 0x09, 0xf4, 0x1d, 0xb6, 0xab, 0xea, 0x91, 0x46, 0x5b, 0x5f, 0xe6,
    0xe8, 0x04, 0x86, 0x7c, 0xf8, 0x76, 0xbd, 0x44, 0xa0, 0x9f, 0xe6, 0x3b, 0xa9, 0x75, 0x51, 0x08,
];

/// Closed semantic page kinds in the first Archive slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ArchiveSubjectKind {
    /// United States county identified by five-digit Census FIPS.
    County,
    /// Census-designated place identified by seven-digit place GEOID.
    Place,
}

impl ArchiveSubjectKind {
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
pub struct ArchivePageRef {
    kind: ArchiveSubjectKind,
    id: String,
}

impl ArchivePageRef {
    /// Construct one exact county or place identity.
    ///
    /// # Errors
    /// Refuses the wrong digit width or an unbounded identity.
    pub fn try_new(kind: ArchiveSubjectKind, id: String) -> Result<Self, SemanticArchiveError> {
        let expected = match kind {
            ArchiveSubjectKind::County => 5,
            ArchiveSubjectKind::Place => 7,
        };
        if id.len() != expected
            || id.len() > MAX_ID_BYTES
            || !id.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(SemanticArchiveError::InvalidIdentity);
        }
        Ok(Self { kind, id })
    }

    /// Return the closed subject kind.
    #[must_use]
    pub const fn kind(&self) -> ArchiveSubjectKind {
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
pub struct ArchiveSubject {
    page_ref: ArchivePageRef,
    title: String,
}

impl ArchiveSubject {
    /// Construct one page subject.
    ///
    /// # Errors
    /// Refuses an invalid identity or unsafe title.
    pub fn try_new(
        kind: ArchiveSubjectKind,
        id: String,
        title: String,
    ) -> Result<Self, SemanticArchiveError> {
        validate_text(&title)?;
        Ok(Self {
            page_ref: ArchivePageRef::try_new(kind, id)?,
            title,
        })
    }

    /// Borrow the stable page reference.
    #[must_use]
    pub const fn page_ref(&self) -> &ArchivePageRef {
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
pub struct ArchiveCitation {
    source_id: String,
    locator: String,
}

#[derive(Deserialize)]
struct UnvalidatedArchiveCitation {
    source_id: String,
    locator: String,
}

impl<'de> Deserialize<'de> for ArchiveCitation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let citation = UnvalidatedArchiveCitation::deserialize(deserializer)?;
        Self::try_new(citation.source_id, citation.locator).map_err(serde::de::Error::custom)
    }
}

impl ArchiveCitation {
    /// Construct a bounded source citation.
    ///
    /// # Errors
    /// Refuses an empty, NUL-containing, or unbounded component.
    pub fn try_new(source_id: String, locator: String) -> Result<Self, SemanticArchiveError> {
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
pub struct ArchiveSignal {
    grant_key: String,
    label: String,
    value: String,
    citation: ArchiveCitation,
}

impl ArchiveSignal {
    /// Construct one bounded signal.
    ///
    /// # Errors
    /// Refuses an unsafe grant key, label, or value.
    pub fn try_new(
        grant_key: String,
        label: String,
        value: String,
        citation: ArchiveCitation,
    ) -> Result<Self, SemanticArchiveError> {
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
    pub const fn citation(&self) -> &ArchiveCitation {
        &self.citation
    }
}

/// One outbound semantic page link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveLink {
    target: ArchivePageRef,
    known_label: String,
}

impl ArchiveLink {
    /// Construct one outbound link whose label may be shown only when known.
    ///
    /// # Errors
    /// Refuses an unsafe label.
    pub fn try_new(
        target: ArchivePageRef,
        known_label: String,
    ) -> Result<Self, SemanticArchiveError> {
        validate_text(&known_label)?;
        Ok(Self {
            target,
            known_label,
        })
    }

    /// Borrow the exact link target identity.
    #[must_use]
    pub const fn target(&self) -> &ArchivePageRef {
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
pub struct ArchivePageInput {
    subject: ArchiveSubject,
    verified_tick: u64,
    tick_content_hash: [u8; 32],
    decision_question: String,
    signals: Vec<ArchiveSignal>,
    links: Vec<ArchiveLink>,
}

impl ArchivePageInput {
    /// Construct one bounded dirty-subject work item.
    ///
    /// # Errors
    /// Refuses synthetic tick zero, duplicate keys, or unbounded collections.
    pub fn try_new(
        subject: ArchiveSubject,
        verified_tick: u64,
        tick_content_hash: [u8; 32],
        decision_question: String,
        signals: Vec<ArchiveSignal>,
        links: Vec<ArchiveLink>,
    ) -> Result<Self, SemanticArchiveError> {
        if verified_tick == 0 || verified_tick > i64::MAX as u64 {
            return Err(SemanticArchiveError::InvalidVerifiedTick);
        }
        validate_text(&decision_question)?;
        if signals.len() > MAX_SIGNALS || links.len() > MAX_LINKS {
            return Err(SemanticArchiveError::CollectionBound);
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
            return Err(SemanticArchiveError::DuplicateKey);
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
    pub const fn subject(&self) -> &ArchiveSubject {
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
    pub fn signals(&self) -> &[ArchiveSignal] {
        &self.signals
    }

    /// Borrow the ordered outbound links.
    #[must_use]
    pub fn links(&self) -> &[ArchiveLink] {
        &self.links
    }
}

/// Exact SQL-derived knowledge grants supplied to the pure renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveKnowledge {
    grants: BTreeMap<(ArchivePageRef, String), ArchiveKnowledgeGrant>,
}

impl ArchiveKnowledge {
    /// Validate an exact SQL grant result.
    ///
    /// # Errors
    /// Refuses duplicate rows or a malformed key or citation.
    pub fn try_new(grants: Vec<ArchiveKnowledgeGrant>) -> Result<Self, SemanticArchiveError> {
        let mut indexed = BTreeMap::new();
        for grant in grants {
            validate_key(&grant.grant_key)?;
            validate_text(grant.citation.source_id())?;
            validate_text(grant.citation.locator())?;
            let key = (grant.page_ref.clone(), grant.grant_key.clone());
            if indexed.insert(key, grant).is_some() {
                return Err(SemanticArchiveError::DuplicateGrant);
            }
        }
        Ok(Self { grants: indexed })
    }

    pub(crate) fn knows_subject(&self, page_ref: &ArchivePageRef) -> bool {
        self.grant(page_ref, "subject").is_some()
    }

    pub(crate) fn knows_field(&self, page_ref: &ArchivePageRef, grant_key: &str) -> bool {
        self.grant(page_ref, grant_key).is_some()
    }

    pub(crate) fn grant(
        &self,
        page_ref: &ArchivePageRef,
        grant_key: &str,
    ) -> Option<&ArchiveKnowledgeGrant> {
        self.grants.get(&(page_ref.clone(), grant_key.to_owned()))
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = &ArchiveKnowledgeGrant> {
        self.grants.values()
    }

    /// Hash every exact, ordered knowledge-grant row in this snapshot.
    #[must_use]
    pub fn sha256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(ARCHIVE_KNOWLEDGE_DOMAIN);
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
pub struct RenderedArchivePage {
    markdown: String,
    search_text: String,
    citations: Vec<ArchiveCitation>,
    sha256: [u8; 32],
}

impl RenderedArchivePage {
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
    pub fn citations(&self) -> &[ArchiveCitation] {
        &self.citations
    }

    /// Return SHA-256 of exact Markdown bytes.
    #[must_use]
    pub const fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
}

/// Pinned strict `MiniJinja` rendering authority.
pub struct FogSafeArchiveRenderer {
    environment: Environment<'static>,
}

impl FogSafeArchiveRenderer {
    /// Compile the one embedded template with strict undefined behavior.
    ///
    /// # Errors
    /// Returns [`SemanticArchiveError::Template`] for checked-in syntax drift.
    pub fn new() -> Result<Self, SemanticArchiveError> {
        let mut environment = Environment::empty();
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment
            .add_template("archive-page-v1", ARCHIVE_PAGE_TEMPLATE)
            .map_err(|_| SemanticArchiveError::Template)?;
        Ok(Self { environment })
    }

    /// Render one known subject with SQL-derived field and link grants.
    ///
    /// # Errors
    /// Refuses an unknown subject or any strict template failure.
    pub fn render(
        &self,
        input: &ArchivePageInput,
        knowledge: &ArchiveKnowledge,
    ) -> Result<RenderedArchivePage, SemanticArchiveError> {
        self.render_with_emission(input, knowledge)
            .map(|(page, _)| page)
    }

    pub(crate) fn render_with_emission(
        &self,
        input: &ArchivePageInput,
        knowledge: &ArchiveKnowledge,
    ) -> Result<(RenderedArchivePage, ArchiveEmissionManifest), SemanticArchiveError> {
        let subject_grant = knowledge
            .grant(input.subject.page_ref(), "subject")
            .ok_or(SemanticArchiveError::UnknownSubject)?;
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
                ArchiveEmissionLink::try_new(
                    link.target.clone(),
                    knowledge
                        .knows_subject(&link.target)
                        .then(|| link.known_label.clone()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let emission = ArchiveEmissionManifest::try_new(
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
        subject: &ArchiveSubject,
        verified_tick: u64,
        source_hash: &[u8; 32],
        emission: &ArchiveEmissionManifest,
    ) -> Result<RenderedArchivePage, SemanticArchiveError> {
        let signals = emission
            .signals()
            .iter()
            .map(TemplateSignal::from)
            .collect::<Vec<_>>();
        let links = emission
            .links()
            .iter()
            .map(|link| TemplateLink {
                page_key: link.target().page_key(),
                known_label: link.known_label(),
            })
            .collect::<Vec<_>>();
        let tick_content_hash = hex_digest(source_hash);
        let template = self
            .environment
            .get_template("archive-page-v1")
            .map_err(|_| SemanticArchiveError::Template)?;
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
            .map_err(|_| SemanticArchiveError::Template)?;
        let search_text = emission.search_text(subject);
        let citations = emission.citations();
        if markdown.len() > MAX_PAGE_BYTES || search_text.len() > MAX_PAGE_BYTES {
            return Err(SemanticArchiveError::CollectionBound);
        }
        let sha256 = sha256_of(markdown.as_bytes());
        Ok(RenderedArchivePage {
            markdown,
            search_text,
            citations,
            sha256,
        })
    }

    /// Return the checked-in template identity.
    #[must_use]
    pub const fn template_sha256(&self) -> [u8; 32] {
        ARCHIVE_PAGE_TEMPLATE_SHA256
    }
}

/// One bounded batch bound to a single committed dirty receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDirtyBatch {
    resolve_tick: u64,
    tick_content_hash: [u8; 32],
    pages: Vec<ArchivePageInput>,
}

impl ArchiveDirtyBatch {
    /// Maximum number of dirty pages consumed from one committed receipt.
    pub const MAX_PAGES: usize = 256;

    /// Borrow the ordered page inputs bound to this receipt.
    #[must_use]
    pub fn pages(&self) -> &[ArchivePageInput] {
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
        pages: Vec<ArchivePageInput>,
    ) -> Result<Self, SemanticArchiveError> {
        if resolve_tick == 0 || resolve_tick > i64::MAX as u64 {
            return Err(SemanticArchiveError::InvalidVerifiedTick);
        }
        if pages.len() > Self::MAX_PAGES {
            return Err(SemanticArchiveError::CollectionBound);
        }
        let subjects = pages
            .iter()
            .map(|page| page.subject.page_ref())
            .collect::<BTreeSet<_>>();
        if subjects.len() != pages.len() {
            return Err(SemanticArchiveError::DuplicateKey);
        }
        if pages.iter().any(|page| {
            page.verified_tick != resolve_tick || page.tick_content_hash != tick_content_hash
        }) {
            return Err(SemanticArchiveError::ReceiptMismatch);
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
        hasher.update(ARCHIVE_DIRTY_BATCH_DOMAIN);
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
pub struct ArchiveKnowledgeGrant {
    pub(crate) page_ref: ArchivePageRef,
    pub(crate) grant_key: String,
    pub(crate) granted_tick: u64,
    pub(crate) citation: ArchiveCitation,
}

impl ArchiveKnowledgeGrant {
    /// Construct one subject (`subject`) or field grant.
    ///
    /// # Errors
    /// Refuses an unsafe key or a tick outside `PostgreSQL` `BIGINT`.
    pub fn try_new(
        page_ref: ArchivePageRef,
        grant_key: String,
        granted_tick: u64,
        citation: ArchiveCitation,
    ) -> Result<Self, SemanticArchiveError> {
        validate_key(&grant_key)?;
        validate_text(citation.source_id())?;
        validate_text(citation.locator())?;
        if granted_tick > i64::MAX as u64 {
            return Err(SemanticArchiveError::InvalidVerifiedTick);
        }
        Ok(Self {
            page_ref,
            grant_key,
            granted_tick,
            citation,
        })
    }
}

/// Idempotent receipt-consumption result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMaterializeDisposition {
    /// This invocation consumed and rendered the receipt.
    Applied,
    /// The exact batch, worker, and knowledge snapshot already consumed the receipt.
    AlreadyConsumed,
}

/// How one materialization pass relates to the receipt's consumption row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMaterializeMode {
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
pub struct MaterializedArchivePage {
    pub(crate) page_ref: ArchivePageRef,
    pub(crate) page: RenderedArchivePage,
    pub(crate) persisted: bool,
    pub(crate) atoms: ArchiveAtomMint,
}

impl MaterializedArchivePage {
    /// Borrow the stable page identity.
    #[must_use]
    pub const fn page_ref(&self) -> &ArchivePageRef {
        &self.page_ref
    }

    /// Borrow the rendered artifact.
    #[must_use]
    pub const fn page(&self) -> &RenderedArchivePage {
        &self.page
    }

    /// Whether this page replaced the current materialization.
    #[must_use]
    pub const fn persisted(&self) -> bool {
        self.persisted
    }

    /// Borrow the atom mint result for this page.
    #[must_use]
    pub const fn atoms(&self) -> &ArchiveAtomMint {
        &self.atoms
    }
}

/// Receipt-level worker report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveMaterializeReport {
    pub(crate) disposition: ArchiveMaterializeDisposition,
    pub(crate) pages: Vec<MaterializedArchivePage>,
}

impl ArchiveMaterializeReport {
    /// Return whether this invocation applied or observed an exact retry.
    #[must_use]
    pub const fn disposition(&self) -> ArchiveMaterializeDisposition {
        self.disposition
    }

    /// Borrow rendered page results in caller-supplied order.
    #[must_use]
    pub fn pages(&self) -> &[MaterializedArchivePage] {
        &self.pages
    }
}

/// Governed evidence classification carried by every semantic atom
/// (constitutional compact: Observed, Derived, Calibrated, Designed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArchiveEvidenceClass {
    /// A fact read from a pinned source.
    Observed,
    /// A deterministic measure produced from pinned facts.
    Derived,
    /// A value fitted against observation under a declared rule.
    Calibrated,
    /// A value fixed by game design.
    Designed,
}

impl ArchiveEvidenceClass {
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
/// concepts. Unlike [`ArchiveSubjectKind`] this domain is not a page kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ArchiveAtomSubjectKind {
    /// United States county identified by five-digit Census FIPS.
    County,
    /// Census-designated place identified by seven-digit place GEOID.
    Place,
    /// Glossary concept identified by its concept key.
    Concept,
}

impl ArchiveAtomSubjectKind {
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
pub struct ArchiveAtomSubject {
    kind: ArchiveAtomSubjectKind,
    id: String,
}

impl ArchiveAtomSubject {
    /// Construct one bounded atom subject identity.
    ///
    /// # Errors
    /// Refuses a malformed per-kind id: five-digit county, seven-digit place,
    /// or a concept key matching ``^[a-z0-9][a-z0-9-]{0,127}$`` exactly.
    pub fn try_new(kind: ArchiveAtomSubjectKind, id: String) -> Result<Self, SemanticArchiveError> {
        let exact = match kind {
            ArchiveAtomSubjectKind::County => {
                id.len() == 5 && id.bytes().all(|byte| byte.is_ascii_digit())
            }
            ArchiveAtomSubjectKind::Place => {
                id.len() == 7 && id.bytes().all(|byte| byte.is_ascii_digit())
            }
            ArchiveAtomSubjectKind::Concept => {
                let mut bytes = id.bytes();
                matches!(bytes.next(), Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit())
                    && bytes.all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            }
        };
        if !exact || id.len() > MAX_ID_BYTES || id.is_empty() {
            return Err(SemanticArchiveError::InvalidIdentity);
        }
        Ok(Self { kind, id })
    }

    /// Adapt one page subject reference into an atom subject.
    ///
    /// # Errors
    /// Refuses a malformed identity; page references are prevalidated, so this
    /// cannot fail for refs produced by [`ArchivePageRef::try_new`].
    pub fn from_page_ref(page_ref: &ArchivePageRef) -> Result<Self, SemanticArchiveError> {
        let kind = match page_ref.kind() {
            ArchiveSubjectKind::County => ArchiveAtomSubjectKind::County,
            ArchiveSubjectKind::Place => ArchiveAtomSubjectKind::Place,
        };
        Self::try_new(kind, page_ref.id().to_owned())
    }

    /// Return the closed atom subject kind.
    #[must_use]
    pub const fn kind(&self) -> ArchiveAtomSubjectKind {
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
pub enum ArchiveAtomValue {
    /// Bounded UTF-8 text.
    Text(String),
    /// Canonical finite binary64; `-0.0` normalizes to `+0.0`.
    F64(f64),
    /// Exact unsigned 64-bit integer.
    U64(u64),
    /// Exact boolean.
    Bool(bool),
}

impl ArchiveAtomValue {
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
pub struct ArchiveAtom {
    campaign_id: CampaignId,
    subject: ArchiveAtomSubject,
    signal_key: String,
    grant_key: String,
    evidence_class: ArchiveEvidenceClass,
    value: ArchiveAtomValue,
    citation: ArchiveCitation,
    valid_tick: u64,
    atom_id: [u8; 32],
}

impl ArchiveAtom {
    /// Validate and mint one canonical atom.
    ///
    /// # Errors
    /// Refuses a malformed subject, key, citation, or tick; refuses a
    /// non-finite `f64` value with [`SemanticArchiveError::NonFiniteValue`]
    /// so refusal vectors can name it (R1).
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        campaign_id: CampaignId,
        subject: ArchiveAtomSubject,
        signal_key: String,
        grant_key: String,
        evidence_class: ArchiveEvidenceClass,
        value: &ArchiveAtomValue,
        citation: ArchiveCitation,
        valid_tick: u64,
    ) -> Result<Self, SemanticArchiveError> {
        validate_key(&signal_key)?;
        validate_key(&grant_key)?;
        validate_text(citation.source_id())?;
        validate_text(citation.locator())?;
        if valid_tick > i64::MAX as u64 {
            return Err(SemanticArchiveError::InvalidVerifiedTick);
        }
        let canonical = match value {
            ArchiveAtomValue::Text(text) => {
                if text.is_empty() || text.len() > MAX_TEXT_BYTES || text.as_bytes().contains(&0) {
                    return Err(SemanticArchiveError::InvalidText);
                }
                ArchiveAtomValue::Text(text.clone())
            }
            ArchiveAtomValue::F64(number) => {
                if !number.is_finite() {
                    return Err(SemanticArchiveError::NonFiniteValue);
                }
                // Parity doctrine: -0.0 canonicalizes to +0.0 so a sign-only
                // bit difference never mints a second identity.
                ArchiveAtomValue::F64(if *number == 0.0 { 0.0 } else { *number })
            }
            ArchiveAtomValue::U64(number) => ArchiveAtomValue::U64(*number),
            ArchiveAtomValue::Bool(flag) => ArchiveAtomValue::Bool(*flag),
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
    pub const fn subject(&self) -> &ArchiveAtomSubject {
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
    pub const fn evidence_class(&self) -> ArchiveEvidenceClass {
        self.evidence_class
    }

    /// Borrow the typed canonical value.
    #[must_use]
    pub const fn value(&self) -> &ArchiveAtomValue {
        &self.value
    }

    /// Borrow the pinned provenance citation.
    #[must_use]
    pub const fn citation(&self) -> &ArchiveCitation {
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
pub const fn archive_atom_visible(
    atom: &ArchiveAtom,
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
pub struct ArchiveAtomMint {
    minted: usize,
    joined: usize,
}

impl ArchiveAtomMint {
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
    subject: &ArchiveAtomSubject,
    signal_key: &str,
    grant_key: &str,
    evidence_class: ArchiveEvidenceClass,
    value: &ArchiveAtomValue,
    citation: &ArchiveCitation,
    valid_tick: u64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ARCHIVE_ATOM_DOMAIN);
    hasher.update(campaign_id.as_bytes());
    hasher.update([subject.kind.tag()]);
    hash_bytes(&mut hasher, subject.id.as_bytes());
    hash_bytes(&mut hasher, signal_key.as_bytes());
    hash_bytes(&mut hasher, grant_key.as_bytes());
    hasher.update([evidence_class.tag()]);
    hasher.update([value.tag()]);
    match value {
        ArchiveAtomValue::Text(text) => hash_bytes(&mut hasher, text.as_bytes()),
        ArchiveAtomValue::F64(number) => {
            let canonical = if *number == 0.0 { 0.0 } else { *number };
            hasher.update(canonical.to_bits().to_be_bytes());
        }
        ArchiveAtomValue::U64(number) => hasher.update(number.to_be_bytes()),
        ArchiveAtomValue::Bool(flag) => hasher.update([u8::from(*flag)]),
    }
    hash_citation(&mut hasher, citation);
    hasher.update(valid_tick.to_be_bytes());
    hasher.finalize().into()
}

/// `PostgreSQL` adapter for the client-owned epistemic Archive tier.
#[derive(Clone)]
pub struct SemanticArchiveStore {
    config: Config,
}

impl SemanticArchiveStore {
    /// Bind the worker to one Rust-authoritative `PostgreSQL` target.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            config: crate::current_schema::bounded_config(config),
        }
    }

    /// Verify the current runtime schema and its Archive wake hints.
    ///
    /// # Errors
    /// Refuses unsupported schema identity, altered wake hints,
    /// or database failure. Schema creation belongs to the atomic runtime bootstrap.
    pub fn verify_schema(&self) -> Result<(), SemanticArchiveError> {
        let mut client = self.connect("connect Archive schema verifier")?;
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .start()
            .map_err(|error| database("begin Archive schema verification", &error))?;
        crate::current_schema::require_current_schema(&mut tx)
            .map_err(SemanticArchiveError::CurrentSchema)?;
        crate::archive_wakeup::validate(&mut tx)?;
        tx.commit()
            .map_err(|error| database("commit Archive schema verification", &error))
    }

    /// Insert one immutable subject or field knowledge grant.
    ///
    /// # Errors
    /// Exact retry succeeds; conflicting provenance or tick refuses.
    pub fn grant_knowledge(
        &self,
        campaign_id: CampaignId,
        grant: &ArchiveKnowledgeGrant,
    ) -> Result<(), SemanticArchiveError> {
        let mut client = self.connect("connect Archive knowledge writer")?;
        let mut transaction = client
            .build_transaction()
            .read_only(false)
            .start()
            .map_err(|error| database("begin Archive knowledge grant", &error))?;
        crate::current_schema::require_current_schema(&mut transaction)
            .map_err(SemanticArchiveError::CurrentSchema)?;
        insert_grant_row(
            &mut transaction,
            campaign_id,
            grant.page_ref.kind.as_str(),
            &grant.page_ref.id,
            &grant.grant_key,
            grant.granted_tick,
            &grant.citation,
        )?;
        transaction
            .commit()
            .map_err(|error| database("commit Archive knowledge grant", &error))
    }

    /// Materialize one committed receipt's bounded dirty page batch atomically.
    ///
    /// In [`ArchiveMaterializeMode::Consume`] mode the receipt's full dirty
    /// set drained: the pages write and the consumption row claims the receipt
    /// inside one serializable transaction. In [`ArchiveMaterializeMode::Stage`]
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
        batch: &ArchiveDirtyBatch,
        mode: ArchiveMaterializeMode,
    ) -> Result<ArchiveMaterializeReport, SemanticArchiveError> {
        crate::archive_revision::publication::materialize(self, campaign_id, batch, mode)
    }

    pub(crate) fn connect(
        &self,
        operation: &'static str,
    ) -> Result<postgres::Client, SemanticArchiveError> {
        self.config
            .connect(NoTls)
            .map_err(|error| database(operation, &error))
    }
}

/// Hash the exact schema and template inputs used by the idempotent worker.
#[must_use]
pub fn archive_worker_contract_sha256() -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(ARCHIVE_WORKER_DOMAIN);
    hash.update(CURRENT_ARCHIVE_SCHEMA_SQL.as_bytes());
    hash.update(ARCHIVE_PAGE_TEMPLATE_SHA256);
    hash.finalize().into()
}

/// Insert one immutable knowledge-grant row by exact subject kind and id.
///
/// Page subjects validate through [`ArchivePageRef`]; concept subjects
/// validate through [`ArchiveAtomSubject`] (ADR249 R12) because concepts
/// are grant subjects without being page kinds. The insert is idempotent:
/// an exact retry succeeds and any drifted row refuses `GrantConflict`.
pub(crate) fn insert_grant_row(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    subject_kind: &str,
    subject_id: &str,
    grant_key: &str,
    granted_tick: u64,
    citation: &ArchiveCitation,
) -> Result<(), SemanticArchiveError> {
    validate_key(grant_key)?;
    validate_text(citation.source_id())?;
    validate_text(citation.locator())?;
    match subject_kind {
        "county" => {
            ArchivePageRef::try_new(ArchiveSubjectKind::County, subject_id.to_owned())?;
        }
        "place" => {
            ArchivePageRef::try_new(ArchiveSubjectKind::Place, subject_id.to_owned())?;
        }
        "concept" => {
            ArchiveAtomSubject::try_new(ArchiveAtomSubjectKind::Concept, subject_id.to_owned())?;
        }
        _ => return Err(SemanticArchiveError::InvalidIdentity),
    }
    let granted_tick =
        i64::try_from(granted_tick).map_err(|_| SemanticArchiveError::InvalidVerifiedTick)?;
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
        Err(SemanticArchiveError::GrantConflict)
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
) -> Result<ArchiveKnowledge, SemanticArchiveError> {
    let rows = client
        .query(
            ARCHIVE_KNOWLEDGE_SQL,
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &i64::try_from(MAX_KNOWLEDGE_GRANTS + 1)
                    .map_err(|_| SemanticArchiveError::CollectionBound)?,
            ],
        )
        .map_err(|error| database("read Archive knowledge grants", &error))?;
    if rows.len() > MAX_KNOWLEDGE_GRANTS {
        return Err(SemanticArchiveError::CollectionBound);
    }
    let mut grants = Vec::with_capacity(rows.len());
    for row in rows {
        let kind = decode_subject_kind(&decode::<String>(&row, 0)?)?;
        let page_ref = ArchivePageRef::try_new(kind, decode(&row, 1)?)?;
        let grant_key: String = decode(&row, 2)?;
        let granted_tick = u64::try_from(decode::<i64>(&row, 3)?)
            .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
        let citation = ArchiveCitation::try_new(decode(&row, 4)?, decode(&row, 5)?)?;
        grants.push(ArchiveKnowledgeGrant::try_new(
            page_ref,
            grant_key,
            granted_tick,
            citation,
        )?);
    }
    ArchiveKnowledge::try_new(grants)
}

/// Mint the canonical atom set one known page asserts (ADR249 R1): the
/// subject atom, one atom per known signal, and one link atom per known link
/// target. Ungranted signals and unknown link targets mint nothing, matching
/// the renderer's known-only material.
pub(crate) fn mint_page_atoms(
    campaign_id: CampaignId,
    resolve_tick: u64,
    input: &ArchivePageInput,
    knowledge: &ArchiveKnowledge,
) -> Result<Vec<ArchiveAtom>, SemanticArchiveError> {
    let page_ref = input.subject.page_ref();
    let subject = ArchiveAtomSubject::from_page_ref(page_ref)?;
    let mut atoms = Vec::with_capacity(input.signals.len() + input.links.len() + 1);
    if let Some(grant) = knowledge.grant(page_ref, "subject") {
        atoms.push(ArchiveAtom::try_new(
            campaign_id,
            subject.clone(),
            "subject".to_owned(),
            "subject".to_owned(),
            ArchiveEvidenceClass::Observed,
            &ArchiveAtomValue::Text(input.subject.title().to_owned()),
            grant.citation.clone(),
            resolve_tick,
        )?);
    }
    for signal in &input.signals {
        if !knowledge.knows_field(page_ref, signal.grant_key()) {
            continue;
        }
        let evidence_class = signal_evidence_class(page_ref, signal, knowledge)?;
        atoms.push(ArchiveAtom::try_new(
            campaign_id,
            subject.clone(),
            signal.grant_key().to_owned(),
            signal.grant_key().to_owned(),
            evidence_class,
            &ArchiveAtomValue::Text(signal.value().to_owned()),
            signal.citation().clone(),
            resolve_tick,
        )?);
    }
    for link in &input.links {
        let Some(grant) = knowledge.grant(link.target(), "subject") else {
            continue;
        };
        atoms.push(ArchiveAtom::try_new(
            campaign_id,
            subject.clone(),
            "link".to_owned(),
            "subject".to_owned(),
            ArchiveEvidenceClass::Observed,
            &ArchiveAtomValue::Text(link.target().page_key()),
            grant.citation.clone(),
            resolve_tick,
        )?);
    }
    Ok(atoms)
}

fn signal_evidence_class(
    page_ref: &ArchivePageRef,
    signal: &ArchiveSignal,
    knowledge: &ArchiveKnowledge,
) -> Result<ArchiveEvidenceClass, SemanticArchiveError> {
    use crate::archive_foundation_grants::county_qcew_citation;
    use crate::michigan_economy::{michigan_economy, QCEW_ECONOMICS_FIELD_KEYS};

    if signal.grant_key() == "identity" {
        return Ok(ArchiveEvidenceClass::Observed);
    }
    let Some(index) = QCEW_ECONOMICS_FIELD_KEYS
        .iter()
        .position(|key| *key == signal.grant_key())
        .filter(|_| page_ref.kind() == ArchiveSubjectKind::County)
    else {
        return Ok(ArchiveEvidenceClass::Derived);
    };
    let citation = county_qcew_citation(page_ref.id());
    if signal.citation() != &citation
        || knowledge
            .grant(page_ref, signal.grant_key())
            .is_none_or(|grant| grant.citation != citation)
    {
        return Ok(ArchiveEvidenceClass::Derived);
    }
    let economy = michigan_economy().map_err(|_| SemanticArchiveError::ArtifactDigest)?;
    let Some(county) = economy
        .counties()
        .iter()
        .find(|county| county.county_geoid == page_ref.id())
    else {
        return Ok(ArchiveEvidenceClass::Derived);
    };
    let observed = [
        county.annual_avg_estabs_count,
        county.annual_avg_emplvl,
        county.total_annual_wages,
        county.annual_avg_wkly_wage,
    ][index];
    Ok(if signal.value() == observed.to_string() {
        ArchiveEvidenceClass::Observed
    } else {
        ArchiveEvidenceClass::Derived
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
    atoms: &[ArchiveAtom],
) -> Result<ArchiveAtomMint, SemanticArchiveError> {
    let mut minted = 0usize;
    for atom in atoms {
        let (text_col, f64_col, u64_col, bool_col): (
            Option<&str>,
            Option<f64>,
            Option<i64>,
            Option<bool>,
        ) = match atom.value() {
            ArchiveAtomValue::Text(text) => (Some(text), None, None, None),
            ArchiveAtomValue::F64(number) => (None, Some(*number), None, None),
            ArchiveAtomValue::U64(number) => {
                let exact =
                    i64::try_from(*number).map_err(|_| SemanticArchiveError::CollectionBound)?;
                (None, None, Some(exact), None)
            }
            ArchiveAtomValue::Bool(flag) => (None, None, None, Some(*flag)),
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
                        .map_err(|_| SemanticArchiveError::InvalidVerifiedTick)?,
                ],
            )
            .map_err(|error| database("insert semantic Archive atom", &error))?;
        if affected == 1 {
            minted += 1;
        }
    }
    Ok(ArchiveAtomMint::new(minted, atoms.len()))
}

pub(crate) fn decode_subject_kind(value: &str) -> Result<ArchiveSubjectKind, SemanticArchiveError> {
    match value {
        "county" => Ok(ArchiveSubjectKind::County),
        "place" => Ok(ArchiveSubjectKind::Place),
        _ => Err(SemanticArchiveError::StoredPageMismatch),
    }
}

fn decode_atom_subject_kind(value: &str) -> Result<ArchiveAtomSubjectKind, SemanticArchiveError> {
    match value {
        "county" => Ok(ArchiveAtomSubjectKind::County),
        "place" => Ok(ArchiveAtomSubjectKind::Place),
        "concept" => Ok(ArchiveAtomSubjectKind::Concept),
        _ => Err(SemanticArchiveError::StoredPageMismatch),
    }
}

fn decode_evidence_class(value: &str) -> Result<ArchiveEvidenceClass, SemanticArchiveError> {
    match value {
        "Observed" => Ok(ArchiveEvidenceClass::Observed),
        "Derived" => Ok(ArchiveEvidenceClass::Derived),
        "Calibrated" => Ok(ArchiveEvidenceClass::Calibrated),
        "Designed" => Ok(ArchiveEvidenceClass::Designed),
        _ => Err(SemanticArchiveError::StoredPageMismatch),
    }
}

/// Decode one stored atom row with read-time revalidation: every field is
/// revalidated and the canonical identity is recomputed against the stored
/// `atom_id`, so any stored drift refuses with `StoredPageMismatch`.
pub(crate) fn decode_stored_atom(row: &Row) -> Result<ArchiveAtom, SemanticArchiveError> {
    let campaign_id = CampaignId::from_uuid(decode(row, 0)?);
    let kind = decode_atom_subject_kind(&decode::<String>(row, 1)?)?;
    let subject = ArchiveAtomSubject::try_new(kind, decode(row, 2)?)?;
    let signal_key: String = decode(row, 3)?;
    let grant_key: String = decode(row, 4)?;
    let evidence_class = decode_evidence_class(&decode::<String>(row, 5)?)?;
    let value_kind: String = decode(row, 6)?;
    let value = match value_kind.as_str() {
        "text" => ArchiveAtomValue::Text(decode(row, 7)?),
        "f64" => {
            let number: f64 = decode(row, 8)?;
            if !number.is_finite() {
                return Err(SemanticArchiveError::StoredPageMismatch);
            }
            ArchiveAtomValue::F64(if number == 0.0 { 0.0 } else { number })
        }
        "u64" => {
            let number = u64::try_from(decode::<i64>(row, 9)?)
                .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
            ArchiveAtomValue::U64(number)
        }
        "bool" => ArchiveAtomValue::Bool(decode(row, 10)?),
        _ => return Err(SemanticArchiveError::StoredPageMismatch),
    };
    let citation = ArchiveCitation::try_new(decode(row, 11)?, decode(row, 12)?)?;
    let valid_tick = u64::try_from(decode::<i64>(row, 13)?)
        .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
    let stored_atom_id = decode_digest(row, 14)?;
    let atom = ArchiveAtom::try_new(
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
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    Ok(atom)
}

pub(crate) fn decode<T: FromSqlOwned>(row: &Row, index: usize) -> Result<T, SemanticArchiveError> {
    row.try_get(index)
        .map_err(|error| database("decode semantic Archive row", &error))
}

pub(crate) fn decode_digest(row: &Row, index: usize) -> Result<[u8; 32], SemanticArchiveError> {
    let bytes: Vec<u8> = decode(row, index)?;
    bytes
        .try_into()
        .map_err(|_| SemanticArchiveError::StoredPageMismatch)
}

pub(crate) fn database(operation: &'static str, error: &postgres::Error) -> SemanticArchiveError {
    SemanticArchiveError::Database {
        operation,
        diagnostic: PostgresDiagnostic::capture(error),
    }
}

#[derive(Serialize)]
struct TemplateSignal<'a> {
    label: &'a str,
    value: &'a str,
    citation: &'a ArchiveCitation,
}

impl<'a> From<&'a ArchiveSignal> for TemplateSignal<'a> {
    fn from(signal: &'a ArchiveSignal) -> Self {
        Self {
            label: &signal.label,
            value: &signal.value,
            citation: &signal.citation,
        }
    }
}

#[derive(Serialize)]
struct TemplateLink<'a> {
    page_key: String,
    known_label: Option<&'a str>,
}

/// Stable closed refusal taxonomy for semantic Archive inputs and rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticArchiveError {
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
    /// A worker attempted to pass an earlier pending receipt.
    ArchiveOrderViolation,
    /// The Archive wakeup shape or advisory unlock result was not exact.
    SchemaMismatch,
    /// The runtime schema is absent, unsupported, or altered.
    CurrentSchema(crate::CurrentSchemaError),
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
        diagnostic: PostgresDiagnostic,
    },
}

impl std::fmt::Display for SemanticArchiveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "semantic Archive refusal: {self:?}")
    }
}

impl std::error::Error for SemanticArchiveError {}

pub(crate) fn validate_text(value: &str) -> Result<(), SemanticArchiveError> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.as_bytes().contains(&0) {
        return Err(SemanticArchiveError::InvalidText);
    }
    Ok(())
}

pub(crate) fn validate_key(value: &str) -> Result<(), SemanticArchiveError> {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return Err(SemanticArchiveError::InvalidText);
    };
    if value.len() > MAX_ID_BYTES
        || !(first.is_ascii_lowercase() || first.is_ascii_digit())
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(SemanticArchiveError::InvalidText);
    }
    Ok(())
}

fn hash_page_input(hasher: &mut Sha256, input: &ArchivePageInput) {
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

fn hash_page_ref(hasher: &mut Sha256, page_ref: &ArchivePageRef) {
    hasher.update([match page_ref.kind {
        ArchiveSubjectKind::County => 1,
        ArchiveSubjectKind::Place => 2,
    }]);
    hash_bytes(hasher, page_ref.id.as_bytes());
}

fn hash_citation(hasher: &mut Sha256, citation: &ArchiveCitation) {
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

    fn public_qcew_page() -> (CampaignId, ArchivePageInput, ArchiveKnowledge) {
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(319));
        let county = crate::michigan_economy::michigan_economy()
            .unwrap()
            .counties()
            .iter()
            .find(|county| county.county_geoid == "26163")
            .unwrap();
        let fields = crate::CommittedTerritoryFields::try_from_qcew([
            Some(i64::try_from(county.annual_avg_estabs_count).unwrap()),
            Some(i64::try_from(county.annual_avg_emplvl).unwrap()),
            Some(i64::try_from(county.total_annual_wages).unwrap()),
            Some(i64::try_from(county.annual_avg_wkly_wage).unwrap()),
        ])
        .unwrap();
        let plan = crate::CountyPagePlan::try_new(
            county.county_geoid.clone(),
            "county-26163".to_owned(),
            "Wayne County".to_owned(),
            crate::county_committed_signals(&fields).unwrap(),
            Vec::new(),
        )
        .unwrap();
        let page = crate::county_page_input(&plan, 1, [1; 32]).unwrap();
        let grants = crate::foundation_grant_rows()
            .unwrap()
            .into_iter()
            .filter(|row| {
                row.subject().kind() == ArchiveAtomSubjectKind::County
                    && row.subject().id() == county.county_geoid
            })
            .map(|row| {
                ArchiveKnowledgeGrant::try_new(
                    page.subject().page_ref().clone(),
                    row.grant_key().to_owned(),
                    0,
                    row.citation().clone(),
                )
                .unwrap()
            })
            .collect();
        (campaign, page, ArchiveKnowledge::try_new(grants).unwrap())
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
            assert_eq!(atom.evidence_class(), ArchiveEvidenceClass::Observed);
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
                ArchiveEvidenceClass::Derived,
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
