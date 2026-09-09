//! Behavioral contract for the opt-in synthetic-emergence BSL auditor.

use babylon_bsl::causal_contract::MAX_AST_WALK_DEPTH;
use babylon_bsl::declarations::FieldRegistry;
use babylon_bsl::rule_pipeline::{load_rule, LoadContext};
use babylon_bsl::scenario::{load_scenario, LoadedScenario};
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::vocabulary::EnumKind;
use babylon_bsl::{
    audit_rule_footprint, canonical_bytes, check_rule, read, validate_sfs_rule_profile, Atom,
    BindingVocabulary, BoundError, CardinalityCeilings, ClosedVocabulary, ForbiddenBindingSource,
    GovernedComparisonSite, IntrinsicCosts, SExpr, SfsAuditPolicy, SfsComparisonContext,
    SfsFuelIdentityError, SfsProfileError, Value,
};
use babylon_graph::memory::MemoryGraph;
use babylon_kernel::sha256_of;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_FIXTURE_BYTES: u64 = 262_144;
const MAX_FIXTURE_ROWS: usize = 32;
const EMPTY: [&str; 0] = [];
const ALLOWED_AST_HEX: &str = "50a9d50cf862a846004e68c314d33e9ead66dcce9f29bc2cf49fe7aeb3d7cd45";
const FIRST_SITE_HEX: &str = "d2529413ba20351ab91c63ff45a06930d7b9de1327cddfa67e6f7a74d85d2896";
const SECOND_SITE_HEX: &str = "1989aab489fa9dddc87801dec2389a4f360c20909607209d05c34d70ec8f82e9";
const CARDINALITY_HEX: &str = "58ef2f65a4137c5dfadd41855f6b40282fdcbbd4339cfd1e32047776b56c6474";
const CARDINALITY_TWO_ROW_HEX: &str =
    "9689f3b7c6cfee41597117f3c97c505f3f6c9406c48d98c6f377badff40140cc";
const EMPTY_INTRINSIC_HEX: &str =
    "2d35fcf8f676dfa9869eb8e18920d97d0d227cbce50a862eddc29e8f7dc3c6a1";
const UNUSED_INTRINSIC_HEX: &str =
    "aac6683a351c4f06dcc284d7d2330f6b233b5bc090b9596e5be93a177725e550";
const FORBIDDEN_INTRINSIC_HEX: &str =
    "1efb2cc310a127ffa35220cada6987bcf9aeff21c9c251699b16ee6344d2761c";
const FORBIDDEN_MANIFEST_HEX: &str =
    "e3e7d0c90b7302c441005a4cb482a1aff86c2e9178b06a514b2f9c6304aeca74";
const AUDIT_SOURCE_MANIFEST_HEX: &str =
    "91cfca31b605e3297db7e440db4007b0d15f228ce24461afdde6cb3859ce8487";
static READER_SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);
const FIXTURE_DECLARATIONS: &str = r"(scenario synthetic-source/declarations
  (defvocabulary NodeType (SYNTHETIC_SOURCE ORGANIZATION))
  (defvocabulary EdgeType (SYNTHETIC_LINK))
  (deffield synthetic-source/quanta int extensive)
  (defconst synthetic/minimum-link-strength 0)
  (defconst synthetic/transfer-quantum 1))";

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sfs_profile")
}

