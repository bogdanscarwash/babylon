use std::collections::HashMap;

use babylon_bsl::causal_contract::{
    AuditReceipt, EffectSignature, EvidenceClass, RuleRole, ShapeVerb,
};
use babylon_bsl::evaluator::Value;
use babylon_bsl::exemptions::IntensiveAggregationExemption;
use babylon_bsl::fuel::IntrinsicCosts;
use babylon_bsl::identity_codec::{
    decode_option_presence_v1, encode_bsl_type_v1, encode_const_value_v1,
    encode_effect_signature_v1, encode_enum_kind_v1, encode_evidence_class_v1,
    encode_field_kind_v1, encode_rule_role_v1, encode_shape_verb_v1, encode_value_v1,
    IdentityCodecError,
};
use babylon_bsl::identity_sections::{
    encode_prepared_bsl_sections_v1, encode_tick_payload_sections_v1, PreparedBslSectionsV1,
};
use babylon_bsl::query::EdgeKey;
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::{BslType, EnumRegistry, EnumTypeId, FieldDecl, FieldKind};
use babylon_bsl::vocabulary::{ClosedVocabulary, EnumKind};
use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::StableElementResolverV1;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::{Currency, Ratio};

struct GraphFixture {
    owners: NodeId,
    workers: NodeId,
    coalition: HyperedgeId,
    resolver: StableElementResolverV1,
}

fn graph_fixture() -> GraphFixture {
    let mut graph = MemoryGraph::new();
    let owners = graph.add_node("class").unwrap();
    let workers = graph.add_node("class").unwrap();
    graph.add_edge("solidarity", workers, owners, 0.5).unwrap();
    let coalition = graph
        .add_hyperedge("coalition", &[workers, owners])
        .unwrap();
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/world",
        &HashMap::from([
            (owners, "owners".to_owned()),
            (workers, "workers".to_owned()),
        ]),
        &HashMap::from([(coalition, "coalition-a".to_owned())]),
    )
    .unwrap();
    GraphFixture {
        owners,
        workers,
        coalition,
        resolver,
    }
}

fn encoded_value(value: &Value, resolver: &StableElementResolverV1) -> Vec<u8> {
    let mut output = Vec::new();
    encode_value_v1(value, resolver, &mut output).unwrap();
    output
}

fn str32(value: &str) -> Vec<u8> {
    [
        u32::try_from(value.len()).unwrap().to_be_bytes().as_slice(),
        value.as_bytes(),
    ]
    .concat()
}

#[test]
fn scalar_value_discriminants_and_encoding_are_exact() {
    let fixture = graph_fixture();
    let ratio = Ratio::new(2.0).unwrap();
    let floor = Ratio::new(1.0).unwrap();
    let cap = Ratio::new(3.0).unwrap();
    assert_eq!(
        encoded_value(&Value::Int(-7), &fixture.resolver),
        [vec![0x01], (-7_i64).to_be_bytes().to_vec()].concat()
    );
    assert_eq!(
        encoded_value(
            &Value::Currency(Currency::from_micro_units(-9)),
            &fixture.resolver
        ),
        [vec![0x02], (-9_i128).to_be_bytes().to_vec()].concat()
    );
    assert_eq!(
        encoded_value(&Value::Real(-0.0), &fixture.resolver),
        [vec![0x03], 0_u64.to_be_bytes().to_vec()].concat()
    );
    assert_eq!(
        encoded_value(
            &Value::Ratio {
                value: ratio,
                floor: Some(floor),
                cap: Some(cap)
            },
            &fixture.resolver
        ),
        [
            vec![0x04],
            2.0_f64.to_bits().to_be_bytes().to_vec(),
            vec![1],
            1.0_f64.to_bits().to_be_bytes().to_vec(),
            vec![1],
            3.0_f64.to_bits().to_be_bytes().to_vec()
        ]
        .concat()
    );
    assert_eq!(
        encoded_value(
            &Value::Ratio {
                value: ratio,
                floor: None,
                cap: None,
            },
            &fixture.resolver,
        ),
        [
            vec![0x04],
            2.0_f64.to_bits().to_be_bytes().to_vec(),
            vec![0, 0],
        ]
        .concat()
    );
    assert_eq!(
        encoded_value(&Value::Bool(false), &fixture.resolver),
        vec![0x05, 0]
    );
    assert_eq!(
        encoded_value(&Value::Bool(true), &fixture.resolver),
        vec![0x05, 1]
    );
    assert_eq!(
        encoded_value(
            &Value::Enum {
                enum_type: "OrgKind".to_owned(),
                member: "BUSINESS".to_owned()
            },
            &fixture.resolver
        ),
        [vec![0x06], str32("OrgKind"), str32("BUSINESS")].concat()
    );
}

