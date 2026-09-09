//! Checked immutable publication payload. Adoption never rerenders this record.

use std::collections::BTreeSet;

use sha2::{Digest as _, Sha256};

use super::emission::ArchiveEmissionManifestV2;
use super::{ArchivePublicationOriginV2, ArchiveReadScopeV2};
use crate::archive::{validate_text, MAX_LINKS, MAX_PAGE_BYTES, MAX_SIGNALS};
use crate::{
    ArchiveAtomSubjectV1, ArchiveAtomV1, ArchiveCitationV1, ArchivePageRefV1,
    SemanticArchiveErrorV1,
};

pub(super) const REVISION_DOMAIN: &[u8] = b"babylon.archive-page-revision.v2\0";

/// Exact immutable grants needed by the retained rendered bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GrantDependency {
    pub subject: ArchivePageRefV1,
    pub key: String,
    pub granted_tick: u64,
    pub citation: ArchiveCitationV1,
}

/// Values remain private to the checked storage boundary, not a second renderer.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct RevisionRecord {
    pub source: ArchiveReadScopeV2,
    pub subject: ArchivePageRefV1,
    pub effective_tick: u64,
    pub origin: ArchivePublicationOriginV2,
    pub title: String,
    pub template_sha256: [u8; 32],
    pub content_sha256: [u8; 32],
    pub markdown: String,
    pub search_text: String,
    pub provenance_json: String,
    pub atoms: Vec<ArchiveAtomV1>,
    pub grants: Vec<GrantDependency>,
    pub emission: Option<ArchiveEmissionManifestV2>,
}

