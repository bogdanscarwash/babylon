-- Current Archive relations, confined views, and commit-only wake hints.
-- Installed atomically with the material schema before its sole identity marker.

CREATE TABLE babylon_meta.archive_knowledge_grant_v1 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place', 'concept')),
    subject_id TEXT NOT NULL CHECK (
        (subject_kind = 'county' AND subject_id ~ '^[0-9]{5}$') OR
        (subject_kind = 'place' AND subject_id ~ '^[0-9]{7}$') OR
        (subject_kind = 'concept' AND subject_id ~ '^[a-z0-9][a-z0-9-]{0,127}$')
    ),
    grant_key TEXT NOT NULL CHECK (grant_key ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    granted_tick BIGINT NOT NULL CHECK (granted_tick >= 0),
    provenance_source_id TEXT NOT NULL CHECK (
        pg_catalog.octet_length(provenance_source_id) BETWEEN 1 AND 4096
    ),
    provenance_locator TEXT NOT NULL CHECK (
        pg_catalog.octet_length(provenance_locator) BETWEEN 1 AND 4096
    ),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, grant_key),
    FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE
);

CREATE TABLE babylon_meta.archive_receipt_consumption_v1 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    tick_content_hash BYTEA NOT NULL CHECK (pg_catalog.octet_length(tick_content_hash) = 32),
    batch_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(batch_sha256) = 32),
    worker_contract_sha256 BYTEA NOT NULL CHECK (
        pg_catalog.octet_length(worker_contract_sha256) = 32
    ),
    knowledge_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(knowledge_sha256) = 32),
    PRIMARY KEY (campaign_id, resolve_tick),
    FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.archive_dirty_receipt_v1(campaign_id, resolve_tick)
        ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        ON DELETE CASCADE
);

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
    -- PostgreSQL considers NaN equal to itself; reject it explicitly.
    value_f64 FLOAT8 CHECK (
        value_f64 IS NULL OR (
            value_f64 NOT IN ('NaN'::float8, 'Infinity'::float8, '-Infinity'::float8)
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

-- Publications at the same tick share one immutable disclosure snapshot.
CREATE TABLE babylon_meta.archive_tick_knowledge_v2 (
    campaign_id UUID NOT NULL REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE,
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
    campaign_id UUID NOT NULL REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place')),
    subject_id TEXT NOT NULL CHECK (
        (subject_kind = 'county' AND subject_id ~ '^[0-9]{5}$') OR
        (subject_kind = 'place' AND subject_id ~ '^[0-9]{7}$')
    ),
    effective_tick BIGINT NOT NULL CHECK (effective_tick >= 1),
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
    emission_json TEXT NOT NULL CHECK (octet_length(emission_json) <= 8388608),
    CHECK (source_tick = effective_tick),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, effective_tick),
    FOREIGN KEY (campaign_id, source_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick),
    FOREIGN KEY (campaign_id, effective_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
);

CREATE TABLE babylon_meta.archive_revision_atom_v2 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    effective_tick BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 512),
    atom_id BYTEA NOT NULL REFERENCES babylon_meta.archive_atom_v1(atom_id),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, effective_tick, position),
    UNIQUE (campaign_id, subject_kind, subject_id, effective_tick, atom_id),
    FOREIGN KEY (campaign_id, subject_kind, subject_id, effective_tick)
        REFERENCES babylon_meta.archive_page_revision_v2 ON DELETE CASCADE
);

