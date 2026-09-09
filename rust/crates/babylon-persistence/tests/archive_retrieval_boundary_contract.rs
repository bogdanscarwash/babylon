//! Exact-scope retrieval boundary. V1 rendering/atom identities remain unchanged;
//! immutable revision composition is the sole live dossier and search path.
use babylon_kernel::sha256_of;
use babylon_persistence::archive_revision::{ArchiveDossierBoundsV2, ArchiveReadScopeV2};
use babylon_persistence::{
    ArchiveCitationV1, ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, ArchivePageInputV1,
    ArchivePageRefV1, ArchiveSignalV1, ArchiveSubjectKindV1, ArchiveSubjectV1, CampaignId,
    FogSafeArchiveRendererV1, SemanticArchiveErrorV1, SemanticArchiveReaderErrorV1,
    SemanticArchiveReaderV1,
};
use uuid::Uuid;
const READ: &str = include_str!("../src/archive_revision/read.rs");
const HISTORY: &str = include_str!("../src/archive_revision/read_history.rs");
const SCHEMA: &str = include_str!("../migrations/archive_revision_v2.sql");

fn county() -> ArchiveSubjectV1 {
    ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn signal_citation() -> ArchiveCitationV1 {
    ArchiveCitationV1::try_new(
        "qcew-2024".to_owned(),
        "fact_qcew_county_rollup county_fips=26163".to_owned(),
    )
    .expect("signal citation")
}

fn page_input() -> ArchivePageInputV1 {
    ArchivePageInputV1::try_new(
        county(),
        42,
        [0x11; 32],
        "Which neighboring place should organizers investigate next?".to_owned(),
        vec![ArchiveSignalV1::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "728576 jobs".to_owned(),
            signal_citation(),
        )
        .expect("signal")],
        Vec::new(),
    )
    .expect("page input")
}

