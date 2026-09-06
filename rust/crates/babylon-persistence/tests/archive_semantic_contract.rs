use std::str::FromStr;
#[path = "support/archive_reader.rs"]
mod archive_reader;

use babylon_kernel::sha256_of;
use babylon_persistence::{
    ArchiveCitationV1, ArchiveDirtyBatchV1, ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1,
    ArchiveLinkV1, ArchiveMaterializeDispositionV1, ArchiveMaterializeModeV1, ArchivePageInputV1,
    ArchivePageRefV1, ArchiveSignalV1, ArchiveSubjectKindV1, ArchiveSubjectV1, CampaignId,
    FogSafeArchiveRendererV1, SemanticArchiveErrorV1, SemanticArchiveStoreV1,
    ARCHIVE_KNOWLEDGE_SQL_V1, ARCHIVE_PAGE_TEMPLATE_SHA256_V1, SEMANTIC_ARCHIVE_SCHEMA_V1_SQL,
};
use postgres::{Config, NoTls};
use uuid::Uuid;

const LIVE_DSN_ENV: &str = "BABYLON_LEGACY_ADOPTER_TEST_DSN";
const LIVE_CANARY_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY";
const LIVE_CAMPAIGN_ENV: &str = "BABYLON_ARCHIVE_TEST_CAMPAIGN_ID";

fn county() -> ArchiveSubjectV1 {
    ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn page_input() -> ArchivePageInputV1 {
    page_input_at(
        "Which neighboring place should organizers investigate next?",
        42,
        [0x11; 32],
    )
}

fn page_input_with_question(question: &str) -> ArchivePageInputV1 {
    page_input_at(question, 42, [0x11; 32])
}

fn page_input_at(
    question: &str,
    verified_tick: u64,
    tick_content_hash: [u8; 32],
) -> ArchivePageInputV1 {
    ArchivePageInputV1::try_new(
        county(),
        verified_tick,
        tick_content_hash,
        question.to_owned(),
        vec![ArchiveSignalV1::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "728576 jobs".to_owned(),
            ArchiveCitationV1::try_new(
                "qcew-2024".to_owned(),
                "fact_qcew_county_rollup county_fips=26163".to_owned(),
            )
            .expect("citation"),
        )
        .expect("signal")],
        vec![
            ArchiveLinkV1::try_new(
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2622000".to_owned())
                    .expect("Detroit ref"),
                "Detroit".to_owned(),
            )
            .expect("Detroit link"),
            ArchiveLinkV1::try_new(
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2668880".to_owned())
                    .expect("unknown place ref"),
                "Riverview".to_owned(),
            )
            .expect("unknown place link"),
        ],
    )
    .expect("page input")
}

#[test]
fn receipt_retry_identity_includes_the_exact_dirty_batch() {
    let first = ArchiveDirtyBatchV1::try_new(42, [0x11; 32], vec![page_input()])
        .expect("first dirty batch");
    let changed = ArchiveDirtyBatchV1::try_new(
        42,
        [0x11; 32],
        vec![page_input_with_question(
            "Which workplace should organizers investigate next?",
        )],
    )
    .expect("changed dirty batch");

    assert_ne!(first.sha256(), changed.sha256());
    assert!(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains("batch_sha256 BYTEA NOT NULL"));
}

#[test]
fn dirty_batch_has_an_explicit_page_limit() {
    let pages = vec![page_input(); ArchiveDirtyBatchV1::MAX_PAGES + 1];

    assert_eq!(
        ArchiveDirtyBatchV1::try_new(42, [0x11; 32], pages),
        Err(SemanticArchiveErrorV1::CollectionBound)
    );
}

fn knowledge() -> ArchiveKnowledgeV1 {
    knowledge_with_subject_locator("county/26163")
}

