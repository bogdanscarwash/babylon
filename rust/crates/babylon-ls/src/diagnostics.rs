//! The mapping layer (issue #652 Task 6, plan §6.2/§6.4): `PrepareError`/
//! `ScenarioError`/`ReadError` → [`lsp_types::Diagnostic`] — code, severity,
//! source, `data.{family,precision}` (no `d_records`, wave 2 per §6.2's own
//! carve-out), the declared total order, and the `Vec<Diagnostic>` builder.
//! Push ([`crate::lifecycle`]'s `publishDiagnostics`) and pull
//! (`textDocument/diagnostic`, `workspace/diagnostic`) both call through
//! here — ONE mapping, two delivery mechanisms (§6.1).
//!
//! **Observes-only (global constraint 1).** Every fact this module reports
//! was already computed by the loader (`babylon-bsl`/`babylon-tick`); this
//! module never re-implements or widens a check — it maps what the loader
//! already refused to LSP shape. **Never scans a message string for
//! location or code** (sentinel 7.2) — every field read here is a typed
//! field on the loader's own error type.
//!
//! # The wave-1 precision table (§6.6)
//!
//! | Family | Wave-1 precision | Mechanism |
//! |---|---|---|
//! | `E-LEX` (13 codes) | **Exact** | `ReadError`'s own `position`, expanded to the enclosing token's byte range by [`token_span_at`] — a local re-scan over `text`, not `SpanTable::innermost_at`: a read failure never produces a `SpanTable` at all (`read_all_spanned` discards its partial `entries` on the `?` that propagates the error, `reader.rs:383-397`), so there is no table to query for the one error this tier exists to locate. |
//! | Finite-probability refusals carrying a [`FormPath`] | **Exact** | Resolve the loader-owned path directly through this source's [`SpanTable`]; no semantic locator or message scan. |
//! | Any error carrying an [`ErrorIdentity`] (§2.3) | **Form** | One of [`crate::locator`]'s four strategies, dispatched by [`crate::locator::locate`]. Unique match ⇒ that span. Ambiguous ⇒ file-level range plus one `relatedInformation` entry per candidate, sorted into document order. Absent ⇒ file-level, no `relatedInformation`. `data.precision` reads `"form"` regardless of which of the three outcomes the search landed on — it names the error's own CLASS (it carries typed identity at all), not the search's runtime luck. |
//! | `E-TYPE` (15 codes) | **File** | `TypeError` is `{code, message}` with no struct variants (`typecheck.rs:81-85`) — nothing to locate. Wave 2 gives it identity at the raise site. |
//! | Prose-only variants (`Malformed{message}` in six modules; `DomainError::Undeterminable{candidates}`) | **File** (`Undeterminable` ⇒ file + `relatedInformation`, via [`ErrorIdentity::Ambiguous`]) | By construction: the loader names no single token. |
//! | `E-EVAL` reached with no live session | **File**, distinct message | Not reachable from [`babylon_tick::diagnose_content_set`] today — evaluation never runs during a diagnostic pass (load-time checks only); documented for completeness, unimplemented here because nothing produces it yet (the same disclosed-gap discipline as [`crate::locator`]'s `ErrorIdentity::Edge` note). |

use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Range,
    Uri,
};

use babylon_bsl::rule_pipeline::LoadError;
use babylon_bsl::scenario::ScenarioError;
use babylon_bsl::{read_all_spanned, ErrorIdentity, FormPath, SExpr, SpanTable};
use babylon_tick::kernel_slot::KernelSlotLedgerErrorV1;
use babylon_tick::PrepareError;

use crate::line_index::LineIndex;
use crate::locator::{locate, LocateOutcome};

/// `data.precision` (§6.2's own three wave-1 tiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    /// The exact offending token or loader-owned form span.
    Exact,
    /// Any error carrying an [`ErrorIdentity`]: a located form's span, or a
    /// documented fallback to file level (ambiguous/absent).
    Form,
    /// `E-TYPE`, prose-only variants, or (wave 2) `E-EVAL`: no locatable
    /// token at all — the whole file.
    File,
}

impl Precision {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Form => "form",
            Self::File => "file",
        }
    }
}

/// A normalized view of one loader refusal, built by one adapter per
/// concrete error type (§6.4) — everything [`map_to_diagnostic`] needs,
/// independent of which loader stage raised it.
#[derive(Debug, Clone)]
pub struct Located {
    /// `spec_code()` verbatim, or `None` where the loader names none
    /// (never invented, §6.2).
    pub code: Option<&'static str>,
    /// The census family (§1.1: `E-LEX`/`E-PARSE`/`E-LOAD`/`E-TYPE`/
    /// `E-EVAL`) this refusal belongs to — see [`family_of_load_error`]/
    /// [`family_of_scenario_error`]'s own docs for how an UNCODED error
    /// still gets one.
    pub family: &'static str,
    /// WHAT the error is about, when the loader can name it.
    pub identity: Option<ErrorIdentity>,
    /// The byte offset a `ReadError` (`E-LEX`) detected the failure at.
    pub position: Option<usize>,
    /// The loader-owned AST node that raised a structured refusal. Unlike
    /// [`ErrorIdentity`], this locates one exact occurrence without a second
    /// semantic search in the language server.
    pub form_path: Option<FormPath>,
    /// Human-readable detail (the loader's own `Display`).
    pub message: String,
    /// Error for a refusal — every wave-1 producer is one (§6.2).
    pub severity: DiagnosticSeverity,
}

