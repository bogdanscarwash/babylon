//! Probability-focused authoring responses derived from loader-owned facts.
//!
//! Exact facts come from an [`AuthoringSnapshot`] translated from
//! `babylon-bsl`'s typed content-set analysis plus the loader reader's
//! `SpanTable`. Completion and incomplete-form signature help additionally use
//! a bounded syntax-only prefix scan. That scan never invents probability
//! values, linkage, or validity; it only identifies an ordinary edit context.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, Hover, HoverContents, InsertTextFormat,
    MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, Range, SemanticToken,
    SemanticTokens, SignatureHelp, SignatureInformation,
};

use crate::capabilities::{
    TOKEN_TYPE_ENUM_MEMBER, TOKEN_TYPE_FUNCTION, TOKEN_TYPE_KEYWORD, TOKEN_TYPE_NUMBER,
    TOKEN_TYPE_VARIABLE,
};
use crate::line_index::LineIndex;

/// One byte range in the current source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

impl ByteSpan {
    fn contains(self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    fn contains_cursor(self, offset: usize) -> bool {
        self.start <= offset && offset <= self.end
    }
}

/// One exact executable branch allocation, supplied by the loader analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchAllocationFact {
    pub outcome: String,
    pub mass_nanounits: u64,
    pub ticket_start: u128,
    pub ticket_end: u128,
    pub ticket_count: u128,
}

/// Loader-owned availability of one kernel's executable allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationFact {
    Exact(Vec<BranchAllocationFact>),
    Unavailable {
        reason: String,
    },
    /// The same written kernel resolves to different executable allocations
    /// through multiple content sets. No manifest-order result is selected.
    Ambiguous {
        content_sets: Vec<String>,
    },
}

/// One exactly enumerable event likelihood, supplied by forecasting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLikelihoodFact {
    pub event_type: String,
    pub favorable_outcomes: Vec<String>,
    pub numerator: u128,
    pub denominator: u128,
}

/// Loader/executor stage that refused a scenario-determined forecast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastRefusalStage {
    Preparation,
    Execution,
}

/// Loader-owned availability of exact finite projection likelihoods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventLikelihoodAnalysisFact {
    Exact(Vec<EventLikelihoodFact>),
    StateDependent {
        reason: String,
    },
    /// A paired scenario determined an instance, but the real forecast path
    /// refused to prepare or execute it. This is not state dependence.
    Refused {
        stage: ForecastRefusalStage,
        reason: String,
    },
    /// The same source projection resolves through content sets whose
    /// analyses disagree. No manifest-order result is selected as truth.
    Ambiguous {
        content_sets: Vec<String>,
    },
}

/// One validated adjacent kernel/projection schedule link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelProjectionFact {
    pub kernel_rule_id: String,
    pub projection_rule_id: String,
}

/// Loader-owned semantic fact attached to one written token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoringKind {
    Mass {
        nanounits: u64,
    },
    Choose {
        sample: String,
        allocation: Option<AllocationFact>,
        linkage: Option<KernelProjectionFact>,
        argument_spans: Vec<ByteSpan>,
    },
    Branch {
        outcome: String,
        mass_nanounits: Option<u64>,
        tickets: Option<(u128, u128, u128)>,
        allocation_ambiguity: Option<Vec<String>>,
        argument_spans: Vec<ByteSpan>,
    },
    QuantizeMass {
        argument_spans: Vec<ByteSpan>,
    },
    ProjectsKernel {
        sample: String,
        linkage: Option<KernelProjectionFact>,
        likelihood: Option<EventLikelihoodAnalysisFact>,
        argument_spans: Vec<ByteSpan>,
    },
    Keyword,
    Number,
    EnumMember,
    Sample,
}

/// One typed fact's token range and, for call forms, enclosing form range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringFact {
    pub token_span: ByteSpan,
    pub form_span: ByteSpan,
    pub kind: AuthoringKind,
}

/// Deterministically ordered probability facts for one current source buffer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthoringSnapshot {
    pub facts: Vec<AuthoringFact>,
}

impl AuthoringSnapshot {
    /// Sorts loader facts into the stable order every response consumes.
    #[must_use]
    pub fn new(mut facts: Vec<AuthoringFact>) -> Self {
        facts.sort_by_key(|fact| {
            (
                fact.token_span.start,
                fact.token_span.end,
                semantic_type(&fact.kind),
            )
        });
        facts.dedup();
        Self { facts }
    }

    fn token_at(&self, offset: usize) -> Option<&AuthoringFact> {
        self.facts
            .iter()
            .filter(|fact| fact.token_span.contains(offset))
            .min_by_key(|fact| fact.token_span.end.saturating_sub(fact.token_span.start))
    }

    fn innermost_call_at(&self, offset: usize) -> Option<&AuthoringFact> {
        self.facts
            .iter()
            .filter(|fact| {
                fact.form_span.contains_cursor(offset)
                    && matches!(
                        fact.kind,
                        AuthoringKind::Choose { .. }
                            | AuthoringKind::Branch { .. }
                            | AuthoringKind::QuantizeMass { .. }
                            | AuthoringKind::ProjectsKernel { .. }
                    )
            })
            .min_by_key(|fact| fact.form_span.end.saturating_sub(fact.form_span.start))
    }
}

/// Merge loader snapshots for one source shared by one or more content sets.
/// Identical facts collapse. Conflicting same-span kernel allocations and
/// projection analyses become one explicit, set-identified ambiguity,
/// independent of manifest row order.
#[must_use]
pub fn merge_content_set_snapshots(
    snapshots: impl IntoIterator<Item = (String, AuthoringSnapshot)>,
) -> AuthoringSnapshot {
    type FactKey = (ByteSpan, ByteSpan, String);
    type Candidates = Vec<(String, AuthoringFact)>;

    let mut facts = Vec::new();
    let mut choices: BTreeMap<FactKey, Candidates> = BTreeMap::new();
    let mut branches: BTreeMap<FactKey, Candidates> = BTreeMap::new();
    let mut projections: BTreeMap<FactKey, Candidates> = BTreeMap::new();
    for (content_set, snapshot) in snapshots {
        for fact in snapshot.facts {
            match &fact.kind {
                AuthoringKind::Choose { sample, .. } => choices
                    .entry((fact.token_span, fact.form_span, sample.clone()))
                    .or_default()
                    .push((content_set.clone(), fact)),
                AuthoringKind::Branch { outcome, .. } => branches
                    .entry((fact.token_span, fact.form_span, outcome.clone()))
                    .or_default()
                    .push((content_set.clone(), fact)),
                AuthoringKind::ProjectsKernel { sample, .. } => projections
                    .entry((fact.token_span, fact.form_span, sample.clone()))
                    .or_default()
                    .push((content_set.clone(), fact)),
                _ => facts.push(fact),
            }
        }
    }

    facts.extend(choices.into_values().filter_map(merge_choose_candidates));
    facts.extend(branches.into_values().filter_map(merge_branch_candidates));
    facts.extend(
        projections
            .into_values()
            .filter_map(merge_projection_candidates),
    );
    AuthoringSnapshot::new(facts)
}

fn merge_choose_candidates(mut candidates: Vec<(String, AuthoringFact)>) -> Option<AuthoringFact> {
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    let allocations_identical = candidates.windows(2).all(|pair| {
        let AuthoringKind::Choose {
            allocation: left, ..
        } = &pair[0].1.kind
        else {
            unreachable!("choose map contains only choose facts")
        };
        let AuthoringKind::Choose {
            allocation: right, ..
        } = &pair[1].1.kind
        else {
            unreachable!("choose map contains only choose facts")
        };
        left == right
    });
    let linkages_identical = candidates.windows(2).all(|pair| {
        let AuthoringKind::Choose { linkage: left, .. } = &pair[0].1.kind else {
            unreachable!("choose map contains only choose facts")
        };
        let AuthoringKind::Choose { linkage: right, .. } = &pair[1].1.kind else {
            unreachable!("choose map contains only choose facts")
        };
        left == right
    });
    let content_sets = candidate_content_sets(&candidates);
    let (_, mut selected) = candidates.into_iter().next()?;
    if let AuthoringKind::Choose {
        allocation,
        linkage,
        ..
    } = &mut selected.kind
    {
        if !allocations_identical {
            *allocation = Some(AllocationFact::Ambiguous { content_sets });
        }
        if !linkages_identical {
            *linkage = None;
        }
    }
    Some(selected)
}

