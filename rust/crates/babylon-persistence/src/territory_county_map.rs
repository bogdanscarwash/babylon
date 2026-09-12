//! Declared territory→county mapping persisted once at campaign foundation (PER-22,
//! Director ruling D1, 2026-09-02).
//!
//! The mapping field `territory/county-fips` is governed geography identity:
//! [`babylon_bsl::causal_contract::GOVERNED_WRITE_PROHIBITED_NODE_FIELDS`]
//! refuses every rule write to it at load, so the graph can never rewrite
//! county identity after foundation. Reopening an already-founded campaign
//! verifies the stored rows exactly; missing or divergent mappings refuse open.

use std::collections::BTreeMap;

use babylon_bsl::scenario::{load_scenario, load_scenario_with_prelude};
use babylon_bsl::types::{BslType, FieldKind};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::GraphSubstrate;
use postgres::GenericClient;

use crate::identity::CampaignId;
use crate::postgres_diagnostic::PostgresDiagnostic;

/// Scenario field that declares a territory node's county FIPS mapping.
pub const TERRITORY_COUNTY_MAP_FIELD: &str = "territory/county-fips";
/// Substrate node type string the scenario loader stamps for `NodeType/TERRITORY`.
const TERRITORY_NODE_TYPE: &str = "TERRITORY";
/// Inclusive upper bound of the five-digit county FIPS domain.
const COUNTY_FIPS_MAX: f64 = 99_999.0;

/// One immutable declared territory→county assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerritoryCountyMapRow {
    territory_local_name: String,
    county_geoid: String,
}

impl TerritoryCountyMapRow {
    /// Validate one declared assignment.
    ///
    /// # Errors
    /// Returns [`TerritoryCountyMapError`] for an empty local name or a geoid
    /// outside the exact five-digit census domain.
    pub fn try_new(
        territory_local_name: String,
        county_geoid: String,
    ) -> Result<Self, TerritoryCountyMapError> {
        if territory_local_name.is_empty() {
            return Err(TerritoryCountyMapError::InvalidTerritoryLocalName);
        }
        if !county_geoid.bytes().all(|byte| byte.is_ascii_digit())
            || county_geoid.len() != 5
            || !county_geoid.is_ascii()
        {
            return Err(TerritoryCountyMapError::InvalidCountyGeoid);
        }
        Ok(Self {
            territory_local_name,
            county_geoid,
        })
    }

    /// Stable scenario-local territory name.
    #[must_use]
    pub fn territory_local_name(&self) -> &str {
        &self.territory_local_name
    }

    /// Five-digit county GEOID with leading zeros preserved.
    #[must_use]
    pub fn county_geoid(&self) -> &str {
        &self.county_geoid
    }
}

/// Closed failure boundary for the declared territory-county mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerritoryCountyMapError {
    /// The scenario source could not be re-read for extraction.
    ScenarioLoad,
    /// The scenario declared the mapping field with anything but an `int`
    /// type AND an `extensive` kind; both axes are mandatory.
    FieldDeclRefused,
    /// A `TERRITORY` node did not seed the declared mapping field.
    MissingCountyFips {
        /// Refused scenario-local node name.
        node: String,
    },
    /// A seeded county FIPS value is not an integer in the five-digit domain.
    CountyFipsOutOfRange {
        /// Refused scenario-local node name.
        node: String,
        /// Exact seeded value in canonical decimal rendering.
        value: String,
    },
    /// Two territory nodes declared the same county GEOID in one campaign.
    DuplicateCountyGeoid {
        /// Refused shared five-digit GEOID.
        geoid: String,
        /// First territory node in scenario declaration order.
        first_node: String,
        /// Second territory node in scenario declaration order.
        second_node: String,
    },
    /// A row carried an empty territory local name.
    InvalidTerritoryLocalName,
    /// A row carried a geoid outside the exact five-digit census domain.
    InvalidCountyGeoid,
    /// Stored mapping rows diverge from the scenario-declared mapping. The
    /// durable rows are never overwritten; a human must reconcile.
    StoredMappingDiverged {
        /// Exact number of rows already stored for the campaign.
        stored_rows: usize,
        /// Exact number of rows the scenario declares today.
        declared_rows: usize,
    },
    /// A local-only schema or row operation failed.
    Database {
        /// Stable operation name without caller-supplied text.
        operation: &'static str,
        /// Bounded secret-safe driver diagnostic, when the failure came from `PostgreSQL`.
        diagnostic: Option<PostgresDiagnostic>,
    },
}

impl std::fmt::Display for TerritoryCountyMapError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "territory county map refused: {self:?}")
    }
}

impl std::error::Error for TerritoryCountyMapError {}

impl From<postgres::Error> for TerritoryCountyMapError {
    fn from(error: postgres::Error) -> Self {
        database("territory county map operation", &error)
    }
}

fn database(operation: &'static str, error: &postgres::Error) -> TerritoryCountyMapError {
    TerritoryCountyMapError::Database {
        operation,
        diagnostic: Some(PostgresDiagnostic::capture(error)),
    }
}

