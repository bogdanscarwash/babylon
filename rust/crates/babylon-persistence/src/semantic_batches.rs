//! Graph/event/choice/archive semantic batch composition for the V2 runtime.

use std::collections::TryReserveError;

use babylon_graph::stable_state::{StableGraphState, StableGraphStateRows};
use babylon_tick::choice_receipt::ChoiceReceipt;
use babylon_tick::material_state::{MaterialStateRowRef, MaterialStateRows};
use babylon_tick::replay_session::{IdentifiedTickReport, SuccessfulEvent, SuccessfulEventBatch};

use crate::committed_tick_envelope::{
    validate_committed_tick_envelope_bounds, CommittedTickEnvelopeError, CommittedTickRow,
    CommittedTickRowFamily, COMMITTED_TICK_ROW_FAMILY_COUNT, MAX_COMMITTED_TICK_ROW_BATCH_BYTES,
};
use crate::semantic_codec::{self, SemanticCodecError};

const ROW_LENGTH_BYTES: usize = 8;
const MINIMUM_NONEMPTY_ROW_BYTES: usize = ROW_LENGTH_BYTES + 1;

/// Source-owned proof that the stable graph contained no typed rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableGraphRowsEmptyProof {
    source_digest: [u8; 32],
}

/// Source-owned proof that the successful BSL event section contained no events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuccessfulEventBatchEmptyProof {
    source_digest: [u8; 32],
}

