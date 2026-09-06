//! Closed relation selection; SQL shape and decoding are shared by both readers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredTickReadSourceV1 {
    Runtime,
    FullObserver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredTickRelationV1 {
    ArchiveDirtyReceiptV1,
    CheckpointManifest,
    CheckpointSectionV1,
    GraphEdgeF64V1,
    GraphEdgeV1,
    GraphHyperedgeF64V1,
    GraphHyperedgeMemberV1,
    GraphHyperedgeV1,
    GraphNodeCurrencyV1,
    GraphNodeF64V1,
    GraphNodeV1,
    HexStateDeltaV1,
    OrganizationStateFieldV1,
    OrganizationStateV1,
    OrganizationTerritoryV1,
    TerritoryStateFieldV1,
    TerritoryStateV1,
    TickActionBatchV1,
    TickChoiceReceiptBranchV1,
    TickChoiceReceiptCarrierElementV1,
    TickChoiceReceiptV1,
    TickCommit,
    TickEventFieldV2,
    TickEventV2,
    WorldRegisterV1,
    MaterialTickV3,
}

impl StoredTickReadSourceV1 {
    pub(crate) const fn relation(self, relation: StoredTickRelationV1) -> &'static str {
        match self {
            Self::Runtime => relation.runtime_relation(),
            Self::FullObserver => relation.observer_relation(),
        }
    }
}

impl StoredTickRelationV1 {
    const fn runtime_relation(self) -> &'static str {
        match self {
            Self::ArchiveDirtyReceiptV1 => "babylon_state.archive_dirty_receipt_v1",
            Self::CheckpointManifest => "babylon_state.checkpoint_manifest",
            Self::CheckpointSectionV1 => "babylon_state.checkpoint_section_v1",
            Self::GraphEdgeF64V1 => "babylon_state.graph_edge_f64_v1",
            Self::GraphEdgeV1 => "babylon_state.graph_edge_v1",
            Self::GraphHyperedgeF64V1 => "babylon_state.graph_hyperedge_f64_v1",
            Self::GraphHyperedgeMemberV1 => "babylon_state.graph_hyperedge_member_v1",
            Self::GraphHyperedgeV1 => "babylon_state.graph_hyperedge_v1",
            Self::GraphNodeCurrencyV1 => "babylon_state.graph_node_currency_v1",
            Self::GraphNodeF64V1 => "babylon_state.graph_node_f64_v1",
            Self::GraphNodeV1 => "babylon_state.graph_node_v1",
            Self::HexStateDeltaV1 => "babylon_state.hex_state_delta_v1",
            Self::OrganizationStateFieldV1 => "babylon_state.organization_state_field_v1",
            Self::OrganizationStateV1 => "babylon_state.organization_state_v1",
            Self::OrganizationTerritoryV1 => "babylon_state.organization_territory_v1",
            Self::TerritoryStateFieldV1 => "babylon_state.territory_state_field_v1",
            Self::TerritoryStateV1 => "babylon_state.territory_state_v1",
            Self::TickActionBatchV1 => "babylon_state.tick_action_batch_v1",
            Self::TickChoiceReceiptBranchV1 => "babylon_state.tick_choice_receipt_branch_v1",
            Self::TickChoiceReceiptCarrierElementV1 => {
                "babylon_state.tick_choice_receipt_carrier_element_v1"
            }
            Self::TickChoiceReceiptV1 => "babylon_state.tick_choice_receipt_v1",
            Self::TickCommit => "babylon_state.tick_commit",
            Self::TickEventFieldV2 => "babylon_state.tick_event_field_v2",
            Self::TickEventV2 => "babylon_state.tick_event_v2",
            Self::WorldRegisterV1 => "babylon_state.world_register_v1",
            Self::MaterialTickV3 => "babylon_state.material_tick_v3",
        }
    }
    const fn observer_relation(self) -> &'static str {
        match self {
            Self::ArchiveDirtyReceiptV1 => "public.v_observer_archive_dirty_receipt_v1",
            Self::CheckpointManifest => "public.v_observer_checkpoint_manifest",
            Self::CheckpointSectionV1 => "public.v_observer_checkpoint_section_v1",
            Self::GraphEdgeF64V1 => "public.v_observer_graph_edge_f64_v1",
            Self::GraphEdgeV1 => "public.v_observer_graph_edge_v1",
            Self::GraphHyperedgeF64V1 => "public.v_observer_graph_hyperedge_f64_v1",
            Self::GraphHyperedgeMemberV1 => "public.v_observer_graph_hyperedge_member_v1",
            Self::GraphHyperedgeV1 => "public.v_observer_graph_hyperedge_v1",
            Self::GraphNodeCurrencyV1 => "public.v_observer_graph_node_currency_v1",
            Self::GraphNodeF64V1 => "public.v_observer_graph_node_f64_v1",
            Self::GraphNodeV1 => "public.v_observer_graph_node_v1",
            Self::HexStateDeltaV1 => "public.v_observer_hex_state_delta_v1",
            Self::OrganizationStateFieldV1 => "public.v_observer_organization_state_field_v1",
            Self::OrganizationStateV1 => "public.v_observer_organization_state_v1",
            Self::OrganizationTerritoryV1 => "public.v_observer_organization_territory_v1",
            Self::TerritoryStateFieldV1 => "public.v_observer_territory_state_field_v1",
            Self::TerritoryStateV1 => "public.v_observer_territory_state_v1",
            Self::TickActionBatchV1 => "public.v_observer_tick_action_batch_v1",
            Self::TickChoiceReceiptBranchV1 => "public.v_observer_tick_choice_receipt_branch_v1",
            Self::TickChoiceReceiptCarrierElementV1 => {
                "public.v_observer_tick_choice_receipt_carrier_element_v1"
            }
            Self::TickChoiceReceiptV1 => "public.v_observer_tick_choice_receipt_v1",
            Self::TickCommit => "public.v_committed_tick_status_v1",
            Self::TickEventFieldV2 => "public.v_observer_tick_event_field_v2",
            Self::TickEventV2 => "public.v_observer_tick_event_v2",
            Self::WorldRegisterV1 => "public.v_observer_world_register_v1",
            Self::MaterialTickV3 => "public.v_observer_material_state_v1",
        }
    }
}
