use std::collections::HashMap;
use std::fmt::Write;

use babylon_bsl::scenario::{compose_declaration_preludes, load_scenario_with_prelude};
use babylon_bsl::{
    read, typecheck_aggregation, FieldKind, TypeCode, TypeEnv, EXTENSIVE_INTENSIVE_EXEMPTIONS,
};
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::GraphSubstrate;
use babylon_kernel::sha256_of;
use babylon_practice_contract::{practice_machine_verb, PracticeIdV1, VerbModeV1, VerbStemV1};

const PRACTICE_PRELUDE: &str = include_str!("../content/declarations/organization-practice.bscn");
const PRACTICE_SCENARIO: &str =
    include_str!("../content/scenarios/organization-practice-contract.bscn");
const PRACTICE_SCHEMA: &str = include_str!("../../../../contracts/practice_contract_v1.yaml");
const VECTOR_CORPUS: &str =
    include_str!("../../../../contracts/practice_contract_v1_vectors.jsonl");

const PROMOTED_FIELDS: [&str; 6] = [
    "organization/kind",
    "organization/active",
    "organization/cadre-level",
    "organization/cohesion",
    "organization/consciousness-tendency",
    "organization/action-budget",
];

const PROMOTED_SCENARIOS: [(&str, &str); 11] = [
    ("organization/foundation", "organization-foundation.bscn"),
    (
        "community/carrier-collision",
        "community-carrier-collision-conformance.bscn",
    ),
    ("community/conformance", "community-conformance.bscn"),
    (
        "community/cost-modifier",
        "community-cost-modifier-conformance.bscn",
    ),
    (
        "community/decay-arc",
        "community-decay-arc-conformance.bscn",
    ),
    (
        "community/degenerate",
        "community-degenerate-conformance.bscn",
    ),
    ("community/empty", "community-empty-conformance.bscn"),
    ("community/floor", "community-floor-conformance.bscn"),
    (
        "community/solidarity-seam",
        "community-solidarity-seam-conformance.bscn",
    ),
    ("community/tie", "community-tie-conformance.bscn"),
    (
        "consciousness/ternary-conformance",
        "consciousness-ternary-conformance.bscn",
    ),
];

fn load_contract(prelude: &str, scenario: &str) -> babylon_bsl::scenario::LoadedScenario {
    let mut graph = MemoryGraph::new();
    load_scenario_with_prelude(prelude, scenario, &mut graph).expect("practice contract loads")
}

fn ordinal(loaded: &babylon_bsl::scenario::LoadedScenario, enum_name: &str, member: &str) -> u32 {
    let enum_id = loaded.enums.resolve(enum_name).expect("declared enum");
    loaded
        .enums
        .ordinal(enum_id, member)
        .expect("declared member")
}