fn hex_digest(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    let mut output = [0_u8; 32];
    for index in 0..32 {
        output[index] = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    output
}

fn render_digest(value: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in value {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn string_set(values: &[&str]) -> BTreeSet<String> {
    let mut output = BTreeSet::new();
    for index in 0..64 {
        let Some(value) = values.get(index) else {
            break;
        };
        output.insert((*value).to_owned());
    }
    output
}

fn strict_relative(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    let mut selected = canonical_root.clone();
    let mut components = relative.components();
    for _index in 0..8 {
        let Some(component) = components.next() else {
            return Ok(selected);
        };
        let Component::Normal(name) = component else {
            return Err("fixture path must stay relative".to_owned());
        };
        selected.push(name);
        let metadata = std::fs::symlink_metadata(&selected).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("fixture path contains a symlink".to_owned());
        }
    }
    Err("fixture path exceeds eight components".to_owned())
}

fn read_bounded(root: &Path, relative: &Path) -> Result<Vec<u8>, String> {
    let selected = strict_relative(root, relative)?;
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    let canonical = selected.canonicalize().map_err(|error| error.to_string())?;
    if !canonical.starts_with(&canonical_root) {
        return Err("fixture path escapes its root".to_owned());
    }
    let file = File::open(&selected).map_err(|error| error.to_string())?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_FIXTURE_BYTES {
        return Err("fixture is not a bounded regular file".to_owned());
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap());
    file.take(MAX_FIXTURE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if u64::try_from(bytes.len()).unwrap() != metadata.len() {
        return Err("fixture changed during its bounded read".to_owned());
    }
    Ok(bytes)
}

fn strict_text(bytes: &[u8]) -> &str {
    assert!(!bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(!bytes.contains(&b'\r'));
    assert!(bytes.ends_with(b"\n"));
    assert!(!bytes.ends_with(b"\n\n"));
    std::str::from_utf8(bytes).unwrap()
}

fn fixture_rule(relative: &str) -> SExpr {
    let bytes = read_bounded(&fixture_root(), Path::new(relative)).unwrap();
    let source = strict_text(&bytes);
    let (rule, consumed) = read(source).unwrap();
    assert!(source[consumed..]
        .bytes()
        .all(|byte| byte.is_ascii_whitespace()));
    rule
}

fn fixture_declarations() -> LoadedScenario {
    load_scenario(FIXTURE_DECLARATIONS, &mut MemoryGraph::new()).unwrap()
}

fn fixture_vocabulary() -> ClosedVocabulary {
    fixture_declarations().vocabulary.unwrap()
}

fn fixture_cardinality_ceilings() -> CardinalityCeilings {
    CardinalityCeilings::new(
        HashMap::from([("EdgeType/SYNTHETIC_LINK".to_owned(), 8)]),
        HashMap::new(),
    )
}

fn assert_allowed_fixture_loads_through_declaration_and_type_gates() {
    let source = allowed_source();
    let declared = fixture_declarations();
    let vocabulary = declared.vocabulary.unwrap();
    assert_eq!(
        vocabulary.members(EnumKind::NodeType),
        &["ORGANIZATION".to_owned(), "SYNTHETIC_SOURCE".to_owned()]
    );
    assert_eq!(
        vocabulary.members(EnumKind::EdgeType),
        &["SYNTHETIC_LINK".to_owned()]
    );
    let quanta = &declared.fields["synthetic-source/quanta"];
    assert_eq!(quanta.ty, babylon_bsl::BslType::Int);
    assert_eq!(quanta.kind, babylon_bsl::FieldKind::Extensive);
    assert_eq!(
        vocabulary
            .owner_of_field("synthetic-source/quanta")
            .unwrap(),
        (EnumKind::NodeType, "SYNTHETIC_SOURCE")
    );
    let implicit = FieldRegistry::with_implicit_edge_strength(&vocabulary);
    let strength = implicit.get("synthetic-link/strength").unwrap();
    assert_eq!(strength.decl.ty, babylon_bsl::BslType::Coefficient);
    assert_eq!(strength.decl.kind, babylon_bsl::FieldKind::Extensive);
    assert_eq!(strength.owner_kind, EnumKind::EdgeType);
    assert_eq!(strength.owner_member, "SYNTHETIC_LINK");
    assert!(strength.implicit);
    assert_eq!(
        declared.consts["synthetic/minimum-link-strength"],
        Value::Int(0)
    );
    assert_eq!(declared.consts["synthetic/transfer-quantum"], Value::Int(1));
    let mut type_fields = declared.fields;
    type_fields.insert("synthetic-link/strength".to_owned(), strength.decl.clone());
    let types = TypeEnv {
        fields: type_fields,
        exemptions: &[],
    };
    let binding_vocabulary = BindingVocabulary {
        fields: HashSet::from([
            "synthetic-link/strength".to_owned(),
            "synthetic-source/quanta".to_owned(),
        ]),
        consts: HashSet::from([
            "synthetic/minimum-link-strength".to_owned(),
            "synthetic/transfer-quantum".to_owned(),
        ]),
        probability_consts: HashSet::new(),
        metrics: HashSet::new(),
    };
    let ceilings = fixture_cardinality_ceilings();
    let costs = IntrinsicCosts::default();
    let systems = HashSet::from(["synthetic-source".to_owned()]);
    let enums = babylon_bsl::types::EnumRegistry::default();
    let context = LoadContext {
        vocabulary: &binding_vocabulary,
        types: &types,
        enums: &enums,
        const_values: &declared.consts,
        ceilings: &ceilings,
        intrinsics: &costs,
        systems: &systems,
        vocabulary_registry: Some(&vocabulary),
        rule_file: "tests/fixtures/sfs_profile/allowed/scoped_mechanic.bsl",
    };
    load_rule(&source, &context).unwrap();
}

fn forbidden_intrinsic_costs() -> IntrinsicCosts {
    IntrinsicCosts::new(HashMap::from([
        ("exp".to_owned(), 7),
        ("log".to_owned(), 7),
        ("rng-draw".to_owned(), 12),
        ("sigmoid".to_owned(), 40),
    ]))
}

fn allowed_sites(rule: &SExpr) -> Vec<GovernedComparisonSite> {
    vec![
        GovernedComparisonSite::from_rule_path(
            rule,
            &[0, 11, 1, 1],
            SfsComparisonContext::ConservationRefusal,
        )
        .unwrap(),
        GovernedComparisonSite::from_rule_path(
            rule,
            &[0, 11, 1, 2],
            SfsComparisonContext::EligibilityNoEffect,
        )
        .unwrap(),
    ]
}

fn allowed_policy(rule: &SExpr) -> SfsAuditPolicy {
    SfsAuditPolicy::new(
        "synthetic-source/scoped-mechanic",
        sha256_of(&canonical_bytes(rule).unwrap()),
        31,
        ["synthetic-source/quanta"],
        ["synthetic-link/strength"],
        [
            "synthetic/minimum-link-strength",
            "synthetic/transfer-quantum",
        ],
        ["edges"],
        [">"],
        EMPTY,
        allowed_sites(rule),
        ["node:synthetic-source/quanta"],
    )
    .unwrap()
}

#[test]
fn allowed_rule_equals_its_complete_opt_in_profile() {
    assert_allowed_fixture_loads_through_declaration_and_type_gates();
    let rule = fixture_rule("allowed/scoped_mechanic.bsl");
    let vocabulary = fixture_vocabulary();
    let ceilings = fixture_cardinality_ceilings();
    let intrinsic_costs = IntrinsicCosts::default();
    let canonical_digest = sha256_of(&canonical_bytes(&rule).unwrap());
    assert_eq!(canonical_digest, hex_digest(ALLOWED_AST_HEX));
    let sites = allowed_sites(&rule);
    assert_eq!(sites[0].site_digest(), &hex_digest(FIRST_SITE_HEX));
    assert_eq!(sites[1].site_digest(), &hex_digest(SECOND_SITE_HEX));
    assert_eq!(
        sites[0].profile_entry(),
        format!("conservation-refusal:{FIRST_SITE_HEX}")
    );
    assert_eq!(
        sites[1].profile_entry(),
        format!("eligibility-no-effect:{SECOND_SITE_HEX}")
    );
    let policy = allowed_policy(&rule);
    let audit = validate_sfs_rule_profile(&rule, &vocabulary, &ceilings, &intrinsic_costs, &policy)
        .unwrap();
    let footprint = audit.footprint();
    assert_eq!(footprint.rule_id(), "synthetic-source/scoped-mechanic");
    assert_eq!(footprint.source_digest(), &canonical_digest);
    assert_eq!(footprint.computed_bound(), 31);
    assert_eq!(footprint, policy.expected_footprint());
    assert_eq!(
        footprint.field_reads(),
        &string_set(&["synthetic-source/quanta"])
    );
    assert_eq!(
        footprint.edge_reads(),
        &string_set(&["synthetic-link/strength"])
    );
    assert_eq!(footprint.constant_reads().len(), 2);
    assert_eq!(footprint.queries(), &string_set(&["edges"]));
    assert_eq!(footprint.operators(), &string_set(&[">"]));
    assert!(footprint.intrinsics().is_empty());
    assert_eq!(footprint.comparison_clamp_contexts().len(), 2);
    assert_eq!(
        footprint.effects(),
        &string_set(&["node:synthetic-source/quanta"])
    );
    assert_eq!(audit.declared_fuel(), 128);
    assert_eq!(
        audit.cardinality_input_digest(),
        &hex_digest(CARDINALITY_HEX)
    );
    assert_eq!(
        audit.intrinsic_cost_input_digest(),
        &hex_digest(EMPTY_INTRINSIC_HEX)
    );
}

#[derive(Debug)]
struct ManifestRow {
    label: String,
    relative: String,
    source_hex: String,
    table: String,
    expected: String,
}

fn forbidden_manifest() -> (Vec<ManifestRow>, Vec<u8>) {
    let bytes = read_bounded(&fixture_root(), Path::new("sfs_forbidden_manifest_v1.txt")).unwrap();
    let text = strict_text(&bytes);
    let mut rows = Vec::new();
    let mut lines = text.lines();
    let mut previous = "";
    for index in 0..MAX_FIXTURE_ROWS {
        let Some(line) = lines.next() else { break };
        assert!(!line.is_empty());
        if index > 0 {
            assert!(previous.as_bytes() < line.as_bytes());
        }
        let fields: Vec<&str> = line.split('|').collect();
        assert_eq!(fields.len(), 5);
        rows.push(ManifestRow {
            label: fields[0].to_owned(),
            relative: fields[1].to_owned(),
            source_hex: fields[2].to_owned(),
            table: fields[3].to_owned(),
            expected: fields[4].to_owned(),
        });
        previous = line;
    }
    assert!(lines.next().is_none());
    (rows, bytes)
}

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn source_manifest() -> (Vec<ManifestRow>, Vec<u8>) {
    let bytes = read_bounded(
        &fixture_root(),
        Path::new("sfs_audit_source_manifest_v1.txt"),
    )
    .unwrap();
    let text = strict_text(&bytes);
    let mut rows = Vec::new();
    let mut lines = text.lines();
    let mut previous = "";
    for index in 0..3 {
        let Some(line) = lines.next() else { break };
        if index > 0 {
            assert!(previous.as_bytes() < line.as_bytes());
        }
        let fields: Vec<&str> = line.split('|').collect();
        assert_eq!(fields.len(), 2);
        rows.push(ManifestRow {
            label: fields[0].to_owned(),
            relative: fields[0].to_owned(),
            source_hex: fields[1].to_owned(),
            table: String::new(),
            expected: String::new(),
        });
        previous = line;
    }
    assert!(lines.next().is_none());
    (rows, bytes)
}

fn verify_source_row(row: &ManifestRow, bytes: &[u8]) -> Result<(), SfsProfileError> {
    if sha256_of(bytes) != hex_digest(&row.source_hex) {
        return Err(SfsProfileError::SourceDigestMismatch);
    }
    Ok(())
}

#[test]
fn audit_source_manifest_pins_two_bounded_exact_files_and_mutation_teeth() {
    let (rows, manifest_bytes) = source_manifest();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].relative, "fuel.rs");
    assert_eq!(rows[1].relative, "sfs_profile.rs");
    let mut source_rows = rows.iter();
    for _index in 0..2 {
        let row = source_rows.next().unwrap();
        let bytes = read_bounded(&source_root(), Path::new(&row.relative)).unwrap();
        strict_text(&bytes);
        verify_source_row(row, &bytes).unwrap();
    }
    assert!(read_bounded(&source_root(), Path::new("../Cargo.toml")).is_err());
    let mut preimage = b"babylon.sfs-audit-source-manifest.v1\0".to_vec();
    preimage.extend_from_slice(&manifest_bytes);
    assert_eq!(sha256_of(&preimage), hex_digest(AUDIT_SOURCE_MANIFEST_HEX));

    let fuel_bytes = read_bounded(&source_root(), Path::new("fuel.rs")).unwrap();
    let needle = b"const MAX_SFS_IDENTITY_ROWS: usize = 64;";
    let position = fuel_bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap();
    let mut mutated = fuel_bytes.clone();
    mutated[position + needle.len() - 2] = b'5';
    assert_eq!(
        verify_source_row(&rows[0], &mutated),
        Err(SfsProfileError::SourceDigestMismatch)
    );
    let changed_manifest = strict_text(&manifest_bytes).replacen(
        &rows[0].source_hex,
        &render_digest(&sha256_of(&mutated)),
        1,
    );
    let mut changed_preimage = b"babylon.sfs-audit-source-manifest.v1\0".to_vec();
    changed_preimage.extend_from_slice(changed_manifest.as_bytes());
    assert_ne!(
        sha256_of(&changed_preimage),
        hex_digest(AUDIT_SOURCE_MANIFEST_HEX)
    );
}

struct ReaderScratch {
    root: PathBuf,
}

impl ReaderScratch {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "babylon-sfs-profile-reader-{}-{}",
            std::process::id(),
            READER_SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self { root }
    }
}

impl Drop for ReaderScratch {
    fn drop(&mut self) {
        for name in [
            "maximum.txt",
            "maximum-plus-one.txt",
            "manifest-target.txt",
            "sfs_audit_source_manifest_v1.txt",
        ] {
            let _ignored = std::fs::remove_file(self.root.join(name));
        }
        let _ignored = std::fs::remove_dir(&self.root);
    }
}

#[test]
fn bounded_reader_accepts_262144_and_rejects_262145_bytes() {
    let scratch = ReaderScratch::new();
    let maximum = scratch.root.join("maximum.txt");
    let maximum_plus_one = scratch.root.join("maximum-plus-one.txt");
    std::fs::write(&maximum, vec![b'x'; 262_144]).unwrap();
    std::fs::write(&maximum_plus_one, vec![b'x'; 262_145]).unwrap();
    assert_eq!(
        read_bounded(&scratch.root, Path::new("maximum.txt"))
            .unwrap()
            .len(),
        262_144
    );
    assert_eq!(
        read_bounded(&scratch.root, Path::new("maximum-plus-one.txt")),
        Err("fixture is not a bounded regular file".to_owned())
    );
}

#[cfg(unix)]
#[test]
fn bounded_reader_rejects_a_direct_source_manifest_symlink() {
    use std::os::unix::fs::symlink;

    let scratch = ReaderScratch::new();
    let target = scratch.root.join("manifest-target.txt");
    let link = scratch.root.join("sfs_audit_source_manifest_v1.txt");
    std::fs::write(&target, b"fuel.rs|00\n").unwrap();
    symlink(&target, &link).unwrap();
    assert_eq!(
        read_bounded(&scratch.root, Path::new("sfs_audit_source_manifest_v1.txt")),
        Err("fixture path contains a symlink".to_owned())
    );
}

fn forbidden_rule_id(label: &str) -> &'static str {
    match label {
        "absolute_schedule" => "synthetic-source/absolute-schedule",
        "comparison_selects_magnitude" => "synthetic-source/comparison-selects-magnitude",
        "comparison_without_context" => "synthetic-source/comparison-without-context",
        "dead_comparison_permission" => "synthetic-source/dead-comparison-permission",
        "dead_effect_permission" => "synthetic-source/dead-effect-permission",
        "direct_observable_read" => "synthetic-source/direct-observable-read",
        "direct_observable_write" => "synthetic-source/direct-observable-write",
        "exp_response" => "synthetic-source/exp-response",
        "log_response" => "synthetic-source/log-response",
        "named_shape" => "synthetic-source/named-shape",
        "response_table" => "synthetic-source/response-table",
        "rng_read" => "synthetic-source/rng-read",
        "threshold_ladder" => "synthetic-source/threshold-ladder",
        "tick_cycle_read" => "synthetic-source/tick-cycle-read",
        "tick_of_year_read" => "synthetic-source/tick-of-year-read",
        "tick_read" => "synthetic-source/tick-read",
        "unauthorized_effect" => "synthetic-source/unauthorized-effect",
        "year_read" => "synthetic-source/year-read",
        other => panic!("unknown forbidden fixture {other}"),
    }
}

