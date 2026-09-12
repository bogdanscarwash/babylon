//! Explicit New/Open admission reuses the single durable material runtime.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_practice_contract::OrderedPracticeActionBatch;
use postgres::{Config, NoTls};

use super::{RuntimeSessionErrorCode, RuntimeSessionTail, RuntimeSessionTarget, SessionBackend};
use crate::{
    identity::CampaignId,
    material_runtime::{DurableMaterialRuntime, MaterialRuntimeError},
    michigan_content::{admit_michigan_content, MichiganContentPreset},
    michigan_economy::digest_hex,
};

pub(super) struct DurableBackend {
    config: Config,
    campaign: CampaignId,
    runtime: DurableMaterialRuntime,
    tail: RuntimeSessionTail,
}
impl SessionBackend for DurableBackend {
    fn tail(&self) -> RuntimeSessionTail {
        self.tail.clone()
    }
    fn advance(
        &mut self,
        expected: &RuntimeSessionTail,
    ) -> Result<RuntimeSessionTail, RuntimeSessionErrorCode> {
        if expected != &self.tail || durable_tail(&self.config, self.campaign)? != self.tail {
            return Err(RuntimeSessionErrorCode::StaleExpectedTail);
        }
        let tick = self
            .tail
            .resolve_tick
            .checked_add(1)
            .ok_or(RuntimeSessionErrorCode::CommitRefused)?;
        let actions = OrderedPracticeActionBatch::empty(
            self.runtime
                .session()
                .graph_session()
                .session_identity()
                .clone(),
            tick,
        )
        .map_err(|_| RuntimeSessionErrorCode::CommitRefused)?;
        let receipt = self
            .runtime
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .map_err(|error| match error {
                MaterialRuntimeError::Replay(
                    babylon_tick::material_replay::MaterialReplayError::Horizon,
                ) => RuntimeSessionErrorCode::HorizonComplete,
                MaterialRuntimeError::DatabaseLockRefused(_) => {
                    RuntimeSessionErrorCode::StorageBusy
                }
                MaterialRuntimeError::DatabaseStatementCanceled(_) => {
                    RuntimeSessionErrorCode::StorageCanceled
                }
                _ => RuntimeSessionErrorCode::CommitRefused,
            })?;
        self.tail = RuntimeSessionTail {
            resolve_tick: receipt.resolve_tick(),
            tick_content_hash: Some(digest_hex(receipt.tick_content_hash().as_bytes())),
        };
        Ok(self.tail.clone())
    }
}

fn durable_tail(
    config: &Config,
    campaign: CampaignId,
) -> Result<RuntimeSessionTail, RuntimeSessionErrorCode> {
    let bounded = crate::material_runtime::bounded_material_writer_config(config)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let mut client = bounded
        .connect(NoTls)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let row = client.query_opt("SELECT resolve_tick, tick_content_hash FROM babylon_state.tick_commit WHERE campaign_id = $1 ORDER BY resolve_tick DESC LIMIT 1", &[campaign.as_uuid()]).map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    match row {
        None => Ok(RuntimeSessionTail {
            resolve_tick: 0,
            tick_content_hash: None,
        }),
        Some(row) => {
            let tick: i64 = row
                .try_get(0)
                .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
            let hash: Vec<u8> = row
                .try_get(1)
                .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
            if tick <= 0 || hash.len() != 32 {
                return Err(RuntimeSessionErrorCode::StorageRefused);
            }
            Ok(RuntimeSessionTail {
                resolve_tick: u64::try_from(tick)
                    .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?,
                tick_content_hash: Some(digest_hex(&hash)),
            })
        }
    }
}

