-- Sole live retained-page composition. Historical atoms and content hashes stay V1.
-- The installer holds the schema lock and excludes predecessor Archive writes,
-- copies and validates every retained head, then inserts the marker last.
CREATE TABLE babylon_meta.archive_revision_schema_v2 (
    singleton BOOLEAN PRIMARY KEY CHECK (singleton),
    migration_sha256 BYTEA NOT NULL CHECK (octet_length(migration_sha256) = 32)
);

CREATE TABLE babylon_meta.archive_retention_v2 (
    campaign_id UUID PRIMARY KEY REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE,
    floor_tick BIGINT NOT NULL CHECK (floor_tick >= 0),
    floor_content_hash BYTEA CHECK (octet_length(floor_content_hash) = 32),
    processed_at_adoption BIGINT NOT NULL CHECK (processed_at_adoption >= 0 AND processed_at_adoption <= floor_tick),
    adopted_page_count BIGINT NOT NULL CHECK (adopted_page_count >= 0),
    adopted_heads_sha256 BYTEA NOT NULL CHECK (octet_length(adopted_heads_sha256) = 32),
    adoption_sha256 BYTEA NOT NULL CHECK (octet_length(adoption_sha256) = 32),
    CHECK ((floor_tick = 0) = (floor_content_hash IS NULL))
);

-- A receipt and cutover at the same T share one immutable disclosure snapshot.
CREATE TABLE babylon_meta.archive_tick_knowledge_v2 (
    campaign_id UUID NOT NULL REFERENCES babylon_meta.archive_retention_v2(campaign_id) ON DELETE CASCADE,
    resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    tick_content_hash BYTEA NOT NULL CHECK (octet_length(tick_content_hash) = 32),
    worker_contract_sha256 BYTEA NOT NULL CHECK (octet_length(worker_contract_sha256) = 32),
    knowledge_sha256 BYTEA NOT NULL CHECK (octet_length(knowledge_sha256) = 32),
    grant_count INTEGER NOT NULL CHECK (grant_count BETWEEN 0 AND 65535),
    PRIMARY KEY (campaign_id,resolve_tick),
    FOREIGN KEY (campaign_id,resolve_tick) REFERENCES babylon_state.tick_commit(campaign_id,resolve_tick)
);
CREATE TABLE babylon_meta.archive_tick_knowledge_member_v2 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county','place')),
    subject_id TEXT NOT NULL,
    grant_key TEXT NOT NULL,
    PRIMARY KEY (campaign_id,resolve_tick,subject_kind,subject_id,grant_key),
    FOREIGN KEY (campaign_id,resolve_tick) REFERENCES babylon_meta.archive_tick_knowledge_v2 ON DELETE CASCADE,
    FOREIGN KEY (campaign_id,subject_kind,subject_id,grant_key)
        REFERENCES babylon_meta.archive_knowledge_grant_v1(campaign_id,subject_kind,subject_id,grant_key)
);

CREATE TABLE babylon_meta.archive_page_revision_v2 (
    campaign_id UUID NOT NULL REFERENCES babylon_meta.archive_retention_v2(campaign_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place')),
    subject_id TEXT NOT NULL CHECK (
        (subject_kind = 'county' AND subject_id ~ '^[0-9]{5}$') OR
        (subject_kind = 'place' AND subject_id ~ '^[0-9]{7}$')
    ),
    effective_tick BIGINT NOT NULL CHECK (effective_tick >= 1),
    origin SMALLINT NOT NULL CHECK (origin IN (0, 1)),
    source_tick BIGINT NOT NULL CHECK (source_tick >= 1 AND source_tick <= effective_tick),
    source_content_hash BYTEA NOT NULL CHECK (octet_length(source_content_hash) = 32),
    template_sha256 BYTEA NOT NULL CHECK (octet_length(template_sha256) = 32),
    content_sha256 BYTEA NOT NULL CHECK (octet_length(content_sha256) = 32),
    revision_sha256 BYTEA NOT NULL CHECK (octet_length(revision_sha256) = 32),
    title TEXT NOT NULL CHECK (octet_length(title) BETWEEN 1 AND 4096),
    markdown TEXT NOT NULL CHECK (octet_length(markdown) <= 1048576),
    search_text TEXT NOT NULL CHECK (octet_length(search_text) <= 1048576),
    provenance_json TEXT NOT NULL CHECK (octet_length(provenance_json) <= 1048576),
    atom_count INTEGER NOT NULL CHECK (atom_count BETWEEN 1 AND 513),
    grant_count INTEGER NOT NULL CHECK (grant_count BETWEEN 1 AND 513),
    emission_json TEXT CHECK (octet_length(emission_json) <= 8388608),
    CHECK (origin = 0 OR emission_json IS NOT NULL),
    CHECK (origin = 0 OR source_tick = effective_tick),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, effective_tick, origin),
    FOREIGN KEY (campaign_id, source_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick),
    FOREIGN KEY (campaign_id, effective_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
);

CREATE TABLE babylon_meta.archive_revision_atom_v2 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    effective_tick BIGINT NOT NULL,
    origin SMALLINT NOT NULL,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 512),
    atom_id BYTEA NOT NULL REFERENCES babylon_meta.archive_atom_v1(atom_id),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, effective_tick, origin, position),
    UNIQUE (campaign_id, subject_kind, subject_id, effective_tick, origin, atom_id),
    FOREIGN KEY (campaign_id, subject_kind, subject_id, effective_tick, origin)
        REFERENCES babylon_meta.archive_page_revision_v2 ON DELETE CASCADE
);

