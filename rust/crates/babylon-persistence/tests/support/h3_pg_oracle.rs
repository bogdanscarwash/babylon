//! Test-only h3-pg semantic oracle for the canonical H3 identity relation.

// The native reference test binds the kernel-owned parser and fixture.
use super::h3_cell_vectors::{
    load_fixture, ValidVector, VectorFixture, INVALID_RAW_VECTOR_COUNT, PENTAGON_VECTOR_COUNT,
    VALID_VECTOR_COUNT,
};
use babylon_persistence::{install_current_schema, CurrentSchemaDisposition};
use postgres::error::SqlState;
use postgres::types::ToSql;
use postgres::{Client, Config, NoTls, Transaction};

const H3_PG_VERSION: &str = "4.5.0";
const H3_RESOLUTION_COUNT: usize = 16;
const PENTAGONS_PER_RESOLUTION: usize = 12;

pub(super) fn verify_h3_pg_oracle(owner: &Config, admin: &Config) {
    let first = install_current_schema(owner).expect("H3 oracle current schema installs");
    assert_eq!(first.disposition, CurrentSchemaDisposition::Installed);

    let fixture = load_fixture();
    assert_shared_fixture_transport(&fixture);
    let mut client = admin.connect(NoTls).expect("H3 oracle admin must connect");
    assert_pre_activation_state(&mut client);
    client
        .batch_execute("CREATE EXTENSION h3")
        .expect("test-only H3 oracle extension must activate");
    assert_active_extension_identity(&mut client);

    let mut transaction = client
        .transaction()
        .expect("H3 oracle comparison transaction must begin");
    insert_valid_vectors(&mut transaction, &fixture);
    compare_valid_vectors(&mut transaction, &fixture);
    compare_invalid_raw_vectors(&mut transaction, &fixture);
    compare_exact_pentagon_sets(&mut transaction, &fixture);
    assert_persisted_projection_matches(&mut transaction);
    transaction
        .commit()
        .expect("H3 oracle comparison transaction must commit");
    assert_relational_metadata_rejections(&mut client, &fixture);

    client
        .batch_execute("DROP EXTENSION h3 RESTRICT")
        .expect("production BIGINT relation must not depend on test-only h3-pg");
    assert_post_drop_independence(&mut client);
    drop(client);

    let second = install_current_schema(owner).expect("post-oracle current schema remains valid");
    assert_eq!(second.disposition, CurrentSchemaDisposition::AlreadyCurrent);
    assert_eq!(first.identity, second.identity);
}

fn assert_shared_fixture_transport(fixture: &VectorFixture) {
    for vector in fixture.valid.iter().take(VALID_VECTOR_COUNT) {
        assert_eq!(
            i64::try_from(vector.raw_u64).expect("valid H3 identity must fit positive BIGINT"),
            vector.sql_i64,
            "{} SQL transport",
            vector.label
        );
        assert_eq!(
            vector.raw_u64.to_be_bytes(),
            vector.bytes_be,
            "{} bytes",
            vector.label
        );
    }
    for vector in fixture.invalid_sql.iter().take(1) {
        assert!(!vector.label.is_empty());
        assert!(vector.sql_i64 < 0);
    }
    for vector in fixture.invalid_text.iter().take(6) {
        assert!(!vector.label.is_empty());
        assert!(!vector.text.is_empty());
    }
    for vector in fixture.invalid_ancestor.iter().take(2) {
        assert!(!vector.label.is_empty());
        assert!(!vector.text.is_empty());
        match vector.label.as_str() {
            "too_fine_parent" => assert_eq!(vector.requested_resolution, 11),
            "resolution_above_15" => assert_eq!(vector.requested_resolution, 16),
            other => panic!("unexpected invalid ancestor fixture {other}"),
        }
    }
}

fn assert_pre_activation_state(client: &mut Client) {
    let row = client
        .query_one(
            "SELECT (SELECT pg_catalog.count(*) FROM babylon_meta.current_schema), \
                    (SELECT installed_version FROM pg_catalog.pg_available_extensions \
                     WHERE name = 'h3'), \
                    (SELECT default_version FROM pg_catalog.pg_available_extensions \
                     WHERE name = 'h3')",
            &[],
        )
        .expect("pre-activation H3 state must query");
    assert_eq!(row.get::<_, i64>(0), 1);
    assert_eq!(row.get::<_, Option<String>>(1), None);
    assert_eq!(
        row.get::<_, Option<String>>(2).as_deref(),
        Some(H3_PG_VERSION)
    );
}

