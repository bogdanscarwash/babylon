//! Separate read-only economic observer and per-signal granted preview capabilities.

use babylon_kernel::sha256_of;
use postgres::{Config, IsolationLevel, NoTls};
use serde::{Deserialize, Serialize};

use crate::{
    michigan_content::{
        admit_michigan_content_v1, MichiganContentAdmissionV1, MichiganContentPresetV1,
        MICHIGAN_CONTENT_PRESETS_V1,
    },
    michigan_economy::{digest_hex, michigan_economy_v1, MichiganCountyEconomyV1},
    validate_legacy_connection_target, CampaignId,
};

pub const OBSERVER_DSN_ENV_V1: &str = "BABYLON_OBSERVER_DSN";
pub const OBSERVER_ROLE_NAME_V1: &str = "babylon_observer";
pub const OBSERVER_ECONOMY_SCHEMA_V1_SQL: &str =
    include_str!("../migrations/observer_economy_v1.sql");
const VIEW_NAMES: [&str; 3] = [
    "v_observer_economy_foundation_v1",
    "v_observer_county_economy_v1",
    "v_known_county_economy_v1",
];
const SNAPSHOT_COLUMNS: &str = "campaign_id, resolve_tick, county_geoid, annual_avg_estabs_count, annual_avg_emplvl, total_annual_wages, annual_avg_wkly_wage, establishments_granted, employment_granted, annual_wages_granted, weekly_wage_granted";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverVisibilityV1 {
    FullObserver,
    KnownPreview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverCountyEconomyV1 {
    pub county_geoid: String,
    pub annual_avg_estabs_count: Option<u64>,
    pub annual_avg_emplvl: Option<u64>,
    pub total_annual_wages: Option<u64>,
    pub annual_avg_wkly_wage: Option<u64>,
}

/// Safe local campaign catalog; no material registers or ungranted signals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignSummaryV1 {
    pub id: String,
    pub preset: String,
    pub label: String,
    pub durable_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverEconomySnapshotV1 {
    pub campaign_id: String,
    pub resolve_tick: u64,
    pub foundation_digest: String,
    /// Combined graph and material world identity; distinct from committed evidence.
    pub nominal_world_hash: Option<String>,
    pub tick_content_hash: Option<String>,
    pub envelope_digest: Option<String>,
    pub visibility: ObserverVisibilityV1,
    pub counties: Vec<ObserverCountyEconomyV1>,
    pub production: Option<crate::ProductionSnapshotV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverEconomyErrorV1 {
    MissingDsn,
    InvalidDsn,
    ConnectionTarget,
    Database,
    Authority,
    SchemaDrift,
    CampaignAbsent,
    ScenarioMismatch,
    TickAbsent,
    InvalidProjection,
    Reference,
}
impl std::fmt::Display for ObserverEconomyErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "observer economics refused: {self:?}")
    }
}
impl std::error::Error for ObserverEconomyErrorV1 {}

/// A capability whose visibility is fixed when its separate credential is admitted.
#[derive(Clone)]
pub struct ObserverEconomyReaderV1 {
    config: Config,
    visibility: ObserverVisibilityV1,
}
impl ObserverEconomyReaderV1 {
    /// # Errors
    /// Refuses absent/malformed DSN or any target outside the loopback contract.
    pub fn from_observer_env() -> Result<Self, ObserverEconomyErrorV1> {
        Self::from_env(OBSERVER_DSN_ENV_V1, ObserverVisibilityV1::FullObserver)
    }
    /// # Errors
    /// Refuses absent/malformed DSN or any target outside the loopback contract.
    pub fn from_known_env() -> Result<Self, ObserverEconomyErrorV1> {
        Self::from_env(crate::READER_DSN_ENV_V1, ObserverVisibilityV1::KnownPreview)
    }
    fn from_env(
        name: &str,
        visibility: ObserverVisibilityV1,
    ) -> Result<Self, ObserverEconomyErrorV1> {
        let dsn = std::env::var(name).map_err(|_| ObserverEconomyErrorV1::MissingDsn)?;
        let config: Config = dsn
            .parse()
            .map_err(|_| ObserverEconomyErrorV1::InvalidDsn)?;
        Self::connect(&config, visibility)
    }
    /// Validate the target. Every read rechecks actual database authority.
    /// # Errors
    /// Refuses non-loopback or caller-controlled startup configuration.
    pub fn connect(
        config: &Config,
        visibility: ObserverVisibilityV1,
    ) -> Result<Self, ObserverEconomyErrorV1> {
        validate_legacy_connection_target(config)
            .map_err(|_| ObserverEconomyErrorV1::ConnectionTarget)?;
        Ok(Self {
            config: config.clone(),
            visibility,
        })
    }
    #[must_use]
    pub const fn visibility(&self) -> ObserverVisibilityV1 {
        self.visibility
    }

