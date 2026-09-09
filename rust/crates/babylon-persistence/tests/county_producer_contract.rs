//! Pure PER-22 county dossier producer contracts.
//!
//! These tests pin the producer's semantic decisions without any database:
//! mapping-driven enumeration order, `%.6f` statblock formatting, the D2
//! absence-maximal signal discipline, the grant-visible dirty projection, the
//! receipt-stamp-free semantic hash, loud drain-bound refusal, the pinned
//! read-only SQL, batch/receipt binding, sorted links, and the redlink form of
//! unknown place targets.

use std::collections::BTreeMap;

use babylon_persistence::{
    county_committed_signals_v1, county_page_input_v1, county_page_semantic_sha256_v1,
    desired_county_projection_v1, filter_granted_county_plans_v1, format_county_statblock_value_v1,
    parse_stored_county_page_v1, select_dirty_county_pages_v1, ArchiveCitationV1,
    ArchiveDirtyBatchV1, ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, ArchivePageInputV1,
    ArchivePageRefV1, ArchiveSubjectKindV1, CommittedTerritoryFieldsV1, CountyGrantIndexV1,
    CountyPagePlanV1, CountyPageProjectionV1, CountyPlaceLinkV1, CountySignalProjectionV1,
    CountySignalV1, FogSafeArchiveRendererV1, SemanticArchiveErrorV1,
    ARCHIVE_COUNTY_FIELD_READ_SQL_V1, ARCHIVE_COUNTY_GRANTS_SQL_V1, ARCHIVE_COUNTY_MAP_READ_SQL_V1,
    ARCHIVE_COUNTY_PAGE_READ_SQL_V1, COMMITTED_TICK_SOURCE_ID_V1, COUNTY_DECISION_QUESTION_V1,
    COUNTY_MEDIAN_WAGE_GRANT_KEY_V1, COUNTY_MEDIAN_WAGE_LABEL_V1, COUNTY_PHI_HOUR_GRANT_KEY_V1,
    COUNTY_PHI_HOUR_LABEL_V1,
};

#[test]
fn qcew_baseline_retains_exact_integers_units_and_public_provenance() {
    let fields = CommittedTerritoryFieldsV1::try_from_qcew([
        Some(36_727),
        Some(725_504),
        Some(55_436_615_328),
        Some(1_469),
    ])
    .expect("observed baseline");
    let signals = county_committed_signals_v1(&fields).expect("signals");
    assert_eq!(signals.len(), 4);
    assert_eq!(signals[2].value(), "55436615328");
    assert_eq!(signals[2].label(), "QCEW 2024 total annual wages (USD)");
    assert_eq!(signals[3].value(), "1469");
    assert_eq!(
        signals[3].label(),
        "QCEW 2024 average weekly wage (USD/week)"
    );
    assert!(signals
        .iter()
        .all(|signal| signal.grant_key().starts_with("qcew-")));
    let plan = wayne_plan(signals, vec![]);
    let knowledge = knowledge_for(&plan, true, false);
    let markdown = render_markdown(&plan, 1, &knowledge);
    assert!(markdown.contains("qcew-county-economics-v1; qcew_county_economics_mi_2024.csv.gz#county_geoid=26163&sha256=116affb2998c6c0259d5bf14840f99f835d7e0733aa0b4f4c60a257b2723cd16"));
    assert!(!markdown.contains("Median wage"));
    let parsed = parse_stored_county_page_v1("26163", "Wayne County", &markdown)
        .expect("QCEW source roundtrip");
    let desired =
        desired_county_projection_v1(&plan, &grant_index_for(&plan, true, false)).expect("desired");
    assert_eq!(parsed, desired);
    let later = render_markdown(&plan, 2, &knowledge);
    assert_eq!(
        parse_stored_county_page_v1("26163", "Wayne County", &later).expect("later source"),
        desired
    );
    assert!(
        desired_county_projection_v1(&plan, &grant_index_for(&plan, false, false))
            .expect("redacted")
            .signals()
            .is_empty()
    );
}

