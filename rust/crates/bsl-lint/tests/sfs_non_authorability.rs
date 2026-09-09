//! Behavioral contracts for the T3 downstream non-authorability sentinel.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const CHECK: &str = "sfs-non-authorability";
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

struct ScratchRoot(PathBuf);

impl ScratchRoot {
    fn new(label: &str) -> Self {
        let serial = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("bsl-sfs-{label}-{}-{serial}", std::process::id()));
        std::fs::create_dir_all(&root).expect("scratch root must be created");
        Self(root)
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("scratch root must be removed");
    }
}

fn write_file(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().expect("fixture file parent"))
        .expect("fixture parent must be created");
    std::fs::write(path, bytes).expect("fixture file must be written");
}

fn write_minimal_workspace(root: &Path, member_names: &[&str]) {
    let members = member_names
        .iter()
        .map(|name| format!("\"crates/{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    write_file(
        &root.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{members}]\nresolver = \"2\"\n").as_bytes(),
    );
    for name in member_names {
        write_file(
            &root.join(format!("crates/{name}/Cargo.toml")),
            format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\n").as_bytes(),
        );
    }
}

fn pad_with_comment(path: &Path, maximum: usize) {
    let mut bytes = std::fs::read(path).expect("fixture file must be readable");
    assert!(bytes.len() < maximum);
    bytes.push(b'#');
    bytes.resize(maximum, b'x');
    write_file(path, &bytes);
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sfs_non_authorability")
}

fn run_root(root: &Path) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg(CHECK)
        .arg(root)
        .output()
        .expect("bsl-lint must run");
    let mut report = String::from_utf8_lossy(&output.stdout).into_owned();
    report.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), report)
}

fn run(fixture: &str) -> (i32, String) {
    run_root(&fixtures_dir().join(fixture).join("rust"))
}

#[test]
fn the_clean_downstream_shape_passes() {
    let (code, stdout) = run("clean");
    assert_eq!(code, 0, "stdout was:\n{stdout}");
}

#[test]
fn every_manifest_spelling_resolves_the_forbidden_package_edge() {
    let cases = [
        ("reversed-direct", "babylon-tick -> babylon-evidence"),
        ("reversed-alias", "babylon-tick -> babylon-evidence"),
        ("reversed-target", "babylon-tick -> babylon-evidence"),
        ("reversed-workspace", "babylon-tick -> babylon-evidence"),
        (
            "reversed-two-hop",
            "babylon-tick -> local-helper -> babylon-evidence",
        ),
        ("disallowed-direct", "babylon-evidence -> babylon-tick"),
        ("disallowed-alias", "babylon-evidence -> babylon-tick"),
    ];
    for (fixture, expected_edge) in cases {
        let (code, stdout) = run(fixture);
        assert_eq!(code, 1, "fixture {fixture}; stdout was:\n{stdout}");
        assert!(
            stdout.contains(expected_edge),
            "fixture {fixture}; stdout was:\n{stdout}"
        );
    }
}

#[test]
fn every_reserved_token_and_language_surface_fails() {
    let cases = [
        ("reserved-rust", "aggregate.rs", "sfs/aggregate"),
        ("reserved-rust", "classification.rs", "sfs/classification"),
        ("reserved-rust", "wave_stage.rs", "sfs/wave-stage"),
        (
            "reserved-rust",
            "hinterland_class.rs",
            "sfs/hinterland-class",
        ),
        (
            "reserved-rust",
            "political_subjectivity.rs",
            "sfs/political-subjectivity",
        ),
        ("reserved-rust", "aggregate_type.rs", "SfsAggregate"),
        (
            "reserved-rust",
            "classification_type.rs",
            "SfsClassification",
        ),
        ("reserved-rust", "wave_stage_type.rs", "SfsWaveStage"),
        (
            "reserved-rust",
            "hinterland_class_type.rs",
            "SfsHinterlandClass",
        ),
        (
            "reserved-rust",
            "political_subjectivity_type.rs",
            "SfsPoliticalSubjectivity",
        ),
        ("reserved-bsl", "aggregate.bsl", "sfs/aggregate"),
        ("reserved-bsl", "classification.bsl", "sfs/classification"),
        ("reserved-bsl", "wave_stage.bsl", "sfs/wave-stage"),
        (
            "reserved-bsl",
            "hinterland_class.bsl",
            "sfs/hinterland-class",
        ),
        (
            "reserved-bsl",
            "political_subjectivity.bsl",
            "sfs/political-subjectivity",
        ),
        ("reserved-scenario", "aggregate.bscn", "sfs/aggregate"),
        (
            "reserved-scenario",
            "classification.bscn",
            "sfs/classification",
        ),
        ("reserved-scenario", "wave_stage.bscn", "sfs/wave-stage"),
        (
            "reserved-scenario",
            "hinterland_class.bscn",
            "sfs/hinterland-class",
        ),
        (
            "reserved-scenario",
            "political_subjectivity.bscn",
            "sfs/political-subjectivity",
        ),
        ("reserved-two-hop-helper", "aggregate.rs", "sfs/aggregate"),
    ];
    for (fixture, file, token) in cases {
        let (code, stdout) = run(fixture);
        assert_eq!(code, 1, "fixture {fixture}; stdout was:\n{stdout}");
        assert!(
            stdout.contains(file),
            "missing {file}; stdout was:\n{stdout}"
        );
        assert!(
            stdout.contains(token),
            "missing {token}; stdout was:\n{stdout}"
        );
    }
}

