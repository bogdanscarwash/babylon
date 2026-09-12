//! Pure contracts for the PER-22 place dossier producer.
//!
//! These tests pin the producer's decision semantics without any database:
//! GEOID validation, sorted enumeration, cross-county slice retention, paged
//! dirty selection that never truncates the drain bound, grant-visibility
//! dirty detection, receipt-stamp-insensitive hashing, hash completeness
//! over signals and county names, and the contract-pinned artifact digests.

use std::collections::BTreeMap;
use std::str::FromStr;

use babylon_persistence::{
    desired_place_projection, parse_stored_place_page, place_page_input,
    place_page_semantic_sha256, select_dirty_place_pages, ArchiveCitation, ArchiveDirtyBatch,
    ArchiveKnowledge, ArchiveKnowledgeGrant, ArchivePageInput, ArchivePageRef, ArchiveSubjectKind,
    FogSafeArchiveRenderer, PlaceCountySlice, PlaceDossierProducer, PlaceGrantIndex, PlacePagePlan,
    SemanticArchiveError, ARCHIVE_PLACE_GRANTS_SQL, ARCHIVE_PLACE_PAGE_READ_SQL,
    PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256, PINNED_PLACE_IDENTITY_ARTIFACT_SHA256,
    PLACE_DECISION_QUESTION, PLACE_IDENTITY_GRANT_KEY,
};
use postgres::Config;

const CONTRACT_PLACE_IDENTITY_SHA256: &str =
    "cb864b4f6f43902bb821e84fe9a4055a9039e0a74d8b8399f209ae6ed26a8be7";
const CONTRACT_COUNTY_PLACE_OVERLAP_SHA256: &str =
    "fcb7baaf63a5422accce8709997de8e409936f7131fa0ef6b0a28762fdfee42f";

fn contract_digest(hex: &str) -> [u8; 32] {
    let bytes = hex.as_bytes();
    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        let high = (bytes[index * 2] as char).to_digit(16).expect("hex digit");
        let low = (bytes[index * 2 + 1] as char)
            .to_digit(16)
            .expect("hex digit");
        *byte = u8::try_from((high << 4) | low).expect("two hex nibbles fit one byte");
    }
    digest
}

fn hex_digest(digest: [u8; 32]) -> String {
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(hex, "{byte:02x}").expect("hex digest writes");
    }
    hex
}

fn producer() -> PlaceDossierProducer {
    let config = Config::from_str("postgresql://unused:unused@127.0.0.1:1/unused")
        .expect("idle producer config parses");
    PlaceDossierProducer::try_new(&config).expect("pinned reference products load")
}

fn desired_pages() -> Vec<PlacePagePlan> {
    producer()
        .desired_pages()
        .expect("desired place pages build")
}

fn plan(geoid: &str, title: &str, counties: &[(&str, &str)]) -> PlacePagePlan {
    PlacePagePlan::try_new(
        geoid.to_owned(),
        title.to_owned(),
        counties
            .iter()
            .map(|(geoid, name)| {
                PlaceCountySlice::try_new((*geoid).to_owned(), (*name).to_owned())
                    .expect("county slice")
            })
            .collect(),
    )
    .expect("place page plan")
}

fn detroit_plan() -> PlacePagePlan {
    plan("2622000", "Detroit city", &[("26163", "Wayne County")])
}

fn grant_index(entries: &[(ArchiveSubjectKind, &str, &str)]) -> PlaceGrantIndex {
    PlaceGrantIndex::try_from_rows(
        entries
            .iter()
            .map(|(kind, id, key)| (*kind, (*id).to_owned(), (*key).to_owned())),
    )
    .expect("grant index builds")
}

/// Every grant a fully revealed page needs: place subject, identity field,
/// and one county subject per overlapping slice.
fn full_grants(plan: &PlacePagePlan) -> PlaceGrantIndex {
    let mut entries = vec![
        (
            ArchiveSubjectKind::Place,
            plan.place_geoid(),
            PLACE_IDENTITY_GRANT_KEY,
        ),
        (ArchiveSubjectKind::Place, plan.place_geoid(), "subject"),
    ];
    for slice in plan.county_links() {
        entries.push((ArchiveSubjectKind::County, slice.county_geoid(), "subject"));
    }
    grant_index(&entries)
}