/// Extract the declared territory→county mapping from one scenario source.
///
/// The scenario is re-read through the sole BSL loader — with the campaign's
/// declaration prelude when one exists, exactly as session hydration does —
/// into a disposable graph, so the extraction observes exactly the seeded
/// content identity the campaign foundation persists. A scenario that does
/// not declare [`TERRITORY_COUNTY_MAP_FIELD`] extracts no rows and is
/// never refused; once declared, the declaration must be `int` AND
/// `extensive`, and every `TERRITORY` node must seed an integer in
/// `0..=99999`. No two territory nodes may share a county GEOID.
///
/// The extracted rows are a pure function of the scenario content; the
/// caller is responsible for binding them to the session graph before
/// persisting (see `CampaignFoundationV1::capture`'s scenario bind).
///
/// # Errors
/// Returns [`TerritoryCountyMapError`] for a load failure, a refused
/// field declaration (anything but `int` + `extensive`), a missing seed,
/// an out-of-range seed, or a duplicate GEOID.
pub fn extract_declared_territory_county_map(
    scenario_source: &str,
    prelude_source: Option<&str>,
) -> Result<Vec<TerritoryCountyMapRow>, TerritoryCountyMapError> {
    let mut graph = HypergraphStore::new();
    let loaded = match prelude_source {
        Some(prelude) => load_scenario_with_prelude(prelude, scenario_source, &mut graph),
        None => load_scenario(scenario_source, &mut graph),
    }
    .map_err(|_| TerritoryCountyMapError::ScenarioLoad)?;
    let Some(declaration) = loaded.fields.get(TERRITORY_COUNTY_MAP_FIELD) else {
        return Ok(Vec::new());
    };
    if declaration.ty != BslType::Int || declaration.kind != FieldKind::Extensive {
        return Err(TerritoryCountyMapError::FieldDeclRefused);
    }
    let mut geoid_owner: BTreeMap<String, String> = BTreeMap::new();
    for node_id in graph.nodes(TERRITORY_NODE_TYPE) {
        let local = loaded
            .node_content_ids
            .get(&node_id)
            .cloned()
            .ok_or(TerritoryCountyMapError::ScenarioLoad)?;
        let raw = graph
            .node_attribute(node_id, TERRITORY_COUNTY_MAP_FIELD)
            .map_err(|_| TerritoryCountyMapError::MissingCountyFips {
                node: local.clone(),
            })?;
        if !raw.is_finite() || raw.fract() != 0.0 || !(0.0..=COUNTY_FIPS_MAX).contains(&raw) {
            return Err(TerritoryCountyMapError::CountyFipsOutOfRange {
                node: local,
                value: raw.to_string(),
            });
        }
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the whole-number range check above bounds the cast to 0..=99999"
        )]
        let value = raw as i64;
        let geoid = format!("{value:05}");
        if let Some(first_node) = geoid_owner.insert(geoid.clone(), local.clone()) {
            return Err(TerritoryCountyMapError::DuplicateCountyGeoid {
                geoid,
                first_node,
                second_node: local,
            });
        }
    }
    let rows = geoid_owner
        .into_iter()
        .map(
            |(county_geoid, territory_local_name)| TerritoryCountyMapRow {
                territory_local_name,
                county_geoid,
            },
        )
        .collect::<Vec<_>>();
    Ok(rows)
}

/// Persist one campaign's declared mapping rows once, at foundation time.
///
/// Exact retries reconcile through the primary key; the rows are written only
/// in the campaign-foundation transaction and never per tick.
///
/// # Errors
/// Returns [`TerritoryCountyMapError`] for a database failure.
pub(crate) fn insert_territory_county_map_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    rows: &[TerritoryCountyMapRow],
) -> Result<(), TerritoryCountyMapError> {
    for row in rows {
        client
            .execute(
                "INSERT INTO babylon_meta.territory_county_map_v1 \
                 (campaign_id, territory_local_name, county_geoid) \
                 VALUES ($1::uuid, $2, $3) ON CONFLICT (campaign_id, territory_local_name) DO NOTHING",
                &[
                    campaign_id.as_uuid(),
                    &row.territory_local_name,
                    &row.county_geoid,
                ],
            )
            .map_err(|error| database("insert territory county map row", &error))?;
    }
    Ok(())
}

/// Read one campaign's stored mapping rows in a deterministic order.
///
/// # Errors
/// Returns [`TerritoryCountyMapError`] for a database failure or a stored
/// row that violates the row shape the schema pins.
fn read_territory_county_map_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
) -> Result<Vec<TerritoryCountyMapRow>, TerritoryCountyMapError> {
    let rows = client
        .query(
            "SELECT territory_local_name, county_geoid \
             FROM babylon_meta.territory_county_map_v1 \
             WHERE campaign_id = $1::uuid ORDER BY territory_local_name, county_geoid",
            &[campaign_id.as_uuid()],
        )
        .map_err(|error| database("read territory county map rows", &error))?;
    rows.iter()
        .map(|row| {
            let local: String = row
                .try_get(0)
                .map_err(|error| database("decode territory county map local name", &error))?;
            let geoid: String = row
                .try_get(1)
                .map_err(|error| database("decode territory county map geoid", &error))?;
            TerritoryCountyMapRow::try_new(local, geoid)
        })
        .collect()
}

/// Verify one current campaign's stored mapping against its immutable declaration.
///
/// Reopening never installs schema or repairs missing rows. Foundation creation
/// owns those writes; incomplete development saves must be recreated.
///
/// # Errors
/// Returns [`TerritoryCountyMapError`] for extraction, missing schema, divergent
/// stored rows, or a database failure.
pub(crate) fn verify_territory_county_map(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    scenario_source: &str,
    prelude_source: Option<&str>,
) -> Result<(), TerritoryCountyMapError> {
    let mut declared = extract_declared_territory_county_map(scenario_source, prelude_source)?;
    let stored = read_territory_county_map_rows(client, campaign_id)?;
    declared.sort_by(|left, right| {
        left.territory_local_name
            .cmp(&right.territory_local_name)
            .then_with(|| left.county_geoid.cmp(&right.county_geoid))
    });
    if stored != declared {
        return Err(TerritoryCountyMapError::StoredMappingDiverged {
            stored_rows: stored.len(),
            declared_rows: declared.len(),
        });
    }
    Ok(())
}
