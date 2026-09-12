//! Exact-scope retrieval boundary. V1 rendering/atom identities remain unchanged;
//! immutable revision composition is the sole live dossier and search path.
use babylon_kernel::content_digest::sha256_of;
use babylon_persistence::archive_revision::{ArchiveDossierBounds, ArchiveReadScope};
use babylon_persistence::{
    identity::CampaignId, ArchiveCitation, ArchiveKnowledge, ArchiveKnowledgeGrant,
    ArchivePageInput, ArchivePageRef, ArchiveSignal, ArchiveSubject, ArchiveSubjectKind,
    FogSafeArchiveRenderer, SemanticArchiveError, SemanticArchiveReader,
    SemanticArchiveReaderError,
};
use uuid::Uuid;
const READ: &str = include_str!("../src/archive_revision/read.rs");
const HISTORY: &str = include_str!("../src/archive_revision/read_history.rs");
const SCHEMA: &str = include_str!("../migrations/current_archive.sql");

fn county() -> ArchiveSubject {
    ArchiveSubject::try_new(
        ArchiveSubjectKind::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn signal_citation() -> ArchiveCitation {
    ArchiveCitation::try_new(
        "qcew-2024".to_owned(),
        "fact_qcew_county_rollup county_fips=26163".to_owned(),
    )
    .expect("signal citation")
}

fn page_input() -> ArchivePageInput {
    ArchivePageInput::try_new(
        county(),
        42,
        [0x11; 32],
        "Which neighboring place should organizers investigate next?".to_owned(),
        vec![ArchiveSignal::try_new(
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

fn knowledge() -> ArchiveKnowledge {
    let county_ref = ArchivePageRef::try_new(ArchiveSubjectKind::County, "26163".to_owned())
        .expect("county ref");
    ArchiveKnowledge::try_new(vec![
        ArchiveKnowledgeGrant::try_new(
            county_ref.clone(),
            "subject".to_owned(),
            42,
            ArchiveCitation::try_new("archive-subject".to_owned(), "county/26163".to_owned())
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
    ])
    .expect("knowledge grants")
}

#[test]
fn retained_rendering_preserves_exact_signal_and_provenance_identity() {
    let page = FogSafeArchiveRenderer::new()
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
    assert!(ArchiveReadScope::committed(campaign, 0, [2; 32]).is_err());
    assert!(ArchiveReadScope::committed(campaign, (i64::MAX as u64) + 1, [2; 32]).is_err());
    assert!(ArchiveDossierBounds::try_new(0, None).is_err());
    assert!(ArchiveDossierBounds::try_new(101, None).is_err());
    let mut config = postgres::Config::new();
    config
        .host("127.0.0.1")
        .port(9)
        .user("unconnected_reader")
        .dbname("unconnected_archive");
    let reader = SemanticArchiveReader::new(&config).expect("local target; no connection yet");
    let scope = ArchiveReadScope::committed(campaign, 1, [2; 32]).expect("scope");
    for limit in [0, 101] {
        assert_eq!(
            reader.search_as_of(&scope, "employment", limit),
            Err(SemanticArchiveReaderError::Archive(
                SemanticArchiveError::CollectionBound
            ))
        );
    }
    for query in ["  ".to_owned(), "x".repeat(4097)] {
        assert_eq!(
            reader.search_as_of(&scope, &query, 100),
            Err(SemanticArchiveReaderError::Archive(
                SemanticArchiveError::InvalidText
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
        "v_archive_verification_v1",
        "v_archive_tick_knowledge_v2",
        "v_archive_revision_scope_v2",
        "v_archive_revision_known_v2",
    ] {
        assert!(READ.contains(view), "exact reader requires {view}");
    }
    assert!(READ.contains("crate::archive_worker_contract_sha256()"));
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
        "emission_json TEXT NOT NULL",
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
    assert!(READ.contains("effective_tick DESC"));
}

#[test]
fn language_neutral_successor_names_exact_scope_and_preserved_identity() {
    let contract = include_str!("../../../../contracts/archive_revision_v2.yaml");
    for rule in [
        "version: 2",
        "dossier_as_of",
        "search_as_of",
        "KnowledgeRefresh",
        "Stage stops later evaluation",
        "maximum: 100",
        "Campaign, committed tick, semantic atom and rendered Markdown identities.",
    ] {
        assert!(
            contract.contains(rule),
            "successor explicitly records {rule}"
        );
    }
    assert!(contract.contains("babylon.archive-page-revision.v2"));
}
