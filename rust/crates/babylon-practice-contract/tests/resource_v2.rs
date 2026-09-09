use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    allocate_practice_resources_v2, decode_practice_resource_allocation_contract_v2,
    decode_practice_resource_allocation_outcome_v2, decode_practice_resource_capacity_v2,
    decode_practice_resource_request_v2, derive_practice_resource_request_v2,
    encode_practice_resource_allocation_contract_v2,
    encode_practice_resource_allocation_outcome_v2, encode_practice_resource_capacity_v2,
    encode_practice_resource_request_v2, practice_resource_allocation_contract_v2_digest,
    practice_resource_allocation_outcome_v2_digest, practice_resource_capacity_v2_digest,
    practice_resource_request_v2_digest, InputAuthorityIdV2, PracticeIdV2, PracticeIntentV2,
    PracticeResourceAllocationContractV2, PracticeResourceAllocationModeV2,
    PracticeResourceCapacityV2, PracticeResourceIdV2, PracticeResourceLocatorV2,
    PracticeResourceRequirementV2, PracticeResourceV2Error, PracticeTargetIdentityV2,
    PracticeTargetTagV2, PracticeUnitIdV2, ProposalNonceV2, TaggedPracticeTargetV2,
    MAX_PRACTICE_RESOURCE_CAPACITIES_V2, MAX_PRACTICE_RESOURCE_REQUESTS_V2,
    PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES,
    PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES, PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES,
};

const RESOURCE_CONTRACT_DIGEST: [u8; 32] = [
    0x95, 0xea, 0x4f, 0xf2, 0x1e, 0x74, 0x8c, 0x7a, 0x18, 0x49, 0xd0, 0xa9, 0xf2, 0xd6, 0x87, 0xde,
    0x5d, 0x23, 0x38, 0x6e, 0x61, 0xea, 0x4e, 0x01, 0x59, 0xbc, 0x99, 0xaf, 0xd6, 0xee, 0x91, 0xb9,
];

fn actor_id(value: u64) -> ActorOrganizationIdV2 {
    ActorOrganizationIdV2::from_bytes(value.to_be_bytes())
}

fn intent(authority_marker: u8, actor_org_id: u64) -> PracticeIntentV2 {
    intent_with_nonce(authority_marker, actor_org_id, authority_marker)
}

fn intent_with_nonce(
    authority_marker: u8,
    actor_org_id: u64,
    proposal_marker: u8,
) -> PracticeIntentV2 {
    PracticeIntentV2 {
        schema_version: 2,
        submit_after_tick: 40,
        resolve_tick: 41,
        input_authority_id: InputAuthorityIdV2::from_bytes([authority_marker; 16]),
        actor_org_id: actor_id(actor_org_id),
        practice_id: PracticeIdV2::Strike,
        target: TaggedPracticeTargetV2 {
            tag: PracticeTargetTagV2::LaborProcess,
            identity: PracticeTargetIdentityV2::from_bytes([0x20; 32]),
        },
        proposal_nonce: ProposalNonceV2::from_bytes([proposal_marker; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: RESOURCE_CONTRACT_DIGEST,
        parameters: Vec::new(),
        evidence_digests: Vec::new(),
    }
}

fn shared_requirement(quantity: u64) -> PracticeResourceRequirementV2 {
    PracticeResourceRequirementV2 {
        practice_id: PracticeIdV2::Strike,
        locator: PracticeResourceLocatorV2::Shared,
        resource_id: PracticeResourceIdV2::from_bytes([0x50; 32]),
        unit_id: PracticeUnitIdV2::from_bytes([0x60; 32]),
        quantity,
    }
}

#[test]
fn divisible_scarcity_allocates_the_same_material_fulfillment_ratio() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let large =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(90))
            .expect("matching practice derives its sealed request");
    let small =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(10))
            .expect("matching practice derives its sealed request");
    let capacity = PracticeResourceCapacityV2 {
        owner: large.owner(),
        resource_id: large.resource_id(),
        unit_id: large.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 50,
    };

    let outcome = allocate_practice_resources_v2(&contract, &[large, small], &[capacity])
        .expect("valid sealed requests allocate");

    assert_eq!(outcome.allocations()[0].requested(), 90);
    assert_eq!(outcome.allocations()[0].allocated(), 45);
    assert_eq!(outcome.allocations()[1].requested(), 10);
    assert_eq!(outcome.allocations()[1].allocated(), 5);
    assert_eq!(outcome.balances()[0].allocated(), 50);
    assert_eq!(outcome.balances()[0].unallocated(), 0);
}

