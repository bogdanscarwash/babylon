//! Separate full-observer material capability and exact historical projection.

use babylon_kernel::sha256_of;
use babylon_tick::{
    material_replay::IdentifiedMaterialTickV3,
    material_world::{
        decode_material_receipts_v3, nominal_material_world_hash_v2, MaterialWorldRegisterV2,
    },
};
use postgres::{Config, GenericClient, NoTls};

use crate::{
    material_runtime::install_material_runtime_schema_v3,
    michigan_content::{
        admit_michigan_content_v1, MichiganContentAdmissionV1, MichiganPhysicalProjectionV1,
    },
    michigan_economy::digest_hex,
    observer_reader::{ObserverEconomyErrorV1, ObserverVisibilityV1},
    production_projection::project_material_observation_v1,
    CampaignId, ProductionSnapshotV1,
};

const SCHEMA: &str = include_str!("../migrations/observer_material_v1.sql");
const VIEWS: [&str; 2] = [
    "v_material_campaign_identity_v1",
    "v_observer_material_state_v1",
];

pub(crate) struct MaterialObservationV1 {
    pub(crate) foundation_digest: String,
    pub(crate) production: Option<ProductionSnapshotV1>,
    pub(crate) nominal_world_hash: Option<String>,
}

struct MaterialObservationRow {
    row_campaign: uuid::Uuid,
    row_tick: i64,
    register_bytes: Vec<u8>,
    receipts: Option<Vec<u8>>,
    identity: Option<Vec<u8>>,
    content_hash: Option<Vec<u8>>,
    foundation_bytes: Option<Vec<u8>>,
}

fn decode_material_row(row: &postgres::Row) -> Result<MaterialObservationRow, postgres::Error> {
    Ok(MaterialObservationRow {
        row_campaign: row.try_get(0)?,
        row_tick: row.try_get(1)?,
        register_bytes: row.try_get(2)?,
        receipts: row.try_get(3)?,
        identity: row.try_get(4)?,
        content_hash: row.try_get(5)?,
        foundation_bytes: row.try_get(6)?,
    })
}

pub(crate) fn read_material_header(
    transaction: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
) -> Result<Option<&'static MichiganContentAdmissionV1>, ObserverEconomyErrorV1> {
    let header = transaction.query_opt("SELECT campaign_id, preset_id, horizon_ticks, content_sha256, foundation_sha256 FROM public.v_material_campaign_identity_v1 WHERE campaign_id=$1", &[campaign.as_uuid()]).map_err(|_| ObserverEconomyErrorV1::Database)?;
    let Some(header) = header else {
        return Ok(None);
    };
    let row_campaign: uuid::Uuid = header
        .try_get(0)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let preset_id: String = header
        .try_get(1)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let horizon: i64 = header
        .try_get(2)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let content: Vec<u8> = header
        .try_get(3)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let foundation_digest: Vec<u8> = header
        .try_get(4)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let expected =
        admit_michigan_content_v1(&preset_id, horizon, &content, &foundation_digest, tick)
            .map_err(|_| ObserverEconomyErrorV1::ScenarioMismatch)?;
    if &row_campaign != campaign.as_uuid() {
        return Err(ObserverEconomyErrorV1::ScenarioMismatch);
    }
    Ok(Some(expected))
}

