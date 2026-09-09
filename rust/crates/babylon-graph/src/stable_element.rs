//! Runtime-handle-free graph element identity and sealed topology resolution.

use std::collections::{BTreeMap, BTreeSet, HashMap, TryReserveError};

use babylon_kernel::sha256_of;

use crate::state_hash::CanonicalState;
use crate::substrate::{GraphSubstrate, HyperedgeId, NodeId};

/// Stable element layout version.
pub const STABLE_ELEMENT_LAYOUT_VERSION_V1: u32 = 1;
/// Stable resolver-manifest layout version.
pub const STABLE_ELEMENT_RESOLVER_MANIFEST_LAYOUT_VERSION_V1: u32 = 1;
/// Maximum resolved active-element stack depth in one V2 carrier.
pub const MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2: usize = 256;
/// Maximum canonical V2 carrier byte length.
pub const MAX_STABLE_CARRIER_BYTES_V2: usize = 105_962;
/// Maximum combined node and hyperedge rows in one resolver manifest.
pub const MAX_STABLE_RESOLVER_ROWS_V1: usize = 65_536;
/// Maximum members in one hyperedge while sealing a stable resolver.
pub const MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1: usize = 65_534;
/// Maximum topology rows plus member references while sealing a resolver.
pub const MAX_STABLE_RESOLVER_FACT_UNITS_V1: usize = 1_048_576;
/// Maximum canonical resolver-manifest byte length.
pub const MAX_STABLE_RESOLVER_MANIFEST_BYTES_V1: usize = 8_388_608;

const STABLE_ELEMENT_DOMAIN: &[u8] = b"babylon.stable-element";
const STABLE_RESOLVER_DOMAIN: &[u8] = b"babylon.stable-element-resolver";
const MAX_SYMBOL_BYTES: usize = 64;
const MAX_QNAME_BYTES: usize = 128;
const MAX_QNAME_SEGMENTS: usize = 4;
const MAX_STRUCTURAL_TYPE_BYTES: usize = 128;
const MAX_STABLE_EDGES_V1: usize = 65_536;

type StableNodeMaps = (
    HashMap<NodeId, StableElementKeyV1>,
    BTreeMap<String, NodeId>,
);
type StableHyperedgeMaps = (
    HashMap<HyperedgeId, StableElementKeyV1>,
    BTreeMap<String, HyperedgeId>,
);

/// A stable graph element, independent of runtime handle allocation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StableElementKeyV1 {
    /// One authored scenario node.
    Node {
        /// Scenario scope qname.
        scenario: String,
        /// Authored node local name.
        local_name: String,
    },
    /// One directed dyadic edge.
    Edge {
        /// Scenario scope qname.
        scenario: String,
        /// Structural edge type.
        edge_type: String,
        /// Authored source-node local name.
        source_local_name: String,
        /// Authored target-node local name.
        target_local_name: String,
    },
    /// One authored scenario hyperedge.
    Hyperedge {
        /// Scenario scope qname.
        scenario: String,
        /// Authored hyperedge local name.
        local_name: String,
    },
}

impl StableElementKeyV1 {
    /// Decode one exact standalone binary stable-element key.
    ///
    /// # Errors
    /// Returns a framing, UTF-8, semantic-string, arithmetic, or allocation
    /// refusal. The decoded value must re-encode byte-for-byte.
    pub fn from_canonical_bytes(input: &[u8]) -> Result<Self, StableIdentityError> {
        let header = STABLE_ELEMENT_DOMAIN.len() + 1 + size_of::<u32>() + 1;
        if input.len() < header
            || !input.starts_with(STABLE_ELEMENT_DOMAIN)
            || input.get(STABLE_ELEMENT_DOMAIN.len()) != Some(&0)
            || input.get(STABLE_ELEMENT_DOMAIN.len() + 1..STABLE_ELEMENT_DOMAIN.len() + 5)
                != Some(STABLE_ELEMENT_LAYOUT_VERSION_V1.to_be_bytes().as_slice())
        {
            return Err(StableIdentityError::CanonicalBytes {
                field: "stable element header",
            });
        }
        let mut cursor = STABLE_ELEMENT_DOMAIN.len() + 5;
        let tag = *input
            .get(cursor)
            .ok_or(StableIdentityError::CanonicalBytes {
                field: "stable element tag",
            })?;
        cursor += 1;
        let key = match tag {
            0x01 => Self::Node {
                scenario: read_canonical_string(input, &mut cursor, "stable node scenario")?,
                local_name: read_canonical_string(input, &mut cursor, "stable node local name")?,
            },
            0x02 => Self::Edge {
                scenario: read_canonical_string(input, &mut cursor, "stable edge scenario")?,
                edge_type: read_canonical_string(input, &mut cursor, "stable edge type")?,
                source_local_name: read_canonical_string(input, &mut cursor, "stable edge source")?,
                target_local_name: read_canonical_string(input, &mut cursor, "stable edge target")?,
            },
            0x03 => Self::Hyperedge {
                scenario: read_canonical_string(input, &mut cursor, "stable hyperedge scenario")?,
                local_name: read_canonical_string(
                    input,
                    &mut cursor,
                    "stable hyperedge local name",
                )?,
            },
            _ => {
                return Err(StableIdentityError::CanonicalBytes {
                    field: "stable element tag",
                });
            }
        };
        key.validate()?;
        if cursor != input.len() || key.canonical_bytes()?.as_slice() != input {
            return Err(StableIdentityError::CanonicalBytes {
                field: "stable element trailing bytes",
            });
        }
        Ok(key)
    }

    /// Copy this stable key using checked, fallible string allocation.
    ///
    /// # Errors
    /// Returns [`StableIdentityError::Allocation`] before exposing a partial
    /// key when any owned string cannot reserve its exact byte length.
    pub fn try_owned(&self) -> Result<Self, StableIdentityError> {
        Ok(match self {
            Self::Node {
                scenario,
                local_name,
            } => Self::Node {
                scenario: copy_string("stable node scenario", scenario)?,
                local_name: copy_string("stable node local name", local_name)?,
            },
            Self::Edge {
                scenario,
                edge_type,
                source_local_name,
                target_local_name,
            } => Self::Edge {
                scenario: copy_string("stable edge scenario", scenario)?,
                edge_type: copy_string("stable edge type", edge_type)?,
                source_local_name: copy_string("stable edge source", source_local_name)?,
                target_local_name: copy_string("stable edge target", target_local_name)?,
            },
            Self::Hyperedge {
                scenario,
                local_name,
            } => Self::Hyperedge {
                scenario: copy_string("stable hyperedge scenario", scenario)?,
                local_name: copy_string("stable hyperedge local name", local_name)?,
            },
        })
    }