fn knowledge() -> ArchiveKnowledgeV1 {
    let county_ref = ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
        .expect("county ref");
    ArchiveKnowledgeV1::try_new(vec![
        ArchiveKnowledgeGrantV1::try_new(
            county_ref.clone(),
            "subject".to_owned(),
            42,
            ArchiveCitationV1::try_new("archive-subject".to_owned(), "county/26163".to_owned())
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
    ])
    .expect("knowledge grants")
}

#[test]
fn retained_rendering_preserves_exact_signal_and_provenance_identity() {
    let page = FogSafeArchiveRendererV1::new()
        .expect("pinned template")
        .render(&page_input(), &knowledge())
        .expect("known page");
    let signal = signal_citation();
    assert!(page.markdown().contains("**Employment:** 728576 jobs"));
    assert!(page.markdown().contains(signal.source_id()));
    assert!(page.markdown().contains(signal.locator()));
    assert_eq!(page.citations()[1], signal);
    assert_eq!(page.sha256(), sha256_of(page.markdown().as_bytes()));
}
#[test]
fn exact_scope_refuses_invalid_tick_and_bounds_before_database_access() {
    let campaign = CampaignId::from_uuid(Uuid::from_bytes([1; 16]));
    assert!(ArchiveReadScopeV2::committed(campaign, 0, [2; 32]).is_err());
    assert!(ArchiveReadScopeV2::committed(campaign, (i64::MAX as u64) + 1, [2; 32]).is_err());
    assert!(ArchiveDossierBoundsV2::try_new(0, None).is_err());
    assert!(ArchiveDossierBoundsV2::try_new(101, None).is_err());
    let mut config = postgres::Config::new();
    config
        .host("127.0.0.1")
        .port(9)
        .user("unconnected_reader")
        .dbname("unconnected_archive");
    let reader = SemanticArchiveReaderV1::new(&config).expect("local target; no connection yet");
    let scope = ArchiveReadScopeV2::committed(campaign, 1, [2; 32]).expect("scope");
    for limit in [0, 101] {
        assert_eq!(
            reader.search_as_of(&scope, "employment", limit),
            Err(SemanticArchiveReaderErrorV1::Archive(
                SemanticArchiveErrorV1::CollectionBound
            ))
        );
    }
    for query in ["  ".to_owned(), "x".repeat(4097)] {
        assert_eq!(
            reader.search_as_of(&scope, &query, 100),
            Err(SemanticArchiveReaderErrorV1::Archive(
                SemanticArchiveErrorV1::InvalidText
            ))
        );
    }
}
#[test]
fn dossier_search_and_history_use_one_confined_repeatable_read_scope() {
    for source in [READ, HISTORY] {
        for forbidden in [
            "babylon_meta.",
            "babylon_state.",
            "archive_page_v1",
            "decode_search_hit",
        ] {
            assert!(
                !source.contains(forbidden),
                "confined read cannot name {forbidden}"
            );
        }
    }
    assert_eq!(
        READ.matches(".isolation_level(IsolationLevel::RepeatableRead)")
            .count(),
        2
    );
    assert_eq!(READ.matches(".read_only(true)").count(), 2);
    for view in [
        "v_committed_tick_status_v1",
        "v_archive_retention_v2",
        "v_archive_tick_knowledge_v2",
        "v_archive_revision_scope_v2",
        "v_archive_revision_known_v2",
    ] {
        assert!(READ.contains(view), "exact reader requires {view}");
    }
    assert!(READ.contains("super::publication::worker_contract()"));
    assert!(
        READ.contains("scope.tick() == durable"),
        "late grants affect only the current tail"
    );
    assert!(HISTORY.contains("ArchiveCursorMismatch"));
    assert!(HISTORY.contains("LIMIT 17"));
}
#[test]
fn retained_bytes_require_complete_emission_and_captured_grants() {
    for field in [
        "emission_json IS NOT NULL",
        "grant_count",
        "atom_count",
        "provenance_source_id",
        "provenance_locator",
        "granted_tick",
        "archive_tick_knowledge_member_v2",
    ] {
        assert!(SCHEMA.contains(field), "retained publication binds {field}");
    }
    assert!(SCHEMA.contains("grant_row.granted_tick = dependency.granted_tick"));
    assert!(SCHEMA.contains("grant_row.provenance_locator = dependency.provenance_locator"));
    assert!(SCHEMA.contains("marker.resolve_tick>=revision.effective_tick"));
    assert!(SCHEMA.contains("member.grant_key=dependency.grant_key"));
    assert!(SCHEMA.contains("security_barrier=true"));
    assert!(
        SCHEMA.contains("revision_generation = 2) NOT VALID"),
        "new obsolete quiet claims refuse; old claims are not rewritten"
    );
}
#[test]
fn no_current_head_entry_point_remains_and_search_is_bounded() {
    for source in [
        include_str!("../src/reader.rs"),
        include_str!("../src/archive.rs"),
    ] {
        for retired in [
            "pub fn search_known(",
            "pub fn county_card_atoms(",
            "pub fn subject_atom_history(",
            "struct ArchiveSearchHitV1",
        ] {
            assert!(!source.contains(retired), "retire {retired}");
        }
    }
    assert!(READ.contains("1..=100"));
    assert!(READ.contains("LIMIT $4"));
    assert!(READ.contains("result.truncated"));
    assert!(READ.contains("effective_tick DESC,origin DESC"));
}

#[test]
fn language_neutral_successor_names_exact_scope_and_preserved_identity() {
    let contract = include_str!("../../../../contracts/archive_revision_v2.yaml");
    for rule in [
        "version: 2",
        "dossier_as_of",
        "search_as_of",
        "HistoryNotRetained",
        "KnowledgeRefresh",
        "Stage stops later evaluation",
        "present corrupt seals refuse",
        "maximum: 100",
        "Original campaign, committed tick, semantic atom and rendered Markdown identities.",
    ] {
        assert!(
            contract.contains(rule),
            "successor explicitly records {rule}"
        );
    }
    for domain in [
        "babylon.archive-page-revision.v2",
        "babylon.archive-retention-adoption.v2",
    ] {
        assert!(contract.contains(domain));
    }
    for component in [
        "seal.knowledge_sha256=pin.knowledge_sha256",
        "seal.composition_sha256=composition.digest",
        "seal.worker_contract_sha256=pin.worker_contract_sha256",
    ] {
        assert!(
            SCHEMA.contains(component),
            "cutover proof binds {component}"
        );
    }
}
