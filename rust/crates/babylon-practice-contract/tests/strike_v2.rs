use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    admit_strike_proposal_v2, admitted_strike_proposal_v2_digest,
    decode_admitted_strike_proposal_v2, decode_strike_labor_process_register_v2,
    decode_strike_proposal_contract_v2, encode_admitted_strike_proposal_v2,
    encode_strike_labor_process_register_v2, encode_strike_proposal_contract_v2,
    input_authority_ledger_v2_digest, practice_proposal_key_v2,
    practice_resource_allocation_contract_v2_digest, strike_labor_process_register_v2_digest,
    strike_proposal_contract_v2_digest, validate_strike_labor_process_register_v2, CampaignIdV2,
    InputAuthorityIdV2, PracticeAuthorityKindV2, PracticeIdV2, PracticeInputAuthorityLedgerV2,
    PracticeInputAuthorityV2, PracticeIntentV2, PracticeIntentV2Error, PracticeParameterV2,
    PracticeResourceAllocationContractV2, PracticeTargetIdentityV2, PracticeTargetTagV2,
    ProposalNonceV2, ResolveStrikeProposalV2Error, ResolvedPracticeBatchItemV2,
    ResolvedPracticeBatchV2, StrikeAffectedWorkerCohortV2, StrikeLaborProcessRegisterV2,
    StrikeParticipationStateV2, StrikeProposalContractV2, StrikeProposalV2Error,
    StrikeWorkerCohortIdentityV2, StrikeWorkerOrganizationRelationV2, TaggedPracticeTargetV2,
    ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES, MAX_STRIKE_AFFECTED_COHORTS_V2,
    MAX_STRIKE_ORGANIZATION_RELATIONS_V2, STRIKE_LABOR_PROCESS_REGISTER_V2_DOMAIN_BYTES,
    STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES,
};

const CONTENT_DIGEST: [u8; 32] = [0x30; 32];
const LABOR_PROCESS: [u8; 32] = [0x40; 32];
const OTHER_LABOR_PROCESS: [u8; 32] = [0x41; 32];
const ASSEMBLY_WORKERS: [u8; 32] = [0x50; 32];
const LOGISTICS_WORKERS: [u8; 32] = [0x51; 32];

fn actor_id(value: u64) -> ActorOrganizationIdV2 {
    ActorOrganizationIdV2::from_bytes(value.to_be_bytes())
}

fn strike_intent(actor_org_id: u64) -> PracticeIntentV2 {
    PracticeIntentV2 {
        schema_version: 2,
        submit_after_tick: 40,
        resolve_tick: 41,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x10; 16]),
        actor_org_id: actor_id(actor_org_id),
        practice_id: PracticeIdV2::Strike,
        target: TaggedPracticeTargetV2 {
            tag: PracticeTargetTagV2::LaborProcess,
            identity: PracticeTargetIdentityV2::from_bytes(LABOR_PROCESS),
        },
        proposal_nonce: ProposalNonceV2::from_bytes([0x20; 16]),
        quoted_content_digest: CONTENT_DIGEST,
        quoted_resource_contract_digest: practice_resource_allocation_contract_v2_digest(
            &PracticeResourceAllocationContractV2::conservation_first(),
        )
        .expect("the governed allocation contract has a digest"),
        parameters: Vec::new(),
        evidence_digests: Vec::new(),
    }
}

fn authoritative_context(
    intent: &PracticeIntentV2,
) -> (
    PracticeInputAuthorityLedgerV2,
    ResolvedPracticeBatchV2,
    babylon_practice_contract::PracticeProposalKeyV2,
) {
    let authority = PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x01; 16]),
        authority_kind: PracticeAuthorityKindV2::PlayerSeat,
        input_authority_id: intent.input_authority_id,
        actor_org_id: intent.actor_org_id,
        effective_from_tick: 40,
        effective_through_tick_exclusive: 42,
        decision_content_digest: CONTENT_DIGEST,
    };
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![authority.clone()],
    };
    let batch = ResolvedPracticeBatchV2 {
        schema_version: 2,
        campaign_id: authority.campaign_id,
        resolve_tick: intent.resolve_tick,
        authority_ledger_digest: input_authority_ledger_v2_digest(&ledger).unwrap(),
        resource_allocation_contract_digest: intent.quoted_resource_contract_digest,
        content_digest: intent.quoted_content_digest,
        items: vec![ResolvedPracticeBatchItemV2 {
            authority,
            intent: intent.clone(),
        }],
    };
    (ledger, batch, practice_proposal_key_v2(intent))
}

