//! Bounded repository sentinel for the downstream T3 evidence boundary.

use crate::finding::{Finding, Severity};
use crate::repo::Repo;
use babylon_bsl::causal_contract::{MAX_AST_WALK_DEPTH, MAX_AST_WALK_NODES, MAX_AST_WALK_STACK};
use babylon_bsl::reader::{Atom, SExpr, MAX_READER_NESTING_DEPTH};
use proc_macro2::{LineColumn, TokenStream, TokenTree};
use std::collections::{BTreeSet, VecDeque};
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

pub const CHECK: &str = "sfs-non-authorability";

const ENGINE_CRATES: [&str; 7] = [
    "babylon-kernel",
    "babylon-graph",
    "babylon-bsl",
    "babylon-practice-contract",
    "babylon-tick",
    "babylon-persistence",
    "babylon-client",
];

const ALLOWED_EVIDENCE_PATH_DEPS: [&str; 3] =
    ["babylon-kernel", "babylon-bsl", "babylon-practice-contract"];

const RESERVED_ENGINE_TOKENS: [&str; 10] = [
    "sfs/aggregate",
    "sfs/classification",
    "sfs/wave-stage",
    "sfs/hinterland-class",
    "sfs/political-subjectivity",
    "SfsAggregate",
    "SfsClassification",
    "SfsWaveStage",
    "SfsHinterlandClass",
    "SfsPoliticalSubjectivity",
];

const REGISTRY_NAME: &str = "FORBIDDEN_AUTHORITATIVE_IDENTIFIERS_V1";
const REGISTRY_PATH: &str = "babylon-bsl/src/sfs_profile.rs";
const REGISTRY_DIGEST: &str = "65e7a808f3b078da9c91e424f8fc6ca0a1309eac9882a707c8033aaf0620fb4b";
const MAX_MANIFESTS: usize = 32;
const MAX_MANIFEST_BYTES: usize = 262_144;
const MAX_AGGREGATE_MANIFEST_BYTES: usize = 4_194_304;
const MAX_TOML_TOKENS: usize = 65_536;
const MAX_TOML_PATH_COMPONENTS: usize = 64;
const MAX_TOML_DEPTH: usize = 32;
const MAX_DEPENDENCIES: usize = 256;
const MAX_DIRECTORIES: usize = 512;
const MAX_DIRECTORY_DEPTH: usize = 16;
const MAX_DIRECTORY_ENTRIES: usize = 16_384;
const MAX_SOURCE_PATHS: usize = 4_096;
const MAX_SOURCE_BYTES: usize = 262_144;
const MAX_RUST_TOKENS: usize = 65_536;
const MAX_RUST_DEPTH: usize = 64;
const MAX_RUST_STACK: usize = 65_536;
const MAX_FINDINGS: usize = (MAX_SOURCE_PATHS * MAX_AST_WALK_NODES) + MAX_DEPENDENCIES + 7;

#[derive(Debug)]
struct ManifestDoc {
    path: PathBuf,
    crate_root: PathBuf,
    package: String,
    value: toml::Value,
}

#[derive(Debug)]
struct Workspace {
    root: PathBuf,
    manifests: Vec<ManifestDoc>,
    edges: Vec<(usize, usize)>,
    missing_evidence_manifest: Option<PathBuf>,
}

