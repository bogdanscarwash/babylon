-- Transport hints only. Empty payloads disclose no campaign, subject, or atom.
-- PostgreSQL delivers these hints only when the surrounding transaction commits.
CREATE TABLE babylon_meta.archive_wakeup_schema_v1 (
    singleton BOOLEAN PRIMARY KEY CHECK (singleton),
    migration_sha256 BYTEA NOT NULL CHECK (octet_length(migration_sha256) = 32)
);

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
CREATE TRIGGER archive_wakeup_enrollment_v1 AFTER INSERT ON babylon_meta.archive_retention_v2
FOR EACH STATEMENT EXECUTE FUNCTION babylon_meta.archive_wakeup_v1();

REVOKE ALL ON babylon_meta.archive_wakeup_schema_v1 FROM PUBLIC;
DO $roles$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='babylon_reader') THEN
        REVOKE ALL ON babylon_meta.archive_wakeup_schema_v1 FROM babylon_reader;
    END IF;
END
$roles$;