fn full_grant_rows(plans: &[PlacePagePlan]) -> Vec<(ArchiveSubjectKind, String, String)> {
    plans
        .iter()
        .flat_map(|plan| {
            let mut rows = vec![
                (
                    ArchiveSubjectKind::Place,
                    plan.place_geoid().to_owned(),
                    "subject".to_owned(),
                ),
                (
                    ArchiveSubjectKind::Place,
                    plan.place_geoid().to_owned(),
                    PLACE_IDENTITY_GRANT_KEY.to_owned(),
                ),
            ];
            for slice in plan.county_links() {
                rows.push((
                    ArchiveSubjectKind::County,
                    slice.county_geoid().to_owned(),
                    "subject".to_owned(),
                ));
            }
            rows
        })
        .collect()
}

#[test]
fn geoid_validation_accepts_only_seven_digit_place_identities() {
    assert_eq!(
        PlacePagePlan::try_new("26163".to_owned(), "Too short".to_owned(), Vec::new()),
        Err(SemanticArchiveError::InvalidIdentity)
    );
    assert_eq!(
        PlacePagePlan::try_new("268400A".to_owned(), "Not digits".to_owned(), Vec::new()),
        Err(SemanticArchiveError::InvalidIdentity)
    );
    let valid = PlacePagePlan::try_new("2622000".to_owned(), "Detroit city".to_owned(), Vec::new())
        .expect("valid plan");
    assert_eq!(valid.place_geoid(), "2622000");

    let pages = desired_pages();
    assert_eq!(pages.len(), 745, "exactly the 745 pinned places enumerate");
    for page in &pages {
        assert_eq!(page.place_geoid().len(), 7);
        assert!(page.place_geoid().bytes().all(|byte| byte.is_ascii_digit()));
        assert!(page.place_geoid().starts_with("26"));
        assert!(!page.title().is_empty());
    }
}

#[test]
fn places_enumerate_in_sorted_geoid_order() {
    let pages = desired_pages();
    let mut sorted = pages
        .iter()
        .map(PlacePagePlan::place_geoid)
        .collect::<Vec<_>>();
    sorted.sort_unstable();
    let enumerated = pages
        .iter()
        .map(PlacePagePlan::place_geoid)
        .collect::<Vec<_>>();
    assert_eq!(enumerated, sorted);
    assert_eq!(enumerated.first().copied(), Some("2600380"));
    assert_eq!(enumerated.last().copied(), Some("2689320"));
}

#[test]
fn cross_county_places_keep_every_county_slice() {
    let pages = desired_pages();
    let multi = pages
        .iter()
        .filter(|page| page.county_links().len() > 1)
        .collect::<Vec<_>>();
    assert_eq!(
        multi.len(),
        30,
        "the contract pins exactly 30 cross-county places"
    );

    let three = pages
        .iter()
        .find(|page| page.county_links().len() == 3)
        .expect("at least one three-county place exists");
    let geoids = three
        .county_links()
        .iter()
        .map(PlaceCountySlice::county_geoid)
        .collect::<Vec<_>>();
    let mut sorted = geoids.clone();
    sorted.sort_unstable();
    assert_eq!(geoids, sorted, "county slices stay sorted by county GEOID");

    let input = place_page_input(three, 42, [0x11; 32]).expect("three-county page input");
    let links = input.links();
    assert_eq!(links.len(), 3, "every overlapping county becomes one link");
    for link in links {
        assert_eq!(link.target().kind(), ArchiveSubjectKind::County);
        assert_eq!(link.target().id().len(), 5);
        assert!(!link.known_label().is_empty());
    }
}

