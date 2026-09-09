use babylon_practice_contract::{
    budget_delta_digest, decode_budget_delta, decode_input_authority, decode_intent,
    decode_rejection, encode_budget_delta, encode_input_authority, encode_intent,
    encode_intent_parameters, encode_rejection, input_authority_digest, intent_digest,
    parameter_bytes_digest, rejection_for, submission_rejection_alias,
    target_selection_policy_digest, OrganizationBudgetDeltaV1, PracticeAuthorityKindV1,
    PracticeContractError, PracticeIdV1, PracticeInputAuthorityV1, PracticeIntentV1,
    PracticeParameterV1, PracticeRejectionCodeV1, PracticeSubmissionRejectionV1,
    PracticeTargetDomainV1,
};
use serde_json::{Map, Value};

const CORPUS: &str = include_str!("../../../../contracts/practice_contract_v1_vectors.jsonl");
const MAX_SOURCE_BYTES: usize = 2_097_152;
const MAX_CASES: usize = 512;
const MAX_LINE_BYTES: usize = 65_536;
const MAX_CASE_ID_BYTES: usize = 128;
const MAX_DEPTH: usize = 32;
const KINDS: [&str; 10] = [
    "manifest",
    "authority",
    "intent",
    "budget_delta",
    "rejection",
    "invalid_wire",
    "authority_validation",
    "quote_validation",
    "batch_recipe",
    "organization-practice-prelude",
];

enum JsonFrame {
    Object(Vec<String>),
    Array,
}

fn authority() -> PracticeInputAuthorityV1 {
    PracticeInputAuthorityV1 {
        schema_version: 1,
        authority_kind: PracticeAuthorityKindV1::PlayerSeat,
        actor_org_id: 7,
        producer_content_digest: [0x11; 32],
    }
}

fn intent() -> PracticeIntentV1 {
    PracticeIntentV1 {
        schema_version: 1,
        submit_after_tick: 10,
        resolve_tick: 11,
        actor_org_id: 7,
        practice_id: PracticeIdV1::Organize,
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_node_id: 101,
        quoted_content_digest: [0x22; 32],
        quoted_action_budget_cost: 1,
        parameters: Vec::new(),
        evidence_digests: Vec::new(),
    }
}

fn budget_delta() -> OrganizationBudgetDeltaV1 {
    OrganizationBudgetDeltaV1 {
        schema_version: 1,
        tick: 11,
        actor_node_id: 7,
        pre_action_world_hash: [0x33; 32],
        budget_before: 1,
        governed_cost: 1,
        footprint_count: 2,
        raw_credit: 2,
        credited_credit: 1,
        ceiling_bound: false,
        budget_after: 1,
    }
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    let mut output = Vec::with_capacity(value.len() / 2);
    for chunk in value.as_bytes().chunks_exact(2).take(32_769) {
        let text = std::str::from_utf8(chunk).unwrap();
        output.push(u8::from_str_radix(text, 16).unwrap());
    }
    output
}

fn hex_digest(value: &str) -> [u8; 32] {
    hex_bytes(value).try_into().unwrap()
}

fn expected_data_fields(kind: &str) -> &'static [&'static str] {
    match kind {
        "manifest" => &["parameter_limit_valid_witness", "intent_truncation_offsets"],
        "authority" => &[
            "authority_kind",
            "actor_org_id",
            "producer_content_digest_hex",
            "canonical_hex",
            "digest_hex",
        ],
        "intent" => &[
            "practice_id",
            "actor_org_id",
            "target_node_id",
            "quoted_content_digest_hex",
            "quoted_action_budget_cost",
            "evidence_digests_hex",
            "canonical_hex",
            "digest_hex",
            "parameter_hex",
            "parameter_digest_hex",
            "target_preimage_hex",
            "target_digest_hex",
        ],
        "budget_delta" => &["canonical_hex", "digest_hex"],
        "rejection" => &["reason_code", "canonical_hex"],
        "invalid_wire" => &["codec", "payload_hex", "error"],
        "authority_validation" | "quote_validation" => &["recipe", "error"],
        "batch_recipe" => &["count", "recipe", "error"],
        "organization-practice-prelude" => &["raw_hex", "digest_hex"],
        _ => &[],
    }
}