/// Source-owned proof that the ordered choice-receipt section was empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceReceiptBatchEmptyProof {
    source_digest: [u8; 32],
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SemanticBatchError {
    Codec(SemanticCodecError),
    Envelope(CommittedTickEnvelopeError),
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

impl From<SemanticCodecError> for SemanticBatchError {
    fn from(value: SemanticCodecError) -> Self {
        Self::Codec(value)
    }
}

impl From<CommittedTickEnvelopeError> for SemanticBatchError {
    fn from(value: CommittedTickEnvelopeError) -> Self {
        Self::Envelope(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SemanticFamilyBatch<P> {
    Rows {
        rows: Vec<CommittedTickRow>,
        body_bytes: usize,
    },
    Empty(P),
}

impl<P> SemanticFamilyBatch<P> {
    fn into_rows(self) -> Option<(Vec<CommittedTickRow>, usize)> {
        match self {
            Self::Rows { rows, body_bytes } => Some((rows, body_bytes)),
            Self::Empty(_) => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GraphEventChoiceSemanticBatches {
    graph: SemanticFamilyBatch<StableGraphRowsEmptyProof>,
    event: SemanticFamilyBatch<SuccessfulEventBatchEmptyProof>,
    choice_receipt: SemanticFamilyBatch<ChoiceReceiptBatchEmptyProof>,
}

impl GraphEventChoiceSemanticBatches {
    pub(crate) fn into_rows(
        self,
    ) -> (
        Vec<CommittedTickRow>,
        Vec<CommittedTickRow>,
        Vec<CommittedTickRow>,
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

pub(crate) fn compose_material_state_rows(
    source: &MaterialStateRows,
) -> Result<Vec<CommittedTickRow>, SemanticBatchError> {
    let mut rows = reserve_rows("material state semantic rows", source.source_count())?;
    let mut body_bytes = 0_usize;
    for source_row in source.rows() {
        let row = match source_row {
            MaterialStateRowRef::WorldRegister(row) => {
                semantic_codec::encode_world_register(row.qname(), row.value())?
            }
            MaterialStateRowRef::Territory(row) => {
                let fields = row
                    .ordered_fields()
                    .iter()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect::<Vec<_>>();
                semantic_codec::encode_territory_state(row.territory_id(), &fields)?
            }
            MaterialStateRowRef::DynamicHex(row) => {
                let values = row.value_bits().map(f64::from_bits);
                semantic_codec::encode_dynamic_hex_state(row.cell_id().as_u64(), &values)?
            }
            MaterialStateRowRef::Organization(row) => {
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
        push_encoded_row(
            CommittedTickRowFamily::State,
            &mut rows,
            &mut body_bytes,
            row,
        )?;
    }
    rows.sort_unstable_by(|left, right| left.key().cmp(right.key()));
    Ok(rows)
}

pub(crate) fn compose_graph_event_choice_semantic_batches(
    report: &IdentifiedTickReport,
) -> Result<GraphEventChoiceSemanticBatches, SemanticBatchError> {
    compose_graph_event_choice_sources(
        report.result_stable_graph(),
        report.successful_event_batch(),
        &report.report().choice_receipts,
        report.choice_receipt_source_digest(),
        *report.tick_content_hash().as_bytes(),
    )
}

fn compose_graph_event_choice_sources(
    graph_source: &StableGraphState,
    event_source: &SuccessfulEventBatch,
    choice_source: &[ChoiceReceipt],
    choice_source_digest: [u8; 32],
    tick_content_hash: [u8; 32],
) -> Result<GraphEventChoiceSemanticBatches, SemanticBatchError> {
    let graph_count = stable_graph_source_count(graph_source.rows())?;
    let event_count = event_source.events().len();
    let choice_count = choice_source.len();
    let archive_dirty_receipt = semantic_codec::encode_archive_dirty_receipt(&tick_content_hash)?;
    let archive_body_bytes = row_body_bytes(&archive_dirty_receipt)?;
    preflight_graph_event_choice_counts(
        graph_count,
        event_count,
        choice_count,
        archive_body_bytes,
    )?;

    let (graph_rows, graph_body_bytes) =
        compose_graph_rows_with_encoder(graph_source.rows(), &mut |row: StableGraphRowRef<'_>| {
            row.encode()
        })?;
    let (event_rows, event_body_bytes) = compose_event_rows(event_source)?;
    let (choice_rows, choice_body_bytes) = compose_choice_receipt_rows(choice_source)?;
    preflight_graph_event_choice_bounds(
        graph_rows.len(),
        graph_body_bytes,
        event_rows.len(),
        event_body_bytes,
        choice_rows.len(),
        choice_body_bytes,
        archive_body_bytes,
    )?;

    let graph = if graph_rows.is_empty() {
        SemanticFamilyBatch::Empty(StableGraphRowsEmptyProof {
            source_digest: graph_source.digest().into_bytes(),
        })
    } else {
        SemanticFamilyBatch::Rows {
            rows: graph_rows,
            body_bytes: graph_body_bytes,
        }
    };
    let event = if event_rows.is_empty() {
        SemanticFamilyBatch::Empty(SuccessfulEventBatchEmptyProof {
            source_digest: event_source.source_digest(),
        })
    } else {
        SemanticFamilyBatch::Rows {
            rows: event_rows,
            body_bytes: event_body_bytes,
        }
    };
    let choice_receipt = if choice_rows.is_empty() {
        SemanticFamilyBatch::Empty(ChoiceReceiptBatchEmptyProof {
            source_digest: choice_source_digest,
        })
    } else {
        SemanticFamilyBatch::Rows {
            rows: choice_rows,
            body_bytes: choice_body_bytes,
        }
    };
    Ok(GraphEventChoiceSemanticBatches {
        graph,
        event,
        choice_receipt,
    })
}

fn stable_graph_source_count(source: &StableGraphStateRows) -> Result<usize, SemanticBatchError> {
    let counts = [
        source.nodes().len(),
        source.node_f64().len(),
        source.edges().len(),
        source.hyperedges().len(),
        source.edge_f64().len(),
        source.node_currency().len(),
        source.hyperedge_f64().len(),
    ];
    checked_sum(&counts, "stable graph source rows")
}

fn preflight_graph_event_choice_counts(
    graph_rows: usize,
    event_rows: usize,
    choice_rows: usize,
    archive_body_bytes: usize,
) -> Result<(), SemanticBatchError> {
    let graph_minimum = graph_rows.checked_mul(MINIMUM_NONEMPTY_ROW_BYTES).ok_or(
        SemanticBatchError::CapacityOverflow {
            field: "stable graph minimum row bytes",
        },
    )?;
    let event_minimum = event_rows.checked_mul(MINIMUM_NONEMPTY_ROW_BYTES).ok_or(
        SemanticBatchError::CapacityOverflow {
            field: "successful event minimum row bytes",
        },
    )?;
    let choice_minimum = choice_rows.checked_mul(MINIMUM_NONEMPTY_ROW_BYTES).ok_or(
        SemanticBatchError::CapacityOverflow {
            field: "choice receipt minimum row bytes",
        },
    )?;
    preflight_graph_event_choice_bounds(
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

pub(crate) fn preflight_graph_event_choice_bounds(
    graph_rows: usize,
    graph_body_bytes: usize,
    event_rows: usize,
    event_body_bytes: usize,
    choice_rows: usize,
    choice_body_bytes: usize,
    archive_body_bytes: usize,
) -> Result<usize, SemanticBatchError> {
    let mut row_counts = [0_usize; COMMITTED_TICK_ROW_FAMILY_COUNT];
    row_counts[0] = graph_rows;
    row_counts[2] = event_rows;
    row_counts[3] = choice_rows;
    row_counts[5] = 1;
    let mut body_bytes = [0_usize; COMMITTED_TICK_ROW_FAMILY_COUNT];
    body_bytes[0] = graph_body_bytes;
    body_bytes[2] = event_body_bytes;
    body_bytes[3] = choice_body_bytes;
    body_bytes[5] = archive_body_bytes;
    validate_committed_tick_envelope_bounds(row_counts, body_bytes).map_err(Into::into)
}

fn row_body_bytes(row: &CommittedTickRow) -> Result<usize, SemanticBatchError> {
    ROW_LENGTH_BYTES
        .checked_add(row.key().len())
        .and_then(|value| value.checked_add(row.payload().len()))
        .ok_or(SemanticBatchError::CapacityOverflow {
            field: "semantic batch row body",
        })
}

pub(crate) enum StableGraphRowRef<'a> {
    Node(&'a str, &'a str),
    NodeF64(&'a str, &'a str, f64),
    Edge(&'a str, &'a str, &'a str, f64),
    Hyperedge(&'a str, &'a str, &'a [String]),
    EdgeF64(&'a str, &'a str, &'a str, &'a str, f64),
    NodeCurrency(&'a str, &'a str, i128),
    HyperedgeF64(&'a str, &'a str, f64),
}

impl StableGraphRowRef<'_> {
    pub(crate) fn encode(self) -> Result<CommittedTickRow, SemanticCodecError> {
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

pub(crate) fn compose_graph_rows_with_encoder(
    source: &StableGraphStateRows,
    encode: &mut impl FnMut(StableGraphRowRef<'_>) -> Result<CommittedTickRow, SemanticCodecError>,
) -> Result<(Vec<CommittedTickRow>, usize), SemanticBatchError> {
    let count = stable_graph_source_count(source)?;
    let mut rows = reserve_rows("stable graph semantic rows", count)?;
    let mut body_bytes = 0_usize;
    for (local_name, node_type) in source.nodes() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::Node(local_name, node_type))?,
        )?;
    }
    for (local_name, qname, bits) in source.node_f64() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::NodeF64(
                local_name,
                qname,
                f64::from_bits(*bits),
            ))?,
        )?;
    }
    for (edge_type, source, target, strength_bits) in source.edges() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::Edge(
                edge_type,
                source,
                target,
                f64::from_bits(*strength_bits),
            ))?,
        )?;
    }
    for (local_name, hyperedge_type, members) in source.hyperedges() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::Hyperedge(
                local_name,
                hyperedge_type,
                members,
            ))?,
        )?;
    }
    for (edge_type, source, target, qname, bits) in source.edge_f64() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::EdgeF64(
                edge_type,
                source,
                target,
                qname,
                f64::from_bits(*bits),
            ))?,
        )?;
    }
    for (local_name, qname, micro_units) in source.node_currency() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::NodeCurrency(
                local_name,
                qname,
                *micro_units,
            ))?,
        )?;
    }
    for (local_name, qname, bits) in source.hyperedge_f64() {
        push_encoded_row(
            CommittedTickRowFamily::Graph,
            &mut rows,
            &mut body_bytes,
            encode(StableGraphRowRef::HyperedgeF64(
                local_name,
                qname,
                f64::from_bits(*bits),
            ))?,
        )?;
    }
    rows.sort_unstable_by(|left, right| left.key().cmp(right.key()));
    Ok((rows, body_bytes))
}