#[test]
fn the_real_workspace_has_no_evidence_feedback_edge() {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg(CHECK)
        .output()
        .expect("bsl-lint must run");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn only_zero_or_one_workspace_root_is_accepted() {
    let root = fixtures_dir().join("clean/rust");
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg(CHECK)
        .arg(&root)
        .arg(&root)
        .output()
        .expect("bsl-lint must run");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("accepts zero or one root"),
        "stderr was:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn absent_or_missing_evidence_member_is_a_fail_finding() {
    let scratch = ScratchRoot::new("evidence-absence");
    write_minimal_workspace(&scratch.0, &["babylon-tick"]);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1);
    assert!(report.contains("evidence crate is absent"), "{report}");

    write_file(
        &scratch.0.join("Cargo.toml"),
        b"[workspace]\nmembers = [\"crates/babylon-tick\", \"crates/babylon-evidence\"]\nresolver = \"2\"\n",
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1);
    assert!(
        report.contains("evidence crate manifest is missing"),
        "{report}"
    );
}

#[test]
fn workspace_lock_pins_one_unicode_normalizer_for_bsl_and_evidence() {
    let lock_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let lock_bytes = std::fs::read(&lock_path).expect("workspace lock must be readable");
    let lock: toml::Value = toml::from_slice(&lock_bytes).expect("workspace lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock package array");
    let normalizers: Vec<_> = packages
        .iter()
        .filter(|row| {
            row.get("name").and_then(toml::Value::as_str) == Some("unicode-normalization")
        })
        .collect();
    assert_eq!(normalizers.len(), 1);
    assert_eq!(
        normalizers[0].get("version").and_then(toml::Value::as_str),
        Some("0.1.25")
    );
    for crate_name in ["babylon-bsl", "babylon-evidence"] {
        let package = packages
            .iter()
            .find(|row| row.get("name").and_then(toml::Value::as_str) == Some(crate_name))
            .unwrap_or_else(|| panic!("missing {crate_name} lock row"));
        let dependencies = package
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .expect("package dependency array");
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency.as_str() == Some("unicode-normalization")),
            "{crate_name} must resolve the shared normalizer"
        );
    }
}

#[test]
fn exact_manifest_byte_max_passes_and_plus_one_refuses_before_parse() {
    let scratch = ScratchRoot::new("manifest-bytes");
    write_minimal_workspace(&scratch.0, &["babylon-evidence"]);
    let manifest_path = scratch.0.join("Cargo.toml");
    let mut manifest = std::fs::read(&manifest_path).expect("root manifest");
    manifest.push(b'#');
    manifest.resize(262_144, b'x');
    write_file(&manifest_path, &manifest);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "exact max must pass:\n{report}");

    manifest.push(b'x');
    write_file(&manifest_path, &manifest);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2, "max plus one must be an infrastructure refusal");
    assert!(
        report.contains("262145") && report.contains("262144"),
        "{report}"
    );
}

