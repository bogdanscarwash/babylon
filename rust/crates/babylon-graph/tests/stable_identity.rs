use std::collections::HashMap;

use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::{
    StableElementKeyV1, StableElementResolverV1, StableIdentityError,
    MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2, MAX_STABLE_CARRIER_BYTES_V2,
    MAX_STABLE_RESOLVER_MANIFEST_BYTES_V1, MAX_STABLE_RESOLVER_ROWS_V1,
};
use babylon_graph::stable_state::{encode_stable_graph_state_v1, StableGraphStateV1};
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::Currency;

struct StableFixture {
    graph: MemoryGraph,
    owners: NodeId,
    workers: NodeId,
    coalition_a: HyperedgeId,
    coalition_b: HyperedgeId,
    node_names: HashMap<NodeId, String>,
    hyperedge_names: HashMap<HyperedgeId, String>,
}

fn fixture() -> StableFixture {
    let mut graph = MemoryGraph::new();
    let owners = graph.add_node("class").unwrap();
    let workers = graph.add_node("class").unwrap();
    graph.add_edge("solidarity", workers, owners, 0.75).unwrap();
    let coalition_a = graph
        .add_hyperedge("coalition", &[workers, owners])
        .unwrap();
    let coalition_b = graph
        .add_hyperedge("coalition", &[owners, workers])
        .unwrap();
    let node_names = HashMap::from([
        (workers, "workers".to_owned()),
        (owners, "owners".to_owned()),
    ]);
    let hyperedge_names = HashMap::from([
        (coalition_b, "coalition-b".to_owned()),
        (coalition_a, "coalition-a".to_owned()),
    ]);
    StableFixture {
        graph,
        owners,
        workers,
        coalition_a,
        coalition_b,
        node_names,
        hyperedge_names,
    }
}

fn resolver(value: &StableFixture) -> StableElementResolverV1 {
    StableElementResolverV1::seal(
        &value.graph,
        "demo/world",
        &value.node_names,
        &value.hyperedge_names,
    )
    .unwrap()
}

fn str32(value: &str) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
    output.extend_from_slice(value.as_bytes());
    output
}

#[test]
fn stable_element_binary_keys_and_ascii_segments_are_exact() {
    let value = fixture();
    let resolver = resolver(&value);
    let node = resolver.node_key(value.workers).unwrap();
    let edge = resolver
        .edge_key("solidarity", value.workers, value.owners)
        .unwrap();
    let hyperedge = resolver.hyperedge_key(value.coalition_a).unwrap();

    let expected_node = [
        b"babylon.stable-element\0".as_slice(),
        &1_u32.to_be_bytes(),
        &[0x01],
        &str32("demo/world"),
        &str32("workers"),
    ]
    .concat();
    let expected_edge = [
        b"babylon.stable-element\0".as_slice(),
        &1_u32.to_be_bytes(),
        &[0x02],
        &str32("demo/world"),
        &str32("solidarity"),
        &str32("workers"),
        &str32("owners"),
    ]
    .concat();
    let expected_hyperedge = [
        b"babylon.stable-element\0".as_slice(),
        &1_u32.to_be_bytes(),
        &[0x03],
        &str32("demo/world"),
        &str32("coalition-a"),
    ]
    .concat();

    assert_eq!(node.canonical_bytes().unwrap(), expected_node);
    assert_eq!(edge.canonical_bytes().unwrap(), expected_edge);
    assert_eq!(hyperedge.canonical_bytes().unwrap(), expected_hyperedge);
    assert_eq!(
        node.carrier_segment().unwrap().as_str(),
        "4:node|10:demo/world|7:workers"
    );
    assert_eq!(
        edge.carrier_segment().unwrap().as_str(),
        "4:edge|10:demo/world|10:solidarity|7:workers|6:owners"
    );
    assert_eq!(
        hyperedge.carrier_segment().unwrap().as_str(),
        "9:hyperedge|10:demo/world|11:coalition-a"
    );
}