#[test]
fn place_allowlist_requires_sorted_unique_seven_digit_geoids() {
    let config = Config::from_str("postgresql://unused:unused@127.0.0.1:1/unused")
        .expect("idle producer config parses");
    let allowlist = vec!["2622000".to_owned(), "2684000".to_owned()];
    let producer = PlaceDossierProducer::with_place_allowlist(&config, &allowlist)
        .expect("sorted unique allowlist binds");
    let pages = producer.desired_pages().expect("allowlisted pages build");
    assert_eq!(pages.len(), 2, "only allowlisted places enumerate");
    assert_eq!(pages[0].place_geoid(), "2622000");
    assert_eq!(pages[1].place_geoid(), "2684000");

    for bad in [
        vec!["2684000".to_owned(), "2622000".to_owned()],
        vec!["2622000".to_owned(), "2622000".to_owned()],
        vec!["26163".to_owned()],
        vec!["262200A".to_owned()],
    ] {
        match PlaceDossierProducer::with_place_allowlist(&config, &bad) {
            Err(SemanticArchiveError::InvalidIdentity) => {}
            _ => panic!("unsorted, duplicate, or malformed GEOIDs must refuse: {bad:?}"),
        }
    }
}

#[test]
fn dirty_drain_beyond_one_receipt_pages_in_geoid_order_without_truncating() {
    let pages = desired_pages();
    let grants =
        PlaceGrantIndex::try_from_rows(full_grant_rows(&pages)).expect("full grant index builds");
    let selection = select_dirty_place_pages(
        &pages,
        &BTreeMap::new(),
        &grants,
        ArchiveDirtyBatch::MAX_PAGES,
    )
    .expect("a 745-page bootstrap backlog pages instead of refusing");

    // The head batch is the first `limit` dirty plans in geoid order; the
    // exact undrained tail count rides along so the receipt stays pending.
    assert_eq!(selection.head().len(), ArchiveDirtyBatch::MAX_PAGES);
    assert_eq!(
        selection.remaining(),
        pages.len() - ArchiveDirtyBatch::MAX_PAGES
    );
    let head_geoids: Vec<&str> = selection
        .head()
        .iter()
        .map(|plan| plan.place_geoid())
        .collect();
    let mut sorted = head_geoids.clone();
    sorted.sort_unstable();
    assert_eq!(head_geoids, sorted, "the head batch stays in geoid order");
    assert_eq!(
        head_geoids,
        pages
            .iter()
            .take(ArchiveDirtyBatch::MAX_PAGES)
            .map(PlacePagePlan::place_geoid)
            .collect::<Vec<_>>(),
        "the head batch is exactly the leading geoid prefix of the dirty set"
    );

    // A second pass over the tail plans takes the next prefix exactly, as
    // the following sweep sees once the head pages are stored-current.
    let tail: Vec<PlacePagePlan> = pages
        .iter()
        .skip(ArchiveDirtyBatch::MAX_PAGES)
        .cloned()
        .collect();
    let tail_grants =
        PlaceGrantIndex::try_from_rows(full_grant_rows(&tail)).expect("tail grant index builds");
    let next = select_dirty_place_pages(
        &tail,
        &BTreeMap::new(),
        &tail_grants,
        ArchiveDirtyBatch::MAX_PAGES,
    )
    .expect("the tail plans page in the next sweep");
    assert_eq!(next.head().len(), ArchiveDirtyBatch::MAX_PAGES);
    assert_eq!(
        next.remaining(),
        tail.len() - ArchiveDirtyBatch::MAX_PAGES,
        "the undrained tail keeps shrinking by one batch per pass"
    );

    // The last pass, once the tail fits the batch bound, drains whole.
    let last_plans: Vec<PlacePagePlan> = tail
        .iter()
        .skip(ArchiveDirtyBatch::MAX_PAGES)
        .cloned()
        .collect();
    let last_grants = PlaceGrantIndex::try_from_rows(full_grant_rows(&last_plans))
        .expect("last grant index builds");
    let last = select_dirty_place_pages(
        &last_plans,
        &BTreeMap::new(),
        &last_grants,
        ArchiveDirtyBatch::MAX_PAGES,
    )
    .expect("the last tail drains whole");
    assert_eq!(last.head().len(), last_plans.len());
    assert_eq!(last.remaining(), 0, "the tail drains whole once it fits");

    // An at-most-limit dirty set still drains whole.
    let subset: Vec<PlacePagePlan> = pages
        .iter()
        .take(ArchiveDirtyBatch::MAX_PAGES)
        .cloned()
        .collect();
    let drained = select_dirty_place_pages(
        &subset,
        &BTreeMap::new(),
        &grants,
        ArchiveDirtyBatch::MAX_PAGES,
    )
    .expect("an at-limit dirty set drains");
    assert_eq!(drained.head().len(), ArchiveDirtyBatch::MAX_PAGES);
    assert_eq!(drained.remaining(), 0);
}