fn merge_branch_candidates(mut candidates: Vec<(String, AuthoringFact)>) -> Option<AuthoringFact> {
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    let allocations_identical = candidates.windows(2).all(|pair| {
        let AuthoringKind::Branch {
            mass_nanounits: left_mass,
            tickets: left_tickets,
            allocation_ambiguity: left_ambiguity,
            ..
        } = &pair[0].1.kind
        else {
            unreachable!("branch map contains only branch facts")
        };
        let AuthoringKind::Branch {
            mass_nanounits: right_mass,
            tickets: right_tickets,
            allocation_ambiguity: right_ambiguity,
            ..
        } = &pair[1].1.kind
        else {
            unreachable!("branch map contains only branch facts")
        };
        (left_mass, left_tickets, left_ambiguity) == (right_mass, right_tickets, right_ambiguity)
    });
    let content_sets = candidate_content_sets(&candidates);
    let (_, mut selected) = candidates.into_iter().next()?;
    if !allocations_identical {
        if let AuthoringKind::Branch {
            mass_nanounits,
            tickets,
            allocation_ambiguity,
            ..
        } = &mut selected.kind
        {
            *mass_nanounits = None;
            *tickets = None;
            *allocation_ambiguity = Some(content_sets);
        }
    }
    Some(selected)
}

fn merge_projection_candidates(
    mut candidates: Vec<(String, AuthoringFact)>,
) -> Option<AuthoringFact> {
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    let all_identical = candidates
        .windows(2)
        .all(|pair| pair[0].1.kind == pair[1].1.kind);
    let mut selected = candidates.first()?.1.clone();
    if !all_identical {
        let content_sets = candidate_content_sets(&candidates);
        if let AuthoringKind::ProjectsKernel {
            linkage,
            likelihood,
            ..
        } = &mut selected.kind
        {
            *linkage = None;
            *likelihood = Some(EventLikelihoodAnalysisFact::Ambiguous { content_sets });
        }
    }
    Some(selected)
}

fn candidate_content_sets(candidates: &[(String, AuthoringFact)]) -> Vec<String> {
    let mut content_sets = candidates
        .iter()
        .map(|(content_set, _)| content_set.clone())
        .collect::<Vec<_>>();
    content_sets.sort();
    content_sets.dedup();
    content_sets
}

fn path_child(path: &[u32], index: u32) -> Vec<u32> {
    let mut child = path.to_vec();
    child.push(index);
    child
}

fn byte_span(spans: &babylon_bsl::SpanTable, path: &[u32]) -> Option<ByteSpan> {
    spans.span_of(path).map(|span| ByteSpan {
        start: span.start,
        end: span.end,
    })
}

fn fact(
    spans: &babylon_bsl::SpanTable,
    token_path: &[u32],
    form_path: &[u32],
    kind: AuthoringKind,
) -> Option<AuthoringFact> {
    Some(AuthoringFact {
        token_span: byte_span(spans, token_path)?,
        form_span: byte_span(spans, form_path)?,
        kind,
    })
}

fn argument_spans(
    spans: &babylon_bsl::SpanTable,
    paths: impl IntoIterator<Item = Vec<u32>>,
) -> Vec<ByteSpan> {
    paths
        .into_iter()
        .filter_map(|path| byte_span(spans, &path))
        .collect()
}

fn allocation_fact(
    kernel: &babylon_bsl::probability::FiniteKernelV1,
    allocation: Option<&babylon_bsl::probability::AllocationAnalysisV1>,
) -> Option<AllocationFact> {
    match allocation? {
        babylon_bsl::probability::AllocationAnalysisV1::Exact(exact) => {
            Some(AllocationFact::Exact(
                kernel
                    .branches
                    .iter()
                    .zip(&exact.masses)
                    .zip(&exact.intervals)
                    .map(|((branch, mass), interval)| BranchAllocationFact {
                        outcome: branch.member.clone(),
                        mass_nanounits: mass.nanounits(),
                        ticket_start: interval.start,
                        ticket_end: interval.end,
                        ticket_count: interval.count,
                    })
                    .collect(),
            ))
        }
        babylon_bsl::probability::AllocationAnalysisV1::Unavailable { reason } => {
            Some(AllocationFact::Unavailable {
                reason: reason.clone(),
            })
        }
    }
}

fn branch_facts(
    spans: &babylon_bsl::SpanTable,
    branch: &babylon_bsl::probability::KernelBranchV1,
    allocation: Option<&AllocationFact>,
) -> Vec<AuthoringFact> {
    let mut facts = Vec::new();
    let exact = match allocation {
        Some(AllocationFact::Exact(rows)) => Some(rows.as_slice()),
        Some(AllocationFact::Unavailable { .. } | AllocationFact::Ambiguous { .. }) | None => None,
    };
    let tickets = exact.and_then(|rows| {
        usize::try_from(branch.ordinal)
            .ok()
            .and_then(|ordinal| rows.get(ordinal))
            .map(|row| (row.ticket_start, row.ticket_end, row.ticket_count))
    });
    let arguments = argument_spans(
        spans,
        [
            path_child(&branch.form_path, 1),
            branch.mass_path.clone(),
            path_child(&branch.form_path, 4),
        ],
    );
    if let Some(value) = fact(
        spans,
        &branch.head_path,
        &branch.form_path,
        AuthoringKind::Branch {
            outcome: branch.member.clone(),
            mass_nanounits: branch.static_mass.map(babylon_bsl::Mass::nanounits),
            tickets,
            allocation_ambiguity: None,
            argument_spans: arguments,
        },
    ) {
        facts.push(value);
    }
    for (path, kind) in [
        (path_child(&branch.form_path, 1), AuthoringKind::EnumMember),
        (path_child(&branch.form_path, 2), AuthoringKind::Keyword),
    ] {
        if let Some(value) = fact(spans, &path, &path, kind) {
            facts.push(value);
        }
    }
    for literal in &branch.mass_literals {
        let kind = AuthoringKind::Mass {
            nanounits: literal.mass.nanounits(),
        };
        if let Some(value) = fact(spans, &literal.form_path, &literal.form_path, kind) {
            facts.push(value);
        }
    }
    for head_path in &branch.quantize_mass_paths {
        let mut form_path = head_path.clone();
        let _ = form_path.pop();
        let arguments = argument_spans(spans, [path_child(&form_path, 1)]);
        if let Some(value) = fact(
            spans,
            head_path,
            &form_path,
            AuthoringKind::QuantizeMass {
                argument_spans: arguments,
            },
        ) {
            facts.push(value);
        }
    }
    facts
}

fn kernel_facts(
    spans: &babylon_bsl::SpanTable,
    kernel: &babylon_bsl::probability::FiniteKernelV1,
    allocation: Option<&AllocationFact>,
    linkage: Option<KernelProjectionFact>,
) -> Vec<AuthoringFact> {
    let mut facts = Vec::new();
    let arguments = argument_spans(
        spans,
        [
            path_child(&kernel.form_path, 2),
            path_child(&kernel.form_path, 4),
            kernel.branches.first().map_or_else(
                || kernel.form_path.clone(),
                |branch| branch.form_path.clone(),
            ),
        ],
    );
    if let Some(value) = fact(
        spans,
        &kernel.head_path,
        &kernel.form_path,
        AuthoringKind::Choose {
            sample: kernel.sample.clone(),
            allocation: allocation.cloned(),
            linkage,
            argument_spans: arguments,
        },
    ) {
        facts.push(value);
    }
    for (path, kind) in [
        (path_child(&kernel.form_path, 1), AuthoringKind::Keyword),
        (path_child(&kernel.form_path, 2), AuthoringKind::Sample),
        (path_child(&kernel.form_path, 3), AuthoringKind::Keyword),
        (path_child(&kernel.form_path, 4), AuthoringKind::Number),
    ] {
        if let Some(value) = fact(spans, &path, &path, kind) {
            facts.push(value);
        }
    }
    for branch in &kernel.branches {
        facts.extend(branch_facts(spans, branch, allocation));
    }
    facts
}

