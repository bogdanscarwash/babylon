//! Shared language-neutral vectors for the pinned `ArchiveAtomV1` behavior.

use babylon_persistence::{
    archive_atom_visible_v1, ArchiveAtomSubjectKindV1, ArchiveAtomSubjectV1, ArchiveAtomV1,
    ArchiveAtomValueV1, ArchiveCitationV1, ArchiveEvidenceClassV1, CampaignId,
    SemanticArchiveErrorV1,
};
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

const VECTORS: &str = include_str!("../../../../contracts/archive_atom_v1_vectors.jsonl");
const MAX_ROWS: usize = 32;
const MAX_LINE_BYTES: usize = 16_384;
const MAX_OBJECT_FIELDS: usize = 64;
const CAMPAIGN_UUID: &str = "123e4567-e89b-42d3-a456-426614174000";
const REQUIRED_KINDS: [&str; 4] = ["encoding", "identity", "refusal", "visibility"];
const KIND_TAGS: [(&str, u8); 3] = [("county", 1), ("place", 2), ("concept", 3)];
const EVIDENCE_TAGS: [(&str, u8); 4] = [
    ("Observed", 1),
    ("Derived", 2),
    ("Calibrated", 3),
    ("Designed", 4),
];
const VALUE_TAGS: [(&str, u8); 4] = [("text", 1), ("f64", 2), ("u64", 3), ("bool", 4)];

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for line in VECTORS.lines() {
        assert!(line.len() <= MAX_LINE_BYTES, "bounded vector line");
        let row: Value = serde_json::from_str(line).expect("vector row parses");
        let object = row.as_object().expect("vector row object");
        assert!(object.len() <= MAX_OBJECT_FIELDS, "bounded vector fields");
        assert!(REQUIRED_KINDS.contains(&row["kind"].as_str().expect("kind string")));
        rows.push(row);
    }
    assert!(rows.len() <= MAX_ROWS, "bounded vector rows");
    rows
}

fn rows_of_kind<'a>(rows: &'a [Value], kind: &'a str) -> impl Iterator<Item = &'a Value> + 'a {
    rows.iter().filter(move |row| row["kind"] == kind)
}

fn campaign_id() -> CampaignId {
    CampaignId::from_uuid(CAMPAIGN_UUID.parse().expect("fixed campaign UUID"))
}

fn atom_from_data(data: &Value) -> ArchiveAtomV1 {
    let kind = match data["subject_kind"].as_str().expect("subject kind") {
        "county" => ArchiveAtomSubjectKindV1::County,
        "place" => ArchiveAtomSubjectKindV1::Place,
        "concept" => ArchiveAtomSubjectKindV1::Concept,
        other => panic!("unknown subject kind {other}"),
    };
    let subject = ArchiveAtomSubjectV1::try_new(
        kind,
        data["subject_id"].as_str().expect("subject id").to_owned(),
    )
    .expect("valid atom subject");
    let evidence_class = match data["evidence_class"].as_str().expect("evidence class") {
        "Observed" => ArchiveEvidenceClassV1::Observed,
        "Derived" => ArchiveEvidenceClassV1::Derived,
        "Calibrated" => ArchiveEvidenceClassV1::Calibrated,
        "Designed" => ArchiveEvidenceClassV1::Designed,
        other => panic!("unknown evidence class {other}"),
    };
    let value = match data["value"]["kind"].as_str().expect("value kind") {
        "text" => ArchiveAtomValueV1::Text(
            data["value"]["text"]
                .as_str()
                .expect("text value")
                .to_owned(),
        ),
        "f64" => {
            let bits =
                u64::from_str_radix(data["value"]["bits_hex"].as_str().expect("bits hex"), 16)
                    .expect("f64 bits");
            ArchiveAtomValueV1::F64(f64::from_bits(bits))
        }
        "u64" => ArchiveAtomValueV1::U64(data["value"]["number"].as_u64().expect("u64 value")),
        "bool" => ArchiveAtomValueV1::Bool(data["value"]["flag"].as_bool().expect("bool value")),
        other => panic!("unknown value kind {other}"),
    };
    ArchiveAtomV1::try_new(
        campaign_id(),
        subject,
        data["signal_key"].as_str().expect("signal key").to_owned(),
        data["grant_key"].as_str().expect("grant key").to_owned(),
        evidence_class,
        &value,
        ArchiveCitationV1::try_new(
            data["citation"]["source_id"]
                .as_str()
                .expect("citation source")
                .to_owned(),
            data["citation"]["locator"]
                .as_str()
                .expect("citation locator")
                .to_owned(),
        )
        .expect("valid citation"),
        data["valid_tick"].as_u64().expect("valid tick"),
    )
    .expect("vector atom mints")
}