#[test]
fn carrier_reframes_resolved_elements_in_outermost_to_innermost_order() {
    let value = fixture();
    let resolver = resolver(&value);
    let subject = resolver.node_key(value.workers).unwrap().clone();
    let edge = resolver
        .edge_key("solidarity", value.workers, value.owners)
        .unwrap();
    let hyperedge = resolver.hyperedge_key(value.coalition_a).unwrap().clone();
    let mixed = resolver
        .carrier_key(&subject, &[edge, hyperedge], -7)
        .unwrap();
    assert_eq!(
        mixed.validated_bytes(),
        b"30:4:node|10:demo/world|7:workers|53:4:edge|10:demo/world|10:solidarity|7:workers|6:owners|40:9:hyperedge|10:demo/world|11:coalition-a|2:-7"
    );

    let zero_active = resolver.carrier_key(&subject, &[], 0).unwrap();
    assert_eq!(
        zero_active.validated_bytes(),
        b"30:4:node|10:demo/world|7:workers|1:0"
    );
    let minimum_slot = resolver.carrier_key(&subject, &[], i64::MIN).unwrap();
    assert!(minimum_slot
        .validated_bytes()
        .ends_with(b"|20:-9223372036854775808"));
}

#[test]
fn carrier_accepts_256_active_elements_and_refuses_257_before_allocation() {
    let value = fixture();
    let resolver = resolver(&value);
    let subject = resolver.node_key(value.workers).unwrap().clone();
    let active = vec![subject.clone(); MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2];
    let maximum = resolver.carrier_key(&subject, &active, i64::MAX).unwrap();
    assert!(maximum.validated_bytes().len() <= MAX_STABLE_CARRIER_BYTES_V2);

    let too_many = vec![subject.clone(); MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 + 1];
    assert_eq!(
        resolver.carrier_key(&subject, &too_many, 0),
        Err(StableIdentityError::ActiveElementLimit {
            actual: MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2 + 1,
            maximum: MAX_STABLE_CARRIER_ACTIVE_ELEMENTS_V2,
        })
    );
}

#[test]
fn resolver_manifest_is_exact_sorted_and_parallel_hyperedges_stay_distinct() {
    let value = fixture();
    let resolver = resolver(&value);
    let expected = [
        b"babylon.stable-element-resolver\0".as_slice(),
        &1_u32.to_be_bytes(),
        &[0x01],
        &str32("demo/world"),
        &[0x02],
        &2_u32.to_be_bytes(),
        &str32("owners"),
        &str32("class"),
        &str32("workers"),
        &str32("class"),
        &[0x03],
        &2_u32.to_be_bytes(),
        &str32("coalition-a"),
        &str32("coalition"),
        &str32("coalition-b"),
        &str32("coalition"),
    ]
    .concat();

    assert_eq!(resolver.manifest().canonical_bytes(), expected);
    assert_eq!(
        resolver.manifest().digest(),
        [
            0x93, 0xbe, 0xce, 0x63, 0xe9, 0x36, 0xcb, 0x85, 0xa9, 0x85, 0xed, 0x5e, 0xfa, 0x4f,
            0xe7, 0x94, 0x5d, 0x94, 0x60, 0xba, 0x92, 0xf9, 0xbe, 0x2c, 0x87, 0x31, 0x98, 0x11,
            0xa4, 0x39, 0xf6, 0x11,
        ]
    );
    assert_ne!(
        resolver.hyperedge_key(value.coalition_a).unwrap(),
        resolver.hyperedge_key(value.coalition_b).unwrap()
    );
    assert!(resolver.manifest().canonical_bytes().len() <= MAX_STABLE_RESOLVER_MANIFEST_BYTES_V1);
}

