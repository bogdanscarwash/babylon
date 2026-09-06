use super::{
    apply_material_staffing_v1, exact_real, read_stock, MaterialStaffingErrorV1,
    StaffingCompositionV1, StaffingEffectContextV1, StaffingEffectsV1, StaffingNodeBindingV1,
    EMPLOYED_POPULATION, MAX_EXACT_STAFFING_INTEGER_V1, PREVIOUS_UNRETAINED_HOURS,
    RESERVE_POPULATION, STAFFING_COMPOSITION_ID_V1, STAFFING_FIELDS_V1,
};
use babylon_bsl::causal_contract::{EvidenceClass, RuleRole};
use babylon_bsl::identity_codec::{project_stable_value_v1, StableBslValueV1};
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::{BslType, EnumRegistry, FieldKind};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_graph::working_copy::DetachedCopy;
use babylon_material_circuit::{
    ProcessIdV1, SiteIdV1, StaffingErrorV1, StaffingPolicyV1, StaffingPoolBindingV1,
    StaffingPoolIdV1, StaffingWorkRequestV1, UnitIdV1,
};

struct Fixture {
    graph: HypergraphStore,
    types: TypeEnv,
    enums: EnumRegistry,
    resolver: StableElementResolverV1,
}

impl Fixture {
    fn new(reverse: bool) -> Self {
        let workers = [
            "(node a NodeType/SOCIAL_CLASS (social-class/employed-population 3) (social-class/reserve-population 1) (social-class/previous-unretained-labor-hours 120))",
            "(node b NodeType/SOCIAL_CLASS (social-class/employed-population 1) (social-class/reserve-population 1) (social-class/previous-unretained-labor-hours 40))",
        ];
        let nodes = if reverse {
            [workers[1], workers[0]]
        } else {
            workers
        };
        let source = format!(
            "(scenario staffing/fixture
              (deffield social-class/employed-population int extensive)
              (deffield social-class/reserve-population int extensive)
              (deffield social-class/previous-unretained-labor-hours int extensive)
              {} {} (node land NodeType/TERRITORY))",
            nodes[0], nodes[1]
        );
        let mut graph = HypergraphStore::new();
        let scenario = load_scenario(&source, &mut graph).unwrap();
        let resolver = StableElementResolverV1::seal(
            &graph,
            &scenario.id,
            &scenario.node_content_ids,
            &scenario.hyperedge_content_ids,
        )
        .unwrap();
        Self {
            graph,
            types: TypeEnv {
                fields: scenario.fields,
                exemptions: &[],
            },
            enums: scenario.enums,
            resolver,
        }
    }

    fn node(&self, name: &str) -> NodeId {
        self.resolver.node_handle_by_local_name(name).unwrap()
    }

    fn apply(
        &mut self,
        composition: &StaffingCompositionV1,
        week: u64,
        requests: &[StaffingWorkRequestV1],
    ) -> Result<StaffingEffectsV1, MaterialStaffingErrorV1> {
        apply_material_staffing_v1(
            &mut self.graph,
            StaffingEffectContextV1 {
                types: &self.types,
                enums: &self.enums,
                vocabulary: None,
                resolver: &self.resolver,
            },
            composition,
            week,
            requests,
        )
    }
}

fn pool(key: u8, force: u64, schedule: u64, processes: &[u8]) -> StaffingPoolBindingV1 {
    StaffingPoolBindingV1::try_new(
        StaffingPoolIdV1::from_bytes([key; 32]),
        SiteIdV1::from_bytes([key; 32]),
        UnitIdV1::from_bytes([9; 32]),
        force,
        StaffingPolicyV1::one_week(schedule).unwrap(),
        processes
            .iter()
            .map(|key| ProcessIdV1::from_bytes([*key; 32]))
            .collect(),
    )
    .unwrap()
}

fn subject(name: &str) -> StableElementKeyV1 {
    StableElementKeyV1::Node {
        scenario: "staffing/fixture".to_owned(),
        local_name: name.to_owned(),
    }
}

fn binding(name: &str, pool: StaffingPoolBindingV1) -> StaffingNodeBindingV1 {
    StaffingNodeBindingV1::try_new(subject(name), pool).unwrap()
}

fn composition() -> StaffingCompositionV1 {
    StaffingCompositionV1::try_new(vec![binding("a", pool(1, 4, 40, &[1, 2]))]).unwrap()
}