CREATE TABLE babylon_meta.archive_revision_grant_v2 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    effective_tick BIGINT NOT NULL,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 512),
    grant_subject_kind TEXT NOT NULL CHECK (grant_subject_kind IN ('county', 'place')),
    grant_subject_id TEXT NOT NULL,
    grant_key TEXT NOT NULL CHECK (grant_key ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    granted_tick BIGINT NOT NULL CHECK (granted_tick >= 0),
    provenance_source_id TEXT NOT NULL CHECK (octet_length(provenance_source_id) BETWEEN 1 AND 4096),
    provenance_locator TEXT NOT NULL CHECK (octet_length(provenance_locator) BETWEEN 1 AND 4096),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, effective_tick, position),
    UNIQUE (campaign_id, subject_kind, subject_id, effective_tick, grant_subject_kind, grant_subject_id, grant_key),
    FOREIGN KEY (campaign_id, subject_kind, subject_id, effective_tick)
        REFERENCES babylon_meta.archive_page_revision_v2 ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, grant_subject_kind, grant_subject_id, grant_key)
        REFERENCES babylon_meta.archive_knowledge_grant_v1(campaign_id, subject_kind, subject_id, grant_key)
);

CREATE VIEW public.v_archive_revision_known_v2 WITH (security_barrier = true) AS
SELECT revision.*
FROM babylon_meta.archive_page_revision_v2 revision
JOIN babylon_state.tick_commit source
  ON source.campaign_id = revision.campaign_id AND source.resolve_tick = revision.source_tick
 AND source.tick_content_hash = revision.source_content_hash
WHERE EXISTS (
    SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 grant_row
    WHERE grant_row.campaign_id = revision.campaign_id
      AND grant_row.subject_kind = revision.subject_kind AND grant_row.subject_id = revision.subject_id
      AND grant_row.grant_key = 'subject' AND grant_row.granted_tick <= revision.source_tick
)
AND revision.grant_count = (
    SELECT count(*) FROM babylon_meta.archive_revision_grant_v2 dependency
    WHERE (dependency.campaign_id, dependency.subject_kind, dependency.subject_id, dependency.effective_tick)
        = (revision.campaign_id, revision.subject_kind, revision.subject_id, revision.effective_tick)
)
AND revision.atom_count = (
    SELECT count(*) FROM babylon_meta.archive_revision_atom_v2 membership
    WHERE (membership.campaign_id, membership.subject_kind, membership.subject_id, membership.effective_tick)
        = (revision.campaign_id, revision.subject_kind, revision.subject_id, revision.effective_tick)
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
    WHERE (dependency.campaign_id, dependency.subject_kind, dependency.subject_id, dependency.effective_tick)
        = (revision.campaign_id, revision.subject_kind, revision.subject_id, revision.effective_tick)
      AND (grant_row.campaign_id IS NULL OR grant_row.granted_tick > revision.source_tick)
);

-- Safe candidate identity lets the reader distinguish an unavailable subject
-- from an incomplete/tampered payload which must refuse rather than disappear.
CREATE VIEW public.v_archive_revision_index_v2 WITH (security_barrier = true) AS
SELECT revision.campaign_id, revision.subject_kind, revision.subject_id,
    revision.effective_tick, revision.revision_sha256
FROM babylon_meta.archive_page_revision_v2 revision
JOIN babylon_meta.archive_knowledge_grant_v1 grant_row
  ON grant_row.campaign_id = revision.campaign_id
 AND grant_row.subject_kind = revision.subject_kind AND grant_row.subject_id = revision.subject_id
 AND grant_row.grant_key = 'subject' AND grant_row.granted_tick <= revision.source_tick;

CREATE VIEW public.v_archive_revision_atom_v2 WITH (security_barrier = true) AS
SELECT atom.*, revision.effective_tick, membership.position
FROM public.v_archive_revision_known_v2 revision
JOIN babylon_meta.archive_revision_atom_v2 membership
  USING (campaign_id, subject_kind, subject_id, effective_tick)
JOIN babylon_meta.archive_atom_v1 atom
  ON atom.atom_id = membership.atom_id AND atom.campaign_id = revision.campaign_id
 AND atom.subject_kind = revision.subject_kind AND atom.subject_id = revision.subject_id
 AND atom.valid_tick = revision.source_tick;

