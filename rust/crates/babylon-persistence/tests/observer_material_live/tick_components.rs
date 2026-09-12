//! Exact committed component access belongs exclusively to the full observer.

use super::*;
use babylon_persistence::SemanticArchiveReaderError;
use postgres::GenericClient;

const RELATIONS: [(&str, &str); 24] = [
    ("graph_node_v1", "public.v_observer_graph_node_v1"),
    ("graph_node_f64_v1", "public.v_observer_graph_node_f64_v1"),
    ("graph_edge_v1", "public.v_observer_graph_edge_v1"),
    ("graph_hyperedge_v1", "public.v_observer_graph_hyperedge_v1"),
    (
        "graph_hyperedge_member_v1",
        "public.v_observer_graph_hyperedge_member_v1",
    ),
    ("graph_edge_f64_v1", "public.v_observer_graph_edge_f64_v1"),
    (
        "graph_node_currency_v1",
        "public.v_observer_graph_node_currency_v1",
    ),
    (
        "graph_hyperedge_f64_v1",
        "public.v_observer_graph_hyperedge_f64_v1",
    ),
    ("world_register_v1", "public.v_observer_world_register_v1"),
    ("hex_state_delta_v1", "public.v_observer_hex_state_delta_v1"),
    ("territory_state_v1", "public.v_observer_territory_state_v1"),
    (
        "territory_state_field_v1",
        "public.v_observer_territory_state_field_v1",
    ),
    (
        "organization_state_v1",
        "public.v_observer_organization_state_v1",
    ),
    (
        "organization_state_field_v1",
        "public.v_observer_organization_state_field_v1",
    ),
    (
        "organization_territory_v1",
        "public.v_observer_organization_territory_v1",
    ),
    ("tick_event_v2", "public.v_observer_tick_event_v2"),
    (
        "tick_event_field_v2",
        "public.v_observer_tick_event_field_v2",
    ),
    (
        "tick_choice_receipt_v1",
        "public.v_observer_tick_choice_receipt_v1",
    ),
    (
        "tick_choice_receipt_branch_v1",
        "public.v_observer_tick_choice_receipt_branch_v1",
    ),
    (
        "tick_choice_receipt_carrier_element_v1",
        "public.v_observer_tick_choice_receipt_carrier_element_v1",
    ),
    (
        "checkpoint_manifest",
        "public.v_observer_checkpoint_manifest",
    ),
    (
        "checkpoint_section_v1",
        "public.v_observer_checkpoint_section_v1",
    ),
    (
        "archive_dirty_receipt_v1",
        "public.v_observer_archive_dirty_receipt_v1",
    ),
    (
        "tick_action_batch_v1",
        "public.v_observer_tick_action_batch_v1",
    ),
];

const FOUNDATION_CAMPAIGN: u128 = 41_003;

fn prepared_target() -> DisposableTarget {
    let target = DisposableTarget::create();
    // Keep a second admitted campaign available for the marker-scope mutation.
    drop(
        DurableMaterialRuntime::create(
            &target.writer,
            CampaignId::from_uuid(Uuid::from_u128(FOUNDATION_CAMPAIGN)),
            MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard)
                .create_foundation(&crate::test_support::catalog())
                .unwrap(),
        )
        .unwrap(),
    );
    target
}

fn install(target: &DisposableTarget) {
    install_reader_role(&target.writer).unwrap();
    provision_observer_role(&target.writer).unwrap();
}

fn rows(client: &mut impl GenericClient, relation: &str, campaign: CampaignId) -> Vec<String> {
    client
        .query(
            &format!(
                "SELECT pg_catalog.row_to_json(component)::text FROM {relation} component \
                 WHERE campaign_id=$1::uuid ORDER BY 1"
            ),
            &[campaign.as_uuid()],
        )
        .unwrap()
        .iter()
        .map(|row| row.get(0))
        .collect()
}