#[derive(Debug)]
struct SourceFinding {
    path: PathBuf,
    line: usize,
    token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenSpan {
    start: LineColumn,
    end: LineColumn,
}

#[derive(Debug)]
struct RustToken {
    value: String,
    span: TokenSpan,
}

#[derive(Debug, Default)]
struct RegistryInspection {
    exempt_spans: Vec<TokenSpan>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct SourceWalkBudget {
    directories: usize,
    entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TomlState {
    Plain,
    Comment,
    Basic,
    Literal,
    MultiBasic,
    MultiLiteral,
}

#[derive(Debug)]
struct TomlCounters {
    structural_tokens: usize,
    inline_depth: usize,
    line_dots: usize,
    in_key: bool,
}

impl Default for TomlCounters {
    fn default() -> Self {
        Self {
            structural_tokens: 0,
            inline_depth: 0,
            line_dots: 0,
            in_key: true,
        }
    }
}

/// Check the configured workspace root without granting evidence any engine authority.
///
/// # Errors
/// Returns a bounded, explicit infrastructure error for malformed or over-limit input.
pub fn run(repo: &Repo, roots: &[String]) -> Result<Vec<Finding>, String> {
    if roots.len() > 1 {
        return Err("sfs-non-authorability accepts zero or one root".to_owned());
    }
    let requested = roots
        .first()
        .map_or_else(|| PathBuf::from("rust"), PathBuf::from);
    let root = if requested.is_absolute() {
        requested
    } else {
        repo.root.join(requested)
    };
    let root = root.canonicalize().map_err(|error| {
        format!(
            "{}: cannot canonicalize workspace root: {error}",
            root.display()
        )
    })?;
    let workspace = load_workspace(&root)?;
    let mut findings = dependency_findings(repo, &workspace);
    let mut source = source_findings(repo, &workspace)?;
    for _finding_index in 0..MAX_FINDINGS {
        let Some(finding) = source.pop() else {
            break;
        };
        findings.push(finding);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn sort_findings(findings: &mut [Finding]) {
    for outer in 1..MAX_FINDINGS {
        if outer >= findings.len() {
            break;
        }
        let mut current = outer;
        for _shift in 0..MAX_FINDINGS {
            if current == 0 {
                break;
            }
            let left = &findings[current - 1];
            let right = &findings[current];
            if (&left.file, left.line, &left.evidence) <= (&right.file, right.line, &right.evidence)
            {
                break;
            }
            findings.swap(current - 1, current);
            current -= 1;
        }
    }
}

fn fail(repo: &Repo, path: &Path, line: usize, what: String, evidence: String) -> Finding {
    Finding {
        check: CHECK,
        file: repo.display_path(path),
        line,
        what,
        evidence,
        severity: Severity::Fail,
    }
}

fn load_workspace(root: &Path) -> Result<Workspace, String> {
    let root_manifest_path = root.join("Cargo.toml");
    let mut aggregate_bytes = 0_usize;
    let root_value = read_manifest(&root_manifest_path, &mut aggregate_bytes)?;
    let members = workspace_members(&root_value)?;
    if members.len() >= MAX_MANIFESTS {
        return Err(format!(
            "workspace admits {} local package manifests; maximum is {}",
            members.len(),
            MAX_MANIFESTS - 1
        ));
    }
    let mut manifest_paths = Vec::with_capacity(members.len());
    for index in 0..32 {
        let Some(member) = members.get(index) else {
            break;
        };
        let crate_root = normalize_beneath(root, Path::new(member), root)?;
        manifest_paths.push((crate_root.join("Cargo.toml"), crate_root));
    }
    sort_manifest_paths(&mut manifest_paths);
    let mut manifests = Vec::with_capacity(manifest_paths.len());
    let mut missing_evidence_manifest = None;
    for index in 0..32 {
        let Some((path, crate_root)) = manifest_paths.get(index) else {
            break;
        };
        if !path.is_file()
            && crate_root.file_name().and_then(|name| name.to_str()) == Some("babylon-evidence")
        {
            missing_evidence_manifest = Some(path.clone());
            continue;
        }
        let value = read_manifest(path, &mut aggregate_bytes)?;
        let package = package_name(&value, path)?;
        manifests.push(ManifestDoc {
            path: path.clone(),
            crate_root: crate_root.clone(),
            package,
            value,
        });
    }
    sort_manifests(&mut manifests);
    for index in 1..32 {
        let Some(current) = manifests.get(index) else {
            break;
        };
        let previous = &manifests[index - 1];
        if previous.package == current.package {
            return Err(format!("duplicate local package `{}`", previous.package));
        }
    }
    if missing_evidence_manifest.is_some()
        || package_index_in(&manifests, "babylon-evidence").is_none()
    {
        return Ok(Workspace {
            root: root.to_path_buf(),
            manifests,
            edges: Vec::new(),
            missing_evidence_manifest,
        });
    }
    let edges = resolve_edges(root, &root_value, &manifests)?;
    Ok(Workspace {
        root: root.to_path_buf(),
        manifests,
        edges,
        missing_evidence_manifest: None,
    })
}

fn sort_manifest_paths(paths: &mut [(PathBuf, PathBuf)]) {
    for outer in 1..32 {
        if outer >= paths.len() {
            break;
        }
        let mut current = outer;
        for _shift in 0..32 {
            if current == 0 || paths[current - 1].0.as_os_str() <= paths[current].0.as_os_str() {
                break;
            }
            paths.swap(current - 1, current);
            current -= 1;
        }
    }
}

fn sort_manifests(manifests: &mut [ManifestDoc]) {
    for outer in 1..32 {
        if outer >= manifests.len() {
            break;
        }
        let mut current = outer;
        for _shift in 0..32 {
            if current == 0
                || manifests[current - 1].package.as_bytes()
                    <= manifests[current].package.as_bytes()
            {
                break;
            }
            manifests.swap(current - 1, current);
            current -= 1;
        }
    }
}

fn workspace_members(value: &toml::Value) -> Result<Vec<String>, String> {
    let members = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "root Cargo.toml lacks [workspace].members".to_owned())?;
    if members.len() >= MAX_MANIFESTS {
        return Err(format!(
            "workspace member count {} exceeds {} local packages",
            members.len(),
            MAX_MANIFESTS - 1
        ));
    }
    let mut output = Vec::with_capacity(members.len());
    for index in 0..32 {
        let Some(member) = members.get(index) else {
            break;
        };
        let member = member
            .as_str()
            .ok_or_else(|| format!("workspace member {} is not a string", index + 1))?;
        if member.contains('*') || member.contains('?') || member.contains('[') {
            return Err(format!(
                "workspace member glob `{member}` is outside the closed graph"
            ));
        }
        output.push(member.to_owned());
    }
    Ok(output)
}

fn package_name(value: &toml::Value, path: &Path) -> Result<String, String> {
    let name = value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{}: missing string package.name", path.display()))?;
    Ok(name.to_owned())
}

fn read_manifest(path: &Path, aggregate: &mut usize) -> Result<toml::Value, String> {
    let bytes = read_bounded(path, MAX_MANIFEST_BYTES, "manifest")?;
    *aggregate = aggregate
        .checked_add(bytes.len())
        .ok_or_else(|| "aggregate manifest bytes overflowed usize".to_owned())?;
    if *aggregate > MAX_AGGREGATE_MANIFEST_BYTES {
        return Err(format!(
            "aggregate manifest bytes {} exceed {}",
            *aggregate, MAX_AGGREGATE_MANIFEST_BYTES
        ));
    }
    preflight_toml(path, &bytes)?;
    let value: toml::Value = toml::from_slice(&bytes)
        .map_err(|error| format!("{}: malformed Cargo TOML: {error}", path.display()))?;
    preflight_toml_value(path, &value)?;
    Ok(value)
}

fn read_bounded(path: &Path, maximum: usize, kind: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.len() > maximum as u64 {
        return Err(format!(
            "{}: {kind} byte count {} exceeds {maximum}",
            path.display(),
            metadata.len()
        ));
    }
    let limit = u64::try_from(maximum)
        .map_err(|error| format!("{kind} byte limit conversion failed: {error}"))?
        .checked_add(1)
        .ok_or_else(|| format!("{kind} byte limit overflow"))?;
    let mut bytes = Vec::with_capacity(maximum.min(metadata.len() as usize));
    file.by_ref()
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() > maximum {
        return Err(format!(
            "{}: {kind} byte count {} exceeds {maximum}",
            path.display(),
            bytes.len()
        ));
    }
    Ok(bytes)
}

fn preflight_toml(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut state = TomlState::Plain;
    let mut escaped = false;
    let mut delimiter_tail = 0_usize;
    let mut counters = TomlCounters::default();
    for byte_index in 0..262_144 {
        let Some(&byte) = bytes.get(byte_index) else {
            break;
        };
        if delimiter_tail > 0 {
            delimiter_tail = delimiter_tail.saturating_sub(1);
            continue;
        }
        match state {
            TomlState::Comment => {
                if byte == b'\n' {
                    state = TomlState::Plain;
                    counters.line_dots = 0;
                    counters.in_key = true;
                }
            }
            TomlState::Basic => {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    state = TomlState::Plain;
                }
            }
            TomlState::Literal => {
                if byte == b'\'' {
                    state = TomlState::Plain;
                }
            }
            TomlState::MultiBasic => {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if closes_multiline_quote_run(bytes, byte_index, b'"') {
                    state = TomlState::Plain;
                    delimiter_tail = 2;
                }
            }
            TomlState::MultiLiteral => {
                if closes_multiline_quote_run(bytes, byte_index, b'\'') {
                    state = TomlState::Plain;
                    delimiter_tail = 2;
                }
            }
            TomlState::Plain => {
                state = preflight_plain_toml_byte(path, bytes, byte_index, &mut counters)?;
                if matches!(state, TomlState::MultiBasic | TomlState::MultiLiteral) {
                    delimiter_tail = 2;
                }
            }
        }
    }
    if !matches!(state, TomlState::Plain | TomlState::Comment) {
        return Err(format!(
            "{}: unterminated TOML string state",
            path.display()
        ));
    }
    Ok(())
}

fn closes_multiline_quote_run(bytes: &[u8], index: usize, quote: u8) -> bool {
    for quote_offset in 0..3 {
        if bytes.get(index.saturating_add(quote_offset)) != Some(&quote) {
            return false;
        }
    }
    bytes.get(index.saturating_add(3)) != Some(&quote)
}

fn preflight_plain_toml_byte(
    path: &Path,
    bytes: &[u8],
    index: usize,
    counters: &mut TomlCounters,
) -> Result<TomlState, String> {
    let byte = bytes[index];
    let state = next_toml_state(bytes, index, byte);
    if state != TomlState::Plain {
        return Ok(state);
    }
    if byte == b'\n' {
        counters.line_dots = 0;
        counters.in_key = true;
        return Ok(state);
    }
    if byte == b'.' && counters.in_key {
        counters.line_dots = counters.line_dots.saturating_add(1);
        if counters.line_dots.saturating_add(1) > MAX_TOML_PATH_COMPONENTS {
            return Err(format!(
                "{}: TOML path component 65 exceeds {}",
                path.display(),
                MAX_TOML_PATH_COMPONENTS
            ));
        }
    } else if byte == b'=' {
        counters.line_dots = 0;
        counters.in_key = false;
    }
    if matches!(byte, b'[' | b']' | b'{' | b'}' | b'=' | b',' | b'.') {
        counters.structural_tokens = counters.structural_tokens.saturating_add(1);
        if counters.structural_tokens > MAX_TOML_TOKENS {
            return Err(format!(
                "{}: TOML structural token 65537 exceeds {}",
                path.display(),
                MAX_TOML_TOKENS
            ));
        }
    }
    if matches!(byte, b'[' | b'{') {
        counters.inline_depth = counters.inline_depth.saturating_add(1);
        if counters.inline_depth > MAX_TOML_DEPTH {
            return Err(format!(
                "{}: TOML inline depth 33 exceeds {}",
                path.display(),
                MAX_TOML_DEPTH
            ));
        }
    } else if matches!(byte, b']' | b'}') {
        counters.inline_depth = counters.inline_depth.saturating_sub(1);
    }
    Ok(state)
}

fn next_toml_state(bytes: &[u8], index: usize, byte: u8) -> TomlState {
    let triple = bytes.get(index..index.saturating_add(3));
    match (byte, triple) {
        (b'#', _) => TomlState::Comment,
        (b'"', Some(b"\"\"\"")) => TomlState::MultiBasic,
        (b'\'', Some(b"'''")) => TomlState::MultiLiteral,
        (b'"', _) => TomlState::Basic,
        (b'\'', _) => TomlState::Literal,
        _ => TomlState::Plain,
    }
}

fn preflight_toml_value(path: &Path, root: &toml::Value) -> Result<(), String> {
    let mut stack = vec![(root, 0_usize)];
    for visited in 0..65_536 {
        let Some((value, depth)) = stack.pop() else {
            return Ok(());
        };
        if depth > MAX_TOML_DEPTH {
            return Err(format!(
                "{}: parsed TOML depth {} exceeds {}",
                path.display(),
                depth,
                MAX_TOML_DEPTH
            ));
        }
        let child_count = match value {
            toml::Value::Array(values) => values.len(),
            toml::Value::Table(values) => values.len(),
            _ => 0,
        };
        check_toml_stack_capacity(path, stack.len(), child_count)?;
        for child_offset in 0..65_536 {
            let Some(child_index) = child_count.checked_sub(child_offset + 1) else {
                break;
            };
            let child = match value {
                toml::Value::Array(values) => values.get(child_index),
                toml::Value::Table(values) => toml_table_value_at(values, child_index),
                _ => None,
            };
            let Some(child) = child else {
                return Err(format!(
                    "{}: parsed TOML child index vanished",
                    path.display()
                ));
            };
            stack.push((child, depth.saturating_add(1)));
        }
        if visited == MAX_TOML_TOKENS - 1 && !stack.is_empty() {
            return Err(format!(
                "{}: parsed TOML value 65537 exceeds {}",
                path.display(),
                MAX_TOML_TOKENS
            ));
        }
    }
    if stack.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}: parsed TOML walk did not terminate",
            path.display()
        ))
    }
}