fn knowledge_with_subject_locator(subject_locator: &str) -> ArchiveKnowledgeV1 {
    let county_ref = ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
        .expect("county ref");
    ArchiveKnowledgeV1::try_new(vec![
        ArchiveKnowledgeGrantV1::try_new(
            county_ref.clone(),
            "subject".to_owned(),
            42,
            ArchiveCitationV1::try_new("archive-subject".to_owned(), subject_locator.to_owned())
                .expect("subject citation"),
        )
        .expect("subject grant"),
        ArchiveKnowledgeGrantV1::try_new(
            county_ref,
            "employment".to_owned(),
            42,
            ArchiveCitationV1::try_new(
                "knowledge-event".to_owned(),
                "employment@tick-42".to_owned(),
            )
            .expect("field grant citation"),
        )
        .expect("field grant"),
        ArchiveKnowledgeGrantV1::try_new(
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2622000".to_owned())
                .expect("Detroit ref"),
            "subject".to_owned(),
            42,
            ArchiveCitationV1::try_new("archive-subject".to_owned(), "place/2622000".to_owned())
                .expect("linked subject citation"),
        )
        .expect("linked subject grant"),
    ])
    .expect("knowledge grants")
}

#[test]
fn pinned_strict_renderer_is_deterministic_and_preserves_unknown_redlinks() {
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let first = renderer
        .render(&page_input(), &knowledge())
        .expect("known page renders");
    let second = renderer
        .render(&page_input(), &knowledge())
        .expect("same page renders");

    assert_eq!(first.markdown(), second.markdown());
    assert_eq!(first.sha256(), sha256_of(first.markdown().as_bytes()));
    let expected_template_sha256 = [
        0xd7, 0x90, 0x43, 0x79, 0xcf, 0x09, 0xf4, 0x1d, 0xb6, 0xab, 0xea, 0x91, 0x46, 0x5b, 0x5f,
        0xe6, 0xe8, 0x04, 0x86, 0x7c, 0xf8, 0x76, 0xbd, 0x44, 0xa0, 0x9f, 0xe6, 0x3b, 0xa9, 0x75,
        0x51, 0x08,
    ];
    assert_eq!(ARCHIVE_PAGE_TEMPLATE_SHA256_V1, expected_template_sha256);
    assert_eq!(renderer.template_sha256(), expected_template_sha256);
    assert!(first.markdown().contains("verified_tick: 42"));
    assert!(first.markdown().contains(
        "tick_content_hash: 1111111111111111111111111111111111111111111111111111111111111111"
    ));
    assert!(first.markdown().contains("728576 jobs"));
    assert!(first
        .markdown()
        .contains("[Detroit](subject:place/2622000)"));
    assert!(first.markdown().contains("[](subject:place/2668880)"));
    assert!(!first.markdown().contains("Riverview"));
    assert_eq!(first.citations().len(), 2);
    assert_eq!(first.citations()[0].source_id(), "archive-subject");
    assert_eq!(first.citations()[1].source_id(), "qcew-2024");
    assert!(first
        .search_text()
        .contains("Which neighboring place should organizers investigate next?"));
}

#[test]
fn knowledge_snapshot_identity_includes_exact_grant_provenance() {
    assert_ne!(
        knowledge().sha256(),
        knowledge_with_subject_locator("county/26163/revised").sha256()
    );
    assert!(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains("knowledge_sha256 BYTEA NOT NULL"));
}

#[test]
fn subject_and_signal_grants_are_both_required() {
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let no_subject = ArchiveKnowledgeV1::try_new(Vec::new()).expect("empty knowledge");
    assert_eq!(
        renderer.render(&page_input(), &no_subject),
        Err(SemanticArchiveErrorV1::UnknownSubject)
    );

    let subject_only = ArchiveKnowledgeV1::try_new(vec![ArchiveKnowledgeGrantV1::try_new(
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
            .expect("county ref"),
        "subject".to_owned(),
        42,
        ArchiveCitationV1::try_new("archive-subject".to_owned(), "county/26163".to_owned())
            .expect("subject citation"),
    )
    .expect("subject grant")])
    .expect("subject-only grant");
    let page_without_signals = renderer
        .render(&page_input(), &subject_only)
        .expect("known subject renders");
    assert!(!page_without_signals.markdown().contains("728576 jobs"));
    assert!(!page_without_signals.search_text().contains("728576"));
}