#[test]
#[ignore = "requires the disposable PostgreSQL harness; independent clone ownership"]
fn exact_rows_require_the_matching_commit_marker() {
    let mut target = prepared_target();
    let campaign = CampaignId::from_uuid(Uuid::from_u128(41_001));
    let mut runtime = DurableMaterialRuntime::create(
        &target.writer,
        campaign,
        MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard)
            .create_foundation(&crate::test_support::catalog())
            .unwrap(),
    )
    .unwrap();
    install(&target);
    advance_material_period(&mut runtime);
    let observer_config = target.login("babylon_observer", "components");
    let mut observer = observer_config.connect(NoTls).unwrap();
    let mut writer = target.writer.connect(NoTls).unwrap();
    let mut populated = 0;
    for (relation, view) in RELATIONS {
        let raw = format!("babylon_state.{relation}");
        let expected = rows(&mut writer, &raw, campaign);
        populated += usize::from(!expected.is_empty());
        assert_eq!(rows(&mut observer, view, campaign), expected, "{relation}");
        let raw_columns = writer.prepare(&format!("SELECT * FROM {raw}")).unwrap();
        let view_columns = observer.prepare(&format!("SELECT * FROM {view}")).unwrap();
        let describe = |statement: &postgres::Statement| {
            statement
                .columns()
                .iter()
                .map(|column| (column.name().to_owned(), column.type_().clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            describe(&view_columns),
            describe(&raw_columns),
            "{relation}"
        );
    }
    assert!(
        populated >= 12,
        "real replay must exercise multiple row families"
    );
    // Unsupported layouts are refused by the current schema before a view can
    // observe them. The failed write and rollback preserve the exact marker.
    let marker_before = rows(&mut writer, "babylon_state.tick_commit", campaign);
    let mut tx = writer.transaction().unwrap();
    let error = tx
        .execute(
            "UPDATE babylon_state.tick_commit SET envelope_layout_version=2 WHERE campaign_id=$1::uuid",
            &[campaign.as_uuid()],
        )
        .unwrap_err();
    assert_eq!(
        error.code(),
        Some(&postgres::error::SqlState::CHECK_VIOLATION)
    );
    assert_eq!(
        error.as_db_error().unwrap().constraint(),
        Some("tick_commit_envelope_layout_v3")
    );
    tx.rollback().unwrap();
    assert_eq!(
        rows(&mut writer, "babylon_state.tick_commit", campaign),
        marker_before
    );
    // These remaining corruptions exist only in rolled-back fixture transactions.
    // They isolate each reachable SQL marker predicate without changing saves.
    let wrong_campaign = format!(
        "UPDATE babylon_state.tick_commit SET campaign_id='{}'::uuid WHERE campaign_id=$1::uuid",
        Uuid::from_u128(FOUNDATION_CAMPAIGN)
    );
    for mutation in [
        "DELETE FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
        "UPDATE babylon_state.tick_commit SET resolve_tick=resolve_tick+1 WHERE campaign_id=$1::uuid",
        wrong_campaign.as_str(),
    ] {
        let mut tx = writer.transaction().unwrap();
        tx.batch_execute("SET CONSTRAINTS ALL DEFERRED").unwrap();
        assert_eq!(tx.execute(mutation, &[campaign.as_uuid()]).unwrap(), 1);
        for (relation, view) in RELATIONS {
            assert!(
                rows(&mut tx, view, campaign).is_empty(),
                "{relation} exposed rows with a mismatched marker: {mutation}"
            );
        }
        tx.rollback().unwrap();
    }
    assert!(!rows(&mut observer, "public.v_observer_graph_node_v1", campaign).is_empty());
}

fn assert_sql_denied(config: &Config, relation: &str) {
    let error = config
        .connect(NoTls)
        .unwrap()
        .query(&format!("SELECT * FROM {relation} LIMIT 0"), &[])
        .unwrap_err();
    assert_eq!(
        error.code(),
        Some(&postgres::error::SqlState::INSUFFICIENT_PRIVILEGE),
        "{relation}"
    );
}

fn assert_preview_refused(
    known: &ObserverEconomyReader,
    archive: &SemanticArchiveReader,
    campaign: CampaignId,
) {
    assert_eq!(
        known.snapshot(campaign, 0),
        Err(ObserverEconomyError::Authority)
    );
    let error = archive.committed_tick_status(campaign).unwrap_err();
    let SemanticArchiveReaderError::WriterAuthorityRefused(held) = error else {
        panic!("expected exact Archive privilege refusal, got {error:?}");
    };
    for (relation, view) in RELATIONS {
        assert!(
            held.contains(&format!("{view}:SELECT")),
            "Archive census missed {relation}"
        );
    }
}

#[test]
#[ignore = "requires the disposable PostgreSQL harness; independent clone ownership"]
fn full_observer_requires_every_view_and_preview_refuses_all_grant_paths() {
    let mut target = prepared_target();
    install(&target);
    let observer_config = target.login("babylon_observer", "fullcomponents");
    let known_config = target.login("babylon_reader", "knowncomponents");
    let observer =
        ObserverEconomyReader::connect(&observer_config, ObserverVisibility::FullObserver).unwrap();
    let known =
        ObserverEconomyReader::connect(&known_config, ObserverVisibility::KnownPreview).unwrap();
    let archive = SemanticArchiveReader::new(&known_config).unwrap();
    let absent = CampaignId::from_uuid(Uuid::from_u128(41_002));
    let mut writer = target.writer.connect(NoTls).unwrap();
    assert_eq!(
        known.snapshot(absent, 0),
        Err(ObserverEconomyError::CampaignAbsent)
    );
    assert_eq!(
        observer.snapshot(absent, 0),
        Err(ObserverEconomyError::CampaignAbsent)
    );
    assert_eq!(archive.committed_tick_status(absent).unwrap(), None);
    for (relation, view) in RELATIONS {
        let raw = format!("babylon_state.{relation}");
        assert_sql_denied(&observer_config, &raw);
        assert_sql_denied(&known_config, &raw);
        assert_sql_denied(&known_config, view);
        writer
            .batch_execute(&format!("REVOKE SELECT ON {view} FROM babylon_observer"))
            .unwrap();
        assert_eq!(
            observer.snapshot(absent, 0),
            Err(ObserverEconomyError::Authority),
            "{view}"
        );
        writer
            .batch_execute(&format!("GRANT SELECT ON {view} TO babylon_observer"))
            .unwrap();
    }
    // Exercise all relations through each collector: direct, inherited,
    // column ACL and PUBLIC. Held reader instances must re-census each read.
    for (grantee, columns) in [
        (format!("\"{}\"", known_config.get_user().unwrap()), ""),
        ("babylon_reader".to_owned(), ""),
        (
            format!("\"{}\"", known_config.get_user().unwrap()),
            " (campaign_id)",
        ),
        ("PUBLIC".to_owned(), ""),
    ] {
        for (_, view) in RELATIONS {
            writer
                .batch_execute(&format!("GRANT SELECT{columns} ON {view} TO {grantee}"))
                .unwrap();
        }
        assert_preview_refused(&known, &archive, absent);
        for (_, view) in RELATIONS {
            writer
                .batch_execute(&format!("REVOKE SELECT{columns} ON {view} FROM {grantee}"))
                .unwrap();
        }
        assert_eq!(
            known.snapshot(absent, 0),
            Err(ObserverEconomyError::CampaignAbsent)
        );
        assert_eq!(archive.committed_tick_status(absent).unwrap(), None);
    }
}

#[test]
#[ignore = "requires the disposable PostgreSQL harness; independent clone ownership"]
fn observer_provisioning_refuses_changed_or_missing_component_views_without_repair() {
    let target = DisposableTarget::create();
    install(&target);
    let mut writer = target.writer.connect(NoTls).unwrap();
    let current_state = |writer: &mut postgres::Client| {
        let row = writer
            .query_one(
                "SELECT schema_sha256, CASE \
                 WHEN pg_catalog.to_regclass('public.v_observer_graph_node_v1') IS NULL THEN NULL \
                 ELSE pg_catalog.pg_get_viewdef( \
                     pg_catalog.to_regclass('public.v_observer_graph_node_v1'), false) END \
                 FROM babylon_meta.current_schema WHERE singleton",
                &[],
            )
            .unwrap();
        (row.get::<_, Vec<u8>>(0), row.get::<_, Option<String>>(1))
    };
    let admitted = current_state(&mut writer);
    assert_eq!(
        admitted.0,
        babylon_persistence::current_schema_sha256().to_vec()
    );
    assert!(admitted.1.is_some());
    writer.batch_execute(
        "CREATE OR REPLACE VIEW public.v_observer_graph_node_v1 AS SELECT component.* FROM babylon_state.graph_node_v1 component WHERE false",
    ).unwrap();
    let altered = current_state(&mut writer);
    assert_eq!(altered.0, admitted.0);
    assert_ne!(altered.1, admitted.1);
    assert_eq!(
        provision_observer_role(&target.writer),
        Err(ObserverEconomyError::SchemaDrift)
    );
    assert_eq!(current_state(&mut writer), altered);
    writer
        .batch_execute("DROP VIEW public.v_observer_graph_node_v1")
        .unwrap();
    let absent = current_state(&mut writer);
    assert_eq!(absent, (admitted.0, None));
    assert_eq!(
        provision_observer_role(&target.writer),
        Err(ObserverEconomyError::SchemaDrift)
    );
    assert_eq!(current_state(&mut writer), absent);
}
