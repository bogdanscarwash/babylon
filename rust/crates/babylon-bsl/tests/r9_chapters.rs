//! The R9 gap-fill chapters' conformance families (`bsl-language.rst` §6.2
//! families 10–22). One module per chapter, in the §7 dependency order the
//! chapters were planned in (C1 → C13).
//!
//! **Scope honesty, recorded once here rather than per family.** §6.2's
//! families mix load-time and evaluation-time obligations. Through PR 3
//! (the BSL query-evaluation plan's slice 1, Phase 2), the crate's
//! evaluator served only the §4 *expression core*: queries, folds,
//! selections, accessors and effect-position iteration had no runtime (the
//! `conformance_corpus.rs` header recorded the same boundary for the
//! pre-R9 estate). **PR 4, Task 13 (2026-08-11) retires that boundary for
//! three families**: 14 (element selection), 15 (effect-position
//! iteration) and 17 (typed neighbours and element naming) now EXECUTE —
//! their `fold`/`exists`/`forall`/`select-max`/`select-min`/`field-of`/
//! `for-each` vectors run the real query evaluator over a `MemoryGraph`
//! fixture and assert the RAISED value or written state, not merely the
//! code's identity. Every other family stays exactly as it was: the
//! `E-LEX`/`E-PARSE`/`E-TYPE`/`E-LOAD` classes execute for real everywhere
//! (unchanged, load-time), and an `E-EVAL` row outside families 14/15/17
//! is still pinned as its code's identity and discipline rather than as a
//! raised value — each remaining deferral is named in its own family's
//! module doc with the slice that will serve it (2 for the dyadic edge
//! lane — `edges`/`edge-between`/`the`; 3 for the hyperedge + metric lane
//! — `hyperedges`/`members-of`/`hyperedges-of`/`metric-of`; 4 for
//! attribute STORAGE — `update-edge`/`update-hyperedge`, Constitution
//! III.7), never silently skipped.

use babylon_bsl::bindings::BindingVocabulary;
use babylon_bsl::bound_checker::{expr_cost, rule_bound, BoundError};
use babylon_bsl::declarations::{check_intrinsic_name, FieldRegistry};
use babylon_bsl::evaluator::EvalCode;
use babylon_bsl::fuel::{CardinalityCeilings, IntrinsicCosts};
use babylon_bsl::reader::{read, SExpr};
use babylon_bsl::rule_pipeline::{load_rule, LoadContext};
use babylon_bsl::scope::check_foreign_field_scoping;
use babylon_bsl::typecheck::{typecheck_aggregation, TypeCode, TypeEnv};
use babylon_bsl::types::EnumRegistry;
use babylon_bsl::vocabulary::{ClosedVocabulary, EnumKind};
use std::collections::{HashMap, HashSet};

// ------------------------------------------------------------- fixtures

fn vocabulary() -> ClosedVocabulary {
    ClosedVocabulary::new([
        (
            EnumKind::NodeType,
            vec![
                "SOCIAL_CLASS".to_owned(),
                "ORGANIZATION".to_owned(),
                "TERRITORY".to_owned(),
                "POLITY".to_owned(),
            ],
        ),
        (
            EnumKind::EdgeType,
            vec![
                "SOLIDARITY".to_owned(),
                "EXPLOITATION".to_owned(),
                "IN_SCALE".to_owned(),
            ],
        ),
        (
            EnumKind::HyperedgeType,
            vec!["ECONOMIC_SECTOR".to_owned(), "COMMUNITY".to_owned()],
        ),
        (EnumKind::EventType, vec!["RUPTURE".to_owned()]),
    ])
    .expect("the R9 fixture vocabulary is disjoint")
}

fn ceilings() -> CardinalityCeilings {
    CardinalityCeilings::new(
        HashMap::from([
            ("NodeType/SOCIAL_CLASS".to_owned(), 100),
            ("NodeType/ORGANIZATION".to_owned(), 40),
            ("NodeType/TERRITORY".to_owned(), 3000),
            ("NodeType/POLITY".to_owned(), 1),
            ("EdgeType/SOLIDARITY".to_owned(), 40),
            ("EdgeType/EXPLOITATION".to_owned(), 60),
            ("EdgeType/IN_SCALE".to_owned(), 5000),
            ("HyperedgeType/ECONOMIC_SECTOR".to_owned(), 500),
            ("HyperedgeType/COMMUNITY".to_owned(), 200),
        ]),
        HashMap::from([
            ("HyperedgeType/ECONOMIC_SECTOR".to_owned(), 32),
            ("HyperedgeType/COMMUNITY".to_owned(), 64),
        ]),
    )
}

fn e(source: &str) -> SExpr {
    read(source).expect("test source must parse").0
}

fn cost(source: &str) -> Result<u64, BoundError> {
    expr_cost(&e(source), &ceilings(), &IntrinsicCosts::default())
}

fn bound(source: &str) -> Result<u64, BoundError> {
    rule_bound(&e(source), &ceilings(), &IntrinsicCosts::default())
}

/// No family in this file declares an enum-typed field — an empty registry
/// is the honest "no `defenum`s in scope" input to `FieldRegistry::declare`.
fn enums() -> EnumRegistry {
    EnumRegistry::default()
}

/// The §3.4 environment: the implicit `<edge-type>/strength` rows (D32) plus
/// the authored fields these families read.
fn type_env() -> TypeEnv {
    let v = vocabulary();
    let mut registry = FieldRegistry::with_implicit_edge_strength(&v);
    for source in [
        "(deffield social-class/wealth :type currency :kind extensive)",
        "(deffield social-class/agitation :type intensity :kind intensive)",
        "(deffield social-class/population :type int :kind extensive)",
        "(deffield exploitation/tension :type intensity :kind intensive)",
        "(deffield exploitation/value-flow :type currency :kind extensive)",
        "(deffield economic-sector/output :type currency :kind extensive)",
        "(deffield organization/claim-strength :type coefficient :kind intensive)",
        "(deffield territory/wage-bill :type currency :kind extensive)",
        "(deffield polity/imperial-rent-pool :type currency :kind extensive)",
    ] {
        registry.declare(&e(source), &v, &enums()).expect(source);
    }
    TypeEnv {
        fields: registry.type_env_fields(),
        exemptions: &[],
    }
}

/// The §3.4 verdict for one aggregation shape: `None` accepts, `Some(code)`
/// is the rejection's code.
/// Load one rule through the **composed** pipeline — every gate in §4.6
/// class order, exactly as the engine will.
///
/// The R9 review found this missing: the original suite exercised each new
/// pass in isolation (`check_element_names`, `expr_cost`,
/// `check_selection_scores`), so a rule that every individual pass accepted
/// could still be rejected by `load_rule` — which is what happened to every
/// `:as`-using rule, including §2.6's own worked example. Vectors that
/// claim a construct is *authorable* must go through here.
fn load(source: &str) -> Result<babylon_bsl::LoadedRule, babylon_bsl::LoadError> {
    let v = vocabulary();
    let types = type_env();
    let ceilings = ceilings();
    let intrinsics = IntrinsicCosts::default();
    let systems: HashSet<String> = HashSet::from(["demo".to_owned()]);
    let enums = EnumRegistry::default();
    let const_values = HashMap::new();
    let vocab = BindingVocabulary {
        fields: types.fields.keys().cloned().collect(),
        consts: HashSet::from(["vitality/subsistence-cost".to_owned()]),
        probability_consts: HashSet::new(),
        metrics: HashSet::from(["solidarity-density".to_owned()]),
    };
    let ctx = LoadContext {
        vocabulary: &vocab,
        types: &types,
        enums: &enums,
        const_values: &const_values,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: Some(&v),
        rule_file: "tests/r9_chapters.rs",
    };
    load_rule(source, &ctx)
}

/// The spec code `load_rule` reports for `source`, or `None` if it loads.
fn load_code(source: &str) -> Option<&'static str> {
    match load(source) {
        Ok(_) => None,
        Err(err) => Some(err.spec_code().unwrap_or("<uncoded>")),
    }
}

fn aggregation_code(source: &str) -> Option<TypeCode> {
    match typecheck_aggregation(&e(source), &type_env()) {
        Ok(_) => None,
        Err(err) => Some(err.code.expect("a §3.4 rejection carries its code")),
    }
}

const PREAMBLE: &str =
    ":role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 65536";

fn rule(body: &str) -> String {
    format!("(rule demo/r9 {PREAMBLE} {body})")
}

// ====================================================== family 10 — C1
// Edge and hyperedge attributes (§2.4, §2.5, §2.9, §2.10).
//
// Runtime rows deferred to the query evaluator and named here: the
// `field-of` reads over a live `EdgeRef` / `HyperedgeRef` / `NodeRef`, and
// the `E-EVAL-033` wrong-referent raise. Their codes' identity is pinned
// below; their raise sites arrive with the Phase-2 evaluator.
mod c1_edge_and_hyperedge_attributes {
    use super::{
        aggregation_code, bound, check_foreign_field_scoping, check_intrinsic_name, cost, e, enums,
        vocabulary, ClosedVocabulary, EnumKind, EvalCode, FieldRegistry, TypeCode,
    };
    use babylon_bsl::bindings::parse_bindings;

    /// §2.4's coverage row, both halves: `sum_strength` folds over the
    /// implicit `<edge-type>/strength` (extensive, so `sum` is legal), and
    /// `avg_strength` is its unweighted `mean` — also legal, for the same
    /// reason. An intensive `strength` would have made both `E-TYPE-041`.
    #[test]
    fn the_edge_condition_coverage_row_is_writable_as_sum_and_as_mean() {
        assert_eq!(aggregation_code("(sum solidarity/strength)"), None);
        assert_eq!(aggregation_code("(mean solidarity/strength)"), None);
    }

    /// A genuinely intensive edge attribute carries §3.4's obligation
    /// unchanged: `sum` is `E-TYPE-041` and the unweighted `mean` is
    /// `E-TYPE-042`, while the weighted `mean` accepts.
    #[test]
    fn an_intensive_edge_field_carries_the_weight_obligation() {
        assert_eq!(
            aggregation_code("(sum exploitation/tension)"),
            Some(TypeCode::SumOfIntensive)
        );
        assert_eq!(
            aggregation_code("(mean exploitation/tension)"),
            Some(TypeCode::UnweightedMeanOfIntensive)
        );
        assert!(babylon_bsl::typecheck::typecheck_aggregation(
            &e("(mean exploitation/tension :weight exploitation/value-flow)"),
            &super::type_env()
        )
        .is_ok());
    }

    /// §3.7 (D38): the accessor is a keyed lookup at `1 + operands`, never
    /// multiplied by a ceiling. The §2.10 worked shape's whole bound:
    /// `2 + query(1) + 60 × (field-of(2) + weight field-of(2)) = 243`.
    #[test]
    fn field_of_costs_one_plus_its_operand_and_never_a_ceiling() {
        assert_eq!(cost("(field-of it solidarity/strength)"), Ok(2));
        assert_eq!(cost("(field-of self social-class/wealth)"), Ok(2));
        assert_eq!(
            bound(&super::rule(
                "(bindings) (effects (emit EventType/RUPTURE \
                 (t (fold mean (edges EdgeType/EXPLOITATION) \
                     (field-of it exploitation/tension) \
                     :weight (field-of it exploitation/value-flow)))))"
            )),
            Ok(3 + 243)
        );
    }

    /// D32: re-declaring the implicit `<edge-type>/strength` is a duplicate
    /// field declaration, so its type and kind have exactly one home.
    #[test]
    fn redeclaring_the_implicit_strength_field_is_e_load_001() {
        let v = vocabulary();
        let mut registry = FieldRegistry::with_implicit_edge_strength(&v);
        let err = registry
            .declare(
                &e("(deffield exploitation/strength :type coefficient :kind extensive)"),
                &v,
                &enums(),
            )
            .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
    }

    /// D31: a `deffield` whose first segment names no registered
    /// `NodeType`, `EdgeType` or `HyperedgeType` member.
    #[test]
    fn an_unregistered_field_owner_is_e_load_023() {
        let v = vocabulary();
        let mut registry = FieldRegistry::default();
        let err = registry
            .declare(
                &e("(deffield imperium/rent :type currency :kind extensive)"),
                &v,
                &enums(),
            )
            .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-023"));
    }

    /// D31: the three renderings must be pairwise disjoint, checked once
    /// per content set over the whole vocabulary.
    #[test]
    fn a_node_edge_rendering_collision_is_e_load_032() {
        let err = ClosedVocabulary::new([
            (EnumKind::NodeType, vec!["TENANCY".to_owned()]),
            (EnumKind::EdgeType, vec!["TENANCY".to_owned()]),
        ])
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-032");
    }

    /// D30: the same foreign-type reference under two enclosing bodies of
    /// that type is ambiguous — repaired by naming an element with `:as`.
    #[test]
    fn an_ambiguous_foreign_field_reference_is_e_type_013() {
        let source = super::rule(
            "(bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) \
                        (fold max (nodes NodeType/ORGANIZATION) claim)) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        );
        let form = e(&source);
        let decls = parse_bindings(&form).unwrap();
        let err = check_foreign_field_scoping(&form, &decls, Some("social-class"), &vocabulary())
            .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-013");
    }

    /// D33: every §5.2 form-head symbol is reserved against the intrinsic
    /// namespace, so `(field-of it x/y)` can never be a call.
    #[test]
    fn an_intrinsic_named_for_a_form_head_is_e_load_024() {
        for name in ["field-of", "fold", "the", "edge-between", "select-max"] {
            assert_eq!(
                check_intrinsic_name(name).unwrap_err().spec_code(),
                Some("E-LOAD-024"),
                "{name}"
            );
        }
    }

    /// D34: the accessor's absence discipline has exactly one code, and it
    /// is the same one `E-TYPE-014` degrades to on the update verbs, whose
    /// element is a reference §3.1 gives no static type.
    #[test]
    fn the_accessor_absence_discipline_is_e_eval_033() {
        assert_eq!(
            EvalCode::AccessorTypeOrValueMismatch.spec_code(),
            "E-EVAL-033"
        );
    }
}

// ====================================================== family 11 — C2
// Edge mutation (§2.8's `update-edge`, §2.10's `edge-between`, §3.9's
// hydration key).
//
// **Declared substrate gap, recorded rather than faked.** `update-edge`
// writes a declared field of an edge, and `GraphSubstrate` stores an edge as
// one `f64` strength keyed by `(type, from, to)` — it has no field storage.
// Widening it widens the canonical `state_hash` field set, which is
// hash-relevant and outside this chapter's licence (Constitution III.7). So
// the four `<update-op>` executions, the `E-EVAL-020` range boundary and the
// `E-EVAL-030` I.15 transition are **not** pinned as executions here; the
// verb's grammar, cost, static checks and codes are.
mod c2_edge_mutation {
    use super::{bound, cost, e, vocabulary};
    use babylon_bsl::evaluator::EvalCode;
    use babylon_bsl::grammar::{check_enum_ref_kinds, check_field_init_owners};
    use babylon_bsl::scenario::load_scenario;
    use babylon_graph::memory::MemoryGraph;

    /// D35: `update-edge` is a structural verb and costs like one —
    /// `3 + Σ operands`, with the update-op's own `1 + operand` inside it.
    #[test]
    fn update_edge_is_a_structural_verb_at_the_structural_verb_cost() {
        // 3 + element(1) + qname(0) + scale(1 + 0) = 5, the same shape as
        // §5.6's `update-node`.
        assert_eq!(
            cost("(update-edge it solidarity/strength (scale 0.95c))"),
            Ok(5)
        );
    }