impl Located {
    /// Adapt one `LoadError` (`babylon-bsl`'s rule-load rejection type).
    #[must_use]
    pub fn from_load_error(err: &LoadError) -> Self {
        let position = match err {
            LoadError::Read(read_error) => Some(read_error.position),
            _ => None,
        };
        let form_path = match err {
            LoadError::Probability(error) => error.form_path().map(<[u32]>::to_vec),
            _ => None,
        };
        Self {
            code: err.spec_code(),
            family: family_of_load_error(err),
            identity: babylon_bsl::identity_of(err),
            position,
            form_path,
            message: err.to_string(),
            severity: DiagnosticSeverity::ERROR,
        }
    }

    /// Adapt one `ScenarioError` (`babylon-bsl`'s `.bscn`-load rejection
    /// type).
    #[must_use]
    pub fn from_scenario_error(err: &ScenarioError) -> Self {
        Self {
            code: err.code,
            family: family_of_scenario_error(err),
            identity: err.identity.clone(),
            position: err.position,
            form_path: None,
            message: err.to_string(),
            severity: DiagnosticSeverity::ERROR,
        }
    }

    /// `PrepareError` wraps `LoadError`/`ScenarioError`/`DeclError`, or
    /// (`Composition`) carries `code`/`identity` directly as data (Task 3,
    /// #652) — one delegate per variant, wildcard-free so a new
    /// `PrepareError` variant is a compile error here too.
    #[must_use]
    pub fn from_prepare_error(err: &PrepareError) -> Self {
        match err {
            PrepareError::Scenario(scenario_error) => Self::from_scenario_error(scenario_error),
            PrepareError::Rule { error, .. } => Self::from_load_error(error),
            PrepareError::Intrinsic(decl_error) => {
                // `decl_identity` is `pub(crate)` inside `babylon-bsl`; the
                // crate's own public, wildcard-free `identity_of` already
                // covers `DeclError` via `LoadError::Intrinsic` — reuse
                // that public surface rather than widening a private one.
                // `family_of_load_error` is reused the same way, over the
                // same wrapped value.
                let wrapped = LoadError::Intrinsic(decl_error.clone());
                Self {
                    code: decl_error.spec_code(),
                    family: family_of_load_error(&wrapped),
                    identity: babylon_bsl::identity_of(&wrapped),
                    position: None,
                    form_path: None,
                    message: err.to_string(),
                    severity: DiagnosticSeverity::ERROR,
                }
            }
            PrepareError::Probability { error, .. } => {
                let wrapped = LoadError::Probability(error.clone());
                let mut located = Self::from_load_error(&wrapped);
                located.message = err.to_string();
                located
            }
            PrepareError::KernelSlot(error) => {
                let rule = match error {
                    KernelSlotLedgerErrorV1::MissingLiveReservation { rule, .. }
                    | KernelSlotLedgerErrorV1::LiveSampleMismatch { rule, .. }
                    | KernelSlotLedgerErrorV1::LiveSlotMismatch { rule, .. } => Some(rule),
                    KernelSlotLedgerErrorV1::LiveSampleMoved { actual_rule, .. } => {
                        Some(actual_rule)
                    }
                    KernelSlotLedgerErrorV1::OrdinalCapacity { .. }
                    | KernelSlotLedgerErrorV1::Ordinal { .. }
                    | KernelSlotLedgerErrorV1::Collision { .. }
                    | KernelSlotLedgerErrorV1::Rebind { .. }
                    | KernelSlotLedgerErrorV1::SampleCollision { .. }
                    | KernelSlotLedgerErrorV1::RuleSlotSequence { .. }
                    | KernelSlotLedgerErrorV1::InvalidQName { .. } => None,
                };
                Self {
                    code: None,
                    family: "E-LOAD",
                    identity: rule.cloned().map(ErrorIdentity::RuleId),
                    position: None,
                    form_path: None,
                    message: error.to_string(),
                    severity: DiagnosticSeverity::ERROR,
                }
            }
            PrepareError::Composition {
                code,
                identity,
                message,
            } => Self {
                code: *code,
                // §6.2's own `E-LOAD-001` D32 example is the one live
                // producer of `Composition` today, and every uncoded
                // composition-level refusal (`LoadError::Content`'s own
                // precedent) is content-set-structure, not a written
                // form's shape — `E-LOAD` either way.
                family: code.map_or("E-LOAD", family_from_code),
                identity: identity.clone(),
                position: None,
                form_path: None,
                message: message.clone(),
                severity: DiagnosticSeverity::ERROR,
            },
        }
    }
}