/// Translate one loader-owned rule analysis into LSP authoring facts. This
/// function only resolves typed paths against the same source's span table.
#[must_use]
pub fn snapshot_from_rule_analysis(
    source: &str,
    analysis: &babylon_bsl::probability::RuleProbabilityAnalysisV1,
    linkage: Option<KernelProjectionFact>,
    projection_likelihood: Option<EventLikelihoodAnalysisFact>,
) -> AuthoringSnapshot {
    let Ok((_, spans)) = babylon_bsl::read_all_spanned(source.as_bytes()) else {
        return AuthoringSnapshot::default();
    };
    let mut facts = analysis.kernel.as_ref().map_or_else(Vec::new, |kernel| {
        let allocation = allocation_fact(kernel, analysis.allocation.as_ref());
        kernel_facts(&spans, kernel, allocation.as_ref(), linkage.clone())
    });
    if let Some(projection) = analysis.projection.as_ref() {
        if let (Some(token_span), Some(sample_span)) = (
            byte_span(&spans, &projection.form_path),
            byte_span(&spans, &projection.sample_path),
        ) {
            facts.push(AuthoringFact {
                token_span,
                form_span: ByteSpan {
                    start: token_span.start.min(sample_span.start),
                    end: token_span.end.max(sample_span.end),
                },
                kind: AuthoringKind::ProjectsKernel {
                    sample: projection.sample.clone(),
                    linkage,
                    likelihood: projection_likelihood,
                    argument_spans: vec![sample_span],
                },
            });
        }
        if let Some(value) = fact(
            &spans,
            &projection.sample_path,
            &projection.sample_path,
            AuthoringKind::Sample,
        ) {
            facts.push(value);
        }
    }
    for literal in &analysis.mass_literals {
        if let Some(value) = fact(
            &spans,
            &literal.form_path,
            &literal.form_path,
            AuthoringKind::Mass {
                nanounits: literal.mass.nanounits(),
            },
        ) {
            facts.push(value);
        }
    }
    for node in &analysis.nodes {
        if node.kind != babylon_bsl::probability::ProbabilityAnalysisNodeKindV1::QuantizeMass {
            continue;
        }
        let head_path = &node.form_path;
        let mut form_path = head_path.clone();
        let _ = form_path.pop();
        let arguments = argument_spans(&spans, [path_child(&form_path, 1)]);
        if let Some(value) = fact(
            &spans,
            head_path,
            &form_path,
            AuthoringKind::QuantizeMass {
                argument_spans: arguments,
            },
        ) {
            facts.push(value);
        }
    }
    AuthoringSnapshot::new(facts)
}

/// Render every analyzed rule originating in `source_id`, including its
/// validated adjacent kernel/projection linkage. Exact likelihoods are keyed
/// by projection rule identity; an absent row deliberately renders as
/// state-dependent rather than fabricating an approximation.
#[must_use]
pub fn snapshot_from_content_analysis(
    source_id: &str,
    source: &str,
    analysis: &babylon_bsl::probability::ContentSetAnalysisV1,
    likelihood_overrides: &[(String, EventLikelihoodAnalysisFact)],
) -> AuthoringSnapshot {
    let mut facts = Vec::new();
    if let Ok((_, spans)) = babylon_bsl::read_all_spanned(source.as_bytes()) {
        for declaration in analysis
            .mass_declarations
            .iter()
            .filter(|declaration| declaration.source_id == source_id)
        {
            if let Some(value) = fact(
                &spans,
                &declaration.form_path,
                &declaration.form_path,
                AuthoringKind::Mass {
                    nanounits: declaration.mass.nanounits(),
                },
            ) {
                facts.push(value);
            }
        }
    }
    for rule in analysis
        .rules
        .iter()
        .filter(|rule| rule.source_id == source_id)
    {
        let link = analysis.links.iter().find(|link| {
            link.kernel_rule_id == rule.rule_id || link.projection_rule_id == rule.rule_id
        });
        let linkage = link.map(|link| KernelProjectionFact {
            kernel_rule_id: link.kernel_rule_id.clone(),
            projection_rule_id: link.projection_rule_id.clone(),
        });
        let likelihood = likelihood_overrides
            .iter()
            .find(|(rule_id, _)| rule_id == &rule.rule_id)
            .map(|(_, likelihood)| likelihood.clone())
            .or_else(|| {
                link.map(|link| match &link.likelihood {
                    babylon_bsl::probability::LikelihoodAnalysisV1::Exact(rows) => {
                        EventLikelihoodAnalysisFact::Exact(
                            rows.iter()
                                .map(|row| EventLikelihoodFact {
                                    event_type: row.event_type.clone(),
                                    favorable_outcomes: row.favorable_outcomes.clone(),
                                    numerator: row.numerator,
                                    denominator: row.denominator,
                                })
                                .collect(),
                        )
                    }
                    babylon_bsl::probability::LikelihoodAnalysisV1::StateDependent { reason } => {
                        EventLikelihoodAnalysisFact::StateDependent {
                            reason: reason.clone(),
                        }
                    }
                })
            });
        facts.extend(snapshot_from_rule_analysis(source, rule, linkage, likelihood).facts);
    }
    AuthoringSnapshot::new(facts)
}

/// Canonical fixed-nine-decimal rendering of an exact Mass nanounit value.
#[must_use]
pub fn canonical_mass(nanounits: u64) -> String {
    let units = nanounits / babylon_bsl::MASS_NANOUNITS_PER_UNIT;
    let fractional = nanounits % babylon_bsl::MASS_NANOUNITS_PER_UNIT;
    format!("{units}.{fractional:09}m")
}