    /// Read at most 64 explicitly founded Michigan material campaigns.
    /// # Errors
    /// Refuses authority, malformed identities, unknown presets or invalid clocks.
    pub fn campaigns(&self) -> Result<Vec<CampaignSummaryV1>, ObserverEconomyErrorV1> {
        let mut config = self.config.clone();
        config
            .connect_timeout(crate::LEGACY_ADOPTER_CONNECT_TIMEOUT)
            .tcp_user_timeout(crate::LEGACY_ADOPTER_TCP_USER_TIMEOUT)
            .options(crate::LEGACY_ADOPTER_STARTUP_OPTIONS);
        let mut client = config
            .connect(NoTls)
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        confine_authority(&mut client, self.visibility)?;
        let mut transaction = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        let bindings = CampaignCatalogBindings::admitted()?;
        let rows = transaction
            .query(
                CAMPAIGN_CATALOG_SQL,
                &[
                    &bindings.presets,
                    &bindings.horizons,
                    &bindings.content,
                    &bindings.foundations,
                    &bindings.graphs,
                    &bindings.scenarios,
                ],
            )
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        let result = rows
            .iter()
            .map(campaign_summary)
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .commit()
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        Ok(result)
    }

    /// Project exactly the requested committed tick, or the true foundation at zero.
    /// No query substitutes the latest tick. The client cannot request arbitrary SQL.
    /// # Errors
    /// Refuses writer credentials, other scenario foundations, missing ticks, corrupt
    /// integers, missing observer fields or rows, and mismatched campaign identities.
    pub fn snapshot(
        &self,
        campaign: CampaignId,
        expected_tick: u64,
    ) -> Result<ObserverEconomySnapshotV1, ObserverEconomyErrorV1> {
        let tick = i64::try_from(expected_tick).map_err(|_| ObserverEconomyErrorV1::TickAbsent)?;
        let mut config = self.config.clone();
        config
            .connect_timeout(crate::LEGACY_ADOPTER_CONNECT_TIMEOUT)
            .tcp_user_timeout(crate::LEGACY_ADOPTER_TCP_USER_TIMEOUT)
            .options(crate::LEGACY_ADOPTER_STARTUP_OPTIONS);
        let mut client = config
            .connect(NoTls)
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        confine_authority(&mut client, self.visibility)?;
        let mut transaction = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        let foundation = transaction.query_opt("SELECT campaign_id, foundation_sha256, scenario_sha256 FROM public.v_observer_economy_foundation_v1 WHERE campaign_id = $1", &[campaign.as_uuid()]).map_err(|_| ObserverEconomyErrorV1::Database)?.ok_or(ObserverEconomyErrorV1::CampaignAbsent)?;
        let found_campaign: uuid::Uuid = foundation
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        if &found_campaign != campaign.as_uuid() {
            return Err(ObserverEconomyErrorV1::InvalidProjection);
        }
        let foundation_hash: Vec<u8> = foundation
            .try_get(1)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        let scenario_hash: Vec<u8> = foundation
            .try_get(2)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        let economy = michigan_economy_v1().map_err(|_| ObserverEconomyErrorV1::Reference)?;
        let admission = crate::observer_material::read_material_header(
            &mut transaction,
            campaign,
            expected_tick,
        )?;
        validate_observer_graph(admission, &foundation_hash, &scenario_hash)?;
        let (tick_content_hash, envelope_digest) = if expected_tick == 0 {
            (None, None)
        } else {
            let marker = transaction.query_opt("SELECT tick_content_hash, envelope_digest FROM public.v_committed_tick_status_v1 WHERE campaign_id = $1 AND resolve_tick = $2", &[campaign.as_uuid(), &tick]).map_err(|_| ObserverEconomyErrorV1::Database)?.ok_or(ObserverEconomyErrorV1::TickAbsent)?;
            let content: Vec<u8> = marker
                .try_get(0)
                .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
            let envelope: Vec<u8> = marker
                .try_get(1)
                .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
            if content.len() != 32 || envelope.len() != 32 {
                return Err(ObserverEconomyErrorV1::InvalidProjection);
            }
            (Some(digest_hex(&content)), Some(digest_hex(&envelope)))
        };
        let counties = read_committed_counties(
            &mut transaction,
            campaign,
            expected_tick,
            self.visibility,
            economy.counties(),
        )?;
        let material = if let Some(admission) = admission {
            crate::observer_material::material_observation(
                &mut transaction,
                campaign,
                expected_tick,
                self.visibility,
                admission,
            )?
        } else {
            // Explicitly admitted baseline-only V1 conformance campaigns have no
            // material family. V2 graphs without a material header fail above.
            crate::observer_material::MaterialObservationV1 {
                foundation_digest: digest_hex(&foundation_hash),
                production: None,
                nominal_world_hash: None,
            }
        };
        transaction
            .commit()
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        Ok(ObserverEconomySnapshotV1 {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: expected_tick,
            foundation_digest: material.foundation_digest,
            nominal_world_hash: material.nominal_world_hash,
            tick_content_hash,
            envelope_digest,
            visibility: self.visibility,
            counties,
            production: material.production,
        })
    }
}