#[test]
fn dirty_diff_excludes_receipt_stamped_fields() {
    let plan = detroit_plan();
    let grants = full_grants(&plan);
    let projection = desired_place_projection(&plan, &grants).expect("desired projection");
    let first_hash = place_page_semantic_sha256(plan.place_geoid(), &projection);
    let second_hash = place_page_semantic_sha256(plan.place_geoid(), &projection);
    assert_eq!(
        first_hash, second_hash,
        "the projection hash is deterministic"
    );

    // Receipt-stamped inputs render and parse back to the same projection.
    let renderer = FogSafeArchiveRenderer::new().expect("pinned template compiles");
    let knowledge = knowledge_for(&[&plan], true);
    let stamped_first = renderer
        .render(
            &place_page_input(&plan, 1, [0x11; 32]).expect("first page input"),
            &knowledge,
        )
        .expect("first receipt page renders");
    let stamped_second = renderer
        .render(
            &place_page_input(&plan, 2, [0x22; 32]).expect("second page input"),
            &knowledge,
        )
        .expect("second receipt page renders");
    let stored_first = parse_stored_place_page("2622000", "Detroit city", stamped_first.markdown())
        .expect("first page parses back");
    let stored_second =
        parse_stored_place_page("2622000", "Detroit city", stamped_second.markdown())
            .expect("second page parses back");
    assert_eq!(
        place_page_semantic_sha256("2622000", &stored_first),
        first_hash,
        "the stored projection strips the first receipt stamp"
    );
    assert_eq!(
        place_page_semantic_sha256("2622000", &stored_second),
        second_hash,
        "a later receipt stamp alone never re-publishes an unchanged page"
    );

    let mut stored_map = BTreeMap::new();
    stored_map.insert("2622000".to_owned(), stored_first);
    let pages = [detroit_plan()];
    let clean = select_dirty_place_pages(&pages, &stored_map, &grants, usize::MAX)
        .expect("clean selection");
    assert!(
        clean.head().is_empty(),
        "unchanged semantic content is not re-published"
    );
}

#[test]
fn semantic_hash_covers_signal_citation_county_names_and_template_identity() {
    let plan = detroit_plan();
    let granted = desired_place_projection(&plan, &full_grants(&plan))
        .expect("fully granted desired projection");
    assert_eq!(granted.signals().len(), 1);
    assert_eq!(granted.signals()[0].label(), "Census identity");
    assert_eq!(granted.signals()[0].value(), "Detroit city");
    assert_eq!(
        granted.signals()[0].source_id(),
        "census-place-authority-v1"
    );
    assert!(granted.signals()[0]
        .locator()
        .ends_with("#place_geoid=2622000"));
    assert_eq!(
        granted.counties(),
        &[("26163".to_owned(), Some("Wayne County".to_owned()))]
    );

    let baseline = place_page_semantic_sha256(plan.place_geoid(), &granted);

    // Losing the identity grant drops the signal and changes the hash.
    let subject_and_county = grant_index(&[
        (ArchiveSubjectKind::Place, "2622000", "subject"),
        (ArchiveSubjectKind::County, "26163", "subject"),
    ]);
    let redacted = desired_place_projection(&plan, &subject_and_county)
        .expect("signal-redacted desired projection");
    assert!(redacted.signals().is_empty());
    assert_ne!(
        place_page_semantic_sha256(plan.place_geoid(), &redacted),
        baseline,
        "losing the identity grant changes the semantic hash"
    );

    // Losing the county subject grant drops the link name and changes the hash.
    let no_county = grant_index(&[
        (ArchiveSubjectKind::Place, "2622000", "subject"),
        (
            ArchiveSubjectKind::Place,
            "2622000",
            PLACE_IDENTITY_GRANT_KEY,
        ),
    ]);
    let redlinked =
        desired_place_projection(&plan, &no_county).expect("county-redlink desired projection");
    assert_eq!(redlinked.counties(), &[("26163".to_owned(), None)]);
    assert_ne!(
        place_page_semantic_sha256(plan.place_geoid(), &redlinked),
        baseline,
        "losing the county subject grant changes the semantic hash"
    );
    assert_ne!(
        place_page_semantic_sha256(plan.place_geoid(), &redacted),
        place_page_semantic_sha256(plan.place_geoid(), &redlinked),
        "signal redaction and county redlink redaction hash differently"
    );
}

