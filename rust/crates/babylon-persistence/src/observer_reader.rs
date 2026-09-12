//! Separate read-only economic observer and per-signal granted preview capabilities.

use babylon_kernel::content_digest::sha256_of;
use postgres::{Config, IsolationLevel, NoTls};
use serde::{Deserialize, Serialize};

use crate::{
    identity::CampaignId,
    michigan_content::{
        validate_michigan_header, MichiganContentAdmission, MICHIGAN_CONTENT_PRESETS,
    },
    michigan_economy::{digest_hex, michigan_economy, MichiganCountyEconomy},
    postgres_catalog::validate_connection_target,
};

pub const OBSERVER_DSN_ENV: &str = "BABYLON_OBSERVER_DSN";
pub const OBSERVER_ROLE_NAME: &str = "babylon_observer";
const SNAPSHOT_COLUMNS: &str = "campaign_id, resolve_tick, county_geoid, annual_avg_estabs_count, annual_avg_emplvl, total_annual_wages, annual_avg_wkly_wage, establishments_granted, employment_granted, annual_wages_granted, weekly_wage_granted";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverVisibility {
    FullObserver,
    KnownPreview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverCountyEconomy {
    pub county_geoid: String,
    pub annual_avg_estabs_count: Option<u64>,
    pub annual_avg_emplvl: Option<u64>,
    pub total_annual_wages: Option<u64>,
    pub annual_avg_wkly_wage: Option<u64>,
}

/// Safe local campaign catalog; no material registers or ungranted signals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignSummary {
    pub id: String,
    pub preset: String,
    pub label: String,
    pub durable_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverEconomySnapshot {
    pub campaign_id: String,
    pub resolve_tick: u64,
    pub foundation_digest: String,
    /// Combined graph and material world identity; distinct from committed evidence.
    pub nominal_world_hash: Option<String>,
    pub tick_content_hash: Option<String>,
    pub envelope_digest: Option<String>,
    pub visibility: ObserverVisibility,
    pub counties: Vec<ObserverCountyEconomy>,
    pub production: Option<crate::production_observation::ProductionSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverEconomyError {
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
impl std::fmt::Display for ObserverEconomyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "observer economics refused: {self:?}")
    }
}
impl std::error::Error for ObserverEconomyError {}

/// A capability whose visibility is fixed when its separate credential is admitted.
#[derive(Clone)]
pub struct ObserverEconomyReader {
    config: Config,
    visibility: ObserverVisibility,
}
impl ObserverEconomyReader {
    /// # Errors
    /// Refuses absent/malformed DSN or any target outside the loopback contract.
    pub fn from_observer_env() -> Result<Self, ObserverEconomyError> {
        Self::from_env(OBSERVER_DSN_ENV, ObserverVisibility::FullObserver)
    }
    /// # Errors
    /// Refuses absent/malformed DSN or any target outside the loopback contract.
    pub fn from_known_env() -> Result<Self, ObserverEconomyError> {
        Self::from_env(crate::READER_DSN_ENV, ObserverVisibility::KnownPreview)
    }
    fn from_env(name: &str, visibility: ObserverVisibility) -> Result<Self, ObserverEconomyError> {
        let dsn = std::env::var(name).map_err(|_| ObserverEconomyError::MissingDsn)?;
        let config: Config = dsn.parse().map_err(|_| ObserverEconomyError::InvalidDsn)?;
        Self::connect(&config, visibility)
    }
    /// Validate the target. Every read rechecks actual database authority.
    /// # Errors
    /// Refuses non-loopback or caller-controlled startup configuration.
    pub fn connect(
        config: &Config,
        visibility: ObserverVisibility,
    ) -> Result<Self, ObserverEconomyError> {
        validate_connection_target(config).map_err(|_| ObserverEconomyError::ConnectionTarget)?;
        Ok(Self {
            config: config.clone(),
            visibility,
        })
    }
    #[must_use]
    pub const fn visibility(&self) -> ObserverVisibility {
        self.visibility
    }