#[test]
fn reference_value_discriminants_and_stable_encoding_are_exact() {
    let fixture = graph_fixture();
    assert_eq!(
        encoded_value(&Value::NodeRef(fixture.workers), &fixture.resolver),
        [
            vec![0x07],
            fixture
                .resolver
                .node_key(fixture.workers)
                .unwrap()
                .canonical_bytes()
                .unwrap()
        ]
        .concat()
    );
    assert_eq!(
        encoded_value(&Value::HyperedgeRef(fixture.coalition), &fixture.resolver),
        [
            vec![0x08],
            fixture
                .resolver
                .hyperedge_key(fixture.coalition)
                .unwrap()
                .canonical_bytes()
                .unwrap()
        ]
        .concat()
    );
    assert_eq!(
        encoded_value(
            &Value::EdgeRef(EdgeKey {
                source: fixture.workers,
                target: fixture.owners,
                edge_type: "solidarity".to_owned()
            }),
            &fixture.resolver
        ),
        [
            vec![0x09],
            fixture
                .resolver
                .edge_key("solidarity", fixture.workers, fixture.owners)
                .unwrap()
                .canonical_bytes()
                .unwrap()
        ]
        .concat()
    );
}

#[test]
fn governed_type_and_contract_discriminants_are_exact() {
    let mut enums = EnumRegistry::default();
    let org_kind = enums.declare("OrgKind", &["BUSINESS".to_owned()]).unwrap();
    let type_cases = [
        (BslType::Probability, vec![0x01]),
        (BslType::Intensity, vec![0x02]),
        (BslType::Coefficient, vec![0x03]),
        (BslType::Currency, vec![0x04]),
        (BslType::Real, vec![0x05]),
        (BslType::Int, vec![0x06]),
        (BslType::Bool, vec![0x07]),
        (
            BslType::Enum(org_kind),
            [vec![0x08], str32("OrgKind")].concat(),
        ),
        (
            BslType::NodeSet("SOCIAL_CLASS"),
            [vec![0x09], str32("SOCIAL_CLASS")].concat(),
        ),
        (
            BslType::EdgeSet("SOLIDARITY"),
            [vec![0x0a], str32("SOLIDARITY")].concat(),
        ),
    ];
    for (value, expected) in type_cases {
        let mut output = Vec::new();
        encode_bsl_type_v1(&value, &enums, &mut output).unwrap();
        assert_eq!(output, expected);
    }
    assert_eq!(encode_field_kind_v1(FieldKind::Intensive), 0x01);
    assert_eq!(encode_field_kind_v1(FieldKind::Extensive), 0x02);
    assert_eq!(encode_field_kind_v1(FieldKind::NotApplicable), 0x03);
    assert_eq!(encode_rule_role_v1(RuleRole::Mechanic), 0x01);
    assert_eq!(encode_rule_role_v1(RuleRole::Recognizer), 0x02);
    assert_eq!(encode_rule_role_v1(RuleRole::ExternalEvent), 0x03);
    assert_eq!(encode_rule_role_v1(RuleRole::Intent), 0x04);
    assert_eq!(encode_evidence_class_v1(EvidenceClass::Observed), 0x01);
    assert_eq!(encode_evidence_class_v1(EvidenceClass::Derived), 0x02);
    assert_eq!(encode_evidence_class_v1(EvidenceClass::Calibrated), 0x03);
    assert_eq!(encode_evidence_class_v1(EvidenceClass::Designed), 0x04);
    for (verb, tag) in [
        (ShapeVerb::AddNode, 0x01),
        (ShapeVerb::RemoveNode, 0x02),
        (ShapeVerb::AddEdge, 0x03),
        (ShapeVerb::RemoveEdge, 0x04),
        (ShapeVerb::AddHyperedge, 0x05),
        (ShapeVerb::RemoveHyperedge, 0x06),
    ] {
        assert_eq!(encode_shape_verb_v1(verb), tag);
    }
    for (kind, tag) in [
        (EnumKind::NodeType, 0x01),
        (EnumKind::EdgeType, 0x02),
        (EnumKind::HyperedgeType, 0x03),
        (EnumKind::EventType, 0x04),
    ] {
        assert_eq!(encode_enum_kind_v1(kind), tag);
    }
    let mut effect = Vec::new();
    encode_effect_signature_v1(&EffectSignature::Event("RUPTURE".to_owned()), &mut effect).unwrap();
    assert_eq!(effect, [vec![0x04], str32("EventType/RUPTURE")].concat());
    effect.clear();
    encode_effect_signature_v1(
        &EffectSignature::Shape(ShapeVerb::RemoveHyperedge),
        &mut effect,
    )
    .unwrap();
    assert_eq!(effect, vec![0x05, 0x06]);
    for (signature, tag, name) in [
        (
            EffectSignature::NodeField("class/power".to_owned()),
            0x01,
            "class/power",
        ),
        (
            EffectSignature::EdgeField("solidarity/tension".to_owned()),
            0x02,
            "solidarity/tension",
        ),
        (
            EffectSignature::HyperedgeField("coalition/cohesion".to_owned()),
            0x03,
            "coalition/cohesion",
        ),
    ] {
        effect.clear();
        encode_effect_signature_v1(&signature, &mut effect).unwrap();
        assert_eq!(effect, [vec![tag], str32(name)].concat());
    }
}

