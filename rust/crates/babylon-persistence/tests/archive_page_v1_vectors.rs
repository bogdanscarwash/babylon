//! Shared language-neutral vectors for the pinned fog-safe Archive page renderer.

use babylon_persistence::{
    archive_worker_contract_sha256_v1, ArchiveCitationV1, ArchiveDirtyBatchV1,
    ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, ArchiveLinkV1, ArchivePageInputV1,
    ArchivePageRefV1, ArchiveSignalV1, ArchiveSubjectKindV1, ArchiveSubjectV1,
    FogSafeArchiveRendererV1, SemanticArchiveErrorV1, ARCHIVE_PAGE_TEMPLATE_SHA256_V1,
    SEMANTIC_ARCHIVE_SCHEMA_V1_SQL,
};
use serde_json::{json, Value};

const VECTORS: &str = include_str!("../../../../contracts/archive_page_v1_vectors.jsonl");
const MAX_ROWS: usize = 32;
const MAX_LINE_BYTES: usize = 16_384;
const TICK_CONTENT_HASH_HEX: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const DECISION_QUESTION: &str = "Which neighboring place should organizers investigate next?";

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn hex_decode(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "even hex length");
    text.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let byte = std::str::from_utf8(chunk).expect("ASCII hex");
            u8::from_str_radix(byte, 16).expect("hex byte")
        })
        .collect()
}

fn citation_json(source_id: &str, locator: &str) -> Value {
    json!({"source_id": source_id, "locator": locator})
}

fn county_ref_json() -> Value {
    json!({"kind": "county", "id": "26163"})
}

fn detroit_ref_json() -> Value {
    json!({"kind": "place", "id": "2622000"})
}

fn riverview_ref_json() -> Value {
    json!({"kind": "place", "id": "2668880"})
}

fn page_input_json() -> Value {
    json!({
        "subject": {"kind": "county", "id": "26163", "title": "Wayne County"},
        "verified_tick": 42,
        "tick_content_hash_hex": TICK_CONTENT_HASH_HEX,
        "decision_question": DECISION_QUESTION,
        "signals": [{
            "grant_key": "employment",
            "label": "Employment",
            "value": "728576 jobs",
            "citation": citation_json(
                "qcew-2024",
                "fact_qcew_county_rollup county_fips=26163",
            ),
        }],
        "links": [
            {"target": detroit_ref_json(), "known_label": "Detroit"},
            {"target": riverview_ref_json(), "known_label": "Riverview"},
        ],
    })
}

fn subject_grant_json(page_ref: &Value, locator: &str) -> Value {
    json!({
        "page_ref": page_ref,
        "grant_key": "subject",
        "granted_tick": 42,
        "citation": citation_json("archive-subject", locator),
    })
}

fn full_knowledge_json() -> Value {
    json!([
        subject_grant_json(&county_ref_json(), "county/26163"),
        {
            "page_ref": county_ref_json(),
            "grant_key": "employment",
            "granted_tick": 42,
            "citation": citation_json("knowledge-event", "employment@tick-42"),
        },
        subject_grant_json(&detroit_ref_json(), "place/2622000"),
    ])
}

fn link_grant_absent_knowledge_json() -> Value {
    json!([
        subject_grant_json(&county_ref_json(), "county/26163"),
        {
            "page_ref": county_ref_json(),
            "grant_key": "employment",
            "granted_tick": 42,
            "citation": citation_json("knowledge-event", "employment@tick-42"),
        },
    ])
}

fn unknown_subject_knowledge_json() -> Value {
    json!([subject_grant_json(&detroit_ref_json(), "place/2622000")])
}

fn parse_kind(value: &str) -> ArchiveSubjectKindV1 {
    match value {
        "county" => ArchiveSubjectKindV1::County,
        "place" => ArchiveSubjectKindV1::Place,
        other => panic!("unknown subject kind: {other}"),
    }
}

fn parse_ref(value: &Value) -> ArchivePageRefV1 {
    ArchivePageRefV1::try_new(
        parse_kind(value["kind"].as_str().expect("kind text")),
        value["id"].as_str().expect("id text").to_owned(),
    )
    .expect("valid page reference")
}

fn parse_citation(value: &Value) -> ArchiveCitationV1 {
    ArchiveCitationV1::try_new(
        value["source_id"]
            .as_str()
            .expect("source id text")
            .to_owned(),
        value["locator"].as_str().expect("locator text").to_owned(),
    )
    .expect("valid citation")
}