/// Header reads are safe for preview. Complete material reads are never issued for preview.
pub(crate) fn material_observation(
    transaction: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
    visibility: ObserverVisibilityV1,
    expected: &MichiganContentAdmissionV1,
) -> Result<MaterialObservationV1, ObserverEconomyErrorV1> {
    if visibility == ObserverVisibilityV1::KnownPreview {
        return Ok(MaterialObservationV1 {
            foundation_digest: digest_hex(&expected.digest),
            production: None,
            nominal_world_hash: None,
        });
    }
    let tick_sql = i64::try_from(tick).map_err(|_| ObserverEconomyErrorV1::TickAbsent)?;
    let rows = transaction.query("SELECT campaign_id, resolve_tick, register_bytes, receipt_bytes, identity_bytes, tick_content_hash, foundation_bytes FROM public.v_observer_material_state_v1 WHERE campaign_id=$1 AND resolve_tick <= $2 ORDER BY resolve_tick LIMIT 18", &[campaign.as_uuid(), &tick_sql]).map_err(|_| ObserverEconomyErrorV1::Database)?;
    if u64::try_from(rows.len()).ok() != tick.checked_add(1) {
        return Err(ObserverEconomyErrorV1::TickAbsent);
    }
    let mut register = expected.register.clone();
    let mut opening = None;
    let mut history = Vec::new();
    let mut prior_world = None;
    for (index, row) in rows.into_iter().enumerate() {
        let MaterialObservationRow {
            row_campaign,
            row_tick,
            register_bytes,
            receipts,
            identity,
            content_hash,
            foundation_bytes,
        } = decode_material_row(&row).map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        if &row_campaign != campaign.as_uuid() || usize::try_from(row_tick).ok() != Some(index) {
            return Err(ObserverEconomyErrorV1::InvalidProjection);
        }
        let next = MaterialWorldRegisterV2::decode(&register_bytes)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        if usize::try_from(next.completed_tick()).ok() != Some(index) {
            return Err(ObserverEconomyErrorV1::InvalidProjection);
        }
        if index == 0 {
            if foundation_bytes.as_deref() != Some(expected.canonical_bytes.as_slice())
                || next != expected.register
                || receipts.is_some()
                || identity.is_some()
                || content_hash.is_some()
            {
                return Err(ObserverEconomyErrorV1::ScenarioMismatch);
            }
        } else {
            if foundation_bytes.is_some() {
                return Err(ObserverEconomyErrorV1::InvalidProjection);
            }
            let identity = IdentifiedMaterialTickV3::decode(
                &identity.ok_or(ObserverEconomyErrorV1::InvalidProjection)?,
            )
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
            let receipt_bytes = receipts.ok_or(ObserverEconomyErrorV1::InvalidProjection)?;
            if usize::try_from(identity.resolve_tick()).ok() != Some(index)
                || identity.foundation_digest() != expected.digest
                || content_hash.as_deref()
                    != Some(identity.tick_content_hash().as_bytes().as_slice())
                || sha256_of(&receipt_bytes) != identity.receipt_digest()
                || nominal_material_world_hash_v2(identity.graph_world_after(), &next)
                    != identity.result_world_hash()
                || nominal_material_world_hash_v2(identity.graph_world_before(), &register)
                    != identity.prior_world_hash()
                || prior_world.is_some_and(|prior| prior != identity.prior_world_hash())
            {
                return Err(ObserverEconomyErrorV1::InvalidProjection);
            }
            let receipt = decode_material_receipts_v3(&receipt_bytes)
                .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
            if receipt.resolve_tick != identity.resolve_tick() {
                return Err(ObserverEconomyErrorV1::InvalidProjection);
            }
            history.push((receipt, identity.receipt_digest()));
            prior_world = Some(identity.result_world_hash());
        }
        let previous = std::mem::replace(&mut register, next);
        if index > 0 {
            opening = Some(previous);
        }
    }
    let production = match expected.physical_projection {
        MichiganPhysicalProjectionV1::FiveProcessV1 => project_material_observation_v1(
            expected.preset.delivery(),
            &register,
            opening.as_ref(),
            &history,
        ),
    }
    .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    Ok(MaterialObservationV1 {
        foundation_digest: digest_hex(&expected.digest),
        production: Some(attribute_production(production, expected, visibility)?),
        nominal_world_hash: prior_world.map(|hash| digest_hex(&hash)),
    })
}

fn attribute_production(
    mut production: ProductionSnapshotV1,
    expected: &MichiganContentAdmissionV1,
    visibility: ObserverVisibilityV1,
) -> Result<ProductionSnapshotV1, ObserverEconomyErrorV1> {
    expected
        .preset
        .label()
        .clone_into(&mut production.scenario_label);
    crate::production_projection::context::attach_observed_context_v1(
        expected,
        visibility,
        &mut production,
    )
    .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    Ok(production)
}

/// Exact additive migration: the original economic schema identity remains stable.
pub(crate) fn install_observer_material_schema_v1(
    config: &Config,
) -> Result<(), ObserverEconomyErrorV1> {
    install_material_runtime_schema_v3(config).map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
    let mut client = config
        .connect(NoTls)
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let mut tx = client
        .transaction()
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    tx.query_one(
        "SELECT pg_catalog.pg_advisory_xact_lock($1)",
        &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
    )
    .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let installed: bool = tx
        .query_one(
            "SELECT pg_catalog.to_regclass('public.observer_material_schema_v1') IS NOT NULL",
            &[],
        )
        .map_err(|_| ObserverEconomyErrorV1::Database)?
        .get(0);
    let digest = digest_hex(&sha256_of(SCHEMA.as_bytes()));
    if installed {
        let marker = tx.query_one("SELECT migration_sha256, view_definitions FROM public.observer_material_schema_v1 WHERE singleton", &[]).map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
        let stored: String = marker
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
        let definitions: Vec<String> = marker
            .try_get(1)
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
        if stored != digest || definitions != view_definitions(&mut tx)? {
            return Err(ObserverEconomyErrorV1::SchemaDrift);
        }
    } else {
        for view in VIEWS {
            let name = format!("public.{view}");
            let exists: bool = tx
                .query_one("SELECT pg_catalog.to_regclass($1) IS NOT NULL", &[&name])
                .map_err(|_| ObserverEconomyErrorV1::Database)?
                .get(0);
            if exists {
                return Err(ObserverEconomyErrorV1::SchemaDrift);
            }
        }
        tx.batch_execute(SCHEMA)
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        tx.batch_execute("CREATE TABLE public.observer_material_schema_v1 (singleton boolean PRIMARY KEY CHECK(singleton), migration_sha256 text NOT NULL, view_definitions text[] NOT NULL); REVOKE ALL ON public.observer_material_schema_v1 FROM PUBLIC").map_err(|_| ObserverEconomyErrorV1::Database)?;
        let definitions = view_definitions(&mut tx)?;
        tx.execute(
            "INSERT INTO public.observer_material_schema_v1 VALUES (true,$1,$2)",
            &[&digest, &definitions],
        )
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    }
    tx.commit().map_err(|_| ObserverEconomyErrorV1::Database)
}

fn view_definitions(tx: &mut impl GenericClient) -> Result<Vec<String>, ObserverEconomyErrorV1> {
    VIEWS
        .iter()
        .map(|name| {
            let name = format!("public.{name}");
            tx.query_one(
                "SELECT pg_catalog.pg_get_viewdef($1::text::regclass, false)",
                &[&name],
            )
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)
        })
        .collect()
}