fn check_toml_stack_capacity(path: &Path, current: usize, added: usize) -> Result<(), String> {
    if current.saturating_add(added) > MAX_TOML_TOKENS {
        return Err(format!(
            "{}: parsed TOML stack exceeds {} values",
            path.display(),
            MAX_TOML_TOKENS
        ));
    }
    Ok(())
}

fn toml_table_value_at(
    table: &toml::map::Map<String, toml::Value>,
    selected: usize,
) -> Option<&toml::Value> {
    let mut entries = table.iter();
    for index in 0..65_536 {
        let (_key, value) = entries.next()?;
        if index == selected {
            return Some(value);
        }
    }
    None
}

fn normalize_beneath(base: &Path, relative: &Path, root: &Path) -> Result<PathBuf, String> {
    if relative.is_absolute() {
        return Err(format!(
            "absolute local path `{}` is forbidden",
            relative.display()
        ));
    }
    let mut normalized = base.to_path_buf();
    let mut components = relative.components();
    for _component_index in 0..64 {
        let Some(component) = components.next() else {
            inspect_path_components(root, &normalized)?;
            return Ok(normalized);
        };
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() || !normalized.starts_with(root) {
                    return Err(format!(
                        "local path `{}` escapes workspace root",
                        relative.display()
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "local path `{}` is not relative",
                    relative.display()
                ));
            }
        }
    }
    if components.next().is_some() {
        return Err(format!(
            "local path `{}` exceeds 64 components",
            relative.display()
        ));
    }
    inspect_path_components(root, &normalized)?;
    Ok(normalized)
}

fn inspect_path_components(root: &Path, path: &Path) -> Result<(), String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        format!(
            "{}: canonical package root escapes workspace: {error}",
            path.display()
        )
    })?;
    let mut current = root.to_path_buf();
    let mut components = relative.components();
    for _component_index in 0..64 {
        let Some(component) = components.next() else {
            break;
        };
        current.push(component.as_os_str());
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "{}: package path symlinks are forbidden",
                    current.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("{}: {error}", current.display())),
        }
    }
    if path.exists() {
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "{}: canonical package root escapes workspace root",
                path.display()
            ));
        }
    }
    Ok(())
}

