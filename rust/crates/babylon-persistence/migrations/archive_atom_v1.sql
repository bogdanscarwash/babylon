-- Additive client-owned semantic Archive atom schema (ADR249 R1, R2, R8, R12);
-- not a persistence-authority epoch. These bytes fold into the Archive worker
-- contract identity beside the hash-pinned semantic_archive_v1.sql, which this
-- file never edits. Immutability is writer-code-path + REVOKE + contract: no
-- trigger appears here or anywhere in the migration estate.
CREATE TABLE babylon_meta.archive_atom_schema_v1 (
    contract_id TEXT PRIMARY KEY CHECK (contract_id = 'babylon.archive-atom-schema.v1')
);

-- Immutable, content-addressed semantic atoms. atom_id is SHA-256 of the
-- canonical encoding pinned by contracts/archive_atom_v1.yaml; identical bytes
-- re-mint to the identical atom_id, so writer retries are idempotent. Atoms
-- never mutate: new ticks mint new atoms and pages re-assert their atom set
-- through archive_page_atom_v1.
CREATE TABLE babylon_meta.archive_atom_v1 (
    atom_id BYTEA PRIMARY KEY CHECK (pg_catalog.octet_length(atom_id) = 32),
    campaign_id UUID NOT NULL
        REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place', 'concept')),
    subject_id TEXT NOT NULL CHECK (
        (subject_kind = 'county' AND subject_id ~ '^[0-9]{5}$') OR
        (subject_kind = 'place' AND subject_id ~ '^[0-9]{7}$') OR
        (subject_kind = 'concept' AND subject_id ~ '^[a-z0-9][a-z0-9-]{0,127}$')
    ),
    signal_key TEXT NOT NULL CHECK (signal_key ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    grant_key TEXT NOT NULL CHECK (grant_key ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    evidence_class TEXT NOT NULL CHECK (
        evidence_class IN ('Observed', 'Derived', 'Calibrated', 'Designed')
    ),
    value_kind TEXT NOT NULL CHECK (value_kind IN ('text', 'f64', 'u64', 'bool')),
    value_text TEXT CHECK (
        value_text IS NULL OR pg_catalog.octet_length(value_text) BETWEEN 1 AND 4096
    ),
    -- SQL-level finite backing for R1: NaN refuses (value = value), and both
    -- infinities refuse (abs(value) <> Infinity). The canonical encoding also
    -- normalizes -0.0 to +0.0, so -0.0 never mints a second identity.
    value_f64 FLOAT8 CHECK (
        value_f64 IS NULL OR (
            value_f64 = value_f64
            AND pg_catalog.abs(value_f64) <> 'Infinity'::float8
        )
    ),
    value_u64 BIGINT CHECK (value_u64 IS NULL OR value_u64 >= 0),
    value_bool BOOLEAN,
    provenance_source_id TEXT NOT NULL CHECK (
        pg_catalog.octet_length(provenance_source_id) BETWEEN 1 AND 4096
    ),
    provenance_locator TEXT NOT NULL CHECK (
        pg_catalog.octet_length(provenance_locator) BETWEEN 1 AND 4096
    ),
    valid_tick BIGINT NOT NULL CHECK (valid_tick >= 0),
    CHECK ((value_kind = 'text') = (value_text IS NOT NULL)),
    CHECK ((value_kind = 'f64') = (value_f64 IS NOT NULL)),
    CHECK ((value_kind = 'u64') = (value_u64 IS NOT NULL)),
    CHECK ((value_kind = 'bool') = (value_bool IS NOT NULL))
);

-- Composition record and staleness reverse index: which atoms each page
-- asserts, in position order. The one DELETE the writer performs replaces a
-- superseded page's join rows inside the monotonic-guarded upsert window;
-- the atoms themselves are never touched.
CREATE TABLE babylon_meta.archive_page_atom_v1 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place')),
    subject_id TEXT NOT NULL,
    atom_id BYTEA NOT NULL CHECK (pg_catalog.octet_length(atom_id) = 32)
        REFERENCES babylon_meta.archive_atom_v1(atom_id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    source_resolve_tick BIGINT NOT NULL CHECK (source_resolve_tick >= 1),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, atom_id),
    FOREIGN KEY (campaign_id, subject_kind, subject_id)
        REFERENCES babylon_meta.archive_page_v1(campaign_id, subject_kind, subject_id)
        ON DELETE CASCADE
);

-- R9 changelog reverse query: every page asserting one atom, atom-first.
CREATE INDEX archive_page_atom_v1_atom_idx
    ON babylon_meta.archive_page_atom_v1 (atom_id);

-- R3/R12 widens the grant subject domain to glossary concepts. The durable
-- grant rows are never rewritten; only the declarative check widens, inside
-- this additive file, because the original CHECK bytes are hash-pinned.
ALTER TABLE babylon_meta.archive_knowledge_grant_v1
    DROP CONSTRAINT archive_knowledge_grant_v1_subject_kind_check;
ALTER TABLE babylon_meta.archive_knowledge_grant_v1
    ADD CONSTRAINT archive_knowledge_grant_v1_subject_kind_check CHECK (
        subject_kind IN ('county', 'place', 'concept')
    );
ALTER TABLE babylon_meta.archive_knowledge_grant_v1
    DROP CONSTRAINT archive_knowledge_grant_v1_check;
ALTER TABLE babylon_meta.archive_knowledge_grant_v1
    ADD CONSTRAINT archive_knowledge_grant_v1_subject_id_check CHECK (
        (subject_kind = 'county' AND subject_id ~ '^[0-9]{5}$') OR
        (subject_kind = 'place' AND subject_id ~ '^[0-9]{7}$') OR
        (subject_kind = 'concept' AND subject_id ~ '^[a-z0-9][a-z0-9-]{0,127}$')
    );

-- Fog-safe views live in the public schema: PostgreSQL grants schema USAGE on
-- public to PUBLIC, while babylon_meta is revoked from PUBLIC (the pinned
-- babylon_meta schema-grant census row must not drift). Views are derivable,
-- never hashed: a wrong view is a rebuild, never an incident (ADR249 R2).
--
-- Known-only page search, mirroring the writer's ARCHIVE_SEARCH_SQL_V1 grant
-- boundary so the read-only reader role never names the base page/grant
-- tables.
CREATE VIEW public.v_archive_page_known_v1 AS
SELECT
    page.campaign_id,
    page.subject_kind,
    page.subject_id,
    page.title,
    page.verified_tick,
    page.markdown,
    page.content_sha256,
    page.provenance_json
FROM babylon_meta.archive_page_v1 AS page
JOIN babylon_meta.archive_knowledge_grant_v1 AS knowledge
  ON knowledge.campaign_id = page.campaign_id
 AND knowledge.subject_kind = page.subject_kind
 AND knowledge.subject_id = page.subject_id
 AND knowledge.grant_key = 'subject'
 AND knowledge.granted_tick <= page.verified_tick;

-- The known-only atom set (ADR249 R2): an atom is visible exactly while a
-- grant row covers (campaign, subject, grant_key) at granted_tick <= the
-- atom's valid_tick and the valid_tick is within the acknowledged-commit
-- horizon. The horizon reads the durable committed-tick marker relation
-- directly — the same relation public.v_committed_tick_status_v1 publishes
-- to readers — so this writer-side schema carries no install-order dependency
-- on the reader publication view. The horizon is marker-backed, never
-- MAX(tick) over a raw event ledger.
CREATE VIEW public.v_archive_atom_visible AS
SELECT
    atom.campaign_id,
    atom.atom_id,
    atom.subject_kind,
    atom.subject_id,
    atom.signal_key,
    atom.grant_key,
    atom.evidence_class,
    atom.value_kind,
    atom.value_text,
    atom.value_f64,
    atom.value_u64,
    atom.value_bool,
    atom.provenance_source_id,
    atom.provenance_locator,
    atom.valid_tick,
    composition.subject_kind AS page_subject_kind,
    composition.subject_id AS page_subject_id,
    composition.position,
    composition.source_resolve_tick
FROM babylon_meta.archive_atom_v1 AS atom
JOIN babylon_meta.archive_knowledge_grant_v1 AS grant_row
  ON grant_row.campaign_id = atom.campaign_id
 AND grant_row.subject_kind = atom.subject_kind
 AND grant_row.subject_id = atom.subject_id
 AND grant_row.grant_key = atom.grant_key
 AND grant_row.granted_tick <= atom.valid_tick
JOIN (
    SELECT campaign_id, pg_catalog.max(resolve_tick) AS horizon_tick
    FROM babylon_state.tick_commit
    GROUP BY campaign_id
) AS horizon
  ON horizon.campaign_id = atom.campaign_id
 AND atom.valid_tick <= horizon.horizon_tick
JOIN babylon_meta.archive_page_atom_v1 AS composition
  ON composition.campaign_id = atom.campaign_id
 AND composition.atom_id = atom.atom_id;

-- Per-surface composition (ADR249 R9): the county dossier card reads the
-- visible atoms asserted by county pages, position-ordered by the reader.
CREATE VIEW public.v_county_card_atoms AS
SELECT *
FROM public.v_archive_atom_visible
WHERE page_subject_kind = 'county';

-- R9 changelog history (PER-23 Slice 3 decision 1): subject-scoped atom
-- history under the exact grant + acknowledged-horizon fog predicates of
-- v_archive_atom_visible, WITHOUT the composition join — join rows are
-- delete-replaced on supersession, so the live composition cannot answer
-- "what changed across ticks". Views are rebuildable, never hashed
-- (ADR249 R2): a wrong view is a rebuild, never an incident.
CREATE VIEW public.v_archive_subject_atoms AS
SELECT
    atom.campaign_id,
    atom.atom_id,
    atom.subject_kind,
    atom.subject_id,
    atom.signal_key,
    atom.grant_key,
    atom.evidence_class,
    atom.value_kind,
    atom.value_text,
    atom.value_f64,
    atom.value_u64,
    atom.value_bool,
    atom.provenance_source_id,
    atom.provenance_locator,
    atom.valid_tick
FROM babylon_meta.archive_atom_v1 AS atom
JOIN babylon_meta.archive_knowledge_grant_v1 AS grant_row
  ON grant_row.campaign_id = atom.campaign_id
 AND grant_row.subject_kind = atom.subject_kind
 AND grant_row.subject_id = atom.subject_id
 AND grant_row.grant_key = atom.grant_key
 AND grant_row.granted_tick <= atom.valid_tick
JOIN (
    SELECT campaign_id, pg_catalog.max(resolve_tick) AS horizon_tick
    FROM babylon_state.tick_commit
    GROUP BY campaign_id
) AS horizon
  ON horizon.campaign_id = atom.campaign_id
 AND atom.valid_tick <= horizon.horizon_tick;

INSERT INTO babylon_meta.archive_atom_schema_v1 (contract_id)
VALUES ('babylon.archive-atom-schema.v1');

REVOKE ALL ON TABLE babylon_meta.archive_atom_schema_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.archive_atom_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.archive_page_atom_v1 FROM PUBLIC;

-- Reader grants and refusals, guarded on role existence so this file stays
-- installable before the reader role (mirrors reader_role_v1.sql:20-32).
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader') THEN
        GRANT SELECT ON public.v_archive_page_known_v1 TO babylon_reader;
        GRANT SELECT ON public.v_archive_atom_visible TO babylon_reader;
        GRANT SELECT ON public.v_county_card_atoms TO babylon_reader;
        GRANT SELECT ON public.v_archive_subject_atoms TO babylon_reader;
        REVOKE ALL ON TABLE babylon_meta.archive_atom_v1 FROM babylon_reader;
        REVOKE ALL ON TABLE babylon_meta.archive_page_atom_v1 FROM babylon_reader;
    END IF;
END
$$;