// The admission JOIN precedes the result limit. Unrelated or corrupt rows
// cannot consume a catalog slot or force construction from database metadata.
const CAMPAIGN_CATALOG_SQL: &str = "WITH admitted AS (
 SELECT * FROM unnest($1::text[], $2::bigint[], $3::bytea[], $4::bytea[], $5::bytea[], $6::bytea[])
 AS entry(preset_id,horizon_ticks,content_sha256,foundation_sha256,graph_sha256,scenario_sha256)
)
SELECT header.campaign_id, header.preset_id, header.horizon_ticks, header.content_sha256,
 header.foundation_sha256, COALESCE(max(marker.resolve_tick),0)::bigint AS durable_tick
FROM public.v_material_campaign_identity_v1 AS header
JOIN admitted USING(preset_id,horizon_ticks,content_sha256,foundation_sha256)
JOIN public.v_observer_economy_foundation_v1 AS graph ON graph.campaign_id=header.campaign_id
 AND graph.foundation_sha256=admitted.graph_sha256 AND graph.scenario_sha256=admitted.scenario_sha256
LEFT JOIN public.v_committed_tick_status_v1 AS marker ON marker.campaign_id=header.campaign_id
WHERE header.campaign_id <> '00000000-0000-0000-0000-000000000000'::uuid
GROUP BY header.campaign_id,header.preset_id,header.horizon_ticks,header.content_sha256,header.foundation_sha256
HAVING COALESCE(max(marker.resolve_tick),0) BETWEEN 0 AND header.horizon_ticks
ORDER BY header.campaign_id LIMIT 64";

#[derive(Default)]
struct CampaignCatalogBindings {
    presets: Vec<String>,
    horizons: Vec<i64>,
    content: Vec<Vec<u8>>,
    foundations: Vec<Vec<u8>>,
    graphs: Vec<Vec<u8>>,
    scenarios: Vec<Vec<u8>>,
}
impl CampaignCatalogBindings {
    fn admitted() -> Result<Self, ObserverEconomyErrorV1> {
        let mut bindings = Self::default();
        for preset in MICHIGAN_CONTENT_PRESETS_V1 {
            let entry = preset
                .admitted()
                .map_err(|_| ObserverEconomyErrorV1::Reference)?;
            bindings.presets.push(preset.id().to_owned());
            bindings.horizons.push(
                i64::try_from(entry.horizon_ticks)
                    .map_err(|_| ObserverEconomyErrorV1::Reference)?,
            );
            bindings.content.push(entry.content_digest.to_vec());
            bindings.foundations.push(entry.digest.to_vec());
            bindings.graphs.push(entry.graph_digest.to_vec());
            bindings.scenarios.push(entry.scenario_digest.to_vec());
        }
        Ok(bindings)
    }
}

