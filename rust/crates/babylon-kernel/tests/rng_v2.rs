use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1, RngDomainV2};
use babylon_kernel::{seed_for, seed_for_v2, KernelRng, SessionId};
use sha2::{Digest, Sha256};

const VECTOR_SEED: i64 = -72_623_859_790_382_856;
const VECTOR_TICK: u64 = 0x1020_3040_5060_7080;
const VECTOR_CARRIER: &[u8] = b"33:4:node|12:per60/vector|8:worker-A|2:-7";
const EXPECTED_PREIMAGE: &[u8] = b"babylon.rng-stream\0\0\0\0\x02\x01\xfe\xfd\xfc\xfb\xfa\xf9\xf8\xf8\x02\0\x10per60-replay/A9!\x03\x10\x20\x30\x40\x50\x60\x70\x80\x04\0\0\0\x15vitality/per60-vector\x05\0\0\0\x2933:4:node|12:per60/vector|8:worker-A|2:-7";
const EXPECTED_KEY: [u8; 32] = [
    0x4b, 0xcd, 0x7e, 0x8b, 0xe8, 0xad, 0x6a, 0x8f, 0x78, 0x00, 0xcb, 0x06, 0xf4, 0xf4, 0x7d, 0xb0,
    0x68, 0xdd, 0x07, 0x1c, 0x32, 0xa0, 0x0f, 0x1e, 0x15, 0x82, 0xc7, 0xba, 0x57, 0xcd, 0x5b, 0x4b,
];
const EXPECTED_DRAWS: [u64; 9] = [
    0xb36e_fa0f_f8a5_0391,
    0xfaa2_9b4d_2b52_78dc,
    0x6804_eec3_79fe_a105,
    0xf722_17da_d23c_28b2,
    0xc427_2e13_09ce_8ddb,
    0xd059_b020_2b47_fa5a,
    0x5316_07eb_b892_bac3,
    0x92a4_48a2_3497_ea02,
    0x73db_ed74_ddd5_0f51,
];
const EXPECTED_F64_BITS: u64 = 0x3fe6_6ddf_41ff_14a0;

#[test]
fn v2_preimage_key_and_stream_match_the_language_neutral_vector() {
    let session = ReplaySessionIdV1::try_from("per60-replay/A9!").unwrap();
    let seed = ReplaySeed::new(VECTOR_SEED);
    let domain = RngDomainV2::try_from("vitality/per60-vector").unwrap();

    let independent_preimage = [
        b"babylon.rng-stream\0".as_slice(),
        &2u32.to_be_bytes(),
        &[0x01],
        &VECTOR_SEED.to_be_bytes(),
        &[0x02],
        &16u16.to_be_bytes(),
        b"per60-replay/A9!",
        &[0x03],
        &VECTOR_TICK.to_be_bytes(),
        &[0x04],
        &21u32.to_be_bytes(),
        b"vitality/per60-vector",
        &[0x05],
        &41u32.to_be_bytes(),
        VECTOR_CARRIER,
    ]
    .concat();
    assert_eq!(independent_preimage, EXPECTED_PREIMAGE);
    assert_eq!(Sha256::digest(EXPECTED_PREIMAGE).as_slice(), EXPECTED_KEY);
    assert_eq!(
        seed_for_v2(&session, seed, VECTOR_TICK, &domain, VECTOR_CARRIER).unwrap(),
        EXPECTED_KEY
    );

    let first_block = reference_chacha8_block(EXPECTED_KEY, 0);
    let second_block = reference_chacha8_block(EXPECTED_KEY, 1);
    let reference_draws = draws_from_blocks(&first_block, &second_block);
    assert_eq!(reference_draws, EXPECTED_DRAWS);

    let mut rng =
        KernelRng::for_carrier_v2(&session, seed, VECTOR_TICK, &domain, VECTOR_CARRIER).unwrap();
    let actual_draws = std::array::from_fn(|_| rng.next_u64());
    assert_eq!(actual_draws, EXPECTED_DRAWS);

    let mut fresh =
        KernelRng::for_carrier_v2(&session, seed, VECTOR_TICK, &domain, VECTOR_CARRIER).unwrap();
    assert_eq!(fresh.next_f64().to_bits(), EXPECTED_F64_BITS);
}

