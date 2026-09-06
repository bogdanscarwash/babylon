-- Complete persisted components for authenticated material-tick reconstruction.
-- Only the full observer receives these rows; known and Archive readers do not.

CREATE VIEW public.v_observer_graph_node_v1 AS
SELECT component.*
FROM babylon_state.graph_node_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_node_f64_v1 AS
SELECT component.*
FROM babylon_state.graph_node_f64_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_edge_v1 AS
SELECT component.*
FROM babylon_state.graph_edge_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_hyperedge_v1 AS
SELECT component.*
FROM babylon_state.graph_hyperedge_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_hyperedge_member_v1 AS
SELECT component.*
FROM babylon_state.graph_hyperedge_member_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_edge_f64_v1 AS
SELECT component.*
FROM babylon_state.graph_edge_f64_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_node_currency_v1 AS
SELECT component.*
FROM babylon_state.graph_node_currency_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_graph_hyperedge_f64_v1 AS
SELECT component.*
FROM babylon_state.graph_hyperedge_f64_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_world_register_v1 AS
SELECT component.*
FROM babylon_state.world_register_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_hex_state_delta_v1 AS
SELECT component.*
FROM babylon_state.hex_state_delta_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_territory_state_v1 AS
SELECT component.*
FROM babylon_state.territory_state_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_territory_state_field_v1 AS
SELECT component.*
FROM babylon_state.territory_state_field_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_organization_state_v1 AS
SELECT component.*
FROM babylon_state.organization_state_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_organization_state_field_v1 AS
SELECT component.*
FROM babylon_state.organization_state_field_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_organization_territory_v1 AS
SELECT component.*
FROM babylon_state.organization_territory_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_tick_event_v2 AS
SELECT component.*
FROM babylon_state.tick_event_v2 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_tick_event_field_v2 AS
SELECT component.*
FROM babylon_state.tick_event_field_v2 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_tick_choice_receipt_v1 AS
SELECT component.*
FROM babylon_state.tick_choice_receipt_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_tick_choice_receipt_branch_v1 AS
SELECT component.*
FROM babylon_state.tick_choice_receipt_branch_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_tick_choice_receipt_carrier_element_v1 AS
SELECT component.*
FROM babylon_state.tick_choice_receipt_carrier_element_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_checkpoint_manifest AS
SELECT component.*
FROM babylon_state.checkpoint_manifest AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_checkpoint_section_v1 AS
SELECT component.*
FROM babylon_state.checkpoint_section_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_archive_dirty_receipt_v1 AS
SELECT component.*
FROM babylon_state.archive_dirty_receipt_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

CREATE VIEW public.v_observer_tick_action_batch_v1 AS
SELECT component.*
FROM babylon_state.tick_action_batch_v1 AS component
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = component.campaign_id
 AND marker.resolve_tick = component.resolve_tick
WHERE marker.envelope_layout_version = 3;

REVOKE ALL ON
    public.v_observer_graph_node_v1,
    public.v_observer_graph_node_f64_v1,
    public.v_observer_graph_edge_v1,
    public.v_observer_graph_hyperedge_v1,
    public.v_observer_graph_hyperedge_member_v1,
    public.v_observer_graph_edge_f64_v1,
    public.v_observer_graph_node_currency_v1,
    public.v_observer_graph_hyperedge_f64_v1,
    public.v_observer_world_register_v1,
    public.v_observer_hex_state_delta_v1,
    public.v_observer_territory_state_v1,
    public.v_observer_territory_state_field_v1,
    public.v_observer_organization_state_v1,
    public.v_observer_organization_state_field_v1,
    public.v_observer_organization_territory_v1,
    public.v_observer_tick_event_v2,
    public.v_observer_tick_event_field_v2,
    public.v_observer_tick_choice_receipt_v1,
    public.v_observer_tick_choice_receipt_branch_v1,
    public.v_observer_tick_choice_receipt_carrier_element_v1,
    public.v_observer_checkpoint_manifest,
    public.v_observer_checkpoint_section_v1,
    public.v_observer_archive_dirty_receipt_v1,
    public.v_observer_tick_action_batch_v1
FROM PUBLIC;

GRANT SELECT ON
    public.v_observer_graph_node_v1,
    public.v_observer_graph_node_f64_v1,
    public.v_observer_graph_edge_v1,
    public.v_observer_graph_hyperedge_v1,
    public.v_observer_graph_hyperedge_member_v1,
    public.v_observer_graph_edge_f64_v1,
    public.v_observer_graph_node_currency_v1,
    public.v_observer_graph_hyperedge_f64_v1,
    public.v_observer_world_register_v1,
    public.v_observer_hex_state_delta_v1,
    public.v_observer_territory_state_v1,
    public.v_observer_territory_state_field_v1,
    public.v_observer_organization_state_v1,
    public.v_observer_organization_state_field_v1,
    public.v_observer_organization_territory_v1,
    public.v_observer_tick_event_v2,
    public.v_observer_tick_event_field_v2,
    public.v_observer_tick_choice_receipt_v1,
    public.v_observer_tick_choice_receipt_branch_v1,
    public.v_observer_tick_choice_receipt_carrier_element_v1,
    public.v_observer_checkpoint_manifest,
    public.v_observer_checkpoint_section_v1,
    public.v_observer_archive_dirty_receipt_v1,
    public.v_observer_tick_action_batch_v1
TO babylon_observer;