fn completion(label: &str, detail: &str, snippet: &str, order: u8) -> CompletionItem {
    CompletionItem {
        label: label.to_owned(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(detail.to_owned()),
        sort_text: Some(format!("{order:02}-{label}")),
        insert_text: Some(snippet.to_owned()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..CompletionItem::default()
    }
}

fn branch_completion(order: u8) -> CompletionItem {
    completion(
        "branch",
        "One enum-ordered finite-kernel alternative",
        "(branch ${1:OutcomeType/MEMBER} :mass ${2:0.000000000m}\n  (effects ${3}))",
        order,
    )
}

fn choose_completion(order: u8) -> CompletionItem {
    completion(
        "choose",
        "One direct finite material choice in a Mechanic rule",
        "(choose :sample ${1:sample/qname} :slot ${2:0}\n  ${3})",
        order,
    )
}

fn projects_kernel_completion(order: u8) -> CompletionItem {
    completion(
        ":projects-kernel",
        "Link an adjacent deterministic Recognizer to its kernel",
        ":projects-kernel ${1:sample/qname}",
        order,
    )
}

fn mass_completions(first_order: u8) -> Vec<CompletionItem> {
    vec![
        completion(
            "quantize-mass",
            "Explicit numeric-to-Mass conversion (nearest, ties to even)",
            "(quantize-mass ${1:numeric-expr})",
            first_order,
        ),
        completion(
            "0.000000000m",
            "Exact Mass literal at nanounit precision",
            "${1:0.000000000m}",
            first_order.saturating_add(1),
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EditItem {
    Atom(String),
    List(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EditFrame {
    items: Vec<EditItem>,
}

impl EditFrame {
    fn head(&self) -> Option<&str> {
        match self.items.first() {
            Some(EditItem::Atom(value)) => Some(value),
            Some(EditItem::List(_)) | None => None,
        }
    }

    fn has_atom(&self, expected: &str) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, EditItem::Atom(value) if value == expected))
    }

    fn has_list(&self, expected_head: &str) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, EditItem::List(head) if head == expected_head))
    }

    fn atom_after(&self, keyword: &str) -> Option<&str> {
        self.items
            .windows(2)
            .find_map(|pair| match (&pair[0], &pair[1]) {
                (EditItem::Atom(key), EditItem::Atom(value)) if key == keyword => {
                    Some(value.as_str())
                }
                _ => None,
            })
    }

    fn keyword_accepts_atom(&self, keyword: &str) -> bool {
        let Some(index) = self
            .items
            .iter()
            .position(|item| matches!(item, EditItem::Atom(value) if value == keyword))
        else {
            return false;
        };
        match self.items.get(index.saturating_add(1)) {
            None => true,
            Some(EditItem::Atom(_)) => self.items.len() == index.saturating_add(2),
            Some(EditItem::List(_)) => false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EditScan {
    frames: Vec<EditFrame>,
    in_string: bool,
    in_comment: bool,
}

fn finish_edit_atom(frames: &mut [EditFrame], atom: &mut String) {
    if atom.is_empty() {
        return;
    }
    if let Some(frame) = frames.last_mut() {
        frame.items.push(EditItem::Atom(std::mem::take(atom)));
    } else {
        atom.clear();
    }
}

fn scan_edit_prefix(text: &str, offset: usize) -> EditScan {
    let prefix = &text.as_bytes()[..offset.min(text.len())];
    let mut scan = EditScan::default();
    let mut atom = String::new();
    let mut escaped = false;

    for &byte in prefix {
        if scan.in_comment {
            if byte == b'\n' {
                scan.in_comment = false;
            }
            continue;
        }
        if scan.in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                scan.in_string = false;
                if let Some(frame) = scan.frames.last_mut() {
                    frame.items.push(EditItem::Atom("\"\"".to_owned()));
                }
            }
            continue;
        }

        match byte {
            b';' => {
                finish_edit_atom(&mut scan.frames, &mut atom);
                scan.in_comment = true;
            }
            b'"' => {
                finish_edit_atom(&mut scan.frames, &mut atom);
                scan.in_string = true;
            }
            b'(' => {
                finish_edit_atom(&mut scan.frames, &mut atom);
                scan.frames.push(EditFrame::default());
            }
            b')' => {
                finish_edit_atom(&mut scan.frames, &mut atom);
                if let Some(closed) = scan.frames.pop() {
                    if let Some(parent) = scan.frames.last_mut() {
                        parent
                            .items
                            .push(EditItem::List(closed.head().unwrap_or_default().to_owned()));
                    }
                }
            }
            value if value.is_ascii_whitespace() => {
                finish_edit_atom(&mut scan.frames, &mut atom);
            }
            value => atom.push(char::from(value)),
        }
    }
    if !scan.in_string && !scan.in_comment {
        finish_edit_atom(&mut scan.frames, &mut atom);
    }
    scan
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionContext {
    Suppressed,
    MechanicEffects,
    ChooseBranches,
    MassExpression,
    RecognizerMetadata,
}

fn rule_role(frames: &[EditFrame]) -> Option<&str> {
    frames
        .iter()
        .find(|frame| frame.head() == Some("rule"))
        .and_then(|frame| frame.atom_after(":role"))
}

fn completion_context(text: &str, offset: usize) -> CompletionContext {
    let scan = scan_edit_prefix(text, offset);
    if scan.in_string
        || scan.in_comment
        || scan
            .frames
            .iter()
            .any(|frame| frame.head() == Some("scenario"))
    {
        return CompletionContext::Suppressed;
    }
    let Some(current) = scan.frames.last() else {
        return CompletionContext::Suppressed;
    };
    match current.head() {
        Some("choose") => {
            if current.atom_after(":sample").is_some() && current.atom_after(":slot").is_some() {
                CompletionContext::ChooseBranches
            } else {
                CompletionContext::Suppressed
            }
        }
        Some("branch") => {
            if current.keyword_accepts_atom(":mass") && !current.has_list("effects") {
                CompletionContext::MassExpression
            } else {
                CompletionContext::Suppressed
            }
        }
        Some("binding") => {
            if current.keyword_accepts_atom(":expr") {
                CompletionContext::MassExpression
            } else {
                CompletionContext::Suppressed
            }
        }
        Some("effects") => {
            let parent_is_branch = scan
                .frames
                .iter()
                .rev()
                .nth(1)
                .is_some_and(|frame| frame.head() == Some("branch"));
            if !parent_is_branch
                && rule_role(&scan.frames) == Some("mechanic")
                && !current.has_list("choose")
            {
                CompletionContext::MechanicEffects
            } else {
                CompletionContext::Suppressed
            }
        }
        Some("rule") => {
            if rule_role(&scan.frames) == Some("recognizer")
                && !current.has_atom(":projects-kernel")
            {
                CompletionContext::RecognizerMetadata
            } else {
                CompletionContext::Suppressed
            }
        }
        Some(_) | None => CompletionContext::Suppressed,
    }
}

/// Contextual probability completions. A bounded syntax-only prefix scan
/// identifies ordinary edit positions even while the source is incomplete.
/// It never offers probability forms globally, inside strings/comments, in a
/// scenario, or inside an unrelated form.
#[must_use]
pub fn completion_items(
    text: &str,
    _snapshot: &AuthoringSnapshot,
    offset: usize,
) -> Vec<CompletionItem> {
    match completion_context(text, offset) {
        CompletionContext::MechanicEffects => vec![choose_completion(0)],
        CompletionContext::ChooseBranches => vec![branch_completion(0)],
        CompletionContext::MassExpression => mass_completions(0),
        CompletionContext::RecognizerMetadata => vec![projects_kernel_completion(0)],
        CompletionContext::Suppressed => Vec::new(),
    }
}

fn byte_span_to_range(text: &str, line_index: &LineIndex, span: ByteSpan) -> Option<Range> {
    let start = u32::try_from(span.start).ok()?;
    let end = u32::try_from(span.end).ok()?;
    Some(Range {
        start: line_index.offset_to_position(text, start),
        end: line_index.offset_to_position(text, end),
    })
}

fn allocation_markdown(
    sample: &str,
    allocation: Option<&AllocationFact>,
    linkage: Option<&KernelProjectionFact>,
) -> String {
    let linked = linkage.map_or_else(String::new, |link| {
        format!(
            "\n\nLinked schedule pair: `{}` -> `{}`.",
            link.kernel_rule_id, link.projection_rule_id
        )
    });
    let Some(allocation) = allocation else {
        return format!(
            "**Finite material kernel** `{sample}`{linked}\n\nExecutable ticket allocation is unavailable from the loader analysis."
        );
    };
    let allocation = match allocation {
        AllocationFact::Exact(allocation) => allocation,
        AllocationFact::Unavailable { reason } => {
            return format!(
                "**Finite material kernel** `{sample}`{linked}\n\nExact ticket allocation is **unavailable at load time**: {reason}."
            );
        }
        AllocationFact::Ambiguous { content_sets } => {
            let sets = content_sets
                .iter()
                .map(|content_set| format!("`{content_set}`"))
                .collect::<Vec<_>>()
                .join(", ");
            return format!(
                "**Finite material kernel** `{sample}`{linked}\n\nExecutable ticket allocation is **ambiguous across content sets** {sets}; no manifest-order result was selected."
            );
        }
    };
    let mut value = format!(
        "**Finite material kernel** `{sample}`{linked}\n\nExecutable allocation over `{}` tickets:",
        babylon_bsl::TICKET_DENOMINATOR
    );
    for branch in allocation {
        let _ = write!(
            value,
            "\n\n- `{}`: `{}` -> `[{}, {})` (`{}` tickets)",
            branch.outcome,
            canonical_mass(branch.mass_nanounits),
            branch.ticket_start,
            branch.ticket_end,
            branch.ticket_count
        );
    }
    value
}

fn projection_markdown(sample: &str, likelihood: Option<&EventLikelihoodAnalysisFact>) -> String {
    let Some(likelihood) = likelihood else {
        return format!(
            "**Finite projection** of `{sample}`\n\nNo paired likelihood analysis is available."
        );
    };
    let likelihoods = match likelihood {
        EventLikelihoodAnalysisFact::Exact(likelihoods) => likelihoods,
        EventLikelihoodAnalysisFact::StateDependent { reason } => {
            return format!(
                "**Finite projection** of `{sample}`\n\nExact event likelihood is **state-dependent**: {reason}."
            );
        }
        EventLikelihoodAnalysisFact::Refused { stage, reason } => {
            let stage = match stage {
                ForecastRefusalStage::Preparation => "preparation",
                ForecastRefusalStage::Execution => "execution",
            };
            return format!(
                "**Finite projection** of `{sample}`\n\nExact event likelihood forecast was **refused during {stage}**: {reason}."
            );
        }
        EventLikelihoodAnalysisFact::Ambiguous { content_sets } => {
            let sets = content_sets
                .iter()
                .map(|content_set| format!("`{content_set}`"))
                .collect::<Vec<_>>()
                .join(", ");
            return format!(
                "**Finite projection** of `{sample}`\n\nExact event likelihood is **ambiguous across content sets** {sets}; no manifest-order result was selected."
            );
        }
    };
    let mut value = format!("**Finite projection** of `{sample}`");
    if likelihoods.is_empty() {
        value.push_str("\n\nExact detached forecasting produced no projected event rows.");
        return value;
    }
    for likelihood in likelihoods {
        let favorable = if likelihood.favorable_outcomes.is_empty() {
            "none".to_owned()
        } else {
            likelihood.favorable_outcomes.join(", ")
        };
        let _ = write!(
            value,
            "\n\n- `{}`: `{}/{}`; favorable outcomes: `{favorable}`",
            likelihood.event_type, likelihood.numerator, likelihood.denominator
        );
    }
    value
}

/// Hover for the typed probability token under `offset`.
#[must_use]
pub fn hover(
    text: &str,
    line_index: &LineIndex,
    snapshot: &AuthoringSnapshot,
    offset: usize,
) -> Option<Hover> {
    let fact = snapshot
        .token_at(offset)
        .filter(|fact| {
            matches!(
                fact.kind,
                AuthoringKind::Mass { .. }
                    | AuthoringKind::Choose { .. }
                    | AuthoringKind::Branch { .. }
                    | AuthoringKind::QuantizeMass { .. }
                    | AuthoringKind::ProjectsKernel { .. }
            )
        })
        .or_else(|| snapshot.innermost_call_at(offset))?;
    let value = match &fact.kind {
        AuthoringKind::Mass { nanounits } => format!(
            "**Mass** `{}`\n\nExact value: `{nanounits}` nanounits.",
            canonical_mass(*nanounits)
        ),
        AuthoringKind::Choose {
            sample,
            allocation,
            linkage,
            ..
        } => allocation_markdown(sample, allocation.as_ref(), linkage.as_ref()),
        AuthoringKind::Branch {
            outcome,
            mass_nanounits,
            tickets,
            allocation_ambiguity,
            ..
        } => {
            if let Some(content_sets) = allocation_ambiguity {
                let sets = content_sets
                    .iter()
                    .map(|content_set| format!("`{content_set}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "**Kernel branch** `{outcome}`\n\nMass and ticket allocation are **ambiguous across content sets** {sets}; no manifest-order result was selected."
                )
            } else {
                match (mass_nanounits, tickets) {
                    (Some(mass), Some((start, end, count))) => format!(
                        "**Kernel branch** `{outcome}`\n\nExact mass `{}`; exact tickets `[{start}, {end})` (`{count}` tickets).",
                        canonical_mass(*mass)
                    ),
                    (Some(mass), None) => format!(
                        "**Kernel branch** `{outcome}`\n\nExact static mass `{}`; ticket interval is **not statically determined**.",
                        canonical_mass(*mass)
                    ),
                    (None, Some((start, end, count))) => format!(
                        "**Kernel branch** `{outcome}`\n\nMass expression is **not statically determined**; exact tickets `[{start}, {end})` (`{count}` tickets)."
                    ),
                    (None, None) => format!(
                        "**Kernel branch** `{outcome}`\n\nMass expression and ticket interval are **not statically determined**."
                    ),
                }
            }
        }
        AuthoringKind::QuantizeMass { .. } => "**`quantize-mass`**\n\nThe sole explicit numeric-to-Mass conversion. It rounds to the nearest nanounit, ties to even, and refuses negative or non-finite input.".to_owned(),
        AuthoringKind::ProjectsKernel {
            sample,
            linkage,
            likelihood,
            ..
        } => {
            let mut value = projection_markdown(sample, likelihood.as_ref());
            if let Some(link) = linkage {
                let _ = write!(
                    value,
                    "\n\nLinked schedule pair: `{}` -> `{}`.",
                    link.kernel_rule_id, link.projection_rule_id
                );
            }
            value
        }
        AuthoringKind::Keyword
        | AuthoringKind::Number
        | AuthoringKind::EnumMember
        | AuthoringKind::Sample => return None,
    };
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: byte_span_to_range(text, line_index, fact.token_span),
    })
}

fn signature(
    label: &str,
    documentation: &str,
    parameters: &[&str],
    active_parameter: u32,
) -> SignatureHelp {
    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: label.to_owned(),
            documentation: Some(Documentation::String(documentation.to_owned())),
            parameters: Some(
                parameters
                    .iter()
                    .map(|parameter| ParameterInformation {
                        label: ParameterLabel::Simple((*parameter).to_owned()),
                        documentation: None,
                    })
                    .collect(),
            ),
            active_parameter: Some(active_parameter),
        }],
        active_signature: Some(0),
        active_parameter: Some(active_parameter),
    }
}