fn policy_with_sets(
    row: &ManifestRow,
    rule: &SExpr,
    bound: u64,
    field_reads: impl IntoIterator<Item = &'static str>,
    operators: impl IntoIterator<Item = &'static str>,
    sites: Vec<GovernedComparisonSite>,
    effects: impl IntoIterator<Item = &'static str>,
) -> SfsAuditPolicy {
    SfsAuditPolicy::new(
        forbidden_rule_id(&row.label),
        sha256_of(&canonical_bytes(rule).unwrap()),
        bound,
        field_reads,
        EMPTY,
        EMPTY,
        EMPTY,
        operators,
        EMPTY,
        sites,
        effects,
    )
    .unwrap()
}

fn forbidden_policy(row: &ManifestRow, rule: &SExpr, bound: u64) -> SfsAuditPolicy {
    match row.label.as_str() {
        "comparison_without_context" => policy_with_sets(
            row,
            rule,
            bound,
            ["synthetic-source/quanta"],
            [">"],
            vec![],
            ["node:synthetic-source/quanta"],
        ),
        "dead_comparison_permission" => {
            let allowed = fixture_rule("allowed/scoped_mechanic.bsl");
            policy_with_sets(
                row,
                rule,
                bound,
                EMPTY,
                EMPTY,
                vec![allowed_sites(&allowed).remove(0)],
                ["node:synthetic-source/quanta"],
            )
        }
        "dead_effect_permission" => policy_with_sets(
            row,
            rule,
            bound,
            EMPTY,
            EMPTY,
            vec![],
            ["node:organization/cohesion", "node:synthetic-source/quanta"],
        ),
        "unauthorized_effect" => policy_with_sets(
            row,
            rule,
            bound,
            EMPTY,
            EMPTY,
            vec![],
            ["node:synthetic-source/quanta"],
        ),
        _ => policy_with_sets(row, rule, bound, EMPTY, EMPTY, vec![], EMPTY),
    }
}

