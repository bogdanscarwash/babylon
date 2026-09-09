//! Table-driven diagnostics conformance rows (issue #652 Task 6, plan
//! §5.3/§6): each row asserts `(code, severity, precision, family)`
//! against an EXISTING fixture or a minimal, purpose-built one,
//! exercising the mapping layer ([`babylon_ls::diagnostics`]) and the
//! locator ([`babylon_ls::locator`]) end to end. A later wave improving
//! precision produces a visible, intentional diff here (§5.3's own
//! framing) — this suite pins today's wave-1 contract, not a permanent
//! ceiling. `family` is pinned on every row whose code exercises a
//! distinct census family (§1.1) — rows 1/2/3 span `E-PARSE`/`E-LEX`/
//! `E-LOAD`; row 4 shares `E-LOAD` with row 3 via a DIFFERENT mechanism
//! (`family_of_scenario_error`'s positionless/uncoded default, not a
//! code prefix — `ScenarioError::with_identity`'s duplicate-`defconst`
//! case carries no `code` at all); row 5's Information notice carries no
//! `data` at all (§6.3: not one of the loader's own tiers), asserted
//! explicitly so the omission reads as intentional, not missed.
//!
//! Plus the determinism row (§5.3): diagnosing the same content set twice
//! — and again with its rule sources supplied in reverse order — produces
//! byte-identical serialized diagnostic arrays.

use std::collections::{HashMap, HashSet};

use babylon_bsl::rule_pipeline::{load_rule, LoadContext, LoadError};
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::{
    BindingVocabulary, CardinalityCeilings, DeclError, EnumRegistry, IntrinsicCosts, TypeEnv, Value,
};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_tick::diagnose_content_set;
use lsp_types::{DiagnosticSeverity, NumberOrString, Uri};

use babylon_ls::content_manifest::ContentSetManifest;
use babylon_ls::diagnostics::{diagnostics_for_file, Located};
use babylon_ls::line_index::LineIndex;
use babylon_ls::pass::{diagnose_bsl, FixtureSourceReader};

/// `babylon-bsl`'s own conformance corpus — the deliberately-rejecting
/// `E-PARSE-020` vector this crate's fixtures reuse rather than
/// duplicate.
const EMPTY_WHEN: &str = include_str!("../../babylon-bsl/tests/conformance/empty_when.bsl");

fn uri(path: &str) -> Uri {
    format!("file:///{path}")
        .parse::<Uri>()
        .expect("valid test URI")
}