#[test]
fn semantic_hash_matches_the_pinned_place_page_vectors() {
    let plan = detroit_plan();
    let granted = desired_place_projection(&plan, &full_grants(&plan))
        .expect("fully granted desired projection");
    assert_eq!(
        hex_digest(place_page_semantic_sha256(plan.place_geoid(), &granted)),
        "e1956add538ffeaa7a225af0e723986f185ddc65b59e19b7b0d5f27bdecabc37"
    );
    let subject_only = grant_index(&[(ArchiveSubjectKind::Place, "2622000", "subject")]);
    let redacted = desired_place_projection(&plan, &subject_only).expect("subject-only projection");
    assert_eq!(
        hex_digest(place_page_semantic_sha256(plan.place_geoid(), &redacted)),
        "dbc6bf5247431dcf54031d15109bb28a872e914636a538fffc94ab70b6c92e79"
    );
}

#[test]
fn grant_refresh_republication_folds_grant_visibility_into_dirty_detection() {
    let plan = detroit_plan();
    let renderer = FogSafeArchiveRenderer::new().expect("pinned template compiles");
    let subject_only = grant_index(&[(ArchiveSubjectKind::Place, "2622000", "subject")]);

    // Publish with only the subject grant: no signal section, county redlinks.
    let redacted_render = renderer
        .render(
            &place_page_input(&plan, 1, [0x11; 32]).expect("page input"),
            &knowledge_for(&[&plan], false),
        )
        .expect("subject-granted page renders");
    let redacted_stored =
        parse_stored_place_page("2622000", "Detroit city", redacted_render.markdown())
            .expect("redacted page parses back");
    assert!(redacted_stored.signals().is_empty());
    assert_eq!(redacted_stored.counties(), &[("26163".to_owned(), None)]);

    // Later identity and county grants arrive: the page re-dirties.
    let mut stored_map = BTreeMap::new();
    stored_map.insert("2622000".to_owned(), redacted_stored);
    let pages = [detroit_plan()];
    let refreshed = select_dirty_place_pages(&pages, &stored_map, &full_grants(&plan), usize::MAX)
        .expect("grant-refresh selection");
    assert_eq!(
        refreshed.head().len(),
        1,
        "a grant refresh re-dirties the redacted page"
    );

    // The same stale grant set stays clean.
    let stale = select_dirty_place_pages(&pages, &stored_map, &subject_only, usize::MAX)
        .expect("stale-grant selection");
    assert!(
        stale.head().is_empty(),
        "without a grant change the redacted page stays clean"
    );

    // The republished page, rendered with the refreshed grants, is clean.
    let revealed_render = renderer
        .render(
            &place_page_input(&plan, 2, [0x22; 32]).expect("republished page input"),
            &knowledge_for(&[&plan], true),
        )
        .expect("revealed page renders");
    let revealed_stored =
        parse_stored_place_page("2622000", "Detroit city", revealed_render.markdown())
            .expect("revealed page parses back");
    assert_eq!(revealed_stored.signals().len(), 1);
    assert_eq!(
        revealed_stored.counties(),
        &[("26163".to_owned(), Some("Wayne County".to_owned()))]
    );
    let mut revealed_map = BTreeMap::new();
    revealed_map.insert("2622000".to_owned(), revealed_stored);
    let settled = select_dirty_place_pages(&pages, &revealed_map, &full_grants(&plan), usize::MAX)
        .expect("settled selection");
    assert!(settled.head().is_empty(), "the revealed page stays clean");
}