#[test]
fn options_constants_and_invalid_identity_values_refuse_loudly() {
    let fixture = graph_fixture();
    assert_eq!(decode_option_presence_v1(0), Ok(false));
    assert_eq!(decode_option_presence_v1(1), Ok(true));
    assert_eq!(
        decode_option_presence_v1(2),
        Err(IdentityCodecError::InvalidOptionByte { value: 2 })
    );
    let mut output = Vec::new();
    encode_const_value_v1(&Value::Bool(true), &mut output).unwrap();
    assert_eq!(output, vec![0x05, 1]);
    assert!(matches!(
        encode_const_value_v1(
            &Value::Enum {
                enum_type: "OrgKind".to_owned(),
                member: "BUSINESS".to_owned()
            },
            &mut Vec::new()
        ),
        Err(IdentityCodecError::InvalidConstantKind)
    ));
    assert!(matches!(
        encode_const_value_v1(&Value::NodeRef(fixture.owners), &mut Vec::new()),
        Err(IdentityCodecError::InvalidConstantKind)
    ));
    assert!(matches!(
        encode_bsl_type_v1(
            &BslType::Enum(EnumTypeId(99)),
            &EnumRegistry::default(),
            &mut Vec::new()
        ),
        Err(IdentityCodecError::UnknownEnumType { .. })
    ));
    assert!(matches!(
        encode_value_v1(
            &Value::NodeRef(NodeId(999)),
            &fixture.resolver,
            &mut Vec::new()
        ),
        Err(IdentityCodecError::StableIdentity(_))
    ));
    assert!(matches!(
        encode_value_v1(&Value::Real(f64::NAN), &fixture.resolver, &mut Vec::new()),
        Err(IdentityCodecError::NonFiniteValue)
    ));
}

static EXEMPTIONS: &[IntensiveAggregationExemption] = &[IntensiveAggregationExemption {
    field_name: "class/power",
    reason: "fixture",
    owner: "Director",
    date: "2026-08-26",
}];

fn prepared_sections() -> (PreparedBslSectionsV1, PreparedBslSectionsV1) {
    let mut enums = EnumRegistry::default();
    enums
        .declare("Zed", &["SECOND".to_owned(), "FIRST".to_owned()])
        .unwrap();
    enums
        .declare("Alpha", &["BETA".to_owned(), "ALPHA".to_owned()])
        .unwrap();
    let types = TypeEnv {
        fields: HashMap::from([
            (
                "class/wage".to_owned(),
                FieldDecl {
                    ty: BslType::Currency,
                    kind: FieldKind::Extensive,
                },
            ),
            (
                "class/power".to_owned(),
                FieldDecl {
                    ty: BslType::Intensity,
                    kind: FieldKind::Intensive,
                },
            ),
        ]),
        exemptions: EXEMPTIONS,
    };
    let intrinsics = IntrinsicCosts::new(HashMap::from([
        ("rng-draw".to_owned(), 9),
        ("floor".to_owned(), 2),
    ]));
    let consts = HashMap::from([
        ("z/value".to_owned(), Value::Int(2)),
        ("a/value".to_owned(), Value::Bool(true)),
    ]);
    let vocabulary = ClosedVocabulary::new([
        (EnumKind::NodeType, Vec::new()),
        (EnumKind::EdgeType, vec!["SOLIDARITY".to_owned()]),
    ])
    .unwrap();
    let present =
        encode_prepared_bsl_sections_v1(&types, &intrinsics, &consts, &enums, Some(&vocabulary))
            .unwrap();
    let absent =
        encode_prepared_bsl_sections_v1(&types, &intrinsics, &consts, &enums, None).unwrap();
    (present, absent)
}

#[test]
fn prepared_sections_preserve_vocabulary_presence() {
    let (present, absent) = prepared_sections();
    assert_ne!(present.vocabulary(), absent.vocabulary());
    assert_eq!(present.vocabulary()[0], 1);
    assert_eq!(absent.vocabulary(), &[0]);
    assert_eq!(
        present.vocabulary(),
        [
            vec![1, 0x01, 1],
            0_u32.to_be_bytes().to_vec(),
            vec![0x02, 1],
            1_u32.to_be_bytes().to_vec(),
            str32("SOLIDARITY"),
            vec![0x03, 0, 0x04, 0],
        ]
        .concat()
    );
}