/// The census family prefix ("E-LEX", "E-PARSE", "E-LOAD", "E-TYPE") read
/// off a spec code like `"E-LOAD-001"` — every code in this codebase is
/// exactly `"E-<FAMILY>-<NNN>"` (the closed set `rg -o "E-[A-Z]+-[0-9]+"`
/// across `babylon-bsl` never deviates from that shape), so splitting at
/// the LAST `-` is total and safe. Reads a STRUCTURED field already known
/// to be one of a closed set of literal codes — never `message`/`Display`
/// text (sentinel 7.2's ban is on scanning PROSE, not on reading a
/// substring of an already-typed code).
#[must_use]
pub fn family_from_code(code: &str) -> &str {
    code.rfind('-').map_or(code, |i| &code[..i])
}

/// The census family (§1.1) an UNCODED `LoadError` belongs to — reached
/// only when `err.spec_code()` is `None`; every coded case goes through
/// [`family_from_code`] instead ([`family_of_load_error`]'s own job).
/// Classified per WRAPPED TYPE by reading each module's actual
/// `spec_code()` match (`material_basis.rs`, `bindings.rs`,
/// `mod_anchors.rs`, `domain.rs`, `declarations.rs`,
/// `bound_checker.rs`) — a `Malformed{message}` variant (six modules)
/// shares its family with the module raising it: "malformed shape" is
/// definitionally a §2 grammar/PARSE-level concern regardless of which
/// stage's checker happened to notice it could not even read a valid
/// shape, so every bare `Malformed` case classifies `E-PARSE`. The one
/// exception with a SECOND uncoded, non-`Malformed` arm —
/// `AnchorError::UnregisteredAnchorSystem` (a live producer,
/// `mod_anchors.rs:194`) — is about anchor/system REGISTRATION, the same
/// conceptual family as its coded sibling `NoSystemForRule`
/// (`E-LOAD-002`), so it classifies `E-LOAD` instead. Wildcard-free, the
/// same discipline `identity_of` already holds.
#[must_use]
pub fn family_of_load_error(err: &LoadError) -> &'static str {
    if let Some(code) = err.spec_code() {
        // `family_from_code`'s elided lifetime ties its return to its
        // input's — since `code` here is `&'static str` (`spec_code()`'s
        // own return type), the result is inferred `&'static str` too, no
        // cast needed.
        return family_from_code(code);
    }
    match err {
        LoadError::Read(_) => "E-LEX",
        LoadError::Type(_) => "E-TYPE",
        // The one wrapped type with TWO distinct uncoded arms: `Anchor`'s
        // `UnregisteredAnchorSystem` (a live producer, `mod_anchors.rs:194`)
        // shares its family with its coded sibling `NoSystemForRule`
        // (`E-LOAD-002`, anchor/system REGISTRATION) — every other
        // wrapped type's uncoded arm is a bare `Malformed{message}`
        // (shape-level, `E-PARSE`) alone.
        LoadError::Anchor(babylon_bsl::AnchorError::UnregisteredAnchorSystem { .. })
        | LoadError::Content(_)
        | LoadError::DuplicateRuleId { .. }
        | LoadError::DeferredShapeVerb(_)
        | LoadError::Probability(_)
        // E-LOAD-058/059 return through `spec_code()` above. The bounded
        // same-tick AST-walk refusal is intentionally uncoded but still
        // belongs to the load family, so this arm also classifies that path.
        | LoadError::SameTickOrder(_)
        | LoadError::MintingTypeOperand(_) => "E-LOAD",
        LoadError::Causal(error) => match error {
            babylon_bsl::causal_contract::ContractError::MissingMetadata { .. }
            | babylon_bsl::causal_contract::ContractError::MalformedMetadata { .. }
            | babylon_bsl::causal_contract::ContractError::UnknownMetadataValue { .. }
            | babylon_bsl::causal_contract::ContractError::MalformedRule => "E-PARSE",
            babylon_bsl::causal_contract::ContractError::UnauthorizedEffect { .. }
            | babylon_bsl::causal_contract::ContractError::GovernedFieldWriteProhibited { .. }
            | babylon_bsl::causal_contract::ContractError::GovernedAttributionMismatch { .. }
            | babylon_bsl::causal_contract::ContractError::MismatchedRuleContract { .. }
            | babylon_bsl::causal_contract::ContractError::AstWalkLimit(_)
            | babylon_bsl::causal_contract::ContractError::MismatchedWriteAttribution { .. }
            | babylon_bsl::causal_contract::ContractError::MismatchedWriteOrdinal { .. }
            | babylon_bsl::causal_contract::ContractError::MalformedEventType { .. }
            | babylon_bsl::causal_contract::ContractError::ReceiptOrdinalOverflow => "E-LOAD",
        },
        LoadError::Surface(_)
        | LoadError::Binding(_)
        | LoadError::Grammar(_)
        | LoadError::Anchor(_)
        | LoadError::Domain(_)
        | LoadError::Scope(_)
        | LoadError::ElementName(_)
        | LoadError::Bound(_)
        | LoadError::Intrinsic(_) => "E-PARSE",
    }
}