fn active_parameter(argument_spans: &[ByteSpan], offset: usize) -> u32 {
    let before = argument_spans
        .iter()
        .take_while(|span| span.end <= offset)
        .count();
    let last = argument_spans.len().saturating_sub(1);
    u32::try_from(before.min(last)).unwrap_or(u32::MAX)
}

fn typed_signature_help(fact: &AuthoringFact, offset: usize) -> Option<SignatureHelp> {
    match &fact.kind {
        AuthoringKind::Choose { argument_spans, .. } => Some(signature(
            "(choose :sample <sample-qname> :slot <u32> (branch ...)+)",
            "One direct, non-nestable finite material choice in a Mechanic rule.",
            &["sample-qname", "slot", "branches"],
            active_parameter(argument_spans, offset),
        )),
        AuthoringKind::Branch { argument_spans, .. } => Some(signature(
            "(branch <EnumType/MEMBER> :mass <Mass> (effects ...))",
            "One enum-ordered material alternative.",
            &["outcome", "mass", "effects"],
            active_parameter(argument_spans, offset),
        )),
        AuthoringKind::QuantizeMass { argument_spans } => Some(signature(
            "(quantize-mass <numeric-expr>)",
            "Explicit round-to-nearest, ties-to-even conversion to Mass.",
            &["numeric-expr"],
            active_parameter(argument_spans, offset),
        )),
        AuthoringKind::ProjectsKernel { argument_spans, .. } => Some(signature(
            ":projects-kernel <sample-qname>",
            "Link this deterministic Recognizer to its immediately preceding finite kernel.",
            &["sample-qname"],
            active_parameter(argument_spans, offset),
        )),
        _ => None,
    }
}

fn incomplete_signature_help(text: &str, offset: usize) -> Option<SignatureHelp> {
    let scan = scan_edit_prefix(text, offset);
    if scan.in_string
        || scan.in_comment
        || scan
            .frames
            .iter()
            .any(|frame| frame.head() == Some("scenario"))
    {
        return None;
    }
    let current = scan.frames.last()?;
    match current.head() {
        Some("choose") => {
            let active = if current.atom_after(":sample").is_none() {
                0
            } else if current.atom_after(":slot").is_none() {
                1
            } else {
                2
            };
            Some(signature(
                "(choose :sample <sample-qname> :slot <u32> (branch ...)+)",
                "One direct, non-nestable finite material choice in a Mechanic rule.",
                &["sample-qname", "slot", "branches"],
                active,
            ))
        }
        Some("branch") => {
            let active = if current.items.get(1).is_none() {
                0
            } else if current.atom_after(":mass").is_none() {
                1
            } else {
                2
            };
            Some(signature(
                "(branch <EnumType/MEMBER> :mass <Mass> (effects ...))",
                "One enum-ordered material alternative.",
                &["outcome", "mass", "effects"],
                active,
            ))
        }
        Some("quantize-mass") => Some(signature(
            "(quantize-mass <numeric-expr>)",
            "Explicit round-to-nearest, ties-to-even conversion to Mass.",
            &["numeric-expr"],
            0,
        )),
        Some("rule")
            if rule_role(&scan.frames) == Some("recognizer")
                && current.has_atom(":projects-kernel") =>
        {
            Some(signature(
                ":projects-kernel <sample-qname>",
                "Link this deterministic Recognizer to its immediately preceding finite kernel.",
                &["sample-qname"],
                0,
            ))
        }
        Some(_) | None => None,
    }
}

/// Signature help for the innermost loader-typed probability call, with a
/// syntax-only fallback for an incomplete form the loader cannot yet type.
#[must_use]
pub fn signature_help(
    text: &str,
    snapshot: &AuthoringSnapshot,
    offset: usize,
) -> Option<SignatureHelp> {
    snapshot
        .innermost_call_at(offset)
        .and_then(|fact| typed_signature_help(fact, offset))
        .or_else(|| incomplete_signature_help(text, offset))
}

fn semantic_type(kind: &AuthoringKind) -> u32 {
    match kind {
        AuthoringKind::Keyword | AuthoringKind::ProjectsKernel { .. } => TOKEN_TYPE_KEYWORD,
        AuthoringKind::Choose { .. }
        | AuthoringKind::Branch { .. }
        | AuthoringKind::QuantizeMass { .. } => TOKEN_TYPE_FUNCTION,
        AuthoringKind::Mass { .. } | AuthoringKind::Number => TOKEN_TYPE_NUMBER,
        AuthoringKind::EnumMember => TOKEN_TYPE_ENUM_MEMBER,
        AuthoringKind::Sample => TOKEN_TYPE_VARIABLE,
    }
}

fn readonly(kind: &AuthoringKind) -> bool {
    matches!(kind, AuthoringKind::Mass { .. } | AuthoringKind::Sample)
}