#[test]
fn prepared_intrinsic_identity_accepts_96_bytes_and_refuses_97() {
    let maximum = IntrinsicCosts::new(HashMap::from([("x".repeat(96), 1)]));
    let oversized = IntrinsicCosts::new(HashMap::from([("x".repeat(97), 1)]));
    let types = TypeEnv {
        fields: HashMap::new(),
        exemptions: &[],
    };
    let constants = HashMap::new();
    let enums = EnumRegistry::default();

    assert!(encode_prepared_bsl_sections_v1(&types, &maximum, &constants, &enums, None,).is_ok());
    assert_eq!(
        encode_prepared_bsl_sections_v1(&types, &oversized, &constants, &enums, None,),
        Err(IdentityCodecError::InvalidString {
            field: "intrinsic name",
            index: 97,
        })
    );
}

#[test]
fn prepared_sections_sort_registries_and_preserve_enum_member_order() {
    let (present, _) = prepared_sections();
    assert!(
        present
            .fields_and_exemptions()
            .windows(str32("class/power").len())
            .position(|w| w == str32("class/power"))
            .unwrap()
            < present
                .fields_and_exemptions()
                .windows(str32("class/wage").len())
                .position(|w| w == str32("class/wage"))
                .unwrap()
    );
    assert!(
        present
            .intrinsic_costs()
            .windows(str32("floor").len())
            .position(|w| w == str32("floor"))
            .unwrap()
            < present
                .intrinsic_costs()
                .windows(str32("rng-draw").len())
                .position(|w| w == str32("rng-draw"))
                .unwrap()
    );
    assert!(
        present
            .constants()
            .windows(str32("a/value").len())
            .position(|w| w == str32("a/value"))
            .unwrap()
            < present
                .constants()
                .windows(str32("z/value").len())
                .position(|w| w == str32("z/value"))
                .unwrap()
    );
    let alpha_members = [str32("BETA"), str32("ALPHA")].concat();
    assert!(present
        .enum_types()
        .windows(alpha_members.len())
        .any(|w| w == alpha_members));
    assert!(
        present
            .enum_types()
            .windows(str32("Alpha").len())
            .position(|window| window == str32("Alpha"))
            .unwrap()
            < present
                .enum_types()
                .windows(str32("Zed").len())
                .position(|window| window == str32("Zed"))
                .unwrap()
    );
}

#[test]
fn payload_sections_preserve_governed_and_live_vector_order() {
    let fixture = graph_fixture();
    let outcomes = vec![("z/rule".to_owned(), 2), ("a/rule".to_owned(), 1)];
    let events = vec![
        (
            "FIRST".to_owned(),
            vec![
                ("label".to_owned(), Value::Int(1)),
                ("label".to_owned(), Value::Int(2)),
            ],
        ),
        ("EventType/SECOND".to_owned(), vec![]),
    ];
    let receipts = vec![
        AuditReceipt {
            rule_id: "z/rule".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
            ordinal: 1,
            effect: EffectSignature::NodeField("class/wage".to_owned()),
        },
        AuditReceipt {
            rule_id: "a/rule".to_owned(),
            role: RuleRole::Recognizer,
            evidence: EvidenceClass::Observed,
            ordinal: 0,
            effect: EffectSignature::Event("SECOND".to_owned()),
        },
    ];
    let sections =
        encode_tick_payload_sections_v1(&outcomes, &events, &receipts, &fixture.resolver).unwrap();
    let reversed_events = encode_tick_payload_sections_v1(
        &outcomes,
        &events.iter().cloned().rev().collect::<Vec<_>>(),
        &receipts,
        &fixture.resolver,
    )
    .unwrap();
    let reversed_receipts = encode_tick_payload_sections_v1(
        &outcomes,
        &events,
        &receipts.iter().cloned().rev().collect::<Vec<_>>(),
        &fixture.resolver,
    )
    .unwrap();
    assert_ne!(sections.events(), reversed_events.events());
    assert_ne!(sections.receipts(), reversed_receipts.receipts());
    let duplicate_pair = [
        str32("label"),
        vec![0x01],
        1_i64.to_be_bytes().to_vec(),
        str32("label"),
        vec![0x01],
        2_i64.to_be_bytes().to_vec(),
    ]
    .concat();
    assert!(sections
        .events()
        .windows(duplicate_pair.len())
        .any(|w| w == duplicate_pair));
    assert!(sections.rule_outcomes().starts_with(&2_u32.to_be_bytes()));
    assert!(
        sections
            .rule_outcomes()
            .windows(str32("z/rule").len())
            .position(|window| window == str32("z/rule"))
            .unwrap()
            < sections
                .rule_outcomes()
                .windows(str32("a/rule").len())
                .position(|window| window == str32("a/rule"))
                .unwrap()
    );
    assert_eq!(sections.accepted_action_outcomes(), &0_u16.to_be_bytes());
}
