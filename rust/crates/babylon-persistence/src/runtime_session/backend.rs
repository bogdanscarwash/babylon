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
    defines_path: &std::path::Path,
) -> Result<(DurableBackend, String), RuntimeSessionErrorCodeV3> {
    let campaign = target.campaign()?;
    let requested_catalog = catalog_for_target(target, defines_path)?;
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
    let (runtime, foundation_digest) = match target {
        RuntimeSessionTargetV3::Open { .. } => {
            let mut client = bounded
                .connect(NoTls)
                .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
            let admitted = runtime_content(&mut client, campaign)?;
            (
                DurableMaterialRuntimeV3::open(config, campaign, admitted.digest()),
                digest_hex(&admitted.digest()),
            )
        }
        RuntimeSessionTargetV3::New { preset, .. } => {
            let catalog = requested_catalog
                .as_ref()
                .ok_or(RuntimeSessionErrorCodeV3::DefinesInvalid)?;
            let foundation = MichiganContentPresetV1::new_campaign(preset.delivery())
                .create_foundation(catalog)
                .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
            let digest = digest_hex(&foundation.digest());
            (
                DurableMaterialRuntimeV3::create_new(config, campaign, foundation),
                digest,
            )
        }
    };
    let runtime = runtime.map_err(|error| match error {
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

fn catalog_for_target(
    target: &RuntimeSessionTargetV3,
    defines_path: &std::path::Path,
) -> Result<Option<crate::michigan_material::MichiganMaterialCatalogV1>, RuntimeSessionErrorCodeV3>
{
    let RuntimeSessionTargetV3::New { preset, .. } = target else {
        return Ok(None);
    };
    // Load for each New request. Open never touches the mutable source file.
    let catalog = crate::michigan_material::MichiganMaterialCatalogV1::load_for_preset(
        defines_path,
        preset.delivery(),
    )
    .map_err(|error| {
        eprintln!("{error}");
        match error {
            crate::MichiganDefinesErrorV1::Read(_) => RuntimeSessionErrorCodeV3::DefinesMissing,
            crate::MichiganDefinesErrorV1::TooLarge => RuntimeSessionErrorCodeV3::DefinesTooLarge,
            crate::MichiganDefinesErrorV1::Toml(_) | crate::MichiganDefinesErrorV1::Utf8(_) => {
                RuntimeSessionErrorCodeV3::DefinesMalformed
            }
            _ => RuntimeSessionErrorCodeV3::DefinesInvalid,
        }
    })?;
    Ok(Some(catalog))
}

fn runtime_content(
    client: &mut impl postgres::GenericClient,
    campaign: CampaignId,
) -> Result<crate::michigan_content::MichiganContentAdmissionV1, RuntimeSessionErrorCodeV3> {
    let row = client.query_opt("SELECT f.preset_id,f.horizon_ticks,f.content_sha256,f.foundation_sha256,g.foundation_sha256,pg_catalog.sha256(pg_catalog.convert_to(g.scenario_source,'UTF8')),f.foundation_bytes FROM babylon_state.material_campaign_foundation_v2 f JOIN babylon_state.campaign_foundation g USING(campaign_id) WHERE campaign_id=$1::uuid", &[campaign.as_uuid()])
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
    let bytes: Vec<u8> = row
        .try_get(6)
        .map_err(|_| RuntimeSessionErrorCodeV3::StorageRefused)?;
    let admitted = admit_michigan_content_v1(&id, horizon, &content, &foundation, 0, &bytes)
        .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
    admitted
        .validate_graph(&graph, &scenario)
        .map_err(|_| RuntimeSessionErrorCodeV3::ScenarioMismatch)?;
    Ok(admitted)
}

#[cfg(test)]
mod defines_tests {
    use super::*;
    #[test]
    fn statewide_new_requires_qualified_sources_before_storage_admission() {
        use super::super::RuntimeSessionPresetV3;
        let directory =
            std::env::temp_dir().join(format!("babylon-statewide-defines-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("defines.toml");
        let manifest = directory.join("statewide-sources.json");
        std::fs::write(
            &path,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../content/scenarios/michigan/defines.toml"
            )),
        )
        .unwrap();
        for preset in [
            RuntimeSessionPresetV3::StatewideBaseline,
            RuntimeSessionPresetV3::StatewideFreightConstraint,
            RuntimeSessionPresetV3::StatewidePackagingShortage,
            RuntimeSessionPresetV3::StatewideBoth,
        ] {
            let target = RuntimeSessionTargetV3::New {
                campaign_id: uuid::Uuid::from_u128(31).to_string(),
                preset,
            };
            assert!(matches!(
                catalog_for_target(&target, &path),
                Err(RuntimeSessionErrorCodeV3::DefinesMissing)
            ));
            std::fs::write(&manifest, b"{}").unwrap();
            assert!(matches!(
                catalog_for_target(&target, &path),
                Err(RuntimeSessionErrorCodeV3::DefinesInvalid)
            ));
            std::fs::remove_file(&manifest).unwrap();
        }
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&directory).unwrap();
        let open = RuntimeSessionTargetV3::Open {
            campaign_id: uuid::Uuid::from_u128(31).to_string(),
        };
        assert!(catalog_for_target(&open, &path).unwrap().is_none());
    }
    #[test]
    fn new_reloads_config_while_open_never_reads_it() {
        let path =
            std::env::temp_dir().join(format!("babylon-defines-{}.toml", std::process::id()));
        let new = RuntimeSessionTargetV3::New {
            campaign_id: uuid::Uuid::from_u128(17).to_string(),
            preset: super::super::RuntimeSessionPresetV3::Standard,
        };
        let open = RuntimeSessionTargetV3::Open {
            campaign_id: uuid::Uuid::from_u128(17).to_string(),
        };
        assert!(catalog_for_target(&open, &path).unwrap().is_none());
        assert!(matches!(
            catalog_for_target(&new, &path),
            Err(RuntimeSessionErrorCodeV3::DefinesMissing)
        ));
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/scenarios/michigan/defines.toml"
        ));
        std::fs::write(&path, source).unwrap();
        let first = catalog_for_target(&new, &path).unwrap().unwrap();
        std::fs::write(
            &path,
            source.replace(
                "WORK_HOURS_PER_PERSON_WEEK = 40",
                "WORK_HOURS_PER_PERSON_WEEK = 45",
            ),
        )
        .unwrap();
        let second = catalog_for_target(&new, &path).unwrap().unwrap();
        assert_ne!(first.defines_hash(), second.defines_hash());
        std::fs::write(&path, "malformed = [").unwrap();
        assert!(matches!(
            catalog_for_target(&new, &path),
            Err(RuntimeSessionErrorCodeV3::DefinesMalformed)
        ));
        assert!(catalog_for_target(&open, &path).unwrap().is_none());
        std::fs::remove_file(path).unwrap();
    }
}