/// The census family (§1.1) a `ScenarioError` belongs to. `position.is_some()`
/// is `ScenarioError`'s own E-LEX signal (`scenario.rs`'s `From<ReadError>`
/// sets `position` for EVERY read failure, coded or not — `code` alone
/// would miss the uncoded structural ones, e.g. an unterminated list) and
/// is checked first; next, a genuine `code` (`From<VocabularyError>`, or
/// `coded_err()`'s own callers) gives its family directly; the remaining
/// case — a bare `err()`-built prose message (`From<GraphError>`, or a
/// hand-written shape refusal with no wrapped typed error) — has no
/// structural family signal at all, so it defaults to `E-LOAD`: `.bscn`
/// hydration failures are fundamentally about SCENARIO CONTENT structure
/// (duplicate/malformed declarations, substrate refusals), the same
/// bucket `LoadError::Content`'s own uncoded composition-level checks
/// occupy — a disclosed wave-1 simplification, not a proven fact about
/// every individual `err()` call site (some read more like a written
/// form's shape than a load-time structural fact).
#[must_use]
pub fn family_of_scenario_error(err: &ScenarioError) -> &'static str {
    if err.position.is_some() {
        "E-LEX"
    } else if let Some(code) = err.code {
        family_from_code(code)
    } else {
        "E-LOAD"
    }
}

/// This error's precision tier, from its own shape — independent of
/// whether locating it at runtime actually found a unique span (§6.6's
/// table: `data.precision` is a static fact about the error CLASS).
#[must_use]
pub fn precision_of(located: &Located) -> Precision {
    if located.position.is_some() || located.form_path.is_some() {
        Precision::Exact
    } else if located.identity.is_some() {
        Precision::Form
    } else {
        Precision::File
    }
}

/// The byte range of the token containing (or starting at) `position`,
/// scanning `text` directly — the E-LEX tier's own mechanism (see this
/// module's doc for why it does not use [`SpanTable::innermost_at`]).
/// Bounded by `text.len()` in both directions (Power-of-10 rule 2).
#[must_use]
pub fn token_span_at(text: &str, position: usize) -> (usize, usize) {
    let bytes = text.as_bytes();
    let position = position.min(bytes.len());
    if position >= bytes.len() || is_boundary_byte(bytes[position]) {
        // A delimiter, or past end of text: a one-byte (or empty, at EOF)
        // span at `position` itself — nothing wider to claim.
        return (position, (position + 1).min(bytes.len()).max(position));
    }
    let mut start = position;
    while start > 0 && !is_boundary_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = position;
    while end < bytes.len() && !is_boundary_byte(bytes[end]) {
        end += 1;
    }
    (start, end)
}

/// The same token-boundary alphabet the reader's own `is_delimiter` uses
/// (`reader.rs`): whitespace, `(`, `)`, `;` (comment start). ASCII-only
/// check on a raw byte is safe here because every one of these boundary
/// characters is a single ASCII byte in UTF-8, so it can never land mid
/// multi-byte sequence.
fn is_boundary_byte(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'(' | b')' | b';')
}

/// Convert a byte range into `text` to an LSP [`Range`] via `line_index`.
fn byte_range_to_lsp_range(text: &str, line_index: &LineIndex, start: usize, end: usize) -> Range {
    let start_u32 = u32::try_from(start).unwrap_or(u32::MAX);
    let end_u32 = u32::try_from(end).unwrap_or(u32::MAX);
    Range {
        start: line_index.offset_to_position(text, start_u32),
        end: line_index.offset_to_position(text, end_u32),
    }
}

/// The whole-file range (`0..text.len()`), used for every File-tier
/// diagnostic and for the primary range of an Ambiguous/Absent Form-tier
/// outcome.
fn file_range(text: &str, line_index: &LineIndex) -> Range {
    byte_range_to_lsp_range(text, line_index, 0, text.len())
}