fn compose_event_rows(
    source: &SuccessfulEventBatch,
) -> Result<(Vec<CommittedTickRow>, usize), SemanticBatchError> {
    let mut rows = reserve_rows("successful event semantic rows", source.events().len())?;
    let mut body_bytes = 0_usize;
    for (index, event) in source.events().iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| SemanticBatchError::IntegerConversion {
            field: "successful event ordinal",
            value: index,
        })?;
        let row = encode_successful_event(ordinal, event)?;
        push_encoded_row(
            CommittedTickRowFamily::Event,
            &mut rows,
            &mut body_bytes,
            row,
        )?;
    }
    Ok((rows, body_bytes))
}

fn encode_successful_event(
    ordinal: u32,
    event: &SuccessfulEvent,
) -> Result<CommittedTickRow, SemanticBatchError> {
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(event.fields().len())
        .map_err(|_: TryReserveError| SemanticBatchError::Allocation {
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
            .map(babylon_tick::choice_receipt::ChoiceReceiptRef::encounter_ordinal),
        event.event_type(),
        &fields,
    )
    .map_err(Into::into)
}

fn compose_choice_receipt_rows(
    source: &[ChoiceReceipt],
) -> Result<(Vec<CommittedTickRow>, usize), SemanticBatchError> {
    let mut rows = reserve_rows("choice receipt semantic rows", source.len())?;
    let mut body_bytes = 0_usize;
    for receipt in source {
        let branches = receipt
            .branches()
            .iter()
            .map(|branch| semantic_codec::ChoiceReceiptSemanticBranch {
                outcome_member: branch.member.clone(),
                mass_nanounits: branch.mass.nanounits(),
                ticket_start: branch.tickets.start,
                ticket_end_exclusive: branch.tickets.end,
                ticket_count: branch.tickets.count,
            })
            .collect();
        let semantic = semantic_codec::ChoiceReceiptSemanticRow {
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
        push_encoded_row(
            CommittedTickRowFamily::ChoiceReceipt,
            &mut rows,
            &mut body_bytes,
            row,
        )?;
    }
    rows.sort_unstable_by(|left, right| left.key().cmp(right.key()));
    Ok((rows, body_bytes))
}

fn reserve_rows(
    field: &'static str,
    count: usize,
) -> Result<Vec<CommittedTickRow>, SemanticBatchError> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_: TryReserveError| SemanticBatchError::Allocation {
            field,
            requested: count,
        })?;
    Ok(rows)
}

