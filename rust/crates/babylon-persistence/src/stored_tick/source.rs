//! Closed relation selection; SQL shape and decoding are shared by both readers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredTickReadSource {
    Runtime,
    FullObserver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredTickRelation {
    ArchiveDirtyReceipt,
    CheckpointManifest,
    CheckpointSection,
    GraphEdgeF64,
    GraphEdge,
    GraphHyperedgeF64,
    GraphHyperedgeMember,
    GraphHyperedge,
    GraphNodeCurrency,
    GraphNodeF64,
    GraphNode,
    HexStateDelta,
    OrganizationStateField,
    OrganizationState,
    OrganizationTerritory,
    TerritoryStateField,
    TerritoryState,
    TickActionBatch,
    TickChoiceReceiptBranch,
    TickChoiceReceiptCarrierElement,
    TickChoiceReceipt,
    TickCommit,
    TickEventField,
    TickEvent,
    WorldRegister,
    MaterialTick,
}

impl StoredTickReadSource {
    pub(crate) const fn relation(self, relation: StoredTickRelation) -> &'static str {
        match self {
            Self::Runtime => relation.runtime_relation(),
            Self::FullObserver => relation.observer_relation(),
        }
    }
}

impl StoredTickRelation {
    const fn runtime_relation(self) -> &'static str {
        match self {
            Self::ArchiveDirtyReceipt => "babylon_state.archive_dirty_receipt_v1",
            Self::CheckpointManifest => "babylon_state.checkpoint_manifest",
            Self::CheckpointSection => "babylon_state.checkpoint_section_v1",
            Self::GraphEdgeF64 => "babylon_state.graph_edge_f64_v1",
            Self::GraphEdge => "babylon_state.graph_edge_v1",
            Self::GraphHyperedgeF64 => "babylon_state.graph_hyperedge_f64_v1",
            Self::GraphHyperedgeMember => "babylon_state.graph_hyperedge_member_v1",
            Self::GraphHyperedge => "babylon_state.graph_hyperedge_v1",
            Self::GraphNodeCurrency => "babylon_state.graph_node_currency_v1",
            Self::GraphNodeF64 => "babylon_state.graph_node_f64_v1",
            Self::GraphNode => "babylon_state.graph_node_v1",
            Self::HexStateDelta => "babylon_state.hex_state_delta_v1",
            Self::OrganizationStateField => "babylon_state.organization_state_field_v1",
            Self::OrganizationState => "babylon_state.organization_state_v1",
            Self::OrganizationTerritory => "babylon_state.organization_territory_v1",
            Self::TerritoryStateField => "babylon_state.territory_state_field_v1",
            Self::TerritoryState => "babylon_state.territory_state_v1",
            Self::TickActionBatch => "babylon_state.tick_action_batch_v1",
            Self::TickChoiceReceiptBranch => "babylon_state.tick_choice_receipt_branch_v1",
            Self::TickChoiceReceiptCarrierElement => {
                "babylon_state.tick_choice_receipt_carrier_element_v1"
            }
            Self::TickChoiceReceipt => "babylon_state.tick_choice_receipt_v1",
            Self::TickCommit => "babylon_state.tick_commit",
            Self::TickEventField => "babylon_state.tick_event_field_v2",
            Self::TickEvent => "babylon_state.tick_event_v2",
            Self::WorldRegister => "babylon_state.world_register_v1",
            Self::MaterialTick => "babylon_state.material_tick_v3",
        }
    }
    const fn observer_relation(self) -> &'static str {
        match self {
            Self::ArchiveDirtyReceipt => "public.v_observer_archive_dirty_receipt_v1",
            Self::CheckpointManifest => "public.v_observer_checkpoint_manifest",
            Self::CheckpointSection => "public.v_observer_checkpoint_section_v1",
            Self::GraphEdgeF64 => "public.v_observer_graph_edge_f64_v1",
            Self::GraphEdge => "public.v_observer_graph_edge_v1",
            Self::GraphHyperedgeF64 => "public.v_observer_graph_hyperedge_f64_v1",
            Self::GraphHyperedgeMember => "public.v_observer_graph_hyperedge_member_v1",
            Self::GraphHyperedge => "public.v_observer_graph_hyperedge_v1",
            Self::GraphNodeCurrency => "public.v_observer_graph_node_currency_v1",
            Self::GraphNodeF64 => "public.v_observer_graph_node_f64_v1",
            Self::GraphNode => "public.v_observer_graph_node_v1",
            Self::HexStateDelta => "public.v_observer_hex_state_delta_v1",
            Self::OrganizationStateField => "public.v_observer_organization_state_field_v1",
            Self::OrganizationState => "public.v_observer_organization_state_v1",
            Self::OrganizationTerritory => "public.v_observer_organization_territory_v1",
            Self::TerritoryStateField => "public.v_observer_territory_state_field_v1",
            Self::TerritoryState => "public.v_observer_territory_state_v1",
            Self::TickActionBatch => "public.v_observer_tick_action_batch_v1",
            Self::TickChoiceReceiptBranch => "public.v_observer_tick_choice_receipt_branch_v1",
            Self::TickChoiceReceiptCarrierElement => {
                "public.v_observer_tick_choice_receipt_carrier_element_v1"
            }
            Self::TickChoiceReceipt => "public.v_observer_tick_choice_receipt_v1",
            Self::TickCommit => "public.v_committed_tick_status_v1",
            Self::TickEventField => "public.v_observer_tick_event_field_v2",
            Self::TickEvent => "public.v_observer_tick_event_v2",
            Self::WorldRegister => "public.v_observer_world_register_v1",
            Self::MaterialTick => "public.v_observer_material_state_v1",
        }
    }
}