fn campaign_summary(row: &postgres::Row) -> Result<CampaignSummaryV1, ObserverEconomyErrorV1> {
    let campaign: uuid::Uuid = row
        .try_get(0)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let preset_id: String = row
        .try_get(1)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let horizon: i64 = row
        .try_get(2)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let content: Vec<u8> = row
        .try_get(3)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let foundation: Vec<u8> = row
        .try_get(4)
        .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let tick = u64::try_from(
        row.try_get::<_, i64>(5)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?,
    )
    .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
    let entry = admit_michigan_content_v1(&preset_id, horizon, &content, &foundation, tick)
        .map_err(|_| ObserverEconomyErrorV1::ScenarioMismatch)?;
    if campaign.is_nil() {
        return Err(ObserverEconomyErrorV1::InvalidProjection);
    }
    Ok(CampaignSummaryV1 {
        id: campaign.to_string(),
        preset: preset_id,
        label: entry.preset.label().to_owned(),
        durable_tick: tick,
    })
}

fn validate_observer_graph(
    material: Option<&MichiganContentAdmissionV1>,
    graph: &[u8],
    scenario: &[u8],
) -> Result<(), ObserverEconomyErrorV1> {
    let expected = match material {
        Some(admitted) => admitted,
        None => MichiganContentPresetV1::BaselineStandardV1
            .admitted()
            .map_err(|_| ObserverEconomyErrorV1::Reference)?,
    };
    expected
        .validate_graph(graph, scenario)
        .map_err(|_| ObserverEconomyErrorV1::ScenarioMismatch)
}

/// Read the complete county family through the admitted role's fixed view.
/// All rows remain in the caller's one read-only repeatable-read transaction.
fn read_committed_counties(
    transaction: &mut postgres::Transaction<'_>,
    campaign: CampaignId,
    expected_tick: u64,
    visibility: ObserverVisibilityV1,
    baselines: &[MichiganCountyEconomyV1],
) -> Result<Vec<ObserverCountyEconomyV1>, ObserverEconomyErrorV1> {
    let tick = i64::try_from(expected_tick).map_err(|_| ObserverEconomyErrorV1::TickAbsent)?;
    let view = match visibility {
        ObserverVisibilityV1::FullObserver => "public.v_observer_county_economy_v1",
        ObserverVisibilityV1::KnownPreview => "public.v_known_county_economy_v1",
    };
    let query = format!("SELECT {SNAPSHOT_COLUMNS} FROM {view} WHERE campaign_id = $1 AND resolve_tick = $2 ORDER BY county_geoid LIMIT 84");
    let rows = transaction
        .query(&query, &[campaign.as_uuid(), &tick])
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    if rows.len() != 83 {
        return Err(ObserverEconomyErrorV1::InvalidProjection);
    }
    let mut counties = Vec::with_capacity(83);
    for (row, baseline) in rows.iter().zip(baselines) {
        let row_campaign: uuid::Uuid = row
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        let row_tick: i64 = row
            .try_get(1)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        let geoid: String = row
            .try_get(2)
            .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        if &row_campaign != campaign.as_uuid() || row_tick != tick || geoid != baseline.county_geoid
        {
            return Err(ObserverEconomyErrorV1::InvalidProjection);
        }
        let mut values = [None; 4];
        let mut grants = [false; 4];
        for index in 0..4 {
            values[index] = row
                .try_get::<_, Option<i64>>(index + 3)
                .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
            grants[index] = row
                .try_get(index + 7)
                .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?;
        }
        counties.push(project_county(
            baseline,
            expected_tick,
            visibility,
            values,
            grants,
        )?);
    }
    Ok(counties)
}

