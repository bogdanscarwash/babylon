use babylon_evidence::{
    bind_synthetic_driver, parse_synthetic_driver_contract, Digest32, SfsPreregistrationV1,
    SyntheticDriverContractError,
};
use babylon_kernel::sha256_of;
use babylon_practice_contract::PracticeIdV1;

const DRIVER: &[u8] = include_bytes!("fixtures/sfs_synthetic_driver_contract_v1.txt");
const DRIVER_VECTORS: &str = include_str!("fixtures/sfs_synthetic_driver_v1.txt");
const DRIVER_SOURCE: &[u8] = include_bytes!("../src/driver.rs");

fn digest(tag: u8) -> Digest32 {
    let mut bytes = [0_u8; 32];
    bytes[0] = tag;
    Digest32::from_bytes(bytes)
}

fn preregistration(driver: Digest32) -> SfsPreregistrationV1 {
    SfsPreregistrationV1::new(
        1,
        digest(1),
        digest(2),
        digest(3),
        driver,
        digest(4),
        digest(5),
        10,
        2,
        3,
        PracticeIdV1::Organize,
        digest(6),
        3,
        digest(7),
    )
    .unwrap()
}

#[test]
fn exact_contract_binds_an_opaque_handle() {
    let contract = parse_synthetic_driver_contract(DRIVER).unwrap();
    let prereg = preregistration(contract.manifest_digest());
    assert!(bind_synthetic_driver(&prereg, &contract).is_ok());
}

#[test]
fn source_predicate_and_preregistration_mutations_refuse() {
    let contract = parse_synthetic_driver_contract(DRIVER).unwrap();
    let changed_predicate = String::from_utf8(DRIVER.to_vec())
        .unwrap()
        .replace("candidate-projection|1", "candidate-projection|2");
    assert!(matches!(
        parse_synthetic_driver_contract(changed_predicate.as_bytes()),
        Err(SyntheticDriverContractError::ManifestMalformed { row: 2 })
    ));
    assert!(matches!(
        bind_synthetic_driver(&preregistration(digest(99)), &contract),
        Err(SyntheticDriverContractError::PreregistrationDigestMismatch)
    ));

    let mut changed_source = DRIVER.to_vec();
    let last_hex = changed_source.len() - 2;
    changed_source[last_hex] = if changed_source[last_hex] == b'0' {
        b'1'
    } else {
        b'0'
    };
    assert_eq!(
        parse_synthetic_driver_contract(&changed_source),
        Err(SyntheticDriverContractError::SourceDigestMismatch)
    );
}

#[test]
fn seven_row_schema_refuses_missing_extra_reordered_and_renamed_rows() {
    assert_eq!(
        parse_synthetic_driver_contract(&vec![b'x'; 4_097]),
        Err(SyntheticDriverContractError::ManifestByteLimit { actual: 4_097 })
    );
    let text = String::from_utf8(DRIVER.to_vec()).unwrap();
    let rows = text.lines().collect::<Vec<_>>();
    let missing = format!("{}\n", rows[..6].join("\n"));
    assert!(matches!(
        parse_synthetic_driver_contract(missing.as_bytes()),
        Err(SyntheticDriverContractError::ManifestMalformed { .. })
    ));
    let extra = format!("{text}extra|row\n");
    assert_eq!(
        parse_synthetic_driver_contract(extra.as_bytes()),
        Err(SyntheticDriverContractError::ManifestMalformed { row: 8 })
    );
    let mut reordered = rows.clone();
    reordered.swap(1, 2);
    let reordered = format!("{}\n", reordered.join("\n"));
    assert_eq!(
        parse_synthetic_driver_contract(reordered.as_bytes()),
        Err(SyntheticDriverContractError::ManifestMalformed { row: 2 })
    );
    let renamed = text.replace("candidate-projection", "candidate-projector");
    assert_eq!(
        parse_synthetic_driver_contract(renamed.as_bytes()),
        Err(SyntheticDriverContractError::ManifestMalformed { row: 2 })
    );
}

#[test]
fn rust_recomputes_both_driver_vector_preimages_and_digests() {
    let contract = parse_synthetic_driver_contract(DRIVER).unwrap();
    let rows = DRIVER_VECTORS.lines().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    for row in rows {
        let parts = row.split('|').collect::<Vec<_>>();
        assert_eq!(parts.len(), 4);
        let preimage = hex_bytes(parts[2]);
        let expected = domain_digest(parts[1].as_bytes(), &preimage);
        assert_eq!(expected.to_hex(), parts[3]);
        match parts[0] {
            "driver-contract" => {
                assert_eq!(preimage, DRIVER);
                assert_eq!(expected, contract.manifest_digest());
            }
            "driver-source" => {
                assert_eq!(preimage, DRIVER_SOURCE);
                assert_eq!(expected, contract.source_digest());
            }
            _ => panic!("unexpected driver vector label"),
        }
    }
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn domain_digest(domain: &[u8], payload: &[u8]) -> Digest32 {
    Digest32::from_bytes(sha256_of(&[domain, b"\0", payload].concat()))
}