#[test]
fn v2_seed_derivation_changes_when_any_identity_component_changes() {
    let session = ReplaySessionIdV1::try_from("per60-replay/A9!").unwrap();
    let other_session = ReplaySessionIdV1::try_from("per60-replay/A9?").unwrap();
    let seed = ReplaySeed::new(VECTOR_SEED);
    let domain = RngDomainV2::try_from("vitality/per60-vector").unwrap();
    let other_domain = RngDomainV2::try_from("vitality/per60-vectos").unwrap();
    let original = seed_for_v2(&session, seed, VECTOR_TICK, &domain, VECTOR_CARRIER).unwrap();

    assert_ne!(
        seed_for_v2(
            &session,
            ReplaySeed::new(VECTOR_SEED + 1),
            VECTOR_TICK,
            &domain,
            VECTOR_CARRIER,
        )
        .unwrap(),
        original
    );
    assert_ne!(
        seed_for_v2(&other_session, seed, VECTOR_TICK, &domain, VECTOR_CARRIER).unwrap(),
        original
    );
    assert_ne!(
        seed_for_v2(&session, seed, VECTOR_TICK + 1, &domain, VECTOR_CARRIER).unwrap(),
        original
    );
    assert_ne!(
        seed_for_v2(&session, seed, VECTOR_TICK, &other_domain, VECTOR_CARRIER).unwrap(),
        original
    );
    assert_ne!(
        seed_for_v2(
            &session,
            seed,
            VECTOR_TICK,
            &domain,
            b"33:4:node|12:per60/vector|8:worker-A|2:-8",
        )
        .unwrap(),
        original
    );
}

#[test]
fn v1_conformance_vector_remains_byte_for_byte_unchanged() {
    let session = SessionId::new("conformance").unwrap();
    let mut rng = KernelRng::for_carrier(&session, 1, "conformance-domain", "carrier-0");

    assert_eq!(
        [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ],
        [
            0x6774_721d_2209_092f,
            0x6d42_2bc9_af84_28f1,
            0x0ce2_91ab_fcb1_1e7a,
            0xdd11_9629_7249_5117,
        ]
    );
    assert_ne!(
        seed_for(&session, 1, "conformance-domain", "carrier-0"),
        seed_for(&session, 1, "conformance-domain", "carrier-1")
    );
}

fn draws_from_blocks(first: &[u32; 16], second: &[u32; 16]) -> [u64; 9] {
    let mut words = [0u32; 32];
    words[..16].copy_from_slice(first);
    words[16..].copy_from_slice(second);
    std::array::from_fn(|index| {
        let word_index = index * 2;
        u64::from(words[word_index]) | (u64::from(words[word_index + 1]) << 32)
    })
}

fn reference_chacha8_block(key: [u8; 32], counter: u64) -> [u32; 16] {
    let counter_bytes = counter.to_le_bytes();
    let mut state = [
        0x6170_7865,
        0x3320_646e,
        0x7962_2d32,
        0x6b20_6574,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        u32::from_le_bytes([
            counter_bytes[0],
            counter_bytes[1],
            counter_bytes[2],
            counter_bytes[3],
        ]),
        u32::from_le_bytes([
            counter_bytes[4],
            counter_bytes[5],
            counter_bytes[6],
            counter_bytes[7],
        ]),
        0,
        0,
    ];
    for index in 0..8 {
        let offset = index * 4;
        state[index + 4] = u32::from_le_bytes([
            key[offset],
            key[offset + 1],
            key[offset + 2],
            key[offset + 3],
        ]);
    }

    let original = state;
    for _ in 0..4 {
        quarter_round(&mut state, 0, 4, 8, 12);
        quarter_round(&mut state, 1, 5, 9, 13);
        quarter_round(&mut state, 2, 6, 10, 14);
        quarter_round(&mut state, 3, 7, 11, 15);
        quarter_round(&mut state, 0, 5, 10, 15);
        quarter_round(&mut state, 1, 6, 11, 12);
        quarter_round(&mut state, 2, 7, 8, 13);
        quarter_round(&mut state, 3, 4, 9, 14);
    }
    for index in 0..16 {
        state[index] = state[index].wrapping_add(original[index]);
    }
    state
}

fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}
