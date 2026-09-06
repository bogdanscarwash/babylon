//! Complete typed known-only rendering witness. Markdown is never its authority.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::archive::{decode_subject_kind, validate_text, MAX_LINKS, MAX_PAGE_BYTES, MAX_SIGNALS};
use crate::{
    ArchiveCitationV1, ArchiveLinkV1, ArchivePageInputV1, ArchivePageRefV1, ArchiveSignalV1,
    ArchiveSubjectV1, SemanticArchiveErrorV1,
};

// JSON escapes can expand otherwise lawful control characters up to sixfold.
// Keep the original 1 MiB rendered-page bound; do not tighten its text domain.
const MAX_EMISSION_BYTES: usize = MAX_PAGE_BYTES * 8;

/// The original unknown label is deliberately absent from this type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveEmissionLinkV2 {
    target: ArchivePageRefV1,
    known_label: Option<String>,
}

impl ArchiveEmissionLinkV2 {
    pub(crate) fn try_new(
        target: ArchivePageRefV1,
        known_label: Option<String>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if let Some(label) = &known_label {
            validate_text(label)?;
        }
        Ok(Self {
            target,
            known_label,
        })
    }

    pub(crate) fn target(&self) -> &ArchivePageRefV1 {
        &self.target
    }

    pub(crate) fn known_label(&self) -> Option<&str> {
        self.known_label.as_deref()
    }
}

/// Every field affecting emitted prose, search, citations, or ordered navigation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveEmissionManifestV2 {
    subject_citation: ArchiveCitationV1,
    question: String,
    signals: Vec<ArchiveSignalV1>,
    links: Vec<ArchiveEmissionLinkV2>,
}

impl ArchiveEmissionManifestV2 {
    pub(crate) fn try_new(
        subject_citation: ArchiveCitationV1,
        question: String,
        signals: Vec<ArchiveSignalV1>,
        links: Vec<ArchiveEmissionLinkV2>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        validate_text(&question)?;
        if signals.len() > MAX_SIGNALS || links.len() > MAX_LINKS {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        if signals
            .iter()
            .map(ArchiveSignalV1::grant_key)
            .collect::<BTreeSet<_>>()
            .len()
            != signals.len()
            || links
                .iter()
                .map(ArchiveEmissionLinkV2::target)
                .collect::<BTreeSet<_>>()
                .len()
                != links.len()
        {
            return Err(SemanticArchiveErrorV1::DuplicateKey);
        }
        Ok(Self {
            subject_citation,
            question,
            signals,
            links,
        })
    }

    pub(crate) fn subject_citation(&self) -> &ArchiveCitationV1 {
        &self.subject_citation
    }

    pub(crate) fn question(&self) -> &str {
        &self.question
    }

    pub(crate) fn signals(&self) -> &[ArchiveSignalV1] {
        &self.signals
    }

    pub(crate) fn links(&self) -> &[ArchiveEmissionLinkV2] {
        &self.links
    }

    pub(crate) fn search_text(&self, subject: &ArchiveSubjectV1) -> String {
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

    pub(crate) fn citations(&self) -> Vec<ArchiveCitationV1> {
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
        subject: ArchiveSubjectV1,
        source_tick: u64,
        source_hash: [u8; 32],
    ) -> Result<ArchivePageInputV1, SemanticArchiveErrorV1> {
        let links = self
            .links
            .iter()
            .filter_map(|link| {
                link.known_label
                    .as_ref()
                    .map(|label| ArchiveLinkV1::try_new(link.target.clone(), label.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        ArchivePageInputV1::try_new(
            subject,
            source_tick,
            source_hash,
            self.question.clone(),
            self.signals.clone(),
            links,
        )
    }

    pub(super) fn encode(&self) -> Result<String, SemanticArchiveErrorV1> {
        let encoded = serde_json::to_string(&ManifestWire::from(self))
            .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
        if encoded.len() > MAX_EMISSION_BYTES {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        Ok(encoded)
    }

    pub(super) fn decode(encoded: &str) -> Result<Self, SemanticArchiveErrorV1> {
        if encoded.len() > MAX_EMISSION_BYTES || encoded.as_bytes().contains(&0) {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        let wire: ManifestWire = serde_json::from_str(encoded)
            .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
        let manifest = wire.checked()?;
        if manifest.encode()? != encoded {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        Ok(manifest)
    }

    pub(super) fn verify(
        &self,
        record: &super::record::RevisionRecord,
    ) -> Result<(), SemanticArchiveErrorV1> {
        use crate::{ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, FogSafeArchiveRendererV1};

        let subject = ArchiveSubjectV1::try_new(
            record.subject.kind(),
            record.subject.id().to_owned(),
            record.title.clone(),
        )?;
        let source_hash = record
            .source
            .tick_content_hash()
            .ok_or(SemanticArchiveErrorV1::InvalidVerifiedTick)?;
        let renderer = FogSafeArchiveRendererV1::new()?;
        let page = renderer.render_emission(&subject, record.source.tick(), &source_hash, self)?;
        let expected_provenance = serde_json::to_string(page.citations())
            .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
        if record.template_sha256 != renderer.template_sha256()
            || record.markdown != page.markdown()
            || record.search_text != page.search_text()
            || record.provenance_json != expected_provenance
            || record.content_sha256 != page.sha256()
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        let knowledge = ArchiveKnowledgeV1::try_new(
            record
                .grants
                .iter()
                .map(|grant| {
                    ArchiveKnowledgeGrantV1::try_new(
                        grant.subject.clone(),
                        grant.key.clone(),
                        grant.granted_tick,
                        grant.citation.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        if record.atoms.first().map(crate::ArchiveAtomV1::citation) != Some(self.subject_citation())
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        let input = self.atom_input(subject, record.source.tick(), source_hash)?;
        let expected_count = self
            .signals
            .len()
            .checked_add(input.links().len())
            .and_then(|count| count.checked_add(1))
            .ok_or(SemanticArchiveErrorV1::CollectionBound)?;
        let atoms = crate::archive::mint_page_atoms(
            record.source.campaign_id(),
            record.source.tick(),
            &input,
            &knowledge,
        )?;
        if atoms.len() != expected_count || atoms != record.atoms {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestWire {
    layout_version: u8,
    subject_citation: ArchiveCitationV1,
    question: String,
    signals: Vec<SignalWire>,
    links: Vec<LinkWire>,
}

impl From<&ArchiveEmissionManifestV2> for ManifestWire {
    fn from(manifest: &ArchiveEmissionManifestV2) -> Self {
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
    fn checked(self) -> Result<ArchiveEmissionManifestV2, SemanticArchiveErrorV1> {
        if self.layout_version != 2
            || self.signals.len() > MAX_SIGNALS
            || self.links.len() > MAX_LINKS
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        let signals = self
            .signals
            .into_iter()
            .map(|signal| {
                ArchiveSignalV1::try_new(
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
                ArchiveEmissionLinkV2::try_new(
                    ArchivePageRefV1::try_new(
                        decode_subject_kind(&link.target_kind)?,
                        link.target_id,
                    )?,
                    link.known_label,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        ArchiveEmissionManifestV2::try_new(self.subject_citation, self.question, signals, links)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SignalWire {
    grant_key: String,
    label: String,
    value: String,
    citation: ArchiveCitationV1,
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