CREATE TABLE babylon_meta.archive_revision_grant_v2 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    effective_tick BIGINT NOT NULL,
    origin SMALLINT NOT NULL,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 512),
    grant_subject_kind TEXT NOT NULL CHECK (grant_subject_kind IN ('county', 'place')),
    grant_subject_id TEXT NOT NULL,
    grant_key TEXT NOT NULL CHECK (grant_key ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    granted_tick BIGINT NOT NULL CHECK (granted_tick >= 0),
    provenance_source_id TEXT NOT NULL CHECK (octet_length(provenance_source_id) BETWEEN 1 AND 4096),
    provenance_locator TEXT NOT NULL CHECK (octet_length(provenance_locator) BETWEEN 1 AND 4096),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, effective_tick, origin, position),
    UNIQUE (campaign_id, subject_kind, subject_id, effective_tick, origin, grant_subject_kind, grant_subject_id, grant_key),
    FOREIGN KEY (campaign_id, subject_kind, subject_id, effective_tick, origin)
        REFERENCES babylon_meta.archive_page_revision_v2 ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, grant_subject_kind, grant_subject_id, grant_key)
        REFERENCES babylon_meta.archive_knowledge_grant_v1(campaign_id, subject_kind, subject_id, grant_key)
);

CREATE TABLE babylon_meta.archive_retention_seal_v2 (
    campaign_id UUID PRIMARY KEY REFERENCES babylon_meta.archive_retention_v2(campaign_id) ON DELETE CASCADE,
    floor_tick BIGINT NOT NULL CHECK (floor_tick >= 1),
    floor_content_hash BYTEA NOT NULL CHECK (octet_length(floor_content_hash) = 32),
    adoption_sha256 BYTEA NOT NULL CHECK (octet_length(adoption_sha256) = 32),
    worker_contract_sha256 BYTEA NOT NULL CHECK (octet_length(worker_contract_sha256) = 32),
    knowledge_sha256 BYTEA NOT NULL CHECK (octet_length(knowledge_sha256) = 32),
    composition_sha256 BYTEA NOT NULL CHECK (octet_length(composition_sha256) = 32),
    FOREIGN KEY (campaign_id, floor_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
);

-- Historical claims retain NULL and their original bytes. This new constraint
-- applies to every new row: an obsolete worker cannot consume even a quiet tick.
ALTER TABLE babylon_meta.archive_receipt_consumption_v1 ADD COLUMN revision_generation SMALLINT;
ALTER TABLE babylon_meta.archive_receipt_consumption_v1
    ADD CONSTRAINT archive_receipt_requires_revision_v2
    CHECK (revision_generation IS NOT NULL AND revision_generation = 2) NOT VALID;

-- No alias lets an old writer mutate a new publication. Preserve old tables as
-- inaccessible adoption evidence; the one live reader never queries them.
DROP VIEW public.v_county_card_atoms;
DROP VIEW public.v_archive_atom_visible;
DROP VIEW public.v_archive_page_known_v1;
DROP VIEW public.v_archive_subject_atoms;
ALTER TABLE babylon_meta.archive_page_atom_v1 RENAME TO archive_page_atom_retired_v1;
ALTER TABLE babylon_meta.archive_page_v1 RENAME TO archive_page_retired_v1;

CREATE VIEW public.v_archive_revision_known_v2 WITH (security_barrier = true) AS
SELECT revision.*
FROM babylon_meta.archive_page_revision_v2 revision
JOIN babylon_state.tick_commit source
  ON source.campaign_id = revision.campaign_id AND source.resolve_tick = revision.source_tick
 AND source.tick_content_hash = revision.source_content_hash
WHERE revision.emission_json IS NOT NULL AND EXISTS (
    SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 grant_row
    WHERE grant_row.campaign_id = revision.campaign_id
      AND grant_row.subject_kind = revision.subject_kind AND grant_row.subject_id = revision.subject_id
      AND grant_row.grant_key = 'subject' AND grant_row.granted_tick <= revision.source_tick
)
AND revision.grant_count = (
    SELECT count(*) FROM babylon_meta.archive_revision_grant_v2 dependency
    WHERE (dependency.campaign_id, dependency.subject_kind, dependency.subject_id, dependency.effective_tick, dependency.origin)
        = (revision.campaign_id, revision.subject_kind, revision.subject_id, revision.effective_tick, revision.origin)
)
AND revision.atom_count = (
    SELECT count(*) FROM babylon_meta.archive_revision_atom_v2 membership
    WHERE (membership.campaign_id, membership.subject_kind, membership.subject_id, membership.effective_tick, membership.origin)
        = (revision.campaign_id, revision.subject_kind, revision.subject_id, revision.effective_tick, revision.origin)
)
AND NOT EXISTS (
    SELECT 1 FROM babylon_meta.archive_revision_grant_v2 dependency
    LEFT JOIN babylon_meta.archive_knowledge_grant_v1 grant_row
      ON grant_row.campaign_id = dependency.campaign_id
     AND grant_row.subject_kind = dependency.grant_subject_kind
     AND grant_row.subject_id = dependency.grant_subject_id
     AND grant_row.grant_key = dependency.grant_key
     AND grant_row.granted_tick = dependency.granted_tick
     AND grant_row.provenance_source_id = dependency.provenance_source_id
     AND grant_row.provenance_locator = dependency.provenance_locator
    WHERE (dependency.campaign_id, dependency.subject_kind, dependency.subject_id, dependency.effective_tick, dependency.origin)
        = (revision.campaign_id, revision.subject_kind, revision.subject_id, revision.effective_tick, revision.origin)
      AND (grant_row.campaign_id IS NULL OR grant_row.granted_tick > revision.source_tick)
);

-- Safe candidate identity lets the reader distinguish an unavailable subject
-- from an incomplete/tampered payload which must refuse rather than disappear.
CREATE VIEW public.v_archive_revision_index_v2 WITH (security_barrier = true) AS
SELECT revision.campaign_id, revision.subject_kind, revision.subject_id,
    revision.effective_tick, revision.origin, revision.revision_sha256,
    revision.emission_json IS NOT NULL AS has_emission_witness
FROM babylon_meta.archive_page_revision_v2 revision
JOIN babylon_meta.archive_knowledge_grant_v1 grant_row
  ON grant_row.campaign_id = revision.campaign_id
 AND grant_row.subject_kind = revision.subject_kind AND grant_row.subject_id = revision.subject_id
 AND grant_row.grant_key = 'subject' AND grant_row.granted_tick <= revision.source_tick;

CREATE VIEW public.v_archive_revision_atom_v2 WITH (security_barrier = true) AS
SELECT atom.*, revision.effective_tick, revision.origin, membership.position
FROM public.v_archive_revision_known_v2 revision
JOIN babylon_meta.archive_revision_atom_v2 membership
  USING (campaign_id, subject_kind, subject_id, effective_tick, origin)
JOIN babylon_meta.archive_atom_v1 atom
  ON atom.atom_id = membership.atom_id AND atom.campaign_id = revision.campaign_id
 AND atom.subject_kind = revision.subject_kind AND atom.subject_id = revision.subject_id
 AND atom.valid_tick = revision.source_tick;

CREATE VIEW public.v_archive_revision_grant_v2 WITH (security_barrier = true) AS
SELECT dependency.* FROM public.v_archive_revision_known_v2 revision
JOIN babylon_meta.archive_revision_grant_v2 dependency
  USING (campaign_id, subject_kind, subject_id, effective_tick, origin);

-- The safe view verifies the original V1 knowledge encoding inside the database;
-- it exposes neither private grant labels nor provenance to the confined login.
CREATE VIEW public.v_archive_tick_knowledge_v2 AS
SELECT pin.campaign_id,pin.resolve_tick,pin.tick_content_hash,pin.worker_contract_sha256,pin.knowledge_sha256,
    (pin.tick_content_hash=marker.tick_content_hash
     AND pin.grant_count=members.count AND members.invalid=0
     AND pin.knowledge_sha256=pg_catalog.sha256(
         pg_catalog.convert_to('babylon.semantic-archive-knowledge.v1','UTF8') || pg_catalog.decode('00','hex')
         || pg_catalog.int8send(members.count) || members.bytes)) AS valid,
    EXISTS(SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 grant_row
        WHERE grant_row.campaign_id=pin.campaign_id AND grant_row.subject_kind IN ('county','place')
        AND grant_row.granted_tick<=pin.resolve_tick
        AND NOT EXISTS(SELECT 1 FROM babylon_meta.archive_tick_knowledge_member_v2 member
            WHERE member.campaign_id=pin.campaign_id AND member.resolve_tick=pin.resolve_tick
            AND member.subject_kind=grant_row.subject_kind AND member.subject_id=grant_row.subject_id
            AND member.grant_key=grant_row.grant_key)) AS late_grants
FROM babylon_meta.archive_tick_knowledge_v2 pin
JOIN babylon_state.tick_commit marker USING(campaign_id,resolve_tick)
CROSS JOIN LATERAL (
    SELECT count(*) AS count,
        count(*) FILTER (WHERE grant_row.campaign_id IS NULL OR grant_row.granted_tick>pin.resolve_tick) AS invalid,
        COALESCE(string_agg(
            CASE member.subject_kind WHEN 'county' THEN pg_catalog.decode('01','hex') ELSE pg_catalog.decode('02','hex') END
            || pg_catalog.int8send(octet_length(member.subject_id)::BIGINT) || pg_catalog.convert_to(member.subject_id,'UTF8')
            || pg_catalog.int8send(octet_length(member.grant_key)::BIGINT) || pg_catalog.convert_to(member.grant_key,'UTF8')
            || pg_catalog.int8send(grant_row.granted_tick)
            || pg_catalog.int8send(octet_length(grant_row.provenance_source_id)::BIGINT) || pg_catalog.convert_to(grant_row.provenance_source_id,'UTF8')
            || pg_catalog.int8send(octet_length(grant_row.provenance_locator)::BIGINT) || pg_catalog.convert_to(grant_row.provenance_locator,'UTF8'),
            pg_catalog.decode('','hex') ORDER BY member.subject_kind,member.subject_id,member.grant_key),
            pg_catalog.decode('','hex')) AS bytes
    FROM babylon_meta.archive_tick_knowledge_member_v2 member
    LEFT JOIN babylon_meta.archive_knowledge_grant_v1 grant_row
        USING(campaign_id,subject_kind,subject_id,grant_key)
    WHERE member.campaign_id=pin.campaign_id AND member.resolve_tick=pin.resolve_tick
) members;

CREATE VIEW public.v_archive_retention_v2 AS
SELECT retention.*, (retention.floor_tick=0 OR proof.valid) AS sealed,
    seal.campaign_id IS NOT NULL AS seal_present, proof.valid AS seal_valid,
    seal.worker_contract_sha256 AS seal_worker_contract_sha256
FROM babylon_meta.archive_retention_v2 retention
LEFT JOIN babylon_meta.archive_retention_seal_v2 seal USING(campaign_id)
LEFT JOIN public.v_archive_tick_knowledge_v2 pin
    ON pin.campaign_id=retention.campaign_id AND pin.resolve_tick=retention.floor_tick
LEFT JOIN LATERAL (
    SELECT COALESCE(bool_and(emission_json IS NOT NULL),TRUE) AS all_witnessed,
        pg_catalog.sha256(COALESCE(string_agg(revision_sha256,pg_catalog.decode('','hex')
            ORDER BY subject_kind,subject_id),pg_catalog.decode('','hex'))) AS digest
    FROM (SELECT DISTINCT ON(subject_kind,subject_id) subject_kind,subject_id,revision_sha256,emission_json
        FROM babylon_meta.archive_page_revision_v2 revision
        WHERE revision.campaign_id=retention.campaign_id AND revision.effective_tick<=retention.floor_tick
        ORDER BY subject_kind,subject_id,effective_tick DESC,origin DESC) latest
) composition ON retention.floor_tick>0
CROSS JOIN LATERAL (SELECT (
    seal.floor_tick=retention.floor_tick AND seal.floor_content_hash=retention.floor_content_hash
    AND seal.adoption_sha256=retention.adoption_sha256
    AND pin.valid AND seal.worker_contract_sha256=pin.worker_contract_sha256
    AND seal.knowledge_sha256=pin.knowledge_sha256
    AND composition.all_witnessed AND seal.composition_sha256=composition.digest
) IS TRUE AS valid) proof;

CREATE VIEW public.v_archive_subject_grant_v2 AS
SELECT member.campaign_id,member.resolve_tick,member.subject_kind,member.subject_id,grant_row.granted_tick
FROM babylon_meta.archive_tick_knowledge_member_v2 member
JOIN babylon_meta.archive_knowledge_grant_v1 grant_row USING(campaign_id,subject_kind,subject_id,grant_key)
WHERE member.grant_key='subject' AND grant_row.granted_tick<=member.resolve_tick;

-- Admission proves all emitted fields/labels against requested T's pin. Before
-- that pin exists, only already witnessed bytes whose original dependencies
-- still agree are admitted. The reader reports this incomplete scope Pending;
-- neither fresh grants nor current labels can add to the retained emission.
CREATE VIEW public.v_archive_revision_scope_v2 WITH (security_barrier=true) AS
SELECT revision.campaign_id,revision.subject_kind,revision.subject_id,revision.effective_tick,revision.origin,
    marker.resolve_tick AS observation_tick
FROM public.v_archive_revision_known_v2 revision
JOIN babylon_meta.archive_retention_v2 retention USING(campaign_id)
JOIN babylon_state.tick_commit marker ON marker.campaign_id=revision.campaign_id
    AND marker.resolve_tick>=revision.effective_tick
LEFT JOIN babylon_meta.archive_tick_knowledge_v2 pin
    ON pin.campaign_id=marker.campaign_id AND pin.resolve_tick=marker.resolve_tick
WHERE (pin.campaign_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM babylon_meta.archive_revision_grant_v2 dependency
    WHERE (dependency.campaign_id,dependency.subject_kind,dependency.subject_id,dependency.effective_tick,dependency.origin)
        =(revision.campaign_id,revision.subject_kind,revision.subject_id,revision.effective_tick,revision.origin)
    AND NOT EXISTS(SELECT 1 FROM babylon_meta.archive_tick_knowledge_member_v2 member
        WHERE member.campaign_id=pin.campaign_id AND member.resolve_tick=pin.resolve_tick
        AND member.subject_kind=dependency.grant_subject_kind AND member.subject_id=dependency.grant_subject_id
        AND member.grant_key=dependency.grant_key)))