#[test]
fn resolver_seal_requires_exact_bijections_and_strict_ascii_names() {
    let value = fixture();
    let mut missing = value.node_names.clone();
    missing.remove(&value.workers);
    assert_eq!(
        StableElementResolverV1::seal(&value.graph, "demo/world", &missing, &value.hyperedge_names,),
        Err(StableIdentityError::MissingNodeName {
            node: value.workers,
        })
    );

    let mut duplicate = value.node_names.clone();
    duplicate.insert(value.workers, "owners".to_owned());
    assert_eq!(
        StableElementResolverV1::seal(
            &value.graph,
            "demo/world",
            &duplicate,
            &value.hyperedge_names,
        ),
        Err(StableIdentityError::DuplicateNodeName {
            local_name: "owners".to_owned(),
        })
    );

    let mut extra = value.node_names.clone();
    extra.insert(NodeId(999), "ghost".to_owned());
    assert_eq!(
        StableElementResolverV1::seal(&value.graph, "demo/world", &extra, &value.hyperedge_names,),
        Err(StableIdentityError::ExtraNodeName { node: NodeId(999) })
    );

    let mut non_ascii = value.node_names.clone();
    non_ascii.insert(value.workers, "wörkers".to_owned());
    assert!(matches!(
        StableElementResolverV1::seal(
            &value.graph,
            "demo/world",
            &non_ascii,
            &value.hyperedge_names,
        ),
        Err(StableIdentityError::InvalidString {
            field: "node local name",
            ..
        })
    ));

    let mut missing_hyperedge = value.hyperedge_names.clone();
    missing_hyperedge.remove(&value.coalition_a);
    assert_eq!(
        StableElementResolverV1::seal(
            &value.graph,
            "demo/world",
            &value.node_names,
            &missing_hyperedge,
        ),
        Err(StableIdentityError::MissingHyperedgeName {
            hyperedge: value.coalition_a,
        })
    );

    let mut duplicate_hyperedge = value.hyperedge_names.clone();
    duplicate_hyperedge.insert(value.coalition_b, "coalition-a".to_owned());
    assert_eq!(
        StableElementResolverV1::seal(
            &value.graph,
            "demo/world",
            &value.node_names,
            &duplicate_hyperedge,
        ),
        Err(StableIdentityError::DuplicateHyperedgeName {
            local_name: "coalition-a".to_owned(),
        })
    );

    let mut extra_hyperedge = value.hyperedge_names.clone();
    extra_hyperedge.insert(HyperedgeId(999), "ghost-group".to_owned());
    assert_eq!(
        StableElementResolverV1::seal(
            &value.graph,
            "demo/world",
            &value.node_names,
            &extra_hyperedge,
        ),
        Err(StableIdentityError::ExtraHyperedgeName {
            hyperedge: HyperedgeId(999),
        })
    );
}

#[test]
fn resolver_refuses_dangling_edges_and_never_falls_back_to_runtime_ids() {
    let value = fixture();
    let resolver = resolver(&value);
    assert_eq!(
        resolver.node_key(NodeId(999)),
        Err(StableIdentityError::UnknownNode { node: NodeId(999) })
    );
    assert_eq!(
        resolver.edge_key("solidarity", value.owners, value.workers),
        Err(StableIdentityError::UnknownEdge {
            edge_type: "solidarity".to_owned(),
            source: value.owners,
            target: value.workers,
        })
    );

    let mut dangling_names = value.node_names.clone();
    dangling_names.remove(&value.owners);
    dangling_names.insert(NodeId(999), "owners".to_owned());
    assert_eq!(
        StableElementResolverV1::seal(
            &value.graph,
            "demo/world",
            &dangling_names,
            &value.hyperedge_names,
        ),
        Err(StableIdentityError::MissingNodeName { node: value.owners })
    );
}

#[test]
fn sealed_resolver_detects_every_topology_mutation() {
    let value = fixture();
    let resolver = resolver(&value);
    resolver.validate_topology(&value.graph).unwrap();

    let mut added = value.graph.clone();
    added.add_node("class").unwrap();
    assert_eq!(
        resolver.validate_topology(&added),
        Err(StableIdentityError::TopologyChanged)
    );

    let mut removed = value.graph.clone();
    removed.remove_node(value.owners).unwrap();
    assert_eq!(
        resolver.validate_topology(&removed),
        Err(StableIdentityError::TopologyChanged)
    );

    let mut edge_removed = value.graph.clone();
    edge_removed
        .remove_edge("solidarity", value.workers, value.owners)
        .unwrap();
    assert_eq!(
        resolver.validate_topology(&edge_removed),
        Err(StableIdentityError::TopologyChanged)
    );

    let mut membership_changed = value.graph.clone();
    membership_changed
        .remove_hyperedge(value.coalition_a)
        .unwrap();
    membership_changed
        .add_hyperedge("coalition", &[value.workers])
        .unwrap();
    assert_eq!(
        resolver.validate_topology(&membership_changed),
        Err(StableIdentityError::TopologyChanged)
    );
}

#[test]
fn resolver_accepts_65536_rows_and_refuses_65537_before_manifest_allocation() {
    let mut graph = MemoryGraph::new();
    let mut names = HashMap::new();
    for index in 0..MAX_STABLE_RESOLVER_ROWS_V1 {
        let node = graph.add_node("class").unwrap();
        names.insert(node, format!("n{index}"));
    }
    let maximum =
        StableElementResolverV1::seal(&graph, "demo/world", &names, &HashMap::new()).unwrap();
    assert!(maximum.manifest().canonical_bytes().len() <= MAX_STABLE_RESOLVER_MANIFEST_BYTES_V1);

    let extra = graph.add_node("class").unwrap();
    names.insert(extra, "overflow".to_owned());
    assert_eq!(
        StableElementResolverV1::seal(&graph, "demo/world", &names, &HashMap::new(),),
        Err(StableIdentityError::ResolverRowLimit {
            actual: MAX_STABLE_RESOLVER_ROWS_V1 + 1,
            maximum: MAX_STABLE_RESOLVER_ROWS_V1,
        })
    );
}