    /// Encode the exact standalone binary stable-element key.
    ///
    /// # Errors
    /// Returns a semantic string, arithmetic, or allocation error.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, StableIdentityError> {
        self.validate()?;
        let capacity = self.canonical_capacity()?;
        let mut output = reserve_bytes("stable element key", capacity)?;
        output.extend_from_slice(STABLE_ELEMENT_DOMAIN);
        output.push(0);
        output.extend_from_slice(&STABLE_ELEMENT_LAYOUT_VERSION_V1.to_be_bytes());
        match self {
            Self::Node {
                scenario,
                local_name,
            } => {
                output.push(0x01);
                append_str32(&mut output, "stable node scenario", scenario)?;
                append_str32(&mut output, "stable node local name", local_name)?;
            }
            Self::Edge {
                scenario,
                edge_type,
                source_local_name,
                target_local_name,
            } => {
                output.push(0x02);
                append_str32(&mut output, "stable edge scenario", scenario)?;
                append_str32(&mut output, "stable edge type", edge_type)?;
                append_str32(&mut output, "stable edge source", source_local_name)?;
                append_str32(&mut output, "stable edge target", target_local_name)?;
            }
            Self::Hyperedge {
                scenario,
                local_name,
            } => {
                output.push(0x03);
                append_str32(&mut output, "stable hyperedge scenario", scenario)?;
                append_str32(&mut output, "stable hyperedge local name", local_name)?;
            }
        }
        debug_assert_eq!(output.len(), capacity);
        Ok(output)
    }

    /// Render the graph-owned framed ASCII carrier segment for this key.
    ///
    /// # Errors
    /// Returns a semantic string, arithmetic, or allocation error.
    pub fn carrier_segment(&self) -> Result<StableElementCarrierSegmentV1, StableIdentityError> {
        self.validate()?;
        let framed = match self {
            Self::Node {
                scenario,
                local_name,
            } => frame_segments(
                "stable element carrier segment",
                &["node", scenario, local_name],
                usize::MAX,
            )?,
            Self::Edge {
                scenario,
                edge_type,
                source_local_name,
                target_local_name,
            } => frame_segments(
                "stable element carrier segment",
                &[
                    "edge",
                    scenario,
                    edge_type,
                    source_local_name,
                    target_local_name,
                ],
                usize::MAX,
            )?,
            Self::Hyperedge {
                scenario,
                local_name,
            } => frame_segments(
                "stable element carrier segment",
                &["hyperedge", scenario, local_name],
                usize::MAX,
            )?,
        };
        Ok(StableElementCarrierSegmentV1(framed))
    }

    fn validate(&self) -> Result<(), StableIdentityError> {
        match self {
            Self::Node {
                scenario,
                local_name,
            }
            | Self::Hyperedge {
                scenario,
                local_name,
            } => {
                validate_qname("scenario scope", scenario)?;
                validate_symbol("authored local name", local_name)
            }
            Self::Edge {
                scenario,
                edge_type,
                source_local_name,
                target_local_name,
            } => {
                validate_qname("scenario scope", scenario)?;
                validate_ascii_graphic("edge type", edge_type, 1, MAX_STRUCTURAL_TYPE_BYTES)?;
                validate_symbol("source local name", source_local_name)?;
                validate_symbol("target local name", target_local_name)
            }
        }
    }

    fn canonical_capacity(&self) -> Result<usize, StableIdentityError> {
        let field_bytes = match self {
            Self::Node {
                scenario,
                local_name,
            }
            | Self::Hyperedge {
                scenario,
                local_name,
            } => checked_sum(&[scenario.len(), local_name.len(), 8])?,
            Self::Edge {
                scenario,
                edge_type,
                source_local_name,
                target_local_name,
            } => checked_sum(&[
                scenario.len(),
                edge_type.len(),
                source_local_name.len(),
                target_local_name.len(),
                16,
            ])?,
        };
        checked_sum(&[STABLE_ELEMENT_DOMAIN.len(), 1, 4, 1, field_bytes])
    }
}

/// The graph-owned ASCII rendering of one stable element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableElementCarrierSegmentV1(String);