fn code_of(d: &lsp_types::Diagnostic) -> Option<&str> {
    match &d.code {
        Some(NumberOrString::String(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn precision_of(d: &lsp_types::Diagnostic) -> String {
    d.data
        .as_ref()
        .and_then(|v| v.get("precision"))
        .and_then(|v| v.as_str())
        .unwrap_or("<none>")
        .to_owned()
}

fn family_of(d: &lsp_types::Diagnostic) -> String {
    d.data
        .as_ref()
        .and_then(|v| v.get("family"))
        .and_then(|v| v.as_str())
        .unwrap_or("<none>")
        .to_owned()
}

/// A minimal `LoadContext` builder over empty vocabulary/types/ceilings/
/// intrinsics — the same shape `babylon-bsl/tests/conformance_corpus.rs`'s
/// own `load()` helper uses for `vocabulary_registry: None` runs; `systems`
/// is the one thing a fixture's own rule id actually needs (the §2.3
/// anchor default).
struct Minimal {
    vocabulary: BindingVocabulary,
    types: TypeEnv,
    enums: EnumRegistry,
    const_values: HashMap<String, Value>,
    ceilings: CardinalityCeilings,
    intrinsics: IntrinsicCosts,
    systems: HashSet<String>,
}

impl Minimal {
    fn with_systems(systems: &[&str]) -> Self {
        Self {
            vocabulary: BindingVocabulary::default(),
            types: TypeEnv {
                fields: HashMap::new(),
                exemptions: &[],
            },
            enums: EnumRegistry::default(),
            const_values: HashMap::new(),
            ceilings: CardinalityCeilings::default(),
            intrinsics: IntrinsicCosts::default(),
            systems: systems.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    fn context<'a>(&'a self, rule_file: &'a str) -> LoadContext<'a> {
        LoadContext {
            vocabulary: &self.vocabulary,
            types: &self.types,
            enums: &self.enums,
            const_values: &self.const_values,
            ceilings: &self.ceilings,
            intrinsics: &self.intrinsics,
            systems: &self.systems,
            vocabulary_registry: None,
            rule_file,
        }
    }
}

/// **Row 1**: `empty_when.bsl` ⇒ one diagnostic, `code == "E-PARSE-020"`,
/// `precision == "file"` (a unit `BoundError` variant with no field to
/// locate an identity from — §6.2's Prose/File tier, `bound_checker.rs`'s
/// own `EmptyWhenCondition` doc).
#[test]
fn row_1_empty_when_is_one_e_parse_020_file_tier_diagnostic() {
    let registries = Minimal::with_systems(&["event"]);
    let ctx = registries.context("empty_when.bsl");
    let err = load_rule(EMPTY_WHEN, &ctx).expect_err("(when) must be rejected");
    let located = Located::from_load_error(&err);
    let line_index = LineIndex::new(EMPTY_WHEN);
    let diags = diagnostics_for_file(&uri("empty_when.bsl"), EMPTY_WHEN, &line_index, &[located]);
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(code_of(&diags[0]), Some("E-PARSE-020"));
    assert_eq!(precision_of(&diags[0]), "file");
    assert_eq!(family_of(&diags[0]), "E-PARSE");
    assert_eq!(diags[0].severity, Some(DiagnosticSeverity::ERROR));
}

/// **Row 2**: a lexical error ⇒ `precision == "exact"` with the span
/// covering the offending token — the same `~=` vector
/// `conformance_corpus.rs`'s own correction-3 test uses (`E-LEX-003`,
/// `UnclassifiableToken`).
#[test]
fn row_2_a_lexical_error_is_exact_tier_over_the_offending_token() {
    let source = "(~= agitation 0.5p)";
    let read_err = babylon_bsl::read(source).expect_err("~= is not a valid comparison operator");
    let located = Located::from_load_error(&LoadError::Read(read_err));
    let line_index = LineIndex::new(source);
    let diags = diagnostics_for_file(&uri("x.bsl"), source, &line_index, &[located]);
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(code_of(&diags[0]), Some("E-LEX-003"));
    assert_eq!(precision_of(&diags[0]), "exact");
    assert_eq!(family_of(&diags[0]), "E-LEX");
    let range = diags[0].range;
    assert_eq!((range.start.line, range.start.character), (0, 1));
    assert_eq!((range.end.line, range.end.character), (0, 3));
    assert_eq!(&source[1..3], "~=");
}

const FLOOR_INTRINSIC: &str = "(intrinsic floor :params (real) :returns int :cost 5)";
const PROBE_RULE: &str = "(rule vitality/probe :role mechanic :evidence derived :material-basis \"x\" :fuel 16 (bindings) \
                           (effects (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))";
const PROBE_SCENARIO: &str = "(scenario ft/probe)";

fn duplicate_intrinsic_manifest() -> ContentSetManifest {
    let toml = r#"
schema = 2
[[set]]
id = "probe/duplicate-intrinsic"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/probe.bsl"]
consumers = []
note = "diagnostics_conformance row 3 fixture"
"#;
    ContentSetManifest::parse(std::path::Path::new("content-sets.toml"), toml)
        .expect("valid manifest")
}

/// **Row 3**: a duplicate-intrinsic content set ⇒ `code == "E-LOAD-001"`,
/// `precision == "form"`, range on the SECOND declaration —
/// `DeclError::Duplicate` carries `ErrorIdentity::Name`, located by
/// `by_atom` within the diagnosed file's own forest. Two literal
/// occurrences of the SAME name ⇒ the locator's `Ambiguous` outcome
/// (§6.2: "Ambiguous ⇒ file-level plus one `relatedInformation` per
/// candidate"), sorted into document order — `relatedInformation[1]` IS
/// the second declaration.
#[test]
fn row_3_duplicate_intrinsic_is_form_tier_with_the_second_declaration_related() {
    let manifest = duplicate_intrinsic_manifest();
    let rule_source = format!("{FLOOR_INTRINSIC} {FLOOR_INTRINSIC} {PROBE_RULE}");
    let source = FixtureSourceReader {
        files: [
            ("scenario.bscn".to_owned(), PROBE_SCENARIO.to_owned()),
            ("rules/probe.bsl".to_owned(), rule_source.clone()),
        ]
        .into_iter()
        .collect(),
    };
    let diags = diagnose_bsl(
        &uri("rules/probe.bsl"),
        "rules/probe.bsl",
        &manifest,
        &source,
    );
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(code_of(&diags[0]), Some("E-LOAD-001"));
    assert_eq!(precision_of(&diags[0]), "form");
    assert_eq!(family_of(&diags[0]), "E-LOAD");
    let related = diags[0]
        .related_information
        .as_ref()
        .expect("ambiguous outcome carries relatedInformation");
    assert_eq!(related.len(), 2);
    let second = &related[1].location.range;
    let second_start = usize::try_from(second.start.character).unwrap();
    let second_form_start = rule_source.rfind("(intrinsic floor").unwrap();
    // `second_form_start` is the SECOND `(intrinsic floor ...)` form's own
    // opening paren; its `floor` atom starts 10 bytes later
    // ("(intrinsic ".len()).
    assert_eq!(second_start, second_form_start + "(intrinsic ".len());
}

/// **Row 4**: a `.bscn` duplicate-`defconst` ⇒ `precision == "form"` — the
/// row that proves Task 2's `.bscn` identity fix (`ScenarioError::
/// with_identity`, `scenario.rs`'s `load_defconst`) reaches the client.
/// Same fixture source as `scenario.rs`'s own
/// `a_duplicate_defconst_carries_a_name_identity` test.
#[test]
fn row_4_bscn_duplicate_defconst_is_form_tier() {
    let source = r"
(scenario ft/twice
  (defconst economy/base-subsistence 0.0005c)
  (defconst economy/base-subsistence 0.5c))
";
    let mut graph = HypergraphStore::new();
    let err = load_scenario(source, &mut graph).expect_err("a duplicate defconst must be rejected");
    let located = Located::from_scenario_error(&err);
    let line_index = LineIndex::new(source);
    let diags = diagnostics_for_file(&uri("ft.bscn"), source, &line_index, &[located]);
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(
        code_of(&diags[0]),
        None,
        "load_defconst's duplicate check is uncoded"
    );
    assert_eq!(precision_of(&diags[0]), "form");
    // Uncoded AND positionless — `family_of_scenario_error`'s own
    // default, not a code-prefix split (row 3's mechanism): a duplicate
    // `defconst` is a scenario-hydration structural fact, `E-LOAD`.
    assert_eq!(family_of(&diags[0]), "E-LOAD");
}

/// **Row 5**: a `.bsl` with no manifest row ⇒ one Information diagnostic
/// naming the missing row (§6.3's own manifest-drift alarm).
#[test]
fn row_5_a_bsl_with_no_manifest_row_gets_the_information_notice() {
    let manifest = duplicate_intrinsic_manifest(); // names only "rules/probe.bsl"
    let source = FixtureSourceReader {
        files: [("rules/orphan.bsl".to_owned(), PROBE_RULE.to_owned())]
            .into_iter()
            .collect(),
    };
    let diags = diagnose_bsl(
        &uri("rules/orphan.bsl"),
        "rules/orphan.bsl",
        &manifest,
        &source,
    );
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].severity, Some(DiagnosticSeverity::INFORMATION));
    assert_eq!(diags[0].code, None);
    assert!(diags[0].message.contains("rules/orphan.bsl"), "{diags:?}");
    // Not a loader refusal (§6.3) — `data` (and so `family`/`precision`)
    // is entirely absent, deliberately, not merely unpinned.
    assert_eq!(diags[0].data, None);
    assert_eq!(family_of(&diags[0]), "<none>");
}

/// **Determinism row** (§5.3): diagnosing one content set twice — and
/// again with its rule sources in reverse order — produces byte-identical
/// serialized diagnostic arrays. `diagnose_content_set` loads each rule
/// form independently and appends failures in `rule_srcs`' own order
/// (its own doc); [`diagnostics_for_file`]'s declared total order
/// (`(range, code, message)`) is what neutralizes that input-order
/// dependency — this row is the proof.
#[test]
fn determinism_row_repeat_and_reversed_order_are_byte_identical() {
    let toml = r#"
schema = 2
[[set]]
id = "probe/determinism"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/a.bsl", "rules/b.bsl"]
consumers = []
note = "determinism row fixture"
"#;
    let manifest = ContentSetManifest::parse(std::path::Path::new("content-sets.toml"), toml)
        .expect("valid manifest");
    // Two INDEPENDENT static-shape rejections, each uncoded/File-tier —
    // `(when)`'s own `E-PARSE-020` in one file, a missing `:fuel` in the
    // other — chosen so their outcome does not depend on load ORDER, only
    // on the total-order SORT proving output order is input-order-blind.
    let rule_a = EMPTY_WHEN.replace("event/empty-when", "vitality/a");
    let rule_b =
        "(rule vitality/b :role mechanic :evidence derived :material-basis \"x\" (bindings) \
                  (effects (update-node self social-class/agitation (add 0.05i))))";
    let source = FixtureSourceReader {
        files: [
            ("scenario.bscn".to_owned(), PROBE_SCENARIO.to_owned()),
            ("rules/a.bsl".to_owned(), rule_a),
            ("rules/b.bsl".to_owned(), rule_b.to_owned()),
        ]
        .into_iter()
        .collect(),
    };
    let first = diagnose_bsl(&uri("rules/a.bsl"), "rules/a.bsl", &manifest, &source);
    let second = diagnose_bsl(&uri("rules/a.bsl"), "rules/a.bsl", &manifest, &source);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
    assert!(
        !first.is_empty(),
        "fixture must actually produce diagnostics"
    );

    let reversed_toml = toml.replace(
        r#"rules = ["rules/a.bsl", "rules/b.bsl"]"#,
        r#"rules = ["rules/b.bsl", "rules/a.bsl"]"#,
    );
    let reversed_manifest =
        ContentSetManifest::parse(std::path::Path::new("content-sets.toml"), &reversed_toml)
            .expect("valid manifest");
    let reversed = diagnose_bsl(
        &uri("rules/a.bsl"),
        "rules/a.bsl",
        &reversed_manifest,
        &source,
    );
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&reversed).unwrap(),
        "reversing rule_srcs order must not change the serialized diagnostic array"
    );
}

/// Sanity: `diagnose_content_set`'s own duplicate-intrinsic construction
/// path (`DeclError::Duplicate`) is reachable through `babylon-tick`
/// directly too, not only through the `pass`-layer fixture above — cheap
/// insurance against the two call paths drifting.
#[test]
fn diagnose_content_set_duplicate_intrinsic_carries_e_load_001() {
    let rule_source = format!("{FLOOR_INTRINSIC} {FLOOR_INTRINSIC} {PROBE_RULE}");
    let errors = diagnose_content_set(PROBE_SCENARIO, None, &[&rule_source]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-LOAD-001"));
    let located = Located::from_prepare_error(&errors[0]);
    assert_eq!(located.code, Some("E-LOAD-001"));
    assert_eq!(located.family, "E-LOAD");
    assert!(matches!(
        located.identity,
        Some(babylon_bsl::ErrorIdentity::Name(ref n)) if n == "floor"
    ));
    // Confirms the `DeclError` variant this row exercises really is the
    // one `error_identity.rs`'s roster names for `E-LOAD-001` (`Name` from
    // `DeclError::Duplicate`) — not a different duplicate-shaped error.
    let expected_identity_source = DeclError::Duplicate {
        name: "floor".to_owned(),
        what: "intrinsic",
    };
    assert_eq!(expected_identity_source.spec_code(), Some("E-LOAD-001"));
}

#[test]
fn diagnose_content_set_duplicate_rule_carries_typed_location_data() {
    let errors = diagnose_content_set(PROBE_SCENARIO, None, &[PROBE_RULE, PROBE_RULE]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-LOAD-001"));

    let located = Located::from_prepare_error(&errors[0]);
    assert_eq!(located.code, Some("E-LOAD-001"));
    assert_eq!(located.family, "E-LOAD");
    assert!(matches!(
        located.identity,
        Some(babylon_bsl::ErrorIdentity::RuleId(ref id)) if id == "vitality/probe"
    ));
}