    /// Read at most 64 explicitly founded Michigan material campaigns.
    /// # Errors
    /// Refuses authority, malformed identities, unknown presets or invalid clocks.
    pub fn campaigns(&self) -> Result<Vec<CampaignSummary>, ObserverEconomyError> {
        let mut config = self.config.clone();
        config
            .connect_timeout(crate::postgres_catalog::CATALOG_CONNECT_TIMEOUT)
            .tcp_user_timeout(crate::postgres_catalog::CATALOG_TCP_USER_TIMEOUT)
            .options(crate::postgres_catalog::CATALOG_STARTUP_OPTIONS);
        let mut client = config
            .connect(NoTls)
            .map_err(|_| ObserverEconomyError::Database)?;
        confine_authority(&mut client, self.visibility)?;
        let mut transaction = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|_| ObserverEconomyError::Database)?;
        let presets: Vec<_> = MICHIGAN_CONTENT_PRESETS.iter().map(|p| p.id()).collect();
        let rows = transaction
            .query(CAMPAIGN_CATALOG_SQL, &[&presets])
            .map_err(|_| ObserverEconomyError::Database)?;
        let result = rows
            .iter()
            .map(campaign_summary)
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .commit()
            .map_err(|_| ObserverEconomyError::Database)?;
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
    ) -> Result<ObserverEconomySnapshot, ObserverEconomyError> {
        let tick = i64::try_from(expected_tick).map_err(|_| ObserverEconomyError::TickAbsent)?;
        let mut config = self.config.clone();
        config
            .connect_timeout(crate::postgres_catalog::CATALOG_CONNECT_TIMEOUT)
            .tcp_user_timeout(crate::postgres_catalog::CATALOG_TCP_USER_TIMEOUT)
            .options(crate::postgres_catalog::CATALOG_STARTUP_OPTIONS);
        let mut client = config
            .connect(NoTls)
            .map_err(|_| ObserverEconomyError::Database)?;
        confine_authority(&mut client, self.visibility)?;
        let mut transaction = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|_| ObserverEconomyError::Database)?;
        if self.visibility == ObserverVisibility::FullObserver {
            // Authentication reconstructs the captured statewide circuit in Rust
            // between queries. Bound that work like material runtime reads while
            // retaining the five-second SQL, lock and connection limits.
            transaction
                .batch_execute("SET LOCAL idle_in_transaction_session_timeout = '120s'")
                .map_err(|_| ObserverEconomyError::Database)?;
        }
        let foundation = transaction.query_opt("SELECT campaign_id, foundation_sha256, scenario_sha256 FROM public.v_observer_economy_foundation_v1 WHERE campaign_id = $1", &[campaign.as_uuid()]).map_err(|_| ObserverEconomyError::Database)?.ok_or(ObserverEconomyError::CampaignAbsent)?;
        let found_campaign: uuid::Uuid = foundation
            .try_get(0)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?;
        if &found_campaign != campaign.as_uuid() {
            return Err(ObserverEconomyError::InvalidProjection);
        }
        let foundation_hash: Vec<u8> = foundation
            .try_get(1)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?;
        let scenario_hash: Vec<u8> = foundation
            .try_get(2)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?;
        let economy = michigan_economy().map_err(|_| ObserverEconomyError::Reference)?;
        let material_header = crate::observer_material::read_material_header(
            &mut transaction,
            campaign,
            expected_tick,
            self.visibility,
        )?;
        let admission = material_header
            .as_ref()
            .and_then(|header| header.admission.as_ref());
        if material_header.is_some() && admission.is_none() {
            if foundation_hash.len() != 32
                || scenario_hash.len() != 32
                || foundation_hash.iter().all(|b| *b == 0)
                || scenario_hash.iter().all(|b| *b == 0)
            {
                return Err(ObserverEconomyError::ScenarioMismatch);
            }
        } else {
            validate_observer_graph(admission, &foundation_hash, &scenario_hash)?;
        }
        let (tick_content_hash, envelope_digest) =
            read_commit_identity(&mut transaction, campaign, tick, material_header.is_some())?;
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
            // Baseline conformance has no material family. Restricted preview
            // keeps material content opaque and publishes no material facts.
            crate::observer_material::MaterialObservation {
                foundation_digest: digest_hex(
                    material_header
                        .as_ref()
                        .map_or(foundation_hash.as_slice(), |header| {
                            header.foundation_digest.as_slice()
                        }),
                ),
                production: None,
                nominal_world_hash: None,
            }
        };
        transaction
            .commit()
            .map_err(|_| ObserverEconomyError::Database)?;
        Ok(ObserverEconomySnapshot {
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

fn read_commit_identity(
    transaction: &mut impl postgres::GenericClient,
    campaign: CampaignId,
    tick: i64,
    material: bool,
) -> Result<(Option<String>, Option<String>), ObserverEconomyError> {
    if tick == 0 {
        return Ok((None, None));
    }
    let marker = transaction.query_opt("SELECT tick_content_hash, envelope_digest, envelope_layout_version FROM public.v_committed_tick_status_v1 WHERE campaign_id = $1 AND resolve_tick = $2", &[campaign.as_uuid(), &tick]).map_err(|_| ObserverEconomyError::Database)?.ok_or(ObserverEconomyError::TickAbsent)?;
    let content: Vec<u8> = marker
        .try_get(0)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let envelope: Vec<u8> = marker
        .try_get(1)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let layout: i16 = marker
        .try_get(2)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    if (material && layout != 3) || content.len() != 32 || envelope.len() != 32 {
        return Err(ObserverEconomyError::InvalidProjection);
    }
    Ok((Some(digest_hex(&content)), Some(digest_hex(&envelope))))
}

// Catalog rows expose only safe identities. Dynamic material configuration is
// opaque here; full observation independently reconstructs its stored content.
const CAMPAIGN_CATALOG_SQL: &str = "SELECT header.campaign_id,header.preset_id,header.horizon_ticks,header.content_sha256,
 header.foundation_sha256,COALESCE(max(marker.resolve_tick),0)::bigint AS durable_tick
FROM public.v_material_campaign_identity_v1 AS header
JOIN public.v_observer_economy_foundation_v1 AS graph USING(campaign_id)
LEFT JOIN public.v_committed_tick_status_v1 AS marker ON marker.campaign_id=header.campaign_id
WHERE header.preset_id=ANY($1::text[])
 AND header.campaign_id <> '00000000-0000-0000-0000-000000000000'::uuid
 AND header.horizon_ticks BETWEEN 1 AND 16
 AND octet_length(header.content_sha256)=32 AND header.content_sha256<>decode(repeat('00',32),'hex')
 AND octet_length(header.foundation_sha256)=32 AND header.foundation_sha256<>decode(repeat('00',32),'hex')
 AND octet_length(graph.foundation_sha256)=32 AND graph.foundation_sha256<>decode(repeat('00',32),'hex')
 AND octet_length(graph.scenario_sha256)=32 AND graph.scenario_sha256<>decode(repeat('00',32),'hex')
GROUP BY header.campaign_id,header.preset_id,header.horizon_ticks,header.content_sha256,header.foundation_sha256
HAVING COALESCE(max(marker.resolve_tick),0) BETWEEN 0 AND header.horizon_ticks
 AND bool_and(marker.envelope_layout_version IS NULL OR marker.envelope_layout_version=3)
ORDER BY header.campaign_id LIMIT 64";

fn campaign_summary(row: &postgres::Row) -> Result<CampaignSummary, ObserverEconomyError> {
    let campaign: uuid::Uuid = row
        .try_get(0)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let preset_id: String = row
        .try_get(1)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let horizon: i64 = row
        .try_get(2)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let content: Vec<u8> = row
        .try_get(3)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let foundation: Vec<u8> = row
        .try_get(4)
        .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let tick = u64::try_from(
        row.try_get::<_, i64>(5)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?,
    )
    .map_err(|_| ObserverEconomyError::InvalidProjection)?;
    let entry = validate_michigan_header(&preset_id, horizon, &content, &foundation, tick)
        .map_err(|_| ObserverEconomyError::ScenarioMismatch)?;
    if campaign.is_nil() {
        return Err(ObserverEconomyError::InvalidProjection);
    }
    Ok(CampaignSummary {
        id: campaign.to_string(),
        preset: preset_id,
        label: entry.label().to_owned(),
        durable_tick: tick,
    })
}

fn validate_observer_graph(
    material: Option<&MichiganContentAdmission>,
    graph: &[u8],
    scenario: &[u8],
) -> Result<(), ObserverEconomyError> {
    if let Some(admitted) = material {
        return admitted
            .validate_graph(graph, scenario)
            .map_err(|_| ObserverEconomyError::ScenarioMismatch);
    }
    // A graph-only observer has its own explicit foundation. It never borrows
    // a material preset or admits a predecessor staffed campaign.
    let (expected_graph, expected_scenario) = graph_only_observer_identity()?;
    if graph != expected_graph || scenario != expected_scenario {
        return Err(ObserverEconomyError::ScenarioMismatch);
    }
    Ok(())
}

fn graph_only_observer_identity() -> Result<([u8; 32], [u8; 32]), ObserverEconomyError> {
    type Identity = Result<([u8; 32], [u8; 32]), ObserverEconomyError>;
    static IDENTITY: std::sync::OnceLock<Identity> = std::sync::OnceLock::new();
    *IDENTITY.get_or_init(|| {
        let (session, bundle) = crate::michigan_economy::michigan_observer_foundation()
            .map_err(|_| ObserverEconomyError::Reference)?;
        let foundation = crate::CampaignFoundation::capture(&session, bundle)
            .map_err(|_| ObserverEconomyError::Reference)?;
        Ok((
            sha256_of(foundation.canonical_bytes()),
            sha256_of(foundation.content_bundle().scenario_source_bytes()),
        ))
    })
}

/// Read the complete county family through the admitted role's fixed view.
/// All rows remain in the caller's one read-only repeatable-read transaction.
fn read_committed_counties(
    transaction: &mut postgres::Transaction<'_>,
    campaign: CampaignId,
    expected_tick: u64,
    visibility: ObserverVisibility,
    baselines: &[MichiganCountyEconomy],
) -> Result<Vec<ObserverCountyEconomy>, ObserverEconomyError> {
    let tick = i64::try_from(expected_tick).map_err(|_| ObserverEconomyError::TickAbsent)?;
    let view = match visibility {
        ObserverVisibility::FullObserver => "public.v_observer_county_economy_v1",
        ObserverVisibility::KnownPreview => "public.v_known_county_economy_v1",
    };
    let query = format!("SELECT {SNAPSHOT_COLUMNS} FROM {view} WHERE campaign_id = $1 AND resolve_tick = $2 ORDER BY county_geoid LIMIT 84");
    let rows = transaction
        .query(&query, &[campaign.as_uuid(), &tick])
        .map_err(|_| ObserverEconomyError::Database)?;
    if rows.len() != 83 {
        return Err(ObserverEconomyError::InvalidProjection);
    }
    let mut counties = Vec::with_capacity(83);
    for (row, baseline) in rows.iter().zip(baselines) {
        let row_campaign: uuid::Uuid = row
            .try_get(0)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?;
        let row_tick: i64 = row
            .try_get(1)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?;
        let geoid: String = row
            .try_get(2)
            .map_err(|_| ObserverEconomyError::InvalidProjection)?;
        if &row_campaign != campaign.as_uuid() || row_tick != tick || geoid != baseline.county_geoid
        {
            return Err(ObserverEconomyError::InvalidProjection);
        }
        let mut values = [None; 4];
        let mut grants = [false; 4];
        for index in 0..4 {
            values[index] = row
                .try_get::<_, Option<i64>>(index + 3)
                .map_err(|_| ObserverEconomyError::InvalidProjection)?;
            grants[index] = row
                .try_get(index + 7)
                .map_err(|_| ObserverEconomyError::InvalidProjection)?;
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
    baseline: &MichiganCountyEconomy,
    tick: u64,
    visibility: ObserverVisibility,
    stored: [Option<i64>; 4],
    grants: [bool; 4],
) -> Result<ObserverCountyEconomy, ObserverEconomyError> {
    let baseline_values = [
        baseline.annual_avg_estabs_count,
        baseline.annual_avg_emplvl,
        baseline.total_annual_wages,
        baseline.annual_avg_wkly_wage,
    ];
    let mut values = [None; 4];
    for index in 0..4 {
        if visibility == ObserverVisibility::FullObserver && !grants[index] {
            return Err(ObserverEconomyError::InvalidProjection);
        }
        if !grants[index] {
            if stored[index].is_some() {
                return Err(ObserverEconomyError::InvalidProjection);
            }
            continue;
        }
        values[index] = Some(if tick == 0 {
            if stored[index].is_some() {
                return Err(ObserverEconomyError::InvalidProjection);
            }
            baseline_values[index]
        } else {
            u64::try_from(stored[index].ok_or(ObserverEconomyError::InvalidProjection)?)
                .map_err(|_| ObserverEconomyError::InvalidProjection)?
        });
    }
    Ok(ObserverCountyEconomy {
        county_geoid: baseline.county_geoid.clone(),
        annual_avg_estabs_count: values[0],
        annual_avg_emplvl: values[1],
        total_annual_wages: values[2],
        annual_avg_wkly_wage: values[3],
    })
}

const AUTHORITY_SQL: &str = "SELECT role.rolsuper, role.rolcreatedb, role.rolcreaterole, role.rolreplication, role.rolbypassrls, pg_catalog.pg_has_role(current_user, $1, 'MEMBER'), pg_catalog.pg_has_role(current_user, 'babylon_observer', 'MEMBER') FROM pg_catalog.pg_roles role WHERE role.rolname = current_user";
const HELD_SQL: &str = "WITH RECURSIVE role_closure(oid) AS (SELECT 0::oid UNION SELECT oid FROM pg_catalog.pg_roles WHERE rolname = current_user UNION SELECT membership.roleid FROM pg_catalog.pg_auth_members membership JOIN role_closure ON role_closure.oid = membership.member), restricted AS (SELECT relation.*, namespace.nspname FROM pg_catalog.pg_class relation JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace WHERE relation.relkind IN ('r','p','v','m','f') AND (namespace.nspname IN ('babylon_state','babylon_meta') OR (namespace.nspname = 'public' AND relation.relname IN ('v_committed_tick_status_v1','v_archive_page_known_v1','v_archive_atom_visible','v_county_card_atoms','v_archive_subject_atoms','v_archive_verification_v1','v_observer_economy_foundation_v1','v_observer_county_economy_v1','v_known_county_economy_v1','v_material_campaign_identity_v1','v_observer_material_state_v1','v_archive_revision_known_v2','v_archive_revision_atom_v2','v_archive_revision_grant_v2','v_archive_retention_v2','v_archive_subject_grant_v2','v_archive_revision_index_v2','v_archive_tick_knowledge_v2','v_archive_revision_scope_v2','v_observer_graph_node_v1','v_observer_graph_node_f64_v1','v_observer_graph_edge_v1','v_observer_graph_hyperedge_v1','v_observer_graph_hyperedge_member_v1','v_observer_graph_edge_f64_v1','v_observer_graph_node_currency_v1','v_observer_graph_hyperedge_f64_v1','v_observer_world_register_v1','v_observer_hex_state_delta_v1','v_observer_territory_state_v1','v_observer_territory_state_field_v1','v_observer_organization_state_v1','v_observer_organization_state_field_v1','v_observer_organization_territory_v1','v_observer_tick_event_v2','v_observer_tick_event_field_v2','v_observer_tick_choice_receipt_v1','v_observer_tick_choice_receipt_branch_v1','v_observer_tick_choice_receipt_carrier_element_v1','v_observer_checkpoint_manifest','v_observer_checkpoint_section_v1','v_observer_archive_dirty_receipt_v1','v_observer_tick_action_batch_v1')))) SELECT DISTINCT restricted.nspname || '.' || restricted.relname AS relation_name, acl.privilege_type, acl.is_grantable FROM restricted CROSS JOIN LATERAL pg_catalog.aclexplode(restricted.relacl) acl JOIN role_closure ON role_closure.oid = acl.grantee UNION SELECT restricted.nspname || '.' || restricted.relname, 'OWNERSHIP', false FROM restricted JOIN role_closure ON role_closure.oid = restricted.relowner UNION SELECT restricted.nspname || '.' || restricted.relname, acl.privilege_type, acl.is_grantable FROM restricted JOIN pg_catalog.pg_attribute attribute ON attribute.attrelid = restricted.oid AND attribute.attnum > 0 AND NOT attribute.attisdropped CROSS JOIN LATERAL pg_catalog.aclexplode(attribute.attacl) acl JOIN role_closure ON role_closure.oid = acl.grantee";
fn confine_authority(
    client: &mut postgres::Client,
    visibility: ObserverVisibility,
) -> Result<(), ObserverEconomyError> {
    let role = match visibility {
        ObserverVisibility::FullObserver => "babylon_observer",
        ObserverVisibility::KnownPreview => "babylon_reader",
    };
    let flags = client
        .query_one(AUTHORITY_SQL, &[&role])
        .map_err(|_| ObserverEconomyError::Database)?;
    for index in 0..5 {
        if flags
            .try_get::<_, bool>(index)
            .map_err(|_| ObserverEconomyError::Authority)?
        {
            return Err(ObserverEconomyError::Authority);
        }
    }
    if !flags
        .try_get::<_, bool>(5)
        .map_err(|_| ObserverEconomyError::Authority)?
        || (visibility == ObserverVisibility::KnownPreview
            && flags
                .try_get::<_, bool>(6)
                .map_err(|_| ObserverEconomyError::Authority)?)
    {
        return Err(ObserverEconomyError::Authority);
    }
    let rows = client
        .query(HELD_SQL, &[])
        .map_err(|_| ObserverEconomyError::Database)?;
    let mut held = std::collections::BTreeSet::new();
    for row in rows {
        let relation: String = row
            .try_get(0)
            .map_err(|_| ObserverEconomyError::Authority)?;
        let privilege: String = row
            .try_get(1)
            .map_err(|_| ObserverEconomyError::Authority)?;
        let grantable: bool = row
            .try_get(2)
            .map_err(|_| ObserverEconomyError::Authority)?;
        let allowed = match visibility {
            ObserverVisibility::FullObserver => {
                matches!(
                    relation.as_str(),
                    "public.v_observer_economy_foundation_v1"
                        | "public.v_observer_county_economy_v1"
                        | "public.v_material_campaign_identity_v1"
                        | "public.v_observer_material_state_v1"
                        | "public.v_committed_tick_status_v1"
                ) || crate::observer_tick_components::OBSERVER_TICK_COMPONENT_VIEWS
                    .contains(&relation.as_str())
            }
            ObserverVisibility::KnownPreview => matches!(
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
            return Err(ObserverEconomyError::Authority);
        }
        held.insert(relation);
    }
    let economy_view = match visibility {
        ObserverVisibility::FullObserver => "public.v_observer_county_economy_v1",
        ObserverVisibility::KnownPreview => "public.v_known_county_economy_v1",
    };
    if [
        "public.v_observer_economy_foundation_v1",
        "public.v_committed_tick_status_v1",
        "public.v_material_campaign_identity_v1",
        economy_view,
    ]
    .iter()
    .any(|view| !held.contains(*view))
        || (visibility == ObserverVisibility::FullObserver
            && crate::observer_tick_components::OBSERVER_TICK_COMPONENT_VIEWS
                .iter()
                .any(|view| !held.contains(*view)))
    {
        return Err(ObserverEconomyError::Authority);
    }
    Ok(())
}

/// Provision the confined observer group on an already verified current schema.
///
/// # Errors
/// Refuses schema or privilege drift and roles with administrative attributes.
pub fn provision_observer_role(config: &Config) -> Result<(), ObserverEconomyError> {
    validate_connection_target(config).map_err(|_| ObserverEconomyError::ConnectionTarget)?;
    let mut client = crate::current_schema::bounded_config(config)
        .connect(NoTls)
        .map_err(|_| ObserverEconomyError::Database)?;
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .read_only(false)
        .start()
        .map_err(|_| ObserverEconomyError::Database)?;
    tx.query_one(
        "SELECT pg_catalog.pg_advisory_xact_lock($1)",
        &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
    )
    .map_err(|_| ObserverEconomyError::Database)?;
    crate::current_schema::require_current_schema(&mut tx)
        .map_err(|_| ObserverEconomyError::SchemaDrift)?;
    let role = tx.query_opt("SELECT rolsuper, rolcreatedb, rolcreaterole, rolcanlogin, rolreplication, rolbypassrls FROM pg_catalog.pg_roles WHERE rolname = 'babylon_observer'", &[]).map_err(|_| ObserverEconomyError::Database)?;
    if let Some(role) = role {
        for index in 0..6 {
            if role
                .try_get::<_, bool>(index)
                .map_err(|_| ObserverEconomyError::SchemaDrift)?
            {
                return Err(ObserverEconomyError::Authority);
            }
        }
    } else {
        tx.batch_execute("CREATE ROLE babylon_observer NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS").map_err(|_| ObserverEconomyError::Database)?;
    }
    let mut expected = observer_role_views()
        .iter()
        .map(|view| format!("{view}:SELECT"))
        .collect::<Vec<_>>();
    expected.sort_unstable();
    let held =
        crate::reader::census_role_privileges(&mut tx, OBSERVER_ROLE_NAME, "census observer role")
            .map_err(|_| ObserverEconomyError::Authority)?;
    if held.is_empty() {
        let grants = format!(
            "GRANT SELECT ON {} TO babylon_observer",
            observer_role_views().join(", ")
        );
        tx.batch_execute(&grants)
            .map_err(|_| ObserverEconomyError::Database)?;
    } else if held != expected {
        return Err(ObserverEconomyError::Authority);
    }
    crate::current_schema::require_current_schema(&mut tx)
        .map_err(|_| ObserverEconomyError::SchemaDrift)?;
    let observed =
        crate::reader::census_role_privileges(&mut tx, OBSERVER_ROLE_NAME, "verify observer role")
            .map_err(|_| ObserverEconomyError::Authority)?;
    if observed != expected {
        return Err(ObserverEconomyError::Authority);
    }
    tx.commit().map_err(|_| ObserverEconomyError::Database)
}

fn observer_role_views() -> Vec<&'static str> {
    let mut views = vec![
        "public.v_observer_economy_foundation_v1",
        "public.v_observer_county_economy_v1",
        "public.v_committed_tick_status_v1",
        "public.v_material_campaign_identity_v1",
        "public.v_observer_material_state_v1",
    ];
    views.extend(crate::observer_tick_components::OBSERVER_TICK_COMPONENT_VIEWS);
    views
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_headers_bind_the_matching_graph_and_graph_only_is_separate() {
        let (graph, scenario) = graph_only_observer_identity().unwrap();
        assert!(validate_observer_graph(None, &graph, &scenario).is_ok());
        for preset in MICHIGAN_CONTENT_PRESETS
            .into_iter()
            .filter(|preset| !preset.delivery().is_statewide())
        {
            let entry = preset.admitted(&crate::test_support::catalog()).unwrap();
            assert!(validate_observer_graph(
                Some(&entry),
                &entry.graph_digest,
                &entry.scenario_digest
            )
            .is_ok());
            let baseline_only =
                validate_observer_graph(None, &entry.graph_digest, &entry.scenario_digest);
            assert_eq!(baseline_only, Err(ObserverEconomyError::ScenarioMismatch));
            for other in MICHIGAN_CONTENT_PRESETS
                .into_iter()
                .filter(|preset| !preset.delivery().is_statewide())
            {
                let other = other.admitted(&crate::test_support::catalog()).unwrap();
                assert_eq!(
                    validate_observer_graph(
                        Some(&entry),
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
    fn foundation_grants_mask_individual_fields_before_values_exist() {
        let baseline = &michigan_economy().unwrap().counties()[0];
        let row = project_county(
            baseline,
            0,
            ObserverVisibility::KnownPreview,
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
        let baseline = &michigan_economy().unwrap().counties()[0];
        assert_eq!(
            project_county(
                baseline,
                1,
                ObserverVisibility::FullObserver,
                [Some(1), None, Some(3), Some(4)],
                [true; 4]
            ),
            Err(ObserverEconomyError::InvalidProjection)
        );
        assert_eq!(
            project_county(
                baseline,
                1,
                ObserverVisibility::FullObserver,
                [Some(-1), Some(2), Some(3), Some(4)],
                [true; 4]
            ),
            Err(ObserverEconomyError::InvalidProjection)
        );
        let row = project_county(
            baseline,
            1,
            ObserverVisibility::KnownPreview,
            [Some(1), None, Some(3), None],
            [true, false, true, false],
        )
        .unwrap();
        assert_eq!(row.annual_avg_estabs_count, Some(1));
        assert_eq!(row.annual_avg_emplvl, None);
    }
    #[test]
    fn leaked_ungiven_value_is_refused_even_if_ui_would_hide_it() {
        let baseline = &michigan_economy().unwrap().counties()[0];
        assert_eq!(
            project_county(
                baseline,
                1,
                ObserverVisibility::KnownPreview,
                [Some(1); 4],
                [true, false, true, true]
            ),
            Err(ObserverEconomyError::InvalidProjection)
        );
    }
}