#[test]
fn exact_aggregate_manifest_byte_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("aggregate-manifest-bytes");
    let names: Vec<String> = (0..15).map(|index| format!("helper-{index:02}")).collect();
    let mut borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    borrowed.push("babylon-evidence");
    write_minimal_workspace(&scratch.0, &borrowed);
    let root_bytes = std::fs::metadata(scratch.0.join("Cargo.toml"))
        .expect("root metadata")
        .len() as usize;
    for name in &names {
        pad_with_comment(
            &scratch.0.join(format!("crates/{name}/Cargo.toml")),
            262_144,
        );
    }
    let evidence_path = scratch.0.join("crates/babylon-evidence/Cargo.toml");
    let final_size = 4_194_304 - root_bytes - (15 * 262_144);
    pad_with_comment(&evidence_path, final_size);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "aggregate exact max must pass:\n{report}");

    let mut evidence = std::fs::read(&evidence_path).expect("evidence manifest");
    evidence.push(b'x');
    write_file(&evidence_path, &evidence);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("4194305") && report.contains("4194304"),
        "{report}"
    );
}

#[test]
fn toml_structural_token_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("toml-tokens");
    write_minimal_workspace(&scratch.0, &["babylon-evidence"]);
    let root_path = scratch.0.join("Cargo.toml");
    let values = std::iter::repeat_n("0", 65_528)
        .collect::<Vec<_>>()
        .join(",");
    let base = format!(
        "[workspace]\nmembers = [\"crates/babylon-evidence\"]\nresolver = \"2\"\nvalues = [{values}]\n"
    );
    write_file(&root_path, base.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "65,536 structural tokens must pass:\n{report}");

    let overflow = base.replace("]\n", ",0]\n");
    write_file(&root_path, overflow.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("structural token 65537"), "{report}");
}

#[test]
fn toml_path_and_inline_depth_preflights_refuse_exact_plus_one() {
    let scratch = ScratchRoot::new("toml-depths");
    write_minimal_workspace(&scratch.0, &["babylon-evidence"]);
    let root_path = scratch.0.join("Cargo.toml");
    let header = "[workspace]\nmembers = [\"crates/babylon-evidence\"]\nresolver = \"2\"\n";
    let path64 = std::iter::repeat_n("a", 64).collect::<Vec<_>>().join(".");
    write_file(&root_path, format!("{header}{path64} = 1\n").as_bytes());
    let (_, report) = run_root(&scratch.0);
    assert!(!report.contains("path component 65"), "{report}");
    let path65 = format!("{path64}.a");
    write_file(&root_path, format!("{header}{path65} = 1\n").as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("path component 65"), "{report}");

    write_file(
        &root_path,
        format!("{header}value = {}0{}\n", "[".repeat(33), "]".repeat(33)).as_bytes(),
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("inline depth 33"), "{report}");
}

#[test]
fn exact_source_byte_max_passes_and_plus_one_refuses_before_rust_parse() {
    let scratch = ScratchRoot::new("source-bytes");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    for (extension, prefix) in [("rs", b"//".as_slice()), ("bsl", b";"), ("bscn", b";")] {
        let source_path = scratch
            .0
            .join(format!("crates/babylon-tick/src/source.{extension}"));
        let mut source = vec![b' '; 262_144];
        source[..prefix.len()].copy_from_slice(prefix);
        write_file(&source_path, &source);
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 0, "{extension} exact max must pass:\n{report}");

        source.push(b' ');
        write_file(&source_path, &source);
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 2, "{extension} max plus one must refuse");
        assert!(
            report.contains("262145") && report.contains("262144"),
            "{report}"
        );
        std::fs::remove_file(source_path).expect("source fixture cleanup");
    }
}

#[test]
fn manifest_count_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("manifest-count");
    let names: Vec<String> = (0..30).map(|index| format!("helper-{index:02}")).collect();
    let mut borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    borrowed.push("babylon-evidence");
    write_minimal_workspace(&scratch.0, &borrowed);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(
        code, 0,
        "31 package manifests plus root must pass:\n{report}"
    );

    write_file(
        &scratch.0.join("crates/helper-overflow/Cargo.toml"),
        b"[package]\nname = \"helper-overflow\"\nversion = \"0.0.0\"\n",
    );
    borrowed.push("helper-overflow");
    write_minimal_workspace(&scratch.0, &borrowed);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("32") && report.contains("31"), "{report}");
}