#[test]
fn unused_capacity_remains_explicitly_unallocated() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let capacity = PracticeResourceCapacityV2 {
        owner: babylon_practice_contract::PracticeResourceOwnerV2::Shared,
        resource_id: PracticeResourceIdV2::from_bytes([0x51; 32]),
        unit_id: PracticeUnitIdV2::from_bytes([0x61; 32]),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 7,
    };

    let outcome = allocate_practice_resources_v2(&contract, &[], &[capacity])
        .expect("unused capacity is still conserved");

    assert_eq!(outcome.balances().len(), 1);
    assert_eq!(outcome.balances()[0].allocated(), 0);
    assert_eq!(outcome.balances()[0].unallocated(), 7);
}

#[test]
fn divisible_floor_residual_remains_unallocated() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let first =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(1))
            .unwrap();
    let second =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(1))
            .unwrap();
    let capacity = PracticeResourceCapacityV2 {
        owner: first.owner(),
        resource_id: first.resource_id(),
        unit_id: first.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 1,
    };

    let outcome = allocate_practice_resources_v2(&contract, &[first, second], &[capacity]).unwrap();

    assert_eq!(outcome.allocations()[0].allocated(), 0);
    assert_eq!(outcome.allocations()[1].allocated(), 0);
    assert_eq!(outcome.balances()[0].allocated(), 0);
    assert_eq!(outcome.balances()[0].unallocated(), 1);
}

#[test]
fn full_supply_grants_every_request_and_retains_surplus() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let first =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(3))
            .unwrap();
    let second =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(2))
            .unwrap();
    let capacity = PracticeResourceCapacityV2 {
        owner: first.owner(),
        resource_id: first.resource_id(),
        unit_id: first.unit_id(),
        mode: PracticeResourceAllocationModeV2::ExclusiveAllOrNone,
        available: 10,
    };

    let outcome = allocate_practice_resources_v2(&contract, &[first, second], &[capacity]).unwrap();

    assert_eq!(outcome.allocations()[0].allocated(), 3);
    assert_eq!(outcome.allocations()[1].allocated(), 2);
    assert_eq!(outcome.balances()[0].allocated(), 5);
    assert_eq!(outcome.balances()[0].unallocated(), 5);
}

#[test]
fn exclusive_shared_scarcity_selects_no_winner_by_proposal_order() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let first =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(1))
            .expect("first structurally valid cross-actor request enters resolution");
    let second =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(1))
            .expect("second structurally valid cross-actor request enters resolution");
    let capacity = PracticeResourceCapacityV2 {
        owner: first.owner(),
        resource_id: first.resource_id(),
        unit_id: first.unit_id(),
        mode: PracticeResourceAllocationModeV2::ExclusiveAllOrNone,
        available: 1,
    };

    let outcome = allocate_practice_resources_v2(&contract, &[first, second], &[capacity])
        .expect("cross-actor contention is a material outcome, not a submission failure");

    assert_eq!(outcome.allocations()[0].allocated(), 0);
    assert_eq!(outcome.allocations()[1].allocated(), 0);
    assert_eq!(outcome.balances()[0].allocated(), 0);
    assert_eq!(outcome.balances()[0].unallocated(), 1);
}

#[test]
fn same_authority_exclusive_conflict_refuses_the_complete_group() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let first = derive_practice_resource_request_v2(
        &contract,
        &intent_with_nonce(1, 101, 1),
        &shared_requirement(1),
    )
    .expect("first proposal derives its sealed request");
    let second = derive_practice_resource_request_v2(
        &contract,
        &intent_with_nonce(1, 101, 2),
        &shared_requirement(1),
    )
    .expect("second proposal derives its sealed request");
    let capacity = PracticeResourceCapacityV2 {
        owner: first.owner(),
        resource_id: first.resource_id(),
        unit_id: first.unit_id(),
        mode: PracticeResourceAllocationModeV2::ExclusiveAllOrNone,
        available: 1,
    };

    assert_eq!(
        allocate_practice_resources_v2(&contract, &[first, second], &[capacity]),
        Err(PracticeResourceV2Error::ResourceAuthorityConflict)
    );
}

#[test]
fn allocation_output_is_identical_for_every_request_permutation() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let large =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(90))
            .expect("large request derives from sealed content");
    let small =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(10))
            .expect("small request derives from sealed content");
    let capacity = PracticeResourceCapacityV2 {
        owner: large.owner(),
        resource_id: large.resource_id(),
        unit_id: large.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 50,
    };

    let forward = allocate_practice_resources_v2(
        &contract,
        &[large.clone(), small.clone()],
        std::slice::from_ref(&capacity),
    )
    .expect("forward input allocates");
    let reversed = allocate_practice_resources_v2(&contract, &[small, large], &[capacity])
        .expect("reversed input allocates");

    assert_eq!(forward, reversed);
}