fn admit(
    intent: &PracticeIntentV2,
    register: &StrikeLaborProcessRegisterV2,
) -> Result<babylon_practice_contract::AdmittedStrikeProposalV2, ResolveStrikeProposalV2Error> {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let (ledger, batch, proposal_key) = authoritative_context(intent);
    admit_strike_proposal_v2(&contract, &batch, &ledger, proposal_key, register)
}

fn affected(labor_process: [u8; 32], cohort: [u8; 32]) -> StrikeAffectedWorkerCohortV2 {
    StrikeAffectedWorkerCohortV2 {
        labor_process_id: PracticeTargetIdentityV2::from_bytes(labor_process),
        worker_cohort_id: StrikeWorkerCohortIdentityV2::from_bytes(cohort),
        labor_relation_digest: [cohort[0].wrapping_add(1); 32],
    }
}

fn relation(
    labor_process: [u8; 32],
    cohort: [u8; 32],
    organization_id: u64,
) -> StrikeWorkerOrganizationRelationV2 {
    StrikeWorkerOrganizationRelationV2 {
        labor_process_id: PracticeTargetIdentityV2::from_bytes(labor_process),
        worker_cohort_id: StrikeWorkerCohortIdentityV2::from_bytes(cohort),
        organization_id: actor_id(organization_id),
        membership_attribution_digest: [organization_id.to_be_bytes()[7]; 32],
    }
}

fn register(organization_id: u64) -> StrikeLaborProcessRegisterV2 {
    StrikeLaborProcessRegisterV2 {
        schema_version: 2,
        resolve_tick: 41,
        content_digest: CONTENT_DIGEST,
        affected_cohorts: vec![
            affected(LABOR_PROCESS, ASSEMBLY_WORKERS),
            affected(LABOR_PROCESS, LOGISTICS_WORKERS),
        ],
        organization_relations: vec![relation(LABOR_PROCESS, ASSEMBLY_WORKERS, organization_id)],
    }
}

#[test]
fn materially_connected_worker_organization_can_propose_for_every_affected_cohort() {
    let intent = strike_intent(101);
    let register = register(101);

    let admission = admit(&intent, &register)
        .expect("an inhabited organization with attributed workers in the process is eligible");

    assert_eq!(admission.participation_rows().len(), 2);
    assert_eq!(
        admission.participation_rows()[0].worker_cohort_id(),
        StrikeWorkerCohortIdentityV2::from_bytes(ASSEMBLY_WORKERS)
    );
    assert_eq!(
        admission.participation_rows()[1].worker_cohort_id(),
        StrikeWorkerCohortIdentityV2::from_bytes(LOGISTICS_WORKERS)
    );
    assert!(admission
        .participation_rows()
        .iter()
        .all(|row| row.state() == StrikeParticipationStateV2::PendingIndependentResolution));
}

#[test]
fn outside_supporter_cannot_remotely_command_a_strike() {
    let intent = strike_intent(202);

    assert_eq!(
        admit(&intent, &register(101)),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeOrganizationNotConnected
        ))
    );
}

#[test]
fn proposal_must_exist_in_the_authoritative_resolved_batch() {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let intent = strike_intent(101);
    let (ledger, batch, mut proposal_key) = authoritative_context(&intent);
    proposal_key.proposal_nonce = ProposalNonceV2::from_bytes([0x21; 16]);

    assert_eq!(
        admit_strike_proposal_v2(&contract, &batch, &ledger, proposal_key, &register(101)),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeProposalNotAccepted
        ))
    );
}