#[test]
fn dependency_count_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("dependency-count");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let tick_manifest = scratch.0.join("crates/babylon-tick/Cargo.toml");
    let mut manifest =
        String::from("[package]\nname = \"babylon-tick\"\nversion = \"0.0.0\"\n\n[dependencies]\n");
    for index in 0..256 {
        manifest.push_str(&format!("external-{index:03} = \"1\"\n"));
    }
    write_file(&tick_manifest, manifest.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "256 dependencies must pass:\n{report}");

    manifest.push_str("external-overflow = \"1\"\n");
    write_file(&tick_manifest, manifest.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("dependency entry count exceeds 256"),
        "{report}"
    );
}

#[test]
fn resolved_local_edge_max_passes_and_shared_plus_one_refuses() {
    let scratch = ScratchRoot::new("edge-count");
    let mut names = vec!["babylon-tick".to_owned(), "babylon-evidence".to_owned()];
    for index in 0..29 {
        names.push(format!("helper-{index:02}"));
    }
    let borrowed = names.iter().map(String::as_str).collect::<Vec<_>>();
    write_minimal_workspace(&scratch.0, &borrowed);
    let mut edge_count = 0_usize;
    for source_index in 0..30 {
        let source = if source_index == 0 {
            "babylon-tick"
        } else {
            &names[source_index + 1]
        };
        let requested = if source_index < 28 {
            9
        } else if source_index == 28 {
            4
        } else {
            0
        };
        let mut manifest =
            format!("[package]\nname = \"{source}\"\nversion = \"0.0.0\"\n\n[dependencies]\n");
        for target_offset in 0..9 {
            if target_offset >= requested {
                break;
            }
            let target_index = (source_index + target_offset + 1) % 30;
            let target = if target_index == 0 {
                "babylon-tick"
            } else {
                &names[target_index + 1]
            };
            manifest.push_str(&format!(
                "edge-{target_offset} = {{ package = \"{target}\", path = \"../{target}\" }}\n"
            ));
            edge_count = edge_count.saturating_add(1);
        }
        write_file(
            &scratch.0.join(format!("crates/{source}/Cargo.toml")),
            manifest.as_bytes(),
        );
    }
    assert_eq!(edge_count, 256);
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "256 resolved local edges must pass:\n{report}");

    let overflow_path = scratch.0.join("crates/helper-28/Cargo.toml");
    let mut overflow = std::fs::read_to_string(&overflow_path).expect("overflow manifest");
    overflow.push_str("overflow = { package = \"babylon-tick\", path = \"../babylon-tick\" }\n");
    write_file(&overflow_path, overflow.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("dependency entry count exceeds 256"),
        "{report}"
    );
}

#[test]
fn dev_and_build_dependency_tables_are_closed_too() {
    let scratch = ScratchRoot::new("dependency-sections");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let tick_manifest = scratch.0.join("crates/babylon-tick/Cargo.toml");
    for section in ["dev-dependencies", "build-dependencies"] {
        let manifest = format!(
            "[package]\nname = \"babylon-tick\"\nversion = \"0.0.0\"\n\n[{section}]\nevidence = {{ package = \"babylon-evidence\", path = \"../babylon-evidence\" }}\n"
        );
        write_file(&tick_manifest, manifest.as_bytes());
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 1, "{section} must be closed:\n{report}");
        assert!(
            report.contains("babylon-tick -> babylon-evidence"),
            "{report}"
        );
    }
}

#[test]
fn byte_least_complete_violating_path_is_reported() {
    let scratch = ScratchRoot::new("byte-least-path");
    write_minimal_workspace(
        &scratch.0,
        &["babylon-tick", "a-helper", "babylon-evidence"],
    );
    write_file(
        &scratch.0.join("crates/babylon-tick/Cargo.toml"),
        b"[package]\nname = \"babylon-tick\"\nversion = \"0.0.0\"\n\n[dependencies]\nevidence = { package = \"babylon-evidence\", path = \"../babylon-evidence\" }\na-helper = { path = \"../a-helper\" }\n",
    );
    write_file(
        &scratch.0.join("crates/a-helper/Cargo.toml"),
        b"[package]\nname = \"a-helper\"\nversion = \"0.0.0\"\n\n[dependencies]\nevidence = { package = \"babylon-evidence\", path = \"../babylon-evidence\" }\n",
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1);
    assert!(
        report.contains("babylon-tick -> a-helper -> babylon-evidence"),
        "{report}"
    );
}