fn request(
    binding: &StaffingNodeBindingV1,
    week: u64,
    process: u8,
    hours: u64,
) -> StaffingWorkRequestV1 {
    let pool = binding.pool();
    StaffingWorkRequestV1::new(
        week,
        pool.pool_id(),
        ProcessIdV1::from_bytes([process; 32]),
        pool.site_id(),
        pool.unit_id(),
        hours,
    )
}

fn requests(
    composition: &StaffingCompositionV1,
    week: u64,
    hours: [u64; 2],
) -> Vec<StaffingWorkRequestV1> {
    vec![
        request(&composition.bindings()[0], week, 1, hours[0]),
        request(&composition.bindings()[0], week, 2, hours[1]),
    ]
}

#[test]
fn shared_process_work_retains_only_one_week_then_recovers_with_exact_evidence() {
    let mut fixture = Fixture::new(false);
    let composition = composition();
    let a = fixture.node("a");
    let first = fixture
        .apply(&composition, 1, &requests(&composition, 1, [20, 20]))
        .unwrap();
    assert_eq!(first.staffing_receipts()[0].retained_hours(), 120);
    assert_eq!(first.staffing_receipts()[0].current_unretained_hours(), 40);
    assert_eq!(
        fixture
            .graph
            .node_attribute(a, PREVIOUS_UNRETAINED_HOURS)
            .unwrap()
            .to_bits(),
        40.0_f64.to_bits()
    );
    let second = fixture
        .apply(&composition, 2, &requests(&composition, 2, [20, 20]))
        .unwrap();
    assert_eq!(second.staffing_receipts()[0].closing_employed(), 1);
    assert_eq!(second.staffing_receipts()[0].closing_reserve(), 3);
    assert_eq!(second.staffing_receipts()[0].separations(), 2);
    assert_eq!(second.next_labor()[0].available, 40);
    let third = fixture
        .apply(&composition, 3, &requests(&composition, 3, [20, 21]))
        .unwrap();
    assert_eq!(third.staffing_receipts()[0].hires(), 1);
    assert_eq!(third.next_labor()[0].available, 80);
    assert_eq!(third.next_labor()[0].week, 4);
    assert_eq!(third.writes().len(), 3);
    assert_eq!(third.audit_receipts().len(), 4);
    for (index, receipt) in third.audit_receipts().iter().enumerate() {
        assert_eq!(receipt.ordinal, u32::try_from(index).unwrap());
        assert_eq!(receipt.rule_id, STAFFING_COMPOSITION_ID_V1);
        assert_eq!(receipt.role, RuleRole::Mechanic);
        assert_eq!(receipt.evidence, EvidenceClass::Designed);
    }
    assert_eq!(
        fixture
            .graph
            .node_attribute(a, EMPLOYED_POPULATION)
            .unwrap()
            .to_bits(),
        2.0_f64.to_bits()
    );
    assert!(fixture
        .graph
        .node_attribute(a, "social-class/population")
        .is_err());
    assert!(fixture
        .graph
        .node_attribute(a, "social-class/revolutionary")
        .is_err());
}

#[test]
fn evidence_subject_is_stable_across_different_node_allocation_order() {
    let composition = composition();
    let mut first = Fixture::new(false);
    let mut second = Fixture::new(true);
    assert_ne!(first.node("a"), second.node("a"));
    let left = first
        .apply(&composition, 1, &requests(&composition, 1, [0, 0]))
        .unwrap();
    let right = second
        .apply(&composition, 1, &requests(&composition, 1, [0, 0]))
        .unwrap();
    let stable_payload = |effects: &StaffingEffectsV1, resolver: &StableElementResolverV1| {
        effects.committed_events()[0]
            .payload()
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    project_stable_value_v1(value, resolver).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        stable_payload(&left, &first.resolver),
        stable_payload(&right, &second.resolver)
    );
    assert_eq!(left.audit_receipts(), right.audit_receipts());
    assert_eq!(left.staffing_receipts(), right.staffing_receipts());
    assert!(stable_payload(&left, &first.resolver)
        .iter()
        .any(|(name, value)| name == "subject" && *value == StableBslValueV1::Node(subject("a"))));
    assert_eq!(
        left.committed_events()[0].emitting_rule(),
        STAFFING_COMPOSITION_ID_V1
    );
    assert!(left.committed_events()[0].choice_receipt().is_none());
}