fn assert_expected_error(row: &ManifestRow, error: &SfsProfileError) {
    match row.expected.as_str() {
        "ForbiddenAbsoluteSchedule" => {
            assert_eq!(error, &SfsProfileError::ForbiddenAbsoluteSchedule);
        }
        "ForbiddenBindingSource::Tick" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenBindingSource(ForbiddenBindingSource::Tick)
        ),
        "ForbiddenBindingSource::Year" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenBindingSource(ForbiddenBindingSource::Year)
        ),
        "ForbiddenBindingSource::TickOfYear" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenBindingSource(ForbiddenBindingSource::TickOfYear)
        ),
        "ForbiddenBindingSource::TickInCycle" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenBindingSource(ForbiddenBindingSource::TickInCycle)
        ),
        "ForbiddenIntrinsic(\"exp\")" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenIntrinsic {
                name: "exp".to_owned()
            }
        ),
        "ForbiddenIntrinsic(\"log\")" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenIntrinsic {
                name: "log".to_owned()
            }
        ),
        "ForbiddenIntrinsic(\"rng-draw\")" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenIntrinsic {
                name: "rng-draw".to_owned()
            }
        ),
        "ForbiddenIntrinsic(\"sigmoid\")" => assert_eq!(
            error,
            &SfsProfileError::ForbiddenIntrinsic {
                name: "sigmoid".to_owned()
            }
        ),
        "ForbiddenResponseTable" => assert_eq!(error, &SfsProfileError::ForbiddenResponseTable),
        "ForbiddenThresholdLadder" => {
            assert_eq!(error, &SfsProfileError::ForbiddenThresholdLadder);
        }
        "ForbiddenObservable" => {
            assert!(matches!(error, SfsProfileError::ForbiddenObservable { .. }));
        }
        "MissingComparisonContext" => {
            assert!(matches!(
                error,
                SfsProfileError::MissingComparisonContext { .. }
            ));
        }
        "ForbiddenComparisonUse" => {
            assert!(matches!(
                error,
                SfsProfileError::ForbiddenComparisonUse { .. }
            ));
        }
        "DeadComparisonContext" => {
            assert!(matches!(
                error,
                SfsProfileError::DeadComparisonContext { .. }
            ));
        }
        "FootprintMismatch" => {
            assert_eq!(
                error,
                &SfsProfileError::FootprintMismatch { set: "effects" }
            );
        }
        "UnexpectedEffect(\"node:organization/cohesion\")" => assert_eq!(
            error,
            &SfsProfileError::UnexpectedEffect {
                entry: "node:organization/cohesion".to_owned()
            }
        ),
        other => panic!("unknown manifest expectation {other}"),
    }
}

fn exact_forbidden_file_set(rows: &[ManifestRow]) {
    let directory = fixture_root().join("forbidden");
    let mut entries = std::fs::read_dir(&directory).unwrap();
    let mut actual = Vec::new();
    for _index in 0..MAX_FIXTURE_ROWS {
        let Some(entry) = entries.next() else { break };
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        actual.push(format!("forbidden/{}", entry.file_name().to_string_lossy()));
    }
    assert!(entries.next().is_none());
    actual.sort();
    let mut expected: Vec<String> = rows.iter().map(|row| row.relative.clone()).collect();
    expected.sort();
    assert_eq!(actual, expected);
}

#[test]
fn forbidden_corpus_exact() {
    let (rows, manifest_bytes) = forbidden_manifest();
    assert_eq!(rows.len(), 18);
    exact_forbidden_file_set(&rows);
    let vocabulary = fixture_vocabulary();
    let ceilings = fixture_cardinality_ceilings();
    for index in 0..MAX_FIXTURE_ROWS {
        let Some(row) = rows.get(index) else { break };
        let rule = fixture_rule(&row.relative);
        assert_eq!(
            sha256_of(&canonical_bytes(&rule).unwrap()),
            hex_digest(&row.source_hex),
            "{}",
            row.label
        );
        let costs = match row.table.as_str() {
            "empty" => IntrinsicCosts::default(),
            "forbidden-v1" => forbidden_intrinsic_costs(),
            other => panic!("unknown intrinsic table {other}"),
        };
        let bound = check_rule(&rule, &ceilings, &costs).unwrap();
        let policy = forbidden_policy(row, &rule, bound);
        let error =
            validate_sfs_rule_profile(&rule, &vocabulary, &ceilings, &costs, &policy).unwrap_err();
        assert_expected_error(row, &error);
    }
    let mut preimage = b"babylon.sfs-forbidden-corpus-manifest.v1\0".to_vec();
    preimage.extend_from_slice(&manifest_bytes);
    assert_eq!(sha256_of(&preimage), hex_digest(FORBIDDEN_MANIFEST_HEX));
    assert_eq!(
        forbidden_intrinsic_costs().sfs_identity_digest().unwrap(),
        hex_digest(FORBIDDEN_INTRINSIC_HEX)
    );
}