#[test]
fn local_dependency_paths_cannot_escape_or_omit_the_target_manifest() {
    let scratch = ScratchRoot::new("dependency-paths");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let tick_manifest = scratch.0.join("crates/babylon-tick/Cargo.toml");
    write_file(
        &tick_manifest,
        b"[package]\nname = \"babylon-tick\"\nversion = \"0.0.0\"\n\n[dependencies]\nbad = { path = \"../../../outside\" }\n",
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("escapes workspace root"), "{report}");

    write_file(
        &tick_manifest,
        b"[package]\nname = \"babylon-tick\"\nversion = \"0.0.0\"\n\n[dependencies]\nbad = { path = \"../missing\" }\n",
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("lacks") && report.contains("Cargo.toml"),
        "{report}"
    );
}

#[test]
fn rust_group_depth_max_passes_and_plus_one_refuses_before_syn() {
    let scratch = ScratchRoot::new("rust-depth");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let source_path = scratch.0.join("crates/babylon-tick/src/lib.rs");
    let source = format!(
        "const VALUE: usize = {}1{};\n",
        "(".repeat(64),
        ")".repeat(64)
    );
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "depth 64 must pass:\n{report}");

    let source = format!(
        "const VALUE: usize = {}1{};\n",
        "(".repeat(65),
        ")".repeat(65)
    );
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("depth 65 exceeds 64"), "{report}");
}

#[test]
fn semantic_sexpr_depth_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("sexpr-depth");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    for extension in ["bsl", "bscn"] {
        let source_path = scratch
            .0
            .join(format!("crates/babylon-bsl/src/depth.{extension}"));
        let source = format!("{}x{}\n", "(".repeat(256), ")".repeat(256));
        write_file(&source_path, source.as_bytes());
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 0, "{extension} depth 256 must pass:\n{report}");

        let source = format!("{}x{}\n", "(".repeat(257), ")".repeat(257));
        write_file(&source_path, source.as_bytes());
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 2);
        assert!(report.contains("depth 257 exceeds 256"), "{report}");
        std::fs::remove_file(source_path).expect("depth fixture cleanup");
    }
}

#[test]
fn field_of_inspects_the_production_qname_operand() {
    let scratch = ScratchRoot::new("field-of-qname");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    write_file(
        &scratch.0.join("crates/babylon-bsl/src/query.bsl"),
        b"(defrule query\n  (let ((value (field-of it sfs/aggregate)))\n    value))\n",
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1, "production-shaped field-of must fail:\n{report}");
    assert!(report.contains("sfs/aggregate"), "{report}");
}

#[test]
fn rust_token_and_stack_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("rust-token-count");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let source_path = scratch.0.join("crates/babylon-tick/src/lib.rs");
    let mut source = String::new();
    for _index in 0..21_844 {
        source.push_str("struct A;\n");
    }
    source.push_str("fn a() {}\n");
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "65,536 token trees must pass:\n{report}");

    source = source.replace("fn a() {}", "fn a() {;}");
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("token tree 65537"), "{report}");
}

#[test]
fn rust_explicit_stack_max_passes_and_plus_one_refuses_before_walk() {
    let scratch = ScratchRoot::new("rust-stack-count");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let source_path = scratch.0.join("crates/babylon-tick/src/lib.rs");
    let mut source = String::new();
    for _index in 0..21_844 {
        source.push_str("struct A;\n");
    }
    source.push_str("fn a() {}\n");
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "65,536 initial stack entries must pass:\n{report}");

    source.push(';');
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("Rust token stack exceeds 65536"),
        "{report}"
    );
}

#[test]
fn sexpr_stack_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("sexpr-stack");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    for extension in ["bsl", "bscn"] {
        let source_path = scratch
            .0
            .join(format!("crates/babylon-bsl/src/stack.{extension}"));
        let mut source = String::from("(");
        for _index in 0..65_536 {
            source.push_str("x ");
        }
        source.push_str(")\n");
        write_file(&source_path, source.as_bytes());
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 0, "{extension} stack max must pass:\n{report}");

        source.insert_str(source.len() - 2, "x ");
        write_file(&source_path, source.as_bytes());
        let (code, report) = run_root(&scratch.0);
        assert_eq!(code, 2);
        assert!(report.contains("stack exceeds 65536"), "{report}");
        std::fs::remove_file(source_path).expect("stack fixture cleanup");
    }
}