impl StableElementCarrierSegmentV1 {
    /// Borrow the exact framed ASCII segment.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One graph-validated final RNG V2 carrier key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableCarrierKeyV2(String);

impl StableCarrierKeyV2 {
    /// Borrow the only bytes low-level RNG code may consume.
    #[must_use]
    pub fn validated_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Exact canonical stable resolver-manifest identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableElementResolverManifestV1 {
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
}

impl StableElementResolverManifestV1 {
    /// Borrow the exact canonical manifest bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical manifest bytes.
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SealedTopologyV1 {
    nodes: BTreeMap<NodeId, String>,
    edges: BTreeSet<(String, NodeId, NodeId)>,
    hyperedges: BTreeMap<HyperedgeId, (String, Vec<NodeId>)>,
}

/// Immutable handle-to-authored-name resolver over one sealed topology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableElementResolverV1 {
    scenario_scope: String,
    node_by_handle: HashMap<NodeId, StableElementKeyV1>,
    node_by_name: BTreeMap<String, NodeId>,
    hyperedge_by_handle: HashMap<HyperedgeId, StableElementKeyV1>,
    hyperedge_by_name: BTreeMap<String, HyperedgeId>,
    sealed_topology: SealedTopologyV1,
    manifest: StableElementResolverManifestV1,
}

/// Checked stable identity, resolver, topology, or bounded-codec failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StableIdentityError {
    /// Canonical stable-element bytes were truncated, malformed, or noncanonical.
    CanonicalBytes {
        /// Stable decoding field.
        field: &'static str,
    },
    /// A governed string failed its byte grammar or length bound.
    InvalidString {
        /// Stable semantic field name.
        field: &'static str,
        /// Zero-based offending index or the received length boundary.
        index: usize,
    },
    /// A live node had no authored local name.
    MissingNodeName {
        /// Unnamed live node.
        node: NodeId,
    },
    /// A node-name entry referred to no live node.
    ExtraNodeName {
        /// Unknown mapped node.
        node: NodeId,
    },
    /// Two live nodes shared one authored local name.
    DuplicateNodeName {
        /// Duplicated authored name.
        local_name: String,
    },
    /// A live hyperedge had no authored local name.
    MissingHyperedgeName {
        /// Unnamed live hyperedge.
        hyperedge: HyperedgeId,
    },
    /// A hyperedge-name entry referred to no live hyperedge.
    ExtraHyperedgeName {
        /// Unknown mapped hyperedge.
        hyperedge: HyperedgeId,
    },
    /// Two live hyperedges shared one authored local name.
    DuplicateHyperedgeName {
        /// Duplicated authored name.
        local_name: String,
    },
    /// A requested runtime node has no sealed identity.
    UnknownNode {
        /// Unknown runtime node.
        node: NodeId,
    },
    /// A requested runtime hyperedge has no sealed identity.
    UnknownHyperedge {
        /// Unknown runtime hyperedge.
        hyperedge: HyperedgeId,
    },
    /// A requested directed dyadic edge was not present when sealed.
    UnknownEdge {
        /// Requested edge type.
        edge_type: String,
        /// Requested source node.
        source: NodeId,
        /// Requested target node.
        target: NodeId,
    },
    /// One hyperedge was empty, duplicated a member, or named a dead member.
    InvalidHyperedge {
        /// Invalid hyperedge.
        hyperedge: HyperedgeId,
    },
    /// The combined resolver rows exceeded the declared ceiling.
    ResolverRowLimit {
        /// Received combined row count.
        actual: usize,
        /// Declared maximum.
        maximum: usize,
    },
    /// The sealed dyadic edge count exceeded the declared ceiling.
    EdgeLimit {
        /// Received edge count.
        actual: usize,
        /// Declared maximum.
        maximum: usize,
    },
    /// The active element stack exceeded the declared ceiling.
    ActiveElementLimit {
        /// Received active element count.
        actual: usize,
        /// Declared maximum.
        maximum: usize,
    },
    /// A supplied element did not belong to this sealed resolver.
    ElementNotSealed,
    /// One stable-state section exceeded its governed row ceiling.
    StateSectionLimit {
        /// Stable-state section name.
        section: &'static str,
        /// Received row count.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Stable-state rows plus nested member references exceeded their ceiling.
    FactUnitLimit {
        /// Received aggregate fact-unit count.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Two stable-state rows shared one governed semantic key.
    DuplicateFact {
        /// Stable-state section containing the duplicate.
        section: &'static str,
    },
    /// One node field appeared in both the f64 and Currency lanes.
    NumericLaneCollision {
        /// Stable node local name.
        node_local_name: String,
        /// Duplicated field qname.
        qname: String,
    },
    /// A non-finite binary64 value reached stable-state identity.
    NonFiniteValue {
        /// Stable-state section containing the value.
        section: &'static str,
    },
    /// An edge attribute tried to duplicate the governed strength slot.
    StrengthAttribute,
    /// Live graph topology no longer matches the sealed witness.
    TopologyChanged,
    /// Checked size arithmetic overflowed.
    CapacityOverflow {
        /// Stable capacity name.
        field: &'static str,
    },
    /// Canonical bytes exceeded their declared ceiling.
    ByteLimit {
        /// Stable encoded object name.
        field: &'static str,
        /// Received canonical byte count.
        actual: usize,
        /// Declared maximum.
        maximum: usize,
    },
    /// A checked integer conversion failed.
    IntegerConversion {
        /// Stable converted field name.
        field: &'static str,
        /// Unrepresentable value.
        value: usize,
    },
    /// A bounded allocation failed.
    Allocation {
        /// Stable allocation name.
        field: &'static str,
        /// Requested capacity or row count.
        requested: usize,
    },
}

fn read_canonical_string(
    input: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<String, StableIdentityError> {
    let length_end = cursor
        .checked_add(size_of::<u32>())
        .ok_or(StableIdentityError::CanonicalBytes { field })?;
    let length_bytes: [u8; 4] = input
        .get(*cursor..length_end)
        .ok_or(StableIdentityError::CanonicalBytes { field })?
        .try_into()
        .map_err(|_| StableIdentityError::CanonicalBytes { field })?;
    let length = usize::try_from(u32::from_be_bytes(length_bytes))
        .map_err(|_| StableIdentityError::CanonicalBytes { field })?;
    let value_end = length_end
        .checked_add(length)
        .ok_or(StableIdentityError::CanonicalBytes { field })?;
    let value = std::str::from_utf8(
        input
            .get(length_end..value_end)
            .ok_or(StableIdentityError::CanonicalBytes { field })?,
    )
    .map_err(|_| StableIdentityError::CanonicalBytes { field })?;
    *cursor = value_end;
    copy_string(field, value)
}

impl StableElementResolverV1 {
    /// Seal one complete fixed topology against exact authored identities.
    ///
    /// # Errors
    /// Returns the first string, bijection, topology, bound, arithmetic, or
    /// allocation error.
    pub fn seal<G: GraphSubstrate + CanonicalState>(
        graph: &G,
        scenario_scope: &str,
        node_names: &HashMap<NodeId, String>,
        hyperedge_names: &HashMap<HyperedgeId, String>,
    ) -> Result<Self, StableIdentityError> {
        validate_qname("scenario scope", scenario_scope)?;
        let topology = snapshot_topology(graph)?;
        validate_resolver_row_count(topology.nodes.len(), topology.hyperedges.len())?;
        let (node_by_handle, node_by_name) =
            resolve_nodes(scenario_scope, &topology.nodes, node_names)?;
        let (hyperedge_by_handle, hyperedge_by_name) =
            resolve_hyperedges(scenario_scope, &topology.hyperedges, hyperedge_names)?;
        let manifest =
            encode_manifest(scenario_scope, &topology, &node_by_name, &hyperedge_by_name)?;
        Ok(Self {
            scenario_scope: scenario_scope.to_owned(),
            node_by_handle,
            node_by_name,
            hyperedge_by_handle,
            hyperedge_by_name,
            sealed_topology: topology,
            manifest,
        })
    }

    /// Resolve a runtime node to its stable authored identity.
    ///
    /// # Errors
    /// Returns [`StableIdentityError::UnknownNode`] for an unsealed handle.
    pub fn node_key(&self, node: NodeId) -> Result<&StableElementKeyV1, StableIdentityError> {
        self.node_by_handle
            .get(&node)
            .ok_or(StableIdentityError::UnknownNode { node })
    }

    /// Resolve a runtime hyperedge to its stable authored identity.
    ///
    /// # Errors
    /// Returns [`StableIdentityError::UnknownHyperedge`] for an unsealed handle.
    pub fn hyperedge_key(
        &self,
        hyperedge: HyperedgeId,
    ) -> Result<&StableElementKeyV1, StableIdentityError> {
        self.hyperedge_by_handle
            .get(&hyperedge)
            .ok_or(StableIdentityError::UnknownHyperedge { hyperedge })
    }

    /// Resolve one sealed directed dyadic edge through named endpoints.
    ///
    /// # Errors
    /// Returns a string, unknown-node, or unknown-edge error.
    pub fn edge_key(
        &self,
        edge_type: &str,
        source: NodeId,
        target: NodeId,
    ) -> Result<StableElementKeyV1, StableIdentityError> {
        validate_ascii_graphic("edge type", edge_type, 1, MAX_STRUCTURAL_TYPE_BYTES)?;
        let source_name = self.node_local_name(source)?;
        let target_name = self.node_local_name(target)?;
        let owned_edge = (copy_string("stable edge type", edge_type)?, source, target);
        if !self.sealed_topology.edges.contains(&owned_edge) {
            return Err(StableIdentityError::UnknownEdge {
                edge_type: owned_edge.0,
                source,
                target,
            });
        }
        Ok(StableElementKeyV1::Edge {
            scenario: copy_string("stable edge scenario", &self.scenario_scope)?,
            edge_type: owned_edge.0,
            source_local_name: copy_string("stable edge source", source_name)?,
            target_local_name: copy_string("stable edge target", target_name)?,
        })
    }

    /// Build one final graph-validated RNG V2 carrier key.
    ///
    /// # Errors
    /// Returns a stack, provenance, arithmetic, byte-ceiling, or allocation
    /// error.
    pub fn carrier_key(
        &self,
        subject: &StableElementKeyV1,
        active: &[StableElementKeyV1],
        draw_slot: i64,
    ) -> Result<StableCarrierKeyV2, StableIdentityError> {
        if active.len() > MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 {
            return Err(StableIdentityError::ActiveElementLimit {
                actual: active.len(),
                maximum: MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2,
            });
        }
        self.validate_sealed_element(subject)?;
        let count = active
            .len()
            .checked_add(2)
            .ok_or(StableIdentityError::CapacityOverflow {
                field: "stable carrier segment count",
            })?;
        let mut segments = reserve_strings("stable carrier segments", count)?;
        segments.push(subject.carrier_segment()?.0);
        for key in active
            .iter()
            .take(MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 + 1)
        {
            self.validate_sealed_element(key)?;
            segments.push(key.carrier_segment()?.0);
        }
        segments.push(draw_slot.to_string());
        let mut borrowed = reserve_str_refs("stable carrier segment references", segments.len())?;
        for segment in segments
            .iter()
            .take(MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 + 2)
        {
            borrowed.push(segment.as_str());
        }
        Ok(StableCarrierKeyV2(frame_segments(
            "stable carrier key",
            &borrowed,
            MAX_STABLE_CARRIER_BYTES_V2,
        )?))
    }

    /// Verify that the live graph still has the exact sealed topology.
    ///
    /// # Errors
    /// Returns a topology or semantic snapshot error.
    pub fn validate_topology<G: CanonicalState>(
        &self,
        graph: &G,
    ) -> Result<(), StableIdentityError> {
        let current = snapshot_topology(graph)?;
        if current == self.sealed_topology {
            Ok(())
        } else {
            Err(StableIdentityError::TopologyChanged)
        }
    }

    /// Borrow the exact stable resolver manifest.
    #[must_use]
    pub const fn manifest(&self) -> &StableElementResolverManifestV1 {
        &self.manifest
    }

    /// Check one sealed stable node against its authored structural type.
    ///
    /// # Errors
    /// Returns a stable-key or sealed-membership refusal. A valid sealed node
    /// with another authored type returns `Ok(false)`.
    pub fn sealed_node_has_type(
        &self,
        key: &StableElementKeyV1,
        expected: &str,
    ) -> Result<bool, StableIdentityError> {
        validate_ascii_graphic("expected node type", expected, 1, MAX_STRUCTURAL_TYPE_BYTES)?;
        let StableElementKeyV1::Node {
            scenario,
            local_name,
        } = key
        else {
            return Err(StableIdentityError::ElementNotSealed);
        };
        key.validate()?;
        if scenario != &self.scenario_scope {
            return Err(StableIdentityError::ElementNotSealed);
        }
        let handle = self
            .node_by_name
            .get(local_name)
            .ok_or(StableIdentityError::ElementNotSealed)?;
        let actual = self
            .sealed_topology
            .nodes
            .get(handle)
            .ok_or(StableIdentityError::ElementNotSealed)?;
        Ok(actual == expected)
    }

    /// Check whether one exact directed edge belongs to the sealed topology.
    ///
    /// # Errors
    /// Returns a stable-key, string, or allocation refusal. Valid sealed
    /// endpoints without the requested edge return `Ok(false)`.
    pub fn contains_sealed_edge(
        &self,
        edge_type: &str,
        source: &StableElementKeyV1,
        target: &StableElementKeyV1,
    ) -> Result<bool, StableIdentityError> {
        validate_ascii_graphic("edge type", edge_type, 1, MAX_STRUCTURAL_TYPE_BYTES)?;
        let source = self.sealed_node_handle(source)?;
        let target = self.sealed_node_handle(target)?;
        let edge_type = copy_string("sealed edge type", edge_type)?;
        Ok(self
            .sealed_topology
            .edges
            .contains(&(edge_type, source, target)))
    }

    /// Validate that one stable key belongs to this exact sealed topology.
    ///
    /// # Errors
    /// Returns a stable-key or sealed-membership refusal.
    pub fn validate_sealed_key(&self, key: &StableElementKeyV1) -> Result<(), StableIdentityError> {
        self.validate_sealed_element(key)
    }

    /// Resolve an authored node name inside this sealed scenario.
    ///
    /// # Errors
    /// Returns [`StableIdentityError::ElementNotSealed`] when the name does
    /// not belong to the exact sealed topology.
    pub fn node_handle_by_local_name(
        &self,
        local_name: &str,
    ) -> Result<NodeId, StableIdentityError> {
        self.node_by_name
            .get(local_name)
            .copied()
            .ok_or(StableIdentityError::ElementNotSealed)
    }

    /// Resolve an authored hyperedge name inside this sealed scenario.
    ///
    /// # Errors
    /// Returns [`StableIdentityError::ElementNotSealed`] when the name does
    /// not belong to the exact sealed topology.
    pub fn hyperedge_handle_by_local_name(
        &self,
        local_name: &str,
    ) -> Result<HyperedgeId, StableIdentityError> {
        self.hyperedge_by_name
            .get(local_name)
            .copied()
            .ok_or(StableIdentityError::ElementNotSealed)
    }

    /// Borrow the sealed scenario scope used by stable checkpoint rows.
    #[must_use]
    pub fn scenario_scope(&self) -> &str {
        &self.scenario_scope
    }

    pub(crate) fn node_local_name(&self, node: NodeId) -> Result<&str, StableIdentityError> {
        match self.node_key(node)? {
            StableElementKeyV1::Node { local_name, .. } => Ok(local_name),
            StableElementKeyV1::Edge { .. } | StableElementKeyV1::Hyperedge { .. } => {
                Err(StableIdentityError::ElementNotSealed)
            }
        }
    }

    fn sealed_node_handle(&self, key: &StableElementKeyV1) -> Result<NodeId, StableIdentityError> {
        let StableElementKeyV1::Node {
            scenario,
            local_name,
        } = key
        else {
            return Err(StableIdentityError::ElementNotSealed);
        };
        key.validate()?;
        if scenario != &self.scenario_scope {
            return Err(StableIdentityError::ElementNotSealed);
        }
        self.node_by_name
            .get(local_name)
            .copied()
            .ok_or(StableIdentityError::ElementNotSealed)
    }

    fn validate_sealed_element(&self, key: &StableElementKeyV1) -> Result<(), StableIdentityError> {
        key.validate()?;
        let sealed = match key {
            StableElementKeyV1::Node {
                scenario,
                local_name,
            } => scenario == &self.scenario_scope && self.node_by_name.contains_key(local_name),
            StableElementKeyV1::Hyperedge {
                scenario,
                local_name,
            } => {
                scenario == &self.scenario_scope && self.hyperedge_by_name.contains_key(local_name)
            }
            StableElementKeyV1::Edge {
                scenario,
                edge_type,
                source_local_name,
                target_local_name,
            } => {
                let Some(source) = self.node_by_name.get(source_local_name).copied() else {
                    return Err(StableIdentityError::ElementNotSealed);
                };
                let Some(target) = self.node_by_name.get(target_local_name).copied() else {
                    return Err(StableIdentityError::ElementNotSealed);
                };
                scenario == &self.scenario_scope
                    && self
                        .sealed_topology
                        .edges
                        .contains(&(edge_type.clone(), source, target))
            }
        };
        if sealed {
            Ok(())
        } else {
            Err(StableIdentityError::ElementNotSealed)
        }
    }
}

fn snapshot_topology<G: CanonicalState>(
    graph: &G,
) -> Result<SealedTopologyV1, StableIdentityError> {
    let raw_nodes = graph.all_nodes();
    let raw_hyperedges = graph.all_hyperedges();
    let raw_edges = graph.all_edges();
    validate_resolver_row_count(raw_nodes.len(), raw_hyperedges.len())?;
    validate_resolver_edge_count(raw_edges.len())?;
    validate_resolver_fact_units(&raw_nodes, &raw_edges, &raw_hyperedges)?;
    let mut nodes = BTreeMap::new();
    for (node, node_type) in raw_nodes.into_iter().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        validate_ascii_graphic("node type", &node_type, 1, MAX_STRUCTURAL_TYPE_BYTES)?;
        if nodes.insert(node, node_type).is_some() {
            return Err(StableIdentityError::TopologyChanged);
        }
    }
    let mut hyperedges = BTreeMap::new();
    for (hyperedge, hyperedge_type, mut members) in raw_hyperedges
        .into_iter()
        .take(MAX_STABLE_RESOLVER_ROWS_V1 + 1)
    {
        validate_ascii_graphic(
            "hyperedge type",
            &hyperedge_type,
            1,
            MAX_STRUCTURAL_TYPE_BYTES,
        )?;
        members.sort_unstable();
        validate_hyperedge_members(hyperedge, &members, &nodes)?;
        if hyperedges
            .insert(hyperedge, (hyperedge_type, members))
            .is_some()
        {
            return Err(StableIdentityError::TopologyChanged);
        }
    }
    let mut edges = BTreeSet::new();
    for (edge_type, source, target, _) in raw_edges.into_iter().take(MAX_STABLE_EDGES_V1 + 1) {
        validate_ascii_graphic("edge type", &edge_type, 1, MAX_STRUCTURAL_TYPE_BYTES)?;
        if !nodes.contains_key(&source) || !nodes.contains_key(&target) {
            return Err(StableIdentityError::UnknownEdge {
                edge_type,
                source,
                target,
            });
        }
        if !edges.insert((edge_type, source, target)) {
            return Err(StableIdentityError::TopologyChanged);
        }
    }
    Ok(SealedTopologyV1 {
        nodes,
        edges,
        hyperedges,
    })
}

fn validate_resolver_edge_count(actual: usize) -> Result<(), StableIdentityError> {
    if actual <= MAX_STABLE_EDGES_V1 {
        Ok(())
    } else {
        Err(StableIdentityError::EdgeLimit {
            actual,
            maximum: MAX_STABLE_EDGES_V1,
        })
    }
}

fn validate_resolver_fact_units(
    nodes: &[(NodeId, String)],
    edges: &[(String, NodeId, NodeId, f64)],
    hyperedges: &[(HyperedgeId, String, Vec<NodeId>)],
) -> Result<(), StableIdentityError> {
    let mut actual = nodes
        .len()
        .checked_add(edges.len())
        .and_then(|value| value.checked_add(hyperedges.len()))
        .ok_or(StableIdentityError::CapacityOverflow {
            field: "stable resolver fact units",
        })?;
    for (_, _, members) in hyperedges.iter().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        validate_resolver_member_count(members.len())?;
        actual =
            actual
                .checked_add(members.len())
                .ok_or(StableIdentityError::CapacityOverflow {
                    field: "stable resolver fact units",
                })?;
    }
    validate_resolver_fact_unit_count(actual)
}

fn validate_resolver_fact_unit_count(actual: usize) -> Result<(), StableIdentityError> {
    if actual <= MAX_STABLE_RESOLVER_FACT_UNITS_V1 {
        Ok(())
    } else {
        Err(StableIdentityError::FactUnitLimit {
            actual,
            maximum: MAX_STABLE_RESOLVER_FACT_UNITS_V1,
        })
    }
}

fn validate_resolver_member_count(actual: usize) -> Result<(), StableIdentityError> {
    if actual <= MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1 {
        Ok(())
    } else {
        Err(StableIdentityError::StateSectionLimit {
            section: "resolver hyperedge members",
            actual,
            maximum: MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1,
        })
    }
}

fn validate_hyperedge_members(
    hyperedge: HyperedgeId,
    members: &[NodeId],
    nodes: &BTreeMap<NodeId, String>,
) -> Result<(), StableIdentityError> {
    if members.is_empty() {
        return Err(StableIdentityError::InvalidHyperedge { hyperedge });
    }
    let mut previous = None;
    for member in members.iter().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        if !nodes.contains_key(member) || previous == Some(*member) {
            return Err(StableIdentityError::InvalidHyperedge { hyperedge });
        }
        previous = Some(*member);
    }
    Ok(())
}

fn resolve_nodes(
    scenario: &str,
    nodes: &BTreeMap<NodeId, String>,
    names: &HashMap<NodeId, String>,
) -> Result<StableNodeMaps, StableIdentityError> {
    let mut by_handle = reserve_hashmap("stable node identities", nodes.len())?;
    let mut by_name = BTreeMap::new();
    for node in nodes.keys().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        let local_name = names
            .get(node)
            .ok_or(StableIdentityError::MissingNodeName { node: *node })?;
        validate_symbol("node local name", local_name)?;
        if by_name.insert(local_name.clone(), *node).is_some() {
            return Err(StableIdentityError::DuplicateNodeName {
                local_name: local_name.clone(),
            });
        }
        by_handle.insert(
            *node,
            StableElementKeyV1::Node {
                scenario: scenario.to_owned(),
                local_name: local_name.clone(),
            },
        );
    }
    for node in names.keys().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        if !nodes.contains_key(node) {
            return Err(StableIdentityError::ExtraNodeName { node: *node });
        }
    }
    Ok((by_handle, by_name))
}

