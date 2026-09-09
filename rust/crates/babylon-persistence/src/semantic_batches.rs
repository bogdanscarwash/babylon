//! Graph/event/choice/archive semantic batch composition for the V2 runtime.

use std::collections::TryReserveError;

use babylon_graph::stable_state::{StableGraphStateRowsV1, StableGraphStateV1};
use babylon_tick::choice_receipt::ChoiceReceiptV1;
use babylon_tick::material_state::{MaterialStateRowRefV1, MaterialStateRowsV1};
use babylon_tick::replay_session::{
    IdentifiedTickReportV2, SuccessfulEventBatchV2, SuccessfulEventV2,
};

use crate::committed_tick_envelope::{
    validate_committed_tick_envelope_bounds_v2, CommittedTickEnvelopeErrorV2,
    CommittedTickRowFamilyV2, CommittedTickRowV2, COMMITTED_TICK_ROW_FAMILY_COUNT_V2,
    MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
};
use crate::semantic_codec::{self, SemanticCodecErrorV1};

const ROW_LENGTH_BYTES: usize = 8;
const MINIMUM_NONEMPTY_ROW_BYTES: usize = ROW_LENGTH_BYTES + 1;

/// Source-owned proof that the stable graph contained no typed rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableGraphRowsEmptyProofV1 {
    source_digest: [u8; 32],
}

/// Source-owned proof that the successful BSL event section contained no events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuccessfulEventBatchEmptyProofV2 {
    source_digest: [u8; 32],
}

/// Source-owned proof that the ordered choice-receipt section was empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceReceiptBatchEmptyProofV1 {
    source_digest: [u8; 32],
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SemanticBatchErrorV2 {
    Codec(SemanticCodecErrorV1),
    Envelope(CommittedTickEnvelopeErrorV2),
    CapacityOverflow {
        field: &'static str,
    },
    IntegerConversion {
        field: &'static str,
        value: usize,
    },
    Allocation {
        field: &'static str,
        requested: usize,
    },
}

impl From<SemanticCodecErrorV1> for SemanticBatchErrorV2 {
    fn from(value: SemanticCodecErrorV1) -> Self {
        Self::Codec(value)
    }
}