#[test]
fn allocation_output_is_identical_for_every_capacity_permutation() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let request =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(1))
            .unwrap();
    let used = PracticeResourceCapacityV2 {
        owner: request.owner(),
        resource_id: request.resource_id(),
        unit_id: request.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 1,
    };
    let unused = PracticeResourceCapacityV2 {
        owner: babylon_practice_contract::PracticeResourceOwnerV2::Shared,
        resource_id: PracticeResourceIdV2::from_bytes([0x51; 32]),
        unit_id: PracticeUnitIdV2::from_bytes([0x61; 32]),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 7,
    };

    let forward = allocate_practice_resources_v2(
        &contract,
        std::slice::from_ref(&request),
        &[used.clone(), unused.clone()],
    )
    .unwrap();
    let reversed = allocate_practice_resources_v2(&contract, &[request], &[unused, used]).unwrap();

    assert_eq!(forward, reversed);
}

#[test]
fn allocation_contract_bytes_freeze_every_material_law() {
    const EXPECTED: &[u8] = b"babylon.practice-resource-allocation-contract.v2\0\
        \x00\x02\x40\x01\x01\x01\x01\x00\x10\
        \x00\x01\x00\x00\x00\x01\x00\x00";
    const EXPECTED_DIGEST: [u8; 32] = [
        0x95, 0xea, 0x4f, 0xf2, 0x1e, 0x74, 0x8c, 0x7a, 0x18, 0x49, 0xd0, 0xa9, 0xf2, 0xd6, 0x87,
        0xde, 0x5d, 0x23, 0x38, 0x6e, 0x61, 0xea, 0x4e, 0x01, 0x59, 0xbc, 0x99, 0xaf, 0xd6, 0xee,
        0x91, 0xb9,
    ];
    let contract = PracticeResourceAllocationContractV2::conservation_first();

    assert_eq!(
        encode_practice_resource_allocation_contract_v2(&contract).unwrap(),
        EXPECTED
    );
    assert_eq!(
        decode_practice_resource_allocation_contract_v2(EXPECTED).unwrap(),
        contract
    );
    assert_eq!(
        practice_resource_allocation_contract_v2_digest(&contract).unwrap(),
        EXPECTED_DIGEST
    );
}

#[test]
fn allocator_refuses_a_mutated_law_with_the_same_schema_version() {
    let mut contract = PracticeResourceAllocationContractV2::conservation_first();
    contract.max_requests_per_intent = 17;

    assert_eq!(
        allocate_practice_resources_v2(&contract, &[], &[]),
        Err(PracticeResourceV2Error::ResourceContractValue)
    );
}

#[test]
fn request_derivation_refuses_a_different_quoted_resource_contract() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let mut proposal = intent(1, 101);
    proposal.quoted_resource_contract_digest = [0x41; 32];

    assert_eq!(
        derive_practice_resource_request_v2(&contract, &proposal, &shared_requirement(1)),
        Err(PracticeResourceV2Error::ResourceContractDigestMismatch)
    );
}

#[test]
fn request_derivation_refuses_practice_mismatch_and_zero_quantity() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let proposal = intent(1, 101);
    let mut requirement = shared_requirement(1);
    requirement.practice_id = PracticeIdV2::Organize;
    assert_eq!(
        derive_practice_resource_request_v2(&contract, &proposal, &requirement),
        Err(PracticeResourceV2Error::ResourceRequirementPracticeMismatch)
    );

    requirement.practice_id = PracticeIdV2::Strike;
    requirement.quantity = 0;
    assert_eq!(
        derive_practice_resource_request_v2(&contract, &proposal, &requirement),
        Err(PracticeResourceV2Error::ResourceRequestZero)
    );
}

#[test]
fn derived_request_bytes_bind_proposal_owner_resource_unit_and_quantity() {
    const EXPECTED_DIGEST: [u8; 32] = [
        0x35, 0x69, 0x5f, 0xcf, 0x35, 0x99, 0xd2, 0x00, 0x34, 0x7a, 0x26, 0xa2, 0x89, 0xe1, 0xc0,
        0x4a, 0x83, 0x5f, 0x1f, 0xaa, 0x4b, 0x56, 0xbb, 0xbb, 0x15, 0x63, 0x9d, 0xa7, 0xf5, 0xb2,
        0x0e, 0x27,
    ];
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let request =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(90))
            .expect("sealed content derives the request");

    let canonical = encode_practice_resource_request_v2(&request).unwrap();

    assert_eq!(canonical.len(), 202);
    assert_eq!(
        practice_resource_request_v2_digest(&request).unwrap(),
        EXPECTED_DIGEST
    );
}

