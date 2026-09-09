-- Explicit successor: existing graph-only foundations cannot gain a material row.
CREATE TABLE babylon_state.material_campaign_foundation_v2 (
    campaign_id uuid PRIMARY KEY REFERENCES babylon_state.campaign(campaign_id),
    preset_id text NOT NULL CHECK (octet_length(preset_id) BETWEEN 1 AND 128),
    horizon_ticks bigint NOT NULL CHECK (horizon_ticks > 0),
    content_sha256 bytea NOT NULL CHECK (octet_length(content_sha256) = 32),
    initial_register_bytes bytea NOT NULL CHECK (octet_length(initial_register_bytes) <= 67108864),
    foundation_bytes bytea NOT NULL CHECK (octet_length(foundation_bytes) <= 67108864),
    foundation_sha256 bytea NOT NULL CHECK (octet_length(foundation_sha256) = 32)
);
CREATE TABLE babylon_state.material_tick_v3 (
    campaign_id uuid NOT NULL REFERENCES babylon_state.material_campaign_foundation_v2(campaign_id),
    resolve_tick bigint NOT NULL CHECK (resolve_tick > 0),
    identity_bytes bytea NOT NULL CHECK (octet_length(identity_bytes) <= 1024),
    register_bytes bytea NOT NULL CHECK (octet_length(register_bytes) <= 67108864),
    receipt_bytes bytea NOT NULL CHECK (octet_length(receipt_bytes) <= 67108864),
    PRIMARY KEY (campaign_id, resolve_tick)
);
ALTER TABLE babylon_state.tick_commit DROP CONSTRAINT tick_commit_envelope_layout_v2;
ALTER TABLE babylon_state.tick_commit ADD CONSTRAINT tick_commit_envelope_layout_v3 CHECK (envelope_layout_version IN (2, 3));
CREATE FUNCTION babylon_state.require_material_tick_v3() RETURNS trigger
LANGUAGE plpgsql SET search_path = pg_catalog AS $$
DECLARE material_campaign boolean;
BEGIN
    SELECT EXISTS (SELECT 1 FROM babylon_state.material_campaign_foundation_v2 f WHERE f.campaign_id = NEW.campaign_id) INTO material_campaign;
    IF material_campaign <> (NEW.envelope_layout_version = 3) THEN
        RAISE EXCEPTION 'material campaign version mismatch';
    END IF;
    IF material_campaign AND NOT EXISTS (SELECT 1 FROM babylon_state.material_tick_v3 t WHERE t.campaign_id = NEW.campaign_id AND t.resolve_tick = NEW.resolve_tick) THEN
        RAISE EXCEPTION 'material tick missing before commit marker';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER material_tick_marker_v3 BEFORE INSERT ON babylon_state.tick_commit FOR EACH ROW EXECUTE FUNCTION babylon_state.require_material_tick_v3();
REVOKE ALL ON FUNCTION babylon_state.require_material_tick_v3() FROM PUBLIC;
REVOKE ALL ON babylon_state.material_campaign_foundation_v2, babylon_state.material_tick_v3 FROM PUBLIC;
