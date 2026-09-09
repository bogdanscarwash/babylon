use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::tick_content_hash::{
    OrderedPracticeActionBatchDigestV1, PreparedEnvironmentDigestV1, RefDigestV1,
    StableWorldDigestV1, TickContentPartsV1, TickContentPreimageV1, TickPayloadDigestV1,
};
use babylon_kernel::ContentDigest;

const SESSION: &str = "s:PER-60/alpha";
const RESOLVE_TICK: u64 = 0x0102_0304_0506_0708;
const SEED: i64 = -0x0102_0304_0506_0708;
const SECTION_TAG_OFFSETS: [usize; 10] = [25, 46, 55, 72, 141, 178, 215, 252, 289, 326];
const LAYOUT_OFFSETS: [usize; 11] = [21, 26, 56, 60, 73, 142, 179, 216, 253, 290, 327];

fn compose(
    session: &ReplaySessionIdV1,
    resolve_tick: u64,
    seed: i64,
    content_bytes: [u8; 2],
    digest_bytes: [u8; 6],
) -> TickContentPreimageV1 {
    let content = ContentDigest {
        defines_hash: [content_bytes[0]; 32],
        rules_hash: [content_bytes[1]; 32],
    };
    TickContentPreimageV1::compose(&TickContentPartsV1 {
        session,
        resolve_tick,
        seed: ReplaySeed::new(seed),
        content: &content,
        reference: RefDigestV1::from_bytes([digest_bytes[0]; 32]),
        prepared: PreparedEnvironmentDigestV1::from_bytes([digest_bytes[1]; 32]),
        prior_world: StableWorldDigestV1::from_bytes([digest_bytes[2]; 32]),
        actions: OrderedPracticeActionBatchDigestV1::from_bytes([digest_bytes[3]; 32]),
        result_world: StableWorldDigestV1::from_bytes([digest_bytes[4]; 32]),
        payload: TickPayloadDigestV1::from_bytes([digest_bytes[5]; 32]),
    })
    .expect("the bounded asymmetric vector composes")
}

fn asymmetric_preimage(session: &ReplaySessionIdV1) -> TickContentPreimageV1 {
    compose(
        session,
        RESOLVE_TICK,
        SEED,
        [0x11, 0x22],
        [0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
    )
}

fn assert_digest_section(bytes: &[u8], tag_offset: usize, tag: u8, fill: u8) {
    assert_eq!(bytes[tag_offset], tag);
    assert_eq!(&bytes[tag_offset + 1..tag_offset + 5], &1_u32.to_be_bytes());
    assert_eq!(&bytes[tag_offset + 5..tag_offset + 37], &[fill; 32]);
}

#[test]
fn outer_composer_matches_the_exact_ten_section_vector() {
    let session = ReplaySessionIdV1::try_from(SESSION).expect("fixture session is valid");
    let preimage = asymmetric_preimage(&session);

    assert_eq!(preimage.as_bytes().len(), 349 + SESSION.len());
    assert_eq!(&preimage.as_bytes()[..21], b"babylon.tick-content\0");
    assert_eq!(&preimage.as_bytes()[21..25], &1_u32.to_be_bytes());
    assert_eq!(
        SECTION_TAG_OFFSETS.map(|offset| preimage.as_bytes()[offset]),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );
    assert_eq!(&preimage.as_bytes()[26..30], &1_u32.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[30..32], &14_u16.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[32..46], SESSION.as_bytes());
    assert_eq!(&preimage.as_bytes()[47..55], &RESOLVE_TICK.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[56..60], &1_u32.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[60..64], &2_u32.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[64..72], &SEED.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[73..77], &1_u32.to_be_bytes());
    assert_eq!(&preimage.as_bytes()[77..109], &[0x11; 32]);
    assert_eq!(&preimage.as_bytes()[109..141], &[0x22; 32]);
    assert_digest_section(preimage.as_bytes(), 141, 0x05, 0x33);
    assert_digest_section(preimage.as_bytes(), 178, 0x06, 0x44);
    assert_digest_section(preimage.as_bytes(), 215, 0x07, 0x55);
    assert_digest_section(preimage.as_bytes(), 252, 0x08, 0x66);
    assert_digest_section(preimage.as_bytes(), 289, 0x09, 0x77);
    assert_digest_section(preimage.as_bytes(), 326, 0x0a, 0x88);
    assert_eq!(
        preimage.digest().to_hex(),
        "6d3845ff4cfe053fbeea7b70eb5ad6b0bebbf5f6d40d728b1cd4857d672ca2a7"
    );
}

#[test]
fn every_outer_input_changes_the_tick_content_hash() {
    let session = ReplaySessionIdV1::try_from(SESSION).expect("fixture session is valid");
    let other_session = ReplaySessionIdV1::try_from("s:PER-60/beta").expect("valid mutation");
    let baseline = asymmetric_preimage(&session).digest();
    let mutations = [
        asymmetric_preimage(&other_session),
        compose(
            &session,
            RESOLVE_TICK + 1,
            SEED,
            [0x11, 0x22],
            [0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED + 1,
            [0x11, 0x22],
            [0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x12, 0x22],
            [0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x23],
            [0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x22],
            [0x34, 0x44, 0x55, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x22],
            [0x33, 0x45, 0x55, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x22],
            [0x33, 0x44, 0x56, 0x66, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x22],
            [0x33, 0x44, 0x55, 0x67, 0x77, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x22],
            [0x33, 0x44, 0x55, 0x66, 0x78, 0x88],
        ),
        compose(
            &session,
            RESOLVE_TICK,
            SEED,
            [0x11, 0x22],
            [0x33, 0x44, 0x55, 0x66, 0x77, 0x89],
        ),
    ];

    for mutation in &mutations {
        assert_ne!(mutation.digest(), baseline);
    }
}

#[test]
fn every_layout_version_is_hash_covered() {
    let session = ReplaySessionIdV1::try_from(SESSION).expect("fixture session is valid");
    let baseline = asymmetric_preimage(&session);

    for offset in LAYOUT_OFFSETS {
        let mut mutation = baseline.as_bytes().to_vec();
        mutation[offset + 3] ^= 1;
        assert_ne!(
            babylon_kernel::sha256_of(&mutation),
            *baseline.digest().as_bytes()
        );
    }
}