impl From<CommittedTickEnvelopeErrorV2> for SemanticBatchErrorV2 {
    fn from(value: CommittedTickEnvelopeErrorV2) -> Self {
        Self::Envelope(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SemanticFamilyBatchV2<P> {
    Rows {
        rows: Vec<CommittedTickRowV2>,
        body_bytes: usize,
    },
    Empty(P),
}

impl<P> SemanticFamilyBatchV2<P> {
    fn into_rows(self) -> Option<(Vec<CommittedTickRowV2>, usize)> {
        match self {
            Self::Rows { rows, body_bytes } => Some((rows, body_bytes)),
            Self::Empty(_) => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GraphEventChoiceSemanticBatchesV2 {
    graph: SemanticFamilyBatchV2<StableGraphRowsEmptyProofV1>,
    event: SemanticFamilyBatchV2<SuccessfulEventBatchEmptyProofV2>,
    choice_receipt: SemanticFamilyBatchV2<ChoiceReceiptBatchEmptyProofV1>,
    archive_dirty_receipt: CommittedTickRowV2,
}

impl GraphEventChoiceSemanticBatchesV2 {
    pub(crate) fn graph_row_count(&self) -> usize {
        match &self.graph {
            SemanticFamilyBatchV2::Rows { rows, .. } => rows.len(),
            SemanticFamilyBatchV2::Empty(_) => 0,
        }
    }

    pub(crate) fn event_row_count(&self) -> usize {
        match &self.event {
            SemanticFamilyBatchV2::Rows { rows, .. } => rows.len(),
            SemanticFamilyBatchV2::Empty(_) => 0,
        }
    }

    pub(crate) fn choice_receipt_row_count(&self) -> usize {
        match &self.choice_receipt {
            SemanticFamilyBatchV2::Rows { rows, .. } => rows.len(),
            SemanticFamilyBatchV2::Empty(_) => 0,
        }
    }

    pub(crate) fn into_rows(
        self,
    ) -> (
        Vec<CommittedTickRowV2>,
        Vec<CommittedTickRowV2>,
        Vec<CommittedTickRowV2>,
    ) {
        let graph = self.graph.into_rows().map_or_else(Vec::new, |rows| rows.0);
        let event = self.event.into_rows().map_or_else(Vec::new, |rows| rows.0);
        let choice_receipt = self
            .choice_receipt
            .into_rows()
            .map_or_else(Vec::new, |rows| rows.0);
        (graph, event, choice_receipt)
    }
}

pub(crate) fn compose_material_state_rows_v1(
    source: &MaterialStateRowsV1,
) -> Result<Vec<CommittedTickRowV2>, SemanticBatchErrorV2> {
    let mut rows = reserve_rows_v1("material state semantic rows", source.source_count())?;
    let mut body_bytes = 0_usize;
    for source_row in source.rows() {
        let row = match source_row {
            MaterialStateRowRefV1::WorldRegister(row) => {
                semantic_codec::encode_world_register(row.qname(), row.value())?
            }
            MaterialStateRowRefV1::Territory(row) => {
                let fields = row
                    .ordered_fields()
                    .iter()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect::<Vec<_>>();
                semantic_codec::encode_territory_state(row.territory_id(), &fields)?
            }
            MaterialStateRowRefV1::DynamicHex(row) => {
                let values = row.value_bits().map(f64::from_bits);
                semantic_codec::encode_dynamic_hex_state(row.cell_id().as_u64(), &values)?
            }
            MaterialStateRowRefV1::Organization(row) => {
                let fields = row
                    .ordered_fields()
                    .iter()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect::<Vec<_>>();
                semantic_codec::encode_organization_state(
                    row.organization_id(),
                    row.organization_kind(),
                    row.ordered_territory_ids(),
                    &fields,
                )?
            }
        };
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::State,
            &mut rows,
            &mut body_bytes,
            row,
        )?;
    }
    rows.sort_unstable_by(|left, right| left.key().cmp(right.key()));
    Ok(rows)
}

pub(crate) fn compose_graph_event_choice_semantic_batches_v2(
    report: &IdentifiedTickReportV2,
) -> Result<GraphEventChoiceSemanticBatchesV2, SemanticBatchErrorV2> {
    compose_graph_event_choice_sources_v2(
        report.result_stable_graph(),
        report.successful_event_batch(),
        &report.report().choice_receipts,
        report.choice_receipt_source_digest(),
        *report.tick_content_hash().as_bytes(),
    )
}

fn compose_graph_event_choice_sources_v2(
    graph_source: &StableGraphStateV1,
    event_source: &SuccessfulEventBatchV2,
    choice_source: &[ChoiceReceiptV1],
    choice_source_digest: [u8; 32],
    tick_content_hash: [u8; 32],
) -> Result<GraphEventChoiceSemanticBatchesV2, SemanticBatchErrorV2> {
    let graph_count = stable_graph_source_count_v1(graph_source.rows())?;
    let event_count = event_source.events().len();
    let choice_count = choice_source.len();
    let archive_dirty_receipt = semantic_codec::encode_archive_dirty_receipt(&tick_content_hash)?;
    let archive_body_bytes = row_body_bytes_v1(&archive_dirty_receipt)?;
    preflight_graph_event_choice_counts_v2(
        graph_count,
        event_count,
        choice_count,
        archive_body_bytes,
    )?;

    let (graph_rows, graph_body_bytes) = compose_graph_rows_with_encoder_v1(
        graph_source.rows(),
        &mut |row: StableGraphRowRefV1<'_>| row.encode(),
    )?;
    let (event_rows, event_body_bytes) = compose_event_rows_v2(event_source)?;
    let (choice_rows, choice_body_bytes) = compose_choice_receipt_rows_v1(choice_source)?;
    preflight_graph_event_choice_bounds_v2(
        graph_rows.len(),
        graph_body_bytes,
        event_rows.len(),
        event_body_bytes,
        choice_rows.len(),
        choice_body_bytes,
        archive_body_bytes,
    )?;

    let graph = if graph_rows.is_empty() {
        SemanticFamilyBatchV2::Empty(StableGraphRowsEmptyProofV1 {
            source_digest: graph_source.digest().into_bytes(),
        })
    } else {
        SemanticFamilyBatchV2::Rows {
            rows: graph_rows,
            body_bytes: graph_body_bytes,
        }
    };
    let event = if event_rows.is_empty() {
        SemanticFamilyBatchV2::Empty(SuccessfulEventBatchEmptyProofV2 {
            source_digest: event_source.source_digest(),
        })
    } else {
        SemanticFamilyBatchV2::Rows {
            rows: event_rows,
            body_bytes: event_body_bytes,
        }
    };
    let choice_receipt = if choice_rows.is_empty() {
        SemanticFamilyBatchV2::Empty(ChoiceReceiptBatchEmptyProofV1 {
            source_digest: choice_source_digest,
        })
    } else {
        SemanticFamilyBatchV2::Rows {
            rows: choice_rows,
            body_bytes: choice_body_bytes,
        }
    };
    Ok(GraphEventChoiceSemanticBatchesV2 {
        graph,
        event,
        choice_receipt,
        archive_dirty_receipt,
    })
}

fn stable_graph_source_count_v1(
    source: &StableGraphStateRowsV1,
) -> Result<usize, SemanticBatchErrorV2> {
    let counts = [
        source.nodes().len(),
        source.node_f64().len(),
        source.edges().len(),
        source.hyperedges().len(),
        source.edge_f64().len(),
        source.node_currency().len(),
        source.hyperedge_f64().len(),
    ];
    checked_sum_v1(&counts, "stable graph source rows")
}

fn preflight_graph_event_choice_counts_v2(
    graph_rows: usize,
    event_rows: usize,
    choice_rows: usize,
    archive_body_bytes: usize,
) -> Result<(), SemanticBatchErrorV2> {
    let graph_minimum = graph_rows.checked_mul(MINIMUM_NONEMPTY_ROW_BYTES).ok_or(
        SemanticBatchErrorV2::CapacityOverflow {
            field: "stable graph minimum row bytes",
        },
    )?;
    let event_minimum = event_rows.checked_mul(MINIMUM_NONEMPTY_ROW_BYTES).ok_or(
        SemanticBatchErrorV2::CapacityOverflow {
            field: "successful event minimum row bytes",
        },
    )?;
    let choice_minimum = choice_rows.checked_mul(MINIMUM_NONEMPTY_ROW_BYTES).ok_or(
        SemanticBatchErrorV2::CapacityOverflow {
            field: "choice receipt minimum row bytes",
        },
    )?;
    preflight_graph_event_choice_bounds_v2(
        graph_rows,
        graph_minimum,
        event_rows,
        event_minimum,
        choice_rows,
        choice_minimum,
        archive_body_bytes,
    )
    .map(|_| ())
}

pub(crate) fn preflight_graph_event_choice_bounds_v2(
    graph_rows: usize,
    graph_body_bytes: usize,
    event_rows: usize,
    event_body_bytes: usize,
    choice_rows: usize,
    choice_body_bytes: usize,
    archive_body_bytes: usize,
) -> Result<usize, SemanticBatchErrorV2> {
    let mut row_counts = [0_usize; COMMITTED_TICK_ROW_FAMILY_COUNT_V2];
    row_counts[0] = graph_rows;
    row_counts[2] = event_rows;
    row_counts[3] = choice_rows;
    row_counts[5] = 1;
    let mut body_bytes = [0_usize; COMMITTED_TICK_ROW_FAMILY_COUNT_V2];
    body_bytes[0] = graph_body_bytes;
    body_bytes[2] = event_body_bytes;
    body_bytes[3] = choice_body_bytes;
    body_bytes[5] = archive_body_bytes;
    validate_committed_tick_envelope_bounds_v2(row_counts, body_bytes).map_err(Into::into)
}

fn row_body_bytes_v1(row: &CommittedTickRowV2) -> Result<usize, SemanticBatchErrorV2> {
    ROW_LENGTH_BYTES
        .checked_add(row.key().len())
        .and_then(|value| value.checked_add(row.payload().len()))
        .ok_or(SemanticBatchErrorV2::CapacityOverflow {
            field: "semantic batch row body",
        })
}

pub(crate) enum StableGraphRowRefV1<'a> {
    Node(&'a str, &'a str),
    NodeF64(&'a str, &'a str, f64),
    Edge(&'a str, &'a str, &'a str, f64),
    Hyperedge(&'a str, &'a str, &'a [String]),
    EdgeF64(&'a str, &'a str, &'a str, &'a str, f64),
    NodeCurrency(&'a str, &'a str, i128),
    HyperedgeF64(&'a str, &'a str, f64),
}

impl StableGraphRowRefV1<'_> {
    pub(crate) fn encode(self) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
        match self {
            Self::Node(local_name, node_type) => {
                semantic_codec::encode_stable_graph_node(local_name, node_type)
            }
            Self::NodeF64(local_name, qname, bits) => semantic_codec::encode_stable_graph_node_f64(
                local_name,
                qname,
                f64::from_bits(bits.to_bits()),
            ),
            Self::Edge(edge_type, source, target, strength) => {
                semantic_codec::encode_stable_graph_edge(edge_type, source, target, strength)
            }
            Self::Hyperedge(local_name, hyperedge_type, members) => {
                semantic_codec::encode_stable_graph_hyperedge(local_name, hyperedge_type, members)
            }
            Self::EdgeF64(edge_type, source, target, qname, value) => {
                semantic_codec::encode_stable_graph_edge_f64(
                    edge_type, source, target, qname, value,
                )
            }
            Self::NodeCurrency(local_name, qname, micro_units) => {
                semantic_codec::encode_stable_graph_node_currency(local_name, qname, micro_units)
            }
            Self::HyperedgeF64(local_name, qname, value) => {
                semantic_codec::encode_stable_graph_hyperedge_f64(local_name, qname, value)
            }
        }
    }
}

pub(crate) fn compose_graph_rows_with_encoder_v1(
    source: &StableGraphStateRowsV1,
    encode: &mut impl FnMut(StableGraphRowRefV1<'_>) -> Result<CommittedTickRowV2, SemanticCodecErrorV1>,
) -> Result<(Vec<CommittedTickRowV2>, usize), SemanticBatchErrorV2> {
    let count = stable_graph_source_count_v1(source)?;
    let mut rows = reserve_rows_v1("stable graph semantic rows", count)?;
    let mut body_bytes = 0_usize;
    for (local_name, node_type) in source.nodes() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::Node(local_name, node_type))?,
        )?;
    }
    for (local_name, qname, bits) in source.node_f64() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::NodeF64(
                local_name,
                qname,
                f64::from_bits(*bits),
            ))?,
        )?;
    }
    for (edge_type, source, target, strength_bits) in source.edges() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::Edge(
                edge_type,
                source,
                target,
                f64::from_bits(*strength_bits),
            ))?,
        )?;
    }
    for (local_name, hyperedge_type, members) in source.hyperedges() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::Hyperedge(
                local_name,
                hyperedge_type,
                members,
            ))?,
        )?;
    }
    for (edge_type, source, target, qname, bits) in source.edge_f64() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::EdgeF64(
                edge_type,
                source,
                target,
                qname,
                f64::from_bits(*bits),
            ))?,
        )?;
    }
    for (local_name, qname, micro_units) in source.node_currency() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::NodeCurrency(
                local_name,
                qname,
                *micro_units,
            ))?,
        )?;
    }
    for (local_name, qname, bits) in source.hyperedge_f64() {
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRefV1::HyperedgeF64(
                local_name,
                qname,
                f64::from_bits(*bits),
            ))?,
        )?;
    }
    rows.sort_unstable_by(|left, right| left.key().cmp(right.key()));
    Ok((rows, body_bytes))
}