    /// D36: endpoint-holding rules reach the edge through `edge-between`,
    /// which is a keyed lookup — `1 + Σ operands`, never a ceiling factor.
    #[test]
    fn edge_between_costs_one_plus_its_two_endpoint_operands() {
        assert_eq!(cost("(edge-between EdgeType/SOLIDARITY self other)"), Ok(3));
        // The §2.10 worked shape, whole: 3 + edge-between(3) + qname(0) +
        // scale(1 + 0) = 7.
        assert_eq!(
            cost(
                "(update-edge (edge-between EdgeType/SOLIDARITY self other) \
                 solidarity/strength (scale 0.95c))"
            ),
            Ok(7)
        );
    }

    /// §3.8 item 8's writable idiom, priced: a fold over `neighbors` that
    /// resolves each edge by key. `edge-between` is `1 + 0 + 1 + 1 = 3`,
    /// `field-of` is `1 + 3 = 4`, the fold is
    /// `2 + query(1 + self) + ceiling × 4`, and the `emit` around it adds
    /// its structural-verb base of 3. §3.7 charges the extra keyed lookup
    /// per neighbour and nothing more — the accessors never multiply.
    /// The ceiling is 40 (`EdgeType/SOLIDARITY`), which C8's D52 revision
    /// leaves unchanged here: `min(40, 100)` is still 40.
    #[test]
    fn the_self_anchored_endpoint_idiom_is_bounded() {
        assert_eq!(
            bound(&super::rule(
                "(bindings) (effects (emit EventType/RUPTURE \
                 (s (fold sum (neighbors self EdgeType/SOLIDARITY :in \
                                NodeType/SOCIAL_CLASS) \
                      (field-of (edge-between EdgeType/SOLIDARITY it self) \
                                solidarity/strength)))))"
            )),
            Ok(3 + 164)
        );
    }