#[test]
fn reader_nesting_plus_one_is_a_typed_refusal() {
    let scratch = ScratchRoot::new("reader-depth");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    let source_path = scratch.0.join("crates/babylon-bsl/src/reader-depth.bsl");
    let source = format!("{}x{}\n", "(".repeat(513), ")".repeat(513));
    write_file(&source_path, source.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("reader refusal"), "{report}");
}

#[test]
fn reader_nesting_exact_max_passes_before_semantic_preflight() {
    let source = format!("{}x{}\n", "(".repeat(512), ")".repeat(512));
    assert!(
        babylon_bsl::read_all(source.as_bytes()).is_ok(),
        "reader depth 512 must remain admitted"
    );
    let overflow = format!("{}x{}\n", "(".repeat(513), ")".repeat(513));
    assert!(
        babylon_bsl::read_all(overflow.as_bytes()).is_err(),
        "reader depth 513 must remain refused"
    );
}

#[test]
fn directory_and_source_path_maxima_refuse_only_plus_one() {
    let scratch = ScratchRoot::new("filesystem-bounds");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let tick_root = scratch.0.join("crates/babylon-tick");
    for index in 0..511 {
        std::fs::create_dir(tick_root.join(format!("d{index:03}"))).expect("fixture directory");
    }
    let (code, report) = run_root(&scratch.0);
    assert_eq!(
        code, 0,
        "512 directories including root must pass:\n{report}"
    );
    std::fs::create_dir(tick_root.join("d-overflow")).expect("overflow directory");
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("directory 513 exceeds 512"), "{report}");

    for index in 0..512 {
        std::fs::remove_dir(tick_root.join(format!("d{index:03}"))).ok();
    }
    std::fs::remove_dir(tick_root.join("d-overflow")).expect("overflow cleanup");
    let source_dir = tick_root.join("src");
    for index in 0..4_096 {
        write_file(&source_dir.join(format!("f{index:04}.rs")), b"");
    }
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "4,096 production paths must pass:\n{report}");
    write_file(&source_dir.join("overflow.rs"), b"");
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("source path 4097 exceeds 4096"), "{report}");
}

#[test]
fn directory_entry_max_passes_and_plus_one_refuses() {
    let scratch = ScratchRoot::new("directory-entries");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let tick_root = scratch.0.join("crates/babylon-tick");
    for index in 0..16_383 {
        write_file(&tick_root.join(format!("entry-{index:05}.txt")), b"");
    }
    let (code, report) = run_root(&scratch.0);
    assert_eq!(
        code, 0,
        "16,384 entries including Cargo.toml pass:\n{report}"
    );

    write_file(&tick_root.join("entry-overflow.txt"), b"");
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("directory entry 16385 exceeds 16384"),
        "{report}"
    );
}

#[test]
fn directory_depth_seventeen_is_refused() {
    let scratch = ScratchRoot::new("directory-depth");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let mut deepest = scratch.0.join("crates/babylon-tick");
    for _index in 0..16 {
        deepest.push("d");
    }
    std::fs::create_dir_all(&deepest).expect("depth 16 directory");
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "directory depth 16 must pass:\n{report}");
    std::fs::create_dir(deepest.join("overflow")).expect("depth 17 directory");
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("depth 17 exceeds 16"), "{report}");
}

#[test]
fn only_the_exact_digest_pinned_registry_declaration_is_exempt() {
    let scratch = ScratchRoot::new("registry");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    let registry_path = scratch.0.join("crates/babylon-bsl/src/sfs_profile.rs");
    let rows = [
        "SfsAggregate",
        "SfsClassification",
        "SfsHinterlandClass",
        "SfsPoliticalSubjectivity",
        "SfsWaveStage",
        "sfs/aggregate",
        "sfs/classification",
        "sfs/hinterland-class",
        "sfs/political-subjectivity",
        "sfs/wave-stage",
    ];
    let literals = rows.map(|row| format!("\"{row}\"")).join(", ");
    let valid =
        format!("pub const FORBIDDEN_AUTHORITATIVE_IDENTIFIERS_V1: [&str; 10] = [{literals}];\n");
    write_file(&registry_path, valid.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 0, "exact registry must be exempt:\n{report}");

    let wrong_order = valid.replace(
        "SfsAggregate\", \"SfsClassification",
        "SfsClassification\", \"SfsAggregate",
    );
    write_file(&registry_path, wrong_order.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1);
    assert!(
        report.contains("65e7a808f3b078da9c91e424f8fc6ca0a1309eac9882a707c8033aaf0620fb4b"),
        "{report}"
    );

    write_file(&registry_path, format!("{valid}{valid}").as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1);
    assert!(
        report.contains("2 declarations; maximum is one"),
        "{report}"
    );
}