#[test]
fn forbidden_intrinsic_cost_preflight_precedes_semantic_refusal() {
    let rule = fixture_rule("forbidden/rng_read.bsl");
    assert!(check_rule(
        &rule,
        &fixture_cardinality_ceilings(),
        &forbidden_intrinsic_costs()
    )
    .is_ok());
    assert_eq!(
        check_rule(
            &rule,
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default()
        ),
        Err(BoundError::UndeclaredIntrinsic {
            name: "rng-draw".to_owned()
        })
    );
}

#[test]
fn absolute_schedule_requires_the_time_value_to_participate() {
    let source = r#"(rule synthetic-source/unrelated-time
  :role mechanic :evidence designed :material-basis "test-local time precedence" :fuel 128
  (bindings
    (binding current :tick)
    (binding available :field synthetic-source/quanta))
  (when (> available 1))
  (effects (update-node self synthetic-source/quanta (set 1))))"#;
    let rule = parsed_rule(source);
    assert_eq!(
        audit_rule_footprint(
            &rule,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &[],
        ),
        Err(SfsProfileError::ForbiddenBindingSource(
            ForbiddenBindingSource::Tick
        ))
    );
}

#[test]
fn forbidden_binding_sources_choose_the_byte_least_entry() {
    let rule = parsed_rule(
        r#"(rule synthetic-source/multiple-time-sources
  :role mechanic :evidence designed :material-basis "test-local source order" :fuel 128
  (bindings
    (binding annual :year)
    (binding cycle :tick-in-cycle 52))
  (effects (update-node self synthetic-source/quanta (set 1))))"#,
    );
    assert_eq!(
        audit_rule_footprint(
            &rule,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &[],
        ),
        Err(SfsProfileError::ForbiddenBindingSource(
            ForbiddenBindingSource::TickInCycle
        ))
    );
}

#[test]
fn fuel_table_identities_pin_literals_syntax_and_exact_row_bounds() {
    let base = fixture_cardinality_ceilings();
    assert_eq!(
        base.sfs_identity_digest().unwrap(),
        hex_digest(CARDINALITY_HEX)
    );
    let two_rows = CardinalityCeilings::new(
        HashMap::from([("EdgeType/SYNTHETIC_LINK".to_owned(), 8)]),
        HashMap::from([("HyperedgeType/SYNTHETIC_GROUP".to_owned(), 4)]),
    );
    assert_eq!(
        two_rows.sfs_identity_digest().unwrap(),
        hex_digest(CARDINALITY_TWO_ROW_HEX)
    );
    assert_eq!(
        IntrinsicCosts::default().sfs_identity_digest().unwrap(),
        hex_digest(EMPTY_INTRINSIC_HEX)
    );
    let unused = IntrinsicCosts::new(HashMap::from([("synthetic-unused".to_owned(), 7)]));
    assert_eq!(
        unused.sfs_identity_digest().unwrap(),
        hex_digest(UNUSED_INTRINSIC_HEX)
    );

    let mut sixty_five = HashMap::new();
    for index in 0..65 {
        sixty_five.insert(
            format!("synthetic-{index:02}"),
            u64::try_from(index).unwrap(),
        );
    }
    assert_eq!(
        IntrinsicCosts::new(sixty_five.clone()).sfs_identity_digest(),
        Err(SfsFuelIdentityError::RowLimit {
            table: "intrinsic",
            actual: 65
        })
    );
    sixty_five.remove("synthetic-64");
    assert!(IntrinsicCosts::new(sixty_five)
        .sfs_identity_digest()
        .is_ok());
    assert_eq!(
        IntrinsicCosts::new(HashMap::from([("bad|key".to_owned(), 1)])).sfs_identity_digest(),
        Err(SfsFuelIdentityError::KeyContainsDelimiter { table: "intrinsic" })
    );
    assert_eq!(
        IntrinsicCosts::new(HashMap::from([("é".to_owned(), 1)])).sfs_identity_digest(),
        Err(SfsFuelIdentityError::KeyNonAscii { table: "intrinsic" })
    );
    assert_eq!(
        IntrinsicCosts::new(HashMap::from([(String::new(), 1)])).sfs_identity_digest(),
        Err(SfsFuelIdentityError::KeyEmpty { table: "intrinsic" })
    );
    assert_eq!(
        IntrinsicCosts::new(HashMap::from([("x".repeat(97), 1)])).sfs_identity_digest(),
        Err(SfsFuelIdentityError::KeyTooLong {
            table: "intrinsic",
            actual: 97
        })
    );
    assert!(IntrinsicCosts::new(HashMap::from([("x".repeat(96), 1)]))
        .sfs_identity_digest()
        .is_ok());
    for key in ["bad\nkey", "bad\rkey"] {
        assert_eq!(
            IntrinsicCosts::new(HashMap::from([(key.to_owned(), 1)])).sfs_identity_digest(),
            Err(SfsFuelIdentityError::KeyContainsDelimiter { table: "intrinsic" })
        );
    }

    let mut sixty_five_ceilings = HashMap::new();
    for index in 0..65 {
        sixty_five_ceilings.insert(format!("EdgeType/SYNTHETIC_{index:02}"), 1);
    }
    assert_eq!(
        CardinalityCeilings::new(sixty_five_ceilings.clone(), HashMap::new()).sfs_identity_digest(),
        Err(SfsFuelIdentityError::RowLimit {
            table: "cardinality",
            actual: 65
        })
    );
    sixty_five_ceilings.remove("EdgeType/SYNTHETIC_64");
    assert!(
        CardinalityCeilings::new(sixty_five_ceilings, HashMap::new())
            .sfs_identity_digest()
            .is_ok()
    );
}