fn scan_depth(line: &[u8]) -> Result<(), &'static str> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    for byte in line.iter().take(MAX_LINE_BYTES + 1) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
        } else if *byte == b'"' {
            in_string = true;
        } else if matches!(*byte, b'{' | b'[') {
            depth += 1;
            if depth > MAX_DEPTH {
                return Err("depth");
            }
        } else if matches!(*byte, b'}' | b']') {
            depth = depth.checked_sub(1).ok_or("json")?;
        }
    }
    Ok(())
}

fn json_string_end(line: &[u8], start: usize) -> Result<usize, &'static str> {
    let mut escaped = false;
    for offset in 1..=MAX_LINE_BYTES + 1 {
        let index = start.checked_add(offset).ok_or("json")?;
        let byte = *line.get(index).ok_or("json")?;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return index.checked_add(1).ok_or("json");
        }
    }
    Err("json")
}

fn next_non_whitespace(line: &[u8], start: usize) -> Option<u8> {
    for offset in 0..=MAX_LINE_BYTES {
        let index = start.checked_add(offset)?;
        let byte = *line.get(index)?;
        if !byte.is_ascii_whitespace() {
            return Some(byte);
        }
    }
    None
}

fn reject_duplicate_json_keys(line: &[u8]) -> Result<(), &'static str> {
    let mut frames: Vec<JsonFrame> = Vec::new();
    let mut skip_until = 0_usize;
    for index in 0..=MAX_LINE_BYTES {
        if index == line.len() {
            break;
        }
        if index < skip_until {
            continue;
        }
        match line[index] {
            b'{' => frames.push(JsonFrame::Object(Vec::new())),
            b'[' => frames.push(JsonFrame::Array),
            b'}' | b']' => {
                frames.pop();
            }
            b'"' => {
                let end = json_string_end(line, index)?;
                if next_non_whitespace(line, end) == Some(b':') {
                    let key: String =
                        serde_json::from_slice(&line[index..end]).map_err(|_| "json")?;
                    let Some(JsonFrame::Object(keys)) = frames.last_mut() else {
                        return Err("json");
                    };
                    if keys
                        .iter()
                        .take(MAX_LINE_BYTES + 1)
                        .any(|existing| existing == &key)
                    {
                        return Err("duplicate key");
                    }
                    keys.push(key);
                }
                skip_until = end;
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_test_corpus(payload: &[u8]) -> Result<Vec<Map<String, Value>>, &'static str> {
    if payload.len() > MAX_SOURCE_BYTES {
        return Err("source");
    }
    let mut output = Vec::new();
    let mut case_ids: Vec<String> = Vec::new();
    for (index, raw_line) in payload
        .split_inclusive(|byte| *byte == b'\n')
        .take(MAX_CASES + 1)
        .enumerate()
    {
        if index == MAX_CASES || raw_line.len() > MAX_LINE_BYTES {
            return Err("line");
        }
        let line = raw_line
            .strip_suffix(b"\n")
            .unwrap_or(raw_line)
            .strip_suffix(b"\r")
            .unwrap_or(raw_line.strip_suffix(b"\n").unwrap_or(raw_line));
        scan_depth(line)?;
        reject_duplicate_json_keys(line)?;
        let value: Value = serde_json::from_slice(line).map_err(|_| "json")?;
        let object = value.as_object().ok_or("object")?;
        if object.len() != 3
            || !object.contains_key("case_id")
            || !object.contains_key("kind")
            || !object.contains_key("data")
        {
            return Err("field");
        }
        let case_id = object["case_id"].as_str().ok_or("case")?;
        if case_id.is_empty() || case_id.len() > MAX_CASE_ID_BYTES {
            return Err("case");
        }
        if case_ids
            .iter()
            .take(MAX_CASES + 1)
            .any(|item| item == case_id)
        {
            return Err("duplicate");
        }
        let kind = object["kind"].as_str().ok_or("kind")?;
        if !KINDS.contains(&kind) {
            return Err("kind");
        }
        let data = object["data"].as_object().ok_or("data")?;
        let fields = expected_data_fields(kind);
        if data.len() != fields.len()
            || !data
                .keys()
                .take(MAX_LINE_BYTES + 1)
                .all(|key| fields.contains(&key.as_str()))
        {
            return Err("data field");
        }
        case_ids.push(case_id.to_owned());
        output.push(object.clone());
    }
    Ok(output)
}

fn corpus_cases(kind: &str) -> Vec<Map<String, Value>> {
    parse_test_corpus(CORPUS.as_bytes())
        .unwrap()
        .into_iter()
        .take(MAX_CASES + 1)
        .filter(|object| object.get("kind").and_then(Value::as_str) == Some(kind))
        .collect()
}

fn data(case: &Map<String, Value>) -> &Map<String, Value> {
    case.get("data").and_then(Value::as_object).unwrap()
}

#[test]
fn shared_root_corpus_and_fixed_records_round_trip() {
    let authority = authority();
    let authority_bytes = encode_input_authority(&authority).unwrap();
    assert_eq!(decode_input_authority(&authority_bytes).unwrap(), authority);
    let intent = intent();
    let intent_bytes = encode_intent(&intent).unwrap();
    assert_eq!(decode_intent(&intent_bytes).unwrap(), intent);
    let delta = budget_delta();
    let delta_bytes = encode_budget_delta(&delta).unwrap();
    assert_eq!(decode_budget_delta(&delta_bytes).unwrap(), delta);
    let rejection = rejection_for(
        [0x44; 32],
        PracticeRejectionCodeV1::PracticeUnwired,
        10,
        [0x22; 32],
    );
    let rejection_bytes = encode_rejection(&rejection).unwrap();
    assert_eq!(decode_rejection(&rejection_bytes).unwrap(), rejection);
}

#[test]
fn organization_practice_prelude_vector_pins_raw_bytes_and_digest() {
    let cases = corpus_cases("organization-practice-prelude");
    assert_eq!(cases.len(), 1);
    let item = data(&cases[0]);
    let raw = hex_bytes(item["raw_hex"].as_str().unwrap());
    let prelude =
        include_bytes!("../../babylon-tick/content/declarations/organization-practice.bscn");
    assert_eq!(raw, prelude);
    assert_eq!(
        babylon_kernel::sha256_of(&raw),
        hex_digest(item["digest_hex"].as_str().unwrap())
    );
}

#[test]
fn digest_and_alias_interfaces_are_exactly_typed() {
    assert_eq!(input_authority_digest(&authority()).unwrap().len(), 32);
    assert_eq!(encode_intent_parameters(&intent()).unwrap(), [0, 0]);
    assert_eq!(intent_digest(&intent()).unwrap().len(), 32);
    assert_eq!(parameter_bytes_digest(&intent()).unwrap().len(), 32);
    assert_eq!(
        target_selection_policy_digest(PracticeTargetDomainV1::SocialClass, 101).len(),
        32
    );
    assert_eq!(budget_delta_digest(&budget_delta()).unwrap().len(), 32);
    assert_eq!(
        submission_rejection_alias(PracticeContractError::PracticeTickOverflow),
        Some(PracticeRejectionCodeV1::PracticeTickMismatch)
    );
    assert_eq!(
        submission_rejection_alias(PracticeContractError::PracticeDomain),
        None
    );
    let _: PracticeSubmissionRejectionV1 = rejection_for(
        [0x44; 32],
        PracticeRejectionCodeV1::PracticeStaleContent,
        10,
        [0x22; 32],
    );
}

#[test]
fn shared_vectors_pin_authority_intent_budget_and_all_rejections() {
    for case in corpus_cases("authority").iter().take(3) {
        let item = data(case);
        let value = PracticeInputAuthorityV1 {
            schema_version: 1,
            authority_kind: PracticeAuthorityKindV1::try_from(
                u8::try_from(item["authority_kind"].as_u64().unwrap()).unwrap(),
            )
            .unwrap(),
            actor_org_id: item["actor_org_id"].as_u64().unwrap(),
            producer_content_digest: hex_digest(
                item["producer_content_digest_hex"].as_str().unwrap(),
            ),
        };
        let canonical = hex_bytes(item["canonical_hex"].as_str().unwrap());
        assert_eq!(encode_input_authority(&value).unwrap(), canonical);
        assert_eq!(decode_input_authority(&canonical).unwrap(), value);
        assert_eq!(
            input_authority_digest(&value).unwrap(),
            hex_digest(item["digest_hex"].as_str().unwrap())
        );
    }
    for case in corpus_cases("intent").iter().take(5) {
        let item = data(case);
        let evidence = item["evidence_digests_hex"]
            .as_array()
            .unwrap()
            .iter()
            .take(65)
            .map(|digest| hex_digest(digest.as_str().unwrap()))
            .collect();
        let value = PracticeIntentV1 {
            schema_version: 1,
            submit_after_tick: 10,
            resolve_tick: 11,
            actor_org_id: item["actor_org_id"].as_u64().unwrap(),
            practice_id: PracticeIdV1::try_from(
                u8::try_from(item["practice_id"].as_u64().unwrap()).unwrap(),
            )
            .unwrap(),
            target_domain: PracticeTargetDomainV1::SocialClass,
            target_node_id: item["target_node_id"].as_u64().unwrap(),
            quoted_content_digest: hex_digest(item["quoted_content_digest_hex"].as_str().unwrap()),
            quoted_action_budget_cost: u32::try_from(
                item["quoted_action_budget_cost"].as_u64().unwrap(),
            )
            .unwrap(),
            parameters: Vec::new(),
            evidence_digests: evidence,
        };
        let canonical = hex_bytes(item["canonical_hex"].as_str().unwrap());
        assert_eq!(encode_intent(&value).unwrap(), canonical);
        assert_eq!(decode_intent(&canonical).unwrap(), value);
        assert_eq!(
            intent_digest(&value).unwrap(),
            hex_digest(item["digest_hex"].as_str().unwrap())
        );
        assert_eq!(encode_intent_parameters(&value).unwrap(), [0, 0]);
        assert_eq!(
            parameter_bytes_digest(&value).unwrap(),
            hex_digest(item["parameter_digest_hex"].as_str().unwrap())
        );
        assert_eq!(
            target_selection_policy_digest(value.target_domain, value.target_node_id),
            hex_digest(item["target_digest_hex"].as_str().unwrap())
        );
        let mut target_preimage = b"babylon.fixed-target-selection.v1\0".to_vec();
        target_preimage.push(value.target_domain as u8);
        target_preimage.extend_from_slice(&value.target_node_id.to_be_bytes());
        assert_eq!(
            target_preimage,
            hex_bytes(item["target_preimage_hex"].as_str().unwrap())
        );
        assert_eq!(
            babylon_kernel::sha256_of(&target_preimage),
            hex_digest(item["target_digest_hex"].as_str().unwrap())
        );
    }
    let budget_case = corpus_cases("budget_delta").remove(0);
    let budget_bytes = hex_bytes(data(&budget_case)["canonical_hex"].as_str().unwrap());
    assert_eq!(encode_budget_delta(&budget_delta()).unwrap(), budget_bytes);
    assert_eq!(decode_budget_delta(&budget_bytes).unwrap(), budget_delta());
    assert_eq!(
        budget_delta_digest(&budget_delta()).unwrap(),
        hex_digest(data(&budget_case)["digest_hex"].as_str().unwrap())
    );
    let rejection_cases = corpus_cases("rejection");
    assert_eq!(rejection_cases.len(), 11);
    for case in rejection_cases.iter().take(12) {
        let item = data(case);
        let reason = PracticeRejectionCodeV1::try_from(
            u16::try_from(item["reason_code"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        let value = rejection_for([0x44; 32], reason, 10, [0x22; 32]);
        let canonical = hex_bytes(item["canonical_hex"].as_str().unwrap());
        assert_eq!(encode_rejection(&value).unwrap(), canonical);
        assert_eq!(decode_rejection(&canonical).unwrap(), value);
    }
}

#[test]
fn invalid_wire_vectors_return_exact_errors() {
    for case in corpus_cases("invalid_wire").iter().take(64) {
        let item = data(case);
        let payload = hex_bytes(item["payload_hex"].as_str().unwrap());
        let expected = PracticeContractError::try_from(
            u16::try_from(item["error"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        let actual = match item["codec"].as_str().unwrap() {
            "authority" => decode_input_authority(&payload).map(|_| ()),
            "intent" => decode_intent(&payload).map(|_| ()),
            "budget_delta" => decode_budget_delta(&payload).map(|_| ()),
            "rejection" => decode_rejection(&payload).map(|_| ()),
            unknown => panic!("unknown vector codec {unknown}"),
        };
        assert_eq!(actual.unwrap_err(), expected);
    }
}

#[test]
fn truncation_oversize_and_alias_table_are_exact() {
    let canonical = encode_intent(&intent()).unwrap();
    for offset in [0, 27, 29, 37, 45, 53, 54, 55, 63, 95, 99, 101] {
        assert_eq!(
            decode_intent(&canonical[..offset]).unwrap_err(),
            PracticeContractError::PracticeTruncated
        );
    }
    let mut oversized = canonical.clone();
    oversized.resize(16_385, 0);
    assert_eq!(
        decode_intent(&oversized).unwrap_err(),
        PracticeContractError::PracticeLength
    );
    for error in [
        PracticeContractError::PracticeDomain,
        PracticeContractError::PracticeSchemaVersion,
        PracticeContractError::PracticeEnumCode,
        PracticeContractError::PracticeLength,
        PracticeContractError::PracticeTruncated,
        PracticeContractError::PracticeTrailingBytes,
        PracticeContractError::PracticeBoolean,
        PracticeContractError::PracticeParameter,
        PracticeContractError::PracticeParameterLimit,
        PracticeContractError::PracticeParameterLength,
        PracticeContractError::PracticeEvidenceLimit,
        PracticeContractError::PracticeEvidenceOrder,
        PracticeContractError::PracticeEvidenceDuplicate,
        PracticeContractError::PracticeTickOverflow,
        PracticeContractError::PracticeTickMismatch,
        PracticeContractError::PracticeAuthorityRegistryLimit,
        PracticeContractError::PracticeAuthorityRegistryOrder,
        PracticeContractError::PracticeAuthorityRegistryDuplicate,
        PracticeContractError::PracticeAuthorityUnregistered,
        PracticeContractError::PracticeActorMismatch,
        PracticeContractError::PracticeAuthorityContentMismatch,
        PracticeContractError::PracticeQuoteContentMismatch,
        PracticeContractError::PracticeQuoteCostMismatch,
        PracticeContractError::PracticeBatchLimit,
        PracticeContractError::PracticeDuplicateActor,
        PracticeContractError::PracticeBudgetNonfinite,
        PracticeContractError::PracticeBudgetNegative,
        PracticeContractError::PracticeBudgetFractional,
        PracticeContractError::PracticeBudgetRange,
        PracticeContractError::PracticeBudgetRoundtrip,
        PracticeContractError::PracticeBudgetInsufficient,
        PracticeContractError::PracticeBudgetArithmetic,
        PracticeContractError::PracticeFootprintLimit,
        PracticeContractError::PracticeFootprintOrder,
        PracticeContractError::PracticeFootprintDuplicate,
        PracticeContractError::PracticeFootprintSource,
        PracticeContractError::PracticeFootprintStrengthNonfinite,
        PracticeContractError::PracticeFootprintStrengthNonpositive,
        PracticeContractError::PracticeTopologyOrganizationLimit,
        PracticeContractError::PracticeTopologyOrganizationOrder,
        PracticeContractError::PracticeTopologyOrganizationDuplicate,
        PracticeContractError::PracticeTopologyBudgetMissing,
        PracticeContractError::PracticeTopologyEdgeOrder,
        PracticeContractError::PracticeTopologyEdgeDuplicate,
    ] {
        let expected = match error {
            PracticeContractError::PracticeTickOverflow
            | PracticeContractError::PracticeTickMismatch => {
                Some(PracticeRejectionCodeV1::PracticeTickMismatch)
            }
            PracticeContractError::PracticeAuthorityUnregistered
            | PracticeContractError::PracticeAuthorityContentMismatch => {
                Some(PracticeRejectionCodeV1::PracticeAuthorityUnregistered)
            }
            PracticeContractError::PracticeActorMismatch => {
                Some(PracticeRejectionCodeV1::PracticeActorMismatch)
            }
            PracticeContractError::PracticeQuoteContentMismatch => {
                Some(PracticeRejectionCodeV1::PracticeStaleContent)
            }
            PracticeContractError::PracticeQuoteCostMismatch => {
                Some(PracticeRejectionCodeV1::PracticeCostMismatch)
            }
            PracticeContractError::PracticeBatchLimit => {
                Some(PracticeRejectionCodeV1::PracticeBatchLimit)
            }
            PracticeContractError::PracticeDuplicateActor => {
                Some(PracticeRejectionCodeV1::PracticeDuplicateActor)
            }
            PracticeContractError::PracticeBudgetInsufficient => {
                Some(PracticeRejectionCodeV1::PracticeBudgetInsufficient)
            }
            _ => None,
        };
        assert_eq!(submission_rejection_alias(error), expected);
    }
}

#[test]
fn parameter_refusal_waits_for_every_structural_frame() {
    let valid = PracticeParameterV1 {
        key_u8: 1,
        value_kind_u8: 1,
        value_length_u16: 0,
        value_bytes: Vec::new(),
    };
    let malformed = PracticeParameterV1 {
        key_u8: 2,
        value_kind_u8: 1,
        value_length_u16: 2,
        value_bytes: vec![b'x'],
    };
    let mut value = intent();
    value.parameters = vec![valid, malformed];
    assert_eq!(
        encode_intent_parameters(&value),
        Err(PracticeContractError::PracticeParameterLength)
    );

    let canonical = encode_intent(&intent()).unwrap();
    let parameter_offset = canonical.len() - 4;
    let prefix = &canonical[..parameter_offset];
    let valid_frame = [1, 1, 0, 0];
    let mut malformed_payload = prefix.to_vec();
    malformed_payload.extend_from_slice(&[0, 2]);
    malformed_payload.extend_from_slice(&valid_frame);
    malformed_payload.extend_from_slice(&[2, 1, 1, 1]);
    malformed_payload.extend_from_slice(&vec![0; 257]);
    malformed_payload.extend_from_slice(&[0, 0]);
    assert_eq!(
        decode_intent(&malformed_payload),
        Err(PracticeContractError::PracticeParameterLength)
    );

    let mut truncated_payload = prefix.to_vec();
    truncated_payload.extend_from_slice(&[0, 2]);
    truncated_payload.extend_from_slice(&valid_frame);
    truncated_payload.extend_from_slice(&[2, 1, 0, 2, b'x']);
    assert_eq!(
        decode_intent(&truncated_payload),
        Err(PracticeContractError::PracticeTruncated)
    );

    let mut valid_payload = prefix.to_vec();
    valid_payload.extend_from_slice(&[0, 2]);
    valid_payload.extend_from_slice(&valid_frame);
    valid_payload.extend_from_slice(&[2, 1, 0, 0, 0, 0]);
    assert_eq!(
        decode_intent(&valid_payload),
        Err(PracticeContractError::PracticeParameter)
    );
}

#[test]
fn shared_corpus_reader_refuses_every_fixed_and_closed_bound() {
    let valid = br#"{"case_id":"a","kind":"manifest","data":{"parameter_limit_valid_witness":null,"intent_truncation_offsets":[]}}
"#;
    let mut too_many = Vec::new();
    for index in 0..=MAX_CASES {
        too_many.extend_from_slice(
            format!(
                "{{\"case_id\":\"a{index}\",\"kind\":\"manifest\",\"data\":{{\"parameter_limit_valid_witness\":null,\"intent_truncation_offsets\":[]}}}}\n"
            )
            .as_bytes(),
        );
    }
    let nested = format!(
        "{{\"case_id\":\"a\",\"kind\":\"manifest\",\"data\":{}}}",
        "[".repeat(33) + &"]".repeat(33)
    );
    let witnesses = [
        vec![b'x'; MAX_SOURCE_BYTES + 1],
        too_many,
        vec![b' '; MAX_LINE_BYTES + 1],
        format!(
            "{{\"case_id\":\"{}\",\"kind\":\"manifest\",\"data\":{{\"parameter_limit_valid_witness\":null,\"intent_truncation_offsets\":[]}}}}",
            "x".repeat(MAX_CASE_ID_BYTES + 1)
        )
        .into_bytes(),
        nested.into_bytes(),
        [valid.as_slice(), valid.as_slice()].concat(),
        br#"{"case_id":"a","kind":"unknown","data":{}}"#.to_vec(),
        br#"{"case_id":"a","kind":"manifest","data":{"parameter_limit_valid_witness":null,"intent_truncation_offsets":[],"extra":0}}"#.to_vec(),
        br#"{"case_id":"a","kind":"manifest","data":{"parameter_limit_valid_witness":null,"intent_truncation_offsets":[]}} {}"#.to_vec(),
        br#"{"case_id":"a","case_id":"b","kind":"manifest","data":{"parameter_limit_valid_witness":null,"intent_truncation_offsets":[]}}"#.to_vec(),
        br#"{"case_id":"a","kind":"manifest","data":{"parameter_limit_valid_witness":null,"parameter_limit_valid_witness":null,"intent_truncation_offsets":[]}}"#.to_vec(),
    ];
    for payload in &witnesses {
        assert!(parse_test_corpus(payload).is_err());
    }
}
