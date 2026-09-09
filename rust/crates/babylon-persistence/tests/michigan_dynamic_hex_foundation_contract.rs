//! Executable contract for the pinned Michigan Dynamic-Hex Foundation V1.

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::{sha256_of, ContentDigest, H3CellId};
use babylon_persistence::{
    decode_michigan_dynamic_hex_foundation_v1, michigan_dynamic_hex_foundation_fixture_parts_v1,
    michigan_dynamic_hex_foundation_v1, MichiganDynamicHexFoundationDecodeErrorV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::h3_runtime::{
    MichiganDynamicHexFoundationErrorV1, MichiganDynamicHexFoundationRowV1,
    MichiganDynamicHexFoundationV1, MichiganDynamicHexValueBitsV1, MichiganDynamicHexValuesV1,
    MichiganH3R8ChildParentV1,
};
use babylon_tick::material_state::MaterialStateV1;
use babylon_tick::replay_session::ReplayTickSession;

const DOMAIN: &[u8] = b"babylon.michigan-dynamic-hex-foundation.v1\0";
const LAYOUT: u32 = 1;
const SOURCE_R7_DIGEST: &str = "7f8d126ee81356a60605013b4b1c23942a77a4b2d6f890125d6c938dae70228b";
const BASE_REFERENCE_COHORT_DIGEST: &str =
    "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161";
const R8_SECTION_DIGEST: &str = "b5ebf405140f6f79ddbc44fa1005b195bed0bc28e0eacf2d8e1697cd9c839491";
const REFERENCE_BUNDLE_DIGEST: &str =
    "84bbffa9b2388aa168c065e710a61313fbd46522d2022b628f0919ecffec9831";
const ARTIFACT_DIGEST: &str = "81ee8f8abbee6727655d52c6d56a6f2967af9dfdf01da53dd593da8339d650a4";
const DYNAMIC_BATCH_DIGEST: &str =
    "8138ea071edc5823bd7b7bab89e4a51387917f5646ee8f82eb14b233d2613e70";
const ROW_COUNT: usize = 45_572;
const R8_ROW_COUNT: usize = 319_004;
const H3_RUNTIME_SOURCE: &str = include_str!("../../babylon-tick/src/h3_runtime.rs");
const MATERIAL_STATE_SOURCE: &str = include_str!("../../babylon-tick/src/material_state.rs");
const REPLAY_SCENARIO: &str = r"
(scenario michigan/foundation-replay
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/witness int extensive)
  (node witness NodeType/SOCIAL_CLASS (social-class/witness 1)))
";
const REPLAY_RULE: &str = r#"
(rule vitality/michigan-foundation-replay
  :role mechanic
  :evidence derived
  :material-basis "checked foundation replay projection contract"
  :fuel 16
  (bindings (binding witness :field social-class/witness))
  (when #t)
  (effects (update-node self social-class/witness (add 0))))
"#;

fn digest(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);
    let mut output = [0_u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap();
    }
    output
}

fn value_input(bits: [u64; 9]) -> MichiganDynamicHexValueBitsV1 {
    MichiganDynamicHexValueBitsV1 {
        c: bits[0],
        v: bits[1],
        s: bits[2],
        k: bits[3],
        biocapacity_stock: bits[4],
        energy_stock: bits[5],
        raw_material_stock: bits[6],
        internet_access_pct: bits[7],
        surveillance_coupling: bits[8],
    }
}

fn replay_content() -> ContentDigest {
    let (_, rules) = split_content(REPLAY_RULE).unwrap();
    let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    ContentDigest {
        defines_hash: [0x71; 32],
        rules_hash: rules_hash_of(&forms).unwrap(),
    }
}

#[test]
fn tick_owned_rows_refuse_values_outside_the_existing_dynamic_hex_domains() {
    let valid = [0.0_f64.to_bits(); 9];

    for lane in 0..9 {
        let mut bits = valid;
        bits[lane] = f64::NAN.to_bits();
        assert_eq!(
            MichiganDynamicHexValuesV1::try_new(value_input(bits)),
            Err(MichiganDynamicHexFoundationErrorV1::NonFiniteValue { lane })
        );

        let mut bits = valid;
        bits[lane] = (-0.0_f64).to_bits();
        assert_eq!(
            MichiganDynamicHexValuesV1::try_new(value_input(bits)),
            Err(MichiganDynamicHexFoundationErrorV1::NegativeZero { lane })
        );
    }
    for lane in 0..7 {
        let mut bits = valid;
        bits[lane] = (-1.0_f64).to_bits();
        assert_eq!(
            MichiganDynamicHexValuesV1::try_new(value_input(bits)),
            Err(MichiganDynamicHexFoundationErrorV1::NegativeValue { lane })
        );
    }
    for lane in 7..9 {
        for value in [-0.25_f64, 1.25_f64] {
            let mut bits = valid;
            bits[lane] = value.to_bits();
            assert_eq!(
                MichiganDynamicHexValuesV1::try_new(value_input(bits)),
                Err(MichiganDynamicHexFoundationErrorV1::UnitIntervalValue { lane })
            );
        }
    }
}