    /// D37: the `:strength` operand is the implicit field's only writer at
    /// mint time — two writers in one form is an authoring bug.
    #[test]
    fn an_add_edge_field_init_naming_strength_is_e_parse_041() {
        let err = check_field_init_owners(
            &e("(add-edge EdgeType/SOLIDARITY a b :strength 0.5c (solidarity/strength 0.9c))"),
            &vocabulary(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-041");
    }

    /// D37's other half, static on the minting verbs.
    #[test]
    fn an_add_edge_field_init_owning_off_another_type_is_e_type_014() {
        let err = check_field_init_owners(
            &e("(add-edge EdgeType/SOLIDARITY a b :strength 0.5c (social-class/wealth 5$))"),
            &vocabulary(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-014");
    }

    /// D74 at `edge-between`'s operand: a `NodeType` there is a kind error.
    #[test]
    fn an_edge_between_naming_a_node_type_is_e_type_011() {
        let err = check_enum_ref_kinds(&e("(edge-between NodeType/SOCIAL_CLASS a b)")).unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-011");
    }

    /// D73: hydration seeding one `(source, target, type)` triple twice is
    /// `E-LOAD-044` — the clause that makes the triple a **key** rather than
    /// a sort field, and the one §2.6's total order was resting on.
    #[test]
    fn a_hydration_seeding_one_edge_key_twice_is_e_load_044() {
        let source = "(scenario demo/dup
  (node core NodeType/SOCIAL_CLASS)
  (node periphery NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY core periphery 1)
  (edge EdgeType/SOLIDARITY core periphery 1))";
        let mut graph = MemoryGraph::default();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-044"));
    }

    /// The same pair under a DIFFERENT edge type is a different key and
    /// hydrates — the triple is the key, not the pair.
    #[test]
    fn the_same_pair_under_another_edge_type_is_a_different_key() {
        let source = "(scenario demo/two-types
  (node core NodeType/SOCIAL_CLASS)
  (node periphery NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY core periphery 1)
  (edge EdgeType/EXPLOITATION core periphery 1))";
        let mut graph = MemoryGraph::default();
        assert_eq!(load_scenario(source, &mut graph).unwrap().edge_count, 2);
    }

    /// D36: `edge-between` never yields an absent reference and never
    /// degrades to a no-op write — absence has its own code.
    #[test]
    fn an_unresolvable_edge_between_is_e_eval_034() {
        assert_eq!(EvalCode::NoSuchEdge.spec_code(), "E-EVAL-034");
    }
}

// ====================================================== family 12 — C3
// Graph-scope carriers (§2.10's `the`, §3.6's carrier ruling).
//
// Runtime rows deferred: `the`'s resolution against a hydrated carrier, its
// `E-EVAL-035` against an unhydrated one, and the accumulation vector whose
// value is sensitive to §4.2's subject order — all need the query evaluator
// and D44's subject enumeration. Their codes and static gates are pinned.
mod c3_graph_scope_carriers {
    use super::{cost, e};
    use babylon_bsl::evaluator::EvalCode;
    use babylon_bsl::grammar::check_enum_ref_kinds;
    use babylon_bsl::manifest::{check_rule_against_manifest, Manifest};

    const MANIFEST: &str = "(manifest r9
       (ceiling NodeType/SOCIAL_CLASS :ceiling 100)
       (ceiling NodeType/POLITY :ceiling 1)
       (ceiling NodeType/TERRITORY :ceiling 3000 :invariant)
       (ceiling EdgeType/SOLIDARITY :ceiling 40)
       (ceiling EdgeType/EXPLOITATION :ceiling 60)
       (ceiling EdgeType/IN_SCALE :ceiling 5000 :invariant)
       (ceiling HyperedgeType/COMMUNITY :ceiling 200 :max-members 64)
       (ceiling HyperedgeType/ECONOMIC_SECTOR :ceiling 500 :max-members 32))";

    fn manifest() -> Manifest {
        Manifest::parse(&e(MANIFEST)).expect("the R9 manifest is well formed")
    }

    fn carrier_rule(body: &str) -> babylon_bsl::reader::SExpr {
        e(&super::rule(&format!("(bindings) (effects {body})")))
    }

    /// D40 + §3.7's `cost(the) = 1`: reaching a singleton carrier costs a
    /// keyed lookup, NOT the degenerate fold's `2 + query + ceiling × body`
    /// the language previously forced.
    #[test]
    fn the_costs_one_where_the_degenerate_fold_cost_a_ceiling_factor() {
        assert_eq!(cost("(the NodeType/POLITY)"), Ok(1));
        // (update-node (the …) <qname> (sub drawn)) = 3 + 1 + 0 + (1 + 1) = 6,
        // against 2 + 1 + 1 × 2 = 5 for the fold plus the verb's own 3 + …
        assert_eq!(
            cost("(update-node (the NodeType/POLITY) polity/imperial-rent-pool (sub drawn))"),
            Ok(6)
        );
    }

    /// D40: legality is conditioned on the manifest's `:ceiling` being
    /// exactly 1 — the same declared number §3.7 already uses for the fuel
    /// bound, so the ruling adds no second registry.
    #[test]
    fn the_against_a_ceiling_one_carrier_loads_and_otherwise_is_e_load_043() {
        assert_eq!(
            check_rule_against_manifest(
                &carrier_rule(
                    "(update-node (the NodeType/POLITY) polity/imperial-rent-pool (sub 5$))"
                ),
                &manifest()
            ),
            Ok(())
        );
        let err = check_rule_against_manifest(
            &carrier_rule("(update-node (the NodeType/SOCIAL_CLASS) social-class/wealth (add 5$))"),
            &manifest(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-043"));
    }

    /// A carrier field is read and written as ORDINARY node state — no new
    /// grammar and no second storage class (D39).
    #[test]
    fn a_carrier_field_reads_with_field_of_and_writes_with_update_node() {
        assert_eq!(
            check_rule_against_manifest(
                &carrier_rule(
                    "(guard (< (field-of (the NodeType/POLITY) polity/imperial-rent-pool) 5$) \
                     (update-node (the NodeType/POLITY) polity/imperial-rent-pool (set 0$)))"
                ),
                &manifest()
            ),
            Ok(())
        );
    }

    /// D74 at `the`'s operand.
    #[test]
    fn the_against_an_edge_type_is_e_type_011() {
        let err = check_enum_ref_kinds(&e("(the EdgeType/SOLIDARITY)")).unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-011");
    }

    /// D76: a carrier type the manifest has no row for is `E-LOAD-045` —
    /// `E-LOAD-043`'s "other than 1" test cannot fire on a missing row, so
    /// the omission must be its own rejection.
    #[test]
    fn a_manifest_with_no_row_for_the_carrier_type_is_e_load_045() {
        let err = check_rule_against_manifest(
            &carrier_rule("(update-node (the NodeType/SOVEREIGN) social-class/wealth (add 5$))"),
            &manifest(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-045"));
    }

    /// D76's other half, in the bound checker: a queried type with no row
    /// carries the same code, because `ceiling(query)` is not computable
    /// without it.
    #[test]
    fn a_queried_type_with_no_manifest_row_is_e_load_045_in_the_bound_checker() {
        let err = super::cost("(fold sum (nodes NodeType/SOVEREIGN) it)").unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-045"));
    }

    /// D40's runtime half: a carrier the scenario forgot to hydrate fails
    /// loudly rather than reading as zero.
    #[test]
    fn an_unhydrated_carrier_is_e_eval_035() {
        assert_eq!(EvalCode::UnhydratedCarrier.spec_code(), "E-EVAL-035");
    }
}

// ====================================================== family 13 — C4
// The rule domain (§2.3) — what a rule fires over, and how many times.
//
// Runtime row deferred: `(domain :graph)` *firing exactly once* against a
// multi-node graph needs the tick loop's subject enumeration (D44). The
// static resolution that decides it is pinned here in full.
mod c4_rule_domain {
    use super::{e, vocabulary};
    use babylon_bsl::bindings::parse_bindings;
    use babylon_bsl::canonical_ast::canonical_bytes;
    use babylon_bsl::domain::{resolve_domain, DomainError, RuleDomain};
    use babylon_bsl::grammar::{check_enum_ref_kinds, check_graph_flag_placement};

    fn domain_of(body: &str) -> Result<RuleDomain, DomainError> {
        let form = e(&super::rule(body));
        let decls = parse_bindings(&form).expect("bindings must parse");
        resolve_domain(&form, &decls, &vocabulary())
    }

    /// §5.6's rule, unchanged, is an inferred node domain — the property
    /// D43 relies on to keep the pinned canonical bytes valid.
    #[test]
    fn the_worked_example_infers_a_node_domain_and_keeps_its_bytes() {
        let worked = "(rule demo/hunger \
             :role mechanic :evidence derived :material-basis \"subsistence deficit at the point of reproduction\" \
             :fuel 64 \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 1000.5$)) \
             (effects (update-node self social-class/agitation (add 0.05i))))";
        let form = e(worked);
        let decls = parse_bindings(&form).unwrap();
        assert_eq!(
            resolve_domain(&form, &decls, &vocabulary()),
            Ok(RuleDomain::Node("social-class".to_owned()))
        );
        assert_eq!(
            canonical_bytes(&form).unwrap().len(),
            500,
            "§5.6's bytes retain the optional, absent <domain>"
        );
    }

    /// `|U| = 0` — nothing is self-scoped.
    #[test]
    fn no_self_scoped_reference_and_no_domain_is_e_load_004() {
        let err = domain_of(
            "(bindings (binding now :tick)) (when (< now 5)) \
             (effects (emit EventType/RUPTURE (t now)))",
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-004"));
    }

    /// `|U| > 1` — two node types are.
    #[test]
    fn two_self_scoped_node_types_is_e_load_004() {
        let err = domain_of(
            "(bindings (binding wealth :field social-class/wealth) \
                       (binding claim :field organization/claim-strength)) \
             (when (< wealth claim)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-004"));
    }

    /// An explicit `<domain>` replaces the inference outright; the second
    /// type is then a foreign read, which is `E-TYPE-010` and not an
    /// ambiguity.
    #[test]
    fn an_explicit_domain_overrides_the_inference_and_disagreement_is_e_type_010() {
        let err = domain_of(
            "(domain NodeType/SOCIAL_CLASS) \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< claim 5)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-TYPE-010"));
    }

    /// D43's stated property, proved: a binding referenced only inside a
    /// fold body never enters `U`, so adding one cannot change how many
    /// times a rule fires.
    #[test]
    fn a_fold_scoped_binding_never_changes_the_firing_multiplicity() {
        assert_eq!(
            domain_of(
                "(domain :graph) \
                 (bindings (binding claim :field organization/claim-strength)) \
                 (when (< (fold sum (nodes NodeType/ORGANIZATION) claim) 5)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            ),
            Ok(RuleDomain::Graph)
        );
    }

    /// `self` is not bound in a graph-domain rule.
    #[test]
    fn self_in_a_graph_domain_rule_is_e_type_015() {
        let err = domain_of(
            "(domain :graph) (bindings) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-TYPE-015"));
    }

    /// …and neither is a `:field` binding read outside a query body.
    #[test]
    fn a_rule_scope_field_read_in_a_graph_domain_rule_is_e_type_015() {
        let err = domain_of(
            "(domain :graph) \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 5)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-TYPE-015"));
    }

    /// D42: `:graph` is a flag of the `domain` form and illegal elsewhere.
    #[test]
    fn graph_outside_a_domain_form_is_e_parse_013() {
        let err = check_graph_flag_placement(&e(&super::rule(
            "(bindings (binding g :metric density :graph)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        )))
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-013");
    }

    /// D74 at `domain`'s operand.
    #[test]
    fn a_domain_naming_an_edge_type_is_e_type_011() {
        let err = check_enum_ref_kinds(&e("(domain EdgeType/SOLIDARITY)")).unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-011");
    }

    /// §5.3 (D43): `<domain>` encodes in its grammar position, an enum-ref
    /// domain as one `atom enum` child and `(domain :graph)` as one `opt`
    /// form carrying the flag under D20 — the two shapes are distinct bytes
    /// and neither is a keyword in value position.
    #[test]
    fn both_domain_shapes_have_distinct_canonical_bytes() {
        let typed = canonical_bytes(&e("(domain NodeType/SOCIAL_CLASS)")).unwrap();
        let graph = canonical_bytes(&e("(domain :graph)")).unwrap();
        assert_ne!(typed, graph);
        // D20's flag shape, assembled from §5.1-§5.2 rather than compared
        // against `(domain :graph #t)` — `:graph` takes no operand (§1.6),
        // so that spelling is not BSL and the old comparison was asserting
        // the encoder's adjacency heuristic rather than the spec.
        let mut expected = Vec::new();
        expected.extend_from_slice(&[0x02, 6]);
        expected.extend_from_slice(b"domain");
        expected.extend_from_slice(&1u32.to_be_bytes());
        expected.extend_from_slice(&[0x02, 3]);
        expected.extend_from_slice(b"opt");
        expected.extend_from_slice(&2u32.to_be_bytes());
        expected.extend_from_slice(&[0x01, 2]);
        expected.extend_from_slice(b"kw");
        expected.extend_from_slice(&5u32.to_be_bytes());
        expected.extend_from_slice(b"graph");
        expected.extend_from_slice(&[0x01, 4]);
        expected.extend_from_slice(b"bool");
        expected.extend_from_slice(&1u32.to_be_bytes());
        expected.push(0x01);
        assert_eq!(graph, expected);
    }
}

// ====================================================== family 16 — C7
// Computed bindings (§2.5's `:expr`), landed before C5 because §2.7's score
// classifier resolves a score written as a binding name through its
// declared source, which needs `BindSource::Expr` to exist. The rst's §7
// order is a dependency order; this is one it does not name.
mod c7_computed_bindings {
    use super::{bound, e};
    use super::{enums, type_env};
    use babylon_bsl::bindings::{parse_bindings, BindSource};
    use babylon_bsl::evaluator::Value;
    use babylon_bsl::fuel::IntrinsicCosts;
    use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
    use babylon_bsl::rule_pipeline::resolve_expr_bindings;
    use babylon_kernel::Currency;
    use std::collections::HashMap;

    fn decls(bindings: &str) -> Vec<babylon_bsl::bindings::BindingDecl> {
        let form = e(&super::rule(&format!(
            "(bindings {bindings}) \
             (effects (update-node self social-class/agitation (add 0.05i)))"
        )));
        parse_bindings(&form).expect("bindings must parse")
    }

    fn binding_error(bindings: &str) -> Option<&'static str> {
        let form = e(&super::rule(&format!(
            "(bindings {bindings}) \
             (effects (update-node self social-class/agitation (add 0.05i)))"
        )));
        parse_bindings(&form).err().and_then(|e| e.spec_code())
    }

    /// §2.5's own worked example: a rule may name an intermediate value.
    #[test]
    fn an_expr_binding_names_a_computed_value() {
        let d = decls(
            "(binding wealth      :field social-class/wealth) \
             (binding subsistence :const vitality/subsistence-cost) \
             (binding drained     :expr (- wealth subsistence))",
        );
        assert!(matches!(d[2].source, BindSource::Expr(_)));
    }

    /// D49: resolution is in declaration order; a forward reference and a
    /// self-reference are both `E-PARSE-032`, so no cycle is expressible
    /// and nothing needs a cycle analysis.
    #[test]
    fn a_forward_or_self_reference_is_e_parse_032() {
        assert_eq!(
            binding_error(
                "(binding drained :expr (- wealth 1$)) \
                 (binding wealth  :field social-class/wealth)"
            ),
            Some("E-PARSE-032")
        );
        assert_eq!(
            binding_error("(binding loop :expr (+ loop 1))"),
            Some("E-PARSE-032")
        );
    }

    /// D49: `:optional`/`:default` on a `:expr` — a computed value is never
    /// absent, because its operands were resolved at load or the rule did
    /// not load.
    #[test]
    fn optional_or_default_on_an_expr_is_e_parse_033() {
        assert_eq!(
            binding_error("(binding d :expr (+ 1 2) :optional :default 0)"),
            Some("E-PARSE-033")
        );
        assert_eq!(
            binding_error("(binding d :expr (+ 1 2) :default 0)"),
            Some("E-PARSE-033")
        );
    }

    /// D50: `bound(rule)` gains `Σ cost(:expr bindings)`, and every other
    /// bind-src still contributes nothing — the property §5.6's pinned
    /// `bound = 7` rests on.
    #[test]
    fn only_expr_bindings_enter_the_static_bound() {
        let external = bound(&super::rule(
            "(bindings (binding wealth :field social-class/wealth) (binding now :tick)) \
             (when (< wealth 1000.5$)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        assert_eq!(external, Ok(7), "§5.6's worked bound, unmoved");
        // + cost((- wealth 1000.5$)) = 1 + 1 + 0 = 2.
        let computed = bound(&super::rule(
            "(bindings (binding wealth :field social-class/wealth) \
                       (binding drained :expr (- wealth 1000.5$))) \
             (when (< wealth 1000.5$)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        assert_eq!(computed, Ok(9));
    }

    /// §4.5's asymmetry, executed: the expression is charged **once** at
    /// binding time and each later reference charges a variable-reference 1,
    /// so the same algebra written twice inline costs strictly more.
    #[test]
    fn a_named_expression_costs_strictly_less_than_the_same_algebra_twice() {
        let costs = IntrinsicCosts::default();
        let supplied = |name: &str| {
            HashMap::from([(
                name.to_owned(),
                Value::Currency(Currency::from_micro_units(900_000_000)),
            )])
        };

        // Named once: charge the expression (1 + 1 + 0 = 2) at binding
        // time, then 1 per reference in the two comparisons below.
        let named = decls(
            "(binding wealth  :field social-class/wealth) \
             (binding drained :expr (- wealth 100$))",
        );
        let mut env = supplied("wealth");
        let mut fuel_named = 1_000;
        resolve_expr_bindings(
            &named,
            &mut env,
            &costs,
            &type_env(),
            &enums(),
            None,
            None,
            &EmptyIntrinsicHost,
            &mut fuel_named,
        )
        .expect("the :expr must resolve");
        assert_eq!(1_000 - fuel_named, 2, "charged once, at binding time");
        assert!(env.contains_key("drained"));

        // Written inline twice, the same algebra is charged twice.
        let inline = decls(
            "(binding wealth :field social-class/wealth) \
             (binding a :expr (- wealth 100$)) \
             (binding b :expr (- wealth 100$))",
        );
        let mut env2 = supplied("wealth");
        let mut fuel_inline = 1_000;
        resolve_expr_bindings(
            &inline,
            &mut env2,
            &costs,
            &type_env(),
            &enums(),
            None,
            None,
            &EmptyIntrinsicHost,
            &mut fuel_inline,
        )
        .expect("both :exprs must resolve");
        assert!(
            1_000 - fuel_inline > 1_000 - fuel_named,
            "restating the algebra costs strictly more (§4.5)"
        );
    }

    /// A `:expr` is evaluated at rule scope, so a foreign-node-type
    /// `:field` reference inside one is `E-TYPE-010` — the same rule as
    /// anywhere else at rule scope.
    #[test]
    fn a_foreign_field_inside_an_expr_is_e_type_010() {
        let form = e(&super::rule(
            "(bindings (binding claim :field organization/claim-strength) \
                       (binding doubled :expr (+ claim claim))) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        let d = parse_bindings(&form).unwrap();
        let err = babylon_bsl::scope::check_foreign_field_scoping(
            &form,
            &d,
            Some("social-class"),
            &super::vocabulary(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-010");
    }
}

// ============================================= family 22 (bindings half) — C13
// The calendar bind-srcs (§2.5, D68). The rest of family 22 — the intrinsic
// cap and the RNG carrier key — lands with C13 proper.
mod c13_calendar_bindings {
    use super::e;
    use babylon_bsl::bindings::{parse_bindings, BindSource};

    fn decls_of(bindings: &str) -> Result<Vec<babylon_bsl::bindings::BindingDecl>, &'static str> {
        let form = e(&super::rule(&format!(
            "(bindings {bindings}) \
             (effects (update-node self social-class/agitation (add 0.05i)))"
        )));
        parse_bindings(&form).map_err(|err| err.spec_code().unwrap_or("<uncoded>"))
    }

    /// D68: calendar reads are **bindings, not arithmetic** — a kernel
    /// seam, which is the category R10 sanctions without a rider. No `mod`
    /// and no `floor-div` arrive behind them.
    #[test]
    fn the_three_calendar_bind_srcs_parse_and_bind_int() {
        let d = decls_of(
            "(binding y :year) (binding toy :tick-of-year) (binding phase :tick-in-cycle 52)",
        )
        .expect("the calendar bind-srcs are §2.5 productions");
        assert_eq!(d[0].source, BindSource::Year);
        assert_eq!(d[1].source, BindSource::TickOfYear);
        assert_eq!(d[2].source, BindSource::TickInCycle(52));
    }

    /// §1.6: the cycle length must be `> 0`.
    #[test]
    fn a_zero_or_negative_cycle_length_is_e_parse_014() {
        assert_eq!(decls_of("(binding p :tick-in-cycle 0)"), Err("E-PARSE-014"));
        assert_eq!(
            decls_of("(binding p :tick-in-cycle -4)"),
            Err("E-PARSE-014")
        );
    }

    /// D68's bound, stated precisely: the length is a **literal**, so the
    /// value is a static function of the tick and the content bytes — an
    /// expression there is not expressible, which is what stops a general
    /// mod operator over arbitrary expressions arriving behind the seam.
    #[test]
    fn a_computed_cycle_length_is_not_expressible() {
        assert!(decls_of("(binding p :tick-in-cycle (+ 26 26))").is_err());
    }
}

// ====================================================== family 14 — C5
// Element selection (§2.7's `select-max` / `select-min`).
//
// **PR 4, Task 13 (2026-08-11): EXECUTES.** `select-max`/`select-min` over
// the two heads slice 1 serves (`nodes`, `neighbors`) run for real below:
// the tie vector, the `E-EVAL-021` empty-query RAISE (not merely the
// code's identity), an intensive score's ranking, and a selection feeding
// both `field-of` and `update-node`. The other four query heads (`edges`,
// `hyperedges`, `members-of`, `hyperedges-of`) still refuse at evaluation,
// each naming its own slice (2 or 3) — pinned as a real refusal, not a
// silent skip. The result-type rule, the `E-TYPE-016` score rule and the
// §3.7 cost row stay load-time static, exactly as before.
mod c5_element_selection {
    use super::{cost, e, enums, type_env};
    use babylon_bsl::bindings::parse_bindings;
    use babylon_bsl::evaluator::{evaluate, EvalCode, EvalEnv, EvalError, Value};
    use babylon_bsl::fuel::IntrinsicCosts;
    use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
    use babylon_bsl::score_class::{selection_result_class, ScoreClass};
    use babylon_bsl::structural_verbs::{CollectingSink, EffectExecutor};
    use babylon_bsl::typecheck::{check_selection_scores, TypeCode};
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::substrate::GraphSubstrate;
    use std::collections::HashMap;

    /// Evaluate one `<expr>` (a bare fragment, not a whole rule) against a
    /// graph and a supplied binding map — the shared seam every real
    /// evaluation vector below drives through, mirroring the pattern
    /// `evaluator.rs`'s own `#[cfg(test)]` module uses (`eval_over`) at
    /// this conformance-family level instead of the implementation level.
    fn eval_expr(
        source: &str,
        graph: &dyn GraphSubstrate,
        bindings: HashMap<String, Value>,
        fuel: &mut u64,
    ) -> Result<Value, EvalError> {
        let costs = IntrinsicCosts::default();
        let types = type_env();
        let enums = enums();
        // PR A verifier fix round (2026-08-12): `field_of_node` now
        // refuses loudly on a `None` types/enums pair (mirroring
        // `require_graph`) rather than silently degrading to `Value::
        // Real` — this family's own vectors drive `field-of` through this
        // helper, so it needs the real registries `type_env()`/`enums()`
        // already provide elsewhere in this file, not the untyped shape.
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(graph),
            types: Some(&types),
            enums: Some(&enums),
            elements: Vec::new(),
            draw_context: None,
        };
        evaluate(&e(source), &env, &EmptyIntrinsicHost, fuel)
    }

    fn score_error(body: &str) -> Option<TypeCode> {
        let form = e(&super::rule(body));
        let decls = parse_bindings(&form).expect("bindings must parse");
        check_selection_scores(&form, &type_env(), &decls)
            .err()
            .and_then(|err| err.code)
    }

    /// §2.7: the result type is the query's ELEMENT type — `NodeRef` for
    /// `nodes`/`neighbors`/`members-of`, `EdgeRef` for `edges`,
    /// `HyperedgeRef` for `hyperedges`/`hyperedges-of`. All six heads.
    #[test]
    fn the_result_type_is_the_querys_element_type_for_all_six_heads() {
        let cases = [
            ("(nodes NodeType/SOCIAL_CLASS)", ScoreClass::NodeReference),
            (
                "(neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)",
                ScoreClass::NodeReference,
            ),
            (
                "(members-of h HyperedgeType/COMMUNITY)",
                ScoreClass::NodeReference,
            ),
            ("(edges EdgeType/SOLIDARITY)", ScoreClass::EdgeReference),
            (
                "(hyperedges HyperedgeType/COMMUNITY)",
                ScoreClass::HyperedgeReference,
            ),
            (
                "(hyperedges-of self HyperedgeType/COMMUNITY)",
                ScoreClass::HyperedgeReference,
            ),
        ];
        for (query, expected) in cases {
            assert_eq!(selection_result_class(&e(query)), expected, "{query}");
        }
    }

    /// D46: the score must be a comparable scalar. A `Bool` and an
    /// `Enum<T>` score are `E-TYPE-016`.
    #[test]
    fn a_bool_or_enum_score_is_e_type_016() {
        assert_eq!(
            score_error(
                "(bindings) (effects (update-node \
                 (select-max (nodes NodeType/ORGANIZATION) (< 1 2)) \
                 social-class/agitation (add 0.05i)))"
            ),
            Some(TypeCode::NonComparableScore)
        );
        assert_eq!(
            score_error(
                "(bindings) (effects (update-node \
                 (select-min (nodes NodeType/ORGANIZATION) NodeType/POLITY) \
                 social-class/agitation (add 0.05i)))"
            ),
            Some(TypeCode::NonComparableScore)
        );
        // A reference score is rejected for the same reason: there is no
        // ordering on references in the language (D67).
        assert_eq!(
            score_error(
                "(bindings) (effects (update-node \
                 (select-max (nodes NodeType/ORGANIZATION) it) \
                 social-class/agitation (add 0.05i)))"
            ),
            Some(TypeCode::NonComparableScore)
        );
    }

    /// D46's other half, proved by acceptance: **kind is unconstrained on
    /// the score**. `organization/claim-strength` is declared intensive and
    /// ranking by it must ACCEPT — §3.4 polices aggregation, and ordering
    /// aggregates nothing, so the weighted-mean obligation has nothing to
    /// attach to.
    #[test]
    fn an_intensive_score_accepts_because_ordering_is_not_aggregation() {
        assert_eq!(
            score_error(
                "(bindings) (effects (update-node \
                 (select-max (nodes NodeType/ORGANIZATION) \
                             (field-of it organization/claim-strength)) \
                 social-class/agitation (add 0.05i)))"
            ),
            None
        );
    }

    /// §3.7's row: `2 + cost(query) + ceiling(query) × cost(score)` — the
    /// same shape as a fold with one body and no weight.
    #[test]
    fn a_selection_costs_two_plus_query_plus_ceiling_times_score() {
        // 2 + query(1) + 40 × field-of(2) = 83.
        assert_eq!(
            cost(
                "(select-max (nodes NodeType/ORGANIZATION) \
                 (field-of it organization/claim-strength))"
            ),
            Ok(83)
        );
        // Named with `:as`: `cost(:as name) = 0` (§3.7), so the bound is
        // unchanged — a reference to the name would cost 1 like any other.
        assert_eq!(
            cost(
                "(select-max (nodes NodeType/ORGANIZATION) :as winner \
                 (field-of winner organization/claim-strength))"
            ),
            Ok(83)
        );
    }

    // -------------------------------------------------- PR 4 Task 13: EXECUTES

    /// `select-max`/`select-min` run for real over BOTH heads slice 1
    /// serves: `nodes` and `neighbors`. The `neighbors` case also proves
    /// the D24 filter composes with a selection — only the annotated
    /// `NodeType` is eligible to win.
    #[test]
    fn select_max_and_select_min_execute_over_nodes_and_neighbors() {
        let mut graph = MemoryGraph::new();
        let low = graph.add_node("ORGANIZATION").unwrap();
        let high = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(low, "organization/claim-strength", 0.2)
            .unwrap();
        graph
            .update_node(high, "organization/claim-strength", 0.9)
            .unwrap();
        let mut fuel = 1_000;
        let result = eval_expr(
            "(select-max (nodes NodeType/ORGANIZATION) \
             (field-of it organization/claim-strength))",
            &graph,
            HashMap::new(),
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::NodeRef(high));

        // Over `neighbors`: `self` reaches both ORGANIZATION nodes via
        // SOLIDARITY; the NodeType annotation is the only filter and both
        // qualify, so the selection picks between them exactly as it did
        // over the bare `nodes` query above.
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, low, 1.0).unwrap();
        graph.add_edge("SOLIDARITY", subject, high, 1.0).unwrap();
        let mut fuel2 = 1_000;
        let result2 = eval_expr(
            "(select-min (neighbors self EdgeType/SOLIDARITY :out NodeType/ORGANIZATION) \
             (field-of it organization/claim-strength))",
            &graph,
            HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            &mut fuel2,
        )
        .unwrap();
        assert_eq!(result2, Value::NodeRef(low));
    }

    /// The three hyperedge §2.6 query heads are SERVED (slice 3, Community
    /// Task 3, 2026-08-22) — this test's old "each names its slice"
    /// refusal is retired; what is pinned now is each head's honest
    /// behavior under a selection on an empty or wrongly-typed result:
    /// `hyperedges`/`hyperedges-of` (a node's memberships — none seeded)
    /// refuse the EMPTY query with the §4.4/D45 text; `members-of self`
    /// refuses the operand kind (self is a `NodeRef`; the head wants a
    /// `HyperedgeRef`, §3.1) — never a silent skip, never an
    /// `E-LOAD-021` misdiagnosis, on any of the three. `edges` left this
    /// set at T2 (issue #559) — see
    /// `edge_count_evaluates_for_real_on_an_empty_graph`
    /// (`conformance_corpus.rs`) for its own positive vector.
    #[test]
    fn the_served_hyperedge_heads_refuse_empty_or_mistyped_results_honestly() {
        // `self` binds to a REAL node: were it a dangling id, a future
        // referent-validation pass could fire before the slice refusal
        // and this vector would pin the wrong error (Copilot harvest,
        // #520).
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        for (query, expected) in [
            (
                "(hyperedges HyperedgeType/ECONOMIC_SECTOR)",
                "select-max over an empty query",
            ),
            (
                "(members-of self HyperedgeType/ECONOMIC_SECTOR)",
                "must evaluate to a HyperedgeRef",
            ),
            (
                "(hyperedges-of self HyperedgeType/ECONOMIC_SECTOR)",
                "select-max over an empty query",
            ),
        ] {
            let mut fuel = 1_000;
            let err = eval_expr(
                &format!("(select-max {query} it)"),
                &graph,
                HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
                &mut fuel,
            )
            .unwrap_err();
            assert!(err.message.contains(expected), "{query}: {err}");
        }
    }

    /// D45: the tiebreak is a property of the LANGUAGE, not of each rule —
    /// the first element in §2.6's ascending id byte order wins, for both
    /// operators, EXECUTED against two equally-scored elements.
    #[test]
    fn the_tie_vector_breaks_to_the_smaller_id_for_both_operators() {
        let mut graph = MemoryGraph::new();
        let first = graph.add_node("ORGANIZATION").unwrap(); // id 0
        let second = graph.add_node("ORGANIZATION").unwrap(); // id 1
        graph
            .update_node(first, "organization/claim-strength", 0.5)
            .unwrap();
        graph
            .update_node(second, "organization/claim-strength", 0.5)
            .unwrap();
        for op in ["select-max", "select-min"] {
            let mut fuel = 1_000;
            let result = eval_expr(
                &format!(
                    "({op} (nodes NodeType/ORGANIZATION) \
                     (field-of it organization/claim-strength))"
                ),
                &graph,
                HashMap::new(),
                &mut fuel,
            )
            .unwrap();
            assert_eq!(
                result,
                Value::NodeRef(first),
                "{op}: the smaller id wins a tie"
            );
        }
    }

    /// D45/§4.4: an empty query RAISES `E-EVAL-021` — not merely shares the
    /// code's identity, as the retired pin only proved.
    #[test]
    fn selection_over_an_empty_query_raises_e_eval_021() {
        let graph = MemoryGraph::new();
        for op in ["select-max", "select-min"] {
            let mut fuel = 1_000;
            let err = eval_expr(
                &format!(
                    "({op} (nodes NodeType/ORGANIZATION) \
                     (field-of it organization/claim-strength))"
                ),
                &graph,
                HashMap::new(),
                &mut fuel,
            )
            .unwrap_err();
            assert_eq!(err.code, Some(EvalCode::EmptyAggregate), "{op}: {err}");
            assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-021");
        }
    }

    /// D46's acceptance half, EXECUTED: an intensive score ranks correctly
    /// at evaluation — there is no evaluator-level kind check to enforce
    /// (no `TypeEnv` reaches this far), exactly as §2.7 says §3.4 polices
    /// aggregation, never ordering.
    #[test]
    fn an_intensive_score_ranks_correctly_at_evaluation() {
        let mut graph = MemoryGraph::new();
        let mut ids = Vec::new();
        for i in 0..4 {
            let id = graph.add_node("ORGANIZATION").unwrap();
            graph
                .update_node(id, "organization/claim-strength", f64::from(i))
                .unwrap();
            ids.push(id);
        }
        let mut fuel = 1_000;
        let result = eval_expr(
            "(select-max (nodes NodeType/ORGANIZATION) \
             (field-of it organization/claim-strength))",
            &graph,
            HashMap::new(),
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::NodeRef(*ids.last().unwrap()));
    }

    /// A selection result feeds BOTH consumers §2.7 names: `field-of` (a
    /// read) and `update-node` (a write, Task 11's own concern) — EXECUTED
    /// through the production collect-then-apply path
    /// (`EffectExecutor::collect_effects` + `apply_pending_write`), not
    /// merely accepted at load.
    #[test]
    fn a_selection_feeds_field_of_and_update_node() {
        let mut graph = MemoryGraph::new();
        let low = graph.add_node("ORGANIZATION").unwrap();
        let high = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(low, "organization/claim-strength", 0.2)
            .unwrap();
        graph
            .update_node(high, "organization/claim-strength", 0.9)
            .unwrap();

        // field-of over the selection result — the §2.7 worked example's
        // read half.
        let mut fuel = 1_000;
        let read_back = eval_expr(
            "(field-of \
               (select-max (nodes NodeType/ORGANIZATION) \
                            (field-of it organization/claim-strength)) \
               organization/claim-strength)",
            &graph,
            HashMap::new(),
            &mut fuel,
        )
        .unwrap();
        assert_eq!(read_back, Value::Real(0.9));

        // update-node against the selection result — the write half,
        // driven through the SAME collect-then-apply production path
        // `tick.rs::run_tick` uses (Task 12).
        let (form, _) = babylon_bsl::reader::read(
            "(effects (update-node \
               (select-max (nodes NodeType/ORGANIZATION) \
                            (field-of it organization/claim-strength)) \
               organization/claim-strength (set 0.5c)))",
        )
        .expect("must parse");
        let babylon_bsl::reader::SExpr::List(items) = form else {
            unreachable!()
        };
        let types = type_env();
        let enum_registry = enums();
        // PR A verifier fix round (2026-08-12): same coincidental-safety
        // gap as this module's other direct `EvalEnv` constructions —
        // `types`/`enum_registry` were already built for `EffectExecutor`
        // below.
        let env = EvalEnv {
            bindings: HashMap::new(),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: Some(&graph as &dyn GraphSubstrate),
            types: Some(&types),
            enums: Some(&enum_registry),
            elements: Vec::new(),
            draw_context: None,
        };
        let mut executor = EffectExecutor::new(&types, &enum_registry, None);
        let mut sink = CollectingSink::default();
        let mut fuel2 = 1_000;
        let pending = executor
            .collect_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut sink,
                &mut fuel2,
            )
            .unwrap();
        // Pass 2 uses a FRESH executor, exactly as `tick.rs::run_tick`
        // does — the apply half must not depend on any state the
        // collecting executor accumulated (Copilot harvest, #520).
        let mut apply_executor = EffectExecutor::new(&types, &enum_registry, None);
        for write in &pending {
            apply_executor
                .apply_pending_write(write, &mut graph)
                .unwrap();
        }
        let selected = graph
            .node_attribute(high, "organization/claim-strength")
            .unwrap();
        assert!(
            (selected - 0.5).abs() < 1e-12,
            "the SELECTED (higher-scoring) node was written"
        );
        let untouched = graph
            .node_attribute(low, "organization/claim-strength")
            .unwrap();
        assert!(
            (untouched - 0.2).abs() < 1e-12,
            "the non-selected node was left alone"
        );
    }
}

// ====================================================== family 15 — C6
// Effect-position iteration (§2.8's `for-each`).
//
// **PR 4, Task 13 (2026-08-11): EXECUTES.** `for-each` over `nodes`/
// `neighbors` applies `update-node` and `emit` per element for real below,
// through the SAME collect-then-apply production path `tick.rs::run_tick`
// uses (Task 12): the pre-state materialization proof, the empty-query
// quiet case, and a real nested `for-each`. `update-edge`'s per-element
// results stay pinned — a DIFFERENT, orthogonal gap (C2's declared
// substrate-storage refusal, Constitution III.7), not a query-evaluator
// gap, so the three static cost vectors below (which only ever computed
// §3.7's bound, never executed) keep `update-edge` in their body
// unchanged. The grammar, the arity, the static bound and the
// `E-LOAD-040` interaction stay pinned exactly as before.
mod c6_effect_position_iteration {
    use super::{bound, cost, enums, type_env};
    use babylon_bsl::evaluator::{EvalEnv, EvalError, Value};
    use babylon_bsl::fuel::IntrinsicCosts;
    use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
    use babylon_bsl::structural_verbs::{CollectingSink, EffectExecutor};
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::substrate::GraphSubstrate;
    use std::collections::HashMap;

    /// §3.7's row: `2 + cost(query) + ceiling(query) × Σ cost(effect-items)`
    /// — charged exactly as `exists`/`forall` are, which is what keeps the
    /// totality argument syntactic rather than analysed.
    #[test]
    fn for_each_is_charged_like_exists_and_forall() {
        // 2 + query(1) + 40 × update-edge(3 + 1 + 0 + (1 + 0)) = 203.
        assert_eq!(
            cost(
                "(for-each (edges EdgeType/SOLIDARITY) \
                 (update-edge it solidarity/strength (scale 0.95c)))"
            ),
            Ok(203)
        );
    }

    /// The §2.8 worked shape: two effect items per element, both inside the
    /// one ceiling factor.
    #[test]
    fn the_body_is_summed_inside_the_single_ceiling_factor() {
        // update-edge(5) + emit(3 + field-of(2)) = 10; 2 + 1 + 40 × 10 = 403.
        assert_eq!(
            cost(
                "(for-each (edges EdgeType/SOLIDARITY) \
                 (update-edge it solidarity/strength (scale 0.95c)) \
                 (emit EventType/RUPTURE (strength (field-of it solidarity/strength))))"
            ),
            Ok(403)
        );
    }

    /// Nested `for-each` composes the same way, and its static bound is the
    /// product of the two ceilings — bounded, not a loop.
    #[test]
    fn nested_for_each_multiplies_its_two_ceilings() {
        // inner: 2 + 1 + 40 × 5 = 203; outer: 2 + 1 + 100 × 203 = 20303.
        assert_eq!(
            cost(
                "(for-each (nodes NodeType/SOCIAL_CLASS) \
                 (for-each (edges EdgeType/SOLIDARITY) \
                  (update-edge it solidarity/strength (scale 0.95c))))"
            ),
            Ok(20_303)
        );
    }

    /// A `for-each` one short of its static bound is rejected at LOAD
    /// (`E-LOAD-040`), which is the whole point of §3.7 being static.
    #[test]
    fn a_for_each_one_short_of_its_bound_is_e_load_040() {
        use babylon_bsl::bound_checker::check_rule;
        use babylon_bsl::fuel::IntrinsicCosts;
        let body = "(bindings) (effects (for-each (edges EdgeType/SOLIDARITY) \
                    (update-edge it solidarity/strength (scale 0.95c))))";
        let starved =
            format!("(rule demo/r9 :role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 202 {body})");
        let err = check_rule(
            &super::e(&starved),
            &super::ceilings(),
            &IntrinsicCosts::default(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-040"));
        // …and one MORE than the bound loads (§4.5's off-by-one: budget
        // `bound + 1`, because the meter must stay strictly positive).
        let funded =
            format!("(rule demo/r9 :role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 204 {body})");
        assert_eq!(
            check_rule(
                &super::e(&funded),
                &super::ceilings(),
                &IntrinsicCosts::default()
            ),
            Ok(203)
        );
    }

    /// The bound composes through a rule: `bound(rule)` sums its effect
    /// items, so a `for-each` is simply one of them.
    #[test]
    fn a_for_each_enters_bound_rule_as_one_effect_item() {
        assert_eq!(
            bound(&super::rule(
                "(bindings) (effects \
                 (for-each (edges EdgeType/SOLIDARITY) \
                  (update-edge it solidarity/strength (scale 0.95c))))"
            )),
            Ok(203)
        );
    }

    // -------------------------------------------------- PR 4 Task 13: EXECUTES

    /// Run one `(effects …)` list through the PRODUCTION path (Task 12):
    /// collect against an immutable borrow of `graph`, then — after that
    /// borrow ends — apply every collected write against a mutable one.
    /// The SAME two passes `tick.rs::run_tick` runs, on one shared graph
    /// object — the conformance-family mirror of `structural_verbs.rs`'s
    /// own `collect_then_apply` test helper, built from public API since
    /// this file is a separate integration-test crate.
    #[allow(clippy::type_complexity)]
    fn collect_then_apply(
        graph: &mut MemoryGraph,
        bindings: HashMap<String, Value>,
        effects_source: &str,
        fuel: &mut u64,
    ) -> Result<Vec<(String, Vec<(String, Value)>)>, EvalError> {
        let (form, _) =
            babylon_bsl::reader::read(effects_source).expect("effects source must parse");
        let babylon_bsl::reader::SExpr::List(items) = form else {
            unreachable!()
        };
        let types = type_env();
        let enum_registry = enums();
        let mut sink = CollectingSink::default();
        let pending = {
            // PR A verifier fix round (2026-08-12): `types`/`enum_registry`
            // above were already built for `EffectExecutor` below — the
            // sibling `EvalEnv` now carries them too, closing the same
            // coincidental-safety gap `structural_verbs.rs`'s own
            // `collect_then_apply` had (fixed in the same round).
            let env = EvalEnv {
                bindings,
                intrinsic_costs: &IntrinsicCosts::default(),
                graph: Some(&*graph as &dyn GraphSubstrate),
                types: Some(&types),
                enums: Some(&enum_registry),
                elements: Vec::new(),
                draw_context: None,
            };
            let mut collector = EffectExecutor::new(&types, &enum_registry, None);
            collector.collect_effects(&items[1..], &env, &EmptyIntrinsicHost, &mut sink, fuel)?
        };
        let mut applier = EffectExecutor::new(&types, &enum_registry, None);
        for write in &pending {
            applier.apply_pending_write(write, &mut *graph)?;
        }
        Ok(sink.events)
    }

    /// `for-each` over `nodes` applies `update-node` and `emit` once per
    /// element, in §2.6 ascending-id order — the §2.8 worked shape,
    /// EXECUTED.
    #[test]
    fn for_each_over_nodes_applies_update_node_and_emit_per_element_in_order() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(a, "social-class/agitation", 0.10)
            .unwrap();
        graph
            .update_node(b, "social-class/agitation", 0.20)
            .unwrap();
        let mut fuel = 4_096;
        let events = collect_then_apply(
            &mut graph,
            HashMap::new(),
            "(effects (for-each (nodes NodeType/SOCIAL_CLASS) \
               (update-node it social-class/agitation (set 0.5i)) \
               (emit EventType/RUPTURE (who it))))",
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            events,
            vec![
                (
                    "RUPTURE".to_owned(),
                    vec![("who".to_owned(), Value::NodeRef(a))]
                ),
                (
                    "RUPTURE".to_owned(),
                    vec![("who".to_owned(), Value::NodeRef(b))]
                ),
            ],
            "once per element, in §2.6 ascending-id iteration order"
        );
        for id in [a, b] {
            let stored = graph.node_attribute(id, "social-class/agitation").unwrap();
            assert!((stored - 0.5).abs() < 1e-12, "{id:?}");
        }
    }

    /// `for-each` over `neighbors` composes the same way — D24's type
    /// filter narrows the population `for-each` iterates, EXECUTED.
    #[test]
    fn for_each_over_neighbors_applies_update_node_per_element() {
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let tenant = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(tenant, "social-class/agitation", 0.10)
            .unwrap();
        graph.add_edge("SOLIDARITY", subject, tenant, 1.0).unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 1.0).unwrap();
        let mut fuel = 4_096;
        collect_then_apply(
            &mut graph,
            HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            "(effects (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) \
               (update-node it social-class/agitation (set 0.9i))))",
            &mut fuel,
        )
        .unwrap();
        let stored = graph
            .node_attribute(tenant, "social-class/agitation")
            .unwrap();
        assert!(
            (stored - 0.9).abs() < 1e-12,
            "the typed neighbor was written"
        );
    }

    /// §2.8 chapter C6: an iteration is a COMMAND, and "do it to none" is
    /// fully determined — the one place an empty set is quiet (unlike
    /// mean/min/max/select-*, which must PRODUCE a value and have none to
    /// produce, `E-EVAL-021`). EXECUTED, not merely asserted deferred.
    #[test]
    fn for_each_over_an_empty_query_applies_nothing_and_does_not_error() {
        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(self_id, "social-class/agitation", 0.10)
            .unwrap();
        let mut fuel = 512;
        let events = collect_then_apply(
            &mut graph,
            HashMap::from([("self".to_owned(), Value::NodeRef(self_id))]),
            "(effects (for-each (nodes NodeType/ORGANIZATION) \
               (update-node self social-class/agitation (set 0.99i))))",
            &mut fuel,
        )
        .expect("an empty for-each is not an error");
        assert!(events.is_empty());
        let stored = graph
            .node_attribute(self_id, "social-class/agitation")
            .unwrap();
        assert!(
            (stored - 0.10).abs() < 1e-12,
            "an empty for-each applies NOTHING — the body never ran"
        );
    }

    /// The §6.2 family-15 pre-state vector. §2.8 chapter C6, quoted in the
    /// module: "every expression anywhere in an effects list … is
    /// evaluated against the pre-state". An EARLIER `update-node` in the
    /// SAME effects list has not landed when the `for-each`'s query and
    /// body evaluate — both read through `env.graph`, the collect-time
    /// reborrow, and nothing applies until `collect_effects` returns.
    #[test]
    fn for_each_reads_the_pre_state_not_an_earlier_verbs_effect_in_the_same_list() {
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(subject, "social-class/agitation", 0.10)
            .unwrap();
        let mut fuel = 1_024;
        let events = collect_then_apply(
            &mut graph,
            HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            "(effects \
               (update-node self social-class/agitation (set 0.90i)) \
               (for-each (nodes NodeType/SOCIAL_CLASS) \
                 (emit EventType/RUPTURE (agitation (field-of it social-class/agitation)))))",
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            events,
            vec![(
                "RUPTURE".to_owned(),
                vec![("agitation".to_owned(), Value::Real(0.10))]
            )],
            "for-each's query and body must read the PRE-state (0.10), never \
             the earlier update-node's collected-but-not-yet-applied write \
             (0.90) — §2.8 chapter C6"
        );
        // …and the earlier update-node's own effect DID land, once
        // collect_effects returned and apply_pending_write ran.
        let stored = graph
            .node_attribute(subject, "social-class/agitation")
            .unwrap();
        assert!((stored - 0.90).abs() < 1e-12);
    }

    /// Nested `for-each` composes outer-iteration-then-inner-source-order,
    /// EXECUTED — the real-evaluation twin of
    /// `nested_for_each_multiplies_its_two_ceilings`'s static bound above.
    #[test]
    fn nested_for_each_composes_and_executes() {
        let mut graph = MemoryGraph::new();
        let sc1 = graph.add_node("SOCIAL_CLASS").unwrap();
        let sc2 = graph.add_node("SOCIAL_CLASS").unwrap();
        let org1 = graph.add_node("ORGANIZATION").unwrap();
        let org2 = graph.add_node("ORGANIZATION").unwrap();
        let mut fuel = 16_384;
        let events = collect_then_apply(
            &mut graph,
            HashMap::new(),
            "(effects \
               (for-each (nodes NodeType/SOCIAL_CLASS) :as outer \
                 (for-each (nodes NodeType/ORGANIZATION) \
                   (emit EventType/PAIR (outer outer) (inner it)))))",
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            events,
            vec![
                (
                    "PAIR".to_owned(),
                    vec![
                        ("outer".to_owned(), Value::NodeRef(sc1)),
                        ("inner".to_owned(), Value::NodeRef(org1))
                    ]
                ),
                (
                    "PAIR".to_owned(),
                    vec![
                        ("outer".to_owned(), Value::NodeRef(sc1)),
                        ("inner".to_owned(), Value::NodeRef(org2))
                    ]
                ),
                (
                    "PAIR".to_owned(),
                    vec![
                        ("outer".to_owned(), Value::NodeRef(sc2)),
                        ("inner".to_owned(), Value::NodeRef(org1))
                    ]
                ),
                (
                    "PAIR".to_owned(),
                    vec![
                        ("outer".to_owned(), Value::NodeRef(sc2)),
                        ("inner".to_owned(), Value::NodeRef(org2))
                    ]
                ),
            ],
            "outer = iteration order, inner = source order — composed, not \
             an unordered reduction anywhere (§2.8 chapter C6)"
        );
    }
}

// ====================================================== family 17 — C8
// Typed neighbours and element naming (§2.6).
//
// **The first `neighbors` vectors this document has ever required** — the
// reference's own words. D51 makes the result `NodeType` a mandatory fourth
// operand, a breaking change to a form no vector and no content rule
// exercised, and this crate's bound checker is the one place that carried
// the pre-change spelling.
//
// **PR 4, Task 13 (2026-08-11): EXECUTES.** The multiplicity vector (D72's
// set semantics: two qualifying edges reaching one node count once, not
// twice) and the filtering vector (D24: the annotated `NodeType` excludes
// a wrong-typed neighbour) both run for real below, over `MemoryGraph`.
// The three-operand `E-PARSE-042` arity check, the swapped-operand
// `E-TYPE-011` check, and the lesser-of-two-ceilings static bound were
// already real (all three are load-time static checks, unaffected by the
// query evaluator) and stay as they were. The `:as`/`it` naming rows stay
// load-time static too, EXCEPT the two-hop nested-query shape, which now
// also runs for real (over `nodes`/`neighbors`, the two heads slice 1
// serves — the spec's own worked example uses `hyperedges`/`members-of`,
// slice 3, so its static accept-and-bound pin stays alongside).
mod c8_typed_neighbours_and_naming {
    use super::{cost, e};
    use babylon_bsl::evaluator::{evaluate, EvalEnv, Value};
    use babylon_bsl::fuel::IntrinsicCosts;
    use babylon_bsl::grammar::{check_arities_and_closed_sets, check_enum_ref_kinds};
    use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
    use babylon_bsl::scope::check_element_names;
    use babylon_bsl::typecheck::TypeEnv;
    use babylon_bsl::types::EnumRegistry;
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::substrate::GraphSubstrate;
    use std::collections::HashMap;

    /// D51: the operand is **mandatory**, so the pre-C8 three-operand form
    /// is an arity error rather than a silently edge-type-only bound.
    #[test]
    fn a_three_operand_neighbors_is_e_parse_042() {
        let err = check_arities_and_closed_sets(&e("(neighbors self EdgeType/SOLIDARITY :out)"))
            .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-042");
        assert!(check_arities_and_closed_sets(&e(
            "(neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)"
        ))
        .is_ok());
    }

    /// D74: swapping the two operands is `E-TYPE-011` at both positions.
    #[test]
    fn the_two_operands_swapped_is_e_type_011() {
        let err = check_enum_ref_kinds(&e(
            "(neighbors self NodeType/SOCIAL_CLASS :out EdgeType/SOLIDARITY)",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-011");
    }

    /// D52: `ceiling(neighbors)` is the **lesser** of the two ceilings —
    /// neither bound can be exceeded, so the smaller is the honest one, and
    /// the mandatory operand is what makes the second number available.
    #[test]
    fn the_static_bound_is_the_lesser_of_the_two_ceilings() {
        // EdgeType/SOLIDARITY 40 < NodeType/SOCIAL_CLASS 100 → 40.
        // 2 + query(1 + self) + 40 × 1 = 44.
        assert_eq!(
            cost("(fold sum (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) it)"),
            Ok(44)
        );
        // NodeType/ORGANIZATION 40 vs EdgeType/IN_SCALE 5000 → 40, so the
        // annotation TIGHTENS what the pre-C8 reading would have bounded.
        assert_eq!(
            cost("(fold sum (neighbors self EdgeType/IN_SCALE :in NodeType/ORGANIZATION) it)"),
            Ok(44)
        );
    }

    /// §2.5's foreign-field rule now applies through `neighbors` exactly as
    /// it does through `nodes`: the annotated type legalises its own fields
    /// inside the body. Six systems need exactly this read.
    #[test]
    fn a_fold_body_over_neighbors_legalises_the_annotated_types_fields() {
        use babylon_bsl::bindings::parse_bindings;
        use babylon_bsl::scope::check_foreign_field_scoping;
        let form = e(&super::rule(
            "(bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold mean (neighbors self EdgeType/SOLIDARITY :in \
                                   NodeType/ORGANIZATION) claim) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        let decls = parse_bindings(&form).unwrap();
        assert_eq!(
            check_foreign_field_scoping(&form, &decls, Some("social-class"), &super::vocabulary()),
            Ok(())
        );
    }

    /// D54: `:as` names the element, and the name shares the rule's binding
    /// namespace — `:as it`/`:as self` is `E-PARSE-022` and a collision with
    /// a binding is `E-PARSE-030`.
    #[test]
    fn as_it_and_as_self_are_e_parse_022_and_a_collision_is_e_parse_030() {
        for reserved in ["it", "self"] {
            let form = e(&super::rule(&format!(
                "(bindings) (when (exists (nodes NodeType/ORGANIZATION) :as {reserved} #t)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )));
            let err = check_element_names(&form, &[]).unwrap_err();
            assert_eq!(err.spec_code(), "E-PARSE-022", "{reserved}");
        }
        let form = e(&super::rule(
            "(bindings (binding sector :field social-class/wealth)) \
             (when (exists (hyperedges HyperedgeType/ECONOMIC_SECTOR) :as sector #t)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        let err = check_element_names(&form, &["sector".to_owned()]).unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-030");
    }

    /// D53 + D54: `it` denotes the **innermost** element and `:as` reaches
    /// the outer one — §2.6's own two-hop example, which accepts, and its
    /// static bound, `2 + 1 + 500 × (2 + 32 × 2) = 33503`.
    #[test]
    fn the_two_hop_nested_fold_accepts_and_bounds() {
        let source = "(fold sum (hyperedges HyperedgeType/ECONOMIC_SECTOR) :as sector \
             (fold sum (members-of sector HyperedgeType/ECONOMIC_SECTOR) \
                   (field-of it social-class/wealth)))";
        assert_eq!(
            check_element_names(
                &e(&super::rule(&format!(
                    "(bindings) (when (< {source} 5)) \
                     (effects (update-node self social-class/agitation (add 0.05i)))"
                ))),
                &[]
            ),
            Ok(())
        );
        // inner: 2 + query(1 + sector ref 1) + 32 × field-of(2) = 68;
        // outer: 2 + query(1) + 500 × 68 = 34003.
        assert_eq!(cost(source), Ok(34_003));
    }

    /// D54: a `:as` name referenced outside its body is `E-TYPE-012` — and
    /// `cost(:as name) = 0` while a *reference* to it costs 1, which the
    /// C5 vector already pinned.
    #[test]
    fn it_outside_every_body_is_e_type_012() {
        let form = e(&super::rule(
            "(bindings) (when (< it 5)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        let err = check_element_names(&form, &[]).unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-012");
    }

    // -------------------------------------------------- PR 4 Task 13: EXECUTES

    fn eval_expr(
        source: &str,
        graph: &dyn GraphSubstrate,
        bindings: HashMap<String, Value>,
        fuel: &mut u64,
    ) -> Value {
        let costs = IntrinsicCosts::default();
        // PR A verifier fix round (2026-08-12): an empty pair, threaded
        // `Some` — no vector in this family declares an enum-typed field,
        // so every qname stays "unregistered" and renders `Value::Real`
        // exactly as before, but `field_of_node` now refuses loudly on
        // `None` (mirroring `require_graph`) rather than silently
        // degrading, so `None` here would turn this family's `field-of`
        // vectors into driver-error tests instead of field-read tests.
        let types = TypeEnv {
            fields: HashMap::new(),
            exemptions: &[],
        };
        let enums = EnumRegistry::default();
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(graph),
            types: Some(&types),
            enums: Some(&enums),
            elements: Vec::new(),
            draw_context: None,
        };
        evaluate(&e(source), &env, &EmptyIntrinsicHost, fuel).expect("vector must evaluate")
    }

    /// The §6.2 family-17 multiplicity vector (D72), EXECUTED: two
    /// qualifying edges (one `:out`, one `:in`) reaching one node under
    /// `:any` yield it ONCE — `neighbors` is a set, not a multiset.
    #[test]
    fn neighbors_multiplicity_vector_counts_each_qualifying_node_once() {
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 1.0).unwrap();
        graph.add_edge("SOLIDARITY", other, subject, 1.0).unwrap();
        let mut fuel = 1_000;
        let result = eval_expr(
            "(fold count (neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS) it)",
            &graph,
            HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            &mut fuel,
        );
        assert_eq!(result, Value::Int(1), "one node, once, not twice (D72)");
    }

    /// The §6.2 family-17 filtering vector (D24), EXECUTED: a node reached
    /// via the named edge type that is NOT of the annotated `NodeType` is
    /// simply not in the result — the annotation filters, it does not
    /// assert.
    #[test]
    fn neighbors_filtering_vector_excludes_the_wrong_node_type() {
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let tenant = graph.add_node("SOCIAL_CLASS").unwrap();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.add_edge("SOLIDARITY", tenant, subject, 1.0).unwrap();
        graph.add_edge("SOLIDARITY", org, subject, 1.0).unwrap();
        let mut fuel = 1_000;
        let result = eval_expr(
            "(fold count (neighbors self EdgeType/SOLIDARITY :in NodeType/SOCIAL_CLASS) it)",
            &graph,
            HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            &mut fuel,
        );
        assert_eq!(
            result,
            Value::Int(1),
            "the ORGANIZATION neighbour is excluded by the annotation"
        );
    }

    /// D53 + D54, EXECUTED: `it` denotes the INNERMOST element and `:as`
    /// reaches the outer one, over `nodes`/`neighbors` — the two heads
    /// slice 1 serves (the spec's own `hyperedges`/`members-of` worked
    /// example stays the static accept-and-bound pin above, since slice 1
    /// does not serve those heads).
    #[test]
    fn it_resolves_to_the_inner_element_and_the_as_name_to_the_outer() {
        let mut graph = MemoryGraph::new();
        let outer_a = graph.add_node("SOCIAL_CLASS").unwrap();
        let inner_a = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(inner_a, "organization/claim-strength", 10.0)
            .unwrap();
        graph.add_edge("SOLIDARITY", outer_a, inner_a, 1.0).unwrap();

        let outer_b = graph.add_node("SOCIAL_CLASS").unwrap();
        let inner_b = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(inner_b, "organization/claim-strength", 20.0)
            .unwrap();
        graph.add_edge("SOLIDARITY", outer_b, inner_b, 1.0).unwrap();

        let mut fuel = 10_000;
        // The outer fold names its element `:as outer`; the inner fold's
        // query reads `outer` (the OUTER element, via the neighbors source
        // operand) while its body reads `it` (the INNER element) — both
        // live on the element stack at once, and each name resolves to the
        // right one.
        let result = eval_expr(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) :as outer \
               (fold sum (neighbors outer EdgeType/SOLIDARITY :out NodeType/ORGANIZATION) \
                     (field-of it organization/claim-strength)))",
            &graph,
            HashMap::new(),
            &mut fuel,
        );
        assert_eq!(
            result,
            Value::Real(30.0),
            "10 (outer_a's neighbor) + 20 (outer_b's neighbor) = 30"
        );
    }
}

// ====================================================== family 18 — C9
// Metric registration (§2.11).
//
// Runtime rows deferred: `E-EVAL-036` (a `metric-of` against a referent of
// another type) and `E-EVAL-037` (a value the provider did not produce)
// need the provider seam and the query evaluator; their codes are pinned.
// The **stability vector** — two rules at one anchor reading one metric,
// whose values must be equal — is a determinism obligation on the
// *provider* (§2.11's list), enforced by review and by the determinism
// contract's golden vectors, and is recorded there rather than faked here.
mod c9_metric_registration {
    use super::{cost, e};
    use babylon_bsl::bindings::parse_bindings;
    use babylon_bsl::canonical_ast::canonical_bytes;
    use babylon_bsl::evaluator::EvalCode;
    use babylon_bsl::metrics::{MetricDomain, MetricRegistry};
    use babylon_bsl::typecheck::{typecheck_aggregation, TypeCode, TypeEnv};
    use std::collections::HashMap;

    const GRAPH: &str = "(metric solidarity-density :type coefficient :kind intensive \
        (domain :graph) :provider topology-scores)";
    const ELEMENT: &str = "(metric betweenness-centrality :type coefficient :kind intensive \
        (domain NodeType/ORGANIZATION) :provider topology-scores)";

    fn registry() -> MetricRegistry {
        let mut r = MetricRegistry::default();
        r.declare(&e(GRAPH)).expect("graph metric");
        r.declare(&e(ELEMENT)).expect("element metric");
        r
    }

    fn reading(body: &str) -> Option<&'static str> {
        let form = e(&super::rule(body));
        let decls = parse_bindings(&form).expect("bindings must parse");
        registry()
            .check_reading_forms(&form, &decls)
            .err()
            .and_then(|err| err.spec_code())
    }

    /// §2.11's two domains, and D56's ruling that the element-indexed one
    /// is read by the **accessor** rather than by a `:metric-of` bind-src.
    #[test]
    fn a_graph_metric_binds_and_an_element_metric_is_accessed() {
        let r = registry();
        assert_eq!(
            r.get("solidarity-density").unwrap().domain,
            MetricDomain::Graph
        );
        assert_eq!(
            r.get("betweenness-centrality").unwrap().domain,
            MetricDomain::Element("organization".to_owned())
        );
        assert_eq!(
            reading(
                "(bindings (binding d :metric solidarity-density) \
                           (binding c :expr (metric-of self betweenness-centrality))) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            ),
            None
        );
    }

    /// Reading either through the other's form is `E-LOAD-012` — both
    /// static, because the declaration and the reading form are both
    /// content.
    #[test]
    fn each_read_through_the_wrong_form_is_e_load_012() {
        assert_eq!(
            reading(
                "(bindings (binding c :metric betweenness-centrality)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            ),
            Some("E-LOAD-012")
        );
        assert_eq!(
            reading(
                "(bindings (binding d :expr (metric-of self solidarity-density))) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            ),
            Some("E-LOAD-012")
        );
    }

    /// §6.3's correction, re-proved for **both** reading forms: an
    /// unregistered name is `E-LOAD-011`, never `0.0`.
    #[test]
    fn an_unregistered_name_is_e_load_011_through_both_forms() {
        assert_eq!(
            reading(
                "(bindings (binding x :metric nowhere)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            ),
            Some("E-LOAD-011")
        );
        assert_eq!(
            reading(
                "(bindings (binding x :expr (metric-of self nowhere))) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            ),
            Some("E-LOAD-011")
        );
    }

    /// D55: a declaration disagreeing with the kernel's registration.
    #[test]
    fn kernel_disagreement_is_e_load_025() {
        let r = registry();
        let mut registered = r.get("solidarity-density").unwrap().clone();
        registered.provider = "somewhere-else".to_owned();
        let kernel = HashMap::from([("solidarity-density".to_owned(), registered)]);
        assert_eq!(
            r.check_against_kernel(&kernel).unwrap_err().spec_code(),
            Some("E-LOAD-025")
        );
    }

    /// D55 supersedes D12's metric clause: the **declared** kind
    /// propagates, so an intensive metric feeding an unweighted `mean` is
    /// `E-TYPE-042` exactly as an intensive field is.
    #[test]
    fn the_declared_kind_propagates_into_the_aggregation_law() {
        let mut fields = HashMap::new();
        registry().merge_into_kind_env(&mut fields);
        let env = TypeEnv {
            fields,
            exemptions: &[],
        };
        assert_eq!(
            typecheck_aggregation(&e("(mean solidarity-density)"), &env)
                .unwrap_err()
                .code,
            Some(TypeCode::UnweightedMeanOfIntensive)
        );
    }

    /// D57: the READ costs `1 + cost(operand)` like any other accessor.
    /// The provider's computation is not metered against the reading rule —
    /// a rule cannot bound a betweenness computation, and pretending
    /// otherwise would put a number in `:fuel` that means nothing.
    #[test]
    fn a_metric_read_costs_one_plus_its_operand_and_never_the_provider() {
        assert_eq!(cost("(metric-of self betweenness-centrality)"), Ok(2));
        assert_eq!(
            cost("(metric-of (the NodeType/POLITY) betweenness-centrality)"),
            Ok(2)
        );
    }

    /// §5.5: a `metric` form hashes into its own digest like
    /// `deffield`/`intrinsic`/`manifest`, under both `<domain>` shapes, and
    /// the two shapes are distinct bytes.
    #[test]
    fn both_domain_shapes_have_distinct_canonical_bytes() {
        let graph = canonical_bytes(&e(GRAPH)).unwrap();
        let element = canonical_bytes(&e(ELEMENT)).unwrap();
        assert_ne!(graph, element);
        assert!(!graph.is_empty() && !element.is_empty());
    }

    /// §2.11's two runtime disciplines, pinned as codes: absence is never a
    /// zero, at either failure.
    #[test]
    fn the_two_runtime_metric_failures_have_their_own_codes() {
        assert_eq!(EvalCode::MetricDomainMismatch.spec_code(), "E-EVAL-036");
        assert_eq!(EvalCode::MetricValueAbsent.spec_code(), "E-EVAL-037");
    }
}

// ====================================================== family 19 — C10
// Deliberate absences (§3.8) — a family of *rejecting* vectors, so the
// absences are pinned as loudly as the presences, plus the accepting pairs
// that prove each re-modelling works.
mod c10_deliberate_absences {
    use super::{cost, e};
    use babylon_bsl::grammar::{check_arities_and_closed_sets, check_string_positions};

    /// §3.8 item 1 / D13: there is no `bound?` predicate.
    ///
    /// The rst's family-19 row spells it `(bound? x)` and expects
    /// `E-LOAD-021`. **`bound?` is not a §1.4 `symbol`** — `?` is outside
    /// the alphabet — so the reader refuses it at `E-LEX-003`, one class
    /// earlier and louder. Both refusals are pinned: the spelling the rst
    /// wrote, and the spellable form that actually reaches the
    /// undeclared-intrinsic gate.
    #[test]
    fn bound_is_not_a_predicate_and_neither_spelling_survives() {
        use babylon_bsl::reader::{read, LexCode, ReadErrorKind};
        assert_eq!(
            read("(bound? x)").unwrap_err().kind,
            ReadErrorKind::Lex(LexCode::UnclassifiableToken)
        );
        assert_eq!(
            cost("(bound x)").unwrap_err().spec_code(),
            Some("E-LOAD-021")
        );
    }

    /// §3.8 item 4 / D75: a string literal in an `emit` payload is
    /// `E-PARSE-010` — the string lexes, the position rejects it.
    #[test]
    fn a_string_literal_in_an_emit_payload_is_e_parse_010() {
        let err = check_string_positions(&e(&super::rule(
            "(bindings) (effects (emit EventType/RUPTURE (why \"a description\")))",
        )))
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-010");
        // …while `:material-basis`'s own string is untouched: it is a
        // rule-level option, not an expression.
        assert_eq!(
            check_string_positions(&e(&super::rule(
                "(bindings) (effects (emit EventType/RUPTURE (severity 0.9c)))"
            ))),
            Ok(())
        );
    }

    /// §3.8 item 1 / §2.8: the `<update-op>` set is closed, so the
    /// `(unset …)` the frozen estate reaches for is `E-PARSE-015`.
    #[test]
    fn an_unset_update_op_is_e_parse_015() {
        let err =
            check_arities_and_closed_sets(&e("(update-node self social-class/agitation (unset))"))
                .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-015");
    }

    /// The accepting half — each re-modelling, written out.
    ///
    /// D58: an optional axis takes a **companion presence field**, written
    /// under one `guard` so the pair moves together or not at all.
    /// `:optional`/`:default` is explicitly NOT the mechanism, because a
    /// default converts "never seeded" into "seeded with zero" and changes
    /// the eligibility population.
    #[test]
    fn the_presence_field_remodelling_is_expressible() {
        assert!(cost(
            "(guard (< 1 2) \
             (update-node self social-class/wealth (set 0$)) \
             (update-node self social-class/wealth-present (set #t)))"
        )
        .is_ok());
    }

    /// D59: a FIFO agenda becomes its own bounded `NodeType` carrying a
    /// `queued-at-tick` field, and "the next item" becomes `select-min`.
    #[test]
    fn the_fifo_agenda_remodelling_is_expressible_and_bounded() {
        // 2 + query(1) + 40 × field-of(2) = 83.
        assert_eq!(
            cost(
                "(select-min (nodes NodeType/ORGANIZATION) \
                 (field-of it organization/queued-at-tick))"
            ),
            Ok(83)
        );
    }

    /// D60: no same-tick event-history query — the emitting rule stamps a
    /// field and the consumer reads it as an ordinary `:field`, which makes
    /// the cross-system dependency visible in content, hashable and
    /// inspectable.
    #[test]
    fn the_producer_stamped_tick_field_remodelling_is_expressible() {
        assert!(cost("(update-node self social-class/crisis-tick (set 12))").is_ok());
        assert!(cost("(field-of self social-class/crisis-tick)").is_ok());
    }
}

// ====================================================== family 20 — C11
// Invariant substrate and the scale lattice (§3.9).
mod c11_invariant_substrate {
    use super::{cost, e};
    use babylon_bsl::manifest::{check_rule_against_manifest, Manifest};

    const MANIFEST: &str = "(manifest r9
       (ceiling NodeType/SOCIAL_CLASS :ceiling 100)
       (ceiling NodeType/TERRITORY :ceiling 3000 :invariant)
       (ceiling EdgeType/IN_SCALE :ceiling 5000 :invariant)
       (ceiling EdgeType/SOLIDARITY :ceiling 40)
       (ceiling HyperedgeType/COMMUNITY :ceiling 200 :max-members 64))";

    fn manifest() -> Manifest {
        Manifest::parse(&e(MANIFEST)).expect("well formed")
    }

    fn rule(body: &str) -> babylon_bsl::reader::SExpr {
        e(&super::rule(&format!("(bindings) (effects {body})")))
    }

    /// §3.9: aggregation up one rung is an ordinary one-hop fold, and
    /// distribution down one rung the `for-each` that mirrors it. This
    /// costs closed-vocabulary members and a hydration contract; it costs
    /// **no grammar** — no `group-by`, no keyed collection (D62).
    #[test]
    fn the_lattice_is_a_one_hop_fold_and_its_mirroring_for_each() {
        // ceiling = min(IN_SCALE 5000, TERRITORY 3000) = 3000.
        // 2 + query(1 + self) + 3000 × field-of(2) = 6004.
        assert_eq!(
            cost(
                "(fold sum (neighbors self EdgeType/IN_SCALE :in NodeType/TERRITORY) \
                 (field-of it territory/wage-bill))"
            ),
            Ok(6004)
        );
        // The distribution mirrors it: 2 + 2 + 3000 × update-node(5) = 15004.
        assert_eq!(
            cost(
                "(for-each (neighbors self EdgeType/IN_SCALE :in NodeType/TERRITORY) \
                 (update-node it territory/wage-bill (scale 0.5c)))"
            ),
            Ok(15_004)
        );
    }

    /// D63: an `add-*`/`remove-*` naming an `:invariant` type is
    /// `E-LOAD-013`, statically, off the verb's `<enum-ref>` operand.
    #[test]
    fn structural_verbs_on_invariant_types_are_e_load_013() {
        for body in [
            "(add-node NodeType/TERRITORY t1)",
            "(add-edge EdgeType/IN_SCALE a b :strength 0.5c)",
            "(remove-edge EdgeType/IN_SCALE a b)",
        ] {
            let err = check_rule_against_manifest(&rule(body), &manifest()).expect_err(body);
            assert_eq!(err.spec_code(), Some("E-LOAD-013"), "{body}");
        }
    }

    /// D63's other half, proved by acceptance: **field writes are
    /// unaffected**. A territory's stock changes every tick while the
    /// territory's existence and its rung in the lattice do not, and it is
    /// exactly that distinction the flag encodes.
    #[test]
    fn a_field_write_on_an_invariant_type_must_accept() {
        assert_eq!(
            check_rule_against_manifest(
                &rule("(update-node self territory/wage-bill (add 5$))"),
                &manifest()
            ),
            Ok(())
        );
    }

    /// §2.9: `:invariant` is illegal on a `HyperedgeType` row.
    #[test]
    fn invariant_on_a_hyperedge_row_is_e_load_042() {
        let err = Manifest::parse(&e(
            "(manifest m (ceiling HyperedgeType/COMMUNITY :ceiling 2 :max-members 3 :invariant))",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-042"));
    }

    /// D64: there is no `:reference` bind-src. A keyed reference series is
    /// materialised as a declared node field at hydration and read with an
    /// ordinary `:field` — which is what makes the hydration contract a
    /// **blocking dependency** rather than a source of zeros at tick 1
    /// (§3.5's plain-binding rule, `E-LOAD-010`).
    #[test]
    fn a_reference_series_is_read_with_an_ordinary_field_binding() {
        use babylon_bsl::bindings::{parse_bindings, resolve_bindings, BindingVocabulary};
        use std::collections::HashSet;
        let form = e(&super::rule(
            "(bindings (binding wage :field territory/wage-bill)) \
             (when (< wage 5$)) \
             (effects (update-node self territory/wage-bill (add 1$)))",
        ));
        let decls = parse_bindings(&form).unwrap();
        let seeded = BindingVocabulary {
            fields: HashSet::from(["territory/wage-bill".to_owned()]),
            consts: HashSet::new(),
            probability_consts: HashSet::new(),
            metrics: HashSet::new(),
        };
        assert_eq!(resolve_bindings(&decls, &seeded), Ok(()));
        // The same rule against a hydration that omits the series.
        let empty = BindingVocabulary::default();
        assert_eq!(
            resolve_bindings(&decls, &empty).unwrap_err().spec_code(),
            Some("E-LOAD-010")
        );
    }
}

// ====================================================== family 21 — C12
// Hyperedge fields and reference identity (§2.8, §2.4).
//
// **Declared substrate gap, as for C2's `update-edge`:** a hyperedge
// carries no attributes in `GraphSubstrate`, so the four `<update-op>`
// executions are not pinned as executions. The verb's grammar, cost,
// arity and codes are.
mod c12_hyperedge_fields_and_identity {
    use super::{cost, e, type_env};
    use babylon_bsl::bindings::parse_bindings;
    use babylon_bsl::evaluator::EvalCode;
    use babylon_bsl::grammar::check_arities_and_closed_sets;
    use babylon_bsl::typecheck::{check_reference_comparisons, TypeCode};

    fn comparison_error(body: &str) -> Option<TypeCode> {
        let form = e(&super::rule(body));
        let decls = parse_bindings(&form).expect("bindings must parse");
        check_reference_comparisons(&form, &type_env(), &decls)
            .err()
            .and_then(|err| err.code)
    }

    /// D65: `update-hyperedge` mirrors `update-node` and `update-edge`
    /// operand for operand, under each of the four `<update-op>` forms, and
    /// costs like the structural verb it is.
    #[test]
    fn update_hyperedge_takes_each_update_op_at_the_structural_verb_cost() {
        for op in ["add", "sub", "set", "scale"] {
            assert_eq!(
                cost(&format!(
                    "(update-hyperedge it economic-sector/output ({op} 5$))"
                )),
                Ok(5),
                "{op}"
            );
        }
        // A fifth head there is E-PARSE-015: the set is closed (§2.8).
        let err = check_arities_and_closed_sets(&e(
            "(update-hyperedge it economic-sector/output (unset))",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-015");
    }

    /// D26's member-list half **stands**: a roster change is still
    /// whole-object replacement, `remove-hyperedge` then `add-hyperedge` in
    /// one effect list, so the `:max-members` check stays at a single point.
    #[test]
    fn a_roster_change_is_still_whole_object_replacement() {
        assert!(cost(
            "(guard #t \
             (remove-hyperedge h) \
             (add-hyperedge HyperedgeType/COMMUNITY h2 (members a b c)))"
        )
        .is_ok());
    }

    /// D67: references compare by identity, with `=` and `!=` **only**.
    #[test]
    fn same_kind_identity_comparison_accepts_with_both_operators() {
        for op in ["=", "!="] {
            assert_eq!(
                comparison_error(&format!(
                    "(bindings) (when ({op} self (the NodeType/POLITY))) \
                     (effects (update-node self social-class/agitation (add 0.05i)))"
                )),
                None,
                "{op}"
            );
        }
    }

    /// …and an ordering operator, a cross-kind comparison, and a comparison
    /// against a non-reference are all `E-TYPE-017`. There is no ordering on
    /// references *in the language*: exposing §2.6's iteration order as a
    /// comparison would invite content to depend on id assignment.
    #[test]
    fn ordering_cross_kind_and_non_reference_comparisons_are_e_type_017() {
        for cond in [
            "(< self (the NodeType/POLITY))",
            "(= self (edge-between EdgeType/SOLIDARITY self self))",
            "(= self 5)",
        ] {
            assert_eq!(
                comparison_error(&format!(
                    "(bindings) (when {cond}) \
                     (effects (update-node self social-class/agitation (add 0.05i)))"
                )),
                Some(TypeCode::BadReferenceComparison),
                "{cond}"
            );
        }
    }

    /// §2.7's intersection idiom, now writable because C8's naming and
    /// C12's identity comparison both exist — and **priced visibly**, which
    /// is the whole point of deferring a set-algebra operator rather than
    /// hiding the cost. The arithmetic is spelled out in the body below,
    /// one line per §3.7 row, so a reviewer reads the quadratic rather than
    /// taking a number on trust.
    #[test]
    fn the_intersection_idiom_pays_its_quadratic_cost_in_the_open() {
        let inner_cmp = 1 + 1 + 1; // (= it ha)
        let inner_exists = 2 + (1 + 1) + 200 * inner_cmp;
        let if_cost = 1 + inner_exists; // both branches are literals (0)
        let expected = 2 + (1 + 1) + 200 * if_cost;
        assert_eq!(
            cost(
                "(fold count (hyperedges-of a HyperedgeType/COMMUNITY) :as ha \
                 (if (exists (hyperedges-of b HyperedgeType/COMMUNITY) (= it ha)) 1 0))"
            ),
            Ok(expected)
        );
        assert!(
            expected > 100_000,
            "the deferral pays quadratically, where a reviewer can see it"
        );
    }

    /// D65: a `<qname>` owning off another hyperedge type is `E-EVAL-033`,
    /// since a `HyperedgeRef` carries no static type.
    #[test]
    fn a_wrong_owner_on_update_hyperedge_is_e_eval_033() {
        assert_eq!(
            EvalCode::AccessorTypeOrValueMismatch.spec_code(),
            "E-EVAL-033"
        );
    }
}

// ====================================================== family 22 — C13
// The intrinsic cap (§3.10). The calendar-binding half of this family
// landed with C7; the RNG half is named below.
mod c13_intrinsic_cap {
    use babylon_bsl::declarations::{
        check_intrinsic_cap, check_intrinsic_name, DECLARABLE_INTRINSICS,
    };

    /// §3.10 gate 1, mechanical: the transcendental cap is `{exp, log}`,
    /// and R10 is operative for R9/R10 purposes. `floor` joins under ADR188
    /// Row 2 (D97), `rng-draw` under ADR188 Row 11 (D69, #576 Task 5). The
    /// ADR219 rider train (Director ruling 2026-08-22) grows the set by six:
    /// `sqrt` (ADR188 Row 6's fallback rider, taken), `round-half-even`
    /// (Row 3's ratified housekeeping rider, landed — D70's "recorded, not
    /// resolved" is resolved), and `min`/`max`/`abs`/`clamp` (Rows 4/5's
    /// "no rider" dispositions SUPERSEDED on #591 item 2's accumulated
    /// evidence, Director-approved 2026-08-22). All six are exact-arithmetic
    /// IEEE-754-specified crossings — the transcendental roster stays the
    /// R10 pair alone. `tanh`/`entropy`/`renormalize`/`trunc` stay OUT —
    /// this row remains the standing proof the set never silently grows.
    #[test]
    fn the_nine_intrinsics_are_declarable_after_rng_draw_retirement() {
        assert_eq!(
            DECLARABLE_INTRINSICS,
            [
                "exp",
                "log",
                "floor",
                "sqrt",
                "round-half-even",
                "min",
                "max",
                "abs",
                "clamp"
            ]
        );
        for name in DECLARABLE_INTRINSICS {
            assert_eq!(check_intrinsic_cap(name), Ok(()), "{name}");
        }
        for outside in ["tanh", "entropy", "renormalize", "trunc"] {
            assert!(check_intrinsic_cap(outside).is_err(), "{outside}");
        }
    }

    /// D71: `sigmoid` is prohibited **outright**, not merely undeclared —
    /// the one part of gate 2 that can be made mechanical, and made so.
    #[test]
    fn sigmoid_is_e_load_024_not_merely_outside_the_cap() {
        let err = check_intrinsic_name("sigmoid").unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-024"));
        assert_eq!(
            check_intrinsic_cap("sigmoid").unwrap_err().spec_code(),
            Some("E-LOAD-024"),
            "the cap check runs the prohibition FIRST, so the stronger \
             refusal is the one reported"
        );
    }

    /// A call to an intrinsic outside the declared table is `E-LOAD-021` —
    /// never a default cost, which is what keeps `bound(rule)` computable
    /// from content alone.
    #[test]
    fn a_call_to_an_undeclared_intrinsic_is_e_load_021() {
        assert_eq!(
            super::cost("(tanh 0.5c)").unwrap_err().spec_code(),
            Some("E-LOAD-021")
        );
    }
}

// ================================================== C14 — Amendment AJ
// The finite-kernel boundary retires author-visible rng-draw. The private
// kernel draw is tested through finite-kernel realization instead.
mod c14_rng_draw_retirement {
    use babylon_bsl::declarations::{check_intrinsic_cap, kernel_signature};
    use babylon_bsl::intrinsic_host::{IntrinsicCallCtx, IntrinsicHost, KernelIntrinsicHost};

    #[test]
    fn rng_draw_has_no_declaration_signature_or_dispatch_arm() {
        assert!(check_intrinsic_cap("rng-draw").is_err());
        assert_eq!(kernel_signature("rng-draw"), None);
        let error = KernelIntrinsicHost
            .call("rng-draw", &[], IntrinsicCallCtx::context_free())
            .expect_err("the author-visible RNG path is retired");
        assert!(error.message.contains("no intrinsic registered"));
    }
}

// ============================================ R9 verification repairs
//
// The vectors the adversarial review of this PR showed were missing. Every
// one of them drives the **composed** loader or asserts canonical bytes —
// the two things the original suite never did, and the reason four
// blocker-class defects survived a green run.
mod verification_repairs {
    use super::{ceilings, e, load_code, vocabulary};
    use babylon_bsl::bindings::parse_bindings;
    use babylon_bsl::canonical_ast::canonical_bytes;
    use babylon_bsl::fuel::IntrinsicCosts;
    use babylon_bsl::scope::check_foreign_field_scoping;

    const PRE: &str =
        ":role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 262144";

    fn rule(body: &str) -> String {
        format!("(rule demo/r9 {PRE} {body})")
    }

    // ---------------------------------------------------------- D54 names

    /// §2.6's own worked two-hop example must **load**. It could not: a
    /// reference to a `:as` name was rejected by `check_free_variables` as
    /// an undeclared variable (`E-LOAD-010`).
    #[test]
    fn the_spec_two_hop_worked_example_loads_through_the_composed_pipeline() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (fold sum (hyperedges HyperedgeType/ECONOMIC_SECTOR) :as sector \
                            (fold sum (members-of sector HyperedgeType/ECONOMIC_SECTOR) \
                                  (field-of it social-class/wealth))) 5$)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            None
        );
    }

    /// §2.7's intersection idiom, and the C5 `:as winner` selection — the
    /// other two spellings the defect killed.
    #[test]
    fn the_intersection_idiom_and_a_named_selection_both_load() {
        assert_eq!(
            load_code(&rule(
                "(domain NodeType/SOCIAL_CLASS) (bindings) \
                 (when (< (fold count (hyperedges HyperedgeType/COMMUNITY) :as ha \
                            (if (exists (hyperedges-of self HyperedgeType/COMMUNITY) (= it ha)) \
                                1 0)) 5)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            None
        );
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (effects (update-node (select-max (nodes NodeType/ORGANIZATION) :as winner \
                                        (field-of winner organization/claim-strength)) \
                           organization/claim-strength (set 0.5c)))"
            )),
            None
        );
    }

    /// D54: a `:as` name referenced **outside** its body is `E-TYPE-012` —
    /// the half `it_outside_every_body_is_e_type_012` never pinned, and the
    /// half that previously reported `E-LOAD-010`.
    #[test]
    fn a_as_name_referenced_outside_its_body_is_e_type_012() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (+ (fold sum (nodes NodeType/ORGANIZATION) :as a \
                               (field-of a organization/claim-strength)) \
                             (fold sum (nodes NodeType/TERRITORY) \
                               (field-of a organization/claim-strength))) 5)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-012")
        );
    }

    /// …and a genuinely undeclared variable still reports `E-LOAD-010`, so
    /// widening the name set did not blunt the free-variable gate.
    #[test]
    fn an_undeclared_variable_is_still_e_load_010() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) (when (< nowhere 5)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-LOAD-010")
        );
    }

    /// `:as` names are rule-scoped-unique (§2.6: they "share the rule's
    /// binding namespace"), so a sibling reuse collides.
    #[test]
    fn a_sibling_reuse_of_a_as_name_is_e_parse_030() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (+ (fold sum (nodes NodeType/ORGANIZATION) :as a \
                               (field-of a organization/claim-strength)) \
                             (fold sum (nodes NodeType/TERRITORY) :as a \
                               (field-of a territory/wage-bill))) 5)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-PARSE-030")
        );
    }

    /// A fold carrying `:as` must still face §3.4. Appending a
    /// never-referenced name was a silent bypass of the
    /// unweighted-mean-of-an-intensive variance error.
    #[test]
    fn a_named_fold_does_not_escape_the_aggregation_law() {
        let unnamed = rule(
            "(domain :graph) (bindings) \
             (when (< (fold mean (nodes NodeType/ORGANIZATION) \
                        (field-of it organization/claim-strength)) 0.5c)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        );
        let named = rule(
            "(domain :graph) (bindings) \
             (when (< (fold mean (nodes NodeType/ORGANIZATION) :as o \
                        (field-of o organization/claim-strength)) 0.5c)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        );
        assert_eq!(load_code(&unnamed), Some("E-TYPE-042"));
        assert_eq!(
            load_code(&named),
            Some("E-TYPE-042"),
            "naming the element must not bypass §3.4"
        );
    }

    // -------------------------------------------------- D67 / D46 classes

    /// D67: `it` and the `:as` names are the language's principal source of
    /// references (§3.1). With the element classes threaded, a same-kind
    /// identity comparison accepts…
    #[test]
    fn a_same_kind_comparison_against_an_element_loads() {
        assert_eq!(
            load_code(&rule(
                "(bindings (binding wealth :field social-class/wealth)) \
                 (when (exists (nodes NodeType/SOCIAL_CLASS) (not (= it self)))) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            None,
            "this legal content was WRONGLY rejected as E-TYPE-017 before"
        );
    }

    /// …an ordering comparison on an element is `E-TYPE-017`…
    #[test]
    fn an_ordering_comparison_on_an_element_is_e_type_017() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (exists (nodes NodeType/ORGANIZATION) :as a \
                        (exists (nodes NodeType/TERRITORY) (< it a)))) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-017")
        );
    }

    /// …a comparison of an element against a non-reference is too…
    #[test]
    fn an_element_compared_with_a_scalar_is_e_type_017() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (exists (nodes NodeType/ORGANIZATION) (= it 5))) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-017")
        );
    }

    /// …and a cross-kind element comparison, which previously passed
    /// unchecked because both sides classified as `Unknown`.
    #[test]
    fn a_cross_kind_element_comparison_is_e_type_017() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (exists (nodes NodeType/ORGANIZATION) :as n \
                        (exists (edges EdgeType/SOLIDARITY) (= it n)))) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-017")
        );
    }

    /// D46 through the same map: a selection scored by `it` — a reference —
    /// is `E-TYPE-016`, and the C5 vector that "passed" before did so only
    /// because `it` classified as `Unknown`.
    #[test]
    fn a_selection_scored_by_an_element_reference_is_e_type_016() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (effects (update-node (select-max (nodes NodeType/ORGANIZATION) it) \
                           organization/claim-strength (set 0.5c)))"
            )),
            Some("E-TYPE-016")
        );
    }

