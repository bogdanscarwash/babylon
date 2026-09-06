use super::*;

fn campaign() -> CampaignId {
    CampaignId::from_uuid(uuid::Uuid::from_bytes([1; 16]))
}

fn original() -> AdoptionSeed {
    AdoptionSeed {
        floor: ArchiveReadScopeV2::committed(campaign(), 5, [2; 32]).unwrap(),
        processed: 3,
        count: 1,
        heads_digest: [
            0x49, 0x5e, 0x01, 0x18, 0xda, 0xcf, 0xa8, 0x3b, 0xf0, 0x2b, 0xc7, 0x78, 0xf8, 0x22,
            0xbf, 0xe5, 0x8f, 0x5b, 0x3b, 0xa0, 0xc0, 0x8d, 0xb8, 0x51, 0x0d, 0xa8, 0x93, 0x4d,
            0xdf, 0x4c, 0x4d, 0xc1,
        ],
    }
}

#[test]
fn adoption_digest_has_independent_original_floor_and_head_set_vector() {
    // Independently encoded with Python hashlib + big-endian struct.pack.
    assert_eq!(
        original().digest().unwrap(),
        [
            0x94, 0x60, 0x63, 0xb9, 0x38, 0x50, 0x14, 0xa8, 0xc6, 0x40, 0x77, 0xbb, 0x96, 0x56,
            0x56, 0x29, 0x8a, 0x14, 0x40, 0xb2, 0x9d, 0x4a, 0x79, 0xf6, 0xf1, 0x4c, 0xbb, 0x5c,
            0x3a, 0x01, 0x5f, 0xaa,
        ]
    );
    let seed = AdoptionSeed {
        floor: ArchiveReadScopeV2::foundation(campaign()),
        processed: 0,
        count: 0,
        heads_digest: Sha256::digest([]).into(),
    };
    assert_eq!(
        seed.digest().unwrap(),
        [
            0xf6, 0x6b, 0xdd, 0xf4, 0x84, 0xd0, 0xe1, 0x80, 0x69, 0xa1, 0xd9, 0x9f, 0xe8, 0x98,
            0xac, 0x61, 0x87, 0xaf, 0x79, 0x9c, 0x72, 0x6c, 0x59, 0x0f, 0x0f, 0xa7, 0x4d, 0xaa,
            0x73, 0x01, 0xe5, 0x1f,
        ]
    );
}

#[test]
fn every_original_adoption_component_is_bound() {
    let expected = original().digest().unwrap();
    for change in 0..6 {
        let mut seed = original();
        match change {
            0 => {
                seed.floor = ArchiveReadScopeV2::committed(
                    CampaignId::from_uuid(uuid::Uuid::from_bytes([9; 16])),
                    5,
                    [2; 32],
                )
                .unwrap();
            }
            1 => seed.floor = ArchiveReadScopeV2::committed(campaign(), 6, [2; 32]).unwrap(),
            2 => seed.floor = ArchiveReadScopeV2::committed(campaign(), 5, [9; 32]).unwrap(),
            3 => seed.processed = 4,
            4 => seed.count = 2,
            5 => seed.heads_digest[0] ^= 1,
            _ => unreachable!(),
        }
        assert_ne!(seed.digest().unwrap(), expected);
    }
}

#[test]
fn foundation_cannot_claim_adopted_pages_or_processing_and_prefix_cannot_exceed_floor() {
    let mut seed = original();
    seed.processed = 6;
    assert_eq!(
        seed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    seed.floor = ArchiveReadScopeV2::foundation(campaign());
    seed.processed = 0;
    assert_eq!(
        seed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    seed.count = 0;
    seed.processed = 1;
    assert_eq!(
        seed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}