fn project_county(
    baseline: &MichiganCountyEconomyV1,
    tick: u64,
    visibility: ObserverVisibilityV1,
    stored: [Option<i64>; 4],
    grants: [bool; 4],
) -> Result<ObserverCountyEconomyV1, ObserverEconomyErrorV1> {
    let baseline_values = [
        baseline.annual_avg_estabs_count,
        baseline.annual_avg_emplvl,
        baseline.total_annual_wages,
        baseline.annual_avg_wkly_wage,
    ];
    let mut values = [None; 4];
    for index in 0..4 {
        if visibility == ObserverVisibilityV1::FullObserver && !grants[index] {
            return Err(ObserverEconomyErrorV1::InvalidProjection);
        }
        if !grants[index] {
            if stored[index].is_some() {
                return Err(ObserverEconomyErrorV1::InvalidProjection);
            }
            continue;
        }
        values[index] = Some(if tick == 0 {
            if stored[index].is_some() {
                return Err(ObserverEconomyErrorV1::InvalidProjection);
            }
            baseline_values[index]
        } else {
            u64::try_from(stored[index].ok_or(ObserverEconomyErrorV1::InvalidProjection)?)
                .map_err(|_| ObserverEconomyErrorV1::InvalidProjection)?
        });
    }
    Ok(ObserverCountyEconomyV1 {
        county_geoid: baseline.county_geoid.clone(),
        annual_avg_estabs_count: values[0],
        annual_avg_emplvl: values[1],
        total_annual_wages: values[2],
        annual_avg_wkly_wage: values[3],
    })
}