CREATE VIEW public.v_archive_revision_grant_v2 WITH (security_barrier = true) AS
SELECT dependency.* FROM public.v_archive_revision_known_v2 revision
JOIN babylon_meta.archive_revision_grant_v2 dependency
  USING (campaign_id, subject_kind, subject_id, effective_tick);

-- The safe view verifies the canonical knowledge encoding inside the database;
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
SELECT revision.campaign_id,revision.subject_kind,revision.subject_id,revision.effective_tick,
    marker.resolve_tick AS observation_tick
FROM public.v_archive_revision_known_v2 revision
JOIN babylon_state.tick_commit marker ON marker.campaign_id=revision.campaign_id
    AND marker.resolve_tick>=revision.effective_tick
LEFT JOIN babylon_meta.archive_tick_knowledge_v2 pin
    ON pin.campaign_id=marker.campaign_id AND pin.resolve_tick=marker.resolve_tick
WHERE (pin.campaign_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM babylon_meta.archive_revision_grant_v2 dependency
    WHERE (dependency.campaign_id,dependency.subject_kind,dependency.subject_id,dependency.effective_tick)
        =(revision.campaign_id,revision.subject_kind,revision.subject_id,revision.effective_tick)
    AND NOT EXISTS(SELECT 1 FROM babylon_meta.archive_tick_knowledge_member_v2 member
        WHERE member.campaign_id=pin.campaign_id AND member.resolve_tick=pin.resolve_tick
        AND member.subject_kind=dependency.grant_subject_kind AND member.subject_id=dependency.grant_subject_id
        AND member.grant_key=dependency.grant_key)))
OR pin.campaign_id IS NULL;

CREATE VIEW public.v_archive_verification_v1 AS
SELECT campaign.campaign_id,
       COALESCE(MAX(marker.resolve_tick), 0) AS durable_tick,
       COALESCE(
           MIN(marker.resolve_tick) FILTER (WHERE consumed.campaign_id IS NULL) - 1,
           MAX(marker.resolve_tick), 0
       ) AS processed_tick
FROM babylon_meta.campaign AS campaign
LEFT JOIN babylon_state.tick_commit AS marker USING (campaign_id)
LEFT JOIN babylon_meta.archive_receipt_consumption_v1 AS consumed
  ON consumed.campaign_id = marker.campaign_id
 AND consumed.resolve_tick = marker.resolve_tick
 AND consumed.tick_content_hash = marker.tick_content_hash
GROUP BY campaign.campaign_id;

CREATE FUNCTION babylon_meta.archive_wakeup_v1() RETURNS trigger
LANGUAGE plpgsql SET search_path = pg_catalog AS $body$
BEGIN
    PERFORM pg_catalog.pg_notify('babylon_archive_wakeup_v1', '');
    RETURN NULL;
END
$body$;

-- Only the installer/owner can create triggers with this function. PostgreSQL
-- checks EXECUTE at trigger creation; this is not a per-DML authorization gate.
REVOKE ALL ON FUNCTION babylon_meta.archive_wakeup_v1() FROM PUBLIC;

CREATE TRIGGER archive_wakeup_tick_v1 AFTER INSERT ON babylon_state.tick_commit
FOR EACH STATEMENT EXECUTE FUNCTION babylon_meta.archive_wakeup_v1();
CREATE TRIGGER archive_wakeup_enrollment_v1 AFTER INSERT ON babylon_meta.campaign
FOR EACH STATEMENT EXECUTE FUNCTION babylon_meta.archive_wakeup_v1();

REVOKE ALL ON babylon_meta.archive_knowledge_grant_v1,
    babylon_meta.archive_receipt_consumption_v1, babylon_meta.archive_atom_v1,
    babylon_meta.archive_page_revision_v2,
    babylon_meta.archive_revision_atom_v2, babylon_meta.archive_revision_grant_v2,
    babylon_meta.archive_tick_knowledge_v2,
    babylon_meta.archive_tick_knowledge_member_v2 FROM PUBLIC;