#[test]
fn request_decoder_round_trips_and_refuses_an_unknown_owner_scope() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let request =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(90))
            .unwrap();
    let canonical = encode_practice_resource_request_v2(&request).unwrap();

    assert_eq!(
        decode_practice_resource_request_v2(&canonical).unwrap(),
        request
    );

    let mut unknown_owner = canonical;
    let owner_tag = PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES.len() + 1 + 2 + 82;
    unknown_owner[owner_tag] = 9;
    assert_eq!(
        decode_practice_resource_request_v2(&unknown_owner),
        Err(PracticeResourceV2Error::ResourceEnumCode)
    );
}

#[test]
fn capacity_bytes_bind_owner_resource_unit_mode_and_available_quantity() {
    const EXPECTED_DIGEST: [u8; 32] = [
        0xc2, 0x23, 0x84, 0xe4, 0x7a, 0xd6, 0xce, 0xc8, 0x65, 0x42, 0x99, 0xaf, 0x99, 0x49, 0x85,
        0xa1, 0xac, 0xf4, 0xde, 0xf3, 0x5e, 0xca, 0xb2, 0x89, 0x8c, 0xd7, 0xf1, 0x7e, 0x7b, 0x32,
        0x8e, 0x0e,
    ];
    let capacity = PracticeResourceCapacityV2 {
        owner: babylon_practice_contract::PracticeResourceOwnerV2::Shared,
        resource_id: PracticeResourceIdV2::from_bytes([0x50; 32]),
        unit_id: PracticeUnitIdV2::from_bytes([0x60; 32]),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 50,
    };

    let canonical = encode_practice_resource_capacity_v2(&capacity).unwrap();

    assert_eq!(canonical.len(), 122);
    assert_eq!(
        practice_resource_capacity_v2_digest(&capacity).unwrap(),
        EXPECTED_DIGEST
    );
}

#[test]
fn capacity_decoder_round_trips_and_refuses_an_unknown_allocation_mode() {
    let capacity = PracticeResourceCapacityV2 {
        owner: babylon_practice_contract::PracticeResourceOwnerV2::Shared,
        resource_id: PracticeResourceIdV2::from_bytes([0x50; 32]),
        unit_id: PracticeUnitIdV2::from_bytes([0x60; 32]),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 50,
    };
    let canonical = encode_practice_resource_capacity_v2(&capacity).unwrap();

    assert_eq!(
        decode_practice_resource_capacity_v2(&canonical).unwrap(),
        capacity
    );

    let mut unknown_mode = canonical;
    let mode = PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES.len() + 1 + 2 + 9 + 32 + 32;
    unknown_mode[mode] = 9;
    assert_eq!(
        decode_practice_resource_capacity_v2(&unknown_mode),
        Err(PracticeResourceV2Error::ResourceEnumCode)
    );
}

#[test]
fn outcome_bytes_bind_allocations_and_explicit_capacity_residuals() {
    const EXPECTED_DIGEST: [u8; 32] = [
        0xcd, 0xf9, 0xc8, 0xeb, 0x72, 0x4b, 0x58, 0xc6, 0xd7, 0x14, 0xef, 0xe8, 0xff, 0x54, 0x63,
        0x68, 0x36, 0x6e, 0x87, 0x95, 0xcd, 0x86, 0x30, 0xbd, 0x2f, 0xf0, 0x78, 0x66, 0xd2, 0x37,
        0xdc, 0xd1,
    ];
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let large =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(90))
            .unwrap();
    let small =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(10))
            .unwrap();
    let capacity = PracticeResourceCapacityV2 {
        owner: large.owner(),
        resource_id: large.resource_id(),
        unit_id: large.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 50,
    };
    let outcome = allocate_practice_resources_v2(&contract, &[large, small], &[capacity]).unwrap();

    let canonical = encode_practice_resource_allocation_outcome_v2(&contract, &outcome).unwrap();

    assert_eq!(canonical.len(), 218);
    assert_eq!(
        practice_resource_allocation_outcome_v2_digest(&contract, &outcome).unwrap(),
        EXPECTED_DIGEST
    );
}