#[test]
fn validated_inputs_refuse_ambiguous_or_unbounded_identity() {
    assert_eq!(
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, String::new()),
        Err(SemanticArchiveErrorV1::InvalidIdentity)
    );
    assert_eq!(
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "x".repeat(129)),
        Err(SemanticArchiveErrorV1::InvalidIdentity)
    );
    assert_eq!(
        ArchivePageInputV1::try_new(
            county(),
            0,
            [0; 32],
            "question".to_owned(),
            Vec::new(),
            Vec::new(),
        ),
        Err(SemanticArchiveErrorV1::InvalidVerifiedTick)
    );
    assert_eq!(
        ArchiveKnowledgeGrantV1::try_new(
            county().page_ref().clone(),
            "-hidden".to_owned(),
            42,
            ArchiveCitationV1::try_new("source".to_owned(), "locator".to_owned())
                .expect("valid citation"),
        ),
        Err(SemanticArchiveErrorV1::InvalidText)
    );
}

#[test]
fn serialized_citations_cannot_bypass_validation() {
    assert!(
        serde_json::from_str::<ArchiveCitationV1>(r#"{"source_id":"","locator":"locator"}"#)
            .is_err()
    );
}

#[test]
fn schema_contract_keeps_epistemic_rows_out_of_material_state() {
    for relation in [
        "babylon_meta.archive_knowledge_grant_v1",
        "babylon_meta.archive_receipt_consumption_v1",
        "babylon_meta.archive_page_v1",
    ] {
        assert!(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains(relation));
    }
    assert!(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL
        .contains("REFERENCES babylon_state.archive_dirty_receipt_v1"));
    assert!(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains("REFERENCES babylon_state.tick_commit"));
    assert!(!SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains("CREATE TABLE babylon_state.archive_page"));
    assert!(!SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains("IF NOT EXISTS"));
}

#[test]
fn persistence_queries_enforce_grants_in_sql_and_hide_raw_ledgers() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SemanticArchiveStoreV1>();

    assert!(ARCHIVE_KNOWLEDGE_SQL_V1.contains("babylon_meta.archive_knowledge_grant_v1"));
    assert!(ARCHIVE_KNOWLEDGE_SQL_V1.contains("granted_tick <= $2"));
    assert!(ARCHIVE_KNOWLEDGE_SQL_V1.contains("provenance_source_id"));
    assert!(ARCHIVE_KNOWLEDGE_SQL_V1.contains("provenance_locator"));
    let revision = include_str!("../migrations/archive_revision_v2.sql");
    assert!(revision.contains("grant_row.granted_tick = dependency.granted_tick"));
    assert!(revision.contains("emission_json IS NOT NULL"));
    let read = include_str!("../src/archive_revision/read.rs");
    assert!(!read.contains("babylon_meta."));
    assert!(!read.contains("babylon_state."));
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and one committed tick"]
fn live_store_consumes_searches_and_reconciles_exact_receipt_retries() {
    let (config, campaign_id, tick_content_hash) = live_contract_target();
    let store = SemanticArchiveStoreV1::new(&config);
    grant_live_county_knowledge(&store, campaign_id);
    let batch = ArchiveDirtyBatchV1::try_new(
        1,
        tick_content_hash,
        vec![page_input_at(
            "Which neighboring place should organizers investigate next?",
            1,
            tick_content_hash,
        )],
    )
    .expect("live batch");
    let applied = store
        .materialize_receipt(campaign_id, &batch, ArchiveMaterializeModeV1::Consume)
        .expect("live receipt materializes");
    assert_eq!(
        applied.disposition(),
        ArchiveMaterializeDispositionV1::Applied
    );
    assert_eq!(applied.pages().len(), 1);
    assert!(applied.pages()[0].persisted());

    archive_reader::with_reader(&config, |reader| {
        let scope = archive_reader::scope_at(&config, campaign_id, 1);
        let hits = reader
            .search_as_of(&scope, "728576", 10)
            .expect("scoped known search");
        assert_eq!(hits.hits.len(), 1);
        assert_eq!(hits.hits[0].content_source.tick(), 1);
        let read = reader
            .dossier_as_of(
                &scope,
                &hits.hits[0].subject,
                &babylon_persistence::archive_revision::ArchiveDossierBoundsV2::default(),
            )
            .expect("exact typed dossier");
        let (babylon_persistence::archive_revision::ArchiveDossierStateV2::Ready { page, .. }
        | babylon_persistence::archive_revision::ArchiveDossierStateV2::Pending {
            page: Some(page),
            ..
        }) = read.state
        else {
            panic!("retained witnessed page");
        };
        assert!(page.markdown.contains("728576 jobs"));
        assert!(!page.markdown.contains("Riverview"));
        assert_eq!(page.citations.len(), 2);
    });

    let retry = store
        .materialize_receipt(campaign_id, &batch, ArchiveMaterializeModeV1::Consume)
        .expect("exact retry reconciles");
    assert_eq!(
        retry.disposition(),
        ArchiveMaterializeDispositionV1::AlreadyConsumed
    );
    grant_late_link_knowledge(&store, campaign_id);
    assert_eq!(
        store
            .materialize_receipt(campaign_id, &batch, ArchiveMaterializeModeV1::Consume)
            .expect("same receipt retains its first knowledge pin")
            .disposition(),
        ArchiveMaterializeDispositionV1::AlreadyConsumed
    );
    let changed = ArchiveDirtyBatchV1::try_new(
        1,
        tick_content_hash,
        vec![page_input_at(
            "Which workplace should organizers investigate next?",
            1,
            tick_content_hash,
        )],
    )
    .expect("changed live batch");
    assert_eq!(
        store.materialize_receipt(campaign_id, &changed, ArchiveMaterializeModeV1::Consume),
        Err(SemanticArchiveErrorV1::ReceiptConflict)
    );
}

fn live_contract_target() -> (Config, CampaignId, [u8; 32]) {
    let dsn = std::env::var(LIVE_DSN_ENV).expect("disposable live DSN");
    let expected_canary = std::env::var(LIVE_CANARY_ENV).expect("disposable canary");
    let config = Config::from_str(&dsn).expect("live DSN parses");
    let mut client = config
        .clone()
        .connect(NoTls)
        .expect("live preflight connects");
    let actual_canary: String = client
        .query_one(
            "SELECT pg_catalog.current_setting('babylon.per20_disposable', true)",
            &[],
        )
        .expect("live canary query")
        .try_get(0)
        .expect("live canary decode");
    assert_eq!(actual_canary, expected_canary);

    let campaign_uuid =
        Uuid::parse_str(&std::env::var(LIVE_CAMPAIGN_ENV).expect("live Archive campaign identity"))
            .expect("live Archive campaign UUID");
    let campaign_id = CampaignId::from_uuid(campaign_uuid);
    let receipt: Vec<u8> = client
        .query_one(
            "SELECT tick_content_hash FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = 1",
            &[&campaign_uuid],
        )
        .expect("one committed dirty receipt")
        .try_get(0)
        .expect("dirty receipt digest");
    let tick_content_hash: [u8; 32] = receipt.try_into().expect("exact receipt digest width");
    (config, campaign_id, tick_content_hash)
}

fn grant_live_county_knowledge(store: &SemanticArchiveStoreV1, campaign_id: CampaignId) {
    for grant_key in ["subject", "employment"] {
        store
            .grant_knowledge(
                campaign_id,
                &ArchiveKnowledgeGrantV1::try_new(
                    county().page_ref().clone(),
                    grant_key.to_owned(),
                    1,
                    ArchiveCitationV1::try_new(
                        "live-contract".to_owned(),
                        format!("{grant_key}@tick-1"),
                    )
                    .expect("live grant citation"),
                )
                .expect("live knowledge grant"),
            )
            .expect("knowledge grant persists");
    }
}

fn grant_late_link_knowledge(store: &SemanticArchiveStoreV1, campaign_id: CampaignId) {
    store
        .grant_knowledge(
            campaign_id,
            &ArchiveKnowledgeGrantV1::try_new(
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2674900".to_owned())
                    .expect("newly known place ref"),
                "subject".to_owned(),
                1,
                ArchiveCitationV1::try_new(
                    "live-contract".to_owned(),
                    "subject@tick-1-after-consumption".to_owned(),
                )
                .expect("late subject citation"),
            )
            .expect("late subject grant"),
        )
        .expect("late knowledge grant persists");
}