#[test]
fn complete_request_roster_and_exact_bindings_are_required_before_writes() {
    let composition = composition();
    let mut fixture = Fixture::new(false);
    let before = fixture.graph.state_hash().unwrap();
    let missing = requests(&composition, 1, [0, 0]);
    assert!(matches!(
        fixture.apply(&composition, 1, &missing[..1]),
        Err(MaterialStaffingErrorV1::Core(
            StaffingErrorV1::MissingRequest
        ))
    ));
    let wrong = StaffingWorkRequestV1::new(
        1,
        StaffingPoolIdV1::from_bytes([7; 32]),
        ProcessIdV1::from_bytes([1; 32]),
        SiteIdV1::from_bytes([1; 32]),
        UnitIdV1::from_bytes([9; 32]),
        0,
    );
    assert!(matches!(
        fixture.apply(&composition, 1, &[wrong, missing[1]]),
        Err(MaterialStaffingErrorV1::Core(
            StaffingErrorV1::RequestBinding
        ))
    ));
    assert_eq!(fixture.graph.state_hash().unwrap(), before);
}

#[test]
fn malformed_numeric_readings_never_become_workforce_defaults() {
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.0,
        0.5,
        9_007_199_254_740_994.0,
    ] {
        let mut fixture = Fixture::new(false);
        let a = fixture.node("a");
        fixture
            .graph
            .update_node(a, PREVIOUS_UNRETAINED_HOURS, invalid)
            .unwrap();
        let composition = composition();
        assert!(fixture
            .apply(&composition, 1, &requests(&composition, 1, [0, 0]))
            .is_err());
        assert_eq!(
            fixture
                .graph
                .node_attribute(a, PREVIOUS_UNRETAINED_HOURS)
                .unwrap()
                .to_bits(),
            invalid.to_bits()
        );
        assert_eq!(
            fixture
                .graph
                .node_attribute(a, EMPLOYED_POPULATION)
                .unwrap()
                .to_bits(),
            3.0_f64.to_bits()
        );
    }
}

#[test]
fn exact_boundary_and_negative_zero_are_canonical() {
    assert_eq!(
        exact_real(MAX_EXACT_STAFFING_INTEGER_V1).unwrap().to_bits(),
        9_007_199_254_740_992.0_f64.to_bits()
    );
    assert!(exact_real(MAX_EXACT_STAFFING_INTEGER_V1 + 1).is_err());
    let mut fixture = Fixture::new(false);
    let a = fixture.node("a");
    fixture
        .graph
        .update_node(
            a,
            PREVIOUS_UNRETAINED_HOURS,
            exact_real(MAX_EXACT_STAFFING_INTEGER_V1).unwrap(),
        )
        .unwrap();
    let composition = composition();
    let boundary = fixture
        .apply(&composition, 1, &requests(&composition, 1, [0, 0]))
        .unwrap();
    assert_eq!(
        boundary.staffing_receipts()[0].previous_unretained_hours(),
        MAX_EXACT_STAFFING_INTEGER_V1
    );
    assert_eq!(boundary.staffing_receipts()[0].closing_employed(), 4);
    fixture
        .graph
        .update_node(a, PREVIOUS_UNRETAINED_HOURS, -0.0)
        .unwrap();
    fixture
        .apply(&composition, 1, &requests(&composition, 1, [0, 0]))
        .unwrap();
    for field in [EMPLOYED_POPULATION, PREVIOUS_UNRETAINED_HOURS] {
        assert_eq!(
            fixture.graph.node_attribute(a, field).unwrap().to_bits(),
            0.0_f64.to_bits()
        );
    }
}

#[test]
fn every_output_conversion_finishes_before_the_first_effect() {
    let mut fixture = Fixture::new(false);
    let composition = StaffingCompositionV1::try_new(vec![
        binding("a", pool(1, 4, 40, &[1])),
        binding("b", pool(2, 2, 40, &[2])),
    ])
    .unwrap();
    let before = fixture.graph.state_hash().unwrap();
    let inputs = [
        request(&composition.bindings()[0], 1, 1, 0),
        request(
            &composition.bindings()[1],
            1,
            2,
            MAX_EXACT_STAFFING_INTEGER_V1 + 1,
        ),
    ];
    assert!(matches!(
        fixture.apply(&composition, 1, &inputs),
        Err(MaterialStaffingErrorV1::ExactInteger)
    ));
    assert_eq!(fixture.graph.state_hash().unwrap(), before);
}