#[allow(clippy::too_many_arguments)]
fn atom_json(
    id: &str,
    subject_kind: &str,
    subject_id: &str,
    signal_key: &str,
    evidence_class: &str,
    value: &Value,
    source_id: &str,
    locator: &str,
    valid_tick: u64,
) -> Value {
    json!({
        "id": id,
        "kind": "encoding",
        "data": {
            "campaign_id_uuid": CAMPAIGN_UUID,
            "subject_kind": subject_kind,
            "subject_id": subject_id,
            "signal_key": signal_key,
            "grant_key": signal_key,
            "evidence_class": evidence_class,
            "value": value.clone(),
            "citation": {"source_id": source_id, "locator": locator},
            "valid_tick": valid_tick,
            "atom_id_hex": hex_encode(&atom_from_data(&json!({
                "campaign_id_uuid": CAMPAIGN_UUID,
                "subject_kind": subject_kind,
                "subject_id": subject_id,
                "signal_key": signal_key,
                "grant_key": signal_key,
                "evidence_class": evidence_class,
                "value": value.clone(),
                "citation": {"source_id": source_id, "locator": locator},
                "valid_tick": valid_tick,
            })).atom_id()),
        },
    })
}

fn text_value(text: &str) -> Value {
    json!({"kind": "text", "text": text})
}

fn f64_value(bits_hex: &str) -> Value {
    json!({"kind": "f64", "bits_hex": bits_hex})
}

fn visibility_json(id: &str, granted_tick: Option<u64>, horizon_tick: u64, visible: bool) -> Value {
    let mut data = json!({
        "atom": {
            "campaign_id_uuid": CAMPAIGN_UUID,
            "subject_kind": "county",
            "subject_id": "26001",
            "signal_key": "subject",
            "grant_key": "subject",
            "evidence_class": "Observed",
            "value": text_value("Monroe County"),
            "citation": {
                "source_id": "census-county-authority-v1",
                "locator": "dim_county#county_fips=26001",
            },
            "valid_tick": 42,
        },
        "horizon_tick": horizon_tick,
        "expected_visible": visible,
    });
    match granted_tick {
        Some(tick) => data["granted_tick"] = json!(tick),
        None => data["granted_tick"] = Value::Null,
    }
    json!({"id": id, "kind": "visibility", "data": data})
}

fn refusal_json(id: &str, bits_hex: &str) -> Value {
    json!({
        "id": id,
        "kind": "refusal",
        "data": {
            "operation": "mint",
            "expected_code": "non_finite_value",
            "campaign_id_uuid": CAMPAIGN_UUID,
            "subject_kind": "place",
            "subject_id": "2622000",
            "signal_key": "median-wage",
            "grant_key": "median-wage",
            "evidence_class": "Derived",
            "value": f64_value(bits_hex),
            "citation": {
                "source_id": "qcew-2024",
                "locator": "annual.by_area#geocode=2622000",
            },
            "valid_tick": 42,
        },
    })
}

fn tag_map(tags: &[(&str, u8)]) -> Value {
    let mut map = serde_json::Map::new();
    for (name, tag) in tags {
        map.insert((*name).to_owned(), json!(tag));
    }
    Value::Object(map)
}