/// Full-document semantic tokens in deterministic, LSP delta-encoded order.
#[must_use]
pub fn semantic_tokens(
    text: &str,
    line_index: &LineIndex,
    snapshot: &AuthoringSnapshot,
) -> SemanticTokens {
    let mut positioned = snapshot
        .facts
        .iter()
        .filter_map(|fact| {
            let range = byte_span_to_range(text, line_index, fact.token_span)?;
            (range.start.line == range.end.line && range.end.character > range.start.character)
                .then_some((range, fact))
        })
        .collect::<Vec<_>>();
    positioned.sort_by_key(|(range, fact)| {
        (
            range.start.line,
            range.start.character,
            range.end.character,
            semantic_type(&fact.kind),
        )
    });
    positioned
        .dedup_by_key(|(range, _)| (range.start.line, range.start.character, range.end.character));

    let mut previous_line = 0_u32;
    let mut previous_start = 0_u32;
    let mut data = Vec::with_capacity(positioned.len());
    for (range, fact) in positioned {
        let delta_line = range.start.line - previous_line;
        let delta_start = if delta_line == 0 {
            range.start.character - previous_start
        } else {
            range.start.character
        };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length: range.end.character - range.start.character,
            token_type: semantic_type(&fact.kind),
            token_modifiers_bitset: u32::from(readonly(&fact.kind)),
        });
        previous_line = range.start.line;
        previous_start = range.start.character;
    }
    SemanticTokens {
        result_id: None,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize) -> ByteSpan {
        ByteSpan { start, end }
    }

    fn kernel_snapshot(first_ticket_count: u128) -> AuthoringSnapshot {
        let denominator = babylon_bsl::TICKET_DENOMINATOR;
        let second_ticket_count = denominator - first_ticket_count;
        let allocation = vec![
            BranchAllocationFact {
                outcome: "YES".to_owned(),
                mass_nanounits: 1_000_000_000,
                ticket_start: 0,
                ticket_end: first_ticket_count,
                ticket_count: first_ticket_count,
            },
            BranchAllocationFact {
                outcome: "NO".to_owned(),
                mass_nanounits: if first_ticket_count == denominator / 2 {
                    1_000_000_000
                } else {
                    3_000_000_000
                },
                ticket_start: first_ticket_count,
                ticket_end: denominator,
                ticket_count: second_ticket_count,
            },
        ];
        AuthoringSnapshot::new(vec![
            AuthoringFact {
                token_span: span(1, 7),
                form_span: span(0, 80),
                kind: AuthoringKind::Choose {
                    sample: "struggle/spark".to_owned(),
                    allocation: Some(AllocationFact::Exact(allocation.clone())),
                    linkage: None,
                    argument_spans: Vec::new(),
                },
            },
            AuthoringFact {
                token_span: span(20, 26),
                form_span: span(19, 45),
                kind: AuthoringKind::Branch {
                    outcome: "YES".to_owned(),
                    mass_nanounits: Some(allocation[0].mass_nanounits),
                    tickets: Some((
                        allocation[0].ticket_start,
                        allocation[0].ticket_end,
                        allocation[0].ticket_count,
                    )),
                    allocation_ambiguity: None,
                    argument_spans: Vec::new(),
                },
            },
            AuthoringFact {
                token_span: span(50, 56),
                form_span: span(49, 75),
                kind: AuthoringKind::Branch {
                    outcome: "NO".to_owned(),
                    mass_nanounits: Some(allocation[1].mass_nanounits),
                    tickets: Some((
                        allocation[1].ticket_start,
                        allocation[1].ticket_end,
                        allocation[1].ticket_count,
                    )),
                    allocation_ambiguity: None,
                    argument_spans: Vec::new(),
                },
            },
        ])
    }

    #[test]
    fn conflicting_kernel_facts_are_manifest_order_independent() {
        let quarter = kernel_snapshot(babylon_bsl::TICKET_DENOMINATOR / 4);
        let half = kernel_snapshot(babylon_bsl::TICKET_DENOMINATOR / 2);
        let normal = merge_content_set_snapshots([
            ("probe/quarter".to_owned(), quarter.clone()),
            ("probe/half".to_owned(), half.clone()),
        ]);
        let reversed = merge_content_set_snapshots([
            ("probe/half".to_owned(), half),
            ("probe/quarter".to_owned(), quarter),
        ]);

        assert_eq!(normal, reversed);
        assert_eq!(normal.facts.len(), 3, "one fact per shared source span");

        let expected_sets = vec!["probe/half".to_owned(), "probe/quarter".to_owned()];
        let choose_allocation = normal.facts.iter().find_map(|fact| match &fact.kind {
            AuthoringKind::Choose { allocation, .. } => allocation.as_ref(),
            _ => None,
        });
        assert_eq!(
            choose_allocation,
            Some(&AllocationFact::Ambiguous {
                content_sets: expected_sets.clone(),
            })
        );

        let branches = normal
            .facts
            .iter()
            .filter_map(|fact| match &fact.kind {
                AuthoringKind::Branch {
                    mass_nanounits,
                    tickets,
                    allocation_ambiguity,
                    ..
                } => Some((mass_nanounits, tickets, allocation_ambiguity)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(branches.len(), 2);
        for (mass, tickets, ambiguity) in branches {
            assert_eq!(*mass, None, "must not expose one set's exact mass");
            assert_eq!(*tickets, None, "must not expose one set's ticket interval");
            assert_eq!(ambiguity.as_ref(), Some(&expected_sets));
        }

        let identical = merge_content_set_snapshots([
            (
                "probe/quarter".to_owned(),
                kernel_snapshot(babylon_bsl::TICKET_DENOMINATOR / 4),
            ),
            (
                "probe/same-quarter".to_owned(),
                kernel_snapshot(babylon_bsl::TICKET_DENOMINATOR / 4),
            ),
        ]);
        assert_eq!(identical.facts.len(), 3, "identical facts must deduplicate");
        assert!(identical.facts.iter().all(|fact| match &fact.kind {
            AuthoringKind::Choose { allocation, .. } => {
                matches!(allocation, Some(AllocationFact::Exact(_)))
            }
            AuthoringKind::Branch {
                allocation_ambiguity,
                ..
            } => allocation_ambiguity.is_none(),
            _ => true,
        }));
    }

    #[test]
    fn mass_is_canonical_at_fixed_nanounit_precision() {
        assert_eq!(canonical_mass(0), "0.000000000m");
        assert_eq!(canonical_mass(500_000_000), "0.500000000m");
        assert_eq!(canonical_mass(1_000_000_001), "1.000000001m");
    }

    #[test]
    fn loader_analysis_drives_exact_mass_allocation_hover_and_tokens() {
        use babylon_bsl::causal_contract::{EvidenceClass, RuleContract, RuleRole};
        use babylon_bsl::probability::{analyze_content_set, compile_rule_probability};
        use babylon_bsl::typecheck::TypeEnv;
        use babylon_bsl::{parse_bindings, read, EnumRegistry, LoadedRule};
        use std::collections::HashMap;

        let source = "(rule demo/spark :role mechanic :evidence designed \
            :material-basis \"bounded spark\" :fuel 64 \
            (bindings \
              (binding authored-mass :expr 0.125m) \
              (binding converted-mass :expr (quantize-mass 0.25c))) \
            (effects \
            (choose :sample struggle/spark :slot 0 \
              (branch SparkOutcome/YES :mass 1m (effects)) \
              (branch SparkOutcome/NO :mass 3m (effects)))))";
        let rule = read(source).expect("rule parses").0;
        let contract = RuleContract {
            rule_id: "demo/spark".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Designed,
        };
        let mut enums = EnumRegistry::default();
        enums
            .declare("SparkOutcome", &["YES".to_owned(), "NO".to_owned()])
            .expect("enum");
        let bindings = parse_bindings(&rule).expect("bindings");
        let types = TypeEnv {
            fields: HashMap::new(),
            exemptions: &[],
        };
        let compiled = compile_rule_probability(
            &rule,
            &[0],
            &contract,
            &enums,
            &types,
            &bindings,
            &HashMap::new(),
            &std::collections::HashSet::new(),
        )
        .expect("probability compiles");
        let loaded = LoadedRule {
            source_id: "spark.bsl".to_owned(),
            root_path: vec![0],
            rule,
            bindings,
            anchor: None,
            domain: None,
            contract,
            kernel: compiled.kernel,
            projection: compiled.projection,
            probability_facts: compiled.facts,
            probability_carrier: None,
            static_bound: 1,
            declared_fuel: 64,
            default_findings: Vec::new(),
        };
        let content = analyze_content_set(&[loaded]).expect("content analysis");
        let snapshot = snapshot_from_rule_analysis(source, &content.rules[0], None, None);
        let index = LineIndex::new(source);

        let choose_offset = source.find("choose").expect("choose token") + 1;
        let choose_hover = hover(source, &index, &snapshot, choose_offset).expect("choose hover");
        let HoverContents::Markup(choose_markup) = choose_hover.contents else {
            panic!("markdown hover")
        };
        assert!(choose_markup.value.contains("4611686018427387904"));
        assert!(choose_markup.value.contains("13835058055282163712"));

        let mass_offset = source.find("1m").expect("Mass token");
        let mass_hover = hover(source, &index, &snapshot, mass_offset).expect("Mass hover");
        let HoverContents::Markup(mass_markup) = mass_hover.contents else {
            panic!("markdown hover")
        };
        assert!(mass_markup.value.contains("1.000000000m"));

        let binding_mass_offset = source.find("0.125m").expect("binding Mass token");
        let binding_mass_hover =
            hover(source, &index, &snapshot, binding_mass_offset).expect("binding Mass hover");
        let HoverContents::Markup(binding_mass_markup) = binding_mass_hover.contents else {
            panic!("markdown hover")
        };
        assert!(binding_mass_markup.value.contains("0.125000000m"));

        let quantize_offset = source.find("quantize-mass").expect("quantize-mass token") + 1;
        assert!(signature_help(source, &snapshot, quantize_offset).is_some());

        let tokens = semantic_tokens(source, &index, &snapshot);
        assert!(tokens
            .data
            .iter()
            .any(|token| token.token_type == TOKEN_TYPE_FUNCTION));
        assert!(tokens
            .data
            .iter()
            .any(|token| token.token_type == TOKEN_TYPE_NUMBER));
    }

    #[test]
    fn branch_context_only_offers_mass_expressions() {
        let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(1, 7),
            form_span: span(0, 30),
            kind: AuthoringKind::Branch {
                outcome: "YES".to_owned(),
                mass_nanounits: None,
                tickets: None,
                allocation_ambiguity: None,
                argument_spans: Vec::new(),
            },
        }]);
        let text = "(branch YES :mass ";
        let labels = completion_items(text, &snapshot, text.len())
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["quantize-mass", "0.000000000m"]);

        let effects_context = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(1, 7),
            form_span: span(0, 30),
            kind: AuthoringKind::Branch {
                outcome: "YES".to_owned(),
                mass_nanounits: None,
                tickets: None,
                allocation_ambiguity: None,
                argument_spans: vec![span(8, 11), span(18, 20), span(21, 29)],
            },
        }]);
        let text = "(branch YES :mass 1m (effects ";
        assert!(completion_items(text, &effects_context, text.len()).is_empty());
    }

    #[test]
    fn probability_completions_are_contextual_and_ordered() {
        let mechanic = "(rule demo/spark :role mechanic :evidence designed \
            :material-basis \"bounded spark\" :fuel 64 (bindings) (effects ";
        let items = completion_items(mechanic, &AuthoringSnapshot::default(), mechanic.len());
        let labels = items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["choose"]);
        assert_eq!(
            items
                .iter()
                .map(|item| item.sort_text.as_deref().expect("sort text"))
                .collect::<Vec<_>>(),
            ["00-choose"]
        );

        let recognizer = "(rule demo/observe :role recognizer :evidence derived ";
        let labels = completion_items(recognizer, &AuthoringSnapshot::default(), recognizer.len())
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        assert_eq!(labels, vec![":projects-kernel"]);

        let binding = "(rule demo/spark :role mechanic :evidence designed \
            :material-basis \"bounded spark\" :fuel 64 (bindings (binding spark :expr ";
        let labels = completion_items(binding, &AuthoringSnapshot::default(), binding.len())
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["quantize-mass", "0.000000000m"]);
    }

    #[test]
    fn incomplete_choose_keeps_contextual_completion_and_signature_help() {
        let source = "(rule demo/spark :role mechanic :evidence designed \
            :material-basis \"bounded spark\" :fuel 64 (bindings) (effects \
            (choose :sample struggle/spark :slot 0 ";
        let snapshot = AuthoringSnapshot::default();
        let labels = completion_items(source, &snapshot, source.len())
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["branch"]);

        let help = signature_help(source, &snapshot, source.len()).expect("choose signature");
        assert_eq!(
            help.signatures[0].label,
            "(choose :sample <sample-qname> :slot <u32> (branch ...)+)"
        );
        assert_eq!(help.active_parameter, Some(2));

        let branch = format!("{source}(branch StruggleSparkOutcome/YES :mass ");
        let branch_labels = completion_items(&branch, &snapshot, branch.len())
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        assert_eq!(branch_labels, vec!["quantize-mass", "0.000000000m"]);
        let branch_help =
            signature_help(&branch, &snapshot, branch.len()).expect("branch signature");
        assert_eq!(
            branch_help.signatures[0].label,
            "(branch <EnumType/MEMBER> :mass <Mass> (effects ...))"
        );
        assert_eq!(branch_help.active_parameter, Some(1));
    }

    #[test]
    fn unrelated_string_scenario_and_global_contexts_suppress_probability_completions() {
        let snapshot = AuthoringSnapshot::default();
        let sources = [
            "",
            "(scenario demo/world ",
            "(rule demo/spark :role mechanic :evidence designed :material-basis \"choose ",
            "(rule demo/spark :role mechanic :evidence designed \
                :material-basis \"bounded spark\" :fuel 64 (bindings) (effects (update-node ",
            "(rule demo/spark :role mechanic :evidence designed \
                :material-basis \"bounded spark\" :fuel 64 (bindings) (effects ; choose ",
        ];
        for source in sources {
            assert!(
                completion_items(source, &snapshot, source.len()).is_empty(),
                "unexpected probability completion in {source:?}"
            );
        }
    }

    #[test]
    fn hover_reports_exact_allocation_or_state_dependence_truthfully() {
        let exact = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(1, 7),
            form_span: span(0, 20),
            kind: AuthoringKind::Choose {
                sample: "spark/sample".to_owned(),
                allocation: Some(AllocationFact::Exact(vec![BranchAllocationFact {
                    outcome: "YES".to_owned(),
                    mass_nanounits: 500_000_000,
                    ticket_start: 0,
                    ticket_end: 1_u128 << 63,
                    ticket_count: 1_u128 << 63,
                }])),
                linkage: None,
                argument_spans: Vec::new(),
            },
        }]);
        let text = "(choose ...)";
        let index = LineIndex::new(text);
        let exact_hover = hover(text, &index, &exact, 2).expect("hover");
        let HoverContents::Markup(exact_markup) = exact_hover.contents else {
            panic!("markdown hover")
        };
        assert!(exact_markup.value.contains("0.500000000m"));
        assert!(exact_markup.value.contains("9223372036854775808"));

        let dynamic = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(1, 7),
            form_span: span(0, 20),
            kind: AuthoringKind::Choose {
                sample: "spark/sample".to_owned(),
                allocation: Some(AllocationFact::Unavailable {
                    reason: "mass reads material state".to_owned(),
                }),
                linkage: None,
                argument_spans: Vec::new(),
            },
        }]);
        let dynamic_hover = hover(text, &index, &dynamic, 2).expect("hover");
        let HoverContents::Markup(dynamic_markup) = dynamic_hover.contents else {
            panic!("markdown hover")
        };
        assert!(dynamic_markup.value.contains("unavailable at load time"));
        assert!(dynamic_markup.value.contains("mass reads material state"));

        let ambiguous = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(1, 7),
            form_span: span(0, 20),
            kind: AuthoringKind::Choose {
                sample: "spark/sample".to_owned(),
                allocation: Some(AllocationFact::Ambiguous {
                    content_sets: vec!["probe/half".to_owned(), "probe/quarter".to_owned()],
                }),
                linkage: None,
                argument_spans: Vec::new(),
            },
        }]);
        let ambiguous_hover = hover(text, &index, &ambiguous, 2).expect("hover");
        let HoverContents::Markup(ambiguous_markup) = ambiguous_hover.contents else {
            panic!("markdown hover")
        };
        assert!(ambiguous_markup
            .value
            .contains("ambiguous across content sets"));
        assert!(ambiguous_markup
            .value
            .contains("`probe/half`, `probe/quarter`"));
        assert!(ambiguous_markup
            .value
            .contains("no manifest-order result was selected"));
        assert!(!ambiguous_markup
            .value
            .contains("Executable allocation over"));
    }

    #[test]
    fn branch_hover_reports_mass_and_ticket_knowledge_independently() {
        let text = "branch";
        let index = LineIndex::new(text);
        let hover_value = |mass_nanounits, tickets, allocation_ambiguity| {
            let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
                token_span: span(0, text.len()),
                form_span: span(0, text.len()),
                kind: AuthoringKind::Branch {
                    outcome: "YES".to_owned(),
                    mass_nanounits,
                    tickets,
                    allocation_ambiguity,
                    argument_spans: Vec::new(),
                },
            }]);
            let result = hover(text, &index, &snapshot, 1).expect("branch hover");
            let HoverContents::Markup(markup) = result.contents else {
                panic!("markdown hover")
            };
            markup.value
        };

        let exact = hover_value(Some(500_000_000), Some((0, 12, 12)), None);
        assert!(exact.contains("Exact mass `0.500000000m`"));
        assert!(exact.contains("exact tickets `[0, 12)` (`12` tickets)"));

        let static_mass_unknown_tickets = hover_value(Some(500_000_000), None, None);
        assert!(static_mass_unknown_tickets.contains("Exact static mass `0.500000000m`"));
        assert!(static_mass_unknown_tickets
            .contains("ticket interval is **not statically determined**"));
        assert!(!static_mass_unknown_tickets.contains("Exact mass and ticket interval"));

        let unknown_mass_exact_tickets = hover_value(None, Some((7, 19, 12)), None);
        assert!(
            unknown_mass_exact_tickets.contains("Mass expression is **not statically determined**")
        );
        assert!(unknown_mass_exact_tickets.contains("exact tickets `[7, 19)` (`12` tickets)"));

        let unknown = hover_value(None, None, None);
        assert!(unknown
            .contains("Mass expression and ticket interval are **not statically determined**"));

        let ambiguous = hover_value(
            None,
            None,
            Some(vec!["probe/half".to_owned(), "probe/quarter".to_owned()]),
        );
        assert!(ambiguous.contains("ambiguous across content sets"));
        assert!(ambiguous.contains("`probe/half`, `probe/quarter`"));
        assert!(ambiguous.contains("no manifest-order result was selected"));
        assert!(!ambiguous.contains("Exact mass"));
        assert!(!ambiguous.contains("exact tickets"));
    }

    #[test]
    fn signature_help_uses_loader_argument_spans_for_active_parameter() {
        let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(1, 7),
            form_span: span(0, 50),
            kind: AuthoringKind::Choose {
                sample: "spark/sample".to_owned(),
                allocation: None,
                linkage: None,
                argument_spans: vec![span(9, 21), span(28, 29), span(31, 49)],
            },
        }]);
        let help = signature_help(
            "(choose ........................................)",
            &snapshot,
            30,
        )
        .expect("signature");
        assert_eq!(help.active_parameter, Some(2));
        assert_eq!(help.signatures[0].active_parameter, Some(2));
    }

    #[test]
    fn projection_hover_reports_linkage_and_exact_likelihood_when_supplied() {
        let text = ":projects-kernel struggle/spark";
        let index = LineIndex::new(text);
        let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(0, 16),
            form_span: span(0, 31),
            kind: AuthoringKind::ProjectsKernel {
                sample: "struggle/spark".to_owned(),
                linkage: Some(KernelProjectionFact {
                    kernel_rule_id: "struggle/spark".to_owned(),
                    projection_rule_id: "struggle/spark-event".to_owned(),
                }),
                likelihood: Some(EventLikelihoodAnalysisFact::Exact(vec![
                    EventLikelihoodFact {
                        event_type: "EventType/EXCESSIVE_FORCE".to_owned(),
                        favorable_outcomes: vec!["EXCESSIVE_FORCE".to_owned()],
                        numerator: 1_u128 << 63,
                        denominator: 1_u128 << 64,
                    },
                ])),
                argument_spans: vec![span(17, 31)],
            },
        }]);
        let result = hover(text, &index, &snapshot, 2).expect("projection hover");
        let HoverContents::Markup(markup) = result.contents else {
            panic!("markdown hover")
        };
        assert!(markup.value.contains("struggle/spark-event"));
        assert!(markup.value.contains("EventType/EXCESSIVE_FORCE"));
        assert!(markup
            .value
            .contains("9223372036854775808/18446744073709551616"));
        assert!(markup.value.contains("EXCESSIVE_FORCE"));
        let sample_result = hover(text, &index, &snapshot, 20).expect("projection sample hover");
        let HoverContents::Markup(sample_markup) = sample_result.contents else {
            panic!("markdown hover")
        };
        assert!(sample_markup.value.contains("struggle/spark-event"));
        let signature = signature_help(text, &snapshot, 20).expect("projection signature");
        assert_eq!(
            signature.signatures[0].label,
            ":projects-kernel <sample-qname>"
        );
    }

    #[test]
    fn projection_hover_preserves_loader_state_dependence_reason() {
        let text = ":projects-kernel struggle/spark";
        let index = LineIndex::new(text);
        let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(0, 16),
            form_span: span(0, 31),
            kind: AuthoringKind::ProjectsKernel {
                sample: "struggle/spark".to_owned(),
                linkage: None,
                likelihood: Some(EventLikelihoodAnalysisFact::StateDependent {
                    reason: "paired scenario determines two carriers".to_owned(),
                }),
                argument_spans: vec![span(17, 31)],
            },
        }]);

        let result = hover(text, &index, &snapshot, 2).expect("projection hover");
        let HoverContents::Markup(markup) = result.contents else {
            panic!("markdown hover")
        };
        assert!(markup.value.contains("state-dependent"));
        assert!(markup
            .value
            .contains("paired scenario determines two carriers"));
    }

    #[test]
    fn projection_hover_renders_cross_set_ambiguity_without_selecting_a_value() {
        let text = ":projects-kernel struggle/spark";
        let index = LineIndex::new(text);
        let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(0, 16),
            form_span: span(0, 31),
            kind: AuthoringKind::ProjectsKernel {
                sample: "struggle/spark".to_owned(),
                linkage: None,
                likelihood: Some(EventLikelihoodAnalysisFact::Ambiguous {
                    content_sets: vec!["probe/half".to_owned(), "probe/quarter".to_owned()],
                }),
                argument_spans: vec![span(17, 31)],
            },
        }]);

        let result = hover(text, &index, &snapshot, 2).expect("projection hover");
        let HoverContents::Markup(markup) = result.contents else {
            panic!("markdown hover")
        };
        assert!(markup.value.contains("ambiguous across content sets"));
        assert!(markup.value.contains("`probe/half`, `probe/quarter`"));
        assert!(markup
            .value
            .contains("no manifest-order result was selected"));
        assert!(!markup.value.contains("/18446744073709551616"));
    }

    #[test]
    fn projection_hover_preserves_forecast_refusal_stage_and_reason() {
        let text = ":projects-kernel struggle/spark";
        let index = LineIndex::new(text);
        let snapshot = AuthoringSnapshot::new(vec![AuthoringFact {
            token_span: span(0, 16),
            form_span: span(0, 31),
            kind: AuthoringKind::ProjectsKernel {
                sample: "struggle/spark".to_owned(),
                linkage: None,
                likelihood: Some(EventLikelihoodAnalysisFact::Refused {
                    stage: ForecastRefusalStage::Execution,
                    reason: "forecast subject does not fire the kernel mechanic".to_owned(),
                }),
                argument_spans: vec![span(17, 31)],
            },
        }]);

        let result = hover(text, &index, &snapshot, 2).expect("projection hover");
        let HoverContents::Markup(markup) = result.contents else {
            panic!("markdown hover")
        };
        assert!(markup.value.contains("refused during execution"));
        assert!(markup
            .value
            .contains("forecast subject does not fire the kernel mechanic"));
        assert!(!markup.value.contains("state-dependent"));
    }

    #[test]
    fn semantic_tokens_are_sorted_deduplicated_and_utf16_delta_encoded() {
        let text = "0.5m\n😀choose";
        let index = LineIndex::new(text);
        let snapshot = AuthoringSnapshot::new(vec![
            AuthoringFact {
                token_span: span(9, 15),
                form_span: span(9, 15),
                kind: AuthoringKind::Choose {
                    sample: "x/y".to_owned(),
                    allocation: None,
                    linkage: None,
                    argument_spans: Vec::new(),
                },
            },
            AuthoringFact {
                token_span: span(0, 4),
                form_span: span(0, 4),
                kind: AuthoringKind::Mass {
                    nanounits: 500_000_000,
                },
            },
        ]);
        let tokens = semantic_tokens(text, &index, &snapshot);
        assert_eq!(tokens.data.len(), 2);
        assert_eq!(tokens.data[0].delta_line, 0);
        assert_eq!(tokens.data[0].delta_start, 0);
        assert_eq!(tokens.data[0].length, 4);
        assert_eq!(tokens.data[0].token_type, TOKEN_TYPE_NUMBER);
        assert_eq!(tokens.data[0].token_modifiers_bitset, 1);
        assert_eq!(tokens.data[1].delta_line, 1);
        assert_eq!(tokens.data[1].delta_start, 2);
        assert_eq!(tokens.data[1].length, 6);
        assert_eq!(tokens.data[1].token_type, TOKEN_TYPE_FUNCTION);
    }
}