#[test]
fn outcome_decoder_replays_the_allocator_and_refuses_a_forged_grant() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let large =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(90))
            .unwrap();
    let small =
        derive_practice_resource_request_v2(&contract, &intent(2, 202), &shared_requirement(10))
            .unwrap();
    let capacity = PracticeResourceCapacityV2 {
        owner: large.owner(),
        resource_id: large.resource_id(),
        unit_id: large.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 50,
    };
    let requests = [large, small];
    let capacities = [capacity];
    let outcome = allocate_practice_resources_v2(&contract, &requests, &capacities).unwrap();
    let canonical = encode_practice_resource_allocation_outcome_v2(&contract, &outcome).unwrap();

    assert_eq!(
        decode_practice_resource_allocation_outcome_v2(
            &canonical,
            &contract,
            &requests,
            &capacities,
        )
        .unwrap(),
        outcome
    );

    let mut forged = canonical;
    let first_grant_last_byte =
        PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES.len() + 1 + 2 + 32 + 4 + 32 + 7;
    forged[first_grant_last_byte] = 44;
    assert_eq!(
        decode_practice_resource_allocation_outcome_v2(&forged, &contract, &requests, &capacities,),
        Err(PracticeResourceV2Error::ResourceOutcomeMismatch)
    );
}

#[test]
fn one_intent_refuses_a_seventeenth_distinct_resource_requirement() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let proposal = intent(1, 101);
    let mut requests = Vec::with_capacity(17);
    let mut capacities = Vec::with_capacity(17);
    for marker in 1_u8..=17 {
        let requirement = PracticeResourceRequirementV2 {
            practice_id: PracticeIdV2::Strike,
            locator: PracticeResourceLocatorV2::Shared,
            resource_id: PracticeResourceIdV2::from_bytes([marker; 32]),
            unit_id: PracticeUnitIdV2::from_bytes([0x60; 32]),
            quantity: 1,
        };
        let request =
            derive_practice_resource_request_v2(&contract, &proposal, &requirement).unwrap();
        capacities.push(PracticeResourceCapacityV2 {
            owner: request.owner(),
            resource_id: request.resource_id(),
            unit_id: request.unit_id(),
            mode: PracticeResourceAllocationModeV2::DivisibleProRata,
            available: 1,
        });
        requests.push(request);
    }

    assert_eq!(
        allocate_practice_resources_v2(&contract, &requests, &capacities),
        Err(PracticeResourceV2Error::ResourceRequestsPerIntentLimit)
    );
}

#[test]
fn allocator_refuses_duplicate_requests_capacities_and_missing_capacity() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let request =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(1))
            .unwrap();
    let capacity = PracticeResourceCapacityV2 {
        owner: request.owner(),
        resource_id: request.resource_id(),
        unit_id: request.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 1,
    };

    assert_eq!(
        allocate_practice_resources_v2(
            &contract,
            &[request.clone(), request.clone()],
            std::slice::from_ref(&capacity),
        ),
        Err(PracticeResourceV2Error::ResourceRequestDuplicate)
    );
    assert_eq!(
        allocate_practice_resources_v2(&contract, &[], &[capacity.clone(), capacity]),
        Err(PracticeResourceV2Error::ResourceCapacityDuplicate)
    );
    assert_eq!(
        allocate_practice_resources_v2(&contract, &[request], &[]),
        Err(PracticeResourceV2Error::ResourceCapacityMissing)
    );
}

#[test]
fn allocator_refuses_total_maximum_plus_one_before_nested_work() {
    let contract = PracticeResourceAllocationContractV2::conservation_first();
    let request =
        derive_practice_resource_request_v2(&contract, &intent(1, 101), &shared_requirement(1))
            .unwrap();
    let capacity = PracticeResourceCapacityV2 {
        owner: request.owner(),
        resource_id: request.resource_id(),
        unit_id: request.unit_id(),
        mode: PracticeResourceAllocationModeV2::DivisibleProRata,
        available: 1,
    };

    let too_many_requests = vec![request; MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1];
    assert_eq!(
        allocate_practice_resources_v2(&contract, &too_many_requests, &[]),
        Err(PracticeResourceV2Error::ResourceRequestLimit)
    );

    let too_many_capacities = vec![capacity; MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1];
    assert_eq!(
        allocate_practice_resources_v2(&contract, &[], &too_many_capacities),
        Err(PracticeResourceV2Error::ResourceCapacityLimit)
    );
}

#[test]
fn resource_refusal_codes_are_closed_and_language_neutral() {
    for code in 1_u16..=22 {
        let decoded = PracticeResourceV2Error::try_from(code).unwrap();
        assert_eq!(u16::from(decoded), code);
    }
    assert!(PracticeResourceV2Error::try_from(0_u16).is_err());
    assert!(PracticeResourceV2Error::try_from(23_u16).is_err());
}