fn resolve_hyperedges(
    scenario: &str,
    hyperedges: &BTreeMap<HyperedgeId, (String, Vec<NodeId>)>,
    names: &HashMap<HyperedgeId, String>,
) -> Result<StableHyperedgeMaps, StableIdentityError> {
    let mut by_handle = reserve_hashmap("stable hyperedge identities", hyperedges.len())?;
    let mut by_name = BTreeMap::new();
    for hyperedge in hyperedges.keys().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        let local_name = names
            .get(hyperedge)
            .ok_or(StableIdentityError::MissingHyperedgeName {
                hyperedge: *hyperedge,
            })?;
        validate_symbol("hyperedge local name", local_name)?;
        if by_name.insert(local_name.clone(), *hyperedge).is_some() {
            return Err(StableIdentityError::DuplicateHyperedgeName {
                local_name: local_name.clone(),
            });
        }
        by_handle.insert(
            *hyperedge,
            StableElementKeyV1::Hyperedge {
                scenario: scenario.to_owned(),
                local_name: local_name.clone(),
            },
        );
    }
    for hyperedge in names.keys().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        if !hyperedges.contains_key(hyperedge) {
            return Err(StableIdentityError::ExtraHyperedgeName {
                hyperedge: *hyperedge,
            });
        }
    }
    Ok((by_handle, by_name))
}

