//! Canonical typed-row reader shared by retry reconciliation and restart.

use std::collections::BTreeMap;

use babylon_bsl::identity_codec::StableBslValueV1;
use babylon_graph::stable_element::StableElementKeyV1;
use babylon_graph::stable_state::{
    compose_stable_graph_state_from_rows_v1, StableGraphStateRowsInputV1, StableGraphStateV1,
};
use babylon_kernel::tick_content_hash::TickContentHashV1;
use babylon_kernel::{sha256_of, H3CellId};
use babylon_tick::h3_runtime::MichiganDynamicHexValueBitsV1;
use babylon_tick::material_state::{
    DynamicHexStateRowV1, MaterialStateRowsInputV1, MaterialStateRowsV1, OrganizationStateRowV1,
    TerritoryStateRowV1, WorldRegisterRowV1,
};
use postgres::{GenericClient, Row};

use crate::committed_tick_envelope::{
    CommittedTickEnvelopeV2, CommittedTickRowFamiliesV2, CommittedTickRowV2,
};
use crate::identity::CampaignId;
use crate::runtime::RustPersistenceRuntimeErrorV2;
use crate::semantic_batches::{
    compose_graph_rows_with_encoder_v1, compose_material_state_rows_v1, StableGraphRowRefV1,
};
use crate::semantic_codec;
use crate::tick_commit_claim::TickCommitClaimV1;

pub(crate) struct StoredTypedTickV2 {
    envelope: CommittedTickEnvelopeV2,
    graph_state: StableGraphStateV1,
    material_rows: MaterialStateRowsV1,
    checkpoint_sections: Vec<Vec<u8>>,
    action_layout: i16,
    action_digest: [u8; 32],
    action_bytes: Vec<u8>,
}

impl StoredTypedTickV2 {
    pub(crate) const fn envelope(&self) -> &CommittedTickEnvelopeV2 {
        &self.envelope
    }

    pub(crate) const fn graph_state(&self) -> &StableGraphStateV1 {
        &self.graph_state
    }

    pub(crate) const fn material_rows(&self) -> &MaterialStateRowsV1 {
        &self.material_rows
    }

    pub(crate) fn checkpoint_section(&self, tag: u8) -> Option<&[u8]> {
        tag.checked_sub(1)
            .and_then(|index| self.checkpoint_sections.get(usize::from(index)))
            .map(Vec::as_slice)
    }

    pub(crate) const fn action_layout(&self) -> i16 {
        self.action_layout
    }

    pub(crate) const fn action_digest(&self) -> &[u8; 32] {
        &self.action_digest
    }

    pub(crate) fn action_bytes(&self) -> &[u8] {
        &self.action_bytes
    }
}

pub(crate) fn read_stored_typed_tick_v2(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: u64,
    scenario_scope: &str,
) -> Result<Option<StoredTypedTickV2>, RustPersistenceRuntimeErrorV2> {
    let resolve_tick_sql =
        i64::try_from(resolve_tick).map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let Some(marker) = client
        .query_opt(
            "SELECT envelope_layout_version, tick_content_hash, envelope_digest \
             FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2",
            &[campaign_id.as_uuid(), &resolve_tick_sql],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("read stored tick marker", &error)
        })?
    else {
        return Ok(None);
    };
    let marker_layout: i16 = decode_column(&marker, 0)?;
    let marker_content_hash = decode_digest(&marker, 1)?;
    let marker_envelope_digest = decode_digest(&marker, 2)?;
    if marker_layout != 2 {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }

    let action = client
        .query_opt(
            "SELECT layout_version, action_batch_digest, exact_action_batch_bytes \
             FROM babylon_state.tick_action_batch_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2",
            &[campaign_id.as_uuid(), &resolve_tick_sql],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("read stored action batch", &error)
        })?
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let action_layout: i16 = decode_column(&action, 0)?;
    let action_digest = decode_digest(&action, 1)?;
    let action_bytes: Vec<u8> = decode_column(&action, 2)?;
    if action_layout != 1 {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }

    let graph_state = read_graph_state(client, campaign_id, resolve_tick_sql, scenario_scope)?;
    let (graph, _) =
        compose_graph_rows_with_encoder_v1(graph_state.rows(), &mut |row: StableGraphRowRefV1<
            '_,
        >| row.encode())?;
    let material_rows = read_material_rows(client, campaign_id, resolve_tick_sql)?;
    let state = compose_material_state_rows_v1(&material_rows)?;
    let choice_receipt = read_choice_receipt_rows(client, campaign_id, resolve_tick_sql)?;
    let event = read_event_rows(client, campaign_id, resolve_tick_sql)?;
    let (checkpoint, checkpoint_sections) = read_checkpoint_rows(
        client,
        campaign_id,
        resolve_tick,
        resolve_tick_sql,
        &graph_state,
        &material_rows,
    )?;
    let receipt = read_archive_receipt(client, campaign_id, resolve_tick_sql)?;
    let claim = TickCommitClaimV1::compose(
        campaign_id,
        resolve_tick,
        TickContentHashV1::from_bytes(marker_content_hash),
    );
    let envelope = CommittedTickEnvelopeV2::compose(
        claim,
        CommittedTickRowFamiliesV2 {
            graph,
            state,
            event,
            choice_receipt,
            checkpoint,
            archive_dirty_receipt: receipt,
        },
    )
    .map_err(RustPersistenceRuntimeErrorV2::SemanticEnvelope)?;
    if envelope.digest().as_bytes() != &marker_envelope_digest {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(Some(StoredTypedTickV2 {
        envelope,
        graph_state,
        material_rows,
        checkpoint_sections,
        action_layout,
        action_digest,
        action_bytes,
    }))
}

