use babylon_practice_contract::ActorOrganizationId;
use babylon_practice_contract::{
    decode_practice_intent, encode_practice_intent, fixed_practice_target_digest,
    practice_intent_digest, practice_parameter_bytes_digest, InputAuthorityId, PracticeId,
    PracticeIntent, PracticeIntentError, PracticeTargetIdentity, PracticeTargetTag, ProposalNonce,
    TaggedPracticeTarget, PRACTICE_INTENT_SOURCE_SHA256,
};
use serde_json::Value;

const SCHEMA: &[u8] = include_bytes!("../../../../contracts/practice_intent.yaml");
const VECTORS: &str = include_str!("../../../../contracts/practice_intent_vectors.jsonl");

fn actor_id(value: u64) -> ActorOrganizationId {
    ActorOrganizationId::from_bytes(value.to_be_bytes())
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

fn intent_from_vector(data: &Value) -> PracticeIntent {
    PracticeIntent {
        schema_version: 2,
        submit_after_tick: data["submit_after_tick"].as_u64().unwrap(),
        resolve_tick: data["resolve_tick"].as_u64().unwrap(),
        input_authority_id: InputAuthorityId::from_bytes(
            hex_bytes(data["input_authority_id_hex"].as_str().unwrap())
                .try_into()
                .unwrap(),
        ),
        actor_org_id: actor_id(data["actor_org_id"].as_u64().unwrap()),
        practice_id: PracticeId::try_from(
            u8::try_from(data["practice_id"].as_u64().unwrap()).unwrap(),
        )
        .unwrap(),
        target: TaggedPracticeTarget {
            tag: PracticeTargetTag::try_from(
                u8::try_from(data["target_tag"].as_u64().unwrap()).unwrap(),
            )
            .unwrap(),
            identity: PracticeTargetIdentity::from_bytes(
                hex_bytes(data["target_identity_hex"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
            ),
        },
        proposal_nonce: ProposalNonce::from_bytes(
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
fn intent_schema_and_vectors_drive_the_rust_boundary() {
    assert_eq!(
        babylon_kernel::content_digest::sha256_of(SCHEMA),
        PRACTICE_INTENT_SOURCE_SHA256
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
    assert_eq!(encode_practice_intent(&intent).unwrap(), canonical);
    assert_eq!(decode_practice_intent(&canonical).unwrap(), intent);
    assert_eq!(
        practice_intent_digest(&intent).unwrap().to_vec(),
        hex_bytes(intent_case["data"]["digest_hex"].as_str().unwrap())
    );

    assert_eq!(
        practice_parameter_bytes_digest(&intent).unwrap().as_slice(),
        hex_bytes(
            intent_case["data"]["parameter_digest_hex"]
                .as_str()
                .unwrap()
        )
    );
    assert_eq!(
        fixed_practice_target_digest(intent.target.tag, intent.target.identity).as_slice(),
        hex_bytes(
            intent_case["data"]["fixed_target_digest_hex"]
                .as_str()
                .unwrap()
        )
    );
    let mut unsupported_domain = canonical.clone();
    unsupported_domain[babylon_practice_contract::PRACTICE_INTENT_DOMAIN_BYTES.len() - 1] = b'1';
    assert_eq!(
        decode_practice_intent(&unsupported_domain),
        Err(PracticeIntentError::IntentDomain)
    );

    for case in cases.iter().filter(|case| case["kind"] == "invalid_wire") {
        let payload = hex_bytes(case["data"]["payload_hex"].as_str().unwrap());
        let expected = PracticeIntentError::try_from(
            u16::try_from(case["data"]["error"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(decode_practice_intent(&payload).map(|_| ()), Err(expected));
    }
}
