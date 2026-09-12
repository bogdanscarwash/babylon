use babylon_practice_contract::ActorOrganizationId;
use babylon_practice_contract::{
    admitted_strike_proposal_digest, decode_admitted_strike_proposal,
    decode_strike_labor_process_register, decode_strike_proposal_contract,
    encode_admitted_strike_proposal, encode_strike_labor_process_register,
    encode_strike_proposal_contract, input_authority_ledger_digest, practice_proposal_key,
    practice_resource_allocation_contract_digest, strike_labor_process_register_digest,
    strike_proposal_contract_digest, CampaignId, InputAuthorityId, PracticeAuthorityKind,
    PracticeId, PracticeInputAuthority, PracticeInputAuthorityLedger, PracticeIntent,
    PracticeResourceAllocationContract, PracticeTargetIdentity, PracticeTargetTag, ProposalNonce,
    ResolvedPracticeBatch, ResolvedPracticeBatchItem, StrikeProposalError, TaggedPracticeTarget,
    STRIKE_PROPOSAL_SOURCE_SHA256,
};
use serde_json::Value;

const SCHEMA: &[u8] = include_bytes!("../../../../contracts/strike_proposal.yaml");
const VECTORS: &str = include_str!("../../../../contracts/strike_proposal_vectors.jsonl");

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

fn strike_intent() -> PracticeIntent {
    PracticeIntent {
        schema_version: 2,
        submit_after_tick: 40,
        resolve_tick: 41,
        input_authority_id: InputAuthorityId::from_bytes([0x10; 16]),
        actor_org_id: actor_id(101),
        practice_id: PracticeId::Strike,
        target: TaggedPracticeTarget {
            tag: PracticeTargetTag::LaborProcess,
            identity: PracticeTargetIdentity::from_bytes([0x40; 32]),
        },
        proposal_nonce: ProposalNonce::from_bytes([0x20; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: practice_resource_allocation_contract_digest(
            &PracticeResourceAllocationContract::conservation_first(),
        )
        .unwrap(),
        parameters: Vec::new(),
        evidence_digests: Vec::new(),
    }
}

fn authoritative_context(
    intent: &PracticeIntent,
) -> (PracticeInputAuthorityLedger, ResolvedPracticeBatch) {
    let authority = PracticeInputAuthority {
        schema_version: 2,
        campaign_id: CampaignId::from_bytes([0x01; 16]),
        authority_kind: PracticeAuthorityKind::PlayerSeat,
        input_authority_id: intent.input_authority_id,
        actor_org_id: intent.actor_org_id,
        effective_from_tick: 40,
        effective_through_tick_exclusive: 42,
        decision_content_digest: [0x30; 32],
    };
    let ledger = PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![authority.clone()],
    };
    let batch = ResolvedPracticeBatch {
        schema_version: 2,
        campaign_id: authority.campaign_id,
        resolve_tick: intent.resolve_tick,
        authority_ledger_digest: input_authority_ledger_digest(&ledger).unwrap(),
        resource_allocation_contract_digest: intent.quoted_resource_contract_digest,
        content_digest: intent.quoted_content_digest,
        items: vec![ResolvedPracticeBatchItem {
            authority,
            intent: intent.clone(),
        }],
    };
    (ledger, batch)
}

#[test]
fn language_neutral_strike_vectors_drive_every_rust_digest() {
    assert_eq!(
        babylon_kernel::content_digest::sha256_of(SCHEMA),
        STRIKE_PROPOSAL_SOURCE_SHA256
    );
    let cases = cases();
    assert_eq!(cases.len(), 10);

    let manifest = case(&cases, "manifest");
    assert_eq!(
        hex_bytes(manifest["data"]["schema_sha256"].as_str().unwrap()),
        STRIKE_PROPOSAL_SOURCE_SHA256
    );
    assert_eq!(manifest["data"]["contract_bytes"], 48);
    assert_eq!(manifest["data"]["register_example_bytes"], 387);
    assert_eq!(manifest["data"]["admission_example_bytes"], 286);

    let contract_case = case(&cases, "strike-contract");
    let contract_bytes = hex_bytes(contract_case["data"]["canonical_hex"].as_str().unwrap());
    let contract = decode_strike_proposal_contract(&contract_bytes).unwrap();
    assert_eq!(
        encode_strike_proposal_contract(&contract).unwrap(),
        contract_bytes
    );
    assert_eq!(
        strike_proposal_contract_digest(&contract).unwrap().to_vec(),
        hex_bytes(contract_case["data"]["digest_hex"].as_str().unwrap())
    );

    let register_case = case(&cases, "labor-process-register");
    let register_bytes = hex_bytes(register_case["data"]["canonical_hex"].as_str().unwrap());
    let register = decode_strike_labor_process_register(&register_bytes).unwrap();
    assert_eq!(
        encode_strike_labor_process_register(&register).unwrap(),
        register_bytes
    );
    assert_eq!(
        strike_labor_process_register_digest(&register)
            .unwrap()
            .to_vec(),
        hex_bytes(register_case["data"]["digest_hex"].as_str().unwrap())
    );

    let admission_case = case(&cases, "admitted-strike-proposal");
    let admission_bytes = hex_bytes(admission_case["data"]["canonical_hex"].as_str().unwrap());
    let intent = strike_intent();
    let (ledger, batch) = authoritative_context(&intent);
    let admission = decode_admitted_strike_proposal(
        &admission_bytes,
        &contract,
        &batch,
        &ledger,
        practice_proposal_key(&intent),
        &register,
    )
    .unwrap();
    assert_eq!(
        encode_admitted_strike_proposal(&contract, &admission).unwrap(),
        admission_bytes
    );
    assert_eq!(
        admitted_strike_proposal_digest(&contract, &admission)
            .unwrap()
            .to_vec(),
        hex_bytes(admission_case["data"]["digest_hex"].as_str().unwrap())
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
        let expected = StrikeProposalError::try_from(code).unwrap();
        assert_eq!(
            decode_strike_proposal_contract(&payload),
            Err(expected),
            "{}",
            value["case_id"]
        );
    }
}