fn generate_vectors() -> String {
    let mut lines = Vec::new();
    lines.push(json!({
        "id": "identity-domain-and-layouts",
        "kind": "identity",
        "data": {
            "source_path": "rust/crates/babylon-persistence/src/archive.rs",
            "schema_path": "rust/crates/babylon-persistence/migrations/archive_atom_v1.sql",
            "atom_domain_ascii_nul": "babylon.semantic-archive-atom.v1",
            "atom_schema_contract_id": "babylon.archive-atom-schema.v1",
            "kind_tags": tag_map(&KIND_TAGS),
            "evidence_tags": tag_map(&EVIDENCE_TAGS),
            "value_tags": tag_map(&VALUE_TAGS),
        },
    }));
    lines.push(atom_json(
        "encoding-county-subject-text-observed",
        "county",
        "26001",
        "subject",
        "Observed",
        &text_value("Monroe County"),
        "census-county-authority-v1",
        "dim_county#county_fips=26001",
        42,
    ));
    lines.push(atom_json(
        "encoding-place-f64-derived",
        "place",
        "2622000",
        "median-wage",
        "Derived",
        &f64_value("4029000000000000"),
        "qcew-2024",
        "annual.by_area#geocode=2622000",
        42,
    ));
    lines.push(atom_json(
        "encoding-concept-u64-designed",
        "concept",
        "containment",
        "subject",
        "Designed",
        &json!({"kind": "u64", "number": 83}),
        "glossary-concepts-v1",
        "contracts/fixtures/glossary_concepts_v1.jsonl#concept_id=containment",
        0,
    ));
    lines.push(atom_json(
        "encoding-concept-bool-designed",
        "concept",
        "identity",
        "identity",
        "Designed",
        &json!({"kind": "bool", "flag": true}),
        "glossary-concepts-v1",
        "contracts/fixtures/glossary_concepts_v1.jsonl#concept_id=identity",
        0,
    ));
    lines.push(atom_json(
        "encoding-f64-neg-zero-canonical",
        "place",
        "2622000",
        "median-wage",
        "Derived",
        &f64_value("8000000000000000"),
        "qcew-2024",
        "annual.by_area#geocode=2622000",
        42,
    ));
    lines.push(refusal_json("refuse-f64-pos-inf", "7ff0000000000000"));
    lines.push(refusal_json("refuse-f64-neg-inf", "fff0000000000000"));
    lines.push(refusal_json("refuse-f64-nan", "7ff8000000000000"));
    lines.push(visibility_json(
        "visibility-granted-in-horizon",
        Some(5),
        42,
        true,
    ));
    lines.push(visibility_json(
        "visibility-granted-tick-after-valid",
        Some(43),
        100,
        false,
    ));
    lines.push(visibility_json("visibility-no-grant-row", None, 100, false));
    lines.push(visibility_json(
        "visibility-past-horizon",
        Some(5),
        41,
        false,
    ));
    let mut output = String::new();
    for line in lines {
        let reserialized: Value = serde_json::from_str(&line.to_string()).expect("resort keys");
        output.push_str(&serde_json::to_string(&reserialized).expect("vector row serializes"));
        output.push('\n');
    }
    output
}

#[test]
#[ignore = "regeneration writes contracts/archive_atom_v1_vectors.jsonl from the pinned atom encoder"]
fn generate_atom_vectors_from_the_pinned_encoder() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../contracts/archive_atom_v1_vectors.jsonl"
    );
    std::fs::write(path, generate_vectors()).expect("atom vector corpus written");
}