#[test]
fn qcew_fields_refuse_negative_values_and_preserve_large_exact_counts() {
    assert!(CommittedTerritoryFieldsV1::try_from_qcew([Some(-1), None, None, None]).is_err());
    let fields = CommittedTerritoryFieldsV1::try_from_qcew([None, None, Some(i64::MAX), None])
        .expect("exact integer");
    let signals = county_committed_signals_v1(&fields).expect("signals");
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].value(), "9223372036854775807");
}

fn signal(grant_key: &str, label: &str, value: &str) -> CountySignalV1 {
    CountySignalV1::try_new(grant_key.to_owned(), label.to_owned(), value.to_owned())
        .expect("county signal")
}

fn wayne_plan(signals: Vec<CountySignalV1>, links: Vec<CountyPlaceLinkV1>) -> CountyPagePlanV1 {
    CountyPagePlanV1::try_new(
        "26163".to_owned(),
        "wayne".to_owned(),
        "Wayne County".to_owned(),
        signals,
        links,
    )
    .expect("wayne plan")
}

fn detroit_link() -> CountyPlaceLinkV1 {
    CountyPlaceLinkV1::try_new("2622000".to_owned(), "Detroit city".to_owned())
        .expect("detroit link")
}

fn wayne_plan_full() -> CountyPagePlanV1 {
    wayne_plan(
        vec![
            signal(
                COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
                COUNTY_MEDIAN_WAGE_LABEL_V1,
                "21.000000",
            ),
            signal(
                COUNTY_PHI_HOUR_GRANT_KEY_V1,
                COUNTY_PHI_HOUR_LABEL_V1,
                "1.000000",
            ),
        ],
        vec![
            detroit_link(),
            CountyPlaceLinkV1::try_new("2668880".to_owned(), "Riverview city".to_owned())
                .expect("riverview link"),
        ],
    )
}

fn wayne_page_input_at(
    plan: &CountyPagePlanV1,
    resolve_tick: u64,
    tick_content_hash: [u8; 32],
) -> ArchivePageInputV1 {
    county_page_input_v1(plan, resolve_tick, tick_content_hash).expect("county page input")
}

fn knowledge_grant(
    kind: ArchiveSubjectKindV1,
    id: &str,
    grant_key: &str,
) -> ArchiveKnowledgeGrantV1 {
    ArchiveKnowledgeGrantV1::try_new(
        ArchivePageRefV1::try_new(kind, id.to_owned()).expect("page ref"),
        grant_key.to_owned(),
        1,
        ArchiveCitationV1::try_new(
            "archive-grant".to_owned(),
            format!("{}/{id}@{grant_key}", kind.as_str()),
        )
        .expect("grant citation"),
    )
    .expect("knowledge grant")
}

/// Renderer-side knowledge mirroring one campaign grant snapshot.
fn knowledge_for(
    plan: &CountyPagePlanV1,
    reveal_fields: bool,
    reveal_places: bool,
) -> ArchiveKnowledgeV1 {
    let mut grants = vec![knowledge_grant(
        ArchiveSubjectKindV1::County,
        plan.county_geoid(),
        "subject",
    )];
    if reveal_fields {
        for signal in plan.signals() {
            grants.push(knowledge_grant(
                ArchiveSubjectKindV1::County,
                plan.county_geoid(),
                signal.grant_key(),
            ));
        }
    }
    if reveal_places {
        for link in plan.place_links() {
            grants.push(knowledge_grant(
                ArchiveSubjectKindV1::Place,
                link.place_geoid(),
                "subject",
            ));
        }
    }
    ArchiveKnowledgeV1::try_new(grants).expect("knowledge grants")
}