fn parse_page_input(value: &Value) -> ArchivePageInputV1 {
    let subject = value["subject"].as_object().expect("subject object");
    let signals = value["signals"]
        .as_array()
        .expect("signals array")
        .iter()
        .map(|signal| {
            ArchiveSignalV1::try_new(
                signal["grant_key"]
                    .as_str()
                    .expect("grant key text")
                    .to_owned(),
                signal["label"].as_str().expect("label text").to_owned(),
                signal["value"].as_str().expect("value text").to_owned(),
                parse_citation(&signal["citation"]),
            )
            .expect("valid signal")
        })
        .collect();
    let links = value["links"]
        .as_array()
        .expect("links array")
        .iter()
        .map(|link| {
            ArchiveLinkV1::try_new(
                parse_ref(&link["target"]),
                link["known_label"].as_str().expect("label text").to_owned(),
            )
            .expect("valid link")
        })
        .collect();
    ArchivePageInputV1::try_new(
        ArchiveSubjectV1::try_new(
            parse_kind(subject["kind"].as_str().expect("kind text")),
            subject["id"].as_str().expect("id text").to_owned(),
            subject["title"].as_str().expect("title text").to_owned(),
        )
        .expect("valid subject"),
        value["verified_tick"].as_u64().expect("u64 verified tick"),
        hex_decode(value["tick_content_hash_hex"].as_str().expect("hash hex"))
            .try_into()
            .expect("exact 32-byte hash"),
        value["decision_question"]
            .as_str()
            .expect("decision question text")
            .to_owned(),
        signals,
        links,
    )
    .expect("valid page input")
}

fn parse_knowledge(value: &Value) -> ArchiveKnowledgeV1 {
    let grants = value
        .as_array()
        .expect("knowledge array")
        .iter()
        .map(|grant| {
            ArchiveKnowledgeGrantV1::try_new(
                parse_ref(&grant["page_ref"]),
                grant["grant_key"]
                    .as_str()
                    .expect("grant key text")
                    .to_owned(),
                grant["granted_tick"].as_u64().expect("u64 granted tick"),
                parse_citation(&grant["citation"]),
            )
            .expect("valid knowledge grant")
        })
        .collect();
    ArchiveKnowledgeV1::try_new(grants).expect("valid knowledge snapshot")
}

fn rows() -> Vec<Value> {
    let input = VECTORS.strip_suffix('\n').unwrap_or(VECTORS);
    let mut rows = Vec::with_capacity(MAX_ROWS);
    for (index, line) in input.split('\n').take(MAX_ROWS + 1).enumerate() {
        assert!(index < MAX_ROWS, "bounded vector row count");
        assert!(!line.is_empty() && line.len() <= MAX_LINE_BYTES);
        let row: Value = serde_json::from_str(line).expect("valid bounded vector row");
        assert!(
            row["id"].is_string() && row["kind"].is_string() && row["data"].is_object(),
            "vector row shape"
        );
        rows.push(row);
    }
    rows
}

fn rows_of_kind<'a>(rows: &'a [Value], kind: &'a str) -> impl Iterator<Item = &'a Value> + 'a {
    rows.iter()
        .filter(move |row| row["kind"].as_str() == Some(kind))
}

fn generate_vectors() -> String {
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let mut lines = Vec::new();
    for (id, knowledge_json) in [
        ("render-known-county", full_knowledge_json()),
        (
            "render-link-grant-absent",
            link_grant_absent_knowledge_json(),
        ),
    ] {
        let data = page_input_json();
        let knowledge = parse_knowledge(&knowledge_json);
        let page = renderer
            .render(&parse_page_input(&data), &knowledge)
            .expect("known page renders");
        let markdown = page.markdown().as_bytes();
        let mut row_data = data;
        row_data["knowledge"] = knowledge_json;
        row_data["markdown_hex"] = hex_encode(markdown).into();
        row_data["content_sha256_hex"] = hex_encode(&page.sha256()).into();
        row_data["search_text"] = page.search_text().into();
        row_data["citations"] = page
            .citations()
            .iter()
            .map(|citation| citation_json(citation.source_id(), citation.locator()))
            .collect::<Vec<_>>()
            .into();
        lines.push(json!({"id": id, "kind": "render", "data": row_data}));
    }
    {
        let mut refusal = page_input_json();
        refusal["knowledge"] = unknown_subject_knowledge_json();
        refusal["operation"] = "render".into();
        refusal["expected_code"] = "unknown_subject".into();
        lines.push(json!({"id": "refusal-unknown-subject", "kind": "refusal", "data": refusal}));
    }
    for (id, pages) in [
        ("batch-empty", Vec::new()),
        ("batch-one-page", vec![page_input_json()]),
    ] {
        let parsed_pages = pages.iter().map(parse_page_input).collect();
        let batch =
            ArchiveDirtyBatchV1::try_new(42, [0x11; 32], parsed_pages).expect("valid dirty batch");
        lines.push(json!({
            "id": id,
            "kind": "batch",
            "data": {
                "resolve_tick": 42,
                "tick_content_hash_hex": TICK_CONTENT_HASH_HEX,
                "pages": pages,
                "sha256_hex": hex_encode(&batch.sha256()),
            },
        }));
    }
    lines.push(json!({
        "id": "identity-template-and-worker",
        "kind": "identity",
        "data": {
            "template_path": "rust/crates/babylon-persistence/src/archive_page_v1.md.j2",
            "template_sha256_hex": hex_encode(&ARCHIVE_PAGE_TEMPLATE_SHA256_V1),
            "schema_path": "rust/crates/babylon-persistence/migrations/semantic_archive_v1.sql",
            "worker_domain_ascii_nul": "babylon.semantic-archive-worker.v1",
            "worker_contract_sha256_hex": hex_encode(&archive_worker_contract_sha256_v1()),
        },
    }));
    let mut output = String::new();
    for line in lines {
        output.push_str(&serde_json::to_string(&line).expect("vector row serializes"));
        output.push('\n');
    }
    output
}

