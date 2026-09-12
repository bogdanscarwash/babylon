//! Complete typed known-only rendering witness. Markdown is never its authority.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::archive::{decode_subject_kind, validate_text, MAX_LINKS, MAX_PAGE_BYTES, MAX_SIGNALS};
use crate::{
    ArchiveCitation, ArchiveLink, ArchivePageInput, ArchivePageRef, ArchiveSignal, ArchiveSubject,
    SemanticArchiveError,
};

// JSON escapes can expand otherwise lawful control characters up to sixfold.
// Keep the original 1 MiB rendered-page bound; do not tighten its text domain.
const MAX_EMISSION_BYTES: usize = MAX_PAGE_BYTES * 8;

/// The original unknown label is deliberately absent from this type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveEmissionLink {
    target: ArchivePageRef,
    known_label: Option<String>,
}

impl ArchiveEmissionLink {
    pub(crate) fn try_new(
        target: ArchivePageRef,
        known_label: Option<String>,
    ) -> Result<Self, SemanticArchiveError> {
        if let Some(label) = &known_label {
            validate_text(label)?;
        }
        Ok(Self {
            target,
            known_label,
        })
    }

    pub(crate) fn target(&self) -> &ArchivePageRef {
        &self.target
    }

    pub(crate) fn known_label(&self) -> Option<&str> {
        self.known_label.as_deref()
    }
}

/// Every field affecting emitted prose, search, citations, or ordered navigation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveEmissionManifest {
    subject_citation: ArchiveCitation,
    question: String,
    signals: Vec<ArchiveSignal>,
    links: Vec<ArchiveEmissionLink>,
}

impl ArchiveEmissionManifest {
    pub(crate) fn try_new(
        subject_citation: ArchiveCitation,
        question: String,
        signals: Vec<ArchiveSignal>,
        links: Vec<ArchiveEmissionLink>,
    ) -> Result<Self, SemanticArchiveError> {
        validate_text(&question)?;
        if signals.len() > MAX_SIGNALS || links.len() > MAX_LINKS {
            return Err(SemanticArchiveError::CollectionBound);
        }
        if signals
            .iter()
            .map(ArchiveSignal::grant_key)
            .collect::<BTreeSet<_>>()
            .len()
            != signals.len()
            || links
                .iter()
                .map(ArchiveEmissionLink::target)
                .collect::<BTreeSet<_>>()
                .len()
                != links.len()
        {
            return Err(SemanticArchiveError::DuplicateKey);
        }
        Ok(Self {
            subject_citation,
            question,
            signals,
            links,
        })
    }

    pub(crate) fn subject_citation(&self) -> &ArchiveCitation {
        &self.subject_citation
    }

    pub(crate) fn question(&self) -> &str {
        &self.question
    }

    pub(crate) fn signals(&self) -> &[ArchiveSignal] {
        &self.signals
    }

    pub(crate) fn links(&self) -> &[ArchiveEmissionLink] {
        &self.links
    }

    pub(crate) fn search_text(&self, subject: &ArchiveSubject) -> String {
        let mut parts = vec![
            subject.page_ref().page_key(),
            subject.title().to_owned(),
            self.question.clone(),
        ];
        for signal in &self.signals {
            parts.push(signal.label().to_owned());
            parts.push(signal.value().to_owned());
        }
        for link in &self.links {
            if let Some(label) = &link.known_label {
                parts.push(link.target.page_key());
                parts.push(label.clone());
            }
        }
        parts.join(" ")
    }

    pub(crate) fn citations(&self) -> Vec<ArchiveCitation> {
        let mut citations = vec![self.subject_citation.clone()];
        for signal in &self.signals {
            if !citations.contains(signal.citation()) {
                citations.push(signal.citation().clone());
            }
        }
        citations
    }