/// Map one [`Located`] refusal to a [`Diagnostic`] for `uri`/`text`,
/// locating it against `parsed` (this file's own `(forest, SpanTable)`,
/// when reading `text` succeeded — `None` when it did not, in which case
/// every Form-tier identity search degrades to file level by construction:
/// there is no tree left to search).
#[must_use]
pub fn map_to_diagnostic(
    uri: &Uri,
    text: &str,
    line_index: &LineIndex,
    parsed: Option<&(Vec<SExpr>, SpanTable)>,
    located: &Located,
) -> Diagnostic {
    let precision = precision_of(located);
    let direct_span = located
        .form_path
        .as_deref()
        .and_then(|path| parsed.and_then(|(_, spans)| spans.span_of(path)));
    let (range, related_information) =
        match (precision, direct_span, located.position, &located.identity) {
            (Precision::Exact, Some(span), _, _) => (
                byte_range_to_lsp_range(text, line_index, span.start, span.end),
                None,
            ),
            (Precision::Exact, None, Some(position), _) => {
                let (start, end) = token_span_at(text, position);
                (byte_range_to_lsp_range(text, line_index, start, end), None)
            }
            (Precision::Form, _, _, Some(identity)) => match parsed {
                Some((forest, spans)) => match locate(identity, forest, spans) {
                    LocateOutcome::Unique(span) => (
                        byte_range_to_lsp_range(text, line_index, span.start, span.end),
                        None,
                    ),
                    LocateOutcome::Ambiguous(candidates) => {
                        let related = candidates
                            .iter()
                            .map(|span| DiagnosticRelatedInformation {
                                location: Location {
                                    uri: uri.clone(),
                                    range: byte_range_to_lsp_range(
                                        text, line_index, span.start, span.end,
                                    ),
                                },
                                message: "another declaration of the same name".to_owned(),
                            })
                            .collect();
                        (file_range(text, line_index), Some(related))
                    }
                    LocateOutcome::Absent => (file_range(text, line_index), None),
                },
                None => (file_range(text, line_index), None),
            },
            _ => (file_range(text, line_index), None),
        };
    Diagnostic {
        range,
        severity: Some(located.severity),
        code: located.code.map(|c| NumberOrString::String(c.to_owned())),
        code_description: None,
        source: Some("bsl".to_owned()),
        message: located.message.clone(),
        related_information,
        tags: None,
        data: Some(serde_json::json!({
            "family": located.family,
            "precision": precision.as_str(),
        })),
    }
}

