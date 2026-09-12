//! One closed, scoped protocol for the lifetime of the parent-owned pipes.

use serde::{Deserialize, Serialize};

use super::RuntimeSessionErrorCode;
use crate::{identity::CampaignId, michigan_material::MichiganDeliveryPreset};

pub const RUNTIME_SESSION_PROTOCOL_VERSION: u16 = 3;
pub const RUNTIME_SESSION_MAX_LINE_BYTES: usize = 4096;

/// A lifecycle incarnation, distinct even when the same campaign is reopened.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSessionScope {
    pub epoch: u64,
    pub campaign_id: Option<String>,
}

/// Exact acknowledged durable tail; foundation has no committed tick hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSessionTail {
    pub resolve_tick: u64,
    pub tick_content_hash: Option<String>,
}

/// Closed wire selection of the existing authored delivery presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSessionPreset {
    Standard,
    Delayed,
    #[serde(rename = "shared-freight-ample")]
    SharedFreightAmple,
    #[serde(rename = "shared-freight-constrained")]
    SharedFreightConstrained,
    #[serde(rename = "statewide-baseline")]
    StatewideBaseline,
    #[serde(rename = "statewide-freight-constraint")]
    StatewideFreightConstraint,
    #[serde(rename = "statewide-packaging-shortage")]
    StatewidePackagingShortage,
    #[serde(rename = "statewide-both")]
    StatewideBoth,
}
impl RuntimeSessionPreset {
    pub(super) const fn delivery(self) -> MichiganDeliveryPreset {
        match self {
            Self::Standard => MichiganDeliveryPreset::Standard,
            Self::Delayed => MichiganDeliveryPreset::Delayed,
            Self::SharedFreightAmple => MichiganDeliveryPreset::SharedFreightAmple,
            Self::SharedFreightConstrained => MichiganDeliveryPreset::SharedFreightConstrained,
            Self::StatewideBaseline => MichiganDeliveryPreset::StatewideBaseline,
            Self::StatewideFreightConstraint => MichiganDeliveryPreset::StatewideFreightConstraint,
            Self::StatewidePackagingShortage => MichiganDeliveryPreset::StatewidePackagingShortage,
            Self::StatewideBoth => MichiganDeliveryPreset::StatewideBoth,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSessionTarget {
    New {
        campaign_id: String,
        preset: RuntimeSessionPreset,
    },
    Open {
        campaign_id: String,
    },
}
impl RuntimeSessionTarget {
    pub(super) fn campaign(&self) -> Result<CampaignId, RuntimeSessionErrorCode> {
        let (Self::New { campaign_id, .. } | Self::Open { campaign_id }) = self;
        let id = uuid::Uuid::parse_str(campaign_id)
            .map_err(|_| RuntimeSessionErrorCode::InvalidRequest)?;
        if id.is_nil() || id.to_string() != *campaign_id {
            return Err(RuntimeSessionErrorCode::InvalidRequest);
        }
        Ok(CampaignId::from_uuid(id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSessionRequest {
    Switch {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScope,
        target: RuntimeSessionTarget,
    },
    Advance {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScope,
        expected_tail: RuntimeSessionTail,
    },
    RefreshArchive {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScope,
    },
    Stop {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScope,
    },
}
impl RuntimeSessionRequest {
    pub(super) const fn header(&self) -> (u16, u64, &RuntimeSessionScope) {
        match self {
            Self::Switch {
                protocol_version,
                request_id,
                scope,
                ..
            }
            | Self::Advance {
                protocol_version,
                request_id,
                scope,
                ..
            }
            | Self::RefreshArchive {
                protocol_version,
                request_id,
                scope,
            }
            | Self::Stop {
                protocol_version,
                request_id,
                scope,
            } => (*protocol_version, *request_id, scope),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSessionResponse {
    Hello {
        protocol_version: u16,
        scope: RuntimeSessionScope,
    },
    Switching {
        request_id: u64,
        previous_scope: RuntimeSessionScope,
        scope: RuntimeSessionScope,
    },
    Ready {
        request_id: u64,
        scope: RuntimeSessionScope,
        foundation_digest: String,
        tail: RuntimeSessionTail,
    },
    Committed {
        request_id: u64,
        scope: RuntimeSessionScope,
        tail: RuntimeSessionTail,
    },
    ArchiveProgress {
        request_id: Option<u64>,
        scope: RuntimeSessionScope,
        durable_tick: u64,
        verified_tick: u64,
    },
    Error {
        request_id: Option<u64>,
        scope: RuntimeSessionScope,
        code: RuntimeSessionErrorCode,
        tail: Option<RuntimeSessionTail>,
    },
    Stopped {
        request_id: u64,
        scope: RuntimeSessionScope,
    },
}