#[test]
fn connection_to_a_different_labor_process_grants_no_strike_authority() {
    let intent = strike_intent(101);
    let register = StrikeLaborProcessRegisterV2 {
        schema_version: 2,
        resolve_tick: 41,
        content_digest: CONTENT_DIGEST,
        affected_cohorts: vec![
            affected(LABOR_PROCESS, ASSEMBLY_WORKERS),
            affected(OTHER_LABOR_PROCESS, LOGISTICS_WORKERS),
        ],
        organization_relations: vec![relation(OTHER_LABOR_PROCESS, LOGISTICS_WORKERS, 101)],
    };

    assert_eq!(
        admit(&intent, &register),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeOrganizationNotConnected
        ))
    );
}

#[test]
fn strike_proposal_cannot_author_worker_participation() {
    let mut intent = strike_intent(101);
    intent.parameters.push(PracticeParameterV2 {
        key_u8: 1,
        value_kind_u8: 1,
        value_length_u16: 1,
        value_bytes: vec![1],
    });

    assert_eq!(
        admit(&intent, &register(101)),
        Err(ResolveStrikeProposalV2Error::Batch(
            babylon_practice_contract::ResolvedPracticeBatchV2Error::Intent(
                PracticeIntentV2Error::IntentParameterUnsupported
            )
        ))
    );
}

#[test]
fn non_strike_practice_cannot_enter_the_strike_resolver() {
    let mut intent = strike_intent(101);
    intent.practice_id = PracticeIdV2::Organize;
    intent.target.tag = PracticeTargetTagV2::SocialClass;

    assert_eq!(
        admit(&intent, &register(101)),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikePracticeMismatch
        ))
    );
}

#[test]
fn organization_relation_must_reference_an_affected_cohort() {
    let intent = strike_intent(101);
    let register = StrikeLaborProcessRegisterV2 {
        schema_version: 2,
        resolve_tick: 41,
        content_digest: CONTENT_DIGEST,
        affected_cohorts: vec![affected(LABOR_PROCESS, ASSEMBLY_WORKERS)],
        organization_relations: vec![relation(LABOR_PROCESS, LOGISTICS_WORKERS, 101)],
    };

    assert_eq!(
        admit(&intent, &register),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeRelationCohortMissing
        ))
    );
}

#[test]
fn contract_register_and_admission_round_trip_exact_bytes() {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let intent = strike_intent(101);
    let register = register(101);
    let admission = admit(&intent, &register).unwrap();

    let contract_bytes = encode_strike_proposal_contract_v2(&contract).unwrap();
    let register_bytes = encode_strike_labor_process_register_v2(&register).unwrap();
    let admission_bytes = encode_admitted_strike_proposal_v2(&contract, &admission).unwrap();
    let (ledger, batch, proposal_key) = authoritative_context(&intent);

    assert_eq!(contract_bytes.len(), 48);
    assert_eq!(register_bytes.len(), 387);
    assert_eq!(admission_bytes.len(), 286);
    assert_eq!(
        decode_strike_proposal_contract_v2(&contract_bytes),
        Ok(contract)
    );
    assert_eq!(
        decode_strike_labor_process_register_v2(&register_bytes),
        Ok(register.clone())
    );
    assert_eq!(
        decode_admitted_strike_proposal_v2(
            &admission_bytes,
            &StrikeProposalContractV2::materially_connected_workers(),
            &batch,
            &ledger,
            proposal_key,
            &register,
        ),
        Ok(admission.clone())
    );
    assert_eq!(
        admission.labor_process_register_digest(),
        strike_labor_process_register_v2_digest(&register).unwrap()
    );
    assert_eq!(
        admitted_strike_proposal_v2_digest(
            &StrikeProposalContractV2::materially_connected_workers(),
            &admission,
        )
        .unwrap(),
        babylon_kernel::sha256_of(&admission_bytes)
    );
}

