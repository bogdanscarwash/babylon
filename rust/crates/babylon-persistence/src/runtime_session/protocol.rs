//! One closed, scoped protocol for the lifetime of the parent-owned pipes.

use serde::{Deserialize, Serialize};

use super::RuntimeSessionErrorCodeV3;
use crate::{michigan_material::MichiganDeliveryPresetV1, CampaignId};

pub const RUNTIME_SESSION_PROTOCOL_VERSION_V3: u16 = 3;
pub const RUNTIME_SESSION_MAX_LINE_BYTES_V3: usize = 4096;

/// A lifecycle incarnation, distinct even when the same campaign is reopened.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSessionScopeV3 {
    pub epoch: u64,
    pub campaign_id: Option<String>,
}

/// Exact acknowledged durable tail; foundation has no committed tick hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSessionTailV3 {
    pub resolve_tick: u64,
    pub tick_content_hash: Option<String>,
}

/// Closed wire selection of the existing authored delivery presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSessionPresetV3 {
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
impl RuntimeSessionPresetV3 {
    pub(super) const fn delivery(self) -> MichiganDeliveryPresetV1 {
        match self {
            Self::Standard => MichiganDeliveryPresetV1::Standard,
            Self::Delayed => MichiganDeliveryPresetV1::Delayed,
            Self::SharedFreightAmple => MichiganDeliveryPresetV1::SharedFreightAmple,
            Self::SharedFreightConstrained => MichiganDeliveryPresetV1::SharedFreightConstrained,
            Self::StatewideBaseline => MichiganDeliveryPresetV1::StatewideBaseline,
            Self::StatewideFreightConstraint => {
                MichiganDeliveryPresetV1::StatewideFreightConstraint
            }
            Self::StatewidePackagingShortage => {
                MichiganDeliveryPresetV1::StatewidePackagingShortage
            }
            Self::StatewideBoth => MichiganDeliveryPresetV1::StatewideBoth,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSessionTargetV3 {
    New {
        campaign_id: String,
        preset: RuntimeSessionPresetV3,
    },
    Open {
        campaign_id: String,
    },
}
impl RuntimeSessionTargetV3 {
    pub(super) fn campaign(&self) -> Result<CampaignId, RuntimeSessionErrorCodeV3> {
        let (Self::New { campaign_id, .. } | Self::Open { campaign_id }) = self;
        let id = uuid::Uuid::parse_str(campaign_id)
            .map_err(|_| RuntimeSessionErrorCodeV3::InvalidRequest)?;
        if id.is_nil() || id.to_string() != *campaign_id {
            return Err(RuntimeSessionErrorCodeV3::InvalidRequest);
        }
        Ok(CampaignId::from_uuid(id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSessionRequestV3 {
    Switch {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScopeV3,
        target: RuntimeSessionTargetV3,
    },
    Advance {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScopeV3,
        expected_tail: RuntimeSessionTailV3,
    },
    RefreshArchive {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScopeV3,
    },
    Stop {
        protocol_version: u16,
        request_id: u64,
        scope: RuntimeSessionScopeV3,
    },
}
impl RuntimeSessionRequestV3 {
    pub(super) const fn header(&self) -> (u16, u64, &RuntimeSessionScopeV3) {
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
pub enum RuntimeSessionResponseV3 {
    Hello {
        protocol_version: u16,
        scope: RuntimeSessionScopeV3,
    },
    Switching {
        request_id: u64,
        previous_scope: RuntimeSessionScopeV3,
        scope: RuntimeSessionScopeV3,
    },
    Ready {
        request_id: u64,
        scope: RuntimeSessionScopeV3,
        foundation_digest: String,
        tail: RuntimeSessionTailV3,
    },
    Committed {
        request_id: u64,
        scope: RuntimeSessionScopeV3,
        tail: RuntimeSessionTailV3,
    },
    ArchiveProgress {
        request_id: Option<u64>,
        scope: RuntimeSessionScopeV3,
        durable_tick: u64,
        verified_tick: u64,
        retention_ready: bool,
    },
    Error {
        request_id: Option<u64>,
        scope: RuntimeSessionScopeV3,
        code: RuntimeSessionErrorCodeV3,
        tail: Option<RuntimeSessionTailV3>,
    },
    Stopped {
        request_id: u64,
        scope: RuntimeSessionScopeV3,
    },
}