fn resolve_edges(
    root: &Path,
    root_value: &toml::Value,
    manifests: &[ManifestDoc],
) -> Result<Vec<(usize, usize)>, String> {
    let workspace_dependencies = root_value
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table);
    let workspace_dependency_count = workspace_dependencies.map_or(0, toml::map::Map::len);
    if workspace_dependency_count > MAX_DEPENDENCIES {
        return Err(format!(
            "workspace dependency entry count {} exceeds {}",
            workspace_dependency_count, MAX_DEPENDENCIES
        ));
    }
    let mut pending = Vec::new();
    for manifest_index in 0..32 {
        let Some(manifest) = manifests.get(manifest_index) else {
            break;
        };
        collect_manifest_dependencies(manifest_index, &manifest.value, &mut pending)?;
    }
    if pending.len().saturating_add(workspace_dependency_count) > MAX_DEPENDENCIES {
        return Err(format!(
            "resolved dependency entry {} exceeds {}",
            pending.len().saturating_add(workspace_dependency_count),
            MAX_DEPENDENCIES
        ));
    }
    let mut edges = Vec::new();
    for index in 0..256 {
        let Some((from, alias, row)) = pending.get(index) else {
            break;
        };
        let declaring = &manifests[*from];
        let (resolved, base) = resolve_workspace_row(
            alias,
            row,
            workspace_dependencies,
            root,
            declaring.path.parent().unwrap_or(root),
        )?;
        let Some(table) = resolved.as_table() else {
            continue;
        };
        let Some(path_text) = table.get("path").and_then(toml::Value::as_str) else {
            continue;
        };
        let target_root = normalize_beneath(base, Path::new(path_text), root)?;
        let target_manifest = target_root.join("Cargo.toml");
        if !target_manifest.is_file() {
            return Err(format!(
                "{}: local dependency `{alias}` lacks {}",
                declaring.path.display(),
                target_manifest.display()
            ));
        }
        let Some(to) = package_root_index(manifests, &target_root) else {
            return Err(format!(
                "{}: local dependency `{alias}` targets a package outside workspace.members: {}",
                declaring.path.display(),
                target_root.display()
            ));
        };
        let target_name = &manifests[to].package;
        if let Some(declared_name) = table.get("package").and_then(toml::Value::as_str) {
            if declared_name != target_name {
                return Err(format!(
                    "{}: dependency alias `{alias}` declares package `{declared_name}` but path resolves `{target_name}`",
                    declaring.path.display()
                ));
            }
        }
        edges.push((*from, to));
    }
    sort_edges(&mut edges);
    dedup_edges(&mut edges);
    if edges.len() > MAX_DEPENDENCIES {
        return Err(format!(
            "local edge count {} exceeds {}",
            edges.len(),
            MAX_DEPENDENCIES
        ));
    }
    Ok(edges)
}

fn sort_edges(edges: &mut [(usize, usize)]) {
    for outer in 1..256 {
        if outer >= edges.len() {
            break;
        }
        let mut current = outer;
        for _shift in 0..256 {
            if current == 0 || edges[current - 1] <= edges[current] {
                break;
            }
            edges.swap(current - 1, current);
            current -= 1;
        }
    }
}

fn dedup_edges(edges: &mut Vec<(usize, usize)>) {
    let mut admitted = 0_usize;
    for read in 0..256 {
        let Some(edge) = edges.get(read).copied() else {
            break;
        };
        if admitted == 0 || edges[admitted - 1] != edge {
            edges[admitted] = edge;
            admitted = admitted.saturating_add(1);
        }
    }
    edges.truncate(admitted);
}

fn resolve_workspace_row<'a>(
    alias: &str,
    row: &'a toml::Value,
    workspace_dependencies: Option<&'a toml::map::Map<String, toml::Value>>,
    root: &'a Path,
    manifest_parent: &'a Path,
) -> Result<(&'a toml::Value, &'a Path), String> {
    let inherited = row
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    if !inherited {
        return Ok((row, manifest_parent));
    }
    let dependencies = workspace_dependencies
        .ok_or_else(|| format!("dependency `{alias}` inherits missing [workspace.dependencies]"))?;
    let resolved = dependencies
        .get(alias)
        .ok_or_else(|| format!("dependency `{alias}` has no workspace dependency row"))?;
    Ok((resolved, root))
}

fn collect_manifest_dependencies<'a>(
    from: usize,
    value: &'a toml::Value,
    output: &mut Vec<(usize, String, &'a toml::Value)>,
) -> Result<(), String> {
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        collect_dependency_table(from, value.get(table_name), output)?;
    }
    if let Some(targets) = value.get("target").and_then(toml::Value::as_table) {
        if targets.len() > MAX_DEPENDENCIES {
            return Err(format!(
                "target dependency table count {} exceeds {}",
                targets.len(),
                MAX_DEPENDENCIES
            ));
        }
        for target_index in 0..256 {
            let Some((_target, target_value)) = toml_table_entry_at(targets, target_index) else {
                break;
            };
            for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
                collect_dependency_table(from, target_value.get(table_name), output)?;
            }
        }
    }
    Ok(())
}

fn collect_dependency_table<'a>(
    from: usize,
    value: Option<&'a toml::Value>,
    output: &mut Vec<(usize, String, &'a toml::Value)>,
) -> Result<(), String> {
    let Some(table) = value.and_then(toml::Value::as_table) else {
        return Ok(());
    };
    if output.len().saturating_add(table.len()) > MAX_DEPENDENCIES {
        return Err(format!(
            "dependency entry count exceeds {}",
            MAX_DEPENDENCIES
        ));
    }
    for index in 0..256 {
        let Some((alias, row)) = toml_table_entry_at(table, index) else {
            break;
        };
        output.push((from, alias.clone(), row));
    }
    Ok(())
}

fn toml_table_entry_at(
    table: &toml::map::Map<String, toml::Value>,
    selected: usize,
) -> Option<(&String, &toml::Value)> {
    let mut entries = table.iter();
    for index in 0..65_536 {
        let entry = entries.next()?;
        if index == selected {
            return Some(entry);
        }
    }
    None
}

fn package_root_index(manifests: &[ManifestDoc], root: &Path) -> Option<usize> {
    for index in 0..32 {
        let Some(manifest) = manifests.get(index) else {
            break;
        };
        if manifest.crate_root == root {
            return Some(index);
        }
    }
    None
}

fn dependency_findings(repo: &Repo, workspace: &Workspace) -> Vec<Finding> {
    if let Some(path) = &workspace.missing_evidence_manifest {
        return vec![fail(
            repo,
            path,
            1,
            "the downstream evidence crate manifest is missing".to_owned(),
            "workspace member babylon-evidence must contain Cargo.toml".to_owned(),
        )];
    }
    let Some(evidence) = package_index(workspace, "babylon-evidence") else {
        return vec![fail(
            repo,
            &workspace.root.join("Cargo.toml"),
            1,
            "the downstream evidence crate is absent".to_owned(),
            "workspace.members must contain babylon-evidence".to_owned(),
        )];
    };
    let mut findings = Vec::new();
    for engine in ENGINE_CRATES {
        let Some(start) = package_index(workspace, engine) else {
            continue;
        };
        if let Some(path) = byte_least_path(workspace, start, evidence) {
            findings.push(fail(
                repo,
                &workspace.manifests[start].path,
                1,
                "engine package reaches downstream evidence".to_owned(),
                render_package_path(workspace, &path),
            ));
        }
    }
    for edge_index in 0..256 {
        let Some(&(from, to)) = workspace.edges.get(edge_index) else {
            break;
        };
        if from != evidence {
            continue;
        }
        let target = &workspace.manifests[to].package;
        if !is_allowed_evidence_dependency(target) {
            findings.push(fail(
                repo,
                &workspace.manifests[from].path,
                1,
                "evidence depends on a package outside its closed contracts".to_owned(),
                format!("babylon-evidence -> {target}"),
            ));
        }
    }
    findings
}

fn is_allowed_evidence_dependency(package: &str) -> bool {
    for index in 0..3 {
        let Some(allowed) = ALLOWED_EVIDENCE_PATH_DEPS.get(index) else {
            break;
        };
        if *allowed == package {
            return true;
        }
    }
    false
}

fn package_index(workspace: &Workspace, name: &str) -> Option<usize> {
    package_index_in(&workspace.manifests, name)
}