impl RevisionRecord {
    pub fn validate(&self) -> Result<(), SemanticArchiveErrorV1> {
        validate_text(&self.title)?;
        if self.source.tick() == 0
            || self.effective_tick < self.source.tick()
            || self.effective_tick > i64::MAX as u64
            || (self.origin == ArchivePublicationOriginV2::Materialized
                && self.effective_tick != self.source.tick())
        {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
        }
        for text in [&self.markdown, &self.search_text, &self.provenance_json] {
            if text.len() > MAX_PAGE_BYTES || text.as_bytes().contains(&0) {
                return Err(SemanticArchiveErrorV1::CollectionBound);
            }
        }
        let actual: [u8; 32] = Sha256::digest(self.markdown.as_bytes()).into();
        if actual != self.content_sha256 {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        self.citations()?;
        self.validate_header()?;
        self.validate_atoms()?;
        self.validate_grants()?;
        match &self.emission {
            Some(emission) => emission.verify(self),
            None if self.origin == ArchivePublicationOriginV2::AdoptedHead => Ok(()),
            None => Err(SemanticArchiveErrorV1::StoredPageMismatch),
        }
    }

    fn validate_header(&self) -> Result<(), SemanticArchiveErrorV1> {
        let source = self
            .source
            .tick_content_hash()
            .ok_or(SemanticArchiveErrorV1::InvalidVerifiedTick)?;
        let prefix = format!("---\nschema: babylon.archive-page.v1\nsubject: {}/{}\nverified_tick: {}\ntick_content_hash: {}\n---\n# {}\n",
            self.subject.kind().as_str(), self.subject.id(), self.source.tick(),
            crate::archive::hex_digest(&source), self.title);
        if !self.markdown.starts_with(&prefix) {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        Ok(())
    }

    fn validate_grants(&self) -> Result<(), SemanticArchiveErrorV1> {
        if self.grants.is_empty() || self.grants.len() > 1 + MAX_SIGNALS + MAX_LINKS {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        let mut previous = None;
        for grant in &self.grants {
            crate::archive::validate_key(&grant.key)?;
            if grant.granted_tick > self.source.tick() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            let key = (grant.subject.kind(), grant.subject.id(), grant.key.as_str());
            if previous.is_some_and(|value| value >= key) {
                return Err(SemanticArchiveErrorV1::DuplicateGrant);
            }
            previous = Some(key);
        }
        for (position, atom) in self.atoms.iter().enumerate() {
            let subject = if is_link(atom) {
                let crate::ArchiveAtomValueV1::Text(target) = atom.value() else {
                    return Err(SemanticArchiveErrorV1::StoredPageMismatch);
                };
                parse_page_key(target)?
            } else {
                self.subject.clone()
            };
            let matching = self
                .grants
                .iter()
                .find(|grant| grant.subject == subject && grant.key == atom.grant_key());
            if matching.is_none() {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            if (position == 0 || is_link(atom))
                && matching.is_some_and(|grant| &grant.citation != atom.citation())
            {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
        }
        Ok(())
    }

    fn validate_atoms(&self) -> Result<(), SemanticArchiveErrorV1> {
        if self.atoms.is_empty() || self.atoms.len() > 1 + MAX_SIGNALS + MAX_LINKS {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        let subject = ArchiveAtomSubjectV1::from_page_ref(&self.subject)?;
        let mut ids = BTreeSet::new();
        for atom in &self.atoms {
            if *atom.campaign_id() != self.source.campaign_id()
                || atom.subject() != &subject
                || atom.valid_tick() != self.source.tick()
                || !ids.insert(atom.atom_id())
            {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
        }
        let first = &self.atoms[0];
        if first.signal_key() != "subject"
            || first.grant_key() != "subject"
            || first.value() != &crate::ArchiveAtomValueV1::Text(self.title.clone())
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        Ok(())
    }

    pub fn citations(&self) -> Result<Vec<ArchiveCitationV1>, SemanticArchiveErrorV1> {
        serde_json::from_str(&self.provenance_json)
            .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)
    }

    pub fn digest(&self) -> Result<[u8; 32], SemanticArchiveErrorV1> {
        self.validate()?;
        let mut hash = Sha256::new();
        hash.update(REVISION_DOMAIN);
        hash.update(self.source.campaign_id().as_uuid().as_bytes());
        hash_text(&mut hash, self.subject.kind().as_str());
        hash_text(&mut hash, self.subject.id());
        hash.update(self.effective_tick.to_be_bytes());
        hash.update(self.origin.tag().to_be_bytes());
        hash.update(self.source.tick().to_be_bytes());
        hash.update(
            self.source
                .tick_content_hash()
                .ok_or(SemanticArchiveErrorV1::InvalidVerifiedTick)?,
        );
        hash.update(self.template_sha256);
        hash.update(self.content_sha256);
        for text in [
            &self.title,
            &self.markdown,
            &self.search_text,
            &self.provenance_json,
        ] {
            hash_text(&mut hash, text);
        }
        hash.update(
            u64::try_from(self.atoms.len())
                .map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
                .to_be_bytes(),
        );
        for atom in &self.atoms {
            hash.update(atom.atom_id());
        }
        hash.update(
            u64::try_from(self.grants.len())
                .map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
                .to_be_bytes(),
        );
        for grant in &self.grants {
            hash_text(&mut hash, grant.subject.kind().as_str());
            hash_text(&mut hash, grant.subject.id());
            hash_text(&mut hash, &grant.key);
            hash.update(grant.granted_tick.to_be_bytes());
            hash_text(&mut hash, grant.citation.source_id());
            hash_text(&mut hash, grant.citation.locator());
        }
        hash.update([u8::from(self.emission.is_some())]);
        if let Some(emission) = &self.emission {
            hash_text(&mut hash, &emission.encode()?);
        }
        Ok(hash.finalize().into())
    }
}

pub(super) fn is_link(atom: &ArchiveAtomV1) -> bool {
    atom.signal_key() == "link" && atom.grant_key() == "subject"
}

pub(super) fn parse_page_key(key: &str) -> Result<ArchivePageRefV1, SemanticArchiveErrorV1> {
    let (kind, id) = key
        .split_once('/')
        .ok_or(SemanticArchiveErrorV1::InvalidIdentity)?;
    let kind = crate::archive::decode_subject_kind(kind)?;
    ArchivePageRefV1::try_new(kind, id.to_owned())
}

fn hash_text(hash: &mut Sha256, text: &str) {
    hash.update(
        u64::try_from(text.len())
            .expect("bounded text fits u64")
            .to_be_bytes(),
    );
    hash.update(text.as_bytes());
}

#[cfg(test)]
mod tests;
