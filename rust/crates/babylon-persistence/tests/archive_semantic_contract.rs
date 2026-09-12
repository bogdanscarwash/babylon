use babylon_kernel::content_digest::sha256_of;
use babylon_persistence::{
    ArchiveCitation, ArchiveDirtyBatch, ArchiveKnowledge, ArchiveKnowledgeGrant, ArchiveLink,
    ArchivePageInput, ArchivePageRef, ArchiveSignal, ArchiveSubject, ArchiveSubjectKind,
    FogSafeArchiveRenderer, SemanticArchiveError, SemanticArchiveStore, ARCHIVE_KNOWLEDGE_SQL,
    ARCHIVE_PAGE_TEMPLATE_SHA256, CURRENT_ARCHIVE_SCHEMA_SQL,
};

fn county() -> ArchiveSubject {
    ArchiveSubject::try_new(
        ArchiveSubjectKind::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn page_input() -> ArchivePageInput {
    page_input_at(
        "Which neighboring place should organizers investigate next?",
        42,
        [0x11; 32],
    )
}

fn page_input_with_question(question: &str) -> ArchivePageInput {
    page_input_at(question, 42, [0x11; 32])
}

fn page_input_at(
    question: &str,
    verified_tick: u64,
    tick_content_hash: [u8; 32],
) -> ArchivePageInput {
    ArchivePageInput::try_new(
        county(),
        verified_tick,
        tick_content_hash,
        question.to_owned(),
        vec![ArchiveSignal::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "728576 jobs".to_owned(),
            ArchiveCitation::try_new(
                "qcew-2024".to_owned(),
                "fact_qcew_county_rollup county_fips=26163".to_owned(),
            )
            .expect("citation"),
        )
        .expect("signal")],
        vec![
            ArchiveLink::try_new(
                ArchivePageRef::try_new(ArchiveSubjectKind::Place, "2622000".to_owned())
                    .expect("Detroit ref"),
                "Detroit".to_owned(),
            )
            .expect("Detroit link"),
            ArchiveLink::try_new(
                ArchivePageRef::try_new(ArchiveSubjectKind::Place, "2668880".to_owned())
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
    let first =
        ArchiveDirtyBatch::try_new(42, [0x11; 32], vec![page_input()]).expect("first dirty batch");
    let changed = ArchiveDirtyBatch::try_new(
        42,
        [0x11; 32],
        vec![page_input_with_question(
            "Which workplace should organizers investigate next?",
        )],
    )
    .expect("changed dirty batch");

    assert_ne!(first.sha256(), changed.sha256());
    assert!(CURRENT_ARCHIVE_SCHEMA_SQL.contains("batch_sha256 BYTEA NOT NULL"));
}

#[test]
fn dirty_batch_has_an_explicit_page_limit() {
    let pages = vec![page_input(); ArchiveDirtyBatch::MAX_PAGES + 1];

    assert_eq!(
        ArchiveDirtyBatch::try_new(42, [0x11; 32], pages),
        Err(SemanticArchiveError::CollectionBound)
    );
}

fn knowledge() -> ArchiveKnowledge {
    knowledge_with_subject_locator("county/26163")
}

fn knowledge_with_subject_locator(subject_locator: &str) -> ArchiveKnowledge {
    let county_ref = ArchivePageRef::try_new(ArchiveSubjectKind::County, "26163".to_owned())
        .expect("county ref");
    ArchiveKnowledge::try_new(vec![
        ArchiveKnowledgeGrant::try_new(
            county_ref.clone(),
            "subject".to_owned(),
            42,
            ArchiveCitation::try_new("archive-subject".to_owned(), subject_locator.to_owned())
                .expect("subject citation"),
        )
        .expect("subject grant"),
        ArchiveKnowledgeGrant::try_new(
            county_ref,
            "employment".to_owned(),
            42,
            ArchiveCitation::try_new(
                "knowledge-event".to_owned(),
                "employment@tick-42".to_owned(),
            )
            .expect("field grant citation"),
        )
        .expect("field grant"),
        ArchiveKnowledgeGrant::try_new(
            ArchivePageRef::try_new(ArchiveSubjectKind::Place, "2622000".to_owned())
                .expect("Detroit ref"),
            "subject".to_owned(),
            42,
            ArchiveCitation::try_new("archive-subject".to_owned(), "place/2622000".to_owned())
                .expect("linked subject citation"),
        )
        .expect("linked subject grant"),
    ])
    .expect("knowledge grants")
}

#[test]
fn pinned_strict_renderer_is_deterministic_and_preserves_unknown_redlinks() {
    let renderer = FogSafeArchiveRenderer::new().expect("pinned template compiles");
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
    assert_eq!(ARCHIVE_PAGE_TEMPLATE_SHA256, expected_template_sha256);
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
    assert!(CURRENT_ARCHIVE_SCHEMA_SQL.contains("knowledge_sha256 BYTEA NOT NULL"));
}

#[test]
fn subject_and_signal_grants_are_both_required() {
    let renderer = FogSafeArchiveRenderer::new().expect("pinned template compiles");
    let no_subject = ArchiveKnowledge::try_new(Vec::new()).expect("empty knowledge");
    assert_eq!(
        renderer.render(&page_input(), &no_subject),
        Err(SemanticArchiveError::UnknownSubject)
    );

    let subject_only = ArchiveKnowledge::try_new(vec![ArchiveKnowledgeGrant::try_new(
        ArchivePageRef::try_new(ArchiveSubjectKind::County, "26163".to_owned())
            .expect("county ref"),
        "subject".to_owned(),
        42,
        ArchiveCitation::try_new("archive-subject".to_owned(), "county/26163".to_owned())
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
        ArchivePageRef::try_new(ArchiveSubjectKind::County, String::new()),
        Err(SemanticArchiveError::InvalidIdentity)
    );
    assert_eq!(
        ArchivePageRef::try_new(ArchiveSubjectKind::Place, "x".repeat(129)),
        Err(SemanticArchiveError::InvalidIdentity)
    );
    assert_eq!(
        ArchivePageInput::try_new(
            county(),
            0,
            [0; 32],
            "question".to_owned(),
            Vec::new(),
            Vec::new(),
        ),
        Err(SemanticArchiveError::InvalidVerifiedTick)
    );
    assert_eq!(
        ArchiveKnowledgeGrant::try_new(
            county().page_ref().clone(),
            "-hidden".to_owned(),
            42,
            ArchiveCitation::try_new("source".to_owned(), "locator".to_owned())
                .expect("valid citation"),
        ),
        Err(SemanticArchiveError::InvalidText)
    );
}

#[test]
fn serialized_citations_cannot_bypass_validation() {
    assert!(
        serde_json::from_str::<ArchiveCitation>(r#"{"source_id":"","locator":"locator"}"#).is_err()
    );
}

#[test]
fn schema_contract_keeps_epistemic_rows_out_of_material_state() {
    for relation in [
        "babylon_meta.archive_knowledge_grant_v1",
        "babylon_meta.archive_receipt_consumption_v1",
        "babylon_meta.archive_page_revision_v2",
    ] {
        assert!(CURRENT_ARCHIVE_SCHEMA_SQL.contains(relation));
    }
    assert!(
        CURRENT_ARCHIVE_SCHEMA_SQL.contains("REFERENCES babylon_state.archive_dirty_receipt_v1")
    );
    assert!(CURRENT_ARCHIVE_SCHEMA_SQL.contains("REFERENCES babylon_state.tick_commit"));
    assert!(!CURRENT_ARCHIVE_SCHEMA_SQL.contains("CREATE TABLE babylon_state.archive_page"));
    assert!(!CURRENT_ARCHIVE_SCHEMA_SQL.contains("IF NOT EXISTS"));
}

#[test]
fn persistence_queries_enforce_grants_in_sql_and_hide_raw_ledgers() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SemanticArchiveStore>();

    assert!(ARCHIVE_KNOWLEDGE_SQL.contains("babylon_meta.archive_knowledge_grant_v1"));
    assert!(ARCHIVE_KNOWLEDGE_SQL.contains("granted_tick <= $2"));
    assert!(ARCHIVE_KNOWLEDGE_SQL.contains("provenance_source_id"));
    assert!(ARCHIVE_KNOWLEDGE_SQL.contains("provenance_locator"));
    let revision = include_str!("../migrations/current_archive.sql");
    assert!(revision.contains("grant_row.granted_tick = dependency.granted_tick"));
    assert!(revision.contains("emission_json TEXT NOT NULL"));
    let read = include_str!("../src/archive_revision/read.rs");
    assert!(!read.contains("babylon_meta."));
    assert!(!read.contains("babylon_state."));
}
