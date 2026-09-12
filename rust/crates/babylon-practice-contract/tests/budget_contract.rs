use babylon_practice_contract::{
    budget_delta_digest, decode_budget_delta, encode_budget_delta, practice_machine_verb,
    OrganizationBudgetDelta, PracticeContractError, PracticeId, VerbMode, VerbStem,
    PRACTICE_BUDGET_TOPOLOGY_SOURCE_SHA256,
};
use serde_json::Value;

const SCHEMA: &[u8] = include_bytes!("../../../../contracts/practice_budget_topology.yaml");
const VECTORS: &str = include_str!("../../../../contracts/practice_budget_topology_vectors.jsonl");

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn retained_budget_bytes_and_refusals_match_the_independent_contract() {
    assert_eq!(
        babylon_kernel::content_digest::sha256_of(SCHEMA),
        PRACTICE_BUDGET_TOPOLOGY_SOURCE_SHA256
    );
    let delta = OrganizationBudgetDelta {
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
    };
    let cases: Vec<Value> = VECTORS
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let case = cases
        .iter()
        .find(|case| case["case_id"] == "budget-delta")
        .unwrap();
    let bytes = hex_bytes(case["data"]["canonical_hex"].as_str().unwrap());
    assert_eq!(encode_budget_delta(&delta).unwrap(), bytes);
    assert_eq!(decode_budget_delta(&bytes).unwrap(), delta);
    assert_eq!(
        budget_delta_digest(&delta).unwrap().as_slice(),
        hex_bytes(case["data"]["digest_hex"].as_str().unwrap())
    );
    for end in 0..bytes.len() {
        assert!(decode_budget_delta(&bytes[..end]).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        decode_budget_delta(&trailing),
        Err(PracticeContractError::PracticeTrailingBytes)
    );
    let case = cases
        .iter()
        .find(|case| case["case_id"] == "budget-boolean")
        .unwrap();
    assert_eq!(
        decode_budget_delta(&hex_bytes(case["data"]["payload_hex"].as_str().unwrap())),
        Err(PracticeContractError::PracticeBoolean)
    );
}

#[test]
fn machine_verbs_exist_only_for_the_declared_material_mappings() {
    let organize = practice_machine_verb(PracticeId::Organize).unwrap();
    let agitate = practice_machine_verb(PracticeId::Agitate).unwrap();
    let aid = practice_machine_verb(PracticeId::MutualAid).unwrap();
    assert_eq!(
        (organize.stem, organize.mode),
        (VerbStem::Mobilize, Some(VerbMode::Canvass))
    );
    assert_eq!(
        (agitate.stem, agitate.mode),
        (VerbStem::Mobilize, Some(VerbMode::Agitate))
    );
    assert_eq!((aid.stem, aid.mode), (VerbStem::Aid, None));
    for practice in [
        PracticeId::Strike,
        PracticeId::Blockade,
        PracticeId::Occupation,
        PracticeId::Damage,
        PracticeId::CapitalStrike,
    ] {
        assert_eq!(practice_machine_verb(practice), None);
    }
}