/// Producer-side grant index mirroring one campaign grant snapshot.
fn grant_index_for(
    plan: &CountyPagePlanV1,
    reveal_fields: bool,
    reveal_places: bool,
) -> CountyGrantIndexV1 {
    let mut rows = vec![(
        ArchiveSubjectKindV1::County,
        plan.county_geoid().to_owned(),
        "subject".to_owned(),
    )];
    if reveal_fields {
        for signal in plan.signals() {
            rows.push((
                ArchiveSubjectKindV1::County,
                plan.county_geoid().to_owned(),
                signal.grant_key().to_owned(),
            ));
        }
    }
    if reveal_places {
        for link in plan.place_links() {
            rows.push((
                ArchiveSubjectKindV1::Place,
                link.place_geoid().to_owned(),
                "subject".to_owned(),
            ));
        }
    }
    CountyGrantIndexV1::try_from_rows(rows).expect("grant index")
}

fn render_markdown(
    plan: &CountyPagePlanV1,
    resolve_tick: u64,
    knowledge: &ArchiveKnowledgeV1,
) -> String {
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let input = county_page_input_v1(plan, resolve_tick, [0x99; 32]).expect("page input");
    renderer
        .render(&input, knowledge)
        .expect("page renders")
        .markdown()
        .to_owned()
}

fn stored_map(
    geoid: &str,
    title: &str,
    markdown: &str,
) -> BTreeMap<String, CountyPageProjectionV1> {
    let mut stored = BTreeMap::new();
    stored.insert(
        geoid.to_owned(),
        parse_stored_county_page_v1(geoid, title, markdown).expect("stored projection parses"),
    );
    stored
}

#[test]
fn decision_question_matches_the_established_county_fixture_phrasing() {
    assert_eq!(
        COUNTY_DECISION_QUESTION_V1,
        "Which neighboring place should organizers investigate next?"
    );
}

#[test]
fn committed_real_values_pin_python_statblock_formatting() {
    assert_eq!(
        format_county_statblock_value_v1(21.0).expect("format"),
        "21.000000"
    );
    assert_eq!(
        format_county_statblock_value_v1(1.0).expect("format"),
        "1.000000"
    );
    assert_eq!(
        format_county_statblock_value_v1(0.5).expect("format"),
        "0.500000"
    );
    assert_eq!(
        format_county_statblock_value_v1(f64::NAN),
        Err(SemanticArchiveErrorV1::InvalidText)
    );
    assert_eq!(
        format_county_statblock_value_v1(f64::INFINITY),
        Err(SemanticArchiveErrorV1::InvalidText)
    );
}

#[test]
fn negative_zero_canonicalizes_before_formatting() {
    assert_eq!(
        format_county_statblock_value_v1(-0.0).expect("format"),
        "0.000000",
        "negative zero canonicalizes to positive zero so the rendered page never drifts \
         on a sign-only bit difference"
    );
}

#[test]
fn stored_parser_accepts_the_exact_pinned_template_whitespace() {
    let plan = wayne_plan_full();
    let knowledge = knowledge_for(&plan, true, true);
    let markdown = render_markdown(&plan, 1, &knowledge);
    let parsed = parse_stored_county_page_v1("26163", "Wayne County", &markdown)
        .expect("template output parses");
    let projection = desired_county_projection_v1(&plan, &grant_index_for(&plan, true, true))
        .expect("desired projection");
    assert_eq!(
        county_page_semantic_sha256_v1("26163", &parsed),
        county_page_semantic_sha256_v1("26163", &projection),
        "the parser round-trips the renderer's exact whitespace, including the blank lines \
         the pinned template emits after each section heading"
    );

    let redacted = render_markdown(&plan, 1, &knowledge_for(&plan, false, false));
    let parsed_redacted =
        parse_stored_county_page_v1("26163", "Wayne County", &redacted).expect("redacted parses");
    assert!(
        parsed_redacted.signals().is_empty(),
        "a subject-only render carries no Signals section and no signals"
    );
}

