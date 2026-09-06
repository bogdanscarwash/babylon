//! Exact committed component access belongs exclusively to the full observer.

use super::*;
use babylon_persistence::SemanticArchiveReaderErrorV1;
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
    // The observer's existing economy schema depends on Archive relations
    // installed by the normal campaign foundation path.
    drop(
        DurableMaterialRuntimeV3::create(
            &target.writer,
            CampaignId::from_uuid(Uuid::from_u128(FOUNDATION_CAMPAIGN)),
            MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
                .create_foundation()
                .unwrap(),
        )
        .unwrap(),
    );
    target
}

fn install(target: &DisposableTarget) {
    install_reader_role_v1(&target.writer).unwrap();
    install_observer_economy_schema_v1(&target.writer).unwrap();
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
#[ignore = "requires the disposable PostgreSQL harness; serial clone ownership"]
fn exact_rows_require_the_matching_v3_commit_marker() {
    let mut target = prepared_target();
    let campaign = CampaignId::from_uuid(Uuid::from_u128(41_001));
    let mut runtime = DurableMaterialRuntimeV3::create(
        &target.writer,
        campaign,
        MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
            .create_foundation()
            .unwrap(),
    )
    .unwrap();
    install(&target);
    advance_material_week(&mut runtime);
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
    // These corruptions exist only within rolled-back fixture transactions.
    // They isolate each part of the SQL marker predicate without changing saves.
    let wrong_campaign = format!(
        "UPDATE babylon_state.tick_commit SET campaign_id='{}'::uuid WHERE campaign_id=$1::uuid",
        Uuid::from_u128(FOUNDATION_CAMPAIGN)
    );
    for mutation in [
        "DELETE FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
        "UPDATE babylon_state.tick_commit SET envelope_layout_version=2 WHERE campaign_id=$1::uuid",
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
    known: &ObserverEconomyReaderV1,
    archive: &SemanticArchiveReaderV1,
    campaign: CampaignId,
) {
    assert_eq!(
        known.snapshot(campaign, 0),
        Err(ObserverEconomyErrorV1::Authority)
    );
    let error = archive.committed_tick_status(campaign).unwrap_err();
    let SemanticArchiveReaderErrorV1::WriterAuthorityRefused(held) = error else {
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
#[ignore = "requires the disposable PostgreSQL harness; serial clone ownership"]
fn full_observer_requires_every_view_and_preview_refuses_all_grant_paths() {
    let mut target = prepared_target();
    install(&target);
    let observer_config = target.login("babylon_observer", "fullcomponents");
    let known_config = target.login("babylon_reader", "knowncomponents");
    let observer =
        ObserverEconomyReaderV1::connect(&observer_config, ObserverVisibilityV1::FullObserver)
            .unwrap();
    let known = ObserverEconomyReaderV1::connect(&known_config, ObserverVisibilityV1::KnownPreview)
        .unwrap();
    let archive = SemanticArchiveReaderV1::new(&known_config).unwrap();
    let absent = CampaignId::from_uuid(Uuid::from_u128(41_002));
    let mut writer = target.writer.connect(NoTls).unwrap();
    assert_eq!(
        known.snapshot(absent, 0),
        Err(ObserverEconomyErrorV1::CampaignAbsent)
    );
    assert_eq!(
        observer.snapshot(absent, 0),
        Err(ObserverEconomyErrorV1::CampaignAbsent)
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
            Err(ObserverEconomyErrorV1::Authority),
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
            Err(ObserverEconomyErrorV1::CampaignAbsent)
        );
        assert_eq!(archive.committed_tick_status(absent).unwrap(), None);
    }
}

#[test]
#[ignore = "requires the disposable PostgreSQL harness; serial clone ownership"]
fn schema_identity_rejects_source_definition_and_partial_install_drift() {
    let target = prepared_target();
    install(&target);
    let mut writer = target.writer.connect(NoTls).unwrap();
    let marker = writer.query_one(
        "SELECT migration_sha256, view_definitions FROM public.observer_tick_components_schema_v1 WHERE singleton", &[],
    ).unwrap();
    let digest: String = marker.get(0);
    let definitions: Vec<String> = marker.get(1);
    assert_eq!(digest.len(), 64);
    assert_eq!(definitions.len(), RELATIONS.len());
    install(&target);
    let repeated = writer.query_one(
        "SELECT migration_sha256, view_definitions FROM public.observer_tick_components_schema_v1 WHERE singleton", &[],
    ).unwrap();
    assert_eq!(repeated.get::<_, String>(0), digest);
    assert_eq!(repeated.get::<_, Vec<String>>(1), definitions);
    writer
        .execute(
            "UPDATE public.observer_tick_components_schema_v1 SET migration_sha256=$1",
            &[&"0".repeat(64)],
        )
        .unwrap();
    assert_eq!(
        install_observer_economy_schema_v1(&target.writer),
        Err(ObserverEconomyErrorV1::SchemaDrift)
    );
    writer
        .execute(
            "UPDATE public.observer_tick_components_schema_v1 SET migration_sha256=$1",
            &[&digest],
        )
        .unwrap();
    let original: String = writer
        .query_one(
            "SELECT pg_catalog.pg_get_viewdef('public.v_observer_graph_node_v1'::regclass, false)",
            &[],
        )
        .unwrap()
        .get(0);
    writer.batch_execute(
        "CREATE OR REPLACE VIEW public.v_observer_graph_node_v1 AS SELECT component.* FROM babylon_state.graph_node_v1 component WHERE false",
    ).unwrap();
    assert_eq!(
        install_observer_economy_schema_v1(&target.writer),
        Err(ObserverEconomyErrorV1::SchemaDrift)
    );
    writer
        .batch_execute(&format!(
            "CREATE OR REPLACE VIEW public.v_observer_graph_node_v1 AS {original}"
        ))
        .unwrap();
    install(&target);
    writer
        .batch_execute("DROP VIEW public.v_observer_graph_node_v1")
        .unwrap();
    assert_eq!(
        install_observer_economy_schema_v1(&target.writer),
        Err(ObserverEconomyErrorV1::SchemaDrift)
    );
    writer.batch_execute(&format!(
        "CREATE VIEW public.v_observer_graph_node_v1 AS {original}; GRANT SELECT ON public.v_observer_graph_node_v1 TO babylon_observer"
    )).unwrap();
    install(&target);
    writer
        .batch_execute("DROP TABLE public.observer_tick_components_schema_v1")
        .unwrap();
    assert_eq!(
        install_observer_economy_schema_v1(&target.writer),
        Err(ObserverEconomyErrorV1::SchemaDrift)
    );
}