fn package_index_in(manifests: &[ManifestDoc], name: &str) -> Option<usize> {
    for index in 0..32 {
        let Some(manifest) = manifests.get(index) else {
            break;
        };
        if manifest.package == name {
            return Some(index);
        }
    }
    None
}

fn byte_least_path(workspace: &Workspace, start: usize, target: usize) -> Option<Vec<usize>> {
    let mut best: Vec<Option<Vec<usize>>> = vec![None; workspace.manifests.len()];
    best[start] = Some(vec![start]);
    for _expansion in 0..32 {
        let previous = best.clone();
        for edge_index in 0..256 {
            let Some(&(from, to)) = workspace.edges.get(edge_index) else {
                break;
            };
            let Some(prefix) = previous[from].as_ref() else {
                continue;
            };
            if package_path_contains(prefix, to) {
                continue;
            }
            let mut candidate = prefix.clone();
            candidate.push(to);
            let replace = best[to]
                .as_ref()
                .is_none_or(|current| package_path_less(workspace, &candidate, current));
            if replace {
                best[to] = Some(candidate);
            }
        }
    }
    best.get(target).cloned().flatten()
}

fn package_path_contains(path: &[usize], selected: usize) -> bool {
    for index in 0..32 {
        let Some(package) = path.get(index) else {
            break;
        };
        if *package == selected {
            return true;
        }
    }
    false
}

fn package_path_less(workspace: &Workspace, left: &[usize], right: &[usize]) -> bool {
    for index in 0..32 {
        match (left.get(index), right.get(index)) {
            (Some(left_index), Some(right_index)) => {
                let order = workspace.manifests[*left_index]
                    .package
                    .as_bytes()
                    .cmp(workspace.manifests[*right_index].package.as_bytes());
                if order.is_lt() {
                    return true;
                }
                if order.is_gt() {
                    return false;
                }
            }
            (None, Some(_)) => return true,
            (Some(_), None) | (None, None) => return false,
        }
    }
    false
}

fn render_package_path(workspace: &Workspace, path: &[usize]) -> String {
    let mut rendered = String::new();
    for index in 0..32 {
        let Some(package_index) = path.get(index) else {
            break;
        };
        if index > 0 {
            rendered.push_str(" -> ");
        }
        rendered.push_str(&workspace.manifests[*package_index].package);
    }
    rendered
}

fn source_findings(repo: &Repo, workspace: &Workspace) -> Result<Vec<Finding>, String> {
    let reachable = reachable_engine_packages(workspace);
    let mut source_paths = Vec::new();
    let mut walk_budget = SourceWalkBudget::default();
    for index in 0..32 {
        let Some(package_index) = reachable.get(index) else {
            break;
        };
        let manifest = &workspace.manifests[*package_index];
        if manifest.package == "babylon-evidence" {
            continue;
        }
        let mut package_sources = enumerate_sources(&manifest.crate_root, &mut walk_budget)?;
        for _source_index in 0..4_096 {
            let Some(source) = package_sources.pop() else {
                break;
            };
            source_paths.push(source);
        }
        if source_paths.len() > MAX_SOURCE_PATHS {
            return Err(format!(
                "production source path {} exceeds {}",
                source_paths.len(),
                MAX_SOURCE_PATHS
            ));
        }
    }
    sort_source_paths(&mut source_paths);
    dedup_source_paths(&mut source_paths);
    if source_paths.len() > MAX_SOURCE_PATHS {
        return Err(format!(
            "production source path {} exceeds {}",
            source_paths.len(),
            MAX_SOURCE_PATHS
        ));
    }
    let mut findings = Vec::new();
    let mut registry_declarations = 0_usize;
    for index in 0..4_096 {
        let Some(path) = source_paths.get(index) else {
            break;
        };
        let bytes = read_bounded(path, MAX_SOURCE_BYTES, "source")?;
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let discovered = match extension {
            "rs" => scan_rust_source(path, &bytes, &mut registry_declarations)?,
            "bsl" | "bscn" => scan_sexpr_source(path, &bytes, extension)?,
            _ => Vec::new(),
        };
        for finding_index in 0..65_536 {
            let Some(source_finding) = discovered.get(finding_index) else {
                break;
            };
            findings.push(fail(
                repo,
                &source_finding.path,
                source_finding.line,
                "reserved T3 evidence identifier is engine-authored".to_owned(),
                source_finding.token.clone(),
            ));
        }
    }
    if registry_declarations > 1 {
        findings.push(fail(
            repo,
            &workspace.root.join("Cargo.toml"),
            1,
            "reserved deny registry has multiple declarations".to_owned(),
            format!("{REGISTRY_NAME} has {registry_declarations} declarations; maximum is one"),
        ));
    }
    Ok(findings)
}

fn sort_source_paths(paths: &mut [PathBuf]) {
    for outer in 1..4_096 {
        if outer >= paths.len() {
            break;
        }
        let mut current = outer;
        for _shift in 0..4_096 {
            if current == 0 || paths[current - 1].as_os_str() <= paths[current].as_os_str() {
                break;
            }
            paths.swap(current - 1, current);
            current -= 1;
        }
    }
}

fn dedup_source_paths(paths: &mut Vec<PathBuf>) {
    let mut admitted = 0_usize;
    for read in 0..4_096 {
        let Some(path) = paths.get(read).cloned() else {
            break;
        };
        if admitted == 0 || paths[admitted - 1] != path {
            paths[admitted] = path;
            admitted = admitted.saturating_add(1);
        }
    }
    paths.truncate(admitted);
}

fn reachable_engine_packages(workspace: &Workspace) -> Vec<usize> {
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();
    for engine in ENGINE_CRATES {
        if let Some(index) = package_index(workspace, engine) {
            reachable.insert(index);
            queue.push_back(index);
        }
    }
    for _expansion in 0..32 {
        let Some(current) = queue.pop_front() else {
            break;
        };
        for edge_index in 0..256 {
            let Some(&(from, to)) = workspace.edges.get(edge_index) else {
                break;
            };
            if from == current && reachable.insert(to) {
                queue.push_back(to);
            }
        }
    }
    let mut ordered = Vec::new();
    for index in 0..32 {
        if reachable.contains(&index) {
            ordered.push(index);
        }
    }
    ordered
}

