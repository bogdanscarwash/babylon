-- Additive client-owned read-only reader schema; not a persistence-authority epoch.
-- The view lives in the public schema because PostgreSQL grants schema USAGE on
-- public to PUBLIC, while babylon_meta is revoked from PUBLIC. A schema-level
-- GRANT USAGE on babylon_meta would change the digest-pinned babylon_meta
-- schema-grant census row, so the view stays where the reader can already reach.
-- babylon_reader is NOLOGIN by design: a deployment provisions one confined
-- LOGIN role as a member of babylon_reader (NOSUPERUSER NOCREATEDB
-- NOCREATEROLE) and points BABYLON_READER_DSN at that credential. The reader
-- handle refuses to operate unless the session privilege census is exactly
-- the footprint: SELECT on this view before the atom schema, SELECT on this
-- view plus the four atom views after it.
CREATE VIEW public.v_committed_tick_status_v1 AS
SELECT
    campaign_id,
    resolve_tick,
    envelope_layout_version,
    tick_content_hash,
    envelope_digest
FROM babylon_state.tick_commit;

GRANT SELECT ON public.v_committed_tick_status_v1 TO babylon_reader;

-- Guarded atom-view grants: when the additive atom schema is already
-- installed, a fresh reader role must hold SELECT on its four fog-safe views
-- too. When the atom schema is absent this block is a no-op, and when the
-- atom schema installs later its own migration grants these views to an
-- existing role, so either install order reconciles to the same footprint.
DO $$
BEGIN
    IF pg_catalog.to_regclass('public.v_archive_page_known_v1') IS NOT NULL THEN
        GRANT SELECT ON public.v_archive_page_known_v1 TO babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('public.v_archive_atom_visible') IS NOT NULL THEN
        GRANT SELECT ON public.v_archive_atom_visible TO babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('public.v_county_card_atoms') IS NOT NULL THEN
        GRANT SELECT ON public.v_county_card_atoms TO babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('public.v_archive_subject_atoms') IS NOT NULL THEN
        GRANT SELECT ON public.v_archive_subject_atoms TO babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('public.v_archive_verification_v1') IS NOT NULL THEN
        GRANT SELECT ON public.v_archive_verification_v1 TO babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('public.v_known_county_economy_v1') IS NOT NULL THEN
        GRANT SELECT ON public.v_known_county_economy_v1 TO babylon_reader;
        GRANT SELECT ON public.v_observer_economy_foundation_v1 TO babylon_reader;
    END IF;
END
$$;

-- The material identity header contains no unfiltered numeric state.
DO $$
BEGIN
    IF pg_catalog.to_regclass('public.v_material_campaign_identity_v1') IS NOT NULL THEN
        GRANT SELECT ON public.v_material_campaign_identity_v1 TO babylon_reader;
    END IF;
END
$$;

-- Defense in depth: these tables are already revoked from PUBLIC, and the role
-- inherits nothing, but pin the refusal explicitly when the Archive schema is
-- present. The guard keeps this file installable before the Archive schema.
DO $$
BEGIN
    IF pg_catalog.to_regclass('babylon_meta.archive_page_v1') IS NOT NULL THEN
        REVOKE ALL ON TABLE babylon_meta.archive_page_v1 FROM babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('babylon_meta.archive_knowledge_grant_v1') IS NOT NULL THEN
        REVOKE ALL ON TABLE babylon_meta.archive_knowledge_grant_v1 FROM babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('babylon_meta.archive_receipt_consumption_v1') IS NOT NULL THEN
        REVOKE ALL ON TABLE babylon_meta.archive_receipt_consumption_v1 FROM babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('babylon_meta.archive_atom_v1') IS NOT NULL THEN
        REVOKE ALL ON TABLE babylon_meta.archive_atom_v1 FROM babylon_reader;
    END IF;
    IF pg_catalog.to_regclass('babylon_meta.archive_page_atom_v1') IS NOT NULL THEN
        REVOKE ALL ON TABLE babylon_meta.archive_page_atom_v1 FROM babylon_reader;
    END IF;
END
$$;

-- A fresh confined login receives only the sole live revision views after cutover.
DO $$ BEGIN
    IF pg_catalog.to_regclass('public.v_archive_revision_known_v2') IS NOT NULL THEN
        GRANT SELECT ON public.v_archive_revision_known_v2, public.v_archive_revision_index_v2,
            public.v_archive_revision_atom_v2, public.v_archive_revision_grant_v2,
            public.v_archive_retention_v2, public.v_archive_subject_grant_v2,
            public.v_archive_tick_knowledge_v2, public.v_archive_revision_scope_v2 TO babylon_reader;
        REVOKE ALL ON babylon_meta.archive_page_retired_v1, babylon_meta.archive_page_atom_retired_v1,
            babylon_meta.archive_revision_schema_v2, babylon_meta.archive_retention_v2,
            babylon_meta.archive_page_revision_v2, babylon_meta.archive_revision_atom_v2,
            babylon_meta.archive_revision_grant_v2, babylon_meta.archive_retention_seal_v2,
            babylon_meta.archive_tick_knowledge_v2, babylon_meta.archive_tick_knowledge_member_v2 FROM babylon_reader;
    END IF;
END $$;