pub(crate) fn read_graph_state(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    scenario_scope: &str,
) -> Result<StableGraphStateV1, RustPersistenceRuntimeErrorV2> {
    compose_stable_graph_state_from_rows_v1(
        scenario_scope,
        StableGraphStateRowsInputV1 {
            nodes: read_graph_nodes(client, campaign_id, resolve_tick)?,
            node_f64: read_graph_node_f64(client, campaign_id, resolve_tick)?,
            edges: read_graph_edges(client, campaign_id, resolve_tick)?,
            hyperedges: read_graph_hyperedges(client, campaign_id, resolve_tick)?,
            edge_f64: read_graph_edge_f64(client, campaign_id, resolve_tick)?,
            node_currency: read_graph_node_currency(client, campaign_id, resolve_tick)?,
            hyperedge_f64: read_graph_hyperedge_f64(client, campaign_id, resolve_tick)?,
        },
    )
    .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)
}

type GraphNodeRowsV1 = Vec<(String, String)>;
type GraphNodeF64RowsV1 = Vec<(String, String, u64)>;
type GraphEdgeRowsV1 = Vec<(String, String, String, u64)>;
type GraphHyperedgeRowsV1 = Vec<(String, String, Vec<String>)>;
type GraphEdgeF64RowsV1 = Vec<(String, String, String, String, u64)>;
type GraphNodeCurrencyRowsV1 = Vec<(String, String, i128)>;
type GraphHyperedgeF64RowsV1 = Vec<(String, String, u64)>;

fn read_graph_nodes(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphNodeRowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT local_name, node_type FROM babylon_state.graph_node_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY local_name",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph nodes", &error))?
        .iter()
        .map(|row| Ok((decode_column(row, 0)?, decode_column(row, 1)?)))
        .collect()
}

fn read_graph_node_f64(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphNodeF64RowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT local_name, qname, value_bits FROM babylon_state.graph_node_f64_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY local_name, qname",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph node f64", &error))?
        .iter()
        .map(|row| {
            let bits: i64 = decode_column(row, 2)?;
            Ok((
                decode_column(row, 0)?,
                decode_column(row, 1)?,
                unsigned_bits(bits),
            ))
        })
        .collect()
}

fn read_graph_edges(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphEdgeRowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT edge_type, source_local_name, target_local_name, strength_bits \
             FROM babylon_state.graph_edge_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY edge_type, source_local_name, target_local_name",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph edges", &error))?
        .iter()
        .map(|row| {
            let bits: i64 = decode_column(row, 3)?;
            Ok((
                decode_column(row, 0)?,
                decode_column(row, 1)?,
                decode_column(row, 2)?,
                unsigned_bits(bits),
            ))
        })
        .collect()
}