#[test]
fn comparison_context_codes_and_duplicate_sites_are_closed() {
    assert_eq!(SfsComparisonContext::InputValidity.code(), "input-validity");
    assert_eq!(
        SfsComparisonContext::EligibilityNoEffect.code(),
        "eligibility-no-effect"
    );
    assert_eq!(
        SfsComparisonContext::ConservationRefusal.code(),
        "conservation-refusal"
    );
    assert_eq!(
        SfsComparisonContext::MaterialRouting.code(),
        "material-routing"
    );
    assert_eq!(SfsComparisonContext::DomainCeiling.code(), "domain-ceiling");
    let rule = fixture_rule("allowed/scoped_mechanic.bsl");
    let first = GovernedComparisonSite::from_rule_path(
        &rule,
        &[0, 11, 1, 1],
        SfsComparisonContext::InputValidity,
    )
    .unwrap();
    let duplicate = GovernedComparisonSite::from_rule_path(
        &rule,
        &[0, 11, 1, 1],
        SfsComparisonContext::DomainCeiling,
    )
    .unwrap();
    assert_eq!(
        SfsAuditPolicy::new(
            "synthetic-source/scoped-mechanic",
            hex_digest(ALLOWED_AST_HEX),
            31,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            vec![first, duplicate],
            EMPTY,
        ),
        Err(SfsProfileError::DuplicatePolicyEntry {
            set: "comparison_clamp_contexts"
        })
    );
}

fn static_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn comparison_rule(count: usize) -> SExpr {
    assert!(count <= 65);
    let mut source = String::from(
        "(rule synthetic-source/comparison-ceiling\n  :role mechanic :evidence designed \
         :material-basis \"test-local complete comparison scan\" :fuel 65536\n  \
         (bindings (binding available :field synthetic-source/quanta))\n  (when (and",
    );
    for index in 0..65 {
        if index == count {
            break;
        }
        source.push_str(" (> available 0)");
    }
    source.push_str("))\n  (effects (update-node self synthetic-source/quanta (set 1))))");
    parsed_rule(&source)
}

fn comparison_sites(rule: &SExpr, count: usize) -> Vec<GovernedComparisonSite> {
    assert!(count <= 65);
    let mut sites = Vec::new();
    for index in 0..65 {
        if index == count {
            break;
        }
        sites.push(
            GovernedComparisonSite::from_rule_path(
                rule,
                &[0, 11, 1, u32::try_from(index + 1).unwrap()],
                SfsComparisonContext::InputValidity,
            )
            .unwrap(),
        );
    }
    sites.sort_by(|left, right| left.site_digest().cmp(right.site_digest()));
    sites
}

fn comparison_limit_policy(rule: &SExpr, sites: Vec<GovernedComparisonSite>) -> SfsAuditPolicy {
    SfsAuditPolicy::new(
        "synthetic-source/comparison-ceiling",
        sha256_of(&canonical_bytes(rule).unwrap()),
        check_rule(
            rule,
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
        )
        .unwrap(),
        ["synthetic-source/quanta"],
        EMPTY,
        EMPTY,
        EMPTY,
        [">"],
        EMPTY,
        sites,
        ["node:synthetic-source/quanta"],
    )
    .unwrap()
}

#[test]
fn comparison_scan_accepts_64_and_reports_the_byte_largest_65th_site() {
    let maximum = comparison_rule(64);
    let maximum_sites = comparison_sites(&maximum, 64);
    assert!(validate_sfs_rule_profile(
        &maximum,
        &fixture_vocabulary(),
        &CardinalityCeilings::default(),
        &IntrinsicCosts::default(),
        &comparison_limit_policy(&maximum, maximum_sites),
    )
    .is_ok());

    let plus_one = comparison_rule(65);
    let mut governed = comparison_sites(&plus_one, 65);
    let omitted = *governed.pop().unwrap().site_digest();
    let policy = comparison_limit_policy(&plus_one, governed);
    assert_eq!(
        validate_sfs_rule_profile(
            &plus_one,
            &fixture_vocabulary(),
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
            &policy,
        ),
        Err(SfsProfileError::MissingComparisonContext {
            site_digest: omitted
        })
    );
    assert_eq!(
        SfsAuditPolicy::new(
            "synthetic-source/comparison-ceiling",
            sha256_of(&canonical_bytes(&plus_one).unwrap()),
            1,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            comparison_sites(&plus_one, 65),
            EMPTY,
        ),
        Err(SfsProfileError::PolicyEntryLimit {
            set: "comparison_clamp_contexts",
            actual: 65
        })
    );
}

fn wide_rule(kind: &str) -> (SExpr, Vec<&'static str>) {
    let mut source = String::from(
        "(rule synthetic-source/complete-scan\n  :role mechanic :evidence designed \
         :material-basis \"test-local complete footprint scan\" :fuel 65536\n",
    );
    let mut rows = Vec::new();
    if kind == "read" {
        source.push_str("  (bindings");
        for index in 0..65 {
            let field = static_string(format!("synthetic-source/read-{index:02}"));
            rows.push(field);
            write!(&mut source, " (binding value-{index:02} :field {field})").unwrap();
        }
        source.push_str(")\n  (effects (update-node self synthetic-source/quanta (set 1))))");
    } else {
        source.push_str("  (bindings)\n  (effects");
        for index in 0..65 {
            let effect = static_string(format!("node:synthetic-source/effect-{index:02}"));
            let field = effect.strip_prefix("node:").unwrap();
            rows.push(effect);
            write!(&mut source, " (update-node self {field} (set 1))").unwrap();
        }
        source.push_str("))");
    }
    (parsed_rule(&source), rows)
}

fn wide_policy(rule: &SExpr, kind: &str, rows: &[&'static str]) -> SfsAuditPolicy {
    let maximum = rows[..64].to_vec();
    let (field_reads, effects): (Vec<&'static str>, Vec<&'static str>) = if kind == "read" {
        (maximum, vec!["node:synthetic-source/quanta"])
    } else {
        (Vec::new(), maximum)
    };
    SfsAuditPolicy::new(
        "synthetic-source/complete-scan",
        sha256_of(&canonical_bytes(rule).unwrap()),
        check_rule(
            rule,
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
        )
        .unwrap(),
        field_reads,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        vec![],
        effects,
    )
    .unwrap()
}

#[test]
fn actual_read_scan_reports_the_65th_extra_before_set_mismatch() {
    let (read_rule, reads) = wide_rule("read");
    assert_eq!(
        validate_sfs_rule_profile(
            &read_rule,
            &fixture_vocabulary(),
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
            &wide_policy(&read_rule, "read", &reads),
        ),
        Err(SfsProfileError::UnexpectedRead {
            entry: "synthetic-source/read-64".to_owned()
        })
    );
}

#[test]
fn actual_effect_scan_reports_the_65th_extra_before_set_mismatch() {
    let (effect_rule, effects) = wide_rule("effect");
    assert_eq!(
        validate_sfs_rule_profile(
            &effect_rule,
            &fixture_vocabulary(),
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
            &wide_policy(&effect_rule, "effect", &effects),
        ),
        Err(SfsProfileError::UnexpectedEffect {
            entry: "node:synthetic-source/effect-64".to_owned()
        })
    );
}

#[test]
fn separated_governed_thresholds_pass_but_a_nested_ladder_refuses() {
    let separated = parsed_rule(
        r#"(rule synthetic-source/separated-thresholds
  :role mechanic :evidence designed :material-basis "test-local flat guard" :fuel 128
  (bindings (binding available :field synthetic-source/quanta))
  (when (and (> available 1) (> available 2)))
  (effects (update-node self synthetic-source/quanta (set 1))))"#,
    );
    let sites = [
        GovernedComparisonSite::from_rule_path(
            &separated,
            &[0, 11, 1, 1],
            SfsComparisonContext::EligibilityNoEffect,
        )
        .unwrap(),
        GovernedComparisonSite::from_rule_path(
            &separated,
            &[0, 11, 1, 2],
            SfsComparisonContext::EligibilityNoEffect,
        )
        .unwrap(),
    ];
    let policy = SfsAuditPolicy::new(
        "synthetic-source/separated-thresholds",
        sha256_of(&canonical_bytes(&separated).unwrap()),
        check_rule(
            &separated,
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
        )
        .unwrap(),
        ["synthetic-source/quanta"],
        EMPTY,
        EMPTY,
        EMPTY,
        [">"],
        EMPTY,
        sites.into(),
        ["node:synthetic-source/quanta"],
    )
    .unwrap();
    assert!(validate_sfs_rule_profile(
        &separated,
        &fixture_vocabulary(),
        &CardinalityCeilings::default(),
        &IntrinsicCosts::default(),
        &policy,
    )
    .is_ok());

    let nested = fixture_rule("forbidden/threshold_ladder.bsl");
    assert_eq!(
        audit_rule_footprint(
            &nested,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &[],
        ),
        Err(SfsProfileError::ForbiddenThresholdLadder)
    );
}

