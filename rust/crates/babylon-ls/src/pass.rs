//! The per-file diagnostic pass (issue #652 Task 6, plan §6.3): resolve a
//! `.bsl` path to its content set(s) via the [`ContentSetManifest`], run
//! `babylon-tick`'s `diagnose_content_set`, and map every resulting error
//! to a [`lsp_types::Diagnostic`] located within THIS file's own forest.
//! Sits above [`crate::diagnostics`] (the mapping layer) and
//! [`crate::locator`]; [`crate::lifecycle`]'s push/pull wiring calls in
//! here, never into the mapping layer directly.
//!
//! **`SourceReader`** is the seam between "how a file's text is obtained"
//! and "what to do with it": a real server reads open documents from its
//! [`crate::document_store::DocumentStore`] and everything else from disk
//! (`DiskSourceReader`); a test injects fixtures in memory
//! (`FixtureSourceReader`) with no filesystem at all.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lsp_types::{Diagnostic, DiagnosticSeverity, Uri};

use babylon_bsl::{compose_declaration_preludes, ContentSetAnalysisV1, FormPath};
use babylon_tick::{
    analyze_content_set_sources_with_kernel_slots, diagnose_content_set_sources,
    forecast_scenario_determined_event_likelihoods_with_kernel_slots, ContentRuleSourceV1,
    ContentSetSourceAnalysisErrorV1, ForecastErrorV1, SourcedPrepareErrorV1,
};

use crate::authoring::{
    merge_content_set_snapshots, snapshot_from_content_analysis, AuthoringSnapshot,
    EventLikelihoodAnalysisFact, EventLikelihoodFact, ForecastRefusalStage,
};
use crate::content_manifest::{ContentSetManifest, KernelSlotReservationMatch};
use crate::diagnostics::{diagnostics_for_file, missing_manifest_row_diagnostic, Located};
use crate::document_store::DocumentStore;
use crate::line_index::LineIndex;
use crate::uri::{file_path_from_uri, uri_from_file_path};

/// Obtain a content-root-relative file's current text — open-document
/// content when open, disk content otherwise (§6.1: "on-disk edits
/// outside open documents are picked up... at pull time").
pub trait SourceReader {
    /// `None` when the path names no readable source at all (never a
    /// panic — a sibling file a set names but that is missing on disk is
    /// a REAL condition, not a bug in this crate).
    fn read(&self, content_relative_path: &str) -> Option<String>;
}

/// Reads every path straight from disk, rooted at `content_root` (the
/// manifest's own directory, §4.1's "relative to THIS FILE's directory").
pub struct DiskSourceReader<'a> {
    /// The content root every manifest path is relative to.
    pub content_root: &'a Path,
}

impl SourceReader for DiskSourceReader<'_> {
    fn read(&self, content_relative_path: &str) -> Option<String> {
        std::fs::read_to_string(self.content_root.join(content_relative_path)).ok()
    }
}

/// The real server's own reader (§6.1: "on-disk edits outside open
/// documents are picked up only at pull time"): an open document's
/// current buffer wins over its on-disk content; anything not open reads
/// straight from disk under `content_root`.
pub struct LiveSourceReader<'a> {
    /// The content root every manifest path is relative to.
    pub content_root: &'a Path,
    /// The in-memory document store — checked first.
    pub store: &'a DocumentStore,
}

impl SourceReader for LiveSourceReader<'_> {
    fn read(&self, content_relative_path: &str) -> Option<String> {
        let absolute = self.content_root.join(content_relative_path);
        if let Some(uri) = uri_from_file_path(&absolute) {
            if let Some(document) = self.store.get(&uri) {
                return Some(document.text.clone());
            }
        }
        std::fs::read_to_string(&absolute).ok()
    }
}

/// An in-memory fixture reader — no filesystem, deterministic, for tests.
#[derive(Debug, Default, Clone)]
pub struct FixtureSourceReader {
    /// Content-root-relative path → source text.
    pub files: HashMap<String, String>,
}

impl SourceReader for FixtureSourceReader {
    fn read(&self, content_relative_path: &str) -> Option<String> {
        self.files.get(content_relative_path).cloned()
    }
}