#[test]
fn tick_owned_foundation_cannot_be_minted_from_alternate_membership_or_values() {
    let foundation =
        michigan_dynamic_hex_foundation_v1().expect("governed fixture must construct once");

    let mut alternate_membership = foundation.rows().to_vec();
    let foreign_cell = H3CellId::try_from(0x0872_8308_28ff_ffff_u64).unwrap();
    let retained_values = *alternate_membership[0].values();
    alternate_membership[0] =
        MichiganDynamicHexFoundationRowV1::try_new(foreign_cell, retained_values).unwrap();
    alternate_membership.sort_unstable_by_key(|row| row.cell_id().as_u64());
    assert_eq!(
        MichiganDynamicHexFoundationV1::try_new(
            alternate_membership,
            foundation.r8_child_parent_rows().to_vec(),
        ),
        Err(MichiganDynamicHexFoundationErrorV1::SourceR7Digest)
    );

    let mut alternate_value = foundation.rows().to_vec();
    let mut bits = alternate_value[0].value_bits();
    bits[0] = (f64::from_bits(bits[0]) + 1.0).to_bits();
    alternate_value[0] = MichiganDynamicHexFoundationRowV1::try_new(
        alternate_value[0].cell_id(),
        MichiganDynamicHexValuesV1::try_new(value_input(bits)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        MichiganDynamicHexFoundationV1::try_new(
            alternate_value,
            foundation.r8_child_parent_rows().to_vec(),
        ),
        Err(MichiganDynamicHexFoundationErrorV1::ArtifactDigest)
    );

    let mut incomplete_r8 = foundation.r8_child_parent_rows().to_vec();
    incomplete_r8.pop();
    assert_eq!(
        MichiganDynamicHexFoundationV1::try_new(foundation.rows().to_vec(), incomplete_r8),
        Err(MichiganDynamicHexFoundationErrorV1::R8RowCount {
            actual: R8_ROW_COUNT - 1,
        })
    );

    let first = foundation.r8_child_parent_rows()[0];
    let wrong_parent = foundation.rows()[1].cell_id();
    assert_eq!(
        MichiganH3R8ChildParentV1::try_new(first.child_cell_id(), wrong_parent),
        Err(MichiganDynamicHexFoundationErrorV1::R8ParentMismatch {
            child: first.child_cell_id(),
            parent: wrong_parent,
        })
    );

    let foreign_parent = H3CellId::try_from(0x0872_8308_28ff_ffff_u64).unwrap();
    let foreign_child = foreign_parent.immediate_children().unwrap().as_slice()[0];
    let mut wrong_complete_set = foundation.r8_child_parent_rows().to_vec();
    wrong_complete_set[0] =
        MichiganH3R8ChildParentV1::try_new(foreign_child, foreign_parent).unwrap();
    wrong_complete_set.sort_unstable_by_key(MichiganH3R8ChildParentV1::child_cell_id);
    assert!(matches!(
        MichiganDynamicHexFoundationV1::try_new(foundation.rows().to_vec(), wrong_complete_set),
        Err(MichiganDynamicHexFoundationErrorV1::R8CoverageMismatch { .. })
    ));
}

#[test]
fn dynamic_runtime_construction_is_private_to_material_state() {
    assert!(
        !H3_RUNTIME_SOURCE.contains("DynamicHexRuntime"),
        "the public foundation module must not expose the private runtime authority"
    );
    assert!(MATERIAL_STATE_SOURCE.contains(concat!(
        "struct DynamicHexRuntimeRowV1 {\n",
        "    cell_id: H3CellId,\n",
        "    value_bits: [u64; 9],\n",
        "}",
    )));
    assert!(MATERIAL_STATE_SOURCE.contains(concat!(
        "struct DynamicHexRuntimeV1 {\n",
        "    rows: Vec<DynamicHexRuntimeRowV1>,\n",
        "    source_r7_digest: [u8; 32],\n",
        "    reference_bundle_digest: [u8; 32],\n",
        "    artifact_sha256: [u8; 32],\n",
        "}",
    )));
    assert_eq!(
        MATERIAL_STATE_SOURCE
            .matches("DynamicHexRuntimeV1 {")
            .count(),
        2,
        "only the private declaration and type-owned impl may name the aggregate runtime"
    );
    let runtime_impl_start = MATERIAL_STATE_SOURCE
        .find("impl DynamicHexRuntimeV1 {")
        .expect("private runtime impl must exist");
    let runtime_impl_end = MATERIAL_STATE_SOURCE[runtime_impl_start..]
        .find("struct MaterialWriter")
        .map(|offset| runtime_impl_start + offset)
        .expect("runtime impl must end before the material writer");
    let runtime_impl = &MATERIAL_STATE_SOURCE[runtime_impl_start..runtime_impl_end];
    assert_eq!(
        MATERIAL_STATE_SOURCE
            .matches("DynamicHexRuntimeRowV1 {")
            .count(),
        4,
        "one private declaration and three type-owned row constructions are governed"
    );
    assert_eq!(
        runtime_impl.matches("DynamicHexRuntimeRowV1 {").count(),
        3,
        "foundation construction, detachment, and the test-only fixture own every row literal"
    );
}

fn assert_row(
    row: &MichiganDynamicHexFoundationRowV1,
    expected_cell: u64,
    expected_bits: [u64; 9],
) {
    assert_eq!(row.cell_id(), H3CellId::try_from(expected_cell).unwrap());
    assert_eq!(row.value_bits(), expected_bits);
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one consensus test binds all three checked archive representations"
)]
fn static_fixture_decodes_the_exact_three_archive_consensus() {
    let parts = michigan_dynamic_hex_foundation_fixture_parts_v1();
    assert_eq!(
        parts.map(<[u8]>::len),
        [
            1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000,
            750_055,
        ]
    );
    assert!(parts.iter().all(|part| part.len() <= 1_000_000));

    let foundation: &MichiganDynamicHexFoundationV1 =
        michigan_dynamic_hex_foundation_v1().expect("checked-in fixture must decode once");
    assert_eq!(foundation.domain(), DOMAIN);
    assert_eq!(foundation.layout(), LAYOUT);
    assert_eq!(foundation.source_r7_digest(), digest(SOURCE_R7_DIGEST));
    assert_eq!(
        foundation.base_reference_cohort_digest(),
        digest(BASE_REFERENCE_COHORT_DIGEST)
    );
    assert_eq!(foundation.r8_section_digest(), digest(R8_SECTION_DIGEST));
    assert_eq!(
        foundation.reference_bundle_digest(),
        digest(REFERENCE_BUNDLE_DIGEST)
    );
    assert_eq!(foundation.rows().len(), ROW_COUNT);
    assert_eq!(foundation.r8_child_parent_rows().len(), R8_ROW_COUNT);
    assert!(
        foundation
            .rows()
            .windows(2)
            .all(|rows| rows[0].cell_id().as_u64() < rows[1].cell_id().as_u64()),
        "foundation rows must be strictly numeric-H3 ordered"
    );
    assert!(foundation
        .r8_child_parent_rows()
        .windows(2)
        .all(|rows| rows[0].child_cell_id().as_u64() < rows[1].child_cell_id().as_u64()));
    assert!(foundation.r8_child_parent_rows().iter().all(|row| {
        row.child_cell_id().resolution() == 8
            && row.parent_r7_cell_id().resolution() == 7
            && row.child_cell_id().immediate_parent() == Some(row.parent_r7_cell_id())
            && foundation
                .rows()
                .binary_search_by_key(
                    &row.parent_r7_cell_id(),
                    MichiganDynamicHexFoundationRowV1::cell_id,
                )
                .is_ok()
    }));
    let audited_child = H3CellId::try_from(0x0882_6648_001f_ffff_u64).unwrap();
    let audited_parent = H3CellId::try_from(0x0872_6648_00ff_ffff_u64).unwrap();
    assert!(foundation.r8_child_parent_rows().iter().any(|row| {
        row.child_cell_id() == audited_child && row.parent_r7_cell_id() == audited_parent
    }));

    assert_row(
        &foundation.rows()[0],
        0x0872_6648_00ff_ffff,
        [
            0x40f4_5362_48e6_f741,
            0x40ec_5ddc_8b3c_8b09,
            0x40f7_4d3a_4c30_d6c5,
            0x4176_d7a8_b8da_353d,
            0x408f_4000_0000_0000,
            0x4085_1907_62bb_ca34,
            0x4085_34ea_cdf6_be8b,
            0x3ff0_0000_0000_0000,
            0x3fe6_b0df_6b0d_f6b0,
        ],
    );
    assert_row(
        &foundation.rows()[22_786],
        0x0872_74dd_49ff_ffff,
        [
            0x4105_a070_7811_535e,
            0x4102_5b39_6fc1_f39c,
            0x4107_f3af_ebf4_845c,
            0x4189_c816_33e3_311b,
            0x408f_4000_0000_0000,
            0x4089_9d79_9c22_90a3,
            0x4075_e225_3b93_6436,
            0x3ff0_0000_0000_0000,
            0x3fe6_d5c8_0a8b_9b48,
        ],
    );
    assert_row(
        &foundation.rows()[ROW_COUNT - 1],
        0x0872_ab6d_b6ff_ffff,
        [
            0x40cd_95c5_6290_d7ed,
            0x40be_0572_783a_bd59,
            0x40ce_792a_4709_a028,
            0x414b_b77e_a3eb_c34a,
            0x408f_4000_0000_0000,
            0x4068_6cf2_4a42_ef09,
            0x4085_5200_0be7_ad4e,
            0x3ff0_0000_0000_0000,
            0x3fe6_8a53_5d61_a055,
        ],
    );

    let mut joined = Vec::with_capacity(foundation.canonical_bytes().len());
    for part in parts {
        joined.extend_from_slice(part);
    }
    assert_eq!(joined, foundation.canonical_bytes());
    assert_eq!(sha256_of(&joined), digest(ARTIFACT_DIGEST));
    assert_eq!(foundation.artifact_sha256(), digest(ARTIFACT_DIGEST));
}