#[test]
fn contract_bytes_have_an_independent_literal_layout() {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let mut expected = STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES.to_vec();
    expected.push(0);
    expected.extend_from_slice(&2_u16.to_be_bytes());
    expected.push(1);
    expected.push(1);
    expected.extend_from_slice(&65_536_u32.to_be_bytes());
    expected.extend_from_slice(&65_536_u32.to_be_bytes());

    assert_eq!(
        encode_strike_proposal_contract_v2(&contract).unwrap(),
        expected
    );
    assert_eq!(
        strike_proposal_contract_v2_digest(&contract).unwrap(),
        babylon_kernel::sha256_of(&expected)
    );
}

#[test]
fn forged_admission_cannot_omit_an_affected_worker_cohort() {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let intent = strike_intent(101);
    let register = register(101);
    let admission = admit(&intent, &register).unwrap();
    let mut payload = encode_admitted_strike_proposal_v2(&contract, &admission).unwrap();
    let (ledger, batch, proposal_key) = authoritative_context(&intent);
    let row_count_offset =
        ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES.len() + 1 + 2 + 32 + 32 + 32 + 82;
    payload[row_count_offset..row_count_offset + 4].copy_from_slice(&1_u32.to_be_bytes());
    payload.truncate(payload.len() - 33);

    assert_eq!(
        decode_admitted_strike_proposal_v2(
            &payload,
            &contract,
            &batch,
            &ledger,
            proposal_key,
            &register,
        ),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeAdmissionMismatch
        ))
    );
}

#[test]
fn forged_admission_cannot_substitute_an_affected_worker_cohort() {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let intent = strike_intent(101);
    let register = register(101);
    let admission = admit(&intent, &register).unwrap();
    let mut payload = encode_admitted_strike_proposal_v2(&contract, &admission).unwrap();
    let (ledger, batch, proposal_key) = authoritative_context(&intent);
    let first_row_offset =
        ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES.len() + 1 + 2 + 32 + 32 + 32 + 82 + 4;
    payload[first_row_offset..first_row_offset + 32].fill(0x52);

    assert_eq!(
        decode_admitted_strike_proposal_v2(
            &payload,
            &contract,
            &batch,
            &ledger,
            proposal_key,
            &register,
        ),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeAdmissionMismatch
        ))
    );
}

#[test]
fn admission_decoder_refuses_a_non_strike_proposal_key_kind() {
    let contract = StrikeProposalContractV2::materially_connected_workers();
    let intent = strike_intent(101);
    let register = register(101);
    let admission = admit(&intent, &register).unwrap();
    let payload = encode_admitted_strike_proposal_v2(&contract, &admission).unwrap();
    let (ledger, batch, proposal_key) = authoritative_context(&intent);
    let proposal_key_offset = ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES.len() + 1 + 2 + 32 + 32 + 32;
    let practice_offset = proposal_key_offset + 8 + 16 + 8;
    let target_tag_offset = practice_offset + 1;

    for (offset, replacement) in [
        (practice_offset, PracticeIdV2::Organize as u8),
        (target_tag_offset, PracticeTargetTagV2::Route as u8),
    ] {
        let mut forged = payload.clone();
        forged[offset] = replacement;
        assert_eq!(
            decode_admitted_strike_proposal_v2(
                &forged,
                &contract,
                &batch,
                &ledger,
                proposal_key,
                &register,
            ),
            Err(ResolveStrikeProposalV2Error::Strike(
                StrikeProposalV2Error::StrikePracticeMismatch
            ))
        );
    }
}

