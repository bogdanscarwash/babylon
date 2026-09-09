use babylon_practice_contract::{
    decode_practice_resource_allocation_contract_v2, decode_practice_resource_capacity_v2,
    decode_practice_resource_request_v2, encode_practice_resource_allocation_contract_v2,
    encode_practice_resource_capacity_v2, encode_practice_resource_request_v2,
    practice_resource_allocation_contract_v2_digest, practice_resource_capacity_v2_digest,
    practice_resource_request_v2_digest, PracticeResourceV2Error,
    PRACTICE_RESOURCE_ALLOCATION_V2_SOURCE_SHA256,
};
use serde_json::Value;

const SCHEMA: &[u8] = include_bytes!("../../../../contracts/practice_resource_allocation_v2.yaml");
const VECTORS: &str =
    include_str!("../../../../contracts/practice_resource_allocation_v2_vectors.jsonl");

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).expect("hex fixture is ASCII");
            u8::from_str_radix(text, 16).expect("hex fixture is valid")
        })
        .collect()
}

fn cases() -> Vec<Value> {
    VECTORS
        .lines()
        .take(32)
        .map(|line| {
            assert!(line.len() <= 65_536);
            serde_json::from_str(line).unwrap()
        })
        .collect()
}

fn case<'a>(cases: &'a [Value], case_id: &str) -> &'a Value {
    cases
        .iter()
        .find(|value| value["case_id"] == case_id)
        .unwrap()
}

#[test]
fn language_neutral_resource_vectors_drive_every_rust_digest() {
    assert_eq!(
        babylon_kernel::sha256_of(SCHEMA),
        PRACTICE_RESOURCE_ALLOCATION_V2_SOURCE_SHA256
    );
    let cases = cases();
    assert_eq!(cases.len(), 11);

    let manifest = case(&cases, "manifest");
    assert_eq!(
        hex_bytes(manifest["data"]["schema_sha256"].as_str().unwrap()),
        PRACTICE_RESOURCE_ALLOCATION_V2_SOURCE_SHA256
    );
    assert_eq!(manifest["data"]["contract_bytes"], 66);
    assert_eq!(manifest["data"]["request_bytes"], 202);
    assert_eq!(manifest["data"]["capacity_bytes"], 122);
    assert_eq!(manifest["data"]["outcome_example_bytes"], 218);
    assert_eq!(manifest["data"]["max_requests_per_intent"], 16);
    assert_eq!(manifest["data"]["max_requests_total"], 65_536);
    assert_eq!(manifest["data"]["max_capacities_total"], 65_536);

    let contract_case = case(&cases, "allocation-contract");
    let contract_bytes = hex_bytes(contract_case["data"]["canonical_hex"].as_str().unwrap());
    let contract = decode_practice_resource_allocation_contract_v2(&contract_bytes).unwrap();
    assert_eq!(
        encode_practice_resource_allocation_contract_v2(&contract).unwrap(),
        contract_bytes
    );
    assert_eq!(
        practice_resource_allocation_contract_v2_digest(&contract)
            .unwrap()
            .to_vec(),
        hex_bytes(contract_case["data"]["digest_hex"].as_str().unwrap())
    );

    let request_case = case(&cases, "request-shared-90");
    let request_bytes = hex_bytes(request_case["data"]["canonical_hex"].as_str().unwrap());
    let request = decode_practice_resource_request_v2(&request_bytes).unwrap();
    assert_eq!(
        encode_practice_resource_request_v2(&request).unwrap(),
        request_bytes
    );
    assert_eq!(
        practice_resource_request_v2_digest(&request)
            .unwrap()
            .to_vec(),
        hex_bytes(request_case["data"]["digest_hex"].as_str().unwrap())
    );

    let capacity_case = case(&cases, "capacity-shared-50");
    let capacity_bytes = hex_bytes(capacity_case["data"]["canonical_hex"].as_str().unwrap());
    let capacity = decode_practice_resource_capacity_v2(&capacity_bytes).unwrap();
    assert_eq!(
        encode_practice_resource_capacity_v2(&capacity).unwrap(),
        capacity_bytes
    );
    assert_eq!(
        practice_resource_capacity_v2_digest(&capacity)
            .unwrap()
            .to_vec(),
        hex_bytes(capacity_case["data"]["digest_hex"].as_str().unwrap())
    );

    let outcome_case = case(&cases, "outcome-pro-rata");
    let outcome_bytes = hex_bytes(outcome_case["data"]["canonical_hex"].as_str().unwrap());
    assert_eq!(
        babylon_kernel::sha256_of(&outcome_bytes).to_vec(),
        hex_bytes(outcome_case["data"]["digest_hex"].as_str().unwrap())
    );
}

#[test]
fn language_neutral_invalid_contract_vectors_keep_exact_refusals() {
    for value in cases()
        .iter()
        .filter(|value| value["kind"] == "invalid_contract_wire")
    {
        let payload = hex_bytes(value["data"]["payload_hex"].as_str().unwrap());
        let code = u16::try_from(value["data"]["error"].as_u64().unwrap()).unwrap();
        let expected = PracticeResourceV2Error::try_from(code).unwrap();
        assert_eq!(
            decode_practice_resource_allocation_contract_v2(&payload),
            Err(expected),
            "{}",
            value["case_id"]
        );
    }
}