fn compose_event_rows_v2(
    source: &SuccessfulEventBatchV2,
) -> Result<(Vec<CommittedTickRowV2>, usize), SemanticBatchErrorV2> {
    let mut rows = reserve_rows_v1("successful event semantic rows", source.events().len())?;
    let mut body_bytes = 0_usize;
    for (index, event) in source.events().iter().enumerate() {
        let ordinal =
            u32::try_from(index).map_err(|_| SemanticBatchErrorV2::IntegerConversion {
                field: "successful event ordinal",
                value: index,
            })?;
        let row = encode_successful_event_v2(ordinal, event)?;
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::Event,
            &mut rows,
            &mut body_bytes,
            row,
        )?;
    }
    Ok((rows, body_bytes))
}

fn encode_successful_event_v2(
    ordinal: u32,
    event: &SuccessfulEventV2,
) -> Result<CommittedTickRowV2, SemanticBatchErrorV2> {
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(event.fields().len())
        .map_err(|_: TryReserveError| SemanticBatchErrorV2::Allocation {
            field: "successful event semantic fields",
            requested: event.fields().len(),
        })?;
    for (name, value) in event.fields() {
        fields.push((name.as_str(), value));
    }
    semantic_codec::encode_successful_event(
        ordinal,
        event.emitting_rule(),
        event
            .choice_receipt()
            .map(babylon_tick::choice_receipt::ChoiceReceiptRefV1::encounter_ordinal),
        event.event_type(),
        &fields,
    )
    .map_err(Into::into)
}

