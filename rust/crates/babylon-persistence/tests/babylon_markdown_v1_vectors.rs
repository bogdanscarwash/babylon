//! Shared language-neutral vectors for the Babylon Markdown profile (ADR249 R4/R5).

use babylon_persistence::{
    fog_chip_v1, git_export_markdown_v1, is_citation_line_v1, validate_babylon_markdown_v1,
    ArchiveCitationV1, ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, ArchiveLinkV1,
    ArchivePageInputV1, ArchivePageRefV1, ArchiveSignalV1, ArchiveSubjectKindV1, ArchiveSubjectV1,
    FogSafeArchiveRendererV1, BABYLON_MARKDOWN_PROFILE_ID_V1, CITATION_LINE_REGEX_V1,
    FOG_CHIP_SEPARATOR_V1,
};
use serde_json::{json, Value};

const VECTORS: &str = include_str!("../../../../contracts/babylon_markdown_v1_vectors.jsonl");
const MAX_ROWS: usize = 24;
const MAX_LINE_BYTES: usize = 16_384;
const TICK_CONTENT_HASH_HEX: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const DECISION_QUESTION: &str = "Which neighboring place should organizers investigate next?";
const SOUTHFIELD_CHIP_V1: &str = "unknown place · 2674900";

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

fn detroit_ref_json() -> Value {
    json!({"kind": "place", "id": "2622000"})
}

fn riverview_ref_json() -> Value {
    json!({"kind": "place", "id": "2668880"})
}

fn subject_grant_json(page_ref: &Value, locator: &str) -> Value {
    json!({
        "page_ref": page_ref,
        "grant_key": "subject",
        "granted_tick": 42,
        "citation": citation_json("archive-subject", locator),
    })
}