const AUTHORITY_SQL: &str = "SELECT role.rolsuper, role.rolcreatedb, role.rolcreaterole, role.rolreplication, role.rolbypassrls, pg_catalog.pg_has_role(current_user, $1, 'MEMBER'), pg_catalog.pg_has_role(current_user, 'babylon_observer', 'MEMBER') FROM pg_catalog.pg_roles role WHERE role.rolname = current_user";
const HELD_SQL: &str = "WITH RECURSIVE role_closure(oid) AS (SELECT 0::oid UNION SELECT oid FROM pg_catalog.pg_roles WHERE rolname = current_user UNION SELECT membership.roleid FROM pg_catalog.pg_auth_members membership JOIN role_closure ON role_closure.oid = membership.member), restricted AS (SELECT relation.*, namespace.nspname FROM pg_catalog.pg_class relation JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace WHERE relation.relkind IN ('r','p','v','m','f') AND (namespace.nspname IN ('babylon_state','babylon_meta') OR (namespace.nspname = 'public' AND relation.relname IN ('v_committed_tick_status_v1','v_archive_page_known_v1','v_archive_atom_visible','v_county_card_atoms','v_archive_subject_atoms','v_archive_verification_v1','v_observer_economy_foundation_v1','v_observer_county_economy_v1','v_known_county_economy_v1','v_material_campaign_identity_v1','v_observer_material_state_v1','v_archive_revision_known_v2','v_archive_revision_atom_v2','v_archive_revision_grant_v2','v_archive_retention_v2','v_archive_subject_grant_v2','v_archive_revision_index_v2','v_archive_tick_knowledge_v2','v_archive_revision_scope_v2')))) SELECT DISTINCT restricted.nspname || '.' || restricted.relname AS relation_name, acl.privilege_type, acl.is_grantable FROM restricted CROSS JOIN LATERAL pg_catalog.aclexplode(restricted.relacl) acl JOIN role_closure ON role_closure.oid = acl.grantee UNION SELECT restricted.nspname || '.' || restricted.relname, 'OWNERSHIP', false FROM restricted JOIN role_closure ON role_closure.oid = restricted.relowner UNION SELECT restricted.nspname || '.' || restricted.relname, acl.privilege_type, acl.is_grantable FROM restricted JOIN pg_catalog.pg_attribute attribute ON attribute.attrelid = restricted.oid AND attribute.attnum > 0 AND NOT attribute.attisdropped CROSS JOIN LATERAL pg_catalog.aclexplode(attribute.attacl) acl JOIN role_closure ON role_closure.oid = acl.grantee";
fn confine_authority(
    client: &mut postgres::Client,
    visibility: ObserverVisibilityV1,
) -> Result<(), ObserverEconomyErrorV1> {
    let role = match visibility {
        ObserverVisibilityV1::FullObserver => "babylon_observer",
        ObserverVisibilityV1::KnownPreview => "babylon_reader",
    };
    let flags = client
        .query_one(AUTHORITY_SQL, &[&role])
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    for index in 0..5 {
        if flags
            .try_get::<_, bool>(index)
            .map_err(|_| ObserverEconomyErrorV1::Authority)?
        {
            return Err(ObserverEconomyErrorV1::Authority);
        }
    }
    if !flags
        .try_get::<_, bool>(5)
        .map_err(|_| ObserverEconomyErrorV1::Authority)?
        || (visibility == ObserverVisibilityV1::KnownPreview
            && flags
                .try_get::<_, bool>(6)
                .map_err(|_| ObserverEconomyErrorV1::Authority)?)
    {
        return Err(ObserverEconomyErrorV1::Authority);
    }
    let rows = client
        .query(HELD_SQL, &[])
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let mut held = std::collections::BTreeSet::new();
    for row in rows {
        let relation: String = row
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::Authority)?;
        let privilege: String = row
            .try_get(1)
            .map_err(|_| ObserverEconomyErrorV1::Authority)?;
        let grantable: bool = row
            .try_get(2)
            .map_err(|_| ObserverEconomyErrorV1::Authority)?;
        let allowed = match visibility {
            ObserverVisibilityV1::FullObserver => matches!(
                relation.as_str(),
                "public.v_observer_economy_foundation_v1"
                    | "public.v_observer_county_economy_v1"
                    | "public.v_material_campaign_identity_v1"
                    | "public.v_observer_material_state_v1"
                    | "public.v_committed_tick_status_v1"
            ),
            ObserverVisibilityV1::KnownPreview => matches!(
                relation.as_str(),
                "public.v_observer_economy_foundation_v1"
                    | "public.v_known_county_economy_v1"
                    | "public.v_material_campaign_identity_v1"
                    | "public.v_committed_tick_status_v1"
                    | "public.v_archive_page_known_v1"
                    | "public.v_archive_atom_visible"
                    | "public.v_county_card_atoms"
                    | "public.v_archive_subject_atoms"
                    | "public.v_archive_verification_v1"
                    | "public.v_archive_revision_known_v2"
                    | "public.v_archive_revision_atom_v2"
                    | "public.v_archive_revision_grant_v2"
                    | "public.v_archive_retention_v2"
                    | "public.v_archive_subject_grant_v2"
                    | "public.v_archive_revision_index_v2"
                    | "public.v_archive_tick_knowledge_v2"
                    | "public.v_archive_revision_scope_v2"
            ),
        };
        if !allowed || privilege != "SELECT" || grantable {
            return Err(ObserverEconomyErrorV1::Authority);
        }
        held.insert(relation);
    }
    let economy_view = match visibility {
        ObserverVisibilityV1::FullObserver => "public.v_observer_county_economy_v1",
        ObserverVisibilityV1::KnownPreview => "public.v_known_county_economy_v1",
    };
    if [
        "public.v_observer_economy_foundation_v1",
        "public.v_committed_tick_status_v1",
        "public.v_material_campaign_identity_v1",
        economy_view,
    ]
    .iter()
    .any(|view| !held.contains(*view))
    {
        return Err(ObserverEconomyErrorV1::Authority);
    }
    Ok(())
}