fn push_encoded_row(
    family: CommittedTickRowFamily,
    rows: &mut Vec<CommittedTickRow>,
    body_bytes: &mut usize,
    row: CommittedTickRow,
) -> Result<(), SemanticBatchError> {
    let next_body_bytes = body_bytes
        .checked_add(ROW_LENGTH_BYTES)
        .and_then(|value| value.checked_add(row.key().len()))
        .and_then(|value| value.checked_add(row.payload().len()))
        .ok_or(SemanticBatchError::CapacityOverflow {
            field: "semantic batch row body",
        })?;
    if next_body_bytes > MAX_COMMITTED_TICK_ROW_BATCH_BYTES {
        return Err(SemanticBatchError::Envelope(
            CommittedTickEnvelopeError::BatchBytes {
                family,
                actual: next_body_bytes,
                maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES,
            },
        ));
    }
    *body_bytes = next_body_bytes;
    rows.push(row);
    Ok(())
}

fn checked_sum(values: &[usize], field: &'static str) -> Result<usize, SemanticBatchError> {
    values.iter().try_fold(0_usize, |total, value| {
        total
            .checked_add(*value)
            .ok_or(SemanticBatchError::CapacityOverflow { field })
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use babylon_bsl::canonical_ast::rules_hash_of;
    use babylon_bsl::rule_pipeline::split_content;
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_kernel::content_digest::ContentDigest;
    use babylon_kernel::replay::{ReplaySeed, ReplaySessionId};
    use babylon_kernel::tick_content_hash::RefDigest;
    use babylon_practice_contract::OrderedPracticeActionBatch;
    use babylon_tick::material_state::MaterialState;
    use babylon_tick::replay_session::{IdentifiedTickReport, ReplayTickSession};

    use crate::committed_tick_envelope::{
        compose_row_families, CommittedTickEnvelopeError, CommittedTickRow,
        CommittedTickRowFamilies, CommittedTickRowFamily, MAX_COMMITTED_TICK_ROWS,
        MAX_COMMITTED_TICK_ROW_BATCH_BYTES,
    };
    use crate::michigan_dynamic_hex_foundation;
    use crate::semantic_codec::SemanticCodecError;

    use super::{
        compose_graph_event_choice_semantic_batches, compose_graph_rows_with_encoder,
        preflight_graph_event_choice_bounds, push_encoded_row, SemanticBatchError,
        SemanticFamilyBatch, StableGraphRowRef, StableGraphRowsEmptyProof,
        SuccessfulEventBatchEmptyProof,
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

    fn report_for(scenario: &str, session_name: &str) -> IdentifiedTickReport {
        let (_, rules) = split_content(RULE).expect("test rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: [0x21; 32],
            rules_hash: rules_hash_of(&forms).expect("test rule hashes"),
        };
        let session_id = ReplaySessionId::try_from(session_name).expect("test session id");
        let foundation =
            michigan_dynamic_hex_foundation().expect("governed foundation must decode once");
        let mut session = ReplayTickSession::new(
            scenario,
            None,
            RULE,
            HypergraphStore::new(),
            session_id.clone(),
            ReplaySeed::new(41),
            content,
            RefDigest::from_bytes(foundation.reference_bundle_digest()),
            MaterialState::try_new(foundation)
                .expect("test material state requires the exact governed dynamic-H3 foundation"),
        )
        .expect("test replay prepares");
        let actions =
            OrderedPracticeActionBatch::empty(session_id, 1).expect("test empty action batch");
        session
            .advance(&mut CollectingSink::default(), &actions)
            .expect("test replay advances")
    }

    fn report() -> IdentifiedTickReport {
        report_for(SCENARIO, "per281/semantic-batches")
    }

    #[test]
    fn proof_types_are_send_and_only_true_empty_sources_produce_them() {
        fn assert_send<T: Send>() {}
        assert_send::<StableGraphRowsEmptyProof>();
        assert_send::<SuccessfulEventBatchEmptyProof>();

        let empty_report = report_for(EMPTY_SCENARIO, "per281/semantic-batches-empty");
        let empty = compose_graph_event_choice_semantic_batches(&empty_report)
            .expect("true typed empty sources compose");
        let SemanticFamilyBatch::Empty(graph_proof) = empty.graph else {
            panic!("true empty graph source must produce its typed proof")
        };
        assert_eq!(
            graph_proof.source_digest,
            empty_report.result_stable_graph().digest().into_bytes()
        );
        let SemanticFamilyBatch::Empty(event_proof) = empty.event else {
            panic!("true empty event source must produce its typed proof")
        };
        assert_eq!(
            event_proof.source_digest,
            empty_report.successful_event_batch().source_digest()
        );

        let nonempty = compose_graph_event_choice_semantic_batches(&report())
            .expect("real report composes graph and event rows");
        assert!(matches!(nonempty.graph, SemanticFamilyBatch::Rows { .. }));
        assert!(matches!(nonempty.event, SemanticFamilyBatch::Rows { .. }));
    }

    #[test]
    fn preflight_accepts_exact_count_body_and_aggregate_bounds_and_refuses_plus_one() {
        let minimum_body = MAX_COMMITTED_TICK_ROWS * 9;
        preflight_graph_event_choice_bounds(
            MAX_COMMITTED_TICK_ROWS - 1,
            minimum_body - 9,
            0,
            0,
            0,
            0,
            9,
        )
        .expect("exact aggregate row ceiling");
        assert!(matches!(
            preflight_graph_event_choice_bounds(
                MAX_COMMITTED_TICK_ROWS,
                minimum_body,
                0,
                0,
                0,
                0,
                9,
            ),
            Err(SemanticBatchError::Envelope(
                CommittedTickEnvelopeError::AggregateRows { .. }
            ))
        ));
        preflight_graph_event_choice_bounds(1, MAX_COMMITTED_TICK_ROW_BATCH_BYTES, 0, 0, 0, 0, 9)
            .expect("exact graph body ceiling");
        assert!(matches!(
            preflight_graph_event_choice_bounds(
                1,
                MAX_COMMITTED_TICK_ROW_BATCH_BYTES + 1,
                0,
                0,
                0,
                0,
                9,
            ),
            Err(SemanticBatchError::Envelope(
                CommittedTickEnvelopeError::BatchBytes { .. }
            ))
        ));
    }

    #[test]
    fn incremental_family_byte_plus_one_refuses_before_row_publication() {
        let row = crate::semantic_codec::encode_stable_graph_node("class-a", "SOCIAL_CLASS")
            .expect("small graph row encodes");
        let row_bytes = row_body_bytes(&row);
        let mut body_bytes = MAX_COMMITTED_TICK_ROW_BATCH_BYTES - row_bytes + 1;
        let body_before = body_bytes;
        let mut rows = Vec::new();

        assert_eq!(
            push_encoded_row(
                CommittedTickRowFamily::Graph,
                &mut rows,
                &mut body_bytes,
                row,
            ),
            Err(SemanticBatchError::Envelope(
                CommittedTickEnvelopeError::BatchBytes {
                    family: CommittedTickRowFamily::Graph,
                    actual: MAX_COMMITTED_TICK_ROW_BATCH_BYTES + 1,
                    maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES,
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
        let result = compose_graph_rows_with_encoder(
            report.result_stable_graph().rows(),
            &mut |row: StableGraphRowRef<'_>| {
                let call = calls.get();
                calls.set(call + 1);
                if call == 1 {
                    return Err(SemanticCodecError::Invalid("injected second row"));
                }
                row.encode()
            },
        );
        assert_eq!(calls.get(), 2);
        assert_eq!(
            result,
            Err(SemanticBatchError::Codec(SemanticCodecError::Invalid(
                "injected second row"
            )))
        );
    }

    #[test]
    fn real_report_rows_are_vector_compatible_ordered_and_envelope_accepted() {
        let report = report();
        let batches = compose_graph_event_choice_semantic_batches(&report)
            .expect("real report semantic batches");
        let (graph, graph_body_bytes) = batches.graph.into_rows().expect("graph rows");
        let (event, event_body_bytes) = batches.event.into_rows().expect("event rows");
        let archive_dirty_receipt = crate::semantic_codec::encode_archive_dirty_receipt(
            report.tick_content_hash().as_bytes(),
        )
        .expect("current report receipt");
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

        compose_row_families(CommittedTickRowFamilies {
            graph,
            state: vec![],
            event,
            choice_receipt: vec![],
            checkpoint: vec![],
            archive_dirty_receipt,
        })
        .expect("existing envelope laws accept both ordered families");
    }

    #[test]
    fn encoded_graph_key_order_is_canonical_across_variable_text_lengths() {
        let report = report_for(
            VARIABLE_KEY_LENGTH_SCENARIO,
            "per281/semantic-batches-variable-keys",
        );
        let batches = compose_graph_event_choice_semantic_batches(&report)
            .expect("variable-length graph keys compose");
        let (graph, _) = batches.graph.into_rows().expect("graph rows");
        assert!(graph.windows(2).all(|rows| rows[0].key() < rows[1].key()));
    }

    fn row_body_bytes(row: &CommittedTickRow) -> usize {
        8 + row.key().len() + row.payload().len()
    }
}