fn assert_active_extension_identity(client: &mut Client) {
    let row = client
        .query_one(
            "SELECT extension.extversion::pg_catalog.text, \
                    namespace.nspname::pg_catalog.text \
             FROM pg_catalog.pg_extension AS extension \
             JOIN pg_catalog.pg_namespace AS namespace \
               ON namespace.oid = extension.extnamespace \
             WHERE extension.extname = 'h3'",
            &[],
        )
        .expect("active H3 extension identity must query");
    assert_eq!(row.get::<_, String>(0), H3_PG_VERSION);
    assert_eq!(row.get::<_, String>(1), "public");
}

fn insert_valid_vectors(transaction: &mut Transaction<'_>, fixture: &VectorFixture) {
    let statement = transaction
        .prepare(
            "INSERT INTO babylon_ref.h3_cell ( \
                 cell_id, resolution, immediate_parent, ancestor_r4, ancestor_r5, \
                 ancestor_r6, ancestor_r7 \
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .expect("H3 vector insert must prepare");
    for vector in fixture.valid.iter().take(VALID_VECTOR_COUNT) {
        let resolution = i16::from(vector.resolution);
        let immediate_parent = optional_text_identity(vector.immediate_parent.as_deref());
        let ancestor_r4 = optional_text_identity(vector.ancestor_at(4));
        let ancestor_r5 = optional_text_identity(vector.ancestor_at(5));
        let ancestor_r6 = optional_text_identity(vector.ancestor_at(6));
        let ancestor_r7 = optional_text_identity(vector.ancestor_at(7));
        assert_eq!(
            transaction
                .execute(
                    &statement,
                    &[
                        &vector.sql_i64,
                        &resolution,
                        &immediate_parent,
                        &ancestor_r4,
                        &ancestor_r5,
                        &ancestor_r6,
                        &ancestor_r7,
                    ],
                )
                .expect("valid H3 vector must insert"),
            1,
            "valid H3 vector {} must insert exactly once",
            vector.label
        );
    }
}

fn compare_valid_vectors(transaction: &mut Transaction<'_>, fixture: &VectorFixture) {
    let statement = transaction
        .prepare(
            "SELECT (($1::pg_catalog.int8)::public.h3index)::pg_catalog.text, \
                    public.h3_get_resolution(($1::pg_catalog.int8)::public.h3index), \
                    public.h3_is_valid_cell(($1::pg_catalog.int8)::public.h3index), \
                    public.h3_is_pentagon(($1::pg_catalog.int8)::public.h3index), \
                    CASE WHEN $2::pg_catalog.int4 = 0 THEN NULL \
                         ELSE public.h3_cell_to_parent( \
                             ($1::pg_catalog.int8)::public.h3index \
                         )::pg_catalog.int8 END, \
                    CASE WHEN $2 >= 4 THEN public.h3_cell_to_parent( \
                         ($1::pg_catalog.int8)::public.h3index, 4 \
                    )::pg_catalog.int8 END, \
                    CASE WHEN $2 >= 5 THEN public.h3_cell_to_parent( \
                         ($1::pg_catalog.int8)::public.h3index, 5 \
                    )::pg_catalog.int8 END, \
                    CASE WHEN $2 >= 6 THEN public.h3_cell_to_parent( \
                         ($1::pg_catalog.int8)::public.h3index, 6 \
                    )::pg_catalog.int8 END, \
                    CASE WHEN $2 >= 7 THEN public.h3_cell_to_parent( \
                         ($1::pg_catalog.int8)::public.h3index, 7 \
                    )::pg_catalog.int8 END",
        )
        .expect("H3 semantic comparison must prepare");
    for vector in fixture.valid.iter().take(VALID_VECTOR_COUNT) {
        let resolution = i32::from(vector.resolution);
        let row = transaction
            .query_one(&statement, &[&vector.sql_i64, &resolution])
            .unwrap_or_else(|error| panic!("H3 vector {} must query: {error}", vector.label));
        assert_vector_projection(&row, vector);
    }
}

fn assert_vector_projection(row: &postgres::Row, vector: &ValidVector) {
    assert_eq!(
        row.get::<_, String>(0),
        vector.text,
        "{} text",
        vector.label
    );
    assert_eq!(
        row.get::<_, i32>(1),
        i32::from(vector.resolution),
        "{} resolution",
        vector.label
    );
    assert!(row.get::<_, bool>(2), "{} validity", vector.label);
    assert_eq!(
        row.get::<_, bool>(3),
        vector.is_pentagon(),
        "{} pentagon",
        vector.label
    );
    assert_eq!(
        row.get::<_, Option<i64>>(4),
        optional_text_identity(vector.immediate_parent.as_deref()),
        "{} parent",
        vector.label
    );
    for (column, resolution) in [(5, 4_u8), (6, 5_u8), (7, 6_u8), (8, 7_u8)] {
        assert_eq!(
            row.get::<_, Option<i64>>(column),
            optional_text_identity(vector.ancestor_at(resolution)),
            "{} ancestor r{resolution}",
            vector.label
        );
    }
}

fn compare_invalid_raw_vectors(transaction: &mut Transaction<'_>, fixture: &VectorFixture) {
    let statement = transaction
        .prepare(
            "SELECT public.h3_is_valid_cell( \
                 ($1::pg_catalog.int8)::public.h3index \
             )",
        )
        .expect("invalid H3 comparison must prepare");
    for vector in fixture.invalid_raw.iter().take(INVALID_RAW_VECTOR_COUNT) {
        let raw = i64::try_from(vector.raw_u64).expect("invalid vector must fit positive BIGINT");
        let row = transaction
            .query_one(&statement, &[&raw])
            .unwrap_or_else(|error| {
                panic!("invalid H3 vector {} must query: {error}", vector.label)
            });
        assert!(!row.get::<_, bool>(0), "{} must stay invalid", vector.label);
    }
}

fn compare_exact_pentagon_sets(transaction: &mut Transaction<'_>, fixture: &VectorFixture) {
    assert_eq!(
        fixture
            .valid
            .iter()
            .take(VALID_VECTOR_COUNT)
            .filter(|vector| vector.is_pentagon())
            .count(),
        PENTAGON_VECTOR_COUNT
    );
    let statement = transaction
        .prepare(
            "SELECT pentagon::pg_catalog.int8 \
             FROM public.h3_get_pentagons($1) AS pentagon ORDER BY 1 LIMIT 13",
        )
        .expect("pentagon oracle query must prepare");
    for resolution in 0..H3_RESOLUTION_COUNT {
        let resolution_u8 = u8::try_from(resolution).expect("bounded H3 resolution must fit");
        let resolution_i32 = i32::from(resolution_u8);
        let mut expected = fixture
            .valid
            .iter()
            .take(VALID_VECTOR_COUNT)
            .filter(|vector| vector.is_pentagon() && vector.resolution == resolution_u8)
            .map(|vector| vector.sql_i64)
            .collect::<Vec<_>>();
        expected.sort_unstable();
        assert_eq!(expected.len(), PENTAGONS_PER_RESOLUTION);
        let rows = transaction
            .query(&statement, &[&resolution_i32])
            .expect("pentagon oracle set must query");
        assert_eq!(
            rows.len(),
            PENTAGONS_PER_RESOLUTION,
            "resolution {resolution} oracle must return exactly 12 pentagons"
        );
        let actual = rows
            .iter()
            .take(PENTAGONS_PER_RESOLUTION + 1)
            .map(|row| row.get::<_, i64>(0))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "resolution {resolution} pentagon set");
    }
}

