CREATE VIEW public.v_material_campaign_identity_v1 AS
SELECT campaign_id, preset_id, horizon_ticks, content_sha256, foundation_sha256
FROM babylon_state.material_campaign_foundation_v2;

-- Only the observer group receives complete material bytes. Known preview has
-- no grant on this view or the underlying tables; projection cannot undo that.
CREATE VIEW public.v_observer_material_state_v1 AS
SELECT campaign_id, 0::bigint AS resolve_tick, initial_register_bytes AS register_bytes,
       NULL::bytea AS receipt_bytes, NULL::bytea AS identity_bytes,
       NULL::bytea AS tick_content_hash, foundation_bytes
FROM babylon_state.material_campaign_foundation_v2
UNION ALL
SELECT state.campaign_id, state.resolve_tick, state.register_bytes,
       state.receipt_bytes, state.identity_bytes, marker.tick_content_hash,
       NULL::bytea AS foundation_bytes
FROM babylon_state.material_tick_v3 AS state
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = state.campaign_id AND marker.resolve_tick = state.resolve_tick
WHERE marker.envelope_layout_version = 3;

REVOKE ALL ON public.v_material_campaign_identity_v1,
              public.v_observer_material_state_v1 FROM PUBLIC;
GRANT SELECT ON public.v_material_campaign_identity_v1 TO babylon_observer, babylon_reader;
GRANT SELECT ON public.v_observer_material_state_v1 TO babylon_observer;