#[test]
fn ungranted_county_is_filtered_before_batch_construction() {
    let plan = wayne_plan_full();
    let granted = grant_index_for(&plan, false, false);
    let granted_only = filter_granted_county_plans_v1(std::slice::from_ref(&plan), &granted);
    assert_eq!(
        granted_only.len(),
        1,
        "the subject grant keeps the county in this sweep's batch construction"
    );

    let empty = CountyGrantIndexV1::try_from_rows(Vec::new()).expect("empty grant index");
    let filtered = filter_granted_county_plans_v1(std::slice::from_ref(&plan), &empty);
    assert!(
        filtered.is_empty(),
        "a county without the subject grant produces no page this sweep; the renderer would \
         refuse the unknown subject and abort the whole sweep"
    );

    let stored = BTreeMap::new();
    let filtered_owned: Vec<CountyPagePlanV1> = filtered.into_iter().cloned().collect();
    let dirty = select_dirty_county_pages_v1(&filtered_owned, &stored, &empty, 256)
        .expect("empty filtered selection");
    assert!(
        dirty.head().is_empty(),
        "an all-ungranted sweep yields an empty batch and the receipt defers, never an error"
    );
}

#[test]
fn grant_visible_projection_hides_ungranted_signals_and_link_names() {
    let plan = wayne_plan_full();
    let empty = CountyGrantIndexV1::try_from_rows(Vec::new()).expect("empty grant index");
    let hidden = desired_county_projection_v1(&plan, &empty).expect("hidden projection");
    assert!(
        hidden.signals().is_empty(),
        "no field grant means no grant-visible signal"
    );
    assert_eq!(
        hidden.places(),
        &[("2622000".to_owned(), None), ("2668880".to_owned(), None)],
        "no place subject grant means every link stays a redlink"
    );

    let revealed = desired_county_projection_v1(&plan, &grant_index_for(&plan, true, true))
        .expect("revealed projection");
    assert_eq!(
        revealed.signals(),
        &[
            CountySignalProjectionV1::try_new(
                COUNTY_MEDIAN_WAGE_LABEL_V1.to_owned(),
                "21.000000".to_owned(),
                COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
                "wayne".to_owned(),
            )
            .expect("median projection"),
            CountySignalProjectionV1::try_new(
                COUNTY_PHI_HOUR_LABEL_V1.to_owned(),
                "1.000000".to_owned(),
                COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
                "wayne".to_owned(),
            )
            .expect("phi projection"),
        ]
    );
    assert_eq!(
        revealed.places(),
        &[
            ("2622000".to_owned(), Some("Detroit city".to_owned())),
            ("2668880".to_owned(), Some("Riverview city".to_owned())),
        ],
        "granted place subjects reveal their governed labels"
    );
}

#[test]
fn grant_arrival_redirties_the_redacted_page_and_the_reveal_settles() {
    let plan = wayne_plan_full();
    let subject_only = grant_index_for(&plan, false, false);
    let stored_markdown = render_markdown(&plan, 1, &knowledge_for(&plan, false, false));
    assert!(
        !stored_markdown.contains("## Signals"),
        "the subject-only render omits the Signals section"
    );
    assert!(
        stored_markdown.contains("[](subject:place/2622000)"),
        "the subject-only render keeps redlinks"
    );
    let stored = stored_map("26163", "Wayne County", &stored_markdown);
    let plans = [plan.clone()];
    let clean = select_dirty_county_pages_v1(&plans, &stored, &subject_only, 256)
        .expect("subject-only selection");
    assert!(
        clean.head().is_empty(),
        "the redacted stored page matches the subject-only projection exactly"
    );

    let revealed = grant_index_for(&plan, true, true);
    let dirty =
        select_dirty_county_pages_v1(&plans, &stored, &revealed, 256).expect("revealed selection");
    assert_eq!(
        dirty.head().len(),
        1,
        "grant arrival re-dirties the page so the next receipt republishes it revealed"
    );

    let revealed_markdown = render_markdown(&plan, 2, &knowledge_for(&plan, true, true));
    let settled = stored_map("26163", "Wayne County", &revealed_markdown);
    let clean =
        select_dirty_county_pages_v1(&plans, &settled, &revealed, 256).expect("settled selection");
    assert!(
        clean.head().is_empty(),
        "the revealed page settles: receipt stamps alone never re-publish it"
    );
}