#[test]
fn forged_stable_element_cannot_enter_a_resolver_owned_carrier() {
    let value = fixture();
    let resolver = resolver(&value);
    let forged = StableElementKeyV1::Node {
        scenario: "demo/world".to_owned(),
        local_name: "ghost".to_owned(),
    };
    assert_eq!(
        resolver.carrier_key(&forged, &[], 0),
        Err(StableIdentityError::ElementNotSealed)
    );
}

struct StateFixture<G> {
    graph: G,
    node_names: HashMap<NodeId, String>,
    hyperedge_names: HashMap<HyperedgeId, String>,
}

#[derive(Clone)]
struct StateFacts {
    nodes: Vec<(NodeId, String)>,
    node_f64: Vec<(NodeId, String, f64)>,
    edges: Vec<(String, NodeId, NodeId, f64)>,
    hyperedges: Vec<(HyperedgeId, String, Vec<NodeId>)>,
    edge_f64: Vec<(String, NodeId, NodeId, String, f64)>,
    node_currency: Vec<(NodeId, String, Currency)>,
    hyperedge_f64: Vec<(HyperedgeId, String, f64)>,
}

impl StateFacts {
    fn from_graph(graph: &MemoryGraph) -> Self {
        Self {
            nodes: graph.all_nodes(),
            node_f64: graph.all_attributes(),
            edges: graph.all_edges(),
            hyperedges: graph.all_hyperedges(),
            edge_f64: graph.all_edge_attributes(),
            node_currency: graph.all_currency_attributes(),
            hyperedge_f64: graph.all_hyperedge_attributes(),
        }
    }
}

impl CanonicalState for StateFacts {
    fn all_nodes(&self) -> Vec<(NodeId, String)> {
        self.nodes.clone()
    }
    fn all_attributes(&self) -> Vec<(NodeId, String, f64)> {
        self.node_f64.clone()
    }
    fn all_edges(&self) -> Vec<(String, NodeId, NodeId, f64)> {
        self.edges.clone()
    }
    fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)> {
        self.hyperedges.clone()
    }
    fn all_edge_attributes(&self) -> Vec<(String, NodeId, NodeId, String, f64)> {
        self.edge_f64.clone()
    }
    fn all_currency_attributes(&self) -> Vec<(NodeId, String, Currency)> {
        self.node_currency.clone()
    }
    fn all_hyperedge_attributes(&self) -> Vec<(HyperedgeId, String, f64)> {
        self.hyperedge_f64.clone()
    }
}

fn state_fixture<G: GraphSubstrate + Default>(shift_handles: bool) -> StateFixture<G> {
    let mut graph = G::default();
    if shift_handles {
        let discarded = graph.add_node("discarded").unwrap();
        graph.remove_node(discarded).unwrap();
    }
    let (owners, workers) = if shift_handles {
        let workers = graph.add_node("class").unwrap();
        let owners = graph.add_node("class").unwrap();
        (owners, workers)
    } else {
        let owners = graph.add_node("class").unwrap();
        let workers = graph.add_node("class").unwrap();
        (owners, workers)
    };
    graph.add_edge("solidarity", workers, owners, 0.75).unwrap();
    if shift_handles {
        let discarded = graph
            .add_hyperedge("discarded", &[owners, workers])
            .unwrap();
        graph.remove_hyperedge(discarded).unwrap();
    }
    let coalition = graph
        .add_hyperedge("coalition", &[workers, owners])
        .unwrap();
    graph.update_node(owners, "class/power", -0.0).unwrap();
    graph.update_node(workers, "class/wage", 1.5).unwrap();
    graph
        .update_node_currency(
            owners,
            "class/wealth",
            Currency::from_micro_units(-123_456_789),
        )
        .unwrap();
    graph
        .update_edge("solidarity", workers, owners, "solidarity/tension", -2.5)
        .unwrap();
    graph
        .update_hyperedge_attribute(coalition, "coalition/cohesion", 0.25)
        .unwrap();
    StateFixture {
        graph,
        node_names: HashMap::from([
            (workers, "workers".to_owned()),
            (owners, "owners".to_owned()),
        ]),
        hyperedge_names: HashMap::from([(coalition, "coalition-a".to_owned())]),
    }
}