fn compose_choice_receipt_rows_v1(
    source: &[ChoiceReceiptV1],
) -> Result<(Vec<CommittedTickRowV2>, usize), SemanticBatchErrorV2> {
    let mut rows = reserve_rows_v1("choice receipt semantic rows", source.len())?;
    let mut body_bytes = 0_usize;
    for receipt in source {
        let branches = receipt
            .branches()
            .iter()
            .map(|branch| semantic_codec::ChoiceReceiptSemanticBranchV1 {
                outcome_member: branch.member.clone(),
                mass_nanounits: branch.mass.nanounits(),
                ticket_start: branch.tickets.start,
                ticket_end_exclusive: branch.tickets.end,
                ticket_count: branch.tickets.count,
            })
            .collect();
        let semantic = semantic_codec::ChoiceReceiptSemanticRowV1 {
            encounter_ordinal: receipt.encounter_ordinal(),
            rule_id: receipt.rule_id().to_owned(),
            sample: receipt.sample().to_owned(),
            slot: receipt.slot(),
            outcome_enum: receipt.outcome_enum().to_owned(),
            stable_carrier: receipt.stable_carrier().clone(),
            active_elements: receipt.active_elements().to_vec(),
            branches,
            draw_ticket: receipt.draw_ticket(),
            selected_outcome: receipt.selected_outcome().to_owned(),
            allocation_digest: receipt.allocation_digest(),
            instance_digest: receipt.instance_digest(),
        };
        let row = semantic_codec::encode_choice_receipt(&semantic)?;
        push_encoded_row_v1(
            CommittedTickRowFamilyV2::ChoiceReceipt,
            &mut rows,
            &mut body_bytes,
            row,
        )?;
    }
    rows.sort_unstable_by(|left, right| left.key().cmp(right.key()));
    Ok((rows, body_bytes))
}