#[test]
fn semantic_hash_covers_every_receipt_independent_rendered_input() {
    let plan = wayne_plan_full();
    let grants = grant_index_for(&plan, true, true);
    let base = desired_county_projection_v1(&plan, &grants).expect("base projection");
    let base_hash = county_page_semantic_sha256_v1("26163", &base);

    let same = desired_county_projection_v1(&plan, &grants).expect("same projection");
    assert_eq!(
        base_hash,
        county_page_semantic_sha256_v1("26163", &same),
        "an unchanged projection hashes identically"
    );

    let drifted_label = CountyPageProjectionV1::try_new(
        base.title().to_owned(),
        base.question().to_owned(),
        vec![CountySignalProjectionV1::try_new(
            "Median wages".to_owned(),
            "21.000000".to_owned(),
            COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
            "wayne".to_owned(),
        )
        .expect("drifted signal")],
        base.places().to_vec(),
    )
    .expect("drifted projection");
    assert_ne!(
        base_hash,
        county_page_semantic_sha256_v1("26163", &drifted_label),
        "a pinned signal label change republishes"
    );

    let drifted_citation = CountyPageProjectionV1::try_new(
        base.title().to_owned(),
        base.question().to_owned(),
        vec![CountySignalProjectionV1::try_new(
            COUNTY_MEDIAN_WAGE_LABEL_V1.to_owned(),
            "21.000000".to_owned(),
            "committed-tick-v2".to_owned(),
            "wayne".to_owned(),
        )
        .expect("drifted citation")],
        base.places().to_vec(),
    )
    .expect("drifted projection");
    assert_ne!(
        base_hash,
        county_page_semantic_sha256_v1("26163", &drifted_citation),
        "a citation source identity change republishes"
    );

    let drifted_provenance = CountyPageProjectionV1::try_new(
        base.title().to_owned(),
        base.question().to_owned(),
        vec![CountySignalProjectionV1::try_new(
            COUNTY_MEDIAN_WAGE_LABEL_V1.to_owned(),
            "21.000000".to_owned(),
            COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
            "wayne-old".to_owned(),
        )
        .expect("drifted provenance")],
        base.places().to_vec(),
    )
    .expect("drifted projection");
    assert_ne!(
        base_hash,
        county_page_semantic_sha256_v1("26163", &drifted_provenance),
        "a provenance name change republishes"
    );

    let redlinked = CountyPageProjectionV1::try_new(
        base.title().to_owned(),
        base.question().to_owned(),
        base.signals().to_vec(),
        vec![("2622000".to_owned(), None), ("2668880".to_owned(), None)],
    )
    .expect("redlinked projection");
    assert_ne!(
        base_hash,
        county_page_semantic_sha256_v1("26163", &redlinked),
        "link-label visibility is part of the projection"
    );

    let requestioned = CountyPageProjectionV1::try_new(
        base.title().to_owned(),
        "Which place needs organizers next?".to_owned(),
        base.signals().to_vec(),
        base.places().to_vec(),
    )
    .expect("requestioned projection");
    assert_ne!(
        base_hash,
        county_page_semantic_sha256_v1("26163", &requestioned),
        "a decision-question change republishes"
    );
}

#[test]
fn stored_page_parser_round_trips_exact_renderer_output() {
    let plan = wayne_plan_full();
    for (reveal_fields, reveal_places) in [(false, false), (true, false), (true, true)] {
        let markdown = render_markdown(
            &plan,
            9,
            &knowledge_for(&plan, reveal_fields, reveal_places),
        );
        let parsed = parse_stored_county_page_v1("26163", "Wayne County", &markdown)
            .expect("exact renderer output parses");
        let expected = desired_county_projection_v1(
            &plan,
            &grant_index_for(&plan, reveal_fields, reveal_places),
        )
        .expect("desired projection");
        assert_eq!(
            parsed, expected,
            "renderer output round-trips at any grant level"
        );
    }
}