fn stable_state<G: GraphSubstrate + CanonicalState>(value: &StateFixture<G>) -> StableGraphStateV1 {
    let resolver = StableElementResolverV1::seal(
        &value.graph,
        "demo/world",
        &value.node_names,
        &value.hyperedge_names,
    )
    .unwrap();
    encode_stable_graph_state_v1(&value.graph, &resolver).unwrap()
}

#[test]
fn stable_graph_state_eight_sections_and_scalar_bytes_are_exact() {
    let value = state_fixture::<MemoryGraph>(false);
    let expected = [
        b"babylon.stable-graph\0".as_slice(),
        &1_u32.to_be_bytes(),
        &[0x01],
        &str32("demo/world"),
        &[0x02],
        &2_u32.to_be_bytes(),
        &str32("owners"),
        &str32("class"),
        &str32("workers"),
        &str32("class"),
        &[0x03],
        &2_u32.to_be_bytes(),
        &str32("owners"),
        &str32("class/power"),
        &0_u64.to_be_bytes(),
        &str32("workers"),
        &str32("class/wage"),
        &1.5_f64.to_bits().to_be_bytes(),
        &[0x04],
        &1_u32.to_be_bytes(),
        &str32("solidarity"),
        &str32("workers"),
        &str32("owners"),
        &0.75_f64.to_bits().to_be_bytes(),
        &[0x05],
        &1_u32.to_be_bytes(),
        &str32("coalition-a"),
        &str32("coalition"),
        &2_u32.to_be_bytes(),
        &str32("owners"),
        &str32("workers"),
        &[0x06],
        &1_u32.to_be_bytes(),
        &str32("solidarity"),
        &str32("workers"),
        &str32("owners"),
        &str32("solidarity/tension"),
        &(-2.5_f64).to_bits().to_be_bytes(),
        &[0x07],
        &1_u32.to_be_bytes(),
        &str32("owners"),
        &str32("class/wealth"),
        &(-123_456_789_i128).to_be_bytes(),
        &[0x08],
        &1_u32.to_be_bytes(),
        &str32("coalition-a"),
        &str32("coalition/cohesion"),
        &0.25_f64.to_bits().to_be_bytes(),
    ]
    .concat();

    assert_eq!(stable_state(&value).canonical_bytes(), expected);
}

#[test]
fn stable_graph_state_writes_every_empty_section() {
    let mut graph = MemoryGraph::new();
    let node = graph.add_node("class").unwrap();
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/world",
        &HashMap::from([(node, "workers".to_owned())]),
        &HashMap::new(),
    )
    .unwrap();
    let state = encode_stable_graph_state_v1(&graph, &resolver).unwrap();
    let expected = [
        b"babylon.stable-graph\0".as_slice(),
        &1_u32.to_be_bytes(),
        &[0x01],
        &str32("demo/world"),
        &[0x02],
        &1_u32.to_be_bytes(),
        &str32("workers"),
        &str32("class"),
        &[0x03],
        &0_u32.to_be_bytes(),
        &[0x04],
        &0_u32.to_be_bytes(),
        &[0x05],
        &0_u32.to_be_bytes(),
        &[0x06],
        &0_u32.to_be_bytes(),
        &[0x07],
        &0_u32.to_be_bytes(),
        &[0x08],
        &0_u32.to_be_bytes(),
    ]
    .concat();
    assert_eq!(state.canonical_bytes(), expected);
}

#[test]
fn stable_graph_state_ignores_substrate_and_runtime_handle_allocation() {
    let memory = state_fixture::<MemoryGraph>(false);
    let hypergraph = state_fixture::<HypergraphStore>(true);
    assert_ne!(
        memory.graph.state_hash().unwrap(),
        hypergraph.graph.state_hash().unwrap(),
        "the fixture must actually exercise different runtime handles"
    );
    let memory_state = stable_state(&memory);
    let hypergraph_state = stable_state(&hypergraph);
    assert_eq!(
        memory_state.canonical_bytes(),
        hypergraph_state.canonical_bytes()
    );
    assert_eq!(memory_state.digest(), hypergraph_state.digest());
}