fn read_graph_hyperedges(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphHyperedgeRowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT edge.local_name, edge.hyperedge_type, \
                    ARRAY(SELECT member.position FROM babylon_state.graph_hyperedge_member_v1 AS member \
                          WHERE member.campaign_id = edge.campaign_id \
                            AND member.resolve_tick = edge.resolve_tick \
                            AND member.local_name = edge.local_name ORDER BY member.position), \
                    ARRAY(SELECT member.member FROM babylon_state.graph_hyperedge_member_v1 AS member \
                          WHERE member.campaign_id = edge.campaign_id \
                            AND member.resolve_tick = edge.resolve_tick \
                            AND member.local_name = edge.local_name ORDER BY member.position) \
             FROM babylon_state.graph_hyperedge_v1 AS edge \
             WHERE edge.campaign_id = $1::uuid AND edge.resolve_tick = $2 ORDER BY edge.local_name",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph hyperedges", &error))?
        .iter()
        .map(|row| {
            let positions: Vec<i32> = decode_column(row, 2)?;
            let members: Vec<String> = decode_column(row, 3)?;
            if positions.len() != members.len()
                || positions
                    .iter()
                    .enumerate()
                    .any(|(expected, actual)| usize::try_from(*actual).ok() != Some(expected))
            {
                return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
            }
            Ok((
                decode_column(row, 0)?,
                decode_column(row, 1)?,
                members,
            ))
        })
        .collect()
}

fn read_graph_edge_f64(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphEdgeF64RowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT edge_type, source_local_name, target_local_name, qname, value_bits \
             FROM babylon_state.graph_edge_f64_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY edge_type, source_local_name, target_local_name, qname",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph edge f64", &error))?
        .iter()
        .map(|row| {
            let bits: i64 = decode_column(row, 4)?;
            Ok((
                decode_column(row, 0)?,
                decode_column(row, 1)?,
                decode_column(row, 2)?,
                decode_column(row, 3)?,
                unsigned_bits(bits),
            ))
        })
        .collect()
}

fn read_graph_node_currency(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphNodeCurrencyRowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT local_name, qname, micro_units::text \
             FROM babylon_state.graph_node_currency_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY local_name, qname",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph Currency", &error))?
        .iter()
        .map(|row| {
            let value: String = decode_column(row, 2)?;
            Ok((
                decode_column(row, 0)?,
                decode_column(row, 1)?,
                value
                    .parse::<i128>()
                    .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            ))
        })
        .collect()
}

