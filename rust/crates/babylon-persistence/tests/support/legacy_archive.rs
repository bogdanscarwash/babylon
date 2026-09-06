//! Exact legacy fixture construction in an already canary-validated scratch database.
//!
//! This is test setup, never an application downgrade: retained V1 bytes and atom
//! memberships come from the actual renderer, then the successor is removed so
//! its real installer can exercise adoption of the predecessor storage shape.

use babylon_persistence::ARCHIVE_ATOM_SCHEMA_V1_SQL;
use postgres::{Config, NoTls};

pub fn restore_legacy_heads(config: &Config) {
    let mut client = config.connect(NoTls).expect("legacy fixture connection");
    let mut tx = client.transaction().expect("legacy fixture transaction");
    tx.batch_execute(
        "INSERT INTO babylon_meta.archive_page_retired_v1 \
        (campaign_id,subject_kind,subject_id,title,verified_tick,source_resolve_tick,source_tick_content_hash, \
         template_sha256,content_sha256,markdown,search_text,provenance_json) \
        SELECT DISTINCT ON(campaign_id,subject_kind,subject_id) campaign_id,subject_kind,subject_id,title, \
         source_tick,source_tick,source_content_hash,template_sha256,content_sha256,markdown,search_text,provenance_json \
        FROM babylon_meta.archive_page_revision_v2 \
        ORDER BY campaign_id,subject_kind,subject_id,effective_tick DESC,origin DESC; \
        INSERT INTO babylon_meta.archive_page_atom_retired_v1 \
        (campaign_id,subject_kind,subject_id,atom_id,position,source_resolve_tick) \
        SELECT membership.campaign_id,membership.subject_kind,membership.subject_id,membership.atom_id,membership.position,page.source_tick \
        FROM (SELECT DISTINCT ON(campaign_id,subject_kind,subject_id) * FROM babylon_meta.archive_page_revision_v2 \
          ORDER BY campaign_id,subject_kind,subject_id,effective_tick DESC,origin DESC) page \
        JOIN babylon_meta.archive_revision_atom_v2 membership USING(campaign_id,subject_kind,subject_id,effective_tick,origin); \
        DROP FUNCTION babylon_meta.archive_wakeup_v1() CASCADE; \
        DROP TABLE babylon_meta.archive_wakeup_schema_v1; \
        DROP VIEW public.v_archive_revision_scope_v2,public.v_archive_revision_atom_v2, \
          public.v_archive_revision_grant_v2,public.v_archive_revision_known_v2, \
          public.v_archive_revision_index_v2,public.v_archive_retention_v2, \
          public.v_archive_subject_grant_v2,public.v_archive_tick_knowledge_v2; \
        DROP TABLE babylon_meta.archive_tick_knowledge_member_v2,babylon_meta.archive_tick_knowledge_v2, \
          babylon_meta.archive_revision_atom_v2,babylon_meta.archive_revision_grant_v2, \
          babylon_meta.archive_retention_seal_v2,babylon_meta.archive_page_revision_v2, \
          babylon_meta.archive_retention_v2,babylon_meta.archive_revision_schema_v2; \
        ALTER TABLE babylon_meta.archive_page_retired_v1 RENAME TO archive_page_v1; \
        ALTER TABLE babylon_meta.archive_page_atom_retired_v1 RENAME TO archive_page_atom_v1; \
        ALTER TABLE babylon_meta.archive_receipt_consumption_v1 DROP COLUMN revision_generation;"
    ).expect("preserve exact heads and construct predecessor storage");
    let views = ARCHIVE_ATOM_SCHEMA_V1_SQL
        .split_once("CREATE VIEW public.v_archive_page_known_v1")
        .expect("pinned original views begin")
        .1
        .split_once("INSERT INTO babylon_meta.archive_atom_schema_v1")
        .expect("pinned original views end")
        .0;
    tx.batch_execute(&format!(
        "CREATE VIEW public.v_archive_page_known_v1{views}"
    ))
    .expect("restore exact predecessor reader views");
    tx.commit().expect("publish explicit legacy test fixture");
}