#[test]
fn shared_atom_vectors_match_the_pinned_atom_identity() {
    let rows = rows();
    let encoding = rows_of_kind(&rows, "encoding").collect::<Vec<_>>();
    assert_eq!(encoding.len(), 5, "one vector per encoding scenario");
    for row in encoding {
        let atom = atom_from_data(&row["data"]);
        assert_eq!(
            hex_encode(&atom.atom_id()),
            row["data"]["atom_id_hex"].as_str().expect("atom id hex"),
            "{}",
            row["id"]
        );
    }
    let refusals = rows_of_kind(&rows, "refusal").collect::<Vec<_>>();
    assert_eq!(refusals.len(), 3, "NaN and both infinities refuse");
    for row in refusals {
        let kind = row["data"]["value"]["kind"].as_str().expect("value kind");
        assert_eq!(kind, "f64");
        assert_eq!(
            row["data"]["expected_code"].as_str(),
            Some("non_finite_value"),
            "{}",
            row["id"]
        );
        let bits = u64::from_str_radix(
            row["data"]["value"]["bits_hex"].as_str().expect("bits hex"),
            16,
        )
        .expect("f64 bits");
        assert!(!f64::from_bits(bits).is_finite(), "{}", row["id"]);
        let atom = ArchiveAtomV1::try_new(
            campaign_id(),
            ArchiveAtomSubjectV1::try_new(
                ArchiveAtomSubjectKindV1::Place,
                row["data"]["subject_id"].as_str().expect("id").to_owned(),
            )
            .expect("subject"),
            row["data"]["signal_key"].as_str().expect("key").to_owned(),
            row["data"]["grant_key"].as_str().expect("key").to_owned(),
            ArchiveEvidenceClassV1::Derived,
            &ArchiveAtomValueV1::F64(f64::from_bits(bits)),
            ArchiveCitationV1::try_new(
                row["data"]["citation"]["source_id"]
                    .as_str()
                    .expect("source")
                    .to_owned(),
                row["data"]["citation"]["locator"]
                    .as_str()
                    .expect("locator")
                    .to_owned(),
            )
            .expect("citation"),
            row["data"]["valid_tick"].as_u64().expect("tick"),
        );
        assert_eq!(
            atom,
            Err(SemanticArchiveErrorV1::NonFiniteValue),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn shared_atom_vectors_cover_neg_zero_canonicalization() {
    let rows = rows();
    let neg_zero = rows_of_kind(&rows, "encoding")
        .find(|row| row["id"] == "encoding-f64-neg-zero-canonical")
        .expect("neg zero vector");
    let neg_atom = atom_from_data(&neg_zero["data"]);
    let mut pos_data = neg_zero["data"].clone();
    pos_data["value"] = f64_value("0000000000000000");
    let pos_atom = atom_from_data(&pos_data);
    assert_eq!(neg_atom.atom_id(), pos_atom.atom_id());
    assert_eq!(pos_atom.value(), &ArchiveAtomValueV1::F64(0.0));
}

#[test]
fn shared_visibility_vectors_match_the_pure_fog_predicate() {
    let rows = rows();
    let visibility = rows_of_kind(&rows, "visibility").collect::<Vec<_>>();
    assert_eq!(visibility.len(), 4, "four visibility boundary cases");
    for row in visibility {
        let atom = atom_from_data(&row["data"]["atom"]);
        let granted_tick = row["data"]["granted_tick"].as_u64();
        let horizon_tick = row["data"]["horizon_tick"].as_u64().expect("horizon");
        let expected = row["data"]["expected_visible"].as_bool().expect("visible");
        assert_eq!(
            archive_atom_visible_v1(&atom, granted_tick, horizon_tick),
            expected,
            "{}",
            row["id"]
        );
    }
}

#[test]
fn shared_identity_vectors_match_the_pinned_domain_and_tags() {
    let rows = rows();
    let identity = rows_of_kind(&rows, "identity").collect::<Vec<_>>();
    assert_eq!(identity.len(), 1);
    let data = &identity[0]["data"];
    assert_eq!(
        data["atom_domain_ascii_nul"].as_str(),
        Some("babylon.semantic-archive-atom.v1")
    );
    assert_eq!(
        data["atom_schema_contract_id"].as_str(),
        Some("babylon.archive-atom-schema.v1")
    );
    let schema_bytes = include_bytes!("../migrations/archive_atom_v1.sql");
    assert!(sha256_hex(schema_bytes).len() == 64);
    let schema = std::str::from_utf8(schema_bytes).expect("schema utf8");
    assert!(schema.contains("babylon.archive-atom-schema.v1"));
    assert!(schema.contains("value_f64 = value_f64"));
    assert!(schema.contains("abs(value_f64) <> 'Infinity'::float8"));
}