#[test]
fn malformed_stored_pages_are_dirty_not_fatal() {
    assert!(parse_stored_place_page("2622000", "Detroit city", "not a page").is_none());
    assert!(parse_stored_place_page("2622000", "Other title", "# Detroit city\n\nq").is_none());

    // A stored row whose title column disagrees with its markdown republishes.
    let title_drift = render_markdown(&detroit_plan());
    assert!(parse_stored_place_page("2622000", "Different city", &title_drift).is_none());
    let parsed = parse_stored_place_page("2622000", "Detroit city", &title_drift)
        .expect("well-formed stored page parses");

    // A drifted signal bullet line refuses to parse, so the page republishes.
    let corrupted_signals = title_drift.replace(
        "- **Census identity:** Detroit city — census-place-authority-v1; \
         census_place_identity_mi_2023.csv.gz#place_geoid=2622000",
        "- **Census identity:** Detroit city",
    );
    assert!(
        parse_stored_place_page("2622000", "Detroit city", &corrupted_signals).is_none(),
        "a signal bullet that lost its citation is drifted"
    );

    let pages = vec![detroit_plan()];
    let grants = full_grants(&detroit_plan());
    let mut stored = BTreeMap::new();
    stored.insert("2622000".to_owned(), parsed);
    let clean =
        select_dirty_place_pages(&pages, &stored, &grants, usize::MAX).expect("clean selection");
    assert_eq!(clean.head().len(), 0);
}

#[test]
fn artifact_digests_match_the_governing_contracts() {
    assert_eq!(
        PINNED_PLACE_IDENTITY_ARTIFACT_SHA256,
        contract_digest(CONTRACT_PLACE_IDENTITY_SHA256)
    );
    assert_eq!(
        PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256,
        contract_digest(CONTRACT_COUNTY_PLACE_OVERLAP_SHA256)
    );
}

#[test]
fn place_page_read_sql_is_pinned() {
    assert_eq!(
        ARCHIVE_PLACE_PAGE_READ_SQL,
        "SELECT subject_id, title, markdown \
FROM babylon_meta.archive_page_v1 \
WHERE campaign_id = $1::uuid AND subject_kind = 'place' \
ORDER BY subject_id"
    );
    assert!(ARCHIVE_PLACE_PAGE_READ_SQL.contains("babylon_meta.archive_page_v1"));
    assert!(ARCHIVE_PLACE_PAGE_READ_SQL.contains("subject_kind = 'place'"));
}

#[test]
fn place_grants_sql_is_pinned() {
    assert_eq!(
        ARCHIVE_PLACE_GRANTS_SQL,
        "SELECT subject_kind, subject_id, grant_key \
FROM babylon_meta.archive_knowledge_grant_v1 \
WHERE campaign_id = $1::uuid AND granted_tick <= $2 \
  AND subject_kind IN ('county', 'place') \
ORDER BY subject_kind, subject_id, grant_key"
    );
    assert!(ARCHIVE_PLACE_GRANTS_SQL.contains("babylon_meta.archive_knowledge_grant_v1"));
    assert!(ARCHIVE_PLACE_GRANTS_SQL.contains("granted_tick <= $2"));
    assert!(
        ARCHIVE_PLACE_GRANTS_SQL.contains("subject_kind IN ('county', 'place')"),
        "page-subject knowledge only: seeded concept grants widen the grant \
         table's subject domain (ADR249 R3/R12) but never enter the page grant \
         snapshot, which decodes through the page-domain subject kind"
    );
}