#[test]
#[ignore = "regeneration writes contracts/archive_page_v1_vectors.jsonl from the pinned renderer"]
fn generate_shared_vectors_from_the_pinned_renderer() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/archive_page_v1_vectors.jsonl"
    );
    std::fs::write(path, generate_vectors()).expect("vector corpus written");
}

#[test]
fn shared_vectors_reproduce_exact_renderer_bytes() {
    let rows = rows();
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let render_rows: Vec<&Value> = rows_of_kind(&rows, "render").collect();
    assert_eq!(render_rows.len(), 2);
    for row in &render_rows {
        let row = *row;
        let data = &row["data"];
        let page = renderer
            .render(
                &parse_page_input(data),
                &parse_knowledge(&data["knowledge"]),
            )
            .expect("vector page renders");
        assert_eq!(
            page.markdown().as_bytes(),
            hex_decode(data["markdown_hex"].as_str().expect("markdown hex")).as_slice(),
            "{}",
            row["id"]
        );
        assert_eq!(
            hex_encode(&page.sha256()),
            data["content_sha256_hex"].as_str().expect("content sha"),
            "{}",
            row["id"]
        );
        assert_eq!(
            page.search_text(),
            data["search_text"].as_str().expect("search text"),
            "{}",
            row["id"]
        );
        let expected_citations = data["citations"].as_array().expect("citations array");
        assert_eq!(
            page.citations().len(),
            expected_citations.len(),
            "{}",
            row["id"]
        );
        for (actual, expected) in page.citations().iter().zip(expected_citations) {
            assert_eq!(
                actual.source_id(),
                expected["source_id"].as_str().expect("source id"),
                "{}",
                row["id"]
            );
            assert_eq!(
                actual.locator(),
                expected["locator"].as_str().expect("locator"),
                "{}",
                row["id"]
            );
        }
    }
    let known = render_rows[0]["data"]["markdown_hex"]
        .as_str()
        .expect("markdown hex");
    let sparse = render_rows[1]["data"]["markdown_hex"]
        .as_str()
        .expect("markdown hex");
    assert_ne!(
        known, sparse,
        "grant absence must move the rendered page bytes"
    );
}

#[test]
fn shared_refusal_vectors_match_the_closed_renderer_refusal() {
    let rows = rows();
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let refusal_rows: Vec<&Value> = rows_of_kind(&rows, "refusal").collect();
    assert_eq!(refusal_rows.len(), 1);
    for row in refusal_rows {
        let data = &row["data"];
        assert_eq!(data["operation"].as_str(), Some("render"));
        let result = renderer.render(
            &parse_page_input(data),
            &parse_knowledge(&data["knowledge"]),
        );
        assert_eq!(
            result,
            Err(SemanticArchiveErrorV1::UnknownSubject),
            "{}",
            row["id"]
        );
        assert_eq!(data["expected_code"].as_str(), Some("unknown_subject"));
    }
}

#[test]
fn shared_batch_vectors_match_the_exact_batch_identity() {
    let rows = rows();
    let batch_rows: Vec<&Value> = rows_of_kind(&rows, "batch").collect();
    assert_eq!(batch_rows.len(), 2);
    for row in batch_rows {
        let data = &row["data"];
        let pages = data["pages"]
            .as_array()
            .expect("pages array")
            .iter()
            .map(parse_page_input)
            .collect();
        let batch = ArchiveDirtyBatchV1::try_new(
            data["resolve_tick"].as_u64().expect("u64 resolve tick"),
            hex_decode(data["tick_content_hash_hex"].as_str().expect("hash hex"))
                .try_into()
                .expect("exact 32-byte hash"),
            pages,
        )
        .expect("vector batch");
        assert_eq!(
            hex_encode(&batch.sha256()),
            data["sha256_hex"].as_str().expect("batch sha"),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn shared_identity_vectors_match_the_pinned_template_and_worker_contract() {
    let rows = rows();
    let identity_rows: Vec<&Value> = rows_of_kind(&rows, "identity").collect();
    assert_eq!(identity_rows.len(), 1);
    let data = &identity_rows[0]["data"];
    assert_eq!(
        data["template_sha256_hex"].as_str(),
        Some(hex_encode(&ARCHIVE_PAGE_TEMPLATE_SHA256_V1).as_str())
    );
    assert_eq!(
        data["worker_contract_sha256_hex"].as_str(),
        Some(hex_encode(&archive_worker_contract_sha256_v1()).as_str())
    );
    assert!(SEMANTIC_ARCHIVE_SCHEMA_V1_SQL.contains("archive_page_v1"));
}