fn enumerate_sources(
    crate_root: &Path,
    budget: &mut SourceWalkBudget,
) -> Result<Vec<PathBuf>, String> {
    let canonical_root = crate_root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", crate_root.display()))?;
    budget.directories = budget.directories.saturating_add(1);
    if budget.directories > MAX_DIRECTORIES {
        return Err(format!(
            "directory {} exceeds {} beneath workspace roots",
            budget.directories, MAX_DIRECTORIES
        ));
    }
    let mut directories = VecDeque::from([(canonical_root.clone(), 0_usize)]);
    let mut sources = Vec::new();
    for _directory_index in 0..512 {
        let Some((directory, depth)) = directories.pop_front() else {
            return Ok(sources);
        };
        let mut entries = std::fs::read_dir(&directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        for _entry_index in 0..=16_384 {
            let Some(entry) = entries.next() else { break };
            let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
            budget.entries = budget.entries.saturating_add(1);
            if budget.entries > MAX_DIRECTORY_ENTRIES {
                return Err(format!(
                    "directory entry {} exceeds {} beneath {}",
                    budget.entries,
                    MAX_DIRECTORY_ENTRIES,
                    crate_root.display()
                ));
            }
            admit_source_entry(
                entry,
                depth,
                &canonical_root,
                crate_root,
                &mut directories,
                budget,
                &mut sources,
            )?;
        }
    }
    if directories.is_empty() {
        Ok(sources)
    } else {
        Err(format!(
            "directory {} exceeds {} beneath {}",
            budget.directories,
            MAX_DIRECTORIES,
            crate_root.display()
        ))
    }
}

fn admit_source_entry(
    entry: std::fs::DirEntry,
    depth: usize,
    canonical_root: &Path,
    crate_root: &Path,
    directories: &mut VecDeque<(PathBuf, usize)>,
    budget: &mut SourceWalkBudget,
    sources: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let file_type = entry
        .file_type()
        .map_err(|error| format!("{}: {error}", entry.path().display()))?;
    if file_type.is_symlink() {
        return Err(format!(
            "{}: symlinks are forbidden",
            entry.path().display()
        ));
    }
    let path = entry.path();
    let relative = path
        .strip_prefix(canonical_root)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if relative.to_str().is_none() {
        return Err(format!("{}: non-UTF-8 source path", path.display()));
    }
    if path_contains_component(relative, "tests") {
        return Ok(());
    }
    if file_type.is_dir() {
        let child_depth = depth.saturating_add(1);
        if child_depth > MAX_DIRECTORY_DEPTH {
            return Err(format!(
                "{}: directory depth {} exceeds {}",
                path.display(),
                child_depth,
                MAX_DIRECTORY_DEPTH
            ));
        }
        budget.directories = budget.directories.saturating_add(1);
        if budget.directories > MAX_DIRECTORIES {
            return Err(format!(
                "directory {} exceeds {} beneath {}",
                budget.directories,
                MAX_DIRECTORIES,
                crate_root.display()
            ));
        }
        directories.push_back((path, child_depth));
        return Ok(());
    }
    if !file_type.is_file() {
        return Err(format!("{}: unsupported filesystem entry", path.display()));
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !matches!(extension, "rs" | "bsl" | "bscn") {
        return Ok(());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !canonical.starts_with(canonical_root) {
        return Err(format!(
            "{}: canonical source escapes crate root",
            path.display()
        ));
    }
    sources.push(canonical);
    if sources.len() > MAX_SOURCE_PATHS {
        return Err(format!(
            "production source path {} exceeds {} beneath {}",
            sources.len(),
            MAX_SOURCE_PATHS,
            crate_root.display()
        ));
    }
    Ok(())
}

fn path_contains_component(path: &Path, selected: &str) -> bool {
    let mut components = path.components();
    for _index in 0..=16 {
        let Some(component) = components.next() else {
            return false;
        };
        if component.as_os_str() == selected {
            return true;
        }
    }
    false
}

fn scan_rust_source(
    path: &Path,
    bytes: &[u8],
    registry_declarations: &mut usize,
) -> Result<Vec<SourceFinding>, String> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("{}: Rust source is not UTF-8: {error}", path.display()))?;
    let stream = TokenStream::from_str(source)
        .map_err(|error| format!("{}: Rust tokenization failed: {error}", path.display()))?;
    let tokens = bounded_rust_tokens(path, stream)?;
    let file = syn::parse_file(source)
        .map_err(|error| format!("{}: Rust parse failed: {error}", path.display()))?;
    let registry = inspect_registry(path, &file, registry_declarations);
    let mut findings = Vec::new();
    if let Some(error) = registry.error {
        findings.push(SourceFinding {
            path: path.to_path_buf(),
            line: line_of(source, REGISTRY_NAME),
            token: error,
        });
    }
    for token_index in 0..65_536 {
        let Some(token) = tokens.get(token_index) else {
            break;
        };
        if token_span_is_exempt(token.span, &registry.exempt_spans) {
            continue;
        }
        if token.value == REGISTRY_NAME {
            findings.push(SourceFinding {
                path: path.to_path_buf(),
                line: token.span.start.line,
                token: token.value.clone(),
            });
            continue;
        }
        if is_reserved_engine_token(&token.value) {
            findings.push(SourceFinding {
                path: path.to_path_buf(),
                line: token.span.start.line,
                token: token.value.clone(),
            });
        }
    }
    Ok(findings)
}

fn is_reserved_engine_token(token: &str) -> bool {
    for index in 0..10 {
        let Some(reserved) = RESERVED_ENGINE_TOKENS.get(index) else {
            break;
        };
        if *reserved == token {
            return true;
        }
    }
    false
}

fn token_span(tree: &TokenTree) -> TokenSpan {
    TokenSpan {
        start: tree.span().start(),
        end: tree.span().end(),
    }
}

fn bounded_rust_tokens(path: &Path, stream: TokenStream) -> Result<Vec<RustToken>, String> {
    let mut stack = Vec::new();
    push_token_stream(path, stream, 0, &mut stack)?;
    let mut inspected = Vec::new();
    for visited in 0..65_536 {
        let Some((token, depth)) = stack.pop() else {
            return Ok(inspected);
        };
        let span = token_span(&token);
        match token {
            TokenTree::Group(group) => {
                let child_depth = depth.saturating_add(1);
                if child_depth > MAX_RUST_DEPTH {
                    return Err(format!(
                        "{}: Rust token-group depth {} exceeds {}",
                        path.display(),
                        child_depth,
                        MAX_RUST_DEPTH
                    ));
                }
                push_token_stream(path, group.stream(), child_depth, &mut stack)?;
            }
            TokenTree::Ident(identifier) => inspected.push(RustToken {
                value: identifier.to_string(),
                span,
            }),
            TokenTree::Literal(literal) => {
                if let Ok(syn::Lit::Str(string)) = syn::parse_str::<syn::Lit>(&literal.to_string())
                {
                    inspected.push(RustToken {
                        value: string.value(),
                        span,
                    });
                }
            }
            TokenTree::Punct(_) => {}
        }
        if visited == MAX_RUST_TOKENS - 1 && !stack.is_empty() {
            return Err(format!(
                "{}: Rust token tree 65537 exceeds {}",
                path.display(),
                MAX_RUST_TOKENS
            ));
        }
    }
    if stack.is_empty() {
        Ok(inspected)
    } else {
        Err(format!(
            "{}: Rust token walk did not terminate",
            path.display()
        ))
    }
}

fn push_token_stream(
    path: &Path,
    stream: TokenStream,
    depth: usize,
    stack: &mut Vec<(TokenTree, usize)>,
) -> Result<(), String> {
    let mut added = Vec::new();
    let mut iterator = stream.into_iter();
    for _index in 0..=65_536 {
        let Some(token) = iterator.next() else { break };
        added.push(token);
        if stack.len().saturating_add(added.len()) > MAX_RUST_STACK {
            return Err(format!(
                "{}: Rust token stack exceeds {}",
                path.display(),
                MAX_RUST_STACK
            ));
        }
    }
    for token_offset in 0..65_536 {
        let Some(token_index) = added.len().checked_sub(token_offset + 1) else {
            break;
        };
        stack.push((added[token_index].clone(), depth));
    }
    Ok(())
}

fn inspect_registry(
    path: &Path,
    file: &syn::File,
    registry_declarations: &mut usize,
) -> RegistryInspection {
    let mut inspection = RegistryInspection::default();
    for item_index in 0..65_536 {
        let Some(item) = file.items.get(item_index) else {
            break;
        };
        let syn::Item::Const(item_const) = item else {
            continue;
        };
        if item_const.ident != REGISTRY_NAME {
            continue;
        }
        *registry_declarations = registry_declarations.saturating_add(1);
        if !path_ends_with(path, REGISTRY_PATH) {
            inspection.error = Some(format!(
                "{REGISTRY_NAME} declaration is outside {REGISTRY_PATH}"
            ));
            return inspection;
        }
        let syn::Expr::Array(array) = item_const.expr.as_ref() else {
            inspection.error = Some(format!("{REGISTRY_NAME} must be an array declaration"));
            return inspection;
        };
        if array.elems.len() != RESERVED_ENGINE_TOKENS.len() {
            inspection.error = Some(format!(
                "{REGISTRY_NAME} row count {} does not match digest {REGISTRY_DIGEST}",
                array.elems.len()
            ));
            return inspection;
        }
        let mut rows = Vec::with_capacity(array.elems.len());
        let mut spans = Vec::with_capacity(array.elems.len().saturating_add(1));
        spans.push(TokenSpan {
            start: item_const.ident.span().start(),
            end: item_const.ident.span().end(),
        });
        for element_index in 0..10 {
            let Some(syn::Expr::Lit(expression)) = array.elems.get(element_index) else {
                inspection.error = Some(format!("{REGISTRY_NAME} contains a non-literal row"));
                return inspection;
            };
            let syn::Lit::Str(value) = &expression.lit else {
                inspection.error = Some(format!("{REGISTRY_NAME} contains a non-string row"));
                return inspection;
            };
            rows.push(value.value());
            spans.push(TokenSpan {
                start: value.span().start(),
                end: value.span().end(),
            });
        }
        if let Err(error) = validate_registry_rows(&rows) {
            inspection.error = Some(error);
            return inspection;
        }
        inspection.exempt_spans = spans;
    }
    inspection
}

fn validate_registry_rows(rows: &[String]) -> Result<(), String> {
    for row_index in 1..10 {
        if rows[row_index - 1].as_bytes() >= rows[row_index].as_bytes() {
            return Err(format!(
                "{REGISTRY_NAME} bytes do not match digest {REGISTRY_DIGEST}"
            ));
        }
    }
    let mut rendered = Vec::new();
    for row_index in 0..10 {
        let Some(row) = rows.get(row_index) else {
            return Err(format!("{REGISTRY_NAME} rows lack LF termination"));
        };
        rendered.extend_from_slice(row.as_bytes());
        rendered.push(b'\n');
    }
    let digest = babylon_kernel::sha256_of(&rendered);
    let mut digest_hex = String::with_capacity(64);
    for byte_index in 0..32 {
        let Some(byte) = digest.get(byte_index) else {
            return Err(format!("{REGISTRY_NAME} digest is incomplete"));
        };
        digest_hex.push_str(&format!("{byte:02x}"));
    }
    if digest_hex != REGISTRY_DIGEST {
        return Err(format!(
            "{REGISTRY_NAME} bytes do not match digest {REGISTRY_DIGEST}"
        ));
    }
    Ok(())
}

fn token_span_is_exempt(span: TokenSpan, exemptions: &[TokenSpan]) -> bool {
    for exemption_index in 0..11 {
        let Some(exemption) = exemptions.get(exemption_index) else {
            break;
        };
        if span == *exemption {
            return true;
        }
    }
    false
}

fn path_ends_with(path: &Path, suffix: &str) -> bool {
    path.ends_with(Path::new(suffix))
}

fn line_of(source: &str, token: &str) -> usize {
    let Some(offset) = source.find(token) else {
        return 1;
    };
    let bytes = source.as_bytes();
    let mut line = 1_usize;
    for byte_index in 0..262_144 {
        if byte_index >= offset {
            break;
        }
        let Some(byte) = bytes.get(byte_index) else {
            break;
        };
        if *byte == b'\n' {
            line = line.saturating_add(1);
        }
    }
    line
}

fn scan_sexpr_source(
    path: &Path,
    bytes: &[u8],
    extension: &str,
) -> Result<Vec<SourceFinding>, String> {
    let forms = babylon_bsl::read_all(bytes).map_err(|error| {
        format!(
            "{}: {extension} reader refusal at byte {}: {:?}",
            path.display(),
            error.position,
            error.kind
        )
    })?;
    preflight_sexpr(path, &forms)?;
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("{}: source is not UTF-8: {error}", path.display()))?;
    let tokens = semantic_sexpr_tokens(path, &forms, extension)?;
    let mut findings = Vec::with_capacity(tokens.len());
    for token_index in 0..1_048_576 {
        let Some(token) = tokens.get(token_index) else {
            break;
        };
        findings.push(SourceFinding {
            path: path.to_path_buf(),
            line: line_of(source, token),
            token: token.clone(),
        });
    }
    Ok(findings)
}