fn nested_comparison(depth: usize) -> SExpr {
    let mut expression = read("(> 1 0)").unwrap().0;
    for index in 0..MAX_AST_WALK_DEPTH {
        if index == depth {
            break;
        }
        expression = SExpr::List(vec![
            SExpr::Atom(Atom::Symbol("and".to_owned())),
            expression,
        ]);
    }
    expression
}

#[test]
fn governed_form_paths_and_policy_sets_pin_maximum_plus_one_and_duplicates() {
    let deep = nested_comparison(MAX_AST_WALK_DEPTH - 1);
    let mut maximum_path = vec![0_u32];
    for _index in 0..MAX_AST_WALK_DEPTH - 1 {
        maximum_path.push(1);
    }
    assert_eq!(maximum_path.len(), MAX_AST_WALK_DEPTH);
    assert!(GovernedComparisonSite::from_rule_path(
        &deep,
        &maximum_path,
        SfsComparisonContext::InputValidity
    )
    .is_ok());
    maximum_path.push(1);
    assert_eq!(maximum_path.len(), MAX_AST_WALK_DEPTH + 1);
    assert!(!matches!(
        GovernedComparisonSite::from_rule_path(
            &deep,
            &maximum_path,
            SfsComparisonContext::InputValidity
        ),
        Err(SfsProfileError::FormPathLimit { .. })
    ));
    maximum_path.push(1);
    assert_eq!(
        GovernedComparisonSite::from_rule_path(
            &deep,
            &maximum_path,
            SfsComparisonContext::InputValidity
        ),
        Err(SfsProfileError::FormPathLimit {
            actual: MAX_AST_WALK_DEPTH + 2
        })
    );
    let rule = fixture_rule("allowed/scoped_mechanic.bsl");
    assert_eq!(
        GovernedComparisonSite::from_rule_path(&rule, &[], SfsComparisonContext::InputValidity),
        Err(SfsProfileError::EmptyFormPath)
    );
    assert_eq!(
        GovernedComparisonSite::from_rule_path(
            &rule,
            &[0, 99],
            SfsComparisonContext::InputValidity
        ),
        Err(SfsProfileError::UnknownFormPath)
    );
    assert_eq!(
        GovernedComparisonSite::from_rule_path(
            &rule,
            &[0, 10],
            SfsComparisonContext::InputValidity
        ),
        Err(SfsProfileError::NotComparisonOrClamp)
    );

    let duplicate = SfsAuditPolicy::new(
        "synthetic-source/scoped-mechanic",
        hex_digest(ALLOWED_AST_HEX),
        31,
        ["same", "same"],
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        vec![],
        EMPTY,
    );
    assert_eq!(
        duplicate,
        Err(SfsProfileError::DuplicatePolicyEntry { set: "field_reads" })
    );
    let over_limit = SfsAuditPolicy::new(
        "synthetic-source/scoped-mechanic",
        hex_digest(ALLOWED_AST_HEX),
        31,
        ["same"; 65],
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        vec![],
        EMPTY,
    );
    assert_eq!(
        over_limit,
        Err(SfsProfileError::PolicyEntryLimit {
            set: "field_reads",
            actual: 65
        })
    );
}

fn allowed_source() -> String {
    let bytes = read_bounded(&fixture_root(), Path::new("allowed/scoped_mechanic.bsl")).unwrap();
    strict_text(&bytes).to_owned()
}

fn parsed_rule(source: &str) -> SExpr {
    let (rule, consumed) = read(source).unwrap();
    assert!(source[consumed..]
        .bytes()
        .all(|byte| byte.is_ascii_whitespace()));
    rule
}

#[test]
fn formatting_only_preserves_identity() {
    let source = allowed_source();
    let formatted = format!("; formatting-only witness\n\n  {source}");
    let original = parsed_rule(&source);
    let rewrite = parsed_rule(&formatted);
    assert_eq!(canonical_bytes(&original), canonical_bytes(&rewrite));
    let policy = allowed_policy(&original);
    let audit = validate_sfs_rule_profile(
        &rewrite,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &IntrinsicCosts::default(),
        &policy,
    )
    .unwrap();
    assert_eq!(audit.footprint(), policy.expected_footprint());
}

#[test]
fn semantic_change_moves_identity() {
    let source = allowed_source();
    let changed_source = source.replacen("(> available quantum)", "(>= available quantum)", 1);
    let original = parsed_rule(&source);
    let changed = parsed_rule(&changed_source);
    assert_ne!(canonical_bytes(&original), canonical_bytes(&changed));
    let original_audit = audit_rule_footprint(
        &original,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &IntrinsicCosts::default(),
        &allowed_sites(&original),
    )
    .unwrap();
    let changed_audit = audit_rule_footprint(
        &changed,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &IntrinsicCosts::default(),
        &allowed_sites(&changed),
    )
    .unwrap();
    assert_ne!(original_audit.footprint(), changed_audit.footprint());
}