#[test]
fn register_rows_are_canonical_and_unique() {
    let mut unordered = register(101);
    unordered.affected_cohorts.reverse();
    assert_eq!(
        validate_strike_labor_process_register_v2(&unordered),
        Err(StrikeProposalV2Error::StrikeAffectedCohortOrder)
    );

    let mut duplicate = register(101);
    duplicate
        .affected_cohorts
        .insert(1, duplicate.affected_cohorts[0]);
    assert_eq!(
        validate_strike_labor_process_register_v2(&duplicate),
        Err(StrikeProposalV2Error::StrikeAffectedCohortDuplicate)
    );

    let mut relation_duplicate = register(101);
    relation_duplicate
        .organization_relations
        .push(relation_duplicate.organization_relations[0]);
    assert_eq!(
        validate_strike_labor_process_register_v2(&relation_duplicate),
        Err(StrikeProposalV2Error::StrikeOrganizationRelationDuplicate)
    );

    let mut relation_unordered = register(102);
    relation_unordered
        .organization_relations
        .push(relation(LABOR_PROCESS, ASSEMBLY_WORKERS, 101));
    assert_eq!(
        validate_strike_labor_process_register_v2(&relation_unordered),
        Err(StrikeProposalV2Error::StrikeOrganizationRelationOrder)
    );
}

#[test]
fn register_and_intent_identity_must_match_exactly() {
    let intent = strike_intent(101);

    let mut wrong_tick = register(101);
    wrong_tick.resolve_tick = 42;
    assert_eq!(
        admit(&intent, &wrong_tick),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeResolveTickMismatch
        ))
    );

    let mut wrong_content = register(101);
    wrong_content.content_digest = [0x31; 32];
    assert_eq!(
        admit(&intent, &wrong_content),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeContentDigestMismatch
        ))
    );

    let empty_target = StrikeLaborProcessRegisterV2 {
        schema_version: 2,
        resolve_tick: 41,
        content_digest: CONTENT_DIGEST,
        affected_cohorts: vec![affected(OTHER_LABOR_PROCESS, ASSEMBLY_WORKERS)],
        organization_relations: vec![relation(OTHER_LABOR_PROCESS, ASSEMBLY_WORKERS, 101)],
    };
    assert_eq!(
        admit(&intent, &empty_target),
        Err(ResolveStrikeProposalV2Error::Strike(
            StrikeProposalV2Error::StrikeTargetNoAffectedCohort
        ))
    );
}

#[test]
fn maximum_plus_one_cohorts_refuses_before_encoding_rows() {
    let repeated = affected(LABOR_PROCESS, ASSEMBLY_WORKERS);
    let register = StrikeLaborProcessRegisterV2 {
        schema_version: 2,
        resolve_tick: 41,
        content_digest: CONTENT_DIGEST,
        affected_cohorts: vec![repeated; MAX_STRIKE_AFFECTED_COHORTS_V2 + 1],
        organization_relations: Vec::new(),
    };

    assert_eq!(
        encode_strike_labor_process_register_v2(&register),
        Err(StrikeProposalV2Error::StrikeAffectedCohortLimit)
    );
}

#[test]
fn maximum_plus_one_organization_relations_refuses_before_encoding_rows() {
    let repeated = relation(LABOR_PROCESS, ASSEMBLY_WORKERS, 101);
    let register = StrikeLaborProcessRegisterV2 {
        schema_version: 2,
        resolve_tick: 41,
        content_digest: CONTENT_DIGEST,
        affected_cohorts: vec![affected(LABOR_PROCESS, ASSEMBLY_WORKERS)],
        organization_relations: vec![repeated; MAX_STRIKE_ORGANIZATION_RELATIONS_V2 + 1],
    };

    assert_eq!(
        encode_strike_labor_process_register_v2(&register),
        Err(StrikeProposalV2Error::StrikeOrganizationRelationLimit)
    );
}

#[test]
fn domains_and_all_error_codes_remain_closed() {
    assert_eq!(
        STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES,
        b"babylon.strike-proposal-contract.v2"
    );
    assert_eq!(
        STRIKE_LABOR_PROCESS_REGISTER_V2_DOMAIN_BYTES,
        b"babylon.strike-labor-process-register.v2"
    );
    assert_eq!(
        ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES,
        b"babylon.admitted-strike-proposal.v2"
    );
    for code in 1_u16..=23 {
        let error = StrikeProposalV2Error::try_from(code).unwrap();
        assert_eq!(u16::from(error), code);
    }
    assert!(StrikeProposalV2Error::try_from(0).is_err());
    assert!(StrikeProposalV2Error::try_from(24).is_err());
}