fn encode_manifest(
    scenario: &str,
    topology: &SealedTopologyV1,
    node_by_name: &BTreeMap<String, NodeId>,
    hyperedge_by_name: &BTreeMap<String, HyperedgeId>,
) -> Result<StableElementResolverManifestV1, StableIdentityError> {
    let capacity = manifest_capacity(scenario, topology, node_by_name, hyperedge_by_name)?;
    if capacity > MAX_STABLE_RESOLVER_MANIFEST_BYTES_V1 {
        return Err(StableIdentityError::ByteLimit {
            field: "stable element resolver manifest",
            actual: capacity,
            maximum: MAX_STABLE_RESOLVER_MANIFEST_BYTES_V1,
        });
    }
    let node_count = checked_u32("stable resolver node count", node_by_name.len())?;
    let hyperedge_count = checked_u32("stable resolver hyperedge count", hyperedge_by_name.len())?;
    let mut canonical_bytes = reserve_bytes("stable element resolver manifest", capacity)?;
    canonical_bytes.extend_from_slice(STABLE_RESOLVER_DOMAIN);
    canonical_bytes.push(0);
    canonical_bytes
        .extend_from_slice(&STABLE_ELEMENT_RESOLVER_MANIFEST_LAYOUT_VERSION_V1.to_be_bytes());
    canonical_bytes.push(0x01);
    append_str32(&mut canonical_bytes, "stable resolver scenario", scenario)?;
    canonical_bytes.push(0x02);
    canonical_bytes.extend_from_slice(&node_count.to_be_bytes());
    append_manifest_nodes(&mut canonical_bytes, topology, node_by_name)?;
    canonical_bytes.push(0x03);
    canonical_bytes.extend_from_slice(&hyperedge_count.to_be_bytes());
    append_manifest_hyperedges(&mut canonical_bytes, topology, hyperedge_by_name)?;
    debug_assert_eq!(canonical_bytes.len(), capacity);
    let digest = sha256_of(&canonical_bytes);
    Ok(StableElementResolverManifestV1 {
        canonical_bytes,
        digest,
    })
}