    // ------------------------------------------------- `:expr` body rules

    /// §2.5: a `:expr` is evaluated at rule scope, so `it` inside one is
    /// `E-TYPE-012` — family 16's required vector.
    #[test]
    fn it_inside_an_expr_binding_is_e_type_012() {
        assert_eq!(
            load_code(&rule(
                "(bindings (binding wealth :field social-class/wealth) \
                           (binding bad :expr (+ it 1))) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            Some("E-TYPE-012")
        );
    }

    /// …while a fold *nested* inside a `:expr` binds `it` normally, which
    /// §2.5 explicitly permits — the fix is a scope extension, not a ban.
    #[test]
    fn a_fold_nested_inside_an_expr_binding_still_binds_it() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) \
                 (bindings (binding total :expr (fold sum (nodes NodeType/SOCIAL_CLASS) \
                                                  (field-of it social-class/wealth)))) \
                 (when (< total 5$)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            None
        );
    }

    /// §1.6/D75: a string literal in a `:expr` operand is an expression
    /// position, so `E-PARSE-010`.
    #[test]
    fn a_string_literal_inside_an_expr_binding_is_e_parse_010() {
        assert_eq!(
            load_code(&rule(
                "(bindings (binding wealth :field social-class/wealth) \
                           (binding bad :expr (= wealth \"prose\"))) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            Some("E-PARSE-010")
        );
    }

    /// Family 16's kind-propagation row: a `:expr` binding whose kind is
    /// intensive, feeding an unweighted `mean`, is `E-TYPE-042` — proving
    /// kind propagates *through the binding* and not only through a
    /// `:field` one.
    #[test]
    fn an_intensive_expr_binding_feeding_an_unweighted_mean_is_e_type_042() {
        // `social-class/agitation` is declared intensive; the `:expr`
        // binding names it, and the unweighted `mean` over it must still be
        // the recorded variance error. Before this repair the `:expr` name
        // fell through to `resolve_field` and was rejected UNCODED as an
        // unknown field.
        assert_eq!(
            load_code(&rule(
                "(bindings (binding agitation :field social-class/agitation) \
                           (binding derived  :expr agitation)) \
                 (when (< (fold mean (nodes NodeType/SOCIAL_CLASS) derived) 0.5c)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            Some("E-TYPE-042")
        );
    }

    // ------------------------------------------ query-operand scoping

    /// §2.6: `neighbors` yields "nodes reachable from the operand", so its
    /// element operand is the **source**, evaluated in the outer scope —
    /// and `neighbors` has no predicate at all. Reading a foreign field
    /// there is `E-TYPE-010`.
    ///
    /// The vector this replaces used `hyperedges-of`, whose
    /// `query_node_type_segment` is always `None`, so it passed under
    /// either reading and pinned nothing.
    #[test]
    fn a_neighbors_operand_is_outside_the_body_it_introduces() {
        let form = e(&rule(
            "(bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold count (neighbors claim EdgeType/SOLIDARITY :out \
                                    NodeType/ORGANIZATION) 1) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        let decls = parse_bindings(&form).unwrap();
        let err = check_foreign_field_scoping(&form, &decls, Some("social-class"), &vocabulary())
            .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-010");
    }

    /// …and the dual: a legal read under exactly one enclosing body of that
    /// type must NOT be reported ambiguous just because a `neighbors` sits
    /// between. The old walk double-counted and fired `E-TYPE-013`.
    #[test]
    fn one_enclosing_body_plus_a_neighbors_operand_is_not_ambiguous() {
        // The `neighbors` result type must BE the owner type, or
        // `matches_owner` is false and the vector passes against the
        // pre-fix walk too — the vacuity the re-review caught.
        let form = e(&rule(
            "(bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) \
                        (fold count (neighbors claim EdgeType/SOLIDARITY :out \
                                      NodeType/ORGANIZATION) 1)) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ));
        let decls = parse_bindings(&form).unwrap();
        assert_eq!(
            check_foreign_field_scoping(&form, &decls, Some("social-class"), &vocabulary()),
            Ok(())
        );
    }

    // ------------------------------------------------ family 6 — `:cas`

    /// §6.2's closing obligation: **each new form tag owes a `:cas`
    /// vector**. Each R9 tag is pinned to a header assembled from §5.1–§5.2
    /// — the tag byte, its length-prefixed name and its child count — and
    /// the encoding is then decoded back to prove §5.2's self-delimitation
    /// claim for that tag rather than merely asserting the bytes are
    /// non-empty.
    #[test]
    fn every_r9_form_tag_has_pinned_canonical_bytes() {
        let cases: [(&str, usize, &str); 9] = [
            (
                "neighbors",
                4,
                "(neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)",
            ),
            ("field-of", 2, "(field-of it solidarity/strength)"),
            (
                "edge-between",
                3,
                "(edge-between EdgeType/SOLIDARITY self other)",
            ),
            ("the", 1, "(the NodeType/POLITY)"),
            ("metric-of", 2, "(metric-of self solidarity-density)"),
            (
                "select-max",
                2,
                "(select-max (nodes NodeType/ORGANIZATION) it)",
            ),
            (
                "select-min",
                2,
                "(select-min (nodes NodeType/ORGANIZATION) it)",
            ),
            (
                "for-each",
                2,
                "(for-each (edges EdgeType/SOLIDARITY) (remove-node it))",
            ),
            (
                "update-edge",
                3,
                "(update-edge it solidarity/strength (scale 0.95c))",
            ),
        ];
        for (tag, children, source) in cases {
            let bytes = canonical_bytes(&e(source)).expect(source);
            let mut header = vec![0x02, u8::try_from(tag.len()).unwrap()];
            header.extend_from_slice(tag.as_bytes());
            header.extend_from_slice(&u32::try_from(children).unwrap().to_be_bytes());
            assert!(
                bytes.starts_with(&header),
                "{source}: expected tag {tag} with {children} children"
            );
            // §5.2: "self-delimiting, so the encoding is unambiguously
            // parseable back to the AST". A minimal decoder must consume
            // exactly the bytes the encoder produced — no more, no fewer.
            assert_eq!(
                consumed_by_one_node(&bytes),
                bytes.len(),
                "{source}: the encoding is not self-delimiting"
            );
        }
    }

    /// How many bytes one CAS node occupies, per §5.1-§5.2's two shapes.
    /// Independent of the encoder: it reads only the length prefixes.
    fn consumed_by_one_node(bytes: &[u8]) -> usize {
        let name_len = usize::from(bytes[1]);
        let mut pos = 2 + name_len;
        let count =
            u32::from_be_bytes(bytes[pos..pos + 4].try_into().expect("u32 length")) as usize;
        pos += 4;
        match bytes[0] {
            0x01 => pos + count,
            0x02 => {
                for _ in 0..count {
                    pos += consumed_by_one_node(&bytes[pos..]);
                }
                pos
            }
            other => panic!("unknown node type byte {other:#04x}"),
        }
    }

    /// §5.6's pinned bytes and both digests are **asserted**, not assumed.
    #[test]
    fn the_worked_examples_500_bytes_and_digests_include_causal_metadata() {
        use babylon_bsl::canonical_ast::rules_hash_of;
        use sha2::{Digest, Sha256};
        let worked = e("(rule demo/hunger \
             :role mechanic :evidence derived :material-basis \"subsistence deficit at the point of reproduction\" \
             :fuel 64 \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 1000.5$)) \
             (effects (update-node self social-class/agitation (add 0.05i))))");
        let bytes = canonical_bytes(&worked).unwrap();
        assert_eq!(bytes.len(), 500);
        let hex = |b: &[u8]| {
            use std::fmt::Write as _;
            b.iter().fold(String::new(), |mut acc, byte| {
                let _ = write!(acc, "{byte:02x}");
                acc
            })
        };
        assert_eq!(
            hex(&Sha256::digest(&bytes)),
            "9b49c1e87a63ccb64dee96bd179c8ffecd3fe476c81a4f81771daac237156396"
        );
        assert_eq!(
            hex(&rules_hash_of(&[worked]).unwrap()),
            "bf2f15517d1b4d10e44af36983c76249b49d8fa2dc71e6d8c1d2607b37b2f20f"
        );
    }

    /// The fuel bound is unchanged by any of this: `cost(:as name) = 0` and
    /// the accessors stay keyed lookups.
    #[test]
    fn the_two_hop_bound_is_unchanged() {
        use babylon_bsl::bound_checker::expr_cost;
        assert_eq!(
            expr_cost(
                &e(
                    "(fold sum (hyperedges HyperedgeType/ECONOMIC_SECTOR) :as sector \
                    (fold sum (members-of sector HyperedgeType/ECONOMIC_SECTOR) \
                          (field-of it social-class/wealth)))"
                ),
                &ceilings(),
                &IntrinsicCosts::default()
            ),
            Ok(34_003)
        );
    }
}

// ==================================== R9 re-verification repairs (round 2)
//
// The second adversarial pass confirmed both blocker classes repaired and
// found a REGRESSION the first repair introduced, three same-class gaps it
// had not reached, and D90 arriving from PR #481. Every vector here drives
// `load_rule`.
mod reverification_repairs {
    use super::{e, load_code, type_env, vocabulary};
    use babylon_bsl::bindings::parse_bindings;
    use babylon_bsl::domain::{resolve_domain, RuleDomain};
    use babylon_bsl::typecheck::{typecheck_aggregation, TypeCode, TypeEnv};

    const PRE: &str =
        ":role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 262144";

    fn rule(body: &str) -> String {
        format!("(rule demo/rv {PRE} {body})")
    }

    // ------------------------------------------------ N1: both directions

    /// **The regression.** `scope.seen` was populated in walk order, so a
    /// reference appearing structurally BEFORE its `:as` declaration was
    /// never in it — no `E-TYPE-012` — while `declared_element_names` is
    /// order-independent, so `check_free_variables` admitted it too. The
    /// rule loaded silently where the pre-repair code at least rejected it
    /// (with the wrong code): a III.11 inversion.
    #[test]
    fn a_forward_out_of_body_as_reference_is_e_type_012() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (+ (fold sum (nodes NodeType/TERRITORY) \
                               (field-of a territory/wage-bill)) \
                             (fold sum (nodes NodeType/ORGANIZATION) :as a \
                               (field-of a territory/wage-bill))) 5$)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-012")
        );
    }

    /// The backward direction, which the first repair did fix — pinned
    /// beside its twin so neither can regress alone.
    #[test]
    fn a_backward_out_of_body_as_reference_is_e_type_012() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (+ (fold sum (nodes NodeType/ORGANIZATION) :as a \
                               (field-of a territory/wage-bill)) \
                             (fold sum (nodes NodeType/TERRITORY) \
                               (field-of a territory/wage-bill))) 5$)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-012")
        );
    }

    /// A name that is BOTH a binding and a `:as` name is `E-PARSE-030` at
    /// the declaration — references to the binding stay legal, so the
    /// rule-wide reference test must exclude declared bindings.
    #[test]
    fn a_as_name_colliding_with_a_binding_is_still_e_parse_030() {
        assert_eq!(
            load_code(&rule(
                "(bindings (binding wealth :field social-class/wealth)) \
                 (when (< (+ wealth (fold sum (nodes NodeType/ORGANIZATION) :as wealth \
                                      (field-of wealth organization/claim-strength))) 5$)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            Some("E-PARSE-030")
        );
    }

    // ------------------------------------------------- N2: `:as` in `:expr`

    /// §2.5 permits a `:expr` to contain a fold of its own, and §2.6 lets
    /// that fold name its element. The name is declared by the very
    /// expression being checked, so the declaration-order check must admit
    /// it — it was refusing spec-legal content as `E-PARSE-032`.
    #[test]
    fn a_as_name_inside_an_expr_binding_is_not_a_forward_reference() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) \
                 (bindings (binding total :expr (fold sum (nodes NodeType/SOCIAL_CLASS) :as c \
                                                  (field-of c social-class/wealth)))) \
                 (when (< total 5$)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            None
        );
    }

    /// …and a genuine forward reference inside a `:expr` is still
    /// `E-PARSE-032`, so admitting element names did not blunt D49.
    #[test]
    fn a_genuine_forward_reference_inside_an_expr_is_still_e_parse_032() {
        assert_eq!(
            load_code(&rule(
                "(bindings (binding early :expr (+ later 1)) \
                           (binding later :field social-class/population)) \
                 (effects (update-node self social-class/agitation (add 0.05i)))"
            )),
            Some("E-PARSE-032")
        );
    }

    // ------------------------------------ N3: folds inside `:expr` and §3.4

    /// A fold in a `:expr` operand faces §3.4 exactly as one in `<when>`
    /// does. The pair is the point: identical folds, identical verdict.
    #[test]
    fn a_fold_inside_an_expr_binding_faces_the_aggregation_law() {
        let in_when = rule(
            "(domain :graph) (bindings) \
             (when (< (fold mean (nodes NodeType/SOCIAL_CLASS) \
                        (field-of it social-class/agitation)) 0.5i)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        );
        let in_expr = rule(
            "(domain :graph) \
             (bindings (binding avg :expr (fold mean (nodes NodeType/SOCIAL_CLASS) \
                                            (field-of it social-class/agitation)))) \
             (when (< avg 0.5i)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        );
        assert_eq!(load_code(&in_when), Some("E-TYPE-042"));
        assert_eq!(
            load_code(&in_expr),
            Some("E-TYPE-042"),
            "a fold does not escape §3.4 by moving into a :expr binding"
        );
    }

    // ------------------------------- N7: domain inference and query operands

    /// §2.3's inference asks whether a `:field` binding is referenced
    /// **outside every query body**. A `neighbors`/`members-of` element
    /// operand IS outside one, so a binding read only there is self-scoped
    /// and must determine the domain. `referenced_at_rule_scope` treated
    /// every query child as inside a body and mis-inferred `E-LOAD-004`.
    #[test]
    fn a_binding_read_only_in_a_query_operand_still_determines_the_domain() {
        let form = e(&rule(
            "(bindings (binding home :field social-class/wealth)) \
             (when (< (fold count (members-of home HyperedgeType/COMMUNITY) 1) 5)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        ));
        let decls = parse_bindings(&form).unwrap();
        assert_eq!(
            resolve_domain(&form, &decls, &vocabulary()),
            Ok(RuleDomain::Node("social-class".to_owned())),
            "an element operand is evaluated at rule scope, so the binding is \
             self-scoped and `|U| = 1`"
        );
    }

    /// The dual: a binding read only inside a real query **body** stays
    /// fold-scoped and does NOT enter the inference — D43's stated
    /// property, which the fix must not break.
    #[test]
    fn a_binding_read_only_inside_a_query_body_still_stays_out_of_the_inference() {
        let form = e(&rule(
            "(domain :graph) \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) claim) 5)) \
             (effects (emit EventType/RUPTURE (x 1)))",
        ));
        let decls = parse_bindings(&form).unwrap();
        assert_eq!(
            resolve_domain(&form, &decls, &vocabulary()),
            Ok(RuleDomain::Graph)
        );
    }

    // ------------------------------------------ D90 (family 4, §3.4)

    /// D90, merged in PR #481: a **weighted** `mean` over an intensive body
    /// has result kind **intensive** — the one cell §3.4's table left blank
    /// while stating a result for its four other rows. Unit algebra
    /// (`Σ(w × x) / Σ(w)` is in the units of `x`), not new mathematics.
    ///
    /// The family-4 pair, with plain node fields: the same weighted fold
    /// under an outer `sum` must reject `E-TYPE-041`…
    #[test]
    fn an_outer_sum_over_a_weighted_intensive_mean_is_e_type_041() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (fold sum (nodes NodeType/SOCIAL_CLASS) \
                            (fold mean (nodes NodeType/SOCIAL_CLASS) \
                                  (field-of it social-class/agitation) \
                                  :weight (field-of it social-class/population))) 5)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            Some("E-TYPE-041")
        );
    }

    /// …and under an outer `max`, whose row is kind-neutral over any body,
    /// must accept.
    #[test]
    fn an_outer_max_over_a_weighted_intensive_mean_accepts() {
        assert_eq!(
            load_code(&rule(
                "(domain :graph) (bindings) \
                 (when (< (fold max (nodes NodeType/SOCIAL_CLASS) \
                            (fold mean (nodes NodeType/SOCIAL_CLASS) \
                                  (field-of it social-class/agitation) \
                                  :weight (field-of it social-class/population))) 0.5i)) \
                 (effects (emit EventType/RUPTURE (x 1)))"
            )),
            None
        );
    }

    /// The weighted fold is legal **at its own level** — D90 is about the
    /// result kind an enclosing fold sees, not about the inner fold.
    #[test]
    fn the_weighted_intensive_mean_is_itself_legal() {
        let env: TypeEnv = type_env();
        assert!(typecheck_aggregation(
            &e("(mean social-class/agitation :weight social-class/population)"),
            &env
        )
        .is_ok());
        assert_eq!(
            typecheck_aggregation(&e("(mean social-class/agitation)"), &env)
                .unwrap_err()
                .code,
            Some(TypeCode::UnweightedMeanOfIntensive)
        );
    }
}