/// Install exact additive observer/preview views and group grants. No login secrets.
/// The marker binds the migration bytes and original PostgreSQL-rendered view
/// definitions; subsequent starts refuse drift rather than replacing views.
/// # Errors
/// Refuses role attributes, partial installation, changed definitions or database failure.
pub fn install_observer_economy_schema_v1(config: &Config) -> Result<(), ObserverEconomyErrorV1> {
    validate_legacy_connection_target(config)
        .map_err(|_| ObserverEconomyErrorV1::ConnectionTarget)?;
    crate::install_territory_county_map_schema_v1(config)
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let mut client = config
        .connect(NoTls)
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let mut transaction = client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .start()
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    transaction
        .query_one(
            "SELECT pg_catalog.pg_advisory_xact_lock($1)",
            &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
        )
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let installed: bool = transaction
        .query_one(
            "SELECT pg_catalog.to_regclass('public.observer_economy_schema_v1') IS NOT NULL",
            &[],
        )
        .map_err(|_| ObserverEconomyErrorV1::Database)?
        .get(0);
    let role = transaction.query_opt("SELECT rolsuper, rolcreatedb, rolcreaterole, rolcanlogin, rolreplication, rolbypassrls FROM pg_catalog.pg_roles WHERE rolname = 'babylon_observer'", &[]).map_err(|_| ObserverEconomyErrorV1::Database)?;
    if let Some(role) = role {
        for index in 0..6 {
            if role
                .try_get::<_, bool>(index)
                .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?
            {
                return Err(ObserverEconomyErrorV1::SchemaDrift);
            }
        }
    } else if installed {
        return Err(ObserverEconomyErrorV1::SchemaDrift);
    } else {
        transaction.batch_execute("CREATE ROLE babylon_observer NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS").map_err(|_| ObserverEconomyErrorV1::Database)?;
    }
    let migration_sha = digest_hex(&sha256_of(OBSERVER_ECONOMY_SCHEMA_V1_SQL.as_bytes()));
    if installed {
        let marker = transaction.query_one("SELECT migration_sha256, view_definitions FROM public.observer_economy_schema_v1 WHERE singleton", &[]).map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
        let stored_sha: String = marker
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
        let stored_definitions: Vec<String> = marker
            .try_get(1)
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
        if stored_sha != migration_sha || stored_definitions != view_definitions(&mut transaction)?
        {
            return Err(ObserverEconomyErrorV1::SchemaDrift);
        }
    } else {
        for name in VIEW_NAMES {
            let full = format!("public.{name}");
            let exists: bool = transaction
                .query_one("SELECT pg_catalog.to_regclass($1) IS NOT NULL", &[&full])
                .map_err(|_| ObserverEconomyErrorV1::Database)?
                .get(0);
            if exists {
                return Err(ObserverEconomyErrorV1::SchemaDrift);
            }
        }
        transaction
            .batch_execute(OBSERVER_ECONOMY_SCHEMA_V1_SQL)
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
        transaction.batch_execute("CREATE TABLE public.observer_economy_schema_v1 (singleton boolean PRIMARY KEY CHECK (singleton), migration_sha256 text NOT NULL, view_definitions text[] NOT NULL); REVOKE ALL ON public.observer_economy_schema_v1 FROM PUBLIC").map_err(|_| ObserverEconomyErrorV1::Database)?;
        let definitions = view_definitions(&mut transaction)?;
        transaction
            .execute(
                "INSERT INTO public.observer_economy_schema_v1 VALUES (true, $1, $2)",
                &[&migration_sha, &definitions],
            )
            .map_err(|_| ObserverEconomyErrorV1::Database)?;
    }
    transaction
        .commit()
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    crate::observer_material::install_observer_material_schema_v1(config)
}
fn view_definitions(
    client: &mut impl postgres::GenericClient,
) -> Result<Vec<String>, ObserverEconomyErrorV1> {
    VIEW_NAMES
        .iter()
        .map(|name| {
            let full = format!("public.{name}");
            client
                .query_one(
                    "SELECT pg_catalog.pg_get_viewdef($1::text::regclass, false)",
                    &[&full],
                )
                .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?
                .try_get(0)
                .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_headers_bind_the_matching_graph_and_baseline_only_is_explicit_v1() {
        for preset in MICHIGAN_CONTENT_PRESETS_V1 {
            let entry = preset.admitted().unwrap();
            assert!(validate_observer_graph(
                Some(entry),
                &entry.graph_digest,
                &entry.scenario_digest
            )
            .is_ok());
            let baseline_only =
                validate_observer_graph(None, &entry.graph_digest, &entry.scenario_digest);
            assert_eq!(
                baseline_only.is_ok(),
                matches!(
                    preset,
                    MichiganContentPresetV1::BaselineStandardV1
                        | MichiganContentPresetV1::BaselineDelayedV1
                )
            );
            for other in MICHIGAN_CONTENT_PRESETS_V1 {
                let other = other.admitted().unwrap();
                assert_eq!(
                    validate_observer_graph(
                        Some(entry),
                        &other.graph_digest,
                        &other.scenario_digest
                    )
                    .is_ok(),
                    entry.graph_digest == other.graph_digest
                );
            }
        }
    }
    #[test]
    fn catalog_query_bindings_keep_each_complete_admission_tuple_aligned() {
        let bindings = CampaignCatalogBindings::admitted().unwrap();
        assert_eq!(
            bindings.presets,
            [
                "michigan-material-standard-v1",
                "michigan-material-delayed-v1",
                "michigan-material-standard-v2",
                "michigan-material-delayed-v2",
                "michigan-material-standard-v3",
                "michigan-material-delayed-v3",
            ]
        );
        assert_eq!(bindings.horizons, [16; 6]);
        assert_eq!(bindings.content.len(), 6);
        assert_eq!(bindings.foundations.len(), 6);
        assert_eq!(bindings.graphs.len(), 6);
        assert_eq!(bindings.scenarios.len(), 6);
        for index in 0..6 {
            let entry = admit_michigan_content_v1(
                &bindings.presets[index],
                bindings.horizons[index],
                &bindings.content[index],
                &bindings.foundations[index],
                0,
            )
            .unwrap();
            entry
                .validate_graph(&bindings.graphs[index], &bindings.scenarios[index])
                .unwrap();
        }
    }

    #[test]
    fn foundation_grants_mask_individual_fields_before_values_exist() {
        let baseline = &michigan_economy_v1().unwrap().counties()[0];
        let row = project_county(
            baseline,
            0,
            ObserverVisibilityV1::KnownPreview,
            [None; 4],
            [true, false, true, false],
        )
        .unwrap();
        assert_eq!(row.annual_avg_estabs_count, Some(214));
        assert_eq!(row.annual_avg_emplvl, None);
        assert_eq!(row.total_annual_wages, Some(62_042_985));
        assert_eq!(row.annual_avg_wkly_wage, None);
    }
    #[test]
    fn committed_values_never_fallback_to_baseline_or_turn_missing_into_zero() {
        let baseline = &michigan_economy_v1().unwrap().counties()[0];
        assert_eq!(
            project_county(
                baseline,
                1,
                ObserverVisibilityV1::FullObserver,
                [Some(1), None, Some(3), Some(4)],
                [true; 4]
            ),
            Err(ObserverEconomyErrorV1::InvalidProjection)
        );
        assert_eq!(
            project_county(
                baseline,
                1,
                ObserverVisibilityV1::FullObserver,
                [Some(-1), Some(2), Some(3), Some(4)],
                [true; 4]
            ),
            Err(ObserverEconomyErrorV1::InvalidProjection)
        );
        let row = project_county(
            baseline,
            1,
            ObserverVisibilityV1::KnownPreview,
            [Some(1), None, Some(3), None],
            [true, false, true, false],
        )
        .unwrap();
        assert_eq!(row.annual_avg_estabs_count, Some(1));
        assert_eq!(row.annual_avg_emplvl, None);
    }
    #[test]
    fn leaked_ungiven_value_is_refused_even_if_ui_would_hide_it() {
        let baseline = &michigan_economy_v1().unwrap().counties()[0];
        assert_eq!(
            project_county(
                baseline,
                1,
                ObserverVisibilityV1::KnownPreview,
                [Some(1); 4],
                [true, false, true, true]
            ),
            Err(ObserverEconomyErrorV1::InvalidProjection)
        );
    }
}