#[test]
fn stored_page_parser_refuses_template_drift() {
    assert!(
        parse_stored_county_page_v1("26163", "Wayne County", "not a page").is_none(),
        "malformed stored pages are treated as dirty"
    );
    let revealed = render_markdown(
        &wayne_plan_full(),
        9,
        &knowledge_for(&wayne_plan_full(), true, true),
    );
    let wrong_subject = revealed.replace("subject: county/26163", "subject: county/26125");
    assert!(parse_stored_county_page_v1("26163", "Wayne County", &wrong_subject).is_none());
    let wrong_title = revealed.replace("# Wayne County", "# Wayne");
    assert!(parse_stored_county_page_v1("26163", "Wayne County", &wrong_title).is_none());
    let drifted_citation_tick = revealed.replace("campaign/9/wayne", "campaign/8/wayne");
    assert!(
        parse_stored_county_page_v1("26163", "Wayne County", &drifted_citation_tick).is_none(),
        "a citation locator that disagrees with the receipt stamp is drift"
    );
    let deep_locator = revealed.replace("campaign/9/wayne", "campaign/9/wayne/extra");
    assert!(
        parse_stored_county_page_v1("26163", "Wayne County", &deep_locator).is_none(),
        "a locator outside the pinned campaign/tick/name shape is drift"
    );
}

#[test]
fn grant_index_refuses_malformed_rows() {
    assert!(
        CountyGrantIndexV1::try_from_rows([(
            ArchiveSubjectKindV1::County,
            "26163".to_owned(),
            String::new(),
        )])
        .is_err(),
        "an empty grant key refuses"
    );
    assert!(
        CountyGrantIndexV1::try_from_rows([(
            ArchiveSubjectKindV1::County,
            "x".to_owned(),
            "subject".to_owned(),
        )])
        .is_err(),
        "a malformed subject identity refuses"
    );
}

#[test]
fn signal_projection_refuses_round_trip_unsafe_text() {
    for (label, value) in [
        ("Median wage:** ", "21.000000"),
        ("Median wage", "21.000000 — x"),
    ] {
        assert!(
            CountySignalProjectionV1::try_new(
                label.to_owned(),
                value.to_owned(),
                COMMITTED_TICK_SOURCE_ID_V1.to_owned(),
                "wayne".to_owned(),
            )
            .is_err(),
            "round-trip-unsafe text refuses: {label:?} {value:?}"
        );
    }
}

#[test]
fn absent_committed_fields_emit_no_signal() {
    let plan = wayne_plan(Vec::new(), vec![detroit_link()]);
    let page = wayne_page_input_at(&plan, 1, [0x11; 32]);
    assert!(
        page.signals().is_empty(),
        "a county with no committed median-wage or phi-hour emits no signal"
    );

    let stored = BTreeMap::new();
    let plans = [plan.clone()];
    let dirty = select_dirty_county_pages_v1(&plans, &stored, &CountyGrantIndexV1::default(), 256)
        .expect("absent-field selection");
    assert_eq!(
        dirty.head().len(),
        1,
        "a brand-new county page is still dirty"
    );
}