OR (pin.campaign_id IS NULL AND marker.resolve_tick>=retention.floor_tick);

REVOKE ALL ON babylon_meta.archive_revision_schema_v2, babylon_meta.archive_retention_v2,
    babylon_meta.archive_page_revision_v2, babylon_meta.archive_revision_atom_v2,
    babylon_meta.archive_revision_grant_v2, babylon_meta.archive_retention_seal_v2,
    babylon_meta.archive_page_retired_v1, babylon_meta.archive_page_atom_retired_v1,
    babylon_meta.archive_tick_knowledge_v2, babylon_meta.archive_tick_knowledge_member_v2 FROM PUBLIC;

DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader') THEN
        GRANT SELECT ON public.v_archive_revision_known_v2, public.v_archive_revision_atom_v2,
            public.v_archive_revision_grant_v2, public.v_archive_retention_v2,
            public.v_archive_subject_grant_v2, public.v_archive_revision_index_v2,
            public.v_archive_tick_knowledge_v2, public.v_archive_revision_scope_v2 TO babylon_reader;
        REVOKE ALL ON babylon_meta.archive_revision_schema_v2, babylon_meta.archive_retention_v2,
            babylon_meta.archive_page_revision_v2, babylon_meta.archive_revision_atom_v2,
            babylon_meta.archive_revision_grant_v2, babylon_meta.archive_retention_seal_v2,
            babylon_meta.archive_page_retired_v1, babylon_meta.archive_page_atom_retired_v1,
            babylon_meta.archive_tick_knowledge_v2, babylon_meta.archive_tick_knowledge_member_v2 FROM babylon_reader;
    END IF;
END $$;