#[test]
fn stable_graph_state_refuses_topology_and_fact_ambiguity() {
    let value = state_fixture::<MemoryGraph>(false);
    let resolver = StableElementResolverV1::seal(
        &value.graph,
        "demo/world",
        &value.node_names,
        &value.hyperedge_names,
    )
    .unwrap();
    let owners = *value
        .node_names
        .iter()
        .find_map(|(node, name)| (name == "owners").then_some(node))
        .unwrap();

    let mut changed = value.graph.clone();
    changed.add_node("class").unwrap();
    assert_eq!(
        encode_stable_graph_state_v1(&changed, &resolver),
        Err(StableIdentityError::TopologyChanged)
    );

    let mut collision = value.graph.clone();
    collision
        .update_node_currency(owners, "class/power", Currency::from_micro_units(1))
        .unwrap();
    assert!(matches!(
        encode_stable_graph_state_v1(&collision, &resolver),
        Err(StableIdentityError::NumericLaneCollision { .. })
    ));

    let mut duplicate = StateFacts::from_graph(&value.graph);
    duplicate.node_f64.push(duplicate.node_f64[0].clone());
    assert_eq!(
        encode_stable_graph_state_v1(&duplicate, &resolver),
        Err(StableIdentityError::DuplicateFact {
            section: "node f64 attributes",
        })
    );

    let mut unknown_owner = StateFacts::from_graph(&value.graph);
    unknown_owner
        .node_f64
        .push((NodeId(999), "class/ghost".to_owned(), 1.0));
    assert_eq!(
        encode_stable_graph_state_v1(&unknown_owner, &resolver),
        Err(StableIdentityError::UnknownNode { node: NodeId(999) })
    );

    let mut absent_edge = StateFacts::from_graph(&value.graph);
    absent_edge.edge_f64.push((
        "solidarity".to_owned(),
        owners,
        owners,
        "solidarity/tension".to_owned(),
        1.0,
    ));
    assert!(matches!(
        encode_stable_graph_state_v1(&absent_edge, &resolver),
        Err(StableIdentityError::UnknownEdge { .. })
    ));

    let mut duplicate_member = StateFacts::from_graph(&value.graph);
    duplicate_member.hyperedges[0].2.push(owners);
    assert_eq!(
        encode_stable_graph_state_v1(&duplicate_member, &resolver),
        Err(StableIdentityError::InvalidHyperedge {
            hyperedge: duplicate_member.hyperedges[0].0,
        })
    );

    let mut empty_hyperedge = StateFacts::from_graph(&value.graph);
    empty_hyperedge.hyperedges[0].2.clear();
    assert!(matches!(
        encode_stable_graph_state_v1(&empty_hyperedge, &resolver),
        Err(StableIdentityError::InvalidHyperedge { .. })
    ));

    let mut unknown_member = StateFacts::from_graph(&value.graph);
    unknown_member.hyperedges[0].2[0] = NodeId(999);
    assert!(matches!(
        encode_stable_graph_state_v1(&unknown_member, &resolver),
        Err(StableIdentityError::InvalidHyperedge { .. })
    ));
}

#[test]
fn stable_graph_state_refuses_non_finite_values_and_strength_aliases() {
    let value = state_fixture::<MemoryGraph>(false);
    let resolver = StableElementResolverV1::seal(
        &value.graph,
        "demo/world",
        &value.node_names,
        &value.hyperedge_names,
    )
    .unwrap();
    let workers = *value
        .node_names
        .iter()
        .find_map(|(node, name)| (name == "workers").then_some(node))
        .unwrap();
    let owners = *value
        .node_names
        .iter()
        .find_map(|(node, name)| (name == "owners").then_some(node))
        .unwrap();

    let mut non_finite = value.graph.clone();
    non_finite
        .update_node(workers, "class/wage", f64::INFINITY)
        .unwrap();
    assert_eq!(
        encode_stable_graph_state_v1(&non_finite, &resolver),
        Err(StableIdentityError::NonFiniteValue {
            section: "node f64 attributes",
        })
    );

    let mut nan = StateFacts::from_graph(&value.graph);
    nan.hyperedge_f64[0].2 = f64::NAN;
    assert_eq!(
        encode_stable_graph_state_v1(&nan, &resolver),
        Err(StableIdentityError::NonFiniteValue {
            section: "hyperedge f64 attributes",
        })
    );

    let mut invalid = StateFacts::from_graph(&value.graph);
    invalid.edge_f64 = vec![(
        "solidarity".to_owned(),
        workers,
        owners,
        "solidarity/strength".to_owned(),
        1.0,
    )];
    assert_eq!(
        encode_stable_graph_state_v1(&invalid, &resolver),
        Err(StableIdentityError::StrengthAttribute)
    );
}