/// The server's own "no content set declares this file" notice (§6.1,
/// §6.3) — not a loader refusal, so it never goes through [`Located`]/
/// [`map_to_diagnostic`]: `source`/`severity` are fixed, there is no
/// `code`, and `data` carries no `precision` (this is not one of the
/// loader's own tiers).
#[must_use]
pub fn missing_manifest_row_diagnostic(
    text: &str,
    line_index: &LineIndex,
    path: &str,
) -> Diagnostic {
    Diagnostic {
        range: file_range(text, line_index),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: None,
        code_description: None,
        source: Some("bsl".to_owned()),
        message: format!(
            "{path} names no row in content-sets.toml — only declaration-independent \
             checks run (read, split_content, surface/:material-basis, static shape); \
             this is also a manifest-drift alarm"
        ),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// The declared total order (§6.2): `(range.start, range.end, code,
/// message)` — deterministic serialization independent of the order
/// `errors` arrived in.
#[must_use]
pub fn sort_key(d: &Diagnostic) -> (u32, u32, u32, u32, String, String) {
    let code = match &d.code {
        Some(NumberOrString::String(s)) => s.clone(),
        Some(NumberOrString::Number(n)) => n.to_string(),
        None => String::new(),
    };
    (
        d.range.start.line,
        d.range.start.character,
        d.range.end.line,
        d.range.end.character,
        code,
        d.message.clone(),
    )
}

/// Build the full `Vec<Diagnostic>` for one file: locate every `located`
/// refusal against `text`'s own `(forest, SpanTable)` (re-read once, here
/// — `babylon-bsl`'s own reader, per the global constraint that this
/// crate owns no parsing of its own), and sort into the declared total
/// order. Bounded by `located.len()` (Power-of-10 rule 2).
#[must_use]
pub fn diagnostics_for_file(
    uri: &Uri,
    text: &str,
    line_index: &LineIndex,
    located: &[Located],
) -> Vec<Diagnostic> {
    let parsed = read_all_spanned(text.as_bytes()).ok();
    let mut diagnostics: Vec<Diagnostic> = located
        .iter()
        .map(|loc| map_to_diagnostic(uri, text, line_index, parsed.as_ref(), loc))
        .collect();
    diagnostics.sort_by_key(sort_key);
    diagnostics
}

/// `resultId` (§6.1): `sha256` over the ordered `(uri, bytes)` tuples of a
/// content set plus the manifest bytes — deterministic across restarts (a
/// monotonic counter would have been state, and wrong). `entries` must
/// already be caller-sorted by `uri` (this function does not sort — the
/// caller controls the ordering source, e.g. a `BTreeMap` walk, keeping
/// the sort visible at the call site rather than hidden in here).
#[must_use]
pub fn compute_result_id(entries: &[(&Uri, &[u8])], manifest_bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let mut hasher = Sha256::new();
    for (uri, bytes) in entries {
        hasher.update(uri.as_str().as_bytes());
        hasher.update([0u8]); // a NUL separator: no valid URI byte is NUL.
        hasher.update(bytes);
        hasher.update([0u8]);
    }
    hasher.update(manifest_bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        compute_result_id, diagnostics_for_file, family_from_code, family_of_load_error,
        family_of_scenario_error, missing_manifest_row_diagnostic, precision_of, token_span_at,
        Located, Precision,
    };
    use crate::line_index::LineIndex;
    use babylon_bsl::rule_pipeline::LoadError;
    use babylon_bsl::scenario::{load_scenario, ScenarioError};
    use babylon_bsl::{read, DeclError, ErrorIdentity};
    use babylon_graph::hypergraph_store::HypergraphStore;
    use lsp_types::{DiagnosticSeverity, Uri};

    fn uri() -> Uri {
        "file:///a.bsl".parse::<Uri>().expect("valid test URI")
    }

    #[test]
    fn precision_exact_when_position_is_set() {
        let located = Located {
            code: Some("E-LEX-003"),
            family: "E-LEX",
            identity: None,
            position: Some(3),
            form_path: None,
            message: "bad token".to_owned(),
            severity: DiagnosticSeverity::ERROR,
        };
        assert_eq!(precision_of(&located), Precision::Exact);
    }

    #[test]
    fn precision_form_when_identity_is_set_and_position_is_not() {
        let located = Located {
            code: Some("E-LOAD-001"),
            family: "E-LOAD",
            identity: Some(ErrorIdentity::Name("foo".to_owned())),
            position: None,
            form_path: None,
            message: "duplicate".to_owned(),
            severity: DiagnosticSeverity::ERROR,
        };
        assert_eq!(precision_of(&located), Precision::Form);
    }

    #[test]
    fn precision_file_when_neither_is_set() {
        let located = Located {
            code: None,
            family: "E-PARSE",
            identity: None,
            position: None,
            form_path: None,
            message: "malformed".to_owned(),
            severity: DiagnosticSeverity::ERROR,
        };
        assert_eq!(precision_of(&located), Precision::File);
    }

    #[test]
    fn loader_probability_form_path_maps_to_the_exact_form_span() {
        let text = "(foo bar)";
        let line_index = LineIndex::new(text);
        let error = LoadError::Probability(babylon_bsl::ProbabilityError::InvalidForm {
            message: "probability refusal".to_owned(),
            form_path: vec![0, 1],
        });
        let located = Located::from_load_error(&error);
        let diagnostics = diagnostics_for_file(&uri(), text, &line_index, &[located]);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start.character, 5);
        assert_eq!(diagnostics[0].range.end.character, 8);
        assert_eq!(
            diagnostics[0].data,
            Some(serde_json::json!({"family": "E-LOAD", "precision": "exact"}))
        );
    }

    // ---------------------------------------------------------- family_*

    #[test]
    fn family_from_code_splits_at_the_last_dash() {
        assert_eq!(family_from_code("E-LEX-003"), "E-LEX");
        assert_eq!(family_from_code("E-PARSE-020"), "E-PARSE");
        assert_eq!(family_from_code("E-LOAD-001"), "E-LOAD");
        assert_eq!(family_from_code("E-TYPE-010"), "E-TYPE");
    }

    #[test]
    fn family_of_load_error_reads_the_coded_case_off_its_own_code() {
        // `E-LOAD-001`: `DeclError::Duplicate`, wrapped as `LoadError::Intrinsic`.
        let err = LoadError::Intrinsic(DeclError::Duplicate {
            name: "floor".to_owned(),
            what: "intrinsic",
        });
        assert_eq!(family_of_load_error(&err), "E-LOAD");
    }

    #[test]
    fn family_of_load_error_classifies_the_uncoded_when_case_e_parse() {
        // `BoundError::EmptyWhenCondition` is now coded (`E-PARSE-020`,
        // #652 Task 6.2) — this row instead exercises a genuinely UNCODED
        // `BoundError::Malformed` (missing `:fuel`), the same shape-level
        // family every bare `Malformed` variant classifies.
        let rule = babylon_bsl::read(
            "(rule demo/no-fuel :role mechanic :evidence derived :material-basis \"x\" (bindings) \
             (effects (update-node self social-class/agitation (add 0.05i))))",
        )
        .expect("must parse")
        .0;
        let err = babylon_bsl::check_rule(
            &rule,
            &babylon_bsl::CardinalityCeilings::default(),
            &babylon_bsl::IntrinsicCosts::default(),
        )
        .expect_err("missing :fuel must be rejected");
        let load_err = LoadError::Bound(err);
        assert_eq!(load_err.spec_code(), None);
        assert_eq!(family_of_load_error(&load_err), "E-PARSE");
    }

    #[test]
    fn family_of_load_error_lexical_is_e_lex_even_when_uncoded() {
        // An unterminated list: `ReadErrorKind::UnterminatedList`, no
        // `LexCode` at all — uncoded, but still the reader's own stage.
        let err = babylon_bsl::read("(rule foo").expect_err("must be a read error");
        let load_err = LoadError::Read(err);
        assert_eq!(load_err.spec_code(), None);
        assert_eq!(family_of_load_error(&load_err), "E-LEX");
    }

    #[test]
    fn causal_metadata_and_authorization_keep_their_parse_and_load_families() {
        let missing = LoadError::Causal(
            babylon_bsl::causal_contract::ContractError::MissingMetadata { keyword: "role" },
        );
        assert_eq!(missing.spec_code(), None);
        assert_eq!(family_of_load_error(&missing), "E-PARSE");

        let unauthorized = LoadError::Causal(
            babylon_bsl::causal_contract::ContractError::UnauthorizedEffect {
                rule_id: "vitality/probe".to_owned(),
                role: babylon_bsl::causal_contract::RuleRole::ExternalEvent,
                effect: babylon_bsl::causal_contract::EffectSignature::NodeField(
                    "social-class/deaths".to_owned(),
                ),
            },
        );
        assert_eq!(unauthorized.spec_code(), Some("E-LOAD-060"));
        assert_eq!(family_of_load_error(&unauthorized), "E-LOAD");

        let mismatch = LoadError::Causal(
            babylon_bsl::causal_contract::ContractError::GovernedAttributionMismatch {
                rule_id: "control-ratio/c03-crisis".to_owned(),
                expected_role: babylon_bsl::causal_contract::RuleRole::Recognizer,
                actual_role: babylon_bsl::causal_contract::RuleRole::Mechanic,
                expected_evidence: babylon_bsl::causal_contract::EvidenceClass::Derived,
                actual_evidence: babylon_bsl::causal_contract::EvidenceClass::Derived,
            },
        );
        assert_eq!(mismatch.spec_code(), None);
        assert_eq!(family_of_load_error(&mismatch), "E-LOAD");

        let paired_with_wrong_contract = LoadError::Causal(
            babylon_bsl::causal_contract::ContractError::MismatchedRuleContract {
                ast_contract: babylon_bsl::causal_contract::RuleContract {
                    rule_id: "vitality/probe".to_owned(),
                    role: babylon_bsl::causal_contract::RuleRole::ExternalEvent,
                    evidence: babylon_bsl::causal_contract::EvidenceClass::Designed,
                },
                supplied_contract: babylon_bsl::causal_contract::RuleContract {
                    rule_id: "control-ratio/c03-crisis".to_owned(),
                    role: babylon_bsl::causal_contract::RuleRole::Recognizer,
                    evidence: babylon_bsl::causal_contract::EvidenceClass::Derived,
                },
            },
        );
        assert_eq!(paired_with_wrong_contract.spec_code(), None);
        assert_eq!(family_of_load_error(&paired_with_wrong_contract), "E-LOAD");

        let bounded_walk =
            LoadError::Causal(babylon_bsl::causal_contract::ContractError::AstWalkLimit(
                babylon_bsl::causal_contract::AstWalkError {
                    analyzer: "causal effect footprint",
                    limit: babylon_bsl::causal_contract::AstWalkLimit::Depth,
                    maximum: 256,
                },
            ));
        assert_eq!(bounded_walk.spec_code(), None);
        assert_eq!(family_of_load_error(&bounded_walk), "E-LOAD");

        let ordinal = LoadError::Causal(
            babylon_bsl::causal_contract::ContractError::MismatchedWriteOrdinal {
                expected: 0,
                actual: 1,
            },
        );
        assert_eq!(ordinal.spec_code(), None);
        assert_eq!(family_of_load_error(&ordinal), "E-LOAD");
    }

    #[test]
    fn family_of_scenario_error_is_e_lex_when_position_is_set() {
        let mut graph = HypergraphStore::new();
        let err = load_scenario("(scenario ft/bad (defconst", &mut graph)
            .expect_err("must be a read error routed through ScenarioError");
        assert!(err.position.is_some(), "{err:?}");
        assert_eq!(family_of_scenario_error(&err), "E-LEX");
    }

    #[test]
    fn family_of_scenario_error_defaults_e_load_when_uncoded_and_positionless() {
        let err = ScenarioError {
            message: "expected (defconst <qname> <literal>)".to_owned(),
            code: None,
            position: None,
            identity: None,
        };
        assert_eq!(family_of_scenario_error(&err), "E-LOAD");
    }

    #[test]
    fn token_span_at_expands_to_the_enclosing_word() {
        let text = "(rule bogus-token)";
        let (start, end) = token_span_at(text, 8); // inside "bogus-token"
        assert_eq!(&text[start..end], "bogus-token");
    }

    #[test]
    fn token_span_at_a_delimiter_is_one_byte() {
        let text = "(rule)";
        let (start, end) = token_span_at(text, 0); // the '(' itself
        assert_eq!(&text[start..end], "(");
    }

    #[test]
    fn a_real_lexical_error_position_locates_the_offending_token() {
        // `~=` is not a valid comparison operator — `ReadErrorKind::Lex
        // (LexCode::UnclassifiableToken)`, `E-LEX-003`, the same vector
        // `conformance_corpus.rs`'s own correction-3 test uses.
        let err = read("(~= agitation 0.5p)").expect_err("must be a read error");
        let located = Located {
            code: Some(match &err.kind {
                babylon_bsl::ReadErrorKind::Lex(code) => code.spec_code(),
                _ => unreachable!("this fixture is a lex error"),
            }),
            family: "E-LEX",
            identity: None,
            position: Some(err.position),
            form_path: None,
            message: err.message.clone(),
            severity: DiagnosticSeverity::ERROR,
        };
        assert_eq!(precision_of(&located), Precision::Exact);
        assert_eq!(located.family, "E-LEX");
        let (start, end) = token_span_at("(~= agitation 0.5p)", located.position.unwrap());
        assert_eq!(&"(~= agitation 0.5p)"[start..end], "~=");
    }

    #[test]
    fn missing_manifest_row_is_information_severity_file_level_no_code() {
        let text = "(rule x (bindings) (effects))";
        let line_index = LineIndex::new(text);
        let d = missing_manifest_row_diagnostic(text, &line_index, "orphan.bsl");
        assert_eq!(d.severity, Some(DiagnosticSeverity::INFORMATION));
        assert_eq!(d.code, None);
        assert!(d.message.contains("orphan.bsl"));
        assert_eq!(d.range.start.line, 0);
    }

    #[test]
    fn diagnostics_for_file_sorts_into_the_declared_total_order() {
        let text = "(intrinsic floor :params (real) :returns int :cost 5) \
                    (intrinsic floor :params (real) :returns int :cost 6) \
                    (rule event/probe :role mechanic :evidence derived :material-basis \"x\" :fuel 16 (bindings) (effects (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))";
        let line_index = LineIndex::new(text);
        let located = vec![
            Located {
                code: Some("E-LOAD-001"),
                family: "E-LOAD",
                identity: Some(ErrorIdentity::Name("floor".to_owned())),
                position: None,
                form_path: None,
                message: "duplicate intrinsic".to_owned(),
                severity: DiagnosticSeverity::ERROR,
            },
            Located {
                code: None,
                family: "E-PARSE",
                identity: None,
                position: None,
                form_path: None,
                message: "aaa file-level comes first alphabetically among equal ranges".to_owned(),
                severity: DiagnosticSeverity::ERROR,
            },
        ];
        let diags = diagnostics_for_file(&uri(), text, &line_index, &located);
        assert_eq!(diags.len(), 2);
        // Both fall back to file-level (ambiguous "floor" + File-tier
        // prose) — the SAME range — so the total order's `message`
        // tiebreak decides, and "aaa..." < "duplicate intrinsic".
        assert!(diags[0].message.starts_with("aaa"));
        assert_eq!(diags[0].data.as_ref().unwrap()["family"], "E-PARSE");
        assert_eq!(diags[1].data.as_ref().unwrap()["family"], "E-LOAD");
    }

    #[test]
    fn duplicate_intrinsic_locates_ambiguous_with_two_related_information_entries() {
        let text = "(intrinsic floor :params (real) :returns int :cost 5) \
                    (intrinsic floor :params (real) :returns int :cost 6) \
                    (rule event/probe :role mechanic :evidence derived :material-basis \"x\" :fuel 16 (bindings) (effects (emit EventType/CONSCIOUSNESS_SHIFT (gate 0))))";
        let line_index = LineIndex::new(text);
        let load_err = LoadError::Intrinsic(babylon_bsl::DeclError::Duplicate {
            name: "floor".to_owned(),
            what: "intrinsic",
        });
        let located = Located::from_load_error(&load_err);
        assert_eq!(located.code, Some("E-LOAD-001"));
        assert_eq!(located.family, "E-LOAD");
        let diags = diagnostics_for_file(&uri(), text, &line_index, std::slice::from_ref(&located));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].data.as_ref().unwrap()["family"], "E-LOAD");
        let related = diags[0]
            .related_information
            .as_ref()
            .expect("ambiguous outcome carries relatedInformation");
        assert_eq!(related.len(), 2);
        // "the second declaration": relatedInformation[1] is the later
        // (higher-offset) `floor` occurrence, sorted into document order.
        assert!(
            related[0].location.range.start.character < related[1].location.range.start.character
        );
    }

    #[test]
    fn compute_result_id_is_deterministic_and_order_sensitive() {
        let uri_a = "file:///a.bsl".parse::<Uri>().unwrap();
        let uri_b = "file:///b.bsl".parse::<Uri>().unwrap();
        let manifest = b"schema = 2";
        let ab = compute_result_id(&[(&uri_a, b"one"), (&uri_b, b"two")], manifest);
        let ab_again = compute_result_id(&[(&uri_a, b"one"), (&uri_b, b"two")], manifest);
        let ba = compute_result_id(&[(&uri_b, b"two"), (&uri_a, b"one")], manifest);
        assert_eq!(ab, ab_again);
        assert_ne!(ab, ba, "entry order is part of the hashed contract");
    }
}