#[test]
fn field_declarations_require_all_three_int_extensive_fields() {
    for field in STAFFING_FIELDS_V1 {
        for mutation in 0..3 {
            let mut fixture = Fixture::new(false);
            if mutation == 0 {
                fixture.types.fields.remove(field);
            } else {
                let declaration = fixture.types.fields.get_mut(field).unwrap();
                if mutation == 1 {
                    declaration.ty = BslType::Real;
                } else {
                    declaration.kind = FieldKind::Intensive;
                }
            }
            let composition = composition();
            let before = fixture.graph.state_hash().unwrap();
            assert!(matches!(
                fixture.apply(&composition, 1, &requests(&composition, 1, [0, 0])),
                Err(MaterialStaffingErrorV1::FieldDeclaration(_))
            ));
            assert_eq!(fixture.graph.state_hash().unwrap(), before);
        }
    }
}

#[test]
fn missing_node_fields_wrong_owners_and_foreign_scopes_refuse() {
    let mut fixture = Fixture::new(false);
    let land = StaffingCompositionV1::try_new(vec![binding("land", pool(1, 4, 40, &[1]))]).unwrap();
    assert!(matches!(
        fixture.apply(&land, 1, &[request(&land.bindings()[0], 1, 1, 0)]),
        Err(MaterialStaffingErrorV1::NodeOwner)
    ));
    let mut foreign = subject("a");
    if let StableElementKeyV1::Node { scenario, .. } = &mut foreign {
        *scenario = "other/fixture".to_owned();
    }
    let foreign = StaffingCompositionV1::try_new(vec![StaffingNodeBindingV1::try_new(
        foreign,
        pool(1, 4, 40, &[1]),
    )
    .unwrap()])
    .unwrap();
    assert!(fixture
        .apply(&foreign, 1, &[request(&foreign.bindings()[0], 1, 1, 0)])
        .is_err());
    let empty = fixture.graph.add_node("SOCIAL_CLASS").unwrap();
    assert!(read_stock(
        &fixture.graph,
        empty,
        EMPLOYED_POPULATION,
        &StaffingEffectContextV1 {
            types: &fixture.types,
            enums: &fixture.enums,
            vocabulary: None,
            resolver: &fixture.resolver
        }
    )
    .is_err());
}

#[test]
fn admitted_composition_refuses_schedule_and_duplicate_principals() {
    assert!(StaffingNodeBindingV1::try_new(subject("a"), pool(1, 4, 39, &[1])).is_err());
    let a = binding("a", pool(1, 4, 40, &[1]));
    let same_node = binding("a", pool(2, 2, 40, &[2]));
    assert!(matches!(
        StaffingCompositionV1::try_new(vec![a.clone(), same_node]),
        Err(MaterialStaffingErrorV1::DuplicateNode)
    ));
    let same_pool = binding("b", pool(1, 2, 40, &[2]));
    assert!(matches!(
        StaffingCompositionV1::try_new(vec![a.clone(), same_pool]),
        Err(MaterialStaffingErrorV1::Core(
            StaffingErrorV1::DuplicatePool
        ))
    ));
    let same_process = binding("b", pool(2, 2, 40, &[1]));
    assert!(matches!(
        StaffingCompositionV1::try_new(vec![a, same_process]),
        Err(MaterialStaffingErrorV1::Core(
            StaffingErrorV1::DuplicateProcess
        ))
    ));
    assert!(StaffingCompositionV1::try_new(vec![]).is_err());
}

#[test]
fn different_pool_ids_cannot_double_count_one_site_unit() {
    let a = binding("a", pool(1, 4, 40, &[1]));
    let b = binding(
        "b",
        StaffingPoolBindingV1::try_new(
            StaffingPoolIdV1::from_bytes([2; 32]),
            a.pool().site_id(),
            a.pool().unit_id(),
            2,
            StaffingPolicyV1::one_week(40).unwrap(),
            vec![ProcessIdV1::from_bytes([2; 32])],
        )
        .unwrap(),
    );
    assert!(matches!(
        StaffingCompositionV1::try_new(vec![a, b]),
        Err(MaterialStaffingErrorV1::Core(
            StaffingErrorV1::DuplicateSiteUnit
        ))
    ));
}

