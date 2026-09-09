use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    decode_intent, decode_practice_intent_v2, encode_intent, encode_practice_intent_v2,
    practice_intent_v2_digest, InputAuthorityIdV2, PracticeContractError, PracticeIdV1,
    PracticeIdV2, PracticeIntentV1, PracticeIntentV2, PracticeIntentV2Error,
    PracticeTargetDomainV1, PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2,
    TaggedPracticeTargetV2, PRACTICE_INTENT_V2_SOURCE_SHA256,
};
use serde_json::Value;

const SCHEMA: &[u8] = include_bytes!("../../../../contracts/practice_intent_v2.yaml");
const VECTORS: &str = include_str!("../../../../contracts/practice_intent_v2_vectors.jsonl");

fn actor_id(value: u64) -> ActorOrganizationIdV2 {
    ActorOrganizationIdV2::from_bytes(value.to_be_bytes())
}

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

fn intent_from_vector(data: &Value) -> PracticeIntentV2 {
    PracticeIntentV2 {
        schema_version: 2,
        submit_after_tick: data["submit_after_tick"].as_u64().unwrap(),
        resolve_tick: data["resolve_tick"].as_u64().unwrap(),
        input_authority_id: InputAuthorityIdV2::from_bytes(
            hex_bytes(data["input_authority_id_hex"].as_str().unwrap())
                .try_into()
                .unwrap(),
        ),
        actor_org_id: actor_id(data["actor_org_id"].as_u64().unwrap()),
        practice_id: PracticeIdV2::try_from(
            u8::try_from(data["practice_id"].as_u64().unwrap()).unwrap(),
        )
        .unwrap(),
        target: TaggedPracticeTargetV2 {
            tag: PracticeTargetTagV2::try_from(
                u8::try_from(data["target_tag"].as_u64().unwrap()).unwrap(),
            )
            .unwrap(),
            identity: PracticeTargetIdentityV2::from_bytes(
                hex_bytes(data["target_identity_hex"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
            ),
        },
        proposal_nonce: ProposalNonceV2::from_bytes(
            hex_bytes(data["proposal_nonce_hex"].as_str().unwrap())
                .try_into()
                .unwrap(),
        ),
        quoted_content_digest: hex_bytes(data["quoted_content_digest_hex"].as_str().unwrap())
            .try_into()
            .unwrap(),
        quoted_resource_contract_digest: hex_bytes(
            data["quoted_resource_contract_digest_hex"]
                .as_str()
                .unwrap(),
        )
        .try_into()
        .unwrap(),
        parameters: Vec::new(),
        evidence_digests: data["evidence_digests_hex"]
            .as_array()
            .unwrap()
            .iter()
            .take(65)
            .map(|digest| hex_bytes(digest.as_str().unwrap()).try_into().unwrap())
            .collect(),
    }
}

#[test]
fn intent_v2_schema_and_vectors_drive_the_rust_boundary() {
    assert_eq!(
        babylon_kernel::sha256_of(SCHEMA),
        PRACTICE_INTENT_V2_SOURCE_SHA256
    );
    let cases: Vec<Value> = VECTORS
        .lines()
        .take(65)
        .map(|line| {
            assert!(line.len() <= 4_096);
            serde_json::from_str(line).unwrap()
        })
        .collect();
    assert_eq!(cases.len(), 15);

    let intent_case = cases
        .iter()
        .find(|case| case["case_id"] == "intent-strike")
        .unwrap();
    let intent = intent_from_vector(&intent_case["data"]);
    let canonical = hex_bytes(intent_case["data"]["canonical_hex"].as_str().unwrap());
    assert_eq!(canonical.len(), 251);
    assert_eq!(encode_practice_intent_v2(&intent).unwrap(), canonical);
    assert_eq!(decode_practice_intent_v2(&canonical).unwrap(), intent);
    assert_eq!(
        practice_intent_v2_digest(&intent).unwrap().to_vec(),
        hex_bytes(intent_case["data"]["digest_hex"].as_str().unwrap())
    );

    for case in cases.iter().filter(|case| case["kind"] == "invalid_wire") {
        let payload = hex_bytes(case["data"]["payload_hex"].as_str().unwrap());
        let expected = PracticeIntentV2Error::try_from(
            u16::try_from(case["data"]["error"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            decode_practice_intent_v2(&payload).map(|_| ()),
            Err(expected)
        );
    }
}

#[test]
fn v1_and_v2_intent_domains_refuse_cross_version_decoding() {
    let v1 = PracticeIntentV1 {
        schema_version: 1,
        submit_after_tick: 10,
        resolve_tick: 11,
        actor_org_id: 7,
        practice_id: PracticeIdV1::Organize,
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_node_id: 8,
        quoted_content_digest: [0x30; 32],
        quoted_action_budget_cost: 1,
        parameters: Vec::new(),
        evidence_digests: Vec::new(),
    };
    let v2 = intent_from_vector(
        &VECTORS
            .lines()
            .take(65)
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .find(|case| case["case_id"] == "intent-strike")
            .unwrap()["data"],
    );

    assert_eq!(
        decode_practice_intent_v2(&encode_intent(&v1).unwrap()),
        Err(PracticeIntentV2Error::IntentDomain)
    );
    assert_eq!(
        decode_intent(&encode_practice_intent_v2(&v2).unwrap()),
        Err(PracticeContractError::PracticeDomain)
    );
}