fn county_page_input_json() -> Value {
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

fn all_subjects_known_json() -> Value {
    json!([
        subject_grant_json(&json!({"kind": "county", "id": "26163"}), "county/26163"),
        {
            "page_ref": {"kind": "county", "id": "26163"},
            "grant_key": "employment",
            "granted_tick": 42,
            "citation": citation_json("knowledge-event", "employment@tick-42"),
        },
        subject_grant_json(&detroit_ref_json(), "place/2622000"),
        subject_grant_json(&riverview_ref_json(), "place/2668880"),
    ])
}

fn riverview_ungranted_json() -> Value {
    json!([
        subject_grant_json(&json!({"kind": "county", "id": "26163"}), "county/26163"),
        {
            "page_ref": {"kind": "county", "id": "26163"},
            "grant_key": "employment",
            "granted_tick": 42,
            "citation": citation_json("knowledge-event", "employment@tick-42"),
        },
        subject_grant_json(&detroit_ref_json(), "place/2622000"),
    ])
}

/// Display-assembled profile bytes exercising every pinned form: a citation
/// line, a granted county link, a bare Southfield fog link, the pending
/// strikethrough display form, and one GFM table.
fn assembled_forms_markdown() -> String {
    "# Oakland County\n\
     \n\
     Should organizers canvass Southfield before the deadline?\n\
     \n\
     - **Median wage:** 25.000000 — committed-tick-v1; campaign/2/oakland\n\
     - [Oakland County](subject:county/26125)\n\
     - [](subject:place/2674900)\n\
     ~~[Detroit](subject:place/2622000)~~\n\
     \n\
     | gear | count |\n\
     | --- | --- |\n\
     | leaflets | 12 |\n"
        .to_owned()
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

fn render_archive_page(knowledge_json: &Value) -> String {
    let renderer = FogSafeArchiveRendererV1::new().expect("pinned template compiles");
    let data = county_page_input_json();
    let page = renderer
        .render(&parse_page_input(&data), &parse_knowledge(knowledge_json))
        .expect("known county page renders");
    page.markdown().to_owned()
}

fn valid_row(id: &str, markdown: &str) -> Value {
    let export = git_export_markdown_v1(markdown).expect("valid markdown exports");
    json!({
        "id": id,
        "kind": "valid",
        "data": {
            "markdown_hex": hex_encode(markdown.as_bytes()),
            "export_hex": hex_encode(export.as_bytes()),
        },
    })
}

fn refusal_row(id: &str, markdown: &str, expected_code: &str) -> Value {
    let error = validate_babylon_markdown_v1(markdown.as_bytes())
        .expect_err("refusal fixture must violate the profile");
    assert_eq!(
        error.code(),
        expected_code,
        "{id} refuses with the pinned code"
    );
    json!({
        "id": id,
        "kind": "refusal",
        "data": {
            "markdown_hex": hex_encode(markdown.as_bytes()),
            "expected_code": expected_code,
        },
    })
}

/// The pinned citation corpus: each row pins one line and whether the
/// byte-level detector and the pinned regex both recognize it.
fn citation_fixtures() -> [(&'static str, &'static str, bool); 7] {
    [
        (
            "citation-pinned-example",
            "- **Median wage:** 25.000000 — committed-tick-v1; campaign/2/oakland",
            true,
        ),
        (
            "citation-label-star-refuses",
            "- **Med*ian wage:** 25.000000 — committed-tick-v1; campaign/2/oakland",
            false,
        ),
        (
            "citation-source-semicolon-refuses",
            "- **Median wage:** 25.000000 — committed-tick-v1;extra; campaign/2/oakland",
            false,
        ),
        (
            "citation-double-separator-backtracks",
            "- **Median wage:** 25.000000 — committed-tick-v1; campaign/2 — east",
            true,
        ),
        (
            "citation-double-separator-refuses",
            "- **Median wage:** 25.000000 — committed-tick-v1 — campaign/2/oakland",
            false,
        ),
        (
            "citation-empty-value-refuses",
            "- **Median wage:**  — committed-tick-v1; campaign/2/oakland",
            false,
        ),
        (
            "citation-locator-semicolon-accepted",
            "- **Median wage:** 25.000000 — committed-tick-v1; campaign/2; extra",
            true,
        ),
    ]
}

fn generate_vectors() -> String {
    let mut lines = Vec::new();
    lines.push(valid_row(
        "valid-archive-page-granted-links",
        &render_archive_page(&all_subjects_known_json()),
    ));
    let bare_page = render_archive_page(&riverview_ungranted_json());
    lines.push(valid_row("valid-archive-page-bare-link", &bare_page));
    lines.push(valid_row(
        "valid-assembled-profile-forms",
        &assembled_forms_markdown(),
    ));
    lines.push(refusal_row(
        "refusal-crlf-ending",
        &bare_page.replacen('\n', "\r\n", 1),
        "crlf_ending",
    ));
    lines.push(refusal_row(
        "refusal-raw-html",
        &format!("{bare_page}<div>hidden</div>\n"),
        "raw_html",
    ));
    lines.push(refusal_row(
        "refusal-disallowed-link-scheme",
        &bare_page.replace("subject:place/2668880", "https://example.invalid/2668880"),
        "disallowed_link_scheme",
    ));
    lines.push(refusal_row(
        "refusal-malformed-subject-link",
        &bare_page.replace("subject:place/2668880", "subject:place/26688"),
        "malformed_subject_link",
    ));
    lines.push(refusal_row(
        "refusal-stray-open-bracket",
        &format!("{bare_page}A [ bracket drifts.\n"),
        "malformed_subject_link",
    ));
    lines.push(refusal_row(
        "refusal-strikethrough-without-link",
        &format!("{bare_page}~~pending rumor~~\n"),
        "strikethrough_without_link",
    ));
    lines.push(json!({
        "id": "identity-profile-constants",
        "kind": "identity",
        "data": {
            "profile_id": BABYLON_MARKDOWN_PROFILE_ID_V1,
            "citation_line_regex": CITATION_LINE_REGEX_V1,
            "chip_separator": FOG_CHIP_SEPARATOR_V1,
            "fog_chip_place_2674900": SOUTHFIELD_CHIP_V1,
        },
    }));
    for (id, line, recognized) in citation_fixtures() {
        assert_eq!(
            is_citation_line_v1(line),
            recognized,
            "{id}: the byte-level detector must match the pinned regex truth"
        );
        lines.push(json!({
            "id": id,
            "kind": "citation",
            "data": {"line": line, "recognized": recognized},
        }));
    }
    let mut output = String::new();
    for line in lines {
        output.push_str(&serde_json::to_string(&line).expect("vector row serializes"));
        output.push('\n');
    }
    output
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

#[test]
#[ignore = "regeneration writes contracts/babylon_markdown_v1_vectors.jsonl from the pinned validators"]
fn generate_shared_vectors_from_the_pinned_validators() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/babylon_markdown_v1_vectors.jsonl"
    );
    std::fs::write(path, generate_vectors()).expect("vector corpus written");
}

#[test]
fn shared_valid_vectors_validate_and_export_exact_bytes() {
    let rows = rows();
    let valid_rows: Vec<&Value> = rows_of_kind(&rows, "valid").collect();
    assert_eq!(valid_rows.len(), 3);
    for row in &valid_rows {
        let data = &row["data"];
        let markdown_bytes = hex_decode(data["markdown_hex"].as_str().expect("markdown hex"));
        validate_babylon_markdown_v1(&markdown_bytes)
            .unwrap_or_else(|error| panic!("{} validates: {error}", row["id"]));
        let markdown = std::str::from_utf8(&markdown_bytes).expect("profile markdown is UTF-8");
        let export = git_export_markdown_v1(markdown)
            .unwrap_or_else(|error| panic!("{} exports: {error}", row["id"]));
        assert_eq!(
            export.as_bytes(),
            hex_decode(data["export_hex"].as_str().expect("export hex")).as_slice(),
            "{}",
            row["id"]
        );
    }
    let bare_bytes = hex_decode(
        valid_rows[1]["data"]["markdown_hex"]
            .as_str()
            .expect("markdown hex"),
    );
    let bare = std::str::from_utf8(&bare_bytes).expect("profile markdown is UTF-8");
    let bare_export_bytes = hex_decode(
        valid_rows[1]["data"]["export_hex"]
            .as_str()
            .expect("export hex"),
    );
    let bare_export = std::str::from_utf8(&bare_export_bytes).expect("export is UTF-8");
    assert!(
        bare.contains("[](subject:place/2668880)"),
        "the bare fog form carries zero label bytes"
    );
    let mut link_cursor = 0;
    while let Some(offset) = bare[link_cursor..].find("](subject:place/2668880)") {
        let close = link_cursor + offset;
        let open = bare[..close]
            .rfind('[')
            .expect("a label bracket opens the link");
        assert!(
            bare[open + 1..close].is_empty(),
            "the fog bytes never carry a label for the ungranted place"
        );
        link_cursor = close + 1;
    }
    assert!(
        bare_export.contains("unknown place · 2668880"),
        "the export renders the bare link as the synthesized fog chip"
    );
    assert!(
        !bare_export.contains("Riverview"),
        "the export chip carries zero label bytes"
    );
    assert!(
        !bare_export.contains("](./place/2668880.md)"),
        "the export never carries a labeled link for the ungranted place"
    );
}

#[test]
fn shared_refusal_vectors_match_the_closed_validator() {
    let rows = rows();
    let refusal_rows: Vec<&Value> = rows_of_kind(&rows, "refusal").collect();
    assert_eq!(refusal_rows.len(), 6);
    for row in refusal_rows {
        let data = &row["data"];
        let markdown = hex_decode(data["markdown_hex"].as_str().expect("markdown hex"));
        let error = validate_babylon_markdown_v1(&markdown).expect_err("refusal row refuses");
        assert_eq!(
            error.code(),
            data["expected_code"].as_str().expect("expected code"),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn shared_identity_vectors_match_the_pinned_profile_constants() {
    let rows = rows();
    let identity_rows: Vec<&Value> = rows_of_kind(&rows, "identity").collect();
    assert_eq!(identity_rows.len(), 1);
    let data = &identity_rows[0]["data"];
    assert_eq!(
        data["profile_id"].as_str(),
        Some(BABYLON_MARKDOWN_PROFILE_ID_V1)
    );
    assert_eq!(
        data["citation_line_regex"].as_str(),
        Some(CITATION_LINE_REGEX_V1)
    );
    assert_eq!(data["chip_separator"].as_str(), Some(FOG_CHIP_SEPARATOR_V1));
    assert_eq!(
        data["fog_chip_place_2674900"].as_str(),
        Some(SOUTHFIELD_CHIP_V1)
    );
}

#[test]
fn shared_citation_vectors_match_the_pinned_regex_language() {
    let rows = rows();
    let citation_rows: Vec<&Value> = rows_of_kind(&rows, "citation").collect();
    assert_eq!(citation_rows.len(), 7);
    for row in citation_rows {
        let data = &row["data"];
        let line = data["line"].as_str().expect("citation line text");
        let recognized = data["recognized"].as_bool().expect("recognized flag");
        assert_eq!(
            is_citation_line_v1(line),
            recognized,
            "{}: the byte-level detector must match the pinned regex truth",
            row["id"]
        );
    }
}

#[test]
fn bare_link_chip_synthesizes_from_kind_and_id_with_zero_label_bytes() {
    let chip = fog_chip_v1("place", "2674900");
    assert_eq!(chip, SOUTHFIELD_CHIP_V1);
    assert_eq!(chip, fog_chip_v1("place", "2674900"));
    assert!(
        !chip.contains("Southfield"),
        "the chip is synthesized from kind and id alone; no label bytes exist"
    );
    let exported = git_export_markdown_v1("[](subject:place/2674900)").expect("bare link exports");
    assert_eq!(exported, SOUTHFIELD_CHIP_V1);
}

#[test]
fn citation_line_form_is_pinned_and_recognized() {
    let line = "- **Median wage:** 25.000000 — committed-tick-v1; campaign/2/oakland";
    assert!(is_citation_line_v1(line));
    assert!(!is_citation_line_v1("- **Median wage:** 25.000000"));
    assert!(!is_citation_line_v1("plain prose line"));
    let rows = rows();
    let valid_rows: Vec<&Value> = rows_of_kind(&rows, "valid").collect();
    let granted = hex_decode(
        valid_rows[0]["data"]["markdown_hex"]
            .as_str()
            .expect("markdown hex"),
    );
    let granted = std::str::from_utf8(&granted).expect("profile markdown is UTF-8");
    assert!(
        granted
            .lines()
            .any(|line| is_citation_line_v1(line) && line.contains("qcew-2024")),
        "a rendered Archive page carries the pinned citation-line format"
    );
}
