use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::tick_content_hash::TickContentHashV1;
use babylon_persistence::identity::CampaignId;
use babylon_persistence::tick_commit_claim::{
    TickCommitClaimConflictV1, TickCommitClaimRetryV1, TickCommitClaimV1,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

const VECTORS: &str = include_str!("../../../../contracts/tick_commit_claim_v1_vectors.jsonl");
const MAX_ROWS: usize = 32;
const MAX_LINE_BYTES: usize = 4_096;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorRow {
    id: String,
    kind: String,
    data: Value,
}

fn rows() -> Vec<VectorRow> {
    let input = VECTORS.strip_suffix('\n').unwrap_or(VECTORS);
    let mut rows = Vec::with_capacity(MAX_ROWS);
    for (index, line) in input.split('\n').take(MAX_ROWS + 1).enumerate() {
        assert!(index < MAX_ROWS, "bounded vector row count");
        assert!(!line.is_empty() && line.len() <= MAX_LINE_BYTES);
        rows.push(serde_json::from_str(line).expect("valid bounded vector row"));
    }
    rows
}

fn text<'a>(data: &'a Value, field: &str) -> &'a str {
    data[field].as_str().expect("text field")
}

fn hex32(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    let mut bytes = [0_u8; 32];
    for (index, output) in bytes.iter_mut().take(32).enumerate() {
        let offset = index * 2;
        *output = u8::from_str_radix(&value[offset..offset + 2], 16).expect("hex byte");
    }
    bytes
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert!(value.len() <= MAX_LINE_BYTES && value.len().is_multiple_of(2));
    value
        .as_bytes()
        .chunks_exact(2)
        .take(MAX_LINE_BYTES / 2)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).expect("ASCII hex");
            u8::from_str_radix(text, 16).expect("hex byte")
        })
        .collect()
}

fn claim(data: &Value) -> TickCommitClaimV1 {
    let campaign = CampaignId::from_uuid(
        Uuid::parse_str(text(data, "campaign_id")).expect("canonical campaign UUID"),
    );
    let resolve_tick = data["resolve_tick"].as_u64().expect("u64 resolve tick");
    let content = TickContentHashV1::from_bytes(hex32(text(data, "tick_content_hash_hex")));
    TickCommitClaimV1::compose(campaign, resolve_tick, content)
}

fn claims(rows: &[VectorRow]) -> BTreeMap<&str, TickCommitClaimV1> {
    rows.iter()
        .take(MAX_ROWS)
        .filter(|row| row.kind == "claim")
        .map(|row| (row.id.as_str(), claim(&row.data)))
        .collect()
}

#[test]
fn production_claim_reconstructs_every_shared_vector() {
    let rows = rows();
    let families: BTreeSet<&str> = rows
        .iter()
        .take(MAX_ROWS)
        .map(|row| row.kind.as_str())
        .collect();
    assert_eq!(
        families,
        BTreeSet::from(["claim", "mutation", "refusal", "retry"])
    );
    for row in rows.iter().take(MAX_ROWS).filter(|row| row.kind == "claim") {
        let actual = claim(&row.data);
        assert_eq!(
            actual.canonical_bytes().as_slice(),
            hex_bytes(text(&row.data, "canonical_hex")),
            "{}",
            row.id
        );
        assert_eq!(actual.canonical_bytes().len(), 93);
    }
}

#[test]
fn every_semantic_field_mutation_moves_the_production_claim() {
    let rows = rows();
    let claims = claims(&rows);
    for row in rows
        .iter()
        .take(MAX_ROWS)
        .filter(|row| row.kind == "mutation")
    {
        let base = claims[text(&row.data, "base_id")];
        let mutated = claims[text(&row.data, "mutated_id")];
        assert_ne!(
            base.canonical_bytes(),
            mutated.canonical_bytes(),
            "{}",
            row.id
        );
    }
}

#[test]
fn production_retry_classification_matches_the_shared_contract() {
    let rows = rows();
    let claims = claims(&rows);
    for row in rows.iter().take(MAX_ROWS).filter(|row| row.kind == "retry") {
        let requested = claims[text(&row.data, "requested_id")];
        let existing = claims[text(&row.data, "existing_id")];
        let result = requested.classify_retry_against(&existing);
        match text(&row.data, "expected") {
            "idempotent" => assert_eq!(result, Ok(TickCommitClaimRetryV1::Idempotent)),
            "key_mismatch" => assert!(matches!(
                result,
                Err(TickCommitClaimConflictV1::KeyMismatch { .. })
            )),
            "content_identity_mismatch" => assert!(matches!(
                result,
                Err(TickCommitClaimConflictV1::ContentIdentityMismatch { .. })
            )),
            unexpected => panic!("unknown expected retry result: {unexpected}"),
        }
    }
}