fn reserve_rows_v1(
    field: &'static str,
    count: usize,
) -> Result<Vec<CommittedTickRowV2>, SemanticBatchErrorV2> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_: TryReserveError| SemanticBatchErrorV2::Allocation {
            field,
            requested: count,
        })?;
    Ok(rows)
}

fn push_encoded_row_v1(
    family: CommittedTickRowFamilyV2,
    rows: &mut Vec<CommittedTickRowV2>,
    body_bytes: &mut usize,
    row: CommittedTickRowV2,
) -> Result<(), SemanticBatchErrorV2> {
    let next_body_bytes = body_bytes
        .checked_add(ROW_LENGTH_BYTES)
        .and_then(|value| value.checked_add(row.key().len()))
        .and_then(|value| value.checked_add(row.payload().len()))
        .ok_or(SemanticBatchErrorV2::CapacityOverflow {
            field: "semantic batch row body",
        })?;
    if next_body_bytes > MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 {
        return Err(SemanticBatchErrorV2::Envelope(
            CommittedTickEnvelopeErrorV2::BatchBytes {
                family,
                actual: next_body_bytes,
                maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
            },
        ));
    }
    *body_bytes = next_body_bytes;
    rows.push(row);
    Ok(())
}

fn checked_sum_v1(values: &[usize], field: &'static str) -> Result<usize, SemanticBatchErrorV2> {
    values.iter().try_fold(0_usize, |total, value| {
        total
            .checked_add(*value)
            .ok_or(SemanticBatchErrorV2::CapacityOverflow { field })
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use babylon_bsl::rule_pipeline::split_content;
    use babylon_bsl::rules_hash_of;
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
    use babylon_kernel::tick_content_hash::{RefDigestV1, TickContentHashV1};
    use babylon_kernel::ContentDigest;
    use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
    use babylon_tick::material_state::MaterialStateV1;
    use babylon_tick::replay_session::{IdentifiedTickReportV2, ReplayTickSession};

    use crate::committed_tick_envelope::{
        CommittedTickEnvelopeErrorV2, CommittedTickEnvelopeV2, CommittedTickRowFamiliesV2,
        CommittedTickRowFamilyV2, CommittedTickRowV2, MAX_COMMITTED_TICK_ROWS_V2,
        MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
    };
    use crate::identity::CampaignId;
    use crate::michigan_dynamic_hex_foundation_v1;
    use crate::semantic_codec::SemanticCodecErrorV1;
    use crate::tick_commit_claim::TickCommitClaimV1;

    use super::{
        compose_graph_event_choice_semantic_batches_v2, compose_graph_rows_with_encoder_v1,
        preflight_graph_event_choice_bounds_v2, push_encoded_row_v1, SemanticBatchErrorV2,
        SemanticFamilyBatchV2, StableGraphRowRefV1, StableGraphRowsEmptyProofV1,
        SuccessfulEventBatchEmptyProofV2,
    };

    const SCENARIO: &str = r"
(scenario demo/persistence-batches
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/draw coefficient extensive)
  (node class-a NodeType/SOCIAL_CLASS (social-class/draw 0.0c))
  (node class-b NodeType/SOCIAL_CLASS (social-class/draw 0.0c)))
";
    const EMPTY_SCENARIO: &str = r"
(scenario demo/persistence-batches-empty
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/draw coefficient extensive))
";
    const VARIABLE_KEY_LENGTH_SCENARIO: &str = r"
(scenario demo/persistence-batches-variable-keys
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield a/attribute-with-a-long-name coefficient extensive)
  (deffield social-class/draw coefficient extensive)
  (deffield z/x coefficient extensive)
  (node class-a NodeType/SOCIAL_CLASS
    (a/attribute-with-a-long-name 0.0c)
    (social-class/draw 0.0c)
    (z/x 0.0c)))