/// Diagnose one `.bsl` file (§6.3's own bullet): resolve its content
/// set(s) from `manifest`, run `diagnose_content_set` against each, and
/// map every resulting `PrepareError` to a `Diagnostic` located in THIS
/// file's own `(forest, SpanTable)`. A path naming no manifest row at all
/// gets exactly the Information notice (§6.3's own "declaration-
/// independent stages" bullet is Task 6's own File-tier default: a file
/// this crate cannot resolve to a content set gets no partial re-run of
/// the pipeline against a fabricated context, only the drift alarm).
///
/// Bounded by `sets.len()` (Power-of-10 rule 2) — a file named by more
/// than one set (`carceral-arc-conformance.bscn`'s two rule packs, §4.2)
/// gets the union of every set's own diagnostics.
#[must_use]
pub fn diagnose_bsl(
    uri: &Uri,
    content_relative_path: &str,
    manifest: &ContentSetManifest,
    source: &dyn SourceReader,
) -> Vec<Diagnostic> {
    let sets = manifest.sets_for(content_relative_path);
    if sets.is_empty() {
        let Some(text) = source.read(content_relative_path) else {
            return Vec::new();
        };
        let line_index = LineIndex::new(&text);
        return vec![missing_manifest_row_diagnostic(
            &text,
            &line_index,
            content_relative_path,
        )];
    }
    if sets.iter().any(|set| set.prelude.len() > 16) {
        return vec![source_refusal_diagnostic(
            "a content set may name at most 16 declaration preludes",
        )];
    }
    let Some(text) = source.read(content_relative_path) else {
        return Vec::new();
    };
    let line_index = LineIndex::new(&text);
    let mut located: Vec<Located> = Vec::new();
    for set in &sets {
        let Some(scenario_src) = source.read(&set.scenario) else {
            continue;
        };
        let mut prelude_srcs = Vec::with_capacity(set.prelude.len());
        let mut missing_prelude = None;
        for prelude_index in 0..16 {
            if prelude_index == set.prelude.len() {
                break;
            }
            let path = &set.prelude[prelude_index];
            if let Some(prelude_src) = source.read(path) {
                prelude_srcs.push(prelude_src);
            } else {
                missing_prelude = Some(path.as_str());
                break;
            }
        }
        if let Some(path) = missing_prelude {
            located.push(source_refusal(format!(
                "content set `{}` names missing declaration prelude `{path}`",
                set.id
            )));
            continue;
        }
        let prelude_refs: Vec<&str> = prelude_srcs.iter().map(String::as_str).collect();
        let named_preludes = set
            .prelude
            .iter()
            .zip(&prelude_srcs)
            .map(|(source_id, source)| ContentRuleSourceV1 { source_id, source })
            .collect::<Vec<_>>();
        let prelude_src = if prelude_refs.is_empty() {
            None
        } else {
            match compose_declaration_preludes(&prelude_refs) {
                Ok(composed) => Some(composed),
                Err(error) => {
                    located.push(Located::from_scenario_error(&error));
                    continue;
                }
            }
        };
        let rule_srcs = set
            .rules
            .iter()
            .filter_map(|path| source.read(path).map(|rule_src| (path.as_str(), rule_src)))
            .collect::<Vec<_>>();
        let named_sources = rule_srcs
            .iter()
            .map(|(source_id, rule_src)| ContentRuleSourceV1 {
                source_id,
                source: rule_src,
            })
            .collect::<Vec<_>>();
        let errors =
            diagnose_content_set_sources(&scenario_src, prelude_src.as_deref(), &named_sources);
        located.extend(prepare_refusals_for_source(&errors, content_relative_path));
        located.extend(kernel_slot_refusals_from_sources(
            ContentRuleSourceV1 {
                source_id: &set.scenario,
                source: &scenario_src,
            },
            &named_preludes,
            &named_sources,
            manifest,
            content_relative_path,
        ));
    }
    diagnostics_for_file(uri, &text, &line_index, &located)
}

fn prepare_refusals_for_source(errors: &[SourcedPrepareErrorV1], source_id: &str) -> Vec<Located> {
    errors
        .iter()
        .filter(|sourced| {
            sourced
                .source_id
                .as_deref()
                .is_none_or(|owner| owner == source_id)
        })
        .map(|sourced| Located::from_prepare_error(&sourced.error))
        .collect()
}

