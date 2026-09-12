//! Immutable checkpoint admission shared by runtime and full-observer reads.

use super::{CampaignFoundation, MaterialRuntimeError};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::content_digest::ContentDigest;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionId};
use babylon_kernel::tick_content_hash::RefDigest;
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::replay_session::ReplayTickSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MaterialComponentIdentity {
    sections: [Vec<u8>; 6],
    session_id: ReplaySessionId,
}

impl MaterialComponentIdentity {
    pub(crate) fn from_foundation(foundation: &CampaignFoundation) -> Self {
        Self::from_parts(
            foundation.resolver_manifest_bytes(),
            foundation.prepared_environment_bytes(),
            foundation.replay_session_identity(),
            foundation.rng_seed(),
            foundation.content_digest(),
            foundation.reference_digest(),
        )
    }

    pub(super) fn from_session(session: &ReplayTickSession<HypergraphStore>) -> Self {
        Self::from_parts(
            session.resolver_manifest_bytes(),
            session.prepared_environment_bytes(),
            session.session_identity(),
            session.rng_seed(),
            session.content_digest(),
            session.reference_digest(),
        )
    }

    fn from_parts(
        resolver: &[u8],
        environment: &[u8],
        session: &ReplaySessionId,
        seed: ReplaySeed,
        content: &ContentDigest,
        reference: RefDigest,
    ) -> Self {
        let mut content_bytes = [0_u8; 64];
        content_bytes[..32].copy_from_slice(&content.defines_hash);
        content_bytes[32..].copy_from_slice(&content.rules_hash);
        Self {
            sections: [
                resolver.to_vec(),
                environment.to_vec(),
                session.as_bytes().to_vec(),
                seed.to_be_bytes().to_vec(),
                content_bytes.to_vec(),
                reference.as_bytes().to_vec(),
            ],
            session_id: session.clone(),
        }
    }

    pub(super) fn validate_sections(
        &self,
        sections: &[Vec<u8>],
    ) -> Result<(), MaterialRuntimeError> {
        if sections.len() != 9 || sections[2..8] != self.sections {
            return Err(MaterialRuntimeError::InvalidCheckpoint);
        }
        Ok(())
    }

    pub(super) fn validate_actions(
        &self,
        tick: u64,
        layout: i16,
        digest: &[u8],
        bytes: &[u8],
    ) -> Result<(), MaterialRuntimeError> {
        let expected = OrderedPracticeActionBatch::empty(self.session_id.clone(), tick)
            .map_err(|_| MaterialRuntimeError::InvalidCheckpoint)?;
        if layout != 1
            || digest != expected.digest().as_bytes()
            || bytes != expected.canonical_bytes()
        {
            return Err(MaterialRuntimeError::InvalidCheckpoint);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