fn read_graph_hyperedge_f64(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<GraphHyperedgeF64RowsV1, RustPersistenceRuntimeErrorV2> {
    client
        .query(
            "SELECT local_name, qname, value_bits \
             FROM babylon_state.graph_hyperedge_f64_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY local_name, qname",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored graph hyperedge f64", &error))?
        .iter()
        .map(|row| {
            let bits: i64 = decode_column(row, 2)?;
            Ok((
                decode_column(row, 0)?,
                decode_column(row, 1)?,
                unsigned_bits(bits),
            ))
        })
        .collect()
}

pub(crate) fn read_material_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<MaterialStateRowsV1, RustPersistenceRuntimeErrorV2> {
    let world_registers = client
        .query(
            "SELECT register_name, value_tag, int_value, currency_value::text, real_bits, \
                    ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, enum_type, enum_member, stable_key \
             FROM babylon_state.world_register_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY register_name",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored world registers", &error))?
        .iter()
        .map(|row| {
            WorldRegisterRowV1::try_new(decode_column(row, 0)?, decode_bsl_value(row, 1)?)
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)
        })
        .collect::<Result<Vec<_>, RustPersistenceRuntimeErrorV2>>()?;
    let territories = read_territories(client, campaign_id, resolve_tick)?;
    let dynamic_hexes = client
        .query(
            "SELECT cell_id, c_bits, v_bits, s_bits, k_bits, biocapacity_stock_bits, \
                    energy_stock_bits, raw_material_stock_bits, internet_access_pct_bits, \
                    surveillance_coupling_bits \
             FROM babylon_state.hex_state_delta_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY cell_id",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored dynamic hex state", &error))?
        .iter()
        .map(|row| {
            let cell: i64 = decode_column(row, 0)?;
            let cell = H3CellId::try_from(cell)
                .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
            let bits = MichiganDynamicHexValueBitsV1 {
                c: unsigned_bits(decode_column(row, 1)?),
                v: unsigned_bits(decode_column(row, 2)?),
                s: unsigned_bits(decode_column(row, 3)?),
                k: unsigned_bits(decode_column(row, 4)?),
                biocapacity_stock: unsigned_bits(decode_column(row, 5)?),
                energy_stock: unsigned_bits(decode_column(row, 6)?),
                raw_material_stock: unsigned_bits(decode_column(row, 7)?),
                internet_access_pct: unsigned_bits(decode_column(row, 8)?),
                surveillance_coupling: unsigned_bits(decode_column(row, 9)?),
            };
            DynamicHexStateRowV1::try_new(cell, bits)
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)
        })
        .collect::<Result<Vec<_>, RustPersistenceRuntimeErrorV2>>()?;
    let organizations = read_organizations(client, campaign_id, resolve_tick)?;
    MaterialStateRowsV1::try_from_rows(MaterialStateRowsInputV1 {
        world_registers,
        territories,
        dynamic_hexes,
        organizations,
    })
    .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)
}

type NamedStableValues = BTreeMap<Vec<u8>, Vec<(String, StableBslValueV1)>>;

fn read_territories(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<Vec<TerritoryStateRowV1>, RustPersistenceRuntimeErrorV2> {
    let mut fields: NamedStableValues = BTreeMap::new();
    for row in client
        .query(
            "SELECT territory_id, position, field_name, value_tag, int_value, currency_value::text, \
                    real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, enum_type, \
                    enum_member, stable_key \
             FROM babylon_state.territory_state_field_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY territory_id, position",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored territory fields", &error))?
    {
        let key: Vec<u8> = decode_column(&row, 0)?;
        let position: i32 = decode_column(&row, 1)?;
        let target = fields.entry(key).or_default();
        require_position(position, target.len())?;
        target.push((decode_column(&row, 2)?, decode_bsl_value(&row, 3)?));
    }
    let mut output = Vec::new();
    for row in client
        .query(
            "SELECT territory_id FROM babylon_state.territory_state_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY territory_id",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored territory state", &error))?
    {
        let bytes: Vec<u8> = decode_column(&row, 0)?;
        let key = decode_stable_key(&bytes)?;
        output.push(
            TerritoryStateRowV1::try_new(key, fields.remove(&bytes).unwrap_or_default())
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?,
        );
    }
    if !fields.is_empty() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(output)
}

type StableKeyLists = BTreeMap<Vec<u8>, Vec<StableElementKeyV1>>;

fn read_organizations(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<Vec<OrganizationStateRowV1>, RustPersistenceRuntimeErrorV2> {
    let mut territories: StableKeyLists = BTreeMap::new();
    for row in client
        .query(
            "SELECT organization_id, position, territory_id \
             FROM babylon_state.organization_territory_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY organization_id, position",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored organization territories", &error))?
    {
        let owner: Vec<u8> = decode_column(&row, 0)?;
        let position: i32 = decode_column(&row, 1)?;
        let target = territories.entry(owner).or_default();
        require_position(position, target.len())?;
        let key: Vec<u8> = decode_column(&row, 2)?;
        target.push(decode_stable_key(&key)?);
    }
    let mut fields: NamedStableValues = BTreeMap::new();
    for row in client
        .query(
            "SELECT organization_id, position, field_name, value_tag, int_value, \
                    currency_value::text, real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, \
                    bool_value, enum_type, enum_member, stable_key \
             FROM babylon_state.organization_state_field_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY organization_id, position",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored organization fields", &error))?
    {
        let owner: Vec<u8> = decode_column(&row, 0)?;
        let position: i32 = decode_column(&row, 1)?;
        let target = fields.entry(owner).or_default();
        require_position(position, target.len())?;
        target.push((decode_column(&row, 2)?, decode_bsl_value(&row, 3)?));
    }
    let mut output = Vec::new();
    for row in client
        .query(
            "SELECT organization_id, organization_kind_tag, organization_kind_int, \
                    organization_kind_currency::text, organization_kind_real_bits, \
                    organization_kind_ratio_bits, organization_kind_ratio_min_bits, \
                    organization_kind_ratio_max_bits, organization_kind_bool, \
                    organization_kind_enum_type, organization_kind_enum_member, \
                    organization_kind_stable_key \
             FROM babylon_state.organization_state_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY organization_id",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored organization state", &error))?
    {
        let bytes: Vec<u8> = decode_column(&row, 0)?;
        output.push(
            OrganizationStateRowV1::try_new(
                decode_stable_key(&bytes)?,
                decode_bsl_value(&row, 1)?,
                territories.remove(&bytes).unwrap_or_default(),
                fields.remove(&bytes).unwrap_or_default(),
            )
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?,
        );
    }
    if !territories.is_empty() || !fields.is_empty() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(output)
}

pub(crate) fn read_event_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<Vec<CommittedTickRowV2>, RustPersistenceRuntimeErrorV2> {
    let mut fields: BTreeMap<i64, Vec<(String, StableBslValueV1)>> = BTreeMap::new();
    for row in client
        .query(
            "SELECT ordinal, position, field_name, value_tag, int_value, currency_value::text, \
                    real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, enum_type, \
                    enum_member, stable_key \
             FROM babylon_state.tick_event_field_v2 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY ordinal, position",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored event fields", &error))?
    {
        let ordinal: i64 = decode_column(&row, 0)?;
        let position: i64 = decode_column(&row, 1)?;
        let target = fields.entry(ordinal).or_default();
        require_i64_position(position, target.len())?;
        target.push((decode_column(&row, 2)?, decode_bsl_value(&row, 3)?));
    }
    let mut output = Vec::new();
    for row in client
        .query(
            "SELECT ordinal, event_type, emitting_rule, choice_receipt_ordinal \
             FROM babylon_state.tick_event_v2 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY ordinal",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored events", &error))?
    {
        let ordinal: i64 = decode_column(&row, 0)?;
        let ordinal_u32 =
            u32::try_from(ordinal).map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        if usize::try_from(ordinal).ok() != Some(output.len()) {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let owned = fields.remove(&ordinal).unwrap_or_default();
        let borrowed = owned
            .iter()
            .map(|(name, value)| (name.as_str(), value))
            .collect::<Vec<_>>();
        output.push(semantic_codec::encode_successful_event(
            ordinal_u32,
            &decode_column::<String>(&row, 2)?,
            decode_column::<Option<i64>>(&row, 3)?
                .map(|value| {
                    u32::try_from(value)
                        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
                })
                .transpose()?,
            &decode_column::<String>(&row, 1)?,
            &borrowed,
        )?);
    }
    if !fields.is_empty() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(output)
}

pub(crate) fn read_choice_receipt_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<Vec<CommittedTickRowV2>, RustPersistenceRuntimeErrorV2> {
    let mut branches: BTreeMap<i64, Vec<semantic_codec::ChoiceReceiptSemanticBranchV1>> =
        BTreeMap::new();
    for row in client
        .query(
            "SELECT encounter_ordinal, position, outcome_member, mass_nanounits::text, \
                    ticket_start::text, ticket_end_exclusive::text, ticket_count::text \
             FROM babylon_state.tick_choice_receipt_branch_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY encounter_ordinal, position",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored choice receipt branches", &error))?
    {
        let ordinal: i64 = decode_column(&row, 0)?;
        let position: i64 = decode_column(&row, 1)?;
        let target = branches.entry(ordinal).or_default();
        require_i64_position(position, target.len())?;
        target.push(semantic_codec::ChoiceReceiptSemanticBranchV1 {
            outcome_member: decode_column(&row, 2)?,
            mass_nanounits: decode_decimal(&row, 3)?,
            ticket_start: decode_decimal(&row, 4)?,
            ticket_end_exclusive: decode_decimal(&row, 5)?,
            ticket_count: decode_decimal(&row, 6)?,
        });
    }

    let mut carriers: BTreeMap<i64, Vec<StableElementKeyV1>> = BTreeMap::new();
    for row in client
        .query(
            "SELECT encounter_ordinal, position, stable_element \
             FROM babylon_state.tick_choice_receipt_carrier_element_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY encounter_ordinal, position",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored choice receipt carriers", &error))?
    {
        let ordinal: i64 = decode_column(&row, 0)?;
        let position: i64 = decode_column(&row, 1)?;
        let target = carriers.entry(ordinal).or_default();
        require_i64_position(position, target.len())?;
        let bytes: Vec<u8> = decode_column(&row, 2)?;
        target.push(decode_stable_key(&bytes)?);
    }

    let mut output = Vec::new();
    for row in client
        .query(
            "SELECT encounter_ordinal, rule_id, sample, slot, outcome_enum, stable_carrier, \
                    draw_ticket::text, selected_outcome, allocation_digest, instance_digest \
             FROM babylon_state.tick_choice_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 \
             ORDER BY encounter_ordinal",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored choice receipts", &error))?
    {
        let ordinal: i64 = decode_column(&row, 0)?;
        require_i64_position(ordinal, output.len())?;
        let stable_carrier: Vec<u8> = decode_column(&row, 5)?;
        let receipt = semantic_codec::ChoiceReceiptSemanticRowV1 {
            encounter_ordinal: u32::try_from(ordinal)
                .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            rule_id: decode_column(&row, 1)?,
            sample: decode_column(&row, 2)?,
            slot: u32::try_from(decode_column::<i64>(&row, 3)?)
                .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            outcome_enum: decode_column(&row, 4)?,
            stable_carrier: decode_stable_key(&stable_carrier)?,
            active_elements: carriers.remove(&ordinal).unwrap_or_default(),
            branches: branches
                .remove(&ordinal)
                .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            draw_ticket: decode_decimal(&row, 6)?,
            selected_outcome: decode_column(&row, 7)?,
            allocation_digest: decode_digest(&row, 8)?,
            instance_digest: decode_digest(&row, 9)?,
        };
        output.push(semantic_codec::encode_choice_receipt(&receipt)?);
    }
    if !branches.is_empty() || !carriers.is_empty() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(output)
}

pub(crate) fn read_checkpoint_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: u64,
    resolve_tick_sql: i64,
    graph: &StableGraphStateV1,
    material: &MaterialStateRowsV1,
) -> Result<(Vec<CommittedTickRowV2>, Vec<Vec<u8>>), RustPersistenceRuntimeErrorV2> {
    let manifest = client
        .query_opt(
            "SELECT completeness_tag, manifest_bytes, manifest_sha256 \
             FROM babylon_state.checkpoint_manifest \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2",
            &[campaign_id.as_uuid(), &resolve_tick_sql],
        )
        .map_err(|error| database("read stored checkpoint manifest", &error))?
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let completeness: i16 = decode_column(&manifest, 0)?;
    let manifest_bytes: Vec<u8> = decode_column(&manifest, 1)?;
    let manifest_digest = decode_digest(&manifest, 2)?;
    if completeness != 1 || sha256_of(&manifest_bytes) != manifest_digest {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    let stored = client
        .query(
            "SELECT section_tag, ordinal, exact_section_bytes \
             FROM babylon_state.checkpoint_section_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2 ORDER BY section_tag, ordinal",
            &[campaign_id.as_uuid(), &resolve_tick_sql],
        )
        .map_err(|error| database("read stored checkpoint sections", &error))?;
    if stored.len() != 9 {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    let mut rows = Vec::new();
    let mut sections = Vec::new();
    let graph_count = graph_row_count(graph)?;
    let material_count = u32::try_from(material.source_count())
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let mut summaries = Vec::new();
    for (index, row) in stored.iter().enumerate() {
        let tag: i16 = decode_column(row, 0)?;
        let ordinal: i64 = decode_column(row, 1)?;
        let expected_tag = i16::try_from(index + 1)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        if tag != expected_tag || ordinal != 0 {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let bytes: Vec<u8> = decode_column(row, 2)?;
        let tag_u8 =
            u8::try_from(tag).map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        let row_count = match tag_u8 {
            1 => graph_count,
            9 => material_count,
            _ => 1,
        };
        summaries.push((tag_u8, row_count, sha256_of(&bytes)));
        rows.push(semantic_codec::encode_checkpoint_row(tag_u8, 0, 1, &bytes)?);
        sections.push(bytes);
    }
    if sections.first().map(Vec::as_slice) != Some(graph.canonical_bytes())
        || sections.get(8).map(Vec::as_slice) != Some(material.canonical_bytes())
        || semantic_codec::encode_full_checkpoint(campaign_id, resolve_tick, &summaries)?
            != manifest_bytes
    {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok((rows, sections))
}

pub(crate) fn read_archive_receipt(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
) -> Result<CommittedTickRowV2, RustPersistenceRuntimeErrorV2> {
    let row = client
        .query_opt(
            "SELECT tick_content_hash FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .map_err(|error| database("read stored archive receipt", &error))?
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    semantic_codec::encode_archive_dirty_receipt(&decode_digest(&row, 0)?).map_err(Into::into)
}

fn decode_bsl_value(
    row: &Row,
    start: usize,
) -> Result<StableBslValueV1, RustPersistenceRuntimeErrorV2> {
    let tag: i16 = decode_column(row, start)?;
    let int_value: Option<i64> = decode_column(row, start + 1)?;
    let currency_value: Option<String> = decode_column(row, start + 2)?;
    let real_bits: Option<i64> = decode_column(row, start + 3)?;
    let ratio_bits: Option<i64> = decode_column(row, start + 4)?;
    let ratio_min_bits: Option<i64> = decode_column(row, start + 5)?;
    let ratio_max_bits: Option<i64> = decode_column(row, start + 6)?;
    let bool_value: Option<bool> = decode_column(row, start + 7)?;
    let enum_type: Option<String> = decode_column(row, start + 8)?;
    let enum_member: Option<String> = decode_column(row, start + 9)?;
    let stable_key: Option<Vec<u8>> = decode_column(row, start + 10)?;
    match tag {
        1 => int_value
            .map(StableBslValueV1::Int)
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict),
        2 => currency_value
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?
            .parse::<i128>()
            .map(StableBslValueV1::CurrencyMicroUnits)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict),
        3 => real_bits
            .map(|value| StableBslValueV1::RealBits(unsigned_bits(value)))
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict),
        4 => ratio_bits
            .map(|value| StableBslValueV1::RatioBits {
                value: unsigned_bits(value),
                floor: ratio_min_bits.map(unsigned_bits),
                cap: ratio_max_bits.map(unsigned_bits),
            })
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict),
        5 => bool_value
            .map(StableBslValueV1::Bool)
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict),
        6 => Ok(StableBslValueV1::Enum {
            enum_type: enum_type.ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            member: enum_member.ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        }),
        7 => Ok(StableBslValueV1::Node(decode_stable_key(
            &stable_key.ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        )?)),
        8 => Ok(StableBslValueV1::Hyperedge(decode_stable_key(
            &stable_key.ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        )?)),
        9 => Ok(StableBslValueV1::Edge(decode_stable_key(
            &stable_key.ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        )?)),
        _ => Err(RustPersistenceRuntimeErrorV2::CampaignConflict),
    }
}

fn decode_stable_key(bytes: &[u8]) -> Result<StableElementKeyV1, RustPersistenceRuntimeErrorV2> {
    StableElementKeyV1::from_canonical_bytes(bytes)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

fn graph_row_count(graph: &StableGraphStateV1) -> Result<u32, RustPersistenceRuntimeErrorV2> {
    let rows = graph.rows();
    let count = [
        rows.nodes().len(),
        rows.node_f64().len(),
        rows.edges().len(),
        rows.hyperedges().len(),
        rows.edge_f64().len(),
        rows.node_currency().len(),
        rows.hyperedge_f64().len(),
    ]
    .into_iter()
    .try_fold(0_usize, usize::checked_add)
    .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    u32::try_from(count).map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

fn require_position(actual: i32, expected: usize) -> Result<(), RustPersistenceRuntimeErrorV2> {
    if usize::try_from(actual).ok() == Some(expected) {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
    }
}

fn require_i64_position(actual: i64, expected: usize) -> Result<(), RustPersistenceRuntimeErrorV2> {
    if usize::try_from(actual).ok() == Some(expected) {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
    }
}

fn decode_decimal<T>(row: &Row, index: usize) -> Result<T, RustPersistenceRuntimeErrorV2>
where
    T: std::str::FromStr,
{
    decode_column::<String>(row, index)?
        .parse()
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

fn unsigned_bits(value: i64) -> u64 {
    u64::from_be_bytes(value.to_be_bytes())
}

fn decode_column<T: postgres::types::FromSqlOwned>(
    row: &Row,
    index: usize,
) -> Result<T, RustPersistenceRuntimeErrorV2> {
    row.try_get(index)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

fn decode_digest(row: &Row, index: usize) -> Result<[u8; 32], RustPersistenceRuntimeErrorV2> {
    let bytes: Vec<u8> = decode_column(row, index)?;
    bytes
        .try_into()
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

fn database(operation: &'static str, error: &postgres::Error) -> RustPersistenceRuntimeErrorV2 {
    RustPersistenceRuntimeErrorV2::postgres(operation, error)
}