#[test]
fn registry_exemption_covers_only_declaration_token_spans() {
    let scratch = ScratchRoot::new("registry-span");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    let registry_path = scratch.0.join("crates/babylon-bsl/src/sfs_profile.rs");
    let valid = concat!(
        "pub const FORBIDDEN_AUTHORITATIVE_IDENTIFIERS_V1: [&str; 10] = [",
        "\"SfsAggregate\", \"SfsClassification\", \"SfsHinterlandClass\", ",
        "\"SfsPoliticalSubjectivity\", \"SfsWaveStage\", \"sfs/aggregate\", ",
        "\"sfs/classification\", \"sfs/hinterland-class\", ",
        "\"sfs/political-subjectivity\", \"sfs/wave-stage\"",
        "];\n",
    );
    write_file(
        &registry_path,
        format!("{valid}pub fn leak() -> &'static str {{ \"sfs/aggregate\" }}\n").as_bytes(),
    );
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1, "an extra reserved use must fail:\n{report}");
    assert!(report.contains("sfs/aggregate"), "{report}");
}

#[test]
fn registry_digest_is_independent_of_the_reserved_token_table() {
    let scratch = ScratchRoot::new("registry-digest");
    write_minimal_workspace(&scratch.0, &["babylon-bsl", "babylon-evidence"]);
    let registry_path = scratch.0.join("crates/babylon-bsl/src/sfs_profile.rs");
    let mutated = concat!(
        "pub const FORBIDDEN_AUTHORITATIVE_IDENTIFIERS_V1: [&str; 10] = [",
        "\"SfsAggregate\", \"SfsClassification\", \"SfsHinterlandClass\", ",
        "\"SfsPoliticalSubjectivity\", \"SfsWaveStage\", \"sfs/aggregate\", ",
        "\"sfs/classification\", \"sfs/hinterland-class\", ",
        "\"sfs/political-subjectivity\", \"sfs/wave-stagf\"",
        "];\n",
    );
    write_file(&registry_path, mutated.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 1);
    assert!(
        report.contains("65e7a808f3b078da9c91e424f8fc6ca0a1309eac9882a707c8033aaf0620fb4b"),
        "{report}"
    );
}

#[test]
fn escaped_triple_quote_keeps_toml_structural_bytes_inside_multiline_string() {
    let scratch = ScratchRoot::new("toml-multiline-escape");
    write_minimal_workspace(&scratch.0, &["babylon-evidence"]);
    let root_path = scratch.0.join("Cargo.toml");
    let manifest = concat!(
        "[workspace]\n",
        "members = [\"crates/babylon-evidence\"]\n",
        "resolver = \"2\"\n",
        "note = \"\"\"escaped \\\"\"\" [ { , . = are still text\n",
        "and this is still text\"\"\"\n",
    );
    write_file(&root_path, manifest.as_bytes());
    let (code, report) = run_root(&scratch.0);
    assert_eq!(
        code, 0,
        "escaped triple quote must not close early:\n{report}"
    );
}