fn append_manifest_nodes(
    output: &mut Vec<u8>,
    topology: &SealedTopologyV1,
    node_by_name: &BTreeMap<String, NodeId>,
) -> Result<(), StableIdentityError> {
    for (local_name, node) in node_by_name.iter().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        let node_type = topology
            .nodes
            .get(node)
            .ok_or(StableIdentityError::TopologyChanged)?;
        append_str32(output, "stable resolver node local name", local_name)?;
        append_str32(output, "stable resolver node type", node_type)?;
    }
    Ok(())
}

fn append_manifest_hyperedges(
    output: &mut Vec<u8>,
    topology: &SealedTopologyV1,
    hyperedge_by_name: &BTreeMap<String, HyperedgeId>,
) -> Result<(), StableIdentityError> {
    for (local_name, hyperedge) in hyperedge_by_name
        .iter()
        .take(MAX_STABLE_RESOLVER_ROWS_V1 + 1)
    {
        let (hyperedge_type, _) = topology
            .hyperedges
            .get(hyperedge)
            .ok_or(StableIdentityError::TopologyChanged)?;
        append_str32(output, "stable resolver hyperedge local name", local_name)?;
        append_str32(output, "stable resolver hyperedge type", hyperedge_type)?;
    }
    Ok(())
}

fn manifest_capacity(
    scenario: &str,
    topology: &SealedTopologyV1,
    node_by_name: &BTreeMap<String, NodeId>,
    hyperedge_by_name: &BTreeMap<String, HyperedgeId>,
) -> Result<usize, StableIdentityError> {
    let mut capacity = checked_sum(&[
        STABLE_RESOLVER_DOMAIN.len(),
        1,
        4,
        1,
        4,
        scenario.len(),
        1,
        4,
    ])?;
    for (name, node) in node_by_name.iter().take(MAX_STABLE_RESOLVER_ROWS_V1 + 1) {
        let node_type = topology
            .nodes
            .get(node)
            .ok_or(StableIdentityError::TopologyChanged)?;
        capacity = checked_sum(&[capacity, 4, name.len(), 4, node_type.len()])?;
    }
    capacity = checked_sum(&[capacity, 1, 4])?;
    for (name, hyperedge) in hyperedge_by_name
        .iter()
        .take(MAX_STABLE_RESOLVER_ROWS_V1 + 1)
    {
        let (hyperedge_type, _) = topology
            .hyperedges
            .get(hyperedge)
            .ok_or(StableIdentityError::TopologyChanged)?;
        capacity = checked_sum(&[capacity, 4, name.len(), 4, hyperedge_type.len()])?;
    }
    Ok(capacity)
}

