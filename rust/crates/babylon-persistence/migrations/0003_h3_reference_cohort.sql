CREATE TABLE babylon_ref.h3_reference_cohort (
    ref_digest BYTEA NOT NULL,
    format_version SMALLINT NOT NULL,
    artifact_name TEXT NOT NULL,
    artifact_manifest_version TEXT NOT NULL,
    artifact_digest BYTEA NOT NULL,
    source_digest BYTEA NOT NULL,
    source_r5_digest BYTEA NOT NULL,
    source_r7_digest BYTEA NOT NULL,
    closure_digest BYTEA NOT NULL,
    membership_digest BYTEA NOT NULL,
    direct_cell_count BIGINT NOT NULL,
    derived_ancestor_count BIGINT NOT NULL,
    closure_cell_count BIGINT NOT NULL,
    CONSTRAINT h3_reference_cohort_pkey PRIMARY KEY (ref_digest),
    CONSTRAINT h3_reference_cohort_artifact_identity UNIQUE (
        format_version, artifact_digest
    ),
    CONSTRAINT h3_reference_cohort_format_v1 CHECK (format_version = 1),
    CONSTRAINT h3_reference_cohort_artifact_name_length CHECK (
        pg_catalog.octet_length(artifact_name) BETWEEN 1 AND 255
    ),
    CONSTRAINT h3_reference_cohort_artifact_manifest_version_length CHECK (
        pg_catalog.octet_length(artifact_manifest_version) BETWEEN 1 AND 64
    ),
    CONSTRAINT h3_reference_cohort_ref_digest_length CHECK (
        pg_catalog.octet_length(ref_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_artifact_digest_length CHECK (
        pg_catalog.octet_length(artifact_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_source_digest_length CHECK (
        pg_catalog.octet_length(source_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_source_r5_digest_length CHECK (
        pg_catalog.octet_length(source_r5_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_source_r7_digest_length CHECK (
        pg_catalog.octet_length(source_r7_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_closure_digest_length CHECK (
        pg_catalog.octet_length(closure_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_membership_digest_length CHECK (
        pg_catalog.octet_length(membership_digest) = 32
    ),
    CONSTRAINT h3_reference_cohort_direct_count_positive CHECK (
        direct_cell_count BETWEEN 1 AND 65536
    ),
    CONSTRAINT h3_reference_cohort_derived_count_nonnegative CHECK (
        derived_ancestor_count BETWEEN 0 AND 1048576
    ),
    CONSTRAINT h3_reference_cohort_closure_count_matches CHECK (
        closure_cell_count BETWEEN 1 AND 1048576
        AND closure_cell_count = direct_cell_count + derived_ancestor_count
    )
);
CREATE TABLE babylon_ref.h3_reference_membership (
    ref_digest BYTEA NOT NULL,
    cell_id BIGINT NOT NULL,
    origin SMALLINT NOT NULL,
    CONSTRAINT h3_reference_membership_pkey PRIMARY KEY (ref_digest, cell_id),
    CONSTRAINT h3_reference_membership_ref_digest_length CHECK (
        pg_catalog.octet_length(ref_digest) = 32
    ),
    CONSTRAINT h3_reference_membership_cell_positive CHECK (cell_id > 0),
    CONSTRAINT h3_reference_membership_origin_closed CHECK (origin IN (1, 2)),
    CONSTRAINT h3_reference_membership_cohort_fkey FOREIGN KEY (ref_digest)
        REFERENCES babylon_ref.h3_reference_cohort(ref_digest)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_reference_membership_cell_fkey FOREIGN KEY (cell_id)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX h3_reference_membership_cell_id_idx
    ON babylon_ref.h3_reference_membership (cell_id, ref_digest);
REVOKE ALL ON TABLE babylon_ref.h3_reference_cohort FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.h3_reference_membership FROM PUBLIC;
