use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    decode_input_authority, decode_input_authority_ledger_v2, decode_input_authority_v2,
    encode_input_authority, encode_input_authority_ledger_v2, encode_input_authority_v2,
    input_authority_ledger_v2_digest, input_authority_v2_digest, CampaignIdV2, InputAuthorityIdV2,
    PracticeAuthorityKindV1, PracticeAuthorityKindV2, PracticeAuthorityV2Error,
    PracticeContractError, PracticeInputAuthorityV1, PracticeInputAuthorityV2,
    PRACTICE_INPUT_AUTHORITY_V2_SOURCE_SHA256,
};
use serde_json::Value;

const SCHEMA: &[u8] = include_bytes!("../../../../contracts/practice_input_authority_v2.yaml");
const VECTORS: &str =
    include_str!("../../../../contracts/practice_input_authority_v2_vectors.jsonl");

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

fn row_from_vector(data: &Value) -> PracticeInputAuthorityV2 {
    let data = data.as_object().expect("row data is an object");
    PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes(
            hex_bytes(data["campaign_id_hex"].as_str().unwrap())
                .try_into()
                .unwrap(),
        ),
        authority_kind: PracticeAuthorityKindV2::try_from(
            u8::try_from(data["authority_kind"].as_u64().unwrap()).unwrap(),
        )
        .unwrap(),
        input_authority_id: InputAuthorityIdV2::from_bytes(
            hex_bytes(data["input_authority_id_hex"].as_str().unwrap())
                .try_into()
                .unwrap(),
        ),
        actor_org_id: actor_id(data["actor_org_id"].as_u64().unwrap()),
        effective_from_tick: data["effective_from_tick"].as_u64().unwrap(),
        effective_through_tick_exclusive: data["effective_through_tick_exclusive"]
            .as_u64()
            .unwrap(),
        decision_content_digest: hex_bytes(data["decision_content_digest_hex"].as_str().unwrap())
            .try_into()
            .unwrap(),
    }
}

#[test]
fn authority_v2_contract_schema_and_vectors_drive_the_rust_boundary() {
    assert_eq!(
        babylon_kernel::sha256_of(SCHEMA),
        PRACTICE_INPUT_AUTHORITY_V2_SOURCE_SHA256
    );
    assert!(VECTORS.len() <= 65_536);
    let cases: Vec<Value> = VECTORS
        .lines()
        .take(65)
        .map(|line| {
            assert!(line.len() <= 4_096);
            serde_json::from_str(line).unwrap()
        })
        .collect();
    assert_eq!(cases.len(), 8);

    let row_case = cases
        .iter()
        .find(|case| case["case_id"] == "authority-player")
        .unwrap();
    let row = row_from_vector(&row_case["data"]);
    let canonical = hex_bytes(row_case["data"]["canonical_hex"].as_str().unwrap());
    let manifest = cases
        .iter()
        .find(|case| case["case_id"] == "manifest")
        .unwrap();
    assert_eq!(
        manifest["data"]["row_canonical_bytes"].as_u64().unwrap(),
        u64::try_from(canonical.len()).unwrap()
    );
    assert_eq!(encode_input_authority_v2(&row).unwrap(), canonical);
    assert_eq!(decode_input_authority_v2(&canonical).unwrap(), row);
    assert_eq!(
        input_authority_v2_digest(&row).unwrap().to_vec(),
        hex_bytes(row_case["data"]["digest_hex"].as_str().unwrap())
    );

    let ledger_case = cases
        .iter()
        .find(|case| case["case_id"] == "ledger-one")
        .unwrap();
    let ledger_bytes = hex_bytes(ledger_case["data"]["canonical_hex"].as_str().unwrap());
    let ledger = decode_input_authority_ledger_v2(&ledger_bytes).unwrap();
    assert_eq!(ledger.rows, vec![row]);
    assert_eq!(
        encode_input_authority_ledger_v2(&ledger).unwrap(),
        ledger_bytes
    );
    assert_eq!(
        input_authority_ledger_v2_digest(&ledger).unwrap().to_vec(),
        hex_bytes(ledger_case["data"]["digest_hex"].as_str().unwrap())
    );

    for case in cases.iter().filter(|case| case["kind"] == "invalid_wire") {
        let payload = hex_bytes(case["data"]["payload_hex"].as_str().unwrap());
        let expected = PracticeAuthorityV2Error::try_from(
            u16::try_from(case["data"]["error"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        let actual = match case["data"]["codec"].as_str().unwrap() {
            "authority" => decode_input_authority_v2(&payload).map(|_| ()),
            "ledger" => decode_input_authority_ledger_v2(&payload).map(|_| ()),
            unknown => panic!("unknown V2 vector codec {unknown}"),
        };
        assert_eq!(actual, Err(expected));
    }
}

#[test]
fn v1_and_v2_authority_domains_refuse_cross_version_decoding() {
    let v1 = PracticeInputAuthorityV1 {
        schema_version: 1,
        authority_kind: PracticeAuthorityKindV1::PlayerSeat,
        actor_org_id: 7,
        producer_content_digest: [0x11; 32],
    };
    let v2 = PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKindV2::PlayerSeat,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
        actor_org_id: actor_id(7),
        effective_from_tick: 10,
        effective_through_tick_exclusive: 20,
        decision_content_digest: [0x30; 32],
    };

    assert_eq!(
        decode_input_authority_v2(&encode_input_authority(&v1).unwrap()),
        Err(PracticeAuthorityV2Error::AuthorityDomain)
    );
    assert_eq!(
        decode_input_authority(&encode_input_authority_v2(&v2).unwrap()),
        Err(PracticeContractError::PracticeDomain)
    );
}