fn validate_resolver_row_count(nodes: usize, hyperedges: usize) -> Result<(), StableIdentityError> {
    let actual = nodes
        .checked_add(hyperedges)
        .ok_or(StableIdentityError::CapacityOverflow {
            field: "stable resolver row count",
        })?;
    if actual <= MAX_STABLE_RESOLVER_ROWS_V1 {
        Ok(())
    } else {
        Err(StableIdentityError::ResolverRowLimit {
            actual,
            maximum: MAX_STABLE_RESOLVER_ROWS_V1,
        })
    }
}

fn frame_segments(
    field: &'static str,
    segments: &[&str],
    maximum: usize,
) -> Result<String, StableIdentityError> {
    let mut capacity = segments.len().saturating_sub(1);
    for segment in segments
        .iter()
        .take(MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 + 2)
    {
        let framed = decimal_digits(segment.len())
            .checked_add(1)
            .and_then(|value| value.checked_add(segment.len()))
            .ok_or(StableIdentityError::CapacityOverflow { field })?;
        capacity = capacity
            .checked_add(framed)
            .ok_or(StableIdentityError::CapacityOverflow { field })?;
    }
    if capacity > maximum {
        return Err(StableIdentityError::ByteLimit {
            field,
            actual: capacity,
            maximum,
        });
    }
    let mut output = reserve_string(field, capacity)?;
    for (index, segment) in segments
        .iter()
        .take(MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 + 2)
        .enumerate()
    {
        if index > 0 {
            output.push('|');
        }
        output.push_str(&segment.len().to_string());
        output.push(':');
        output.push_str(segment);
    }
    debug_assert_eq!(output.len(), capacity);
    Ok(output)
}

pub(crate) fn validate_qname(field: &'static str, value: &str) -> Result<(), StableIdentityError> {
    if value.is_empty() || value.len() > MAX_QNAME_BYTES {
        return Err(StableIdentityError::InvalidString {
            field,
            index: value.len(),
        });
    }
    let mut segment_count = 1usize;
    let mut segment_length = 0usize;
    for index in 0..MAX_QNAME_BYTES {
        let Some(byte) = value.as_bytes().get(index).copied() else {
            break;
        };
        if byte == b'/' {
            if segment_length == 0 || segment_count == MAX_QNAME_SEGMENTS {
                return Err(StableIdentityError::InvalidString { field, index });
            }
            segment_count = segment_count.saturating_add(1);
            segment_length = 0;
        } else if !valid_symbol_byte(byte, segment_length) || segment_length == MAX_SYMBOL_BYTES {
            return Err(StableIdentityError::InvalidString { field, index });
        } else {
            segment_length = segment_length.saturating_add(1);
        }
    }
    if segment_length == 0 {
        Err(StableIdentityError::InvalidString {
            field,
            index: value.len(),
        })
    } else {
        Ok(())
    }
}

fn validate_symbol(field: &'static str, value: &str) -> Result<(), StableIdentityError> {
    if value.is_empty() || value.len() > MAX_SYMBOL_BYTES {
        return Err(StableIdentityError::InvalidString {
            field,
            index: value.len(),
        });
    }
    for index in 0..MAX_SYMBOL_BYTES {
        let Some(byte) = value.as_bytes().get(index).copied() else {
            break;
        };
        if !valid_symbol_byte(byte, index) {
            return Err(StableIdentityError::InvalidString { field, index });
        }
    }
    Ok(())
}

fn valid_symbol_byte(byte: u8, segment_index: usize) -> bool {
    if segment_index == 0 {
        byte.is_ascii_lowercase()
    } else {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
    }
}

fn validate_ascii_graphic(
    field: &'static str,
    value: &str,
    minimum: usize,
    maximum: usize,
) -> Result<(), StableIdentityError> {
    if value.len() < minimum || value.len() > maximum {
        return Err(StableIdentityError::InvalidString {
            field,
            index: value.len(),
        });
    }
    for index in 0..maximum {
        let Some(byte) = value.as_bytes().get(index).copied() else {
            break;
        };
        if !(0x21..=0x7e).contains(&byte) {
            return Err(StableIdentityError::InvalidString { field, index });
        }
    }
    Ok(())
}

fn append_str32(
    output: &mut Vec<u8>,
    field: &'static str,
    value: &str,
) -> Result<(), StableIdentityError> {
    let length = checked_u32(field, value.len())?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn checked_u32(field: &'static str, value: usize) -> Result<u32, StableIdentityError> {
    u32::try_from(value).map_err(|_| StableIdentityError::IntegerConversion { field, value })
}

fn checked_sum(values: &[usize]) -> Result<usize, StableIdentityError> {
    if values.len() > 8 {
        return Err(StableIdentityError::CapacityOverflow {
            field: "stable identity capacity term count",
        });
    }
    let mut total = 0usize;
    for value in values.iter().take(8) {
        total = total
            .checked_add(*value)
            .ok_or(StableIdentityError::CapacityOverflow {
                field: "stable identity canonical bytes",
            })?;
    }
    Ok(total)
}

fn decimal_digits(value: usize) -> usize {
    if value == 0 {
        1
    } else {
        value.ilog10() as usize + 1
    }
}

fn reserve_bytes(field: &'static str, capacity: usize) -> Result<Vec<u8>, StableIdentityError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_: TryReserveError| StableIdentityError::Allocation {
            field,
            requested: capacity,
        })?;
    Ok(output)
}