#[test]
fn dirty_selection_pages_the_head_batch_beyond_one_receipt_bound() {
    let plans = ["26093", "26125", "26163"]
        .into_iter()
        .map(|geoid| {
            CountyPagePlanV1::try_new(
                geoid.to_owned(),
                "territory".to_owned(),
                "County".to_owned(),
                Vec::new(),
                Vec::new(),
            )
            .expect("plan")
        })
        .collect::<Vec<_>>();
    let stored = BTreeMap::new();
    let grants = CountyGrantIndexV1::default();
    let dirty =
        select_dirty_county_pages_v1(&plans, &stored, &grants, 3).expect("within-bound selection");
    assert_eq!(
        dirty
            .head()
            .iter()
            .map(|plan| plan.county_geoid())
            .collect::<Vec<_>>(),
        vec!["26093", "26125", "26163"],
        "selection follows GEOID order"
    );
    assert_eq!(
        dirty.remaining(),
        0,
        "a within-bound dirty set drains whole"
    );

    let paged = select_dirty_county_pages_v1(&plans, &stored, &grants, 2)
        .expect("an over-bound dirty set pages instead of refusing");
    assert_eq!(
        paged
            .head()
            .iter()
            .map(|plan| plan.county_geoid())
            .collect::<Vec<_>>(),
        vec!["26093", "26125"],
        "the head batch keeps the leading GEOID prefix"
    );
    assert_eq!(
        paged.remaining(),
        1,
        "the undrained tail count keeps the receipt pending across sweeps"
    );
}

#[test]
fn plans_sort_signals_and_links_and_refuse_duplicates() {
    let plan = CountyPagePlanV1::try_new(
        "26163".to_owned(),
        "wayne".to_owned(),
        "Wayne County".to_owned(),
        vec![
            signal(
                COUNTY_PHI_HOUR_GRANT_KEY_V1,
                COUNTY_PHI_HOUR_LABEL_V1,
                "1.000000",
            ),
            signal(
                COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
                COUNTY_MEDIAN_WAGE_LABEL_V1,
                "21.000000",
            ),
        ],
        vec![
            CountyPlaceLinkV1::try_new("2668880".to_owned(), "Riverview city".to_owned())
                .expect("riverview link"),
            detroit_link(),
        ],
    )
    .expect("plan");
    let grant_keys = plan
        .signals()
        .iter()
        .map(CountySignalV1::grant_key)
        .collect::<Vec<_>>();
    assert_eq!(
        grant_keys,
        vec![
            COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
            COUNTY_PHI_HOUR_GRANT_KEY_V1
        ]
    );
    let geoids = plan
        .place_links()
        .iter()
        .map(CountyPlaceLinkV1::place_geoid)
        .collect::<Vec<_>>();
    assert_eq!(geoids, vec!["2622000", "2668880"]);

    assert_eq!(
        CountyPagePlanV1::try_new(
            "26163".to_owned(),
            "wayne".to_owned(),
            "Wayne County".to_owned(),
            vec![
                signal(
                    COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
                    COUNTY_MEDIAN_WAGE_LABEL_V1,
                    "21.000000"
                ),
                signal(
                    COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
                    COUNTY_MEDIAN_WAGE_LABEL_V1,
                    "22.000000"
                ),
            ],
            Vec::new(),
        ),
        Err(SemanticArchiveErrorV1::DuplicateKey)
    );
    assert_eq!(
        CountyPagePlanV1::try_new(
            "26163".to_owned(),
            "wayne".to_owned(),
            "Wayne County".to_owned(),
            Vec::new(),
            vec![detroit_link(), detroit_link()],
        ),
        Err(SemanticArchiveErrorV1::DuplicateKey)
    );
}

#[test]
fn page_input_pins_committed_tick_provenance_citations() {
    let page = wayne_page_input_at(&wayne_plan_full(), 7, [0x11; 32]);
    assert_eq!(page.verified_tick(), 7);
    assert_eq!(page.tick_content_hash(), &[0x11; 32]);
    assert_eq!(page.decision_question(), COUNTY_DECISION_QUESTION_V1);
    assert_eq!(page.signals().len(), 2);
    let median = &page.signals()[0];
    assert_eq!(median.grant_key(), COUNTY_MEDIAN_WAGE_GRANT_KEY_V1);
    assert_eq!(median.label(), COUNTY_MEDIAN_WAGE_LABEL_V1);
    assert_eq!(median.value(), "21.000000");
    assert_eq!(median.citation().source_id(), COMMITTED_TICK_SOURCE_ID_V1);
    assert_eq!(median.citation().locator(), "campaign/7/wayne");
    let phi = &page.signals()[1];
    assert_eq!(phi.grant_key(), COUNTY_PHI_HOUR_GRANT_KEY_V1);
    assert_eq!(phi.value(), "1.000000");
    assert_eq!(phi.citation().locator(), "campaign/7/wayne");
    let targets = page
        .links()
        .iter()
        .map(|link| link.target().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        targets,
        vec![
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2622000".to_owned())
                .expect("detroit ref"),
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2668880".to_owned())
                .expect("riverview ref"),
        ]
    );
}