";
    const RULE: &str = r#"
(rule production/typed-batch
  :role mechanic
  :evidence derived
  :material-basis "graph/event semantic batch composition law"
  :fuel 32
  (bindings (binding draw :field social-class/draw))
  (when #t)
  (effects
    (update-node self social-class/draw (set 0.25c))
    (emit EventType/PERSISTENCE_BATCH (subject self))))
"#;

    fn report_for(scenario: &str, session_name: &str) -> IdentifiedTickReportV2 {
        let (_, rules) = split_content(RULE).expect("test rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: [0x21; 32],
            rules_hash: rules_hash_of(&forms).expect("test rule hashes"),
        };
        let session_id = ReplaySessionIdV1::try_from(session_name).expect("test session id");
        let foundation =
            michigan_dynamic_hex_foundation_v1().expect("governed foundation must decode once");
        let mut session = ReplayTickSession::new(
            scenario,
            None,
            RULE,
            HypergraphStore::new(),
            session_id.clone(),
            ReplaySeed::new(41),
            content,
            RefDigestV1::from_bytes(foundation.reference_bundle_digest()),
            MaterialStateV1::try_new(foundation)
                .expect("test material state requires the exact governed dynamic-H3 foundation"),
        )
        .expect("test replay prepares");
        let actions =
            OrderedPracticeActionBatchV1::empty(session_id, 1).expect("test empty action batch");
        session
            .advance(&mut CollectingSink::default(), &actions)
            .expect("test replay advances")
    }

    fn report() -> IdentifiedTickReportV2 {
        report_for(SCENARIO, "per281/semantic-batches")
    }

    #[test]
    fn proof_types_are_send_and_only_true_empty_sources_produce_them() {
        fn assert_send<T: Send>() {}
        assert_send::<StableGraphRowsEmptyProofV1>();
        assert_send::<SuccessfulEventBatchEmptyProofV2>();

        let empty_report = report_for(EMPTY_SCENARIO, "per281/semantic-batches-empty");
        let empty = compose_graph_event_choice_semantic_batches_v2(&empty_report)
            .expect("true typed empty sources compose");
        let SemanticFamilyBatchV2::Empty(graph_proof) = empty.graph else {
            panic!("true empty graph source must produce its typed proof")
        };
        assert_eq!(
            graph_proof.source_digest,
            empty_report.result_stable_graph().digest().into_bytes()
        );
        let SemanticFamilyBatchV2::Empty(event_proof) = empty.event else {
            panic!("true empty event source must produce its typed proof")
        };
        assert_eq!(
            event_proof.source_digest,
            empty_report.successful_event_batch().source_digest()
        );

        let nonempty = compose_graph_event_choice_semantic_batches_v2(&report())
            .expect("real report composes graph and event rows");
        assert!(matches!(nonempty.graph, SemanticFamilyBatchV2::Rows { .. }));
        assert!(matches!(nonempty.event, SemanticFamilyBatchV2::Rows { .. }));
    }

    #[test]
    fn preflight_accepts_exact_count_body_and_aggregate_bounds_and_refuses_plus_one() {
        let minimum_body = MAX_COMMITTED_TICK_ROWS_V2 * 9;
        preflight_graph_event_choice_bounds_v2(
            MAX_COMMITTED_TICK_ROWS_V2 - 1,
            minimum_body - 9,
            0,
            0,
            0,
            0,
            9,
        )
        .expect("exact aggregate row ceiling");
        assert!(matches!(
            preflight_graph_event_choice_bounds_v2(
                MAX_COMMITTED_TICK_ROWS_V2,
                minimum_body,
                0,
                0,
                0,
                0,
                9,
            ),
            Err(SemanticBatchErrorV2::Envelope(
                CommittedTickEnvelopeErrorV2::AggregateRows { .. }
            ))
        ));
        preflight_graph_event_choice_bounds_v2(
            1,
            MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
            0,
            0,
            0,
            0,
            9,
        )
        .expect("exact graph body ceiling");
        assert!(matches!(
            preflight_graph_event_choice_bounds_v2(
                1,
                MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 + 1,
                0,
                0,
                0,
                0,
                9,
            ),
            Err(SemanticBatchErrorV2::Envelope(
                CommittedTickEnvelopeErrorV2::BatchBytes { .. }
            ))
        ));
    }

    #[test]
    fn incremental_family_byte_plus_one_refuses_before_row_publication() {
        let row = crate::semantic_codec::encode_stable_graph_node("class-a", "SOCIAL_CLASS")
            .expect("small graph row encodes");
        let row_bytes = row_body_bytes(&row);
        let mut body_bytes = MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 - row_bytes + 1;
        let body_before = body_bytes;
        let mut rows = Vec::new();

        assert_eq!(
            push_encoded_row_v1(
                CommittedTickRowFamilyV2::Graph,
                &mut rows,
                &mut body_bytes,
                row,
            ),
            Err(SemanticBatchErrorV2::Envelope(
                CommittedTickEnvelopeErrorV2::BatchBytes {
                    family: CommittedTickRowFamilyV2::Graph,
                    actual: MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 + 1,
                    maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
                }
            ))
        );
        assert!(rows.is_empty());
        assert_eq!(body_bytes, body_before);
    }

    #[test]
    fn second_row_codec_refusal_returns_only_error_without_partial_batch() {
        let report = report();
        let calls = Cell::new(0_usize);
        let result = compose_graph_rows_with_encoder_v1(
            report.result_stable_graph().rows(),
            &mut |row: StableGraphRowRefV1<'_>| {
                let call = calls.get();
                calls.set(call + 1);
                if call == 1 {
                    return Err(SemanticCodecErrorV1::Invalid("injected second row"));
                }
                row.encode()
            },
        );
        assert_eq!(calls.get(), 2);
        assert_eq!(
            result,
            Err(SemanticBatchErrorV2::Codec(SemanticCodecErrorV1::Invalid(
                "injected second row"
            )))
        );
    }

    #[test]
    fn real_report_rows_are_vector_compatible_ordered_and_envelope_accepted() {
        let report = report();
        let batches = compose_graph_event_choice_semantic_batches_v2(&report)
            .expect("real report semantic batches");
        let (graph, graph_body_bytes) = batches.graph.into_rows().expect("graph rows");
        let (event, event_body_bytes) = batches.event.into_rows().expect("event rows");
        let archive_dirty_receipt = batches.archive_dirty_receipt;
        assert!(!graph.is_empty());
        assert_eq!(event.len(), 2);
        assert_eq!(
            graph_body_bytes,
            graph.iter().map(row_body_bytes).sum::<usize>()
        );
        assert_eq!(
            event_body_bytes,
            event.iter().map(row_body_bytes).sum::<usize>()
        );
        assert!(graph.windows(2).all(|rows| rows[0].key() < rows[1].key()));
        assert!(event.windows(2).all(|rows| rows[0].key() < rows[1].key()));

        let claim = TickCommitClaimV1::compose(
            CampaignId::from_uuid(uuid::Uuid::nil()),
            1,
            TickContentHashV1::from_bytes([0x33; 32]),
        );
        CommittedTickEnvelopeV2::compose(
            claim,
            CommittedTickRowFamiliesV2 {
                graph,
                state: vec![],
                event,
                choice_receipt: vec![],
                checkpoint: vec![],
                archive_dirty_receipt,
            },
        )
        .expect("existing envelope laws accept both ordered families");
    }

    #[test]
    fn encoded_graph_key_order_is_canonical_across_variable_text_lengths() {
        let report = report_for(
            VARIABLE_KEY_LENGTH_SCENARIO,
            "per281/semantic-batches-variable-keys",
        );
        let batches = compose_graph_event_choice_semantic_batches_v2(&report)
            .expect("variable-length graph keys compose");
        let (graph, _) = batches.graph.into_rows().expect("graph rows");
        assert!(graph.windows(2).all(|rows| rows[0].key() < rows[1].key()));
    }

    fn row_body_bytes(row: &CommittedTickRowV2) -> usize {
        8 + row.key().len() + row.payload().len()
    }
}