fn assert_persisted_projection_matches(transaction: &mut Transaction<'_>) {
    let row = transaction
        .query_one(
            "SELECT pg_catalog.count(*)::pg_catalog.int8, \
                    pg_catalog.count(*) FILTER (WHERE \
                        resolution <> public.h3_get_resolution(cell_id::public.h3index) \
                        OR immediate_parent IS DISTINCT FROM CASE WHEN resolution = 0 \
                           THEN NULL ELSE public.h3_cell_to_parent( \
                               cell_id::public.h3index \
                           )::pg_catalog.int8 END \
                        OR ancestor_r4 IS DISTINCT FROM CASE WHEN resolution >= 4 \
                           THEN public.h3_cell_to_parent( \
                               cell_id::public.h3index, 4 \
                           )::pg_catalog.int8 END \
                        OR ancestor_r5 IS DISTINCT FROM CASE WHEN resolution >= 5 \
                           THEN public.h3_cell_to_parent( \
                               cell_id::public.h3index, 5 \
                           )::pg_catalog.int8 END \
                        OR ancestor_r6 IS DISTINCT FROM CASE WHEN resolution >= 6 \
                           THEN public.h3_cell_to_parent( \
                               cell_id::public.h3index, 6 \
                           )::pg_catalog.int8 END \
                        OR ancestor_r7 IS DISTINCT FROM CASE WHEN resolution >= 7 \
                           THEN public.h3_cell_to_parent( \
                               cell_id::public.h3index, 7 \
                           )::pg_catalog.int8 END \
                    )::pg_catalog.int8 \
             FROM babylon_ref.h3_cell",
            &[],
        )
        .expect("persisted H3 projection must query");
    assert_eq!(
        row.get::<_, i64>(0),
        i64::try_from(VALID_VECTOR_COUNT).unwrap()
    );
    assert_eq!(row.get::<_, i64>(1), 0);
}