fn reserve_string(field: &'static str, capacity: usize) -> Result<String, StableIdentityError> {
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_: TryReserveError| StableIdentityError::Allocation {
            field,
            requested: capacity,
        })?;
    Ok(output)
}

fn copy_string(field: &'static str, source: &str) -> Result<String, StableIdentityError> {
    let mut output = reserve_string(field, source.len())?;
    output.push_str(source);
    Ok(output)
}

fn reserve_strings(field: &'static str, count: usize) -> Result<Vec<String>, StableIdentityError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_: TryReserveError| StableIdentityError::Allocation {
            field,
            requested: count,
        })?;
    Ok(output)
}

fn reserve_str_refs<'a>(
    field: &'static str,
    count: usize,
) -> Result<Vec<&'a str>, StableIdentityError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_: TryReserveError| StableIdentityError::Allocation {
            field,
            requested: count,
        })?;
    Ok(output)
}

fn reserve_hashmap<K: Eq + std::hash::Hash, V>(
    field: &'static str,
    count: usize,
) -> Result<HashMap<K, V>, StableIdentityError> {
    let mut output = HashMap::new();
    output
        .try_reserve(count)
        .map_err(|_: TryReserveError| StableIdentityError::Allocation {
            field,
            requested: count,
        })?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use babylon_kernel::Currency;

    use super::{
        frame_segments, snapshot_topology, validate_resolver_edge_count,
        validate_resolver_fact_unit_count, validate_resolver_member_count, StableIdentityError,
        MAX_STABLE_EDGES_V1, MAX_STABLE_RESOLVER_FACT_UNITS_V1,
        MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1,
    };
    use crate::state_hash::CanonicalState;
    use crate::substrate::{HyperedgeId, NodeId};

    struct Facts {
        nodes: Vec<(NodeId, String)>,
        edges: Vec<(String, NodeId, NodeId, f64)>,
        hyperedges: Vec<(HyperedgeId, String, Vec<NodeId>)>,
    }

    impl CanonicalState for Facts {
        fn all_nodes(&self) -> Vec<(NodeId, String)> {
            self.nodes.clone()
        }

        fn all_attributes(&self) -> Vec<(NodeId, String, f64)> {
            Vec::new()
        }

        fn all_edges(&self) -> Vec<(String, NodeId, NodeId, f64)> {
            self.edges.clone()
        }

        fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)> {
            self.hyperedges.clone()
        }

        fn all_edge_attributes(&self) -> Vec<(String, NodeId, NodeId, String, f64)> {
            Vec::new()
        }

        fn all_currency_attributes(&self) -> Vec<(NodeId, String, Currency)> {
            Vec::new()
        }

        fn all_hyperedge_attributes(&self) -> Vec<(HyperedgeId, String, f64)> {
            Vec::new()
        }
    }

    #[test]
    fn snapshot_refuses_a_dangling_edge_endpoint() {
        let facts = Facts {
            nodes: vec![(NodeId(0), "class".to_owned())],
            edges: vec![("solidarity".to_owned(), NodeId(0), NodeId(1), 1.0)],
            hyperedges: Vec::new(),
        };
        assert_eq!(
            snapshot_topology(&facts),
            Err(StableIdentityError::UnknownEdge {
                edge_type: "solidarity".to_owned(),
                source: NodeId(0),
                target: NodeId(1),
            })
        );
    }

    #[test]
    fn snapshot_refuses_a_dangling_hyperedge_member() {
        let facts = Facts {
            nodes: vec![(NodeId(0), "class".to_owned())],
            edges: Vec::new(),
            hyperedges: vec![(
                HyperedgeId(0),
                "coalition".to_owned(),
                vec![NodeId(0), NodeId(1)],
            )],
        };
        assert_eq!(
            snapshot_topology(&facts),
            Err(StableIdentityError::InvalidHyperedge {
                hyperedge: HyperedgeId(0),
            })
        );
    }

    #[test]
    fn snapshot_refuses_excess_hyperedge_members_before_member_validation() {
        let facts = Facts {
            nodes: Vec::new(),
            edges: Vec::new(),
            hyperedges: vec![(
                HyperedgeId(0),
                "coalition".to_owned(),
                vec![NodeId(0); MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1 + 1],
            )],
        };
        assert_eq!(
            snapshot_topology(&facts),
            Err(StableIdentityError::StateSectionLimit {
                section: "resolver hyperedge members",
                actual: MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1 + 1,
                maximum: MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1,
            })
        );
    }

    #[test]
    fn resolver_member_ceiling_accepts_maximum_and_refuses_plus_one() {
        assert_eq!(
            validate_resolver_member_count(MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1),
            Ok(())
        );
        assert_eq!(
            validate_resolver_member_count(MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1 + 1),
            Err(StableIdentityError::StateSectionLimit {
                section: "resolver hyperedge members",
                actual: MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1 + 1,
                maximum: MAX_STABLE_RESOLVER_HYPEREDGE_MEMBERS_V1,
            })
        );
        assert_eq!(
            validate_resolver_fact_unit_count(MAX_STABLE_RESOLVER_FACT_UNITS_V1),
            Ok(())
        );
        assert_eq!(
            validate_resolver_fact_unit_count(MAX_STABLE_RESOLVER_FACT_UNITS_V1 + 1),
            Err(StableIdentityError::FactUnitLimit {
                actual: MAX_STABLE_RESOLVER_FACT_UNITS_V1 + 1,
                maximum: MAX_STABLE_RESOLVER_FACT_UNITS_V1,
            })
        );
    }

    #[test]
    fn resolver_edge_ceiling_accepts_maximum_and_refuses_plus_one() {
        assert_eq!(validate_resolver_edge_count(MAX_STABLE_EDGES_V1), Ok(()));
        assert_eq!(
            validate_resolver_edge_count(MAX_STABLE_EDGES_V1 + 1),
            Err(StableIdentityError::EdgeLimit {
                actual: MAX_STABLE_EDGES_V1 + 1,
                maximum: MAX_STABLE_EDGES_V1,
            })
        );
    }

    #[test]
    fn frame_segments_enforces_its_supplied_byte_ceiling() {
        let exact = frame_segments("test frame", &["xxxxxxxxxxxx"], 15).unwrap();
        assert_eq!(exact.len(), 15);
        assert_eq!(
            frame_segments("test frame", &["xxxxxxxxxxxxx"], 15),
            Err(StableIdentityError::ByteLimit {
                field: "test frame",
                actual: 16,
                maximum: 15,
            })
        );
    }
}