#[test]
fn every_node_read_finishes_before_the_first_effect() {
    let mut fixture = Fixture::new(false);
    let a = fixture.node("a");
    let b = fixture.node("b");
    fixture
        .graph
        .update_node(b, PREVIOUS_UNRETAINED_HOURS, 0.5)
        .unwrap();
    let composition = StaffingCompositionV1::try_new(vec![
        binding("a", pool(1, 4, 40, &[1])),
        binding("b", pool(2, 2, 40, &[2])),
    ])
    .unwrap();
    let inputs = [
        request(&composition.bindings()[0], 1, 1, 0),
        request(&composition.bindings()[1], 1, 2, 0),
    ];
    assert!(fixture.apply(&composition, 1, &inputs).is_err());
    assert_eq!(
        fixture
            .graph
            .node_attribute(a, PREVIOUS_UNRETAINED_HOURS)
            .unwrap()
            .to_bits(),
        120.0_f64.to_bits()
    );
}

#[test]
fn binding_and_request_permutations_preserve_effect_and_evidence_order() {
    let a = binding("a", pool(1, 4, 40, &[1]));
    let b = binding("b", pool(2, 2, 40, &[2]));
    let left = StaffingCompositionV1::try_new(vec![a.clone(), b.clone()]).unwrap();
    let right = StaffingCompositionV1::try_new(vec![b.clone(), a.clone()]).unwrap();
    let inputs = [request(&b, 1, 2, 81), request(&a, 1, 1, 0)];
    let mut first = Fixture::new(false);
    let mut second = Fixture::new(false);
    let first_effects = first.apply(&left, 1, &inputs).unwrap();
    let second_effects = second.apply(&right, 1, &[inputs[1], inputs[0]]).unwrap();
    assert_eq!(first_effects.writes(), second_effects.writes());
    assert_eq!(
        first_effects.audit_receipts(),
        second_effects.audit_receipts()
    );
    assert_eq!(
        first_effects.committed_events(),
        second_effects.committed_events()
    );
    assert_eq!(first_effects.next_labor(), second_effects.next_labor());
    assert_eq!(
        first.graph.state_hash().unwrap(),
        second.graph.state_hash().unwrap()
    );
}

#[test]
fn population_and_arithmetic_refusals_precede_effects() {
    let mut fixture = Fixture::new(false);
    let a = fixture.node("a");
    fixture
        .graph
        .update_node(a, RESERVE_POPULATION, 0.0)
        .unwrap();
    let composition = composition();
    assert!(matches!(
        fixture.apply(&composition, 1, &requests(&composition, 1, [0, 0])),
        Err(MaterialStaffingErrorV1::Core(
            StaffingErrorV1::PopulationInvariant
        ))
    ));
    fixture
        .graph
        .update_node(a, RESERVE_POPULATION, 1.0)
        .unwrap();
    let before = fixture.graph.state_hash().unwrap();
    assert!(matches!(
        fixture.apply(&composition, 1, &requests(&composition, 1, [u64::MAX, 1])),
        Err(MaterialStaffingErrorV1::Core(StaffingErrorV1::Arithmetic))
    ));
    assert_eq!(fixture.graph.state_hash().unwrap(), before);
}

#[test]
fn dropping_a_successful_detached_candidate_does_not_publish_its_writes() {
    let fixture = Fixture::new(false);
    let before = fixture.graph.state_hash().unwrap();
    let mut candidate = fixture.graph.detached_copy();
    let composition = composition();
    let effects = apply_material_staffing_v1(
        &mut candidate,
        StaffingEffectContextV1 {
            types: &fixture.types,
            enums: &fixture.enums,
            vocabulary: None,
            resolver: &fixture.resolver,
        },
        &composition,
        1,
        &requests(&composition, 1, [0, 0]),
    )
    .unwrap();
    assert_eq!(effects.writes().len(), 3);
    assert_ne!(candidate.state_hash().unwrap(), before);
    drop(candidate);
    assert_eq!(fixture.graph.state_hash().unwrap(), before);
}