fn raw_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes.iter().take(1_048_577) {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

#[test]
fn declaration_ordinals_match_contract() {
    let loaded = load_contract(PRACTICE_PRELUDE, PRACTICE_SCENARIO);
    assert_eq!(ordinal(&loaded, "VerbMode", "CANVASS"), 0);
    assert_eq!(ordinal(&loaded, "VerbMode", "AGITATE"), 1);
    assert_eq!(ordinal(&loaded, "ConsciousnessTendency", "LIBERAL"), 0);
    assert_eq!(ordinal(&loaded, "ConsciousnessTendency", "FASCIST"), 1);
    assert_eq!(
        ordinal(&loaded, "ConsciousnessTendency", "REVOLUTIONARY"),
        2
    );
}

#[test]
fn practice_machine_verbs_match_declared_modes() {
    let loaded = load_contract(PRACTICE_PRELUDE, PRACTICE_SCENARIO);
    let organize = practice_machine_verb(PracticeIdV1::Organize);
    let agitate = practice_machine_verb(PracticeIdV1::Agitate);
    let mutual_aid = practice_machine_verb(PracticeIdV1::MutualAid);
    assert_eq!(
        (organize.stem, organize.mode),
        (VerbStemV1::Mobilize, Some(VerbModeV1::Canvass))
    );
    assert_eq!(
        (agitate.stem, agitate.mode),
        (VerbStemV1::Mobilize, Some(VerbModeV1::Agitate))
    );
    assert_eq!((mutual_aid.stem, mutual_aid.mode), (VerbStemV1::Aid, None));
    for (practice, expected_member) in [
        (PracticeIdV1::Organize, Some("CANVASS")),
        (PracticeIdV1::Agitate, Some("AGITATE")),
        (PracticeIdV1::MutualAid, None),
    ] {
        let machine_member = practice_machine_verb(practice).mode.map(|mode| match mode {
            VerbModeV1::Canvass => "CANVASS",
            VerbModeV1::Agitate => "AGITATE",
        });
        assert_eq!(machine_member, expected_member);
        if let Some(member) = machine_member {
            let verb_mode = loaded.enums.resolve("VerbMode").unwrap();
            assert!(loaded.enums.ordinal(verb_mode, member).is_some());
        }
    }
}

#[test]
fn wire_discriminants_are_stable() {
    assert_eq!(PracticeIdV1::Organize as u8, 1);
    assert_eq!(PracticeIdV1::Agitate as u8, 2);
    assert_eq!(PracticeIdV1::MutualAid as u8, 3);
    assert_eq!(VerbModeV1::Canvass as u8, 1);
    assert_eq!(VerbModeV1::Agitate as u8, 2);
}

#[test]
fn display_labels_match_contract() {
    for label in ["ORGANIZE", "AGITATE", "MUTUAL-AID"] {
        assert!(PRACTICE_SCHEMA.contains(&format!("display_label: {label}")));
    }
}

#[test]
fn prelude_identity_vector_matches_bytes_and_digest() {
    let mut identity = None;
    let mut identity_count = 0_usize;
    for line in VECTOR_CORPUS.lines().take(513) {
        let value: serde_json::Value = serde_json::from_str(line).expect("valid vector JSON");
        if value["kind"] == "organization-practice-prelude" {
            identity_count += 1;
            identity = Some(value);
        }
    }
    assert_eq!(identity_count, 1);
    let identity = identity.expect("one prelude identity vector");
    assert_eq!(
        identity["data"]["raw_hex"].as_str().unwrap(),
        raw_hex(PRACTICE_PRELUDE.as_bytes())
    );
    let digest = babylon_tick::hex(&sha256_of(PRACTICE_PRELUDE.as_bytes()));
    assert_eq!(
        digest,
        "3fe86a1f60114b56a6141ec31a61f824119cadf74be7dd1788e5a166492fdcc4"
    );
    assert_eq!(identity["data"]["digest_hex"].as_str().unwrap(), digest);
}

#[test]
fn prelude_is_declaration_only_and_scenario_is_the_topology_witness() {
    assert!(!PRACTICE_PRELUDE.contains("NodeType"));
    assert!(!PRACTICE_PRELUDE.contains("EdgeType"));
    assert!(!PRACTICE_SCENARIO.contains("(rule "));
    assert!(PRACTICE_SCENARIO.contains(
        "(defvocabulary EdgeType\n    (MEMBERSHIP PRESENCE COMMAND TRANSACTIONAL SOLIDARISTIC \
SOLIDARITY\n     TENANCY ADJACENCY))"
    ));

    let mut graph = MemoryGraph::new();
    let loaded = load_scenario_with_prelude(PRACTICE_PRELUDE, PRACTICE_SCENARIO, &mut graph)
        .expect("topology witness loads");
    let ids: HashMap<&str, _> = loaded
        .node_content_ids
        .iter()
        .map(|(id, name)| (name.as_str(), *id))
        .collect();
    let organization = ids["practice-organization"];
    let territory = ids["practice-territory"];
    let class = ids["practice-class"];
    assert_eq!(graph.edges("PRESENCE"), vec![(organization, territory)]);
    assert_eq!(graph.edges("TENANCY"), vec![(class, territory)]);
    assert_eq!(graph.edges("MEMBERSHIP"), vec![(organization, class)]);
    assert_eq!(graph.edges("SOLIDARITY"), vec![(organization, class)]);
    let strength = graph
        .edge_attribute("SOLIDARITY", organization, class, "solidarity/strength")
        .expect("solidarity strength");
    assert!(strength.is_finite() && strength > 0.0);
}

#[test]
fn action_budget_is_intensive_and_aggregation_law_is_pinned() {
    let loaded = load_contract(PRACTICE_PRELUDE, PRACTICE_SCENARIO);
    let budget = loaded
        .fields
        .get("organization/action-budget")
        .expect("action budget declaration");
    assert_eq!(budget.kind, FieldKind::Intensive);
    let env = TypeEnv {
        fields: loaded.fields,
        exemptions: EXTENSIVE_INTENSIVE_EXEMPTIONS,
    };
    let code = |form: &str| {
        typecheck_aggregation(&read(form).expect("valid aggregation form").0, &env)
            .err()
            .and_then(|error| error.code)
    };
    assert_eq!(
        code("(sum organization/action-budget)"),
        Some(TypeCode::SumOfIntensive)
    );
    assert_eq!(
        code("(mean organization/action-budget)"),
        Some(TypeCode::UnweightedMeanOfIntensive)
    );
    for form in [
        "(min organization/action-budget)",
        "(max organization/action-budget)",
        "(count organization/action-budget)",
    ] {
        let result = typecheck_aggregation(&read(form).expect("valid aggregation form").0, &env);
        assert!(result.is_ok(), "{form}: {result:?}");
    }
}

#[test]
fn prelude_composition_is_ordered_and_bounded() {
    let first = "(defenum A (ONE))\n";
    let second = "(defenum B (TWO))\n";
    assert_eq!(
        compose_declaration_preludes(&[first, second]).unwrap(),
        format!("{first}{second}")
    );
    assert_eq!(compose_declaration_preludes(&["\n"]).unwrap(), "\n");
    assert!(compose_declaration_preludes(&["\n"; 17]).is_err());
    assert!(compose_declaration_preludes(&[&"x".repeat(262_145)]).is_err());
    let max_source = format!("{}\n", "x".repeat(262_143));
    assert_eq!(max_source.len(), 262_144);
    assert_eq!(
        compose_declaration_preludes(&[&max_source, &max_source, &max_source, &max_source])
            .unwrap()
            .len(),
        1_048_576
    );
    assert!(compose_declaration_preludes(&[
        &max_source,
        &max_source,
        &max_source,
        &max_source,
        "\n"
    ])
    .is_err());
    for invalid in [
        "",
        "(defenum A (ONE))",
        "(defenum A (ONE))\n\n",
        "(defenum A\r (ONE))\n",
    ] {
        assert!(
            compose_declaration_preludes(&[invalid]).is_err(),
            "{invalid:?}"
        );
    }
}

#[test]
fn all_promoted_scenarios_use_the_shared_prelude() {
    let content_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("content");
    let manifest =
        std::fs::read_to_string(content_root.join("content-sets.toml")).expect("content manifest");
    for (set_id, scenario_path) in PROMOTED_SCENARIOS {
        let scenario = std::fs::read_to_string(content_root.join("scenarios").join(scenario_path))
            .expect("promoted scenario");
        assert!(manifest.contains(&format!("id        = \"{set_id}\"")));
        let row_start = manifest
            .find(&format!("id        = \"{set_id}\""))
            .expect("manifest row");
        let row = &manifest[row_start..];
        let row_end = row[1..]
            .find("[[set]]")
            .map_or(row.len(), |index| index + 1);
        assert!(
            row[..row_end].contains("declarations/organization-practice.bscn"),
            "{set_id}"
        );
        for enum_name in ["OrgKind", "ConsciousnessTendency"] {
            assert!(
                !scenario.contains(&format!("(defenum {enum_name} ")),
                "{set_id}"
            );
        }
        for field in PROMOTED_FIELDS {
            assert!(
                !scenario.contains(&format!("(deffield {field} ")),
                "{set_id}: {field}"
            );
        }
    }
}

#[test]
fn mismatch_mutations_do_not_satisfy_the_registry_contract() {
    let reordered = PRACTICE_PRELUDE.replace(
        "(defenum VerbMode (CANVASS AGITATE))",
        "(defenum VerbMode (AGITATE CANVASS))",
    );
    let loaded = load_contract(&reordered, PRACTICE_SCENARIO);
    assert_ne!(ordinal(&loaded, "VerbMode", "CANVASS"), 0);

    let pseudo_mode = PRACTICE_PRELUDE.replace(
        "(defenum VerbMode (CANVASS AGITATE))",
        "(defenum VerbMode (CANVASS AGITATE MUTUAL_AID))",
    );
    let loaded = load_contract(&pseudo_mode, PRACTICE_SCENARIO);
    let verb_mode = loaded.enums.resolve("VerbMode").unwrap();
    assert_ne!(loaded.enums.member_count(verb_mode), 2);

    let duplicate = format!(
        "{PRACTICE_PRELUDE}(deffield organization/consciousness-tendency enum ConsciousnessTendency)\n"
    );
    let mut graph = MemoryGraph::new();
    assert!(load_scenario_with_prelude(&duplicate, PRACTICE_SCENARIO, &mut graph).is_err());

    let extensive = PRACTICE_PRELUDE.replace(
        "(deffield organization/action-budget int intensive)",
        "(deffield organization/action-budget int extensive)",
    );
    let mut graph = MemoryGraph::new();
    assert!(load_scenario_with_prelude(&extensive, PRACTICE_SCENARIO, &mut graph).is_err());

    assert_ne!(
        sha256_of(reordered.as_bytes()),
        sha256_of(PRACTICE_PRELUDE.as_bytes())
    );
    assert!(!PRACTICE_SCHEMA
        .replace("display_label: MUTUAL-AID", "display_label: MUTUAL_AID")
        .contains("display_label: MUTUAL-AID"));
}
