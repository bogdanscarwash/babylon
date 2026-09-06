//! Explicit New/Open admission reuses the single durable material runtime.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use postgres::{Config, NoTls};

use super::{
    RuntimeSessionErrorCodeV3, RuntimeSessionTailV3, RuntimeSessionTargetV3, SessionBackend,
};
use crate::{
    material_runtime::{DurableMaterialRuntimeV3, MaterialRuntimeErrorV3},
    michigan_content::{admit_michigan_content_v1, MichiganContentPresetV1},
    michigan_economy::digest_hex,
    CampaignId, SemanticArchiveStoreV1,
};

pub(super) struct DurableBackend {
    config: Config,
    campaign: CampaignId,
    runtime: DurableMaterialRuntimeV3,
    tail: RuntimeSessionTailV3,
}
impl SessionBackend for DurableBackend {
    fn tail(&self) -> RuntimeSessionTailV3 {
        self.tail.clone()
    }
    fn advance(
        &mut self,
        expected: &RuntimeSessionTailV3,
    ) -> Result<RuntimeSessionTailV3, RuntimeSessionErrorCodeV3> {
        if expected != &self.tail || durable_tail(&self.config, self.campaign)? != self.tail {
            return Err(RuntimeSessionErrorCodeV3::StaleExpectedTail);
        }
        let tick = self
            .tail
            .resolve_tick
            .checked_add(1)
            .ok_or(RuntimeSessionErrorCodeV3::CommitRefused)?;
        let actions = OrderedPracticeActionBatchV1::empty(
            self.runtime
                .session()
                .graph_session()
                .session_identity()
                .clone(),
            tick,
        )
        .map_err(|_| RuntimeSessionErrorCodeV3::CommitRefused)?;
        let receipt = self
            .runtime
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .map_err(|error| match error {
                MaterialRuntimeErrorV3::Replay(
                    babylon_tick::material_replay::MaterialReplayErrorV3::Horizon,
                ) => RuntimeSessionErrorCodeV3::HorizonComplete,
                MaterialRuntimeErrorV3::DatabaseLockRefused(_) => {
                    RuntimeSessionErrorCodeV3::StorageBusy
                }
                MaterialRuntimeErrorV3::DatabaseStatementCanceled(_) => {
                    RuntimeSessionErrorCodeV3::StorageCanceled
                }
                _ => RuntimeSessionErrorCodeV3::CommitRefused,
            })?;
        self.tail = RuntimeSessionTailV3 {
            resolve_tick: receipt.resolve_tick(),
            tick_content_hash: Some(digest_hex(receipt.tick_content_hash().as_bytes())),
        };
        Ok(self.tail.clone())
    }
}

fn durable_tail(
    config: &Config,
    campaign: CampaignId,
) -> Result<RuntimeSessionTailV3, RuntimeSessionErrorCodeV3> {
    let bounded = crate::material_runtime::bounded_material_writer_config_v3(config)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let mut client = bounded
        .connect(NoTls)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let row = client.query_opt("SELECT resolve_tick, tick_content_hash FROM babylon_state.tick_commit WHERE campaign_id = $1 ORDER BY resolve_tick DESC LIMIT 1", &[campaign.as_uuid()]).map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    match row {
        None => Ok(RuntimeSessionTailV3 {
            resolve_tick: 0,
            tick_content_hash: None,
        }),
        Some(row) => {
            let tick: i64 = row
                .try_get(0)
                .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
            let hash: Vec<u8> = row
                .try_get(1)
                .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
            if tick <= 0 || hash.len() != 32 {
                return Err(RuntimeSessionErrorCodeV3::StorageRefused);
            }
            Ok(RuntimeSessionTailV3 {
                resolve_tick: u64::try_from(tick)
                    .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?,
                tick_content_hash: Some(digest_hex(&hash)),
            })
        }
    }
}

pub(super) fn open(
    config: &Config,
    target: &RuntimeSessionTargetV3,
) -> Result<(DurableBackend, String), RuntimeSessionErrorCodeV3> {
    let campaign = target.campaign()?;
    let bounded = crate::material_runtime::bounded_material_writer_config_v3(config)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    crate::material_runtime::install_material_runtime_schema_v3(config)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    SemanticArchiveStoreV1::new(config)
        .install_schema()
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    crate::install_reader_role_v1(config).map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    crate::install_observer_economy_schema_v1(config)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let preset = match target {
        RuntimeSessionTargetV3::New { preset, .. } => {
            MichiganContentPresetV1::new_campaign(preset.delivery())
        }
        RuntimeSessionTargetV3::Open { .. } => runtime_content(
            &mut bounded
                .connect(NoTls)
                .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?,
            campaign,
        )?,
    };
    let admitted = preset
        .admitted()
        .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
    let foundation_digest = digest_hex(&admitted.digest());
    let runtime = match target {
        RuntimeSessionTargetV3::Open { .. } => {
            DurableMaterialRuntimeV3::open(config, campaign, admitted.digest())
        }
        RuntimeSessionTargetV3::New { .. } => {
            let foundation = preset
                .create_foundation()
                .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
            DurableMaterialRuntimeV3::create_new(config, campaign, foundation)
        }
    }
    .map_err(|error| match error {
        MaterialRuntimeErrorV3::AlreadyExists => RuntimeSessionErrorCodeV3::CampaignAlreadyExists,
        MaterialRuntimeErrorV3::MissingCampaign => RuntimeSessionErrorCodeV3::CampaignAbsent,
        MaterialRuntimeErrorV3::LegacyCampaign | MaterialRuntimeErrorV3::FoundationMismatch => {
            RuntimeSessionErrorCodeV3::ScenarioMismatch
        }
        _ => RuntimeSessionErrorCodeV3::StorageRefused,
    })?;
    let tail = durable_tail(config, campaign)?;
    if runtime.session().completed_tick() != tail.resolve_tick {
        return Err(RuntimeSessionErrorCodeV3::StorageRefused);
    }
    Ok((
        DurableBackend {
            config: config.clone(),
            campaign,
            runtime,
            tail,
        },
        foundation_digest,
    ))
}

fn runtime_content(
    client: &mut impl postgres::GenericClient,
    campaign: CampaignId,
) -> Result<MichiganContentPresetV1, RuntimeSessionErrorCodeV3> {
    let row = client.query_opt("SELECT f.preset_id,f.horizon_ticks,f.content_sha256,f.foundation_sha256,g.foundation_sha256,pg_catalog.sha256(pg_catalog.convert_to(g.scenario_source,'UTF8')) FROM babylon_state.material_campaign_foundation_v2 f JOIN babylon_state.campaign_foundation g USING(campaign_id) WHERE campaign_id=$1::uuid", &[campaign.as_uuid()])
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let Some(row) = row else {
        return Err(RuntimeSessionErrorCodeV3::CampaignAbsent);
    };
    let id: String = row
        .try_get(0)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let horizon: i64 = row
        .try_get(1)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let content: Vec<u8> = row
        .try_get(2)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let foundation: Vec<u8> = row
        .try_get(3)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let graph: Vec<u8> = row
        .try_get(4)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let scenario: Vec<u8> = row
        .try_get(5)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let admitted = admit_michigan_content_v1(&id, horizon, &content, &foundation, 0)
        .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
    admitted
        .validate_graph(&graph, &scenario)
        .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
    Ok(admitted.preset())
}