#[test]
fn exact_foundation_projects_through_a_real_replay_session() {
    let mut artifact = Vec::new();
    for part in michigan_dynamic_hex_foundation_fixture_parts_v1() {
        artifact.extend_from_slice(part);
    }
    let foundation = decode_michigan_dynamic_hex_foundation_v1(&artifact)
        .expect("a fresh owned foundation must decode");
    let reference_digest = foundation.reference_bundle_digest();
    let expected_rows = foundation.rows().len();
    let expected_selected = [0, 22_786, ROW_COUNT - 1].map(|index| {
        (
            foundation.rows()[index].cell_id(),
            foundation.rows()[index].value_bits(),
        )
    });
    let material = MaterialStateV1::try_new(&foundation).unwrap();
    drop(foundation);
    drop(artifact);
    let replay_id = ReplaySessionIdV1::try_from("per281/michigan-foundation").unwrap();
    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        REPLAY_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(71),
        replay_content(),
        RefDigestV1::from_bytes(reference_digest),
        material,
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = session.advance(&mut sink, &actions).unwrap();
    let dynamic = report.material_state_rows().dynamic_hexes();

    assert_eq!(dynamic.source_count(), expected_rows);
    assert!(dynamic
        .rows()
        .windows(2)
        .all(|rows| rows[0].cell_id().as_u64() < rows[1].cell_id().as_u64()));
    for (index, (expected_cell, expected_bits)) in [0, 22_786, ROW_COUNT - 1]
        .into_iter()
        .zip(expected_selected)
    {
        assert_eq!(dynamic.rows()[index].cell_id(), expected_cell);
        assert_eq!(dynamic.rows()[index].value_bits(), expected_bits);
    }
    assert_eq!(
        dynamic.source_digest(),
        sha256_of(dynamic.canonical_bytes())
    );
    assert_eq!(dynamic.source_digest(), digest(DYNAMIC_BATCH_DIGEST));
}

#[test]
fn decoder_refuses_mutation_truncation_and_trailing_bytes() {
    let parts = michigan_dynamic_hex_foundation_fixture_parts_v1();
    let mut joined = Vec::new();
    for part in parts {
        joined.extend_from_slice(part);
    }

    let mut mutated = joined.clone();
    let last = mutated.len() - 1;
    mutated[last] ^= 1;
    assert_eq!(
        decode_michigan_dynamic_hex_foundation_v1(&mutated),
        Err(MichiganDynamicHexFoundationDecodeErrorV1::ArtifactDigest)
    );

    let truncated = &joined[..joined.len() - 1];
    assert_eq!(
        decode_michigan_dynamic_hex_foundation_v1(truncated),
        Err(MichiganDynamicHexFoundationDecodeErrorV1::Truncated)
    );

    joined.push(0);
    assert_eq!(
        decode_michigan_dynamic_hex_foundation_v1(&joined),
        Err(MichiganDynamicHexFoundationDecodeErrorV1::TrailingBytes)
    );
}