#[test]
fn batch_refuses_pages_bound_to_another_receipt() {
    let plan = wayne_plan_full();
    let first = wayne_page_input_at(&plan, 1, [0x11; 32]);
    let second = wayne_page_input_at(&plan, 2, [0x22; 32]);
    let batch = ArchiveDirtyBatchV1::try_new(1, [0x11; 32], vec![first]).expect("batch");
    assert_eq!(batch.pages().len(), 1);
    assert_eq!(
        ArchiveDirtyBatchV1::try_new(1, [0x11; 32], vec![second]),
        Err(SemanticArchiveErrorV1::ReceiptMismatch)
    );
}

#[test]
fn pinned_read_sql_stays_read_only_and_scope_exact() {
    assert!(ARCHIVE_COUNTY_MAP_READ_SQL_V1.contains("babylon_meta.territory_county_map_v1"));
    assert!(ARCHIVE_COUNTY_MAP_READ_SQL_V1.contains("county_geoid"));
    assert!(ARCHIVE_COUNTY_MAP_READ_SQL_V1.contains("ORDER BY county_geoid"));

    assert!(ARCHIVE_COUNTY_FIELD_READ_SQL_V1.contains("babylon_state.territory_state_v1"));
    assert!(ARCHIVE_COUNTY_FIELD_READ_SQL_V1.contains("babylon_state.territory_state_field_v1"));
    assert!(ARCHIVE_COUNTY_FIELD_READ_SQL_V1.contains("resolve_tick = $2"));
    assert!(ARCHIVE_COUNTY_FIELD_READ_SQL_V1.contains("'median-wage'"));
    assert!(ARCHIVE_COUNTY_FIELD_READ_SQL_V1.contains("'phi-hour'"));

    assert!(ARCHIVE_COUNTY_PAGE_READ_SQL_V1.contains("babylon_meta.archive_page_v1"));
    assert!(ARCHIVE_COUNTY_PAGE_READ_SQL_V1.contains("subject_kind = 'county'"));

    assert!(ARCHIVE_COUNTY_GRANTS_SQL_V1.contains("babylon_meta.archive_knowledge_grant_v1"));
    assert!(ARCHIVE_COUNTY_GRANTS_SQL_V1.contains("granted_tick <= $2"));
    assert!(
        ARCHIVE_COUNTY_GRANTS_SQL_V1.contains("subject_kind IN ('county', 'place')"),
        "page-subject knowledge only: seeded concept grants widen the grant \
         table's subject domain (ADR249 R3/R12) but never enter the page grant \
         snapshot, which decodes through the page-domain subject kind"
    );
    assert!(ARCHIVE_COUNTY_GRANTS_SQL_V1.contains("ORDER BY"));
    for sql in [
        ARCHIVE_COUNTY_MAP_READ_SQL_V1,
        ARCHIVE_COUNTY_FIELD_READ_SQL_V1,
        ARCHIVE_COUNTY_PAGE_READ_SQL_V1,
        ARCHIVE_COUNTY_GRANTS_SQL_V1,
    ] {
        assert!(!sql.contains("archive_dirty_receipt_v1"));
        assert!(!sql.contains("tick_event"));
        assert!(!sql.contains("INSERT"));
        assert!(!sql.contains("UPDATE"));
    }
}