pub(super) fn open(
    config: &Config,
    target: &RuntimeSessionTarget,
    defines_path: &std::path::Path,
) -> Result<(DurableBackend, String), RuntimeSessionErrorCode> {
    let campaign = target.campaign()?;
    let requested_catalog = catalog_for_target(target, defines_path)?;
    let bounded = crate::material_runtime::bounded_material_writer_config(config)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    crate::runtime::verify_runtime_schema(config)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    crate::install_reader_role(config).map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    crate::observer_reader::provision_observer_role(config)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let (runtime, foundation_digest) = match target {
        RuntimeSessionTarget::Open { .. } => {
            let mut client = bounded
                .connect(NoTls)
                .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
            let admitted = runtime_content(&mut client, campaign)?;
            (
                DurableMaterialRuntime::open(config, campaign, admitted.digest()),
                digest_hex(&admitted.digest()),
            )
        }
        RuntimeSessionTarget::New { preset, .. } => {
            let catalog = requested_catalog
                .as_ref()
                .ok_or(RuntimeSessionErrorCode::DefinesInvalid)?;
            let foundation = MichiganContentPreset::new_campaign(preset.delivery())
                .create_foundation(catalog)
                .map_err(|_| RuntimeSessionErrorCode::ScenarioMismatch)?;
            let digest = digest_hex(&foundation.digest());
            (
                DurableMaterialRuntime::create_new(config, campaign, foundation),
                digest,
            )
        }
    };
    let runtime = runtime.map_err(|error| match error {
        MaterialRuntimeError::AlreadyExists => RuntimeSessionErrorCode::CampaignAlreadyExists,
        MaterialRuntimeError::MissingCampaign => RuntimeSessionErrorCode::CampaignAbsent,
        MaterialRuntimeError::LegacyCampaign | MaterialRuntimeError::FoundationMismatch => {
            RuntimeSessionErrorCode::ScenarioMismatch
        }
        _ => RuntimeSessionErrorCode::StorageRefused,
    })?;
    let tail = durable_tail(config, campaign)?;
    if runtime.session().completed_tick() != tail.resolve_tick {
        return Err(RuntimeSessionErrorCode::StorageRefused);
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
    target: &RuntimeSessionTarget,
    defines_path: &std::path::Path,
) -> Result<Option<crate::michigan_material::MichiganMaterialCatalog>, RuntimeSessionErrorCode> {
    let RuntimeSessionTarget::New { preset, .. } = target else {
        return Ok(None);
    };
    // Load for each New request. Open never touches the mutable source file.
    let catalog = crate::michigan_material::MichiganMaterialCatalog::load_for_preset(
        defines_path,
        preset.delivery(),
    )
    .map_err(|error| {
        eprintln!("{error}");
        match error {
            crate::MichiganDefinesError::Read(_) => RuntimeSessionErrorCode::DefinesMissing,
            crate::MichiganDefinesError::TooLarge => RuntimeSessionErrorCode::DefinesTooLarge,
            crate::MichiganDefinesError::Toml(_) | crate::MichiganDefinesError::Utf8(_) => {
                RuntimeSessionErrorCode::DefinesMalformed
            }
            _ => RuntimeSessionErrorCode::DefinesInvalid,
        }
    })?;
    Ok(Some(catalog))
}

fn runtime_content(
    client: &mut impl postgres::GenericClient,
    campaign: CampaignId,
) -> Result<crate::michigan_content::MichiganContentAdmission, RuntimeSessionErrorCode> {
    let row = client.query_opt("SELECT f.preset_id,f.horizon_ticks,f.content_sha256,f.foundation_sha256,g.foundation_sha256,pg_catalog.sha256(pg_catalog.convert_to(g.scenario_source,'UTF8')),f.foundation_bytes FROM babylon_state.material_campaign_foundation_v2 f JOIN babylon_state.campaign_foundation g USING(campaign_id) WHERE campaign_id=$1::uuid", &[campaign.as_uuid()])
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let Some(row) = row else {
        return Err(RuntimeSessionErrorCode::CampaignAbsent);
    };
    let id: String = row
        .try_get(0)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let horizon: i64 = row
        .try_get(1)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let content: Vec<u8> = row
        .try_get(2)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let foundation: Vec<u8> = row
        .try_get(3)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let graph: Vec<u8> = row
        .try_get(4)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let scenario: Vec<u8> = row
        .try_get(5)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let bytes: Vec<u8> = row
        .try_get(6)
        .map_err(|_| RuntimeSessionErrorCode::StorageRefused)?;
    let admitted = admit_michigan_content(&id, horizon, &content, &foundation, 0, &bytes)
        .map_err(|_| RuntimeSessionErrorCode::ScenarioMismatch)?;
    admitted
        .validate_graph(&graph, &scenario)
        .map_err(|_| RuntimeSessionErrorCode::ScenarioMismatch)?;
    Ok(admitted)
}

#[cfg(test)]
mod defines_tests {
    use super::*;
    #[test]
    fn statewide_new_requires_qualified_sources_before_storage_admission() {
        use super::super::RuntimeSessionPreset;
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
            RuntimeSessionPreset::StatewideBaseline,
            RuntimeSessionPreset::StatewideFreightConstraint,
            RuntimeSessionPreset::StatewidePackagingShortage,
            RuntimeSessionPreset::StatewideBoth,
        ] {
            let target = RuntimeSessionTarget::New {
                campaign_id: uuid::Uuid::from_u128(31).to_string(),
                preset,
            };
            assert!(matches!(
                catalog_for_target(&target, &path),
                Err(RuntimeSessionErrorCode::DefinesMissing)
            ));
            std::fs::write(&manifest, b"{}").unwrap();
            assert!(matches!(
                catalog_for_target(&target, &path),
                Err(RuntimeSessionErrorCode::DefinesInvalid)
            ));
            std::fs::remove_file(&manifest).unwrap();
        }
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&directory).unwrap();
        let open = RuntimeSessionTarget::Open {
            campaign_id: uuid::Uuid::from_u128(31).to_string(),
        };
        assert!(catalog_for_target(&open, &path).unwrap().is_none());
    }
    #[test]
    fn new_reloads_config_while_open_never_reads_it() {
        let path =
            std::env::temp_dir().join(format!("babylon-defines-{}.toml", std::process::id()));
        let new = RuntimeSessionTarget::New {
            campaign_id: uuid::Uuid::from_u128(17).to_string(),
            preset: super::super::RuntimeSessionPreset::Standard,
        };
        let open = RuntimeSessionTarget::Open {
            campaign_id: uuid::Uuid::from_u128(17).to_string(),
        };
        assert!(catalog_for_target(&open, &path).unwrap().is_none());
        assert!(matches!(
            catalog_for_target(&new, &path),
            Err(RuntimeSessionErrorCode::DefinesMissing)
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
            Err(RuntimeSessionErrorCode::DefinesMalformed)
        ));
        assert!(catalog_for_target(&open, &path).unwrap().is_none());
        std::fs::remove_file(path).unwrap();
    }
}