#[test]
fn sealed_audit_moves_when_equal_bound_inputs_or_declared_fuel_move() {
    let source = allowed_source();
    let original = parsed_rule(&source);
    let original_audit = audit_rule_footprint(
        &original,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &IntrinsicCosts::default(),
        &allowed_sites(&original),
    )
    .unwrap();
    let changed_fuel = parsed_rule(&source.replacen(":fuel 128", ":fuel 129", 1));
    let fuel_audit = audit_rule_footprint(
        &changed_fuel,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &IntrinsicCosts::default(),
        &allowed_sites(&changed_fuel),
    )
    .unwrap();
    assert_eq!(fuel_audit.footprint().computed_bound(), 31);
    assert_eq!(fuel_audit.declared_fuel(), 129);
    assert_ne!(fuel_audit, original_audit);

    let extra_cardinality = CardinalityCeilings::new(
        HashMap::from([("EdgeType/SYNTHETIC_LINK".to_owned(), 8)]),
        HashMap::from([("HyperedgeType/SYNTHETIC_UNUSED".to_owned(), 4)]),
    );
    let cardinality_audit = audit_rule_footprint(
        &original,
        &fixture_vocabulary(),
        &extra_cardinality,
        &IntrinsicCosts::default(),
        &allowed_sites(&original),
    )
    .unwrap();
    assert_eq!(cardinality_audit.footprint().computed_bound(), 31);
    assert_ne!(
        cardinality_audit.cardinality_input_digest(),
        original_audit.cardinality_input_digest()
    );
    assert_ne!(cardinality_audit, original_audit);

    let unused_cost = IntrinsicCosts::new(HashMap::from([("synthetic-unused".to_owned(), 7)]));
    let intrinsic_audit = audit_rule_footprint(
        &original,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &unused_cost,
        &allowed_sites(&original),
    )
    .unwrap();
    assert_eq!(intrinsic_audit.footprint().computed_bound(), 31);
    assert_ne!(
        intrinsic_audit.intrinsic_cost_input_digest(),
        original_audit.intrinsic_cost_input_digest()
    );
    assert_ne!(intrinsic_audit, original_audit);
}

#[test]
fn static_bound_and_source_mutations_fail_in_declared_precedence() {
    let source = allowed_source();
    let original = parsed_rule(&source);
    let mut wrong_digest = hex_digest(ALLOWED_AST_HEX);
    wrong_digest[0] ^= 1;
    let wrong_source = SfsAuditPolicy::new(
        "synthetic-source/scoped-mechanic",
        wrong_digest,
        31,
        ["synthetic-source/quanta"],
        ["synthetic-link/strength"],
        [
            "synthetic/minimum-link-strength",
            "synthetic/transfer-quantum",
        ],
        ["edges"],
        [">"],
        EMPTY,
        allowed_sites(&original),
        ["node:synthetic-source/quanta"],
    )
    .unwrap();
    assert_eq!(
        validate_sfs_rule_profile(
            &original,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &wrong_source,
        ),
        Err(SfsProfileError::SourceDigestMismatch)
    );

    let underfunded = parsed_rule(&source.replacen(":fuel 128", ":fuel 30", 1));
    assert!(matches!(
        validate_sfs_rule_profile(
            &underfunded,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &allowed_policy(&original),
        ),
        Err(SfsProfileError::Bound(BoundError::BoundExceeded {
            computed_bound: 31,
            declared_budget: 30,
            ..
        }))
    ));
    assert_eq!(
        validate_sfs_rule_profile(
            &original,
            &fixture_vocabulary(),
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
            &allowed_policy(&original),
        ),
        Err(SfsProfileError::Bound(BoundError::MissingCeiling {
            queried_type: "EdgeType/SYNTHETIC_LINK".to_owned()
        }))
    );
}

#[test]
fn changed_ceiling_refuses_equal_source_before_semantic_walk() {
    let rule = fixture_rule("allowed/scoped_mechanic.bsl");
    let changed = CardinalityCeilings::new(
        HashMap::from([("EdgeType/SYNTHETIC_LINK".to_owned(), 9)]),
        HashMap::new(),
    );
    assert_eq!(
        validate_sfs_rule_profile(
            &rule,
            &fixture_vocabulary(),
            &changed,
            &IntrinsicCosts::default(),
            &allowed_policy(&rule),
        ),
        Err(SfsProfileError::ComputedBoundMismatch {
            expected: 31,
            actual: 33,
        })
    );
}

#[test]
fn expected_observable_rows_cannot_authorize_downstream_state() {
    assert!(matches!(
        SfsAuditPolicy::new(
            "synthetic-source/invalid-policy",
            [0; 32],
            1,
            ["sfs/aggregate"],
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            vec![],
            EMPTY,
        ),
        Err(SfsProfileError::ForbiddenObservable { .. })
    ));
    assert!(matches!(
        SfsAuditPolicy::new(
            "synthetic-source/invalid-policy",
            [0; 32],
            1,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            EMPTY,
            vec![],
            ["node:sfs/classification"],
        ),
        Err(SfsProfileError::ForbiddenObservable { .. })
    ));
    assert!(matches!(
        SfsAuditPolicy::new(
            "synthetic-source/invalid-policy",
            [0; 32],
            1,
            EMPTY,
            EMPTY,
            ["sfs/wave-stage"],
            EMPTY,
            EMPTY,
            EMPTY,
            vec![],
            EMPTY,
        ),
        Err(SfsProfileError::ForbiddenObservable { .. })
    ));
    let actual_const = parsed_rule(
        r#"(rule synthetic-source/invalid-constant
  :role mechanic :evidence designed :material-basis "test-local downstream read" :fuel 128
  (bindings (binding prohibited :const sfs/hinterland-class))
  (effects (update-node self synthetic-source/quanta (set prohibited))))"#,
    );
    assert!(matches!(
        audit_rule_footprint(
            &actual_const,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &[],
        ),
        Err(SfsProfileError::ForbiddenObservable { .. })
    ));
}

#[test]
fn role_relabel_moves_identity_and_fails_profile() {
    let source = allowed_source();
    let original = parsed_rule(&source);
    let policy = allowed_policy(&original);
    let intent_source = source.replacen(":role mechanic", ":role intent", 1);
    let relabeled = parsed_rule(&intent_source);
    assert_ne!(canonical_bytes(&original), canonical_bytes(&relabeled));
    assert_eq!(
        validate_sfs_rule_profile(
            &relabeled,
            &fixture_vocabulary(),
            &fixture_cardinality_ceilings(),
            &IntrinsicCosts::default(),
            &policy
        ),
        Err(SfsProfileError::SourceDigestMismatch)
    );
    let restored = parsed_rule(&source);
    assert_eq!(
        sha256_of(&canonical_bytes(&restored).unwrap()),
        hex_digest(ALLOWED_AST_HEX)
    );
    assert!(validate_sfs_rule_profile(
        &restored,
        &fixture_vocabulary(),
        &fixture_cardinality_ceilings(),
        &IntrinsicCosts::default(),
        &policy
    )
    .is_ok());
}