fn assert_post_drop_independence(client: &mut Client) {
    let row = client
        .query_one(
            "SELECT NOT EXISTS (SELECT 1 FROM pg_catalog.pg_extension WHERE extname = 'h3'), \
                    pg_catalog.to_regclass('babylon_ref.h3_cell') IS NOT NULL, \
                    (SELECT pg_catalog.count(*) FROM babylon_ref.h3_cell)",
            &[],
        )
        .expect("post-drop H3 independence must query");
    assert!(row.get::<_, bool>(0));
    assert!(row.get::<_, bool>(1));
    assert_eq!(
        row.get::<_, i64>(2),
        i64::try_from(VALID_VECTOR_COUNT).unwrap()
    );
}

fn assert_relational_metadata_rejections(client: &mut Client, fixture: &VectorFixture) {
    let zero = 0_i64;
    let resolution_zero = 0_i16;
    assert_constraint_rejection(
        client,
        "INSERT INTO babylon_ref.h3_cell (cell_id, resolution) VALUES ($1, $2)",
        &[&zero, &resolution_zero],
        &SqlState::CHECK_VIOLATION,
        "h3_cell_id_positive",
    );

    let ordinary_r0 = identity_by_label(fixture, "ordinary_r0");
    let ordinary_r1 = identity_by_label(fixture, "ordinary_r1");
    let ordinary_r8 = identity_by_label(fixture, "ordinary_r8");
    let pentagon_r4 = fixture
        .valid
        .iter()
        .take(VALID_VECTOR_COUNT)
        .find(|vector| vector.resolution == 4 && vector.is_pentagon())
        .expect("fixture must contain an r4 pentagon")
        .sql_i64;
    assert_constraint_rejection(
        client,
        "UPDATE babylon_ref.h3_cell SET immediate_parent = NULL WHERE cell_id = $1",
        &[&ordinary_r1],
        &SqlState::CHECK_VIOLATION,
        "h3_cell_immediate_parent_matches",
    );
    assert_constraint_rejection(
        client,
        "UPDATE babylon_ref.h3_cell SET ancestor_r4 = $1 WHERE cell_id = $1",
        &[&ordinary_r0],
        &SqlState::CHECK_VIOLATION,
        "h3_cell_ancestor_r4_matches",
    );
    assert_constraint_rejection(
        client,
        "UPDATE babylon_ref.h3_cell SET ancestor_r4 = $1 WHERE cell_id = $2",
        &[&pentagon_r4, &ordinary_r8],
        &SqlState::CHECK_VIOLATION,
        "h3_cell_ancestor_r4_matches",
    );
    assert_constraint_rejection(
        client,
        "DELETE FROM babylon_ref.h3_cell WHERE cell_id = $1",
        &[&ordinary_r0],
        &SqlState::FOREIGN_KEY_VIOLATION,
        "h3_cell_immediate_parent_fkey",
    );
}

fn assert_constraint_rejection(
    client: &mut Client,
    statement: &str,
    parameters: &[&(dyn ToSql + Sync)],
    expected_code: &SqlState,
    expected_constraint: &str,
) {
    let mut transaction = client
        .transaction()
        .expect("H3 negative-case transaction must begin");
    transaction
        .batch_execute("SET CONSTRAINTS ALL IMMEDIATE")
        .expect("H3 negative-case constraints must become immediate");
    let error = transaction
        .execute(statement, parameters)
        .expect_err("invalid H3 relational metadata must be rejected");
    assert_eq!(error.code(), Some(expected_code));
    assert_eq!(
        error
            .as_db_error()
            .and_then(postgres::error::DbError::constraint),
        Some(expected_constraint)
    );
    transaction
        .rollback()
        .expect("H3 negative-case transaction must roll back");
}

fn identity_by_label(fixture: &VectorFixture, label: &str) -> i64 {
    fixture
        .valid
        .iter()
        .take(VALID_VECTOR_COUNT)
        .find(|vector| vector.label == label)
        .unwrap_or_else(|| panic!("fixture must contain {label}"))
        .sql_i64
}

fn optional_text_identity(text: Option<&str>) -> Option<i64> {
    text.map(|value| {
        let raw = u64::from_str_radix(value, 16).expect("fixture hierarchy identity must parse");
        i64::try_from(raw).expect("fixture hierarchy identity must fit positive BIGINT")
    })
}