    /// Unknown links mint no atom, so the existing atom minter receives only
    /// disclosed links. Rendering consumes the full typed manifest separately.
    pub(super) fn atom_input(
        &self,
        subject: ArchiveSubject,
        source_tick: u64,
        source_hash: [u8; 32],
    ) -> Result<ArchivePageInput, SemanticArchiveError> {
        let links = self
            .links
            .iter()
            .filter_map(|link| {
                link.known_label
                    .as_ref()
                    .map(|label| ArchiveLink::try_new(link.target.clone(), label.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        ArchivePageInput::try_new(
            subject,
            source_tick,
            source_hash,
            self.question.clone(),
            self.signals.clone(),
            links,
        )
    }

    pub(super) fn encode(&self) -> Result<String, SemanticArchiveError> {
        let encoded = serde_json::to_string(&ManifestWire::from(self))
            .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
        if encoded.len() > MAX_EMISSION_BYTES {
            return Err(SemanticArchiveError::CollectionBound);
        }
        Ok(encoded)
    }

    pub(super) fn decode(encoded: &str) -> Result<Self, SemanticArchiveError> {
        if encoded.len() > MAX_EMISSION_BYTES || encoded.as_bytes().contains(&0) {
            return Err(SemanticArchiveError::CollectionBound);
        }
        let wire: ManifestWire =
            serde_json::from_str(encoded).map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
        let manifest = wire.checked()?;
        if manifest.encode()? != encoded {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
        Ok(manifest)
    }

    pub(super) fn verify(
        &self,
        record: &super::record::RevisionRecord,
    ) -> Result<(), SemanticArchiveError> {
        use crate::{ArchiveKnowledge, ArchiveKnowledgeGrant, FogSafeArchiveRenderer};

        let subject = ArchiveSubject::try_new(
            record.subject.kind(),
            record.subject.id().to_owned(),
            record.title.clone(),
        )?;
        let source_hash = record
            .source
            .tick_content_hash()
            .ok_or(SemanticArchiveError::InvalidVerifiedTick)?;
        let renderer = FogSafeArchiveRenderer::new()?;
        let page = renderer.render_emission(&subject, record.source.tick(), &source_hash, self)?;
        let expected_provenance = serde_json::to_string(page.citations())
            .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
        if record.template_sha256 != renderer.template_sha256()
            || record.markdown != page.markdown()
            || record.search_text != page.search_text()
            || record.provenance_json != expected_provenance
            || record.content_sha256 != page.sha256()
        {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
        let knowledge = ArchiveKnowledge::try_new(
            record
                .grants
                .iter()
                .map(|grant| {
                    ArchiveKnowledgeGrant::try_new(
                        grant.subject.clone(),
                        grant.key.clone(),
                        grant.granted_tick,
                        grant.citation.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        if record.atoms.first().map(crate::ArchiveAtom::citation) != Some(self.subject_citation()) {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
        let input = self.atom_input(subject, record.source.tick(), source_hash)?;
        let expected_count = self
            .signals
            .len()
            .checked_add(input.links().len())
            .and_then(|count| count.checked_add(1))
            .ok_or(SemanticArchiveError::CollectionBound)?;
        let atoms = crate::archive::mint_page_atoms(
            record.source.campaign_id(),
            record.source.tick(),
            &input,
            &knowledge,
        )?;
        if atoms.len() != expected_count || atoms != record.atoms {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestWire {
    layout_version: u8,
    subject_citation: ArchiveCitation,
    question: String,
    signals: Vec<SignalWire>,
    links: Vec<LinkWire>,
}

impl From<&ArchiveEmissionManifest> for ManifestWire {
    fn from(manifest: &ArchiveEmissionManifest) -> Self {
        Self {
            layout_version: 2,
            subject_citation: manifest.subject_citation.clone(),
            question: manifest.question.clone(),
            signals: manifest
                .signals
                .iter()
                .map(|signal| SignalWire {
                    grant_key: signal.grant_key().to_owned(),
                    label: signal.label().to_owned(),
                    value: signal.value().to_owned(),
                    citation: signal.citation().clone(),
                })
                .collect(),
            links: manifest
                .links
                .iter()
                .map(|link| LinkWire {
                    target_kind: link.target.kind().as_str().to_owned(),
                    target_id: link.target.id().to_owned(),
                    known_label: link.known_label.clone(),
                })
                .collect(),
        }
    }
}

impl ManifestWire {
    fn checked(self) -> Result<ArchiveEmissionManifest, SemanticArchiveError> {
        if self.layout_version != 2
            || self.signals.len() > MAX_SIGNALS
            || self.links.len() > MAX_LINKS
        {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
        let signals = self
            .signals
            .into_iter()
            .map(|signal| {
                ArchiveSignal::try_new(
                    signal.grant_key,
                    signal.label,
                    signal.value,
                    signal.citation,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let links = self
            .links
            .into_iter()
            .map(|link| {
                ArchiveEmissionLink::try_new(
                    ArchivePageRef::try_new(
                        decode_subject_kind(&link.target_kind)?,
                        link.target_id,
                    )?,
                    link.known_label,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        ArchiveEmissionManifest::try_new(self.subject_citation, self.question, signals, links)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SignalWire {
    grant_key: String,
    label: String,
    value: String,
    citation: ArchiveCitation,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkWire {
    target_kind: String,
    target_id: String,
    known_label: Option<String>,
}

#[cfg(test)]
mod tests;