fn kernel_slot_refusals_from_sources(
    scenario_source: ContentRuleSourceV1<'_>,
    prelude_sources: &[ContentRuleSourceV1<'_>],
    rule_sources: &[ContentRuleSourceV1<'_>],
    manifest: &ContentSetManifest,
    source_id: &str,
) -> Vec<Located> {
    let kernel_slots = manifest.borrowed_kernel_slots();
    match analyze_content_set_sources_with_kernel_slots(
        scenario_source,
        prelude_sources,
        rule_sources,
        &kernel_slots,
    ) {
        Err(ContentSetSourceAnalysisErrorV1 {
            error: babylon_tick::PrepareError::KernelSlot(_),
            partial_analysis: Some(analysis),
        }) => kernel_slot_refusals(&analysis, manifest, source_id),
        Ok(_) | Err(_) => Vec::new(),
    }
}

fn kernel_slot_refusals(
    analysis: &ContentSetAnalysisV1,
    manifest: &ContentSetManifest,
    source_id: &str,
) -> Vec<Located> {
    analysis
        .rules
        .iter()
        .filter(|rule| rule.source_id == source_id)
        .filter_map(|rule| {
            let kernel = rule.kernel.as_ref()?;
            let (form_path, message) = match manifest.match_kernel_slot(
                &rule.rule_id,
                &kernel.sample,
                kernel.slot,
            ) {
                KernelSlotReservationMatch::Exact => return None,
                KernelSlotReservationMatch::Missing => (
                    kernel.sample_path.clone(),
                    format!(
                        "finite kernel `{}` sample `{}` slot {} has no permanent [[kernel_slot]] reservation",
                        rule.rule_id, kernel.sample, kernel.slot
                    ),
                ),
                KernelSlotReservationMatch::SampleMismatch { reservation } => (
                    kernel.sample_path.clone(),
                    format!(
                        "finite kernel `{}` slot {} is permanently reserved for sample `{}`, not live sample `{}`",
                        rule.rule_id, kernel.slot, reservation.sample, kernel.sample
                    ),
                ),
                KernelSlotReservationMatch::SlotMismatch { reservation } => (
                    kernel.slot_path.clone(),
                    format!(
                        "finite kernel `{}` sample `{}` is permanently reserved at slot {}, not live slot {}",
                        rule.rule_id, kernel.sample, reservation.slot, kernel.slot
                    ),
                ),
                KernelSlotReservationMatch::SampleMoved { reservation } => (
                    kernel.sample_path.clone(),
                    format!(
                        "finite kernel `{}` sample `{}` slot {} cannot move from permanent owner `{}` slot {} at reservation ordinal {}",
                        rule.rule_id,
                        kernel.sample,
                        kernel.slot,
                        reservation.rule,
                        reservation.slot,
                        reservation.ordinal
                    ),
                ),
            };
            Some(kernel_slot_refusal(form_path, message))
        })
        .collect()
}

fn likelihood_analysis_facts(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_sources: &[ContentRuleSourceV1<'_>],
    analysis: &ContentSetAnalysisV1,
    manifest: &ContentSetManifest,
) -> Vec<(String, EventLikelihoodAnalysisFact)> {
    let kernel_slots = manifest.borrowed_kernel_slots();
    analysis
        .links
        .iter()
        .map(|link| {
            let fact = match forecast_scenario_determined_event_likelihoods_with_kernel_slots(
                scenario_src,
                prelude_src,
                rule_sources,
                &kernel_slots,
                &link.sample,
            ) {
                Ok(rows) => EventLikelihoodAnalysisFact::Exact(
                    rows.into_iter()
                        .map(|row| EventLikelihoodFact {
                            event_type: row.event_type,
                            favorable_outcomes: row.favorable_outcomes,
                            numerator: row.numerator,
                            denominator: row.denominator,
                        })
                        .collect(),
                ),
                Err(ForecastErrorV1::NotExactlyEnumerable { reason }) => {
                    EventLikelihoodAnalysisFact::StateDependent { reason }
                }
                Err(ForecastErrorV1::Preparation(error)) => EventLikelihoodAnalysisFact::Refused {
                    stage: ForecastRefusalStage::Preparation,
                    reason: error.to_string(),
                },
                Err(ForecastErrorV1::Execution(error)) => EventLikelihoodAnalysisFact::Refused {
                    stage: ForecastRefusalStage::Execution,
                    reason: error.to_string(),
                },
            };
            (link.projection_rule_id.clone(), fact)
        })
        .collect()
}

/// Loader-owned probability authoring facts for one manifest-resolved source.
/// Each set is loaded through `babylon-tick`'s source-aware production
/// schedule wrapper; this layer only maps the returned typed paths to the
/// requested source buffer.
#[must_use]
pub fn analyze_probability_authoring(
    content_relative_path: &str,
    manifest: &ContentSetManifest,
    source: &dyn SourceReader,
) -> AuthoringSnapshot {
    let sets = manifest.sets_for(content_relative_path);
    let Some(text) = source.read(content_relative_path) else {
        return AuthoringSnapshot::default();
    };
    let mut snapshots = Vec::new();
    for set in sets {
        let Some(scenario_src) = source.read(&set.scenario) else {
            continue;
        };
        if set.prelude.len() > 16 {
            continue;
        }
        let prelude_srcs = set
            .prelude
            .iter()
            .map(|path| source.read(path))
            .collect::<Option<Vec<_>>>();
        let Some(prelude_srcs) = prelude_srcs else {
            continue;
        };
        let prelude_refs = prelude_srcs.iter().map(String::as_str).collect::<Vec<_>>();
        let named_preludes = set
            .prelude
            .iter()
            .zip(&prelude_srcs)
            .map(|(source_id, source)| ContentRuleSourceV1 { source_id, source })
            .collect::<Vec<_>>();
        let prelude = if prelude_refs.is_empty() {
            None
        } else {
            let Ok(composed) = compose_declaration_preludes(&prelude_refs) else {
                continue;
            };
            Some(composed)
        };
        let rule_sources = set
            .rules
            .iter()
            .map(|path| source.read(path).map(|text| (path.as_str(), text)))
            .collect::<Option<Vec<_>>>();
        let Some(rule_sources) = rule_sources else {
            continue;
        };
        let named_sources = rule_sources
            .iter()
            .map(|(source_id, source)| ContentRuleSourceV1 { source_id, source })
            .collect::<Vec<_>>();
        let kernel_slots = manifest.borrowed_kernel_slots();
        let (Ok(analysis)
        | Err(ContentSetSourceAnalysisErrorV1 {
            partial_analysis: Some(analysis),
            ..
        })) = analyze_content_set_sources_with_kernel_slots(
            ContentRuleSourceV1 {
                source_id: &set.scenario,
                source: &scenario_src,
            },
            &named_preludes,
            &named_sources,
            &kernel_slots,
        )
        else {
            continue;
        };
        let likelihood_overrides = likelihood_analysis_facts(
            &scenario_src,
            prelude.as_deref(),
            &named_sources,
            &analysis,
            manifest,
        );
        snapshots.push((
            set.id.clone(),
            snapshot_from_content_analysis(
                content_relative_path,
                &text,
                &analysis,
                &likelihood_overrides,
            ),
        ));
    }
    merge_content_set_snapshots(snapshots)
}

fn source_refusal(message: impl Into<String>) -> Located {
    Located {
        code: None,
        family: "E-LOAD",
        identity: None,
        position: None,
        form_path: None,
        message: message.into(),
        severity: DiagnosticSeverity::ERROR,
    }
}

fn kernel_slot_refusal(form_path: FormPath, message: impl Into<String>) -> Located {
    Located {
        code: None,
        family: "E-LOAD",
        identity: None,
        position: None,
        form_path: Some(form_path),
        message: message.into(),
        severity: DiagnosticSeverity::ERROR,
    }
}

fn source_refusal_diagnostic(message: impl Into<String>) -> Diagnostic {
    let mut diagnostic = Diagnostic::new_simple(lsp_types::Range::default(), message.into());
    diagnostic.severity = Some(DiagnosticSeverity::ERROR);
    diagnostic.source = Some("bsl-ls".to_owned());
    diagnostic.data = Some(serde_json::json!({
        "family": "E-LOAD",
        "precision": "file",
    }));
    diagnostic
}

/// The manifest's own directory (§4.1: "Paths... are relative to THIS
/// FILE's directory") — the content root [`DiskSourceReader`] joins every
/// path against.
#[must_use]
pub fn content_root_of(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// The content-root-relative path for `uri`, when it lies under
/// `content_root` — `None` for a file this content root does not own (a
/// legitimate case, not a bug: a workspace can hold files `bsl-ls` never
/// resolves to a content set at all).
#[must_use]
pub fn content_relative_path(content_root: &Path, uri: &Uri) -> Option<String> {
    let absolute = file_path_from_uri(uri)?;
    let relative = absolute.strip_prefix(content_root).ok()?;
    // TOML paths use forward slashes (§4.1's own examples); this crate
    // only ever runs on Unix (the flake's own devshells), where `Path`'s
    // components already use them.
    Some(relative.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::{analyze_probability_authoring, diagnose_bsl, FixtureSourceReader, SourceReader};
    use crate::authoring::{AuthoringKind, EventLikelihoodAnalysisFact, ForecastRefusalStage};
    use crate::content_manifest::ContentSetManifest;
    use lsp_types::Uri;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fmt::Write;
    use std::path::Path;

    fn uri() -> Uri {
        "file:///rules/probe.bsl"
            .parse::<Uri>()
            .expect("valid test URI")
    }

    const SCENARIO: &str = "(scenario ft/probe)";
    // "vitality" is one of `babylon-tick`'s own registered namespaces
    // (`registered_systems`) — unlike "event" (a babylon-bsl conformance-
    // corpus-only namespace), a rule anchored under it clears the §2.3
    // anchor default check that `diagnose_content_set` actually runs.
    const RULE: &str = "(rule vitality/probe :role mechanic :evidence derived :material-basis \"x\" :fuel 16 (bindings) \
                         (effects (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))";
    const KERNEL_SCENARIO: &str = "(scenario ft/probe \
        (defvocabulary NodeType (SOCIAL_CLASS)) \
        (defenum SparkOutcome (YES NO)) \
        (deffield social-class/value int extensive) \
        (node worker NodeType/SOCIAL_CLASS (social-class/value 0)))";

    const PROJECTION_SCENARIO: &str = r"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 0)))
";

    const MULTI_CARRIER_PROJECTION_SCENARIO: &str = r"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 0))
  (node second NodeType/SOCIAL_CLASS
    (social-class/last-incident 0)))
";

    const GUARD_FALSE_PROJECTION_SCENARIO: &str = r"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 1)))
";

    const PROJECTION_MECHANIC: &str = r#"
(rule struggle/spark-mechanic
  :role mechanic
  :evidence designed
  :material-basis "finite material alternatives over the incident state"
  :fuel 256
  (bindings
    (binding last-incident :field social-class/last-incident))
  (when (= last-incident 0))
  (effects
    (choose :sample struggle/spark :slot 0
      (branch StruggleSparkOutcome/EXCESSIVE_FORCE
        :mass 1m
        (effects
          (update-node self social-class/last-incident (set 1))))
      (branch StruggleSparkOutcome/NO_INCIDENT
        :mass 3m
        (effects)))))
"#;

    const PROJECTION_RECOGNIZER: &str = r#"
(rule struggle/spark-recognizer
  :role recognizer
  :evidence derived
  :projects-kernel struggle/spark
  :material-basis "deterministically observes the realized incident state"
  :fuel 128
  (bindings
    (binding last-incident :field social-class/last-incident))
  (when (= last-incident 1))
  (effects
    (emit EventType/EXCESSIVE_FORCE
      (incident-tick last-incident))))
"#;

    fn manifest_with_one_set() -> ContentSetManifest {
        let toml = r#"
schema = 2
[[set]]
id = "probe/set"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/probe.bsl"]
consumers = []
note = "test fixture"
"#;
        ContentSetManifest::parse(Path::new("content-sets.toml"), toml).expect("valid manifest")
    }

    fn manifest_with_two_rule_sources() -> ContentSetManifest {
        let toml = r#"
schema = 2
[[set]]
id = "probe/two-rules"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/good.bsl", "rules/bad.bsl"]
consumers = []
note = "source ownership fixture"
"#;
        ContentSetManifest::parse(Path::new("content-sets.toml"), toml).expect("valid manifest")
    }

    fn projection_manifest() -> ContentSetManifest {
        let toml = r#"
schema = 2
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
[[set]]
id = "probe/projection"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/spark-mechanic.bsl", "rules/spark-recognizer.bsl"]
consumers = []
note = "exact likelihood fixture"
"#;
        ContentSetManifest::parse(Path::new("content-sets.toml"), toml).expect("valid manifest")
    }

    fn conflicting_projection_manifest(reverse: bool) -> ContentSetManifest {
        let sets = [
            r#"
[[set]]
id = "probe/quarter"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/spark-quarter.bsl", "rules/spark-recognizer.bsl"]
consumers = []
note = "quarter likelihood fixture"
"#,
            r#"
[[set]]
id = "probe/half"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/spark-half.bsl", "rules/spark-recognizer.bsl"]
consumers = []
note = "half likelihood fixture"
"#,
        ];
        let ordered = if reverse {
            format!("{}{}", sets[1], sets[0])
        } else {
            format!("{}{}", sets[0], sets[1])
        };
        let toml = format!(
            r#"
schema = 2
[[kernel_slot]]
ordinal = 0
rule = "struggle/spark-mechanic"
sample = "struggle/spark"
slot = 0
{ordered}
"#
        );
        ContentSetManifest::parse(Path::new("content-sets.toml"), &toml)
            .expect("valid conflicting manifest")
    }

    fn kernel_manifest(reservations: &str) -> ContentSetManifest {
        let toml = format!(
            r#"
schema = 2
{reservations}
[[set]]
id = "probe/kernel-slot"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/kernel.bsl", "rules/sibling.bsl"]
consumers = []
note = "kernel slot governance fixture"
"#
        );
        ContentSetManifest::parse(Path::new("content-sets.toml"), &toml)
            .expect("valid kernel-slot manifest")
    }

    fn kernel_source(sample: &str, slot: u32) -> String {
        format!(
            "(rule vitality/probe :role mechanic :evidence designed \
             :material-basis \"bounded spark\" :fuel 64 \
             (domain NodeType/SOCIAL_CLASS) \
             (bindings (binding current :field social-class/value)) (effects \
             (choose :sample {sample} :slot {slot} \
               (branch SparkOutcome/YES :mass 1m (effects)) \
               (branch SparkOutcome/NO :mass 3m (effects)))))"
        )
    }

    fn kernel_source_reader(source_text: &str) -> FixtureSourceReader {
        FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), KERNEL_SCENARIO.to_owned()),
                ("rules/kernel.bsl".to_owned(), source_text.to_owned()),
                (
                    "rules/sibling.bsl".to_owned(),
                    RULE.replace("vitality/probe", "vitality/sibling").replace(
                        ":fuel 16 (bindings)",
                        ":fuel 16 (domain NodeType/SOCIAL_CLASS) (bindings)",
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn a_clean_content_set_produces_no_diagnostics() {
        let manifest = manifest_with_one_set();
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), SCENARIO.to_owned()),
                ("rules/probe.bsl".to_owned(), RULE.to_owned()),
            ]
            .into_iter()
            .collect(),
        };
        let diags = diagnose_bsl(&uri(), "rules/probe.bsl", &manifest, &source);
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn exact_kernel_slot_reservation_is_accepted() {
        let manifest = kernel_manifest(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "vitality/probe"
sample = "struggle/spark"
slot = 0
"#,
        );
        let source_text = kernel_source("struggle/spark", 0);
        let source = kernel_source_reader(&source_text);
        let diagnostics = diagnose_bsl(
            &"file:///rules/kernel.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/kernel.bsl",
            &manifest,
            &source,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn missing_kernel_slot_reservation_is_exact_source_owned_and_deterministic() {
        let manifest = kernel_manifest("");
        let source_text = kernel_source("struggle/spark", 0);
        let source = kernel_source_reader(&source_text);
        let kernel_uri = "file:///rules/kernel.bsl"
            .parse::<Uri>()
            .expect("valid test URI");

        let first = diagnose_bsl(&kernel_uri, "rules/kernel.bsl", &manifest, &source);
        let second = diagnose_bsl(&kernel_uri, "rules/kernel.bsl", &manifest, &source);
        assert_eq!(first, second);
        assert_eq!(first.len(), 1, "{first:?}");
        assert!(first[0].message.contains("no permanent [[kernel_slot]]"));
        assert_eq!(
            first[0].data,
            Some(serde_json::json!({"family": "E-LOAD", "precision": "exact"}))
        );
        let sample_start = u32::try_from(source_text.find("struggle/spark").expect("sample token"))
            .expect("bounded sample offset");
        assert_eq!(first[0].range.start.line, 0);
        assert_eq!(first[0].range.start.character, sample_start);
        assert_eq!(
            first[0].range.end.character,
            sample_start + u32::try_from("struggle/spark".len()).expect("bounded sample")
        );

        let sibling = diagnose_bsl(
            &"file:///rules/sibling.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/sibling.bsl",
            &manifest,
            &source,
        );
        assert!(
            sibling.is_empty(),
            "kernel refusal leaked to sibling: {sibling:?}"
        );
    }

    #[test]
    fn live_sample_and_slot_mismatches_are_distinct_exact_refusals() {
        let sample_manifest = kernel_manifest(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "vitality/probe"
sample = "struggle/reserved"
slot = 0
"#,
        );
        let sample_source_text = kernel_source("struggle/live", 0);
        let sample_source = kernel_source_reader(&sample_source_text);
        let sample = diagnose_bsl(
            &"file:///rules/kernel.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/kernel.bsl",
            &sample_manifest,
            &sample_source,
        );
        assert_eq!(sample.len(), 1, "{sample:?}");
        assert!(sample[0]
            .message
            .contains("reserved for sample `struggle/reserved`"));
        assert!(sample[0].message.contains("live sample `struggle/live`"));
        let sample_start = u32::try_from(
            sample_source_text
                .find("struggle/live")
                .expect("live sample token"),
        )
        .expect("bounded sample offset");
        assert_eq!(sample[0].range.start.character, sample_start);
        assert_eq!(
            sample[0].range.end.character,
            sample_start + u32::try_from("struggle/live".len()).expect("bounded sample")
        );

        let slot_manifest = kernel_manifest(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "vitality/probe"
sample = "struggle/spark"
slot = 0
"#,
        );
        let slot_source_text = kernel_source("struggle/spark", 1);
        let slot_source = kernel_source_reader(&slot_source_text);
        let slot = diagnose_bsl(
            &"file:///rules/kernel.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/kernel.bsl",
            &slot_manifest,
            &slot_source,
        );
        assert_eq!(slot.len(), 1, "{slot:?}");
        assert!(slot[0].message.contains("reserved at slot 0"));
        assert!(slot[0].message.contains("live slot 1"));
        let slot_start = slot_source_text.find(":slot 1").expect("live slot form") + ":slot ".len();
        assert_eq!(
            slot[0].range.start.character,
            u32::try_from(slot_start).expect("bounded slot offset")
        );
        assert_eq!(
            slot[0].range.end.character,
            u32::try_from(slot_start + 1).expect("bounded slot offset")
        );
        assert!(sample.iter().chain(&slot).all(|diagnostic| {
            diagnostic.data == Some(serde_json::json!({"family": "E-LOAD", "precision": "exact"}))
        }));
    }

    #[test]
    fn live_sample_move_names_its_permanent_rule_slot_and_ordinal() {
        let manifest = kernel_manifest(
            r#"
[[kernel_slot]]
ordinal = 0
rule = "vitality/elsewhere"
sample = "struggle/spark"
slot = 0
"#,
        );
        let source_text = kernel_source("struggle/spark", 0);
        let source = kernel_source_reader(&source_text);
        let diagnostics = diagnose_bsl(
            &"file:///rules/kernel.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/kernel.bsl",
            &manifest,
            &source,
        );

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        let diagnostic = &diagnostics[0];
        assert!(diagnostic
            .message
            .contains("finite kernel `vitality/probe`"));
        assert!(diagnostic
            .message
            .contains("permanent owner `vitality/elsewhere` slot 0"));
        assert!(diagnostic.message.contains("reservation ordinal 0"));
        let sample_start = u32::try_from(
            source_text
                .find("struggle/spark")
                .expect("live sample token"),
        )
        .expect("bounded sample offset");
        assert_eq!(diagnostic.range.start.character, sample_start);
        assert_eq!(
            diagnostic.range.end.character,
            sample_start + u32::try_from("struggle/spark".len()).expect("bounded sample")
        );
    }

    #[test]
    fn probability_authoring_uses_source_aware_loader_analysis_deterministically() {
        let manifest = manifest_with_one_set();
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), KERNEL_SCENARIO.to_owned()),
                (
                    "rules/probe.bsl".to_owned(),
                    "(rule vitality/probe :role mechanic :evidence designed \
                     :material-basis \"bounded spark\" :fuel 64 \
                     (domain NodeType/SOCIAL_CLASS) \
                     (bindings (binding current :field social-class/value)) (effects \
                     (choose :sample struggle/spark :slot 0 \
                       (branch SparkOutcome/YES :mass 1m (effects)) \
                       (branch SparkOutcome/NO :mass 3m (effects)))))"
                        .to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let first = analyze_probability_authoring("rules/probe.bsl", &manifest, &source);
        let second = analyze_probability_authoring("rules/probe.bsl", &manifest, &source);

        assert_eq!(first, second);
        assert!(first
            .facts
            .iter()
            .any(|fact| matches!(fact.kind, AuthoringKind::Choose { .. })));
        assert_eq!(
            first
                .facts
                .iter()
                .filter(|fact| matches!(fact.kind, AuthoringKind::Mass { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn probability_authoring_preserves_a_later_top_form_span() {
        let manifest = manifest_with_one_set();
        let source_text = "(rule vitality/first :role mechanic :evidence derived \
            :material-basis \"first top form\" :fuel 16 \
            (domain NodeType/SOCIAL_CLASS) (bindings) \
            (effects (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))\n\
            (rule vitality/second :role mechanic :evidence designed \
            :material-basis \"second top form\" :fuel 64 \
            (domain NodeType/SOCIAL_CLASS) \
            (bindings (binding current :field social-class/value)) (effects \
            (choose :sample struggle/later :slot 0 \
              (branch SparkOutcome/YES :mass 1m (effects)) \
              (branch SparkOutcome/NO :mass 3m (effects)))))";
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), KERNEL_SCENARIO.to_owned()),
                ("rules/probe.bsl".to_owned(), source_text.to_owned()),
            ]
            .into_iter()
            .collect(),
        };

        let snapshot = analyze_probability_authoring("rules/probe.bsl", &manifest, &source);
        let choose = snapshot
            .facts
            .iter()
            .find(|fact| matches!(fact.kind, AuthoringKind::Choose { .. }))
            .expect("later choose fact");
        assert_eq!(
            &source_text[choose.token_span.start..choose.token_span.end],
            "choose"
        );
        assert_eq!(
            choose.token_span.start,
            source_text.rfind("choose").expect("later choose token")
        );
    }

    #[test]
    fn probability_diagnostic_preserves_a_later_top_form_span() {
        let manifest = manifest_with_one_set();
        let source_text = "(rule vitality/first :role mechanic :evidence derived \
            :material-basis \"first top form\" :fuel 16 \
            (domain NodeType/SOCIAL_CLASS) (bindings) \
            (effects (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))\n\
            (rule vitality/second :role mechanic :evidence designed \
            :material-basis \"invalid later enum order\" :fuel 64 \
            (domain NodeType/SOCIAL_CLASS) \
            (bindings (binding current :field social-class/value)) (effects \
            (choose :sample struggle/later :slot 0 \
              (branch SparkOutcome/NO :mass 1m (effects)) \
              (branch SparkOutcome/YES :mass 1m (effects)))))";
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), KERNEL_SCENARIO.to_owned()),
                ("rules/probe.bsl".to_owned(), source_text.to_owned()),
            ]
            .into_iter()
            .collect(),
        };

        let diagnostics = diagnose_bsl(&uri(), "rules/probe.bsl", &manifest, &source);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(
            diagnostics[0].data,
            Some(serde_json::json!({"family": "E-LOAD", "precision": "exact"}))
        );
        assert!(
            diagnostics[0].range.start.line > 0,
            "later-form refusal must not map into the first top form: {:?}",
            diagnostics[0].range
        );
    }

    #[test]
    fn scenario_and_prelude_mass_literals_keep_their_manifest_source_identity() {
        let manifest = ContentSetManifest::parse(
            Path::new("content-sets.toml"),
            r#"
schema = 2
[[set]]
id = "probe/mass-declarations"
scenario = "scenario.bscn"
prelude = ["prelude.bsl"]
rules = ["rules/probe.bsl"]
consumers = []
note = "Mass declaration ownership fixture"
"#,
        )
        .expect("valid manifest");
        let source = FixtureSourceReader {
            files: [
                (
                    "scenario.bscn".to_owned(),
                    "(scenario ft/probe (defconst probe/scenario-mass 0.25m))".to_owned(),
                ),
                (
                    "prelude.bsl".to_owned(),
                    "(defconst probe/prelude-mass 0.75m)\n".to_owned(),
                ),
                ("rules/probe.bsl".to_owned(), RULE.to_owned()),
            ]
            .into_iter()
            .collect(),
        };
        let scenario_source = source.read("scenario.bscn").expect("scenario source");
        let prelude_source = source.read("prelude.bsl").expect("prelude source");
        let rule_source = source.read("rules/probe.bsl").expect("rule source");
        let analysis = babylon_tick::analyze_content_set_sources(
            babylon_tick::ContentRuleSourceV1 {
                source_id: "scenario.bscn",
                source: &scenario_source,
            },
            &[babylon_tick::ContentRuleSourceV1 {
                source_id: "prelude.bsl",
                source: &prelude_source,
            }],
            &[babylon_tick::ContentRuleSourceV1 {
                source_id: "rules/probe.bsl",
                source: &rule_source,
            }],
        )
        .expect("named content analysis");
        assert_eq!(analysis.mass_declarations.len(), 2);

        for (source_id, expected) in [
            ("scenario.bscn", 250_000_000_u64),
            ("prelude.bsl", 750_000_000_u64),
        ] {
            let snapshot = analyze_probability_authoring(source_id, &manifest, &source);
            let masses = snapshot
                .facts
                .iter()
                .filter_map(|fact| match &fact.kind {
                    AuthoringKind::Mass { nanounits } => Some(*nanounits),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(masses, [expected], "wrong Mass owner for {source_id}");
        }
    }

    #[test]
    fn probability_diagnostic_is_published_only_for_its_owning_rule_source() {
        let manifest = manifest_with_two_rule_sources();
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), KERNEL_SCENARIO.to_owned()),
                (
                    "rules/good.bsl".to_owned(),
                    RULE.replace(
                        ":fuel 16 (bindings)",
                        ":fuel 16 (domain NodeType/SOCIAL_CLASS) (bindings)",
                    ),
                ),
                (
                    "rules/bad.bsl".to_owned(),
                    "(rule vitality/bad :role mechanic :evidence designed \
                     :material-basis \"invalid enum order\" :fuel 64 \
                     (domain NodeType/SOCIAL_CLASS) \
                     (bindings (binding current :field social-class/value)) (effects \
                     (choose :sample struggle/bad :slot 0 \
                       (branch SparkOutcome/NO :mass 1m (effects)) \
                       (branch SparkOutcome/YES :mass 1m (effects)))))"
                        .to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let good = diagnose_bsl(
            &"file:///rules/good.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/good.bsl",
            &manifest,
            &source,
        );
        let bad = diagnose_bsl(
            &"file:///rules/bad.bsl"
                .parse::<Uri>()
                .expect("valid test URI"),
            "rules/bad.bsl",
            &manifest,
            &source,
        );

        assert!(
            good.is_empty(),
            "sibling refusal leaked into good.bsl: {good:?}"
        );
        assert_eq!(
            bad.len(),
            1,
            "owning source must receive one refusal: {bad:?}"
        );
        assert_eq!(
            bad[0].data,
            Some(serde_json::json!({"family": "E-LOAD", "precision": "exact"}))
        );
    }

    #[test]
    fn paired_single_carrier_source_exposes_exact_likelihood_and_linkage() {
        let manifest = projection_manifest();
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), PROJECTION_SCENARIO.to_owned()),
                (
                    "rules/spark-mechanic.bsl".to_owned(),
                    PROJECTION_MECHANIC.to_owned(),
                ),
                (
                    "rules/spark-recognizer.bsl".to_owned(),
                    PROJECTION_RECOGNIZER.to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let snapshot =
            analyze_probability_authoring("rules/spark-recognizer.bsl", &manifest, &source);
        let projection = snapshot.facts.iter().find_map(|fact| match &fact.kind {
            AuthoringKind::ProjectsKernel {
                linkage,
                likelihood,
                ..
            } => Some((linkage, likelihood)),
            _ => None,
        });
        let (Some(linkage), Some(EventLikelihoodAnalysisFact::Exact(rows))) = projection
            .map(|(linkage, likelihood)| (linkage.as_ref(), likelihood.as_ref()))
            .expect("typed projection fact")
        else {
            panic!("expected exact linked likelihood")
        };

        assert_eq!(linkage.kernel_rule_id, "struggle/spark-mechanic");
        assert_eq!(linkage.projection_rule_id, "struggle/spark-recognizer");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_type, "EXCESSIVE_FORCE");
        assert_eq!(rows[0].favorable_outcomes, ["EXCESSIVE_FORCE"]);
        assert_eq!(rows[0].numerator, babylon_bsl::TICKET_DENOMINATOR / 4);
        assert_eq!(rows[0].denominator, babylon_bsl::TICKET_DENOMINATOR);

        let mut multi_carrier = source.clone();
        multi_carrier.files.insert(
            "scenario.bscn".to_owned(),
            MULTI_CARRIER_PROJECTION_SCENARIO.to_owned(),
        );
        let dynamic =
            analyze_probability_authoring("rules/spark-recognizer.bsl", &manifest, &multi_carrier);
        let reason = dynamic.facts.iter().find_map(|fact| match &fact.kind {
            AuthoringKind::ProjectsKernel {
                likelihood: Some(EventLikelihoodAnalysisFact::StateDependent { reason }),
                ..
            } => Some(reason),
            _ => None,
        });
        assert!(reason.is_some_and(|reason| reason.contains("not exactly one")));
    }

    #[test]
    fn conflicting_cross_set_likelihood_is_explicit_and_manifest_order_independent() {
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), PROJECTION_SCENARIO.to_owned()),
                (
                    "rules/spark-quarter.bsl".to_owned(),
                    PROJECTION_MECHANIC.to_owned(),
                ),
                (
                    "rules/spark-half.bsl".to_owned(),
                    PROJECTION_MECHANIC.replace(":mass 3m", ":mass 1m"),
                ),
                (
                    "rules/spark-recognizer.bsl".to_owned(),
                    PROJECTION_RECOGNIZER.to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let ambiguity = |reverse| {
            let snapshot = analyze_probability_authoring(
                "rules/spark-recognizer.bsl",
                &conflicting_projection_manifest(reverse),
                &source,
            );
            let projections = snapshot
                .facts
                .iter()
                .filter_map(|fact| match &fact.kind {
                    AuthoringKind::ProjectsKernel { likelihood, .. } => likelihood.as_ref(),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(projections.len(), 1, "one same-span projection fact");
            let EventLikelihoodAnalysisFact::Ambiguous { content_sets } = projections[0] else {
                panic!("expected explicit cross-set ambiguity: {projections:?}")
            };
            content_sets.clone()
        };

        assert_eq!(
            ambiguity(false),
            vec!["probe/half".to_owned(), "probe/quarter".to_owned()]
        );
        assert_eq!(ambiguity(true), ambiguity(false));
    }

    #[test]
    fn paired_guard_false_forecast_is_not_reported_as_state_dependence() {
        let manifest = projection_manifest();
        let source = FixtureSourceReader {
            files: [
                (
                    "scenario.bscn".to_owned(),
                    GUARD_FALSE_PROJECTION_SCENARIO.to_owned(),
                ),
                (
                    "rules/spark-mechanic.bsl".to_owned(),
                    PROJECTION_MECHANIC.to_owned(),
                ),
                (
                    "rules/spark-recognizer.bsl".to_owned(),
                    PROJECTION_RECOGNIZER.to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let snapshot =
            analyze_probability_authoring("rules/spark-recognizer.bsl", &manifest, &source);
        let likelihood = snapshot.facts.iter().find_map(|fact| match &fact.kind {
            AuthoringKind::ProjectsKernel { likelihood, .. } => likelihood.as_ref(),
            _ => None,
        });

        let Some(EventLikelihoodAnalysisFact::Refused {
            stage: ForecastRefusalStage::Execution,
            reason,
        }) = likelihood
        else {
            panic!(
                "a guard-false no-instance refusal must remain an execution refusal: {likelihood:?}"
            )
        };
        assert!(reason.contains("does not fire the kernel mechanic"));
    }

    #[test]
    fn paired_zero_total_forecast_is_not_reported_as_state_dependence() {
        let manifest = projection_manifest();
        let zero_total_mechanic = PROJECTION_MECHANIC
            .replace(":mass 1m", ":mass (quantize-mass last-incident)")
            .replace(":mass 3m", ":mass (quantize-mass last-incident)");
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), PROJECTION_SCENARIO.to_owned()),
                ("rules/spark-mechanic.bsl".to_owned(), zero_total_mechanic),
                (
                    "rules/spark-recognizer.bsl".to_owned(),
                    PROJECTION_RECOGNIZER.to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let snapshot =
            analyze_probability_authoring("rules/spark-recognizer.bsl", &manifest, &source);
        let likelihood = snapshot.facts.iter().find_map(|fact| match &fact.kind {
            AuthoringKind::ProjectsKernel { likelihood, .. } => likelihood.as_ref(),
            _ => None,
        });

        let Some(EventLikelihoodAnalysisFact::Refused {
            stage: ForecastRefusalStage::Execution,
            reason,
        }) = likelihood
        else {
            panic!("a zero-total forecast must remain an execution refusal: {likelihood:?}")
        };
        assert!(reason.contains("total mass must be positive"));
    }

    #[test]
    fn a_path_with_no_manifest_row_gets_the_information_notice() {
        let manifest = manifest_with_one_set();
        let source = FixtureSourceReader {
            files: [("orphan.bsl".to_owned(), RULE.to_owned())]
                .into_iter()
                .collect(),
        };
        let diags = diagnose_bsl(&uri(), "orphan.bsl", &manifest, &source);
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags[0].severity,
            Some(lsp_types::DiagnosticSeverity::INFORMATION)
        );
        assert!(diags[0].message.contains("orphan.bsl"));
    }

    #[test]
    fn diagnosing_the_same_content_set_twice_is_byte_identical() {
        let manifest = manifest_with_one_set();
        let source = FixtureSourceReader {
            files: [
                ("scenario.bscn".to_owned(), SCENARIO.to_owned()),
                (
                    "rules/probe.bsl".to_owned(),
                    "(intrinsic floor :params (real) :returns int :cost 5) \
                     (intrinsic floor :params (real) :returns int :cost 6) "
                        .to_owned()
                        + RULE,
                ),
            ]
            .into_iter()
            .collect(),
        };
        let first = diagnose_bsl(&uri(), "rules/probe.bsl", &manifest, &source);
        let second = diagnose_bsl(&uri(), "rules/probe.bsl", &manifest, &source);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert!(!first.is_empty());
    }

    fn two_prelude_manifest(preludes: &str) -> ContentSetManifest {
        let toml = format!(
            r#"
schema = 2
[[set]]
id = "probe/two-preludes"
scenario = "scenario.bscn"
prelude = [{preludes}]
rules = ["rules/probe.bsl"]
consumers = []
note = "ordered prelude fixture"
"#
        );
        ContentSetManifest::parse(Path::new("content-sets.toml"), &toml).expect("valid manifest")
    }

    fn two_prelude_source() -> FixtureSourceReader {
        FixtureSourceReader {
            files: [
                (
                    "scenario.bscn".to_owned(),
                    "(scenario ft/probe (defvocabulary NodeType (SOCIAL_CLASS)) \
                     (node probe NodeType/SOCIAL_CLASS (social-class/marker ProbeKind/READY)))"
                        .to_owned(),
                ),
                (
                    "declarations/enum.bscn".to_owned(),
                    "(defenum ProbeKind (READY))\n".to_owned(),
                ),
                (
                    "declarations/field.bscn".to_owned(),
                    "(deffield social-class/marker enum ProbeKind)\n".to_owned(),
                ),
                (
                    "rules/probe.bsl".to_owned(),
                    RULE.replace(
                        ":fuel 16 (bindings)",
                        ":fuel 16 (domain NodeType/SOCIAL_CLASS) (bindings)",
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn ordered_diagnostics_compose_every_declared_prelude() {
        let ordered =
            two_prelude_manifest("\"declarations/enum.bscn\", \"declarations/field.bscn\"");
        let source = two_prelude_source();
        let ordered_diagnostics = diagnose_bsl(&uri(), "rules/probe.bsl", &ordered, &source);
        assert!(
            ordered_diagnostics.is_empty(),
            "both sources in declared order must satisfy the real loader: {ordered_diagnostics:?}"
        );

        let reversed =
            two_prelude_manifest("\"declarations/field.bscn\", \"declarations/enum.bscn\"");
        assert!(
            !diagnose_bsl(&uri(), "rules/probe.bsl", &reversed, &source).is_empty(),
            "reversing the dependency order must fail"
        );

        let mut missing = two_prelude_source();
        missing.files.remove("declarations/field.bscn");
        let diagnostics = diagnose_bsl(&uri(), "rules/probe.bsl", &ordered, &missing);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("declarations/field.bscn"));
    }

    struct RecordingReader {
        files: HashMap<String, String>,
        reads: RefCell<Vec<String>>,
    }

    impl SourceReader for RecordingReader {
        fn read(&self, content_relative_path: &str) -> Option<String> {
            self.reads
                .borrow_mut()
                .push(content_relative_path.to_owned());
            self.files.get(content_relative_path).cloned()
        }
    }

    #[test]
    fn seventeen_preludes_refuse_before_the_first_source_read() {
        let mut paths = String::new();
        for index in 0..17 {
            if index > 0 {
                paths.push_str(", ");
            }
            write!(&mut paths, "\"declarations/{index}.bscn\"")
                .expect("writing to a String cannot fail");
        }
        let manifest = two_prelude_manifest(&paths);
        let source = RecordingReader {
            files: HashMap::new(),
            reads: RefCell::new(Vec::new()),
        };
        let diagnostics = diagnose_bsl(&uri(), "rules/probe.bsl", &manifest, &source);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("at most 16"));
        assert!(source.reads.borrow().is_empty());
    }
}