fn preflight_sexpr(path: &Path, forms: &[SExpr]) -> Result<(), String> {
    if forms.len() > MAX_AST_WALK_STACK {
        return Err(format!(
            "{}: S-expression root stack {} exceeds {}",
            path.display(),
            forms.len(),
            MAX_AST_WALK_STACK
        ));
    }
    let mut stack = Vec::with_capacity(forms.len());
    for form_offset in 0..65_536 {
        let Some(form_index) = forms.len().checked_sub(form_offset + 1_usize) else {
            break;
        };
        stack.push((&forms[form_index], 0_usize));
    }
    for visited in 0..1_048_576 {
        let Some((expression, depth)) = stack.pop() else {
            return Ok(());
        };
        let SExpr::List(items) = expression else {
            continue;
        };
        let child_depth = depth.saturating_add(1);
        if child_depth > MAX_AST_WALK_DEPTH {
            return Err(format!(
                "{}: S-expression semantic depth {} exceeds {} (reader maximum {})",
                path.display(),
                child_depth,
                MAX_AST_WALK_DEPTH,
                MAX_READER_NESTING_DEPTH
            ));
        }
        if stack.len().saturating_add(items.len()) > MAX_AST_WALK_STACK {
            return Err(format!(
                "{}: S-expression stack exceeds {}",
                path.display(),
                MAX_AST_WALK_STACK
            ));
        }
        for child_offset in 0..65_536 {
            let Some(child_index) = items.len().checked_sub(child_offset + 1) else {
                break;
            };
            stack.push((&items[child_index], child_depth));
        }
        if visited == MAX_AST_WALK_NODES - 1 && !stack.is_empty() {
            return Err(format!(
                "{}: S-expression node 1048577 exceeds {}",
                path.display(),
                MAX_AST_WALK_NODES
            ));
        }
    }
    if stack.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}: S-expression walk did not terminate",
            path.display()
        ))
    }
}