#[test]
fn multiline_strings_close_on_the_final_three_quotes_of_valid_quote_runs() {
    let scratch = ScratchRoot::new("toml-multiline-closing-run");
    write_minimal_workspace(&scratch.0, &["babylon-evidence"]);
    let root_path = scratch.0.join("Cargo.toml");
    let header = concat!(
        "[workspace]\n",
        "members = [\"crates/babylon-evidence\"]\n",
        "resolver = \"2\"\n",
    );
    for (delimiter, quote) in [("\"\"\"", '"'), ("'''", '\'')] {
        for closing_count in [4_usize, 5] {
            let closing = quote.to_string().repeat(closing_count);
            let manifest =
                format!("{header}note = {delimiter}trailing quotes{closing}\nafter = 1\n");
            write_file(&root_path, manifest.as_bytes());
            let (code, report) = run_root(&scratch.0);
            assert_eq!(
                code, 0,
                "{delimiter} with {closing_count} closing quotes must pass:\n{report}"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn intermediate_package_symlinks_and_canonical_escape_are_refused() {
    use std::os::unix::fs::symlink;

    let workspace = ScratchRoot::new("intermediate-symlink");
    let outside = ScratchRoot::new("outside-package");
    write_file(
        &outside.0.join("babylon-evidence/Cargo.toml"),
        b"[package]\nname = \"babylon-evidence\"\nversion = \"0.0.0\"\n",
    );
    write_file(
        &workspace.0.join("Cargo.toml"),
        b"[workspace]\nmembers = [\"crates/link/babylon-evidence\"]\nresolver = \"2\"\n",
    );
    std::fs::create_dir_all(workspace.0.join("crates")).expect("workspace crates directory");
    symlink(&outside.0, workspace.0.join("crates/link")).expect("intermediate symlink");
    let (code, report) = run_root(&workspace.0);
    assert_eq!(code, 2);
    assert!(
        report.contains("symlink") || report.contains("canonical package root escapes"),
        "{report}"
    );
}

#[cfg(unix)]
#[test]
fn a_live_source_symlink_is_refused_instead_of_followed() {
    use std::os::unix::fs::symlink;

    let scratch = ScratchRoot::new("source-symlink");
    write_minimal_workspace(&scratch.0, &["babylon-tick", "babylon-evidence"]);
    let source_dir = scratch.0.join("crates/babylon-tick/src");
    std::fs::create_dir_all(&source_dir).expect("source directory");
    symlink("../Cargo.toml", source_dir.join("linked.rs")).expect("fixture symlink");
    let (code, report) = run_root(&scratch.0);
    assert_eq!(code, 2);
    assert!(report.contains("symlinks are forbidden"), "{report}");
}

#[test]
fn every_declared_bound_and_fixed_loop_stays_literal_in_source() {
    let source_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sfs_non_authorability.rs");
    let source = std::fs::read_to_string(source_path).expect("sentinel source");
    for contract in [
        "const MAX_MANIFESTS: usize = 32;",
        "const MAX_MANIFEST_BYTES: usize = 262_144;",
        "const MAX_AGGREGATE_MANIFEST_BYTES: usize = 4_194_304;",
        "const MAX_TOML_TOKENS: usize = 65_536;",
        "const MAX_TOML_PATH_COMPONENTS: usize = 64;",
        "const MAX_TOML_DEPTH: usize = 32;",
        "const MAX_DEPENDENCIES: usize = 256;",
        "const MAX_DIRECTORIES: usize = 512;",
        "const MAX_DIRECTORY_DEPTH: usize = 16;",
        "const MAX_DIRECTORY_ENTRIES: usize = 16_384;",
        "const MAX_SOURCE_PATHS: usize = 4_096;",
        "const MAX_SOURCE_BYTES: usize = 262_144;",
        "const MAX_RUST_TOKENS: usize = 65_536;",
        "const MAX_RUST_DEPTH: usize = 64;",
        "const MAX_RUST_STACK: usize = 65_536;",
        "for index in 0..32",
        "for index in 0..256",
        "for index in 0..4_096",
    ] {
        assert!(
            source.contains(contract),
            "missing source contract: {contract}"
        );
    }
    assert!(!source.contains("syn::visit"));
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source prefix");
    let compact = production.split_whitespace().collect::<Vec<_>>().join(" ");
    for forbidden in [
        ".iter() .any(",
        ".iter() .map(",
        ".iter() .filter(",
        ".into_iter() .collect(",
        ".components() .any(",
        ".components() .rev(",
        ".binary_search_by(",
        ".collect(",
        ".fold(",
        ".for_each(",
        ".position(",
        ".rposition(",
        ".windows(",
        ".zip(",
        ".sort(",
        ".sort_by(",
        ".sort_unstable(",
        ".dedup(",
        ".extend(",
    ] {
        assert!(
            !compact.contains(forbidden),
            "production traversal must use a literal indexed bound: {forbidden}"
        );
    }
    for line in production.lines() {
        let line = line.trim();
        if !line.starts_with("for ") {
            continue;
        }
        assert!(
            line.contains(" in 0..")
                || line.contains(" in 1..")
                || line.contains(" in 3..")
                || line.contains(" in ENGINE_CRATES")
                || line.contains(" in ["),
            "production for-loop lacks a literal/static bound: {line}"
        );
    }
}