#[test]
fn batch_from_produced_pages_matches_the_receipt() {
    let receipt = babylon_persistence::PendingArchiveReceipt::try_new(7, [0x77; 32])
        .expect("receipt boundary");
    let pages: Vec<PlacePagePlan> = desired_pages().iter().take(3).cloned().collect();
    let grants =
        PlaceGrantIndex::try_from_rows(full_grant_rows(&pages)).expect("subset grant index builds");
    let dirty = select_dirty_place_pages(&pages, &BTreeMap::new(), &grants, 3)
        .expect("small dirty set drains");
    let inputs = dirty
        .head()
        .iter()
        .map(|page| place_page_input(page, receipt.resolve_tick(), *receipt.tick_content_hash()))
        .collect::<Result<Vec<ArchivePageInput>, _>>()
        .expect("page inputs");
    for input in &inputs {
        assert_eq!(input.verified_tick(), receipt.resolve_tick());
        assert_eq!(input.tick_content_hash(), receipt.tick_content_hash());
        assert_eq!(input.decision_question(), PLACE_DECISION_QUESTION);
        assert_eq!(
            input.signals().len(),
            1,
            "one identity citation per place row"
        );
        assert_eq!(input.signals()[0].grant_key(), "identity");
        assert_eq!(
            input.signals()[0].citation().source_id(),
            "census-place-authority-v1"
        );
        assert!(input.signals()[0]
            .citation()
            .locator()
            .contains("#place_geoid="));
    }
    let batch =
        ArchiveDirtyBatch::try_new(receipt.resolve_tick(), *receipt.tick_content_hash(), inputs)
            .expect("dirty batch");
    assert_eq!(batch.pages().len(), 3);
}

fn subject_grant(geoid: &str) -> ArchiveKnowledgeGrant {
    ArchiveKnowledgeGrant::try_new(
        ArchivePageRef::try_new(ArchiveSubjectKind::Place, geoid.to_owned()).expect("place ref"),
        "subject".to_owned(),
        1,
        ArchiveCitation::try_new("archive-subject".to_owned(), format!("place/{geoid}"))
            .expect("subject citation"),
    )
    .expect("subject grant")
}

fn field_grant(geoid: &str, key: &str) -> ArchiveKnowledgeGrant {
    ArchiveKnowledgeGrant::try_new(
        ArchivePageRef::try_new(ArchiveSubjectKind::Place, geoid.to_owned()).expect("place ref"),
        key.to_owned(),
        1,
        ArchiveCitation::try_new("archive-field".to_owned(), format!("{key}@tick-1"))
            .expect("field citation"),
    )
    .expect("field grant")
}

fn county_subject_grant(county_geoid: &str) -> ArchiveKnowledgeGrant {
    ArchiveKnowledgeGrant::try_new(
        ArchivePageRef::try_new(ArchiveSubjectKind::County, county_geoid.to_owned())
            .expect("county ref"),
        "subject".to_owned(),
        1,
        ArchiveCitation::try_new(
            "archive-subject".to_owned(),
            format!("county/{county_geoid}"),
        )
        .expect("county citation"),
    )
    .expect("county grant")
}

fn knowledge_for(plans: &[&PlacePagePlan], reveal_fields: bool) -> ArchiveKnowledge {
    let mut grants = Vec::new();
    for plan in plans {
        grants.push(subject_grant(plan.place_geoid()));
        if reveal_fields {
            grants.push(field_grant(plan.place_geoid(), "identity"));
            for slice in plan.county_links() {
                grants.push(county_subject_grant(slice.county_geoid()));
            }
        }
    }
    ArchiveKnowledge::try_new(grants).expect("knowledge grants")
}

fn render_markdown(plan: &PlacePagePlan) -> String {
    let renderer = FogSafeArchiveRenderer::new().expect("pinned template compiles");
    let input = place_page_input(plan, 9, [0x99; 32]).expect("page input");
    let knowledge = knowledge_for(&[plan], true);
    renderer
        .render(&input, &knowledge)
        .expect("page renders")
        .markdown()
        .to_owned()
}