fn semantic_sexpr_tokens(
    path: &Path,
    forms: &[SExpr],
    extension: &str,
) -> Result<Vec<String>, String> {
    let mut stack = Vec::with_capacity(forms.len());
    for form_offset in 0..65_536 {
        let Some(form_index) = forms.len().checked_sub(form_offset + 1_usize) else {
            break;
        };
        stack.push(&forms[form_index]);
    }
    let mut tokens = Vec::new();
    for visited in 0..1_048_576 {
        let Some(expression) = stack.pop() else {
            return Ok(tokens);
        };
        let SExpr::List(items) = expression else {
            continue;
        };
        inspect_semantic_list(items, extension, &mut tokens);
        if stack.len().saturating_add(items.len()) > MAX_AST_WALK_STACK {
            return Err(format!(
                "{}: semantic S-expression stack exceeds {}",
                path.display(),
                MAX_AST_WALK_STACK
            ));
        }
        for child_offset in 0..65_536 {
            let Some(child_index) = items.len().checked_sub(child_offset + 1) else {
                break;
            };
            stack.push(&items[child_index]);
        }
        if visited == MAX_AST_WALK_NODES - 1 && !stack.is_empty() {
            return Err(format!(
                "{}: semantic S-expression node count exceeds {}",
                path.display(),
                MAX_AST_WALK_NODES
            ));
        }
    }
    if stack.is_empty() {
        Ok(tokens)
    } else {
        Err(format!(
            "{}: semantic S-expression walk did not terminate",
            path.display()
        ))
    }
}

fn inspect_semantic_list(items: &[SExpr], extension: &str, output: &mut Vec<String>) {
    let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
        return;
    };
    match head.as_str() {
        "deffield" => inspect_qname(items.get(1), output),
        "field-of" => inspect_qname(items.get(2), output),
        "update-node" | "update-edge" | "update-hyperedge" => {
            inspect_qname(items.get(2), output);
        }
        "bindings" => inspect_binding_fields(items, output),
        "node" if extension == "bscn" => {
            for attribute_index in 3..65_536 {
                let Some(attribute) = items.get(attribute_index) else {
                    break;
                };
                if let SExpr::List(pair) = attribute {
                    inspect_qname(pair.first(), output);
                }
            }
        }
        "edge-attr" if extension == "bscn" => inspect_qname(items.get(4), output),
        "hyperedge-attr" if extension == "bscn" => inspect_qname(items.get(2), output),
        _ => {}
    }
}

fn inspect_binding_fields(items: &[SExpr], output: &mut Vec<String>) {
    for binding_index in 1..65_536 {
        let Some(binding) = items.get(binding_index) else {
            break;
        };
        let SExpr::List(parts) = binding else {
            continue;
        };
        for part_index in 0..65_536 {
            let Some(current) = parts.get(part_index) else {
                break;
            };
            if matches!(current, SExpr::Atom(Atom::Keyword(keyword)) if keyword == "field") {
                inspect_qname(parts.get(part_index.saturating_add(1)), output);
            }
        }
    }
}

fn inspect_qname(expression: Option<&SExpr>, output: &mut Vec<String>) {
    let Some(SExpr::Atom(Atom::QName(qname))) = expression else {
        return;
    };
    if is_reserved_qname(qname) {
        output.push(qname.clone());
    }
}

fn is_reserved_qname(qname: &str) -> bool {
    for index in 0..5 {
        let Some(reserved) = RESERVED_ENGINE_TOKENS.get(index) else {
            break;
        };
        if *reserved == qname {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested_toml_arrays(depth: usize) -> toml::Value {
        let mut value = toml::Value::Integer(0);
        for _depth_index in 0..33 {
            if _depth_index >= depth {
                break;
            }
            value = toml::Value::Array(vec![value]);
        }
        value
    }

    fn toml_value_fixture(overflow: bool) -> toml::Value {
        let mut groups = Vec::with_capacity(256);
        for group_index in 0..256 {
            let mut leaf_count: usize = if group_index == 255 { 254 } else { 255 };
            if overflow && group_index == 255 {
                leaf_count = leaf_count.saturating_add(1);
            }
            let mut leaves = Vec::with_capacity(leaf_count);
            for leaf_index in 0..255 {
                if leaf_index >= leaf_count {
                    break;
                }
                leaves.push(toml::Value::Integer(0));
            }
            groups.push(toml::Value::Array(leaves));
        }
        toml::Value::Array(groups)
    }

    fn sexpr_node_fixture(overflow: bool) -> SExpr {
        let mut groups = Vec::with_capacity(4_096);
        for group_index in 0..4_096 {
            let mut leaf_count: usize = if group_index == 4_095 { 254 } else { 255 };
            if overflow && group_index == 4_095 {
                leaf_count = leaf_count.saturating_add(1);
            }
            let mut leaves = Vec::with_capacity(leaf_count);
            for leaf_index in 0..255 {
                if leaf_index >= leaf_count {
                    break;
                }
                leaves.push(SExpr::List(Vec::new()));
            }
            groups.push(SExpr::List(leaves));
        }
        SExpr::List(groups)
    }

    #[test]
    fn toml_path_and_inline_depth_have_independent_exact_boundaries() {
        let path = Path::new("Cargo.toml");
        let path64 = std::iter::repeat_n("a", 64).collect::<Vec<_>>().join(".");
        assert!(preflight_toml(path, format!("{path64} = 1\n").as_bytes()).is_ok());
        assert!(preflight_toml(path, format!("{path64}.a = 1\n").as_bytes()).is_err());
        let depth32 = format!("value = {}0{}\n", "[".repeat(32), "]".repeat(32));
        assert!(preflight_toml(path, depth32.as_bytes()).is_ok());
        let depth33 = format!("value = {}0{}\n", "[".repeat(33), "]".repeat(33));
        assert!(preflight_toml(path, depth33.as_bytes()).is_err());
    }

    #[test]
    fn parsed_toml_depth_value_and_stack_bounds_pin_exact_max_plus_one() {
        let path = Path::new("Cargo.toml");
        assert!(preflight_toml_value(path, &nested_toml_arrays(32)).is_ok());
        assert!(preflight_toml_value(path, &nested_toml_arrays(33)).is_err());
        assert!(preflight_toml_value(path, &toml_value_fixture(false)).is_ok());
        assert!(preflight_toml_value(path, &toml_value_fixture(true)).is_err());
        assert!(check_toml_stack_capacity(path, 0, 65_536).is_ok());
        assert!(check_toml_stack_capacity(path, 0, 65_537).is_err());
    }

    #[test]
    fn sexpr_node_bound_pins_exact_max_plus_one() {
        let path = Path::new("source.bsl");
        assert!(preflight_sexpr(path, &[sexpr_node_fixture(false)]).is_ok());
        assert!(preflight_sexpr(path, &[sexpr_node_fixture(true)]).is_err());
    }
}
