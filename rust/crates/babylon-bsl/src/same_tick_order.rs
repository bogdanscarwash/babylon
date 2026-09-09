//! Same-tick ordering as LOAD REFUSALS (Task W2, BSL Hygiene Knock-out —
//! `docs/superpowers/plans/2026-08-18-bsl-hygiene-knockout.md` §Task W2,
//! amended by `.superpowers/sdd/2026-08-18-bsl-hygiene-knockout/task-w2-
//! brief.md` and its binding `prep-adjudication.md`).
//!
//! **Boundary ruling (Director, 2026-08-18 ~13:40 EDT), the reason this is
//! a load-time check and not an external lint:** "if you need a sentinel
//! that's a code smell for you need to expand BSL" — semantic coherence
//! belongs IN the language as a load/type refusal, not bolted on
//! afterward. This module adds two such refusals, in the same family as
//! `E-LOAD-001` (content-set-wide, checked at load, over whatever content
//! set the caller actually loaded — never a hypothetical wider one).
//!
//! **PER-19 rank-aware boundary (after ADR222).** [`diagnose_ranked`] consumes
//! the tick driver's resolved execution ranks and uses D16 bytes only as a
//! same-rank tie-break. [`diagnose`] remains an all-rank-zero adapter for
//! historical audits and fixtures whose rules genuinely share one rank.
//! ADR224 retains deterministic sequential execution for rules at the same
//! rank. Aggregate enforcement is live only after phase compilation over the
//! complete concatenated content set.
//!
//! **Refusal 1 — `E-LOAD-058`, stale-default read.** For every
//! `(binding … :field f :optional :default d)` in rule `R`, compute `f`'s
//! *writer set*: every OTHER rule `W` in the content set with an
//! `(update-node … f (<op> …))` effect, **excluding `W` where
//! `W.rule_id == R.rule_id`** (adjudication §(c), verbatim, keyed on rule
//! IDENTITY, never on write TARGET — `solidarity/p0-transmit` reads `self`
//! and writes `it`, same rule, and must still self-exclude). If any
//! remaining writer's `(execution rank, rule-id bytes)` key does not sort
//! strictly before `R`'s (§4.2/D116), `R` can observe `d` on a tick where
//! that writer already ran and left a real value from an EARLIER tick —
//! refused.
//!
//! **Refusal 2 — `E-LOAD-059`, unreset fan-in.** A field written by 2+
//! distinct rules in the content set needs its execution-earliest writer to be
//! either an unconditional `set` or the D127 unconditional-recompute shape
//! (adjudication §(d), verbatim: unconditional = no `(when …)` at all, or
//! a `(when …)` whose condition is the literal `#t`; the qualifying reset
//! must also be a direct child of `(effects …)`, because nested `guard` and
//! `for-each` bodies may execute zero times) — else refused as an accumulation
//! with no rule-identifiable reset.
//!
//! **ADR224 governed dispositions.** Exact rows below record the landed
//! intentional E058/E059 findings. A disposition is default-deny: an unknown
//! reader, binding, field, writer, execution-earliest writer, or reset shape
//! still refuses. The earlier `ai/_inbox/amendment-prior-tick-draft.md`
//! remains historical evidence only; it is not active authority and no future
//! `:prior-tick` vocabulary is required for this enforcement.

use crate::bindings::{parse_bindings, BindSource};
use crate::causal_contract::{
    validate_ast_walk_bounds, AstWalkError, AstWalkLimit, AstWalkLimits, AST_WALK_LIMITS,
    MAX_AST_WALK_NODES,
};
use crate::material_basis::keyword_value;
use crate::reader::{Atom, SExpr};
use std::collections::{HashMap, HashSet};

const SAME_TICK_WALKER: &str = "same-tick field writes";

/// Whether the post-phase-compile aggregate caller must enforce the rank-aware
/// E058/E059 result through [`Diagnosis::into_enforced_result`].
///
/// This must never guard the historical per-source [`diagnose`] adapter:
/// source boundaries are not causal boundaries, and all-rank-zero analysis is
/// invalid across compiled phase positions.
pub const ENFORCE_RANK_AWARE_AGGREGATE_ORDERING: bool = true;

/// One rule form paired with the resolved execution rank compiled by the
/// engine's governed phase scheduler.
///
/// Pairing the form and rank in one borrowed value makes a missing-rank state
/// unrepresentable at the analyzer boundary. The rank is compared first;
/// ascending rule-ID bytes break ties at one rank.
#[derive(Debug, Clone, Copy)]
pub struct RankedRule<'a> {
    /// The content-set-unique rule identity. [`diagnose_ranked`] verifies it
    /// against the qname inside [`Self::form`] before analyzing any effects.
    pub rule_id: &'a str,
    /// The resolved phase-schedule rank.
    pub execution_rank: u16,
    /// The parsed rule form.
    pub form: &'a SExpr,
}

/// A malformed or forged [`RankedRule`] collection.
///
/// This refusal is deliberately typed and uncoded: phase compilation owns
/// rank assignment, while this module owns proving that the public analyzer
/// received one unique identity per actual rule form. No finding is computed
/// from an invalid collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RankedRuleInputError {
    /// The supplied form is not `(rule <qname-id> ...)`.
    InvalidRuleForm {
        /// The identity the caller tried to associate with the form.
        supplied_rule_id: String,
    },
    /// The caller-supplied identity differs from the form's own identity.
    IdentityMismatch {
        /// The identity supplied alongside the form.
        supplied_rule_id: String,
        /// The qname encoded inside the rule form.
        form_rule_id: String,
    },
    /// One supplied rule identity occurs more than once.
    DuplicateRuleId {
        /// The repeated identity.
        rule_id: String,
    },
}

/// The reset-relevant facts of an E059 finding's execution-earliest writer.
///
/// These are the exact facts the analyzer uses to decide whether a writer is
/// an unconditional reset. Recording them prevents a governed writer ID from
/// silently changing its write shape while retaining its disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarliestWriterSemantics {
    /// Whether the rule has no `when`, or literal `(when #t)`.
    pub unconditional: bool,
    /// Whether this writer uses a direct, exactly-once `set` on the field.
    pub has_set: bool,
    /// Whether the writer cites the D127 recompute shape.
    pub d127_recompute: bool,
}

/// One allowed execution-earliest writer and its reviewed reset semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernedEarliestWriter {
    /// Exact rule identity.
    pub rule_id: &'static str,
    /// Exact reset-relevant facts at the ADR224 review.
    pub semantics: EarliestWriterSemantics,
}

/// One exact ADR224 disposition for an intentional E058 finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernedE058Disposition {
    /// Exact reader rule identity.
    pub reader_rule: &'static str,
    /// Exact binding identity within the reader.
    pub binding_name: &'static str,
    /// Exact field read by the binding.
    pub field: &'static str,
    /// Complete allowed set of non-earlier writer rule identities.
    pub allowed_writers: &'static [&'static str],
    /// Why the sequential read is intentional.
    pub reason: &'static str,
    /// Approving authority.
    pub owner: &'static str,
    /// Approval date.
    pub date: &'static str,
    /// Recording architecture decision.
    pub adr: &'static str,
}

/// One field-keyed ADR224 disposition for an intentional E059 finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernedE059Disposition {
    /// Exact multi-writer field.
    pub field: &'static str,
    /// Exact allowed superset of writer rule identities.
    pub allowed_writers: &'static [&'static str],
    /// Every execution-earliest writer state observed and approved.
    pub allowed_earliest_writers: &'static [GovernedEarliestWriter],
    /// Why the fan-in is intentional without a universal reset.
    pub reason: &'static str,
    /// Approving authority.
    pub owner: &'static str,
    /// Approval date.
    pub date: &'static str,
    /// Recording architecture decision.
    pub adr: &'static str,
}

const DISPOSITION_OWNER: &str = "Director";
const DISPOSITION_DATE: &str = "2026-08-23";
const DISPOSITION_ADR: &str = "ADR224";
const GOVERNED_E058_CARDINALITY: usize = 13;
const GOVERNED_E059_CARDINALITY: usize = 11;
const CONDITIONAL_SET: EarliestWriterSemantics = EarliestWriterSemantics {
    unconditional: false,
    has_set: true,
    d127_recompute: false,
};
const CONDITIONAL_NON_SET: EarliestWriterSemantics = EarliestWriterSemantics {
    unconditional: false,
    has_set: false,
    d127_recompute: false,
};

/// The complete exact E058 disposition table ratified by ADR224.
pub const GOVERNED_E058_DISPOSITIONS: &[GovernedE058Disposition] = &[
    GovernedE058Disposition {
        reader_rule: "consciousness/p0-position",
        binding_name: "f",
        field: "social-class/fascist",
        allowed_writers: &["consciousness/p6-route"],
        reason: "positioning reads the prior ternary before the later routing phase",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p0-position",
        binding_name: "l",
        field: "social-class/liberal",
        allowed_writers: &["consciousness/p6-route"],
        reason: "positioning reads the prior ternary before the later routing phase",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p0-position",
        binding_name: "r",
        field: "social-class/revolutionary",
        allowed_writers: &["consciousness/p6-route"],
        reason: "positioning reads the prior ternary before the later routing phase",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p1-inbox-reset",
        binding_name: "f",
        field: "social-class/fascist",
        allowed_writers: &["consciousness/p6-route"],
        reason: "the retained ternary binding is inert under the unconditional inbox reset",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p1-inbox-reset",
        binding_name: "l",
        field: "social-class/liberal",
        allowed_writers: &["consciousness/p6-route"],
        reason: "the retained ternary binding is inert under the unconditional inbox reset",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p1-inbox-reset",
        binding_name: "r",
        field: "social-class/revolutionary",
        allowed_writers: &["consciousness/p6-route"],
        reason: "the retained ternary binding is inert under the unconditional inbox reset",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p3-class-solidarity-push",
        binding_name: "r",
        field: "social-class/revolutionary",
        allowed_writers: &["consciousness/p6-route"],
        reason: "solidarity transmission intentionally observes the pre-routing class share",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p5-agitation",
        binding_name: "agitation",
        field: "social-class/agitation",
        allowed_writers: &["consciousness/p6-route"],
        reason: "agitation accumulation precedes the route-and-decay write in the same tick",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p5-agitation",
        binding_name: "f",
        field: "social-class/fascist",
        allowed_writers: &["consciousness/p6-route"],
        reason: "agitation eligibility intentionally observes the pre-routing ternary",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p5-agitation",
        binding_name: "l",
        field: "social-class/liberal",
        allowed_writers: &["consciousness/p6-route"],
        reason: "agitation eligibility intentionally observes the pre-routing ternary",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p5-agitation",
        binding_name: "prev-wages",
        field: "social-class/previous-wages",
        allowed_writers: &["consciousness/p7-persist-baselines"],
        reason: "the persisted wage baseline is deliberately one tick behind",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p5-agitation",
        binding_name: "prev-wealth",
        field: "social-class/previous-wealth",
        allowed_writers: &["consciousness/p7-persist-baselines"],
        reason: "the persisted wealth baseline is deliberately one tick behind",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE058Disposition {
        reader_rule: "consciousness/p5-agitation",
        binding_name: "r",
        field: "social-class/revolutionary",
        allowed_writers: &["consciousness/p6-route"],
        reason: "agitation eligibility intentionally observes the pre-routing ternary",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
];

/// The complete field-keyed E059 disposition table ratified by ADR224.
pub const GOVERNED_E059_DISPOSITIONS: &[GovernedE059Disposition] = &[
    GovernedE059Disposition {
        field: "institution/rent-pool",
        allowed_writers: &[
            "imperial-rent/r02-extraction-credit",
            "imperial-rent/r04-tribute-credit",
        ],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "imperial-rent/r02-extraction-credit",
            semantics: CONDITIONAL_NON_SET,
        }],
        reason: "the durable rent pool receives separate extraction and tribute credits",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/active",
        allowed_writers: &[
            "decomposition/p04-enforcer-intake",
            "decomposition/p05-ip-intake",
            "decomposition/p06-la-deactivate",
        ],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "decomposition/p04-enforcer-intake",
            semantics: CONDITIONAL_SET,
        }],
        reason: "role-exclusive decomposition branches activate sinks and deactivate the source",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/agitation",
        allowed_writers: &[
            "consciousness/p0-position",
            "consciousness/p5-agitation",
            "consciousness/p6-route",
        ],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "consciousness/p0-position",
            semantics: CONDITIONAL_SET,
        }],
        reason: "position, accumulate, then route-and-decay is one sequential pipeline",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/community-cost-modifier",
        allowed_writers: &[
            "community/c09-cost-modifier-reset",
            "community/c10-cost-modifier-accumulate",
        ],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "community/c09-cost-modifier-reset",
            semantics: CONDITIONAL_SET,
        }],
        reason: "active classes reset to one before membership factors multiply in",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/fascist",
        allowed_writers: &["consciousness/p0-position", "consciousness/p6-route"],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "consciousness/p0-position",
            semantics: CONDITIONAL_SET,
        }],
        reason: "initial positioning and later routing have mutually exclusive guards",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/liberal",
        allowed_writers: &["consciousness/p0-position", "consciousness/p6-route"],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "consciousness/p0-position",
            semantics: CONDITIONAL_SET,
        }],
        reason: "initial positioning and later routing have mutually exclusive guards",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/population",
        allowed_writers: &[
            "decomposition/p04-enforcer-intake",
            "decomposition/p05-ip-intake",
        ],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "decomposition/p04-enforcer-intake",
            semantics: CONDITIONAL_NON_SET,
        }],
        reason: "role-exclusive decomposition branches transfer population into distinct sinks",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/production-value",
        allowed_writers: &[
            "production/p1-direct-production",
            "production/p2-employed-routing",
            "production/p3-employed-fallback",
        ],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "production/p1-direct-production",
            semantics: CONDITIONAL_SET,
        }],
        reason: "role and employer topology select exactly one production branch per class",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/revolutionary",
        allowed_writers: &["consciousness/p0-position", "consciousness/p6-route"],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "consciousness/p0-position",
            semantics: CONDITIONAL_SET,
        }],
        reason: "initial positioning and later routing have mutually exclusive guards",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "social-class/wealth",
        allowed_writers: &[
            "decomposition/p04-enforcer-intake",
            "decomposition/p05-ip-intake",
            "imperial-rent/r01-extraction",
            "imperial-rent/r03-tribute",
            "production/p1-direct-production",
            "production/p2-employed-routing",
            "production/p3-employed-fallback",
        ],
        allowed_earliest_writers: &[
            GovernedEarliestWriter {
                rule_id: "decomposition/p04-enforcer-intake",
                semantics: CONDITIONAL_NON_SET,
            },
            GovernedEarliestWriter {
                rule_id: "imperial-rent/r01-extraction",
                semantics: CONDITIONAL_NON_SET,
            },
            GovernedEarliestWriter {
                rule_id: "production/p1-direct-production",
                semantics: CONDITIONAL_NON_SET,
            },
        ],
        reason: "wealth is a durable stock changed by production, transfers, and decomposition",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
    GovernedE059Disposition {
        field: "territory/population",
        allowed_writers: &["territory/p2-eviction-pipeline", "territory/p4-camp-decay"],
        allowed_earliest_writers: &[GovernedEarliestWriter {
            rule_id: "territory/p2-eviction-pipeline",
            semantics: CONDITIONAL_NON_SET,
        }],
        reason: "durable population receives eviction flow before same-tick camp decay",
        owner: DISPOSITION_OWNER,
        date: DISPOSITION_DATE,
        adr: DISPOSITION_ADR,
    },
];

/// Refusal 1: rule `reader_rule`'s binding `binding_name` reads field
/// `field` with a declared `:optional :default`, and `writer_rule` — a
/// DIFFERENT rule in the same content set — writes `field` and executes on or
/// after `reader_rule` under resolved phase rank plus D16's same-rank byte
/// tie-break, so `reader_rule` can observe the DECLARED DEFAULT on a tick where
/// `writer_rule` later runs and leaves a value for a subsequent tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleDefaultRead {
    /// The rule doing the exposed `:optional :default` read.
    pub reader_rule: String,
    /// The binding name inside `reader_rule`.
    pub binding_name: String,
    /// The field the binding reads.
    pub field: String,
    /// A rule that writes `field` and does not execute strictly before
    /// `reader_rule` (the execution-earliest such writer, for a deterministic
    /// message — there may be more than one).
    pub writer_rule: String,
}

/// Refusal 2: `field` is written by 2+ distinct rules (`writers`, resolved
/// execution order) in the content set, and the execution-earliest is
/// neither an unconditional `set` nor the D127 unconditional-recompute
/// shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresetFanIn {
    /// The multi-writer field.
    pub field: String,
    /// Every distinct writer rule id, resolved rank then ascending ID bytes.
    pub writers: Vec<String>,
    /// The execution-earliest writer's reset-relevant facts.
    pub earliest_writer_semantics: EarliestWriterSemantics,
}

/// Complete E058 offender evidence retained without changing the stable
/// single-writer display shape of [`StaleDefaultRead`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct StaleDefaultWriterSet {
    reader_rule: String,
    binding_name: String,
    field: String,
    writer_rules: Vec<String>,
}

/// Typed rejections from same-tick analysis. `code`/`Display` follow the
/// same discipline every other load-error family in this crate does (S-11:
/// a typed, named, loud return value, never a warning or a log line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SameTickOrderError {
    /// The public rank-analysis input did not preserve rule identity.
    RankedRuleInput(RankedRuleInputError),
    /// `E-LOAD-058`.
    StaleDefaultRead(StaleDefaultRead),
    /// `E-LOAD-059`.
    UnresetFanIn(UnresetFanIn),
    /// The write-site analyzer exceeded one declared AST resource boundary.
    AstWalkLimit(AstWalkError),
}

impl SameTickOrderError {
    /// The spec code — see `docs/reference/bsl-language.rst`'s E-LOAD
    /// tables (W2.3 adds both rows).
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::StaleDefaultRead(_) => Some("E-LOAD-058"),
            Self::UnresetFanIn(_) => Some("E-LOAD-059"),
            Self::RankedRuleInput(_) | Self::AstWalkLimit(_) => None,
        }
    }
}

impl std::fmt::Display for SameTickOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RankedRuleInput(error) => match error {
                RankedRuleInputError::InvalidRuleForm { supplied_rule_id } => write!(
                    f,
                    "same-tick rank analysis input for {supplied_rule_id} is not a \
                     (rule <qname-id> ...) form"
                ),
                RankedRuleInputError::IdentityMismatch {
                    supplied_rule_id,
                    form_rule_id,
                } => write!(
                    f,
                    "same-tick rank analysis supplied rule id {supplied_rule_id}, but the \
                     form's own id is {form_rule_id}"
                ),
                RankedRuleInputError::DuplicateRuleId { rule_id } => write!(
                    f,
                    "same-tick rank analysis received rule id {rule_id} more than once"
                ),
            },
            Self::StaleDefaultRead(v) => write!(
                f,
                "E-LOAD-058: stale-default read — rule {reader}'s binding `{binding}` reads \
                 field {field} with :optional :default, but rule {writer} writes {field} and \
                 does not execute strictly before {reader} under resolved phase rank plus \
                 D16's same-rank byte tie-break (§4.2/D116) — {reader} can observe the declared \
                 default on a tick where {writer} already ran and left a value from an EARLIER \
                 tick, never this one's write. Refused at load (S-11), content-set-wide, like \
                 E-LOAD-001. An exact ADR224 governed disposition is required for an \
                 intentional sequential dependency.",
                reader = v.reader_rule,
                binding = v.binding_name,
                field = v.field,
                writer = v.writer_rule,
            ),
            Self::UnresetFanIn(v) => write!(
                f,
                "E-LOAD-059: unreset fan-in — field {field} is written by {n} rules in this \
                 content set ({writers}), and the execution-earliest writer is neither an \
                 unconditional `set` (no (when …), or (when #t)) nor the D127 \
                 unconditional-recompute shape (a :material-basis citing D127) — an \
                 accumulation with no rule-identifiable reset under resolved phase rank plus \
                 D16's same-rank byte tie-break. Refused at load (S-11), \
                 content-set-wide, like E-LOAD-001.",
                field = v.field,
                n = v.writers.len(),
                writers = v.writers.join(", "),
            ),
            Self::AstWalkLimit(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SameTickOrderError {}

/// Both causal refusals' findings, or one analyzer-input/resource refusal,
/// against one content set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnosis {
    /// A malformed or forged ranked-rule collection. Findings remain empty
    /// when set.
    pub ranked_rule_input_error: Option<RankedRuleInputError>,
    /// A typed analyzer resource refusal. Findings remain empty when set.
    pub ast_walk_error: Option<AstWalkError>,
    /// Every refusal-1 finding, one per triggering binding (deterministic
    /// order: ascending `(reader_rule, binding_name, field)` bytes).
    pub stale_default_reads: Vec<StaleDefaultRead>,
    /// Every refusal-2 finding, one per triggering field (deterministic
    /// order: ascending `field`).
    pub unreset_fan_ins: Vec<UnresetFanIn>,
    /// Complete refusal-1 offender sets used by governed enforcement.
    stale_default_writer_sets: Vec<StaleDefaultWriterSet>,
}

impl Diagnosis {
    /// Whether the content set has zero findings under either refusal.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.ranked_rule_input_error.is_none()
            && self.ast_walk_error.is_none()
            && self.stale_default_reads.is_empty()
            && self.unreset_fan_ins.is_empty()
    }

    /// Gate-ON semantics: the FIRST finding, deterministically (refusal 1
    /// before refusal 2, each in the order above) — matching S-11's "loud
    /// failure, reject the whole content set" precedent, the same one
    /// `E-LOAD-001` follows (first-encountered, not an exhaustive list;
    /// the exhaustive inventory is what [`Diagnosis`]'s own fields are
    /// for, and what W2.4's audit reads directly).
    ///
    /// # Errors
    /// The first [`SameTickOrderError`], if this diagnosis is not clean.
    pub fn into_result(self) -> Result<(), SameTickOrderError> {
        if let Some(error) = self.ranked_rule_input_error {
            return Err(SameTickOrderError::RankedRuleInput(error));
        }
        if let Some(error) = self.ast_walk_error {
            return Err(SameTickOrderError::AstWalkLimit(error));
        }
        if let Some(v) = self.stale_default_reads.into_iter().next() {
            return Err(SameTickOrderError::StaleDefaultRead(v));
        }
        if let Some(v) = self.unreset_fan_ins.into_iter().next() {
            return Err(SameTickOrderError::UnresetFanIn(v));
        }
        Ok(())
    }

    /// Enforce ADR224's default-deny disposition policy.
    ///
    /// Raw findings remain available on [`Diagnosis`] for audit. This method
    /// filters only exact governed findings, then returns the first remaining
    /// E058 by `(reader, binding, field)` bytes or, if none remains, the first
    /// E059 by field bytes. E058 retains category precedence over E059.
    ///
    /// # Errors
    /// The byte-first finding not covered by an exact governed disposition.
    pub fn into_enforced_result(self) -> Result<(), SameTickOrderError> {
        let Diagnosis {
            ranked_rule_input_error,
            ast_walk_error,
            mut stale_default_reads,
            mut unreset_fan_ins,
            stale_default_writer_sets,
        } = self;
        if let Some(error) = ranked_rule_input_error {
            return Err(SameTickOrderError::RankedRuleInput(error));
        }
        if let Some(error) = ast_walk_error {
            return Err(SameTickOrderError::AstWalkLimit(error));
        }
        stale_default_reads.sort_by(compare_stale_default_reads);
        unreset_fan_ins.sort_by(|left, right| left.field.as_bytes().cmp(right.field.as_bytes()));

        for finding in stale_default_reads {
            let writers = stale_default_writer_sets
                .iter()
                .find(|evidence| stale_evidence_matches(evidence, &finding))
                .map(|evidence| evidence.writer_rules.as_slice());
            if !writers.is_some_and(|actual| e058_is_disposed(&finding, actual)) {
                return Err(SameTickOrderError::StaleDefaultRead(finding));
            }
        }
        for finding in unreset_fan_ins {
            if !e059_is_disposed(&finding) {
                return Err(SameTickOrderError::UnresetFanIn(finding));
            }
        }
        Ok(())
    }
}

fn compare_stale_default_reads(
    left: &StaleDefaultRead,
    right: &StaleDefaultRead,
) -> std::cmp::Ordering {
    (
        left.reader_rule.as_bytes(),
        left.binding_name.as_bytes(),
        left.field.as_bytes(),
    )
        .cmp(&(
            right.reader_rule.as_bytes(),
            right.binding_name.as_bytes(),
            right.field.as_bytes(),
        ))
}

fn stale_evidence_matches(evidence: &StaleDefaultWriterSet, finding: &StaleDefaultRead) -> bool {
    evidence.reader_rule == finding.reader_rule
        && evidence.binding_name == finding.binding_name
        && evidence.field == finding.field
}

fn e058_is_disposed(finding: &StaleDefaultRead, actual_writers: &[String]) -> bool {
    e058_is_disposed_with(GOVERNED_E058_DISPOSITIONS, finding, actual_writers)
}

fn e058_is_disposed_with(
    rows: &[GovernedE058Disposition],
    finding: &StaleDefaultRead,
    actual_writers: &[String],
) -> bool {
    let Some(row) = exact_e058_disposition(rows, finding) else {
        return false;
    };
    !actual_writers.is_empty()
        && actual_writers.first() == Some(&finding.writer_rule)
        && writer_sets_equal(actual_writers, row.allowed_writers)
}

fn writer_sets_equal(actual: &[String], governed: &[&str]) -> bool {
    actual.len() == governed.len()
        && actual
            .iter()
            .all(|writer| governed.contains(&writer.as_str()))
}

fn e059_is_disposed(finding: &UnresetFanIn) -> bool {
    e059_is_disposed_with(GOVERNED_E059_DISPOSITIONS, finding)
}

fn e059_is_disposed_with(rows: &[GovernedE059Disposition], finding: &UnresetFanIn) -> bool {
    let Some(row) = exact_e059_disposition(rows, finding) else {
        return false;
    };
    let writers_allowed = finding
        .writers
        .iter()
        .all(|writer| row.allowed_writers.contains(&writer.as_str()));
    let Some(earliest) = finding.writers.first() else {
        return false;
    };
    let earliest_allowed = row.allowed_earliest_writers.iter().any(|allowed| {
        allowed.rule_id == earliest && allowed.semantics == finding.earliest_writer_semantics
    });
    finding.writers.len() >= 2 && writers_allowed && earliest_allowed
}

fn exact_e058_disposition<'a>(
    rows: &'a [GovernedE058Disposition],
    finding: &StaleDefaultRead,
) -> Option<&'a GovernedE058Disposition> {
    if !e058_table_is_valid(rows) {
        return None;
    }
    let mut matches = rows.iter().filter(|row| e058_row_matches(row, finding));
    match (matches.next(), matches.next()) {
        (Some(row), None) => Some(row),
        _ => None,
    }
}

fn exact_e059_disposition<'a>(
    rows: &'a [GovernedE059Disposition],
    finding: &UnresetFanIn,
) -> Option<&'a GovernedE059Disposition> {
    if !e059_table_is_valid(rows) {
        return None;
    }
    let mut matches = rows.iter().filter(|row| row.field == finding.field);
    match (matches.next(), matches.next()) {
        (Some(row), None) => Some(row),
        _ => None,
    }
}

fn e058_row_matches(row: &GovernedE058Disposition, finding: &StaleDefaultRead) -> bool {
    row.reader_rule == finding.reader_rule
        && row.binding_name == finding.binding_name
        && row.field == finding.field
}

fn disposition_metadata_is_valid(reason: &str, owner: &str, date: &str, adr: &str) -> bool {
    !reason.trim().is_empty()
        && owner == DISPOSITION_OWNER
        && date == DISPOSITION_DATE
        && adr == DISPOSITION_ADR
}

fn string_rows_are_unique(values: &[&str]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(index, value)| !values[..index].contains(value))
}

fn e058_table_is_valid(rows: &[GovernedE058Disposition]) -> bool {
    rows.len() == GOVERNED_E058_CARDINALITY
        && rows.iter().enumerate().all(|(index, row)| {
            disposition_metadata_is_valid(row.reason, row.owner, row.date, row.adr)
                && !row.allowed_writers.is_empty()
                && string_rows_are_unique(row.allowed_writers)
                && !rows[..index].iter().any(|prior| {
                    prior.reader_rule == row.reader_rule
                        && prior.binding_name == row.binding_name
                        && prior.field == row.field
                })
        })
}

fn e059_table_is_valid(rows: &[GovernedE059Disposition]) -> bool {
    rows.len() == GOVERNED_E059_CARDINALITY
        && rows.iter().enumerate().all(|(index, row)| {
            disposition_metadata_is_valid(row.reason, row.owner, row.date, row.adr)
                && !row.allowed_writers.is_empty()
                && string_rows_are_unique(row.allowed_writers)
                && earliest_rows_are_valid(row)
                && !rows[..index].iter().any(|prior| prior.field == row.field)
        })
}

fn earliest_rows_are_valid(row: &GovernedE059Disposition) -> bool {
    !row.allowed_earliest_writers.is_empty()
        && row
            .allowed_earliest_writers
            .iter()
            .enumerate()
            .all(|(index, allowed)| {
                row.allowed_writers.contains(&allowed.rule_id)
                    && !row.allowed_earliest_writers[..index]
                        .iter()
                        .any(|prior| prior == allowed)
            })
}

/// The four update-ops §2.8 admits on an `update-node` field (`set`/`add`/
/// `sub`/`scale`) — the write-side half of what a writer rule DOES to a
/// field, needed for refusal 2's "an unconditional **set**" test (`add`/
/// `sub`/`scale` never discharge it, however unconditional the rule is:
/// an unconditional `add` is still a fan-in contributor, not a reset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WriteOp {
    Set,
    Add,
    Sub,
    Scale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FieldWrite {
    field: String,
    op: WriteOp,
    direct_effect: bool,
}

impl WriteOp {
    fn parse(head: &str) -> Option<Self> {
        match head {
            "set" => Some(Self::Set),
            "add" => Some(Self::Add),
            "sub" => Some(Self::Sub),
            "scale" => Some(Self::Scale),
            _ => None,
        }
    }
}

/// Every `(field, op)` pair one `(effects …)` write site performs, walked
/// depth-first through arbitrary `guard`/`for-each` nesting.
///
/// Mirrors `structural_verbs::find_deferred_shape_verb`'s own `emit`-payload
/// discipline (payload LABELS are not verb invocations — see that
/// function's doc for the G1/H1 history this walk deliberately repeats
/// rather than risks re-breaking with a shallower recursion): once a form's
/// head is confirmed `emit`, only its payload items' VALUES are recursed
/// into, never the items themselves (whose first element is a label, not a
/// verb). Every other head — including `guard`, `for-each`, and any
/// arithmetic form — falls through to the generic per-child recursion,
/// which is what reaches an `update-node` buried under `for-each` > `guard`
/// > `guard` the way `solidarity/p0-transmit` nests one.
fn walk_error(limit: AstWalkLimit, maximum: usize) -> AstWalkError {
    AstWalkError::new(SAME_TICK_WALKER, limit, maximum)
}

fn write_child_depth(depth: usize, limits: AstWalkLimits) -> Result<usize, AstWalkError> {
    let child_depth = depth
        .checked_add(1)
        .ok_or_else(|| walk_error(AstWalkLimit::Depth, limits.depth()))?;
    if child_depth > limits.depth() {
        return Err(walk_error(AstWalkLimit::Depth, limits.depth()));
    }
    Ok(child_depth)
}

fn check_write_stack(
    current: usize,
    additional: usize,
    limits: AstWalkLimits,
) -> Result<(), AstWalkError> {
    let required = current
        .checked_add(additional)
        .ok_or_else(|| walk_error(AstWalkLimit::Stack, limits.stack()))?;
    if required > limits.stack() {
        return Err(walk_error(AstWalkLimit::Stack, limits.stack()));
    }
    Ok(())
}

fn push_write_children<'a>(
    stack: &mut Vec<(&'a SExpr, usize, bool)>,
    items: &'a [SExpr],
    depth: usize,
    limits: AstWalkLimits,
) -> Result<(), AstWalkError> {
    let children = items.get(1..).unwrap_or_default();
    if children.is_empty() {
        return Ok(());
    }
    let child_depth = write_child_depth(depth, limits)?;
    check_write_stack(stack.len(), children.len(), limits)?;
    for child in children.iter().rev() {
        stack.push((child, child_depth, false));
    }
    Ok(())
}

fn emit_write_value_count(items: &[SExpr], limits: AstWalkLimits) -> Result<usize, AstWalkError> {
    let mut count = 0_usize;
    for payload_item in items.iter().skip(2) {
        let SExpr::List(pair) = payload_item else {
            continue;
        };
        count = count
            .checked_add(pair.len().saturating_sub(1))
            .ok_or_else(|| walk_error(AstWalkLimit::Stack, limits.stack()))?;
    }
    Ok(count)
}

fn push_emit_write_values<'a>(
    stack: &mut Vec<(&'a SExpr, usize, bool)>,
    items: &'a [SExpr],
    depth: usize,
    limits: AstWalkLimits,
) -> Result<(), AstWalkError> {
    let value_count = emit_write_value_count(items, limits)?;
    if value_count == 0 {
        return Ok(());
    }
    let pair_depth = write_child_depth(depth, limits)?;
    let value_depth = write_child_depth(pair_depth, limits)?;
    check_write_stack(stack.len(), value_count, limits)?;
    for payload_item in items.iter().skip(2).rev() {
        let SExpr::List(pair) = payload_item else {
            continue;
        };
        for value in pair.iter().skip(1).rev() {
            stack.push((value, value_depth, false));
        }
    }
    Ok(())
}

fn walk_for_writes_with_limits(
    expr: &SExpr,
    limits: AstWalkLimits,
) -> Result<Vec<FieldWrite>, AstWalkError> {
    validate_ast_walk_bounds(expr, limits, SAME_TICK_WALKER)?;
    let mut out = Vec::new();
    let mut stack = vec![(expr, 0_usize, true)];
    for visited in 0..MAX_AST_WALK_NODES {
        if visited >= limits.nodes() {
            break;
        }
        let Some((current, depth, direct_effect)) = stack.pop() else {
            return Ok(out);
        };
        let SExpr::List(items) = current else {
            continue;
        };
        let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
            push_write_children(&mut stack, items, depth, limits)?;
            continue;
        };
        if head == "update-node" {
            if let [_, _target, SExpr::Atom(Atom::QName(field)), SExpr::List(op_form)] =
                items.as_slice()
            {
                if let Some(SExpr::Atom(Atom::Symbol(op_head))) = op_form.first() {
                    if let Some(op) = WriteOp::parse(op_head) {
                        out.push(FieldWrite {
                            field: field.clone(),
                            op,
                            direct_effect,
                        });
                    }
                }
            }
            continue;
        }
        if head == "emit" {
            if matches!(items.get(1), Some(SExpr::Atom(Atom::EnumRef { .. }))) {
                push_emit_write_values(&mut stack, items, depth, limits)?;
            } else {
                push_write_children(&mut stack, items, depth, limits)?;
            }
            continue;
        }
        push_write_children(&mut stack, items, depth, limits)?;
    }
    if stack.is_empty() {
        Ok(out)
    } else {
        Err(walk_error(AstWalkLimit::Nodes, limits.nodes()))
    }
}

/// Every write in the rule, with direct-effect execution proof retained.
fn field_writes(rule: &SExpr) -> Result<Vec<FieldWrite>, AstWalkError> {
    validate_ast_walk_bounds(rule, AST_WALK_LIMITS, SAME_TICK_WALKER)?;
    let SExpr::List(rule_items) = rule else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for child in rule_items {
        let SExpr::List(inner) = child else { continue };
        if !matches!(inner.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "effects") {
            continue;
        }
        for effect in &inner[1..] {
            out.extend(walk_for_writes_with_limits(effect, AST_WALK_LIMITS)?);
        }
    }
    Ok(out)
}

/// The rule's own `(when <expr>)` condition, or `None` when it declares no
/// top-level `when` at all (§2.3: `<when>?`, optional).
fn when_condition(rule_items: &[SExpr]) -> Option<&SExpr> {
    rule_items.iter().find_map(|child| match child {
        SExpr::List(inner)
            if matches!(inner.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "when") =>
        {
            inner.get(1)
        }
        _ => None,
    })
}

/// Rule-level half of adjudication §(d): no `(when …)` at all, or a `(when
/// #t)`. The write-site half is independent: only a direct child of the
/// rule's `(effects …)` sequence proves exactly-once execution. A write under
/// `guard`, `for-each`, `emit`, or any other nested form remains a writer but
/// cannot discharge E059 because its body can execute zero times.
fn is_unconditional(rule_items: &[SExpr]) -> bool {
    match when_condition(rule_items) {
        None | Some(SExpr::Atom(Atom::Bool(true))) => true,
        Some(_) => false,
    }
}

/// The D127 unconditional-recompute shape's discharge test, read the same
/// way the W2 pre-audit table itself established "0 [fields] with a
/// literal unconditional reset or D127 marker" — a literal `D127` citation
/// in the rule's own `:material-basis` prose, the citation-as-declaration
/// idiom this spec already uses pervasively (row 26's own material-basis
/// text cites D116 the same way). Comments are unavailable here — the
/// reader strips them before any `SExpr` exists (`canonical_ast`'s own
/// module doc) — so `:material-basis` is the only AST-visible citation
/// surface, and is the one the W2 pre-audit's own `rg -n D127` methodology
/// searched in the first place.
fn cites_d127(rule_items: &[SExpr]) -> bool {
    matches!(
        keyword_value(rule_items, "material-basis"),
        Some(Atom::Str(text)) if text.contains("D127")
    )
}

/// One rule's facts, precomputed once per [`diagnose_ranked`] call.
struct RuleFacts<'a> {
    id: &'a str,
    execution_rank: u16,
    bindings: Vec<crate::bindings::BindingDecl>,
    writes: Vec<FieldWrite>,
    unconditional: bool,
    d127: bool,
}

struct WriterIndex<'a> {
    ids_by_field: HashMap<String, HashSet<String>>,
    execution_ranks: HashMap<&'a str, u16>,
    direct_writes: HashSet<(String, String)>,
    direct_sets: HashSet<(String, String)>,
}

fn compare_ranked_rule_input_errors(
    left: &RankedRuleInputError,
    right: &RankedRuleInputError,
) -> std::cmp::Ordering {
    fn key(error: &RankedRuleInputError) -> (u8, &[u8], &[u8]) {
        match error {
            RankedRuleInputError::InvalidRuleForm { supplied_rule_id } => {
                (0, supplied_rule_id.as_bytes(), b"")
            }
            RankedRuleInputError::IdentityMismatch {
                supplied_rule_id,
                form_rule_id,
            } => (1, supplied_rule_id.as_bytes(), form_rule_id.as_bytes()),
            RankedRuleInputError::DuplicateRuleId { rule_id } => (2, rule_id.as_bytes(), b""),
        }
    }

    key(left).cmp(&key(right))
}

fn retain_first_ranked_rule_input_error(
    first: &mut Option<RankedRuleInputError>,
    candidate: RankedRuleInputError,
) {
    let replace = match first {
        Some(current) => compare_ranked_rule_input_errors(&candidate, current).is_lt(),
        None => true,
    };
    if replace {
        *first = Some(candidate);
    }
}

fn validate_ranked_rule_inputs(rules: &[RankedRule<'_>]) -> Option<RankedRuleInputError> {
    let mut seen = HashSet::with_capacity(rules.len());
    let mut first = None;
    for rule in rules {
        match crate::canonical_ast::rule_id(rule.form) {
            Ok(form_rule_id) if form_rule_id != rule.rule_id => {
                retain_first_ranked_rule_input_error(
                    &mut first,
                    RankedRuleInputError::IdentityMismatch {
                        supplied_rule_id: rule.rule_id.to_owned(),
                        form_rule_id: form_rule_id.to_owned(),
                    },
                );
            }
            Ok(_) => {}
            Err(_) => retain_first_ranked_rule_input_error(
                &mut first,
                RankedRuleInputError::InvalidRuleForm {
                    supplied_rule_id: rule.rule_id.to_owned(),
                },
            ),
        }
        if !seen.insert(rule.rule_id) {
            retain_first_ranked_rule_input_error(
                &mut first,
                RankedRuleInputError::DuplicateRuleId {
                    rule_id: rule.rule_id.to_owned(),
                },
            );
        }
    }
    first
}

fn collect_rule_facts<'a>(rules: &[RankedRule<'a>]) -> Result<Vec<RuleFacts<'a>>, AstWalkError> {
    let mut facts = Vec::with_capacity(rules.len());
    for rule in rules {
        let SExpr::List(items) = rule.form else {
            continue;
        };
        let Ok(bindings) = parse_bindings(rule.form) else {
            continue;
        };
        facts.push(RuleFacts {
            id: rule.rule_id,
            execution_rank: rule.execution_rank,
            writes: field_writes(rule.form)?,
            unconditional: is_unconditional(items),
            d127: cites_d127(items),
            bindings,
        });
    }
    Ok(facts)
}

impl<'a> WriterIndex<'a> {
    fn from_facts(facts: &[RuleFacts<'a>]) -> Self {
        let mut ids_by_field: HashMap<String, HashSet<String>> = HashMap::new();
        let mut direct_writes = HashSet::new();
        let mut direct_sets = HashSet::new();
        for rule in facts {
            for write in &rule.writes {
                ids_by_field
                    .entry(write.field.clone())
                    .or_default()
                    .insert(rule.id.to_owned());
                if write.direct_effect {
                    let key = (write.field.clone(), rule.id.to_owned());
                    direct_writes.insert(key.clone());
                    if write.op == WriteOp::Set {
                        direct_sets.insert(key);
                    }
                }
            }
        }
        Self {
            ids_by_field,
            execution_ranks: facts
                .iter()
                .map(|rule| (rule.id, rule.execution_rank))
                .collect(),
            direct_writes,
            direct_sets,
        }
    }
}

fn compare_stale_writer_sets(
    left: &StaleDefaultWriterSet,
    right: &StaleDefaultWriterSet,
) -> std::cmp::Ordering {
    (
        left.reader_rule.as_bytes(),
        left.binding_name.as_bytes(),
        left.field.as_bytes(),
    )
        .cmp(&(
            right.reader_rule.as_bytes(),
            right.binding_name.as_bytes(),
            right.field.as_bytes(),
        ))
}

fn diagnose_stale_defaults(
    facts: &[RuleFacts<'_>],
    writers: &WriterIndex<'_>,
) -> (Vec<StaleDefaultRead>, Vec<StaleDefaultWriterSet>) {
    let mut findings = Vec::new();
    let mut evidence = Vec::new();
    for rule in facts {
        for binding in &rule.bindings {
            if !binding.optional || binding.default.is_none() {
                continue;
            }
            let BindSource::Field(field) = &binding.source else {
                continue;
            };
            let Some(field_writers) = writers.ids_by_field.get(field) else {
                continue;
            };
            let mut offenders: Vec<&String> = field_writers
                .iter()
                .filter(|writer| {
                    if writer.as_str() == rule.id {
                        return false;
                    }
                    let writer_rank = writers
                        .execution_ranks
                        .get(writer.as_str())
                        .expect("every writer rule has one resolved execution rank");
                    *writer_rank > rule.execution_rank
                        || (*writer_rank == rule.execution_rank
                            && writer.as_bytes() >= rule.id.as_bytes())
                })
                .collect();
            offenders.sort_by(|left, right| {
                writers.execution_ranks[left.as_str()]
                    .cmp(&writers.execution_ranks[right.as_str()])
                    .then_with(|| left.as_bytes().cmp(right.as_bytes()))
            });
            if let Some(writer) = offenders.first() {
                findings.push(StaleDefaultRead {
                    reader_rule: rule.id.to_owned(),
                    binding_name: binding.name.clone(),
                    field: field.clone(),
                    writer_rule: (*writer).clone(),
                });
                evidence.push(StaleDefaultWriterSet {
                    reader_rule: rule.id.to_owned(),
                    binding_name: binding.name.clone(),
                    field: field.clone(),
                    writer_rules: offenders.into_iter().cloned().collect(),
                });
            }
        }
    }
    findings.sort_by(compare_stale_default_reads);
    evidence.sort_by(compare_stale_writer_sets);
    (findings, evidence)
}

fn diagnose_unreset_fan_ins(facts: &[RuleFacts<'_>], index: &WriterIndex<'_>) -> Vec<UnresetFanIn> {
    let mut findings = Vec::new();
    let mut fields: Vec<&String> = index.ids_by_field.keys().collect();
    fields.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    for field in fields {
        let mut writers: Vec<&String> = index.ids_by_field[field].iter().collect();
        writers.sort_by(|left, right| {
            index.execution_ranks[left.as_str()]
                .cmp(&index.execution_ranks[right.as_str()])
                .then_with(|| left.as_bytes().cmp(right.as_bytes()))
        });
        if writers.len() < 2 {
            continue;
        }
        let earliest = writers[0];
        let earliest_rule = facts
            .iter()
            .find(|rule| rule.id == earliest.as_str())
            .expect("a writer rule id always names a rule in `facts`");
        let key = (field.clone(), earliest.clone());
        let has_direct_write = index.direct_writes.contains(&key);
        let has_set = index.direct_sets.contains(&key);
        let d127_recompute = earliest_rule.d127 && has_direct_write;
        if !earliest_rule.unconditional || (!has_set && !d127_recompute) {
            findings.push(UnresetFanIn {
                field: field.clone(),
                writers: writers.into_iter().cloned().collect(),
                earliest_writer_semantics: EarliestWriterSemantics {
                    unconditional: earliest_rule.unconditional,
                    has_set,
                    d127_recompute,
                },
            });
        }
    }
    findings
}

/// All-rank-zero adapter for historical audits and same-rank fixtures.
///
/// New composition callers must use [`diagnose_ranked`]. This adapter preserves
/// D16's byte order for a set whose rules genuinely share one rank.
///
/// A rule whose `(bindings …)` cannot be parsed at all is silently
/// EXCLUDED from this analysis, never reported here — `rule_pipeline::
/// load_rule_form`'s own call to `bindings::parse_bindings` is what owns
/// that rejection; duplicating it here would be a second, redundant error
/// path for the same defect (this module's job is same-tick ordering, not
/// binding-surface validity).
///
#[must_use]
pub fn diagnose(rules: &[(String, SExpr)]) -> Diagnosis {
    let ranked: Vec<RankedRule<'_>> = rules
        .iter()
        .map(|(id, form)| RankedRule {
            rule_id: id,
            execution_rank: 0,
            form,
        })
        .collect();
    diagnose_ranked(&ranked)
}

/// Diagnose both refusals against rules already paired with their resolved
/// phase-schedule ranks.
///
/// No `LoadContext` is needed: both refusals are decidable from the rule forms
/// and compiled ranks, exactly like `E-LOAD-001`.
///
/// A rule whose `(bindings …)` cannot be parsed at all is silently excluded
/// from this analysis. `rule_pipeline::load_rule_form` owns that rejection;
/// duplicating it here would create a second error path for the same defect.
///
/// The ranked collection itself is validated first. A non-rule form, an
/// AST/supplied-ID mismatch, or a duplicate ID yields a typed refusal and no
/// findings, independent of input order.
///
/// # Panics
/// Never, in practice: writer ids and their execution ranks are inserted from
/// the same `RuleFacts` collection, so the internal rank and earliest-writer
/// lookups cannot miss.
#[must_use]
pub fn diagnose_ranked(rules: &[RankedRule<'_>]) -> Diagnosis {
    if let Some(error) = validate_ranked_rule_inputs(rules) {
        return Diagnosis {
            ranked_rule_input_error: Some(error),
            ..Diagnosis::default()
        };
    }
    let facts = match collect_rule_facts(rules) {
        Ok(facts) => facts,
        Err(error) => {
            return Diagnosis {
                ast_walk_error: Some(error),
                ..Diagnosis::default()
            };
        }
    };
    let writer_index = WriterIndex::from_facts(&facts);
    let (stale_default_reads, stale_default_writer_sets) =
        diagnose_stale_defaults(&facts, &writer_index);
    let unreset_fan_ins = diagnose_unreset_fan_ins(&facts, &writer_index);

    Diagnosis {
        ranked_rule_input_error: None,
        ast_walk_error: None,
        stale_default_reads,
        unreset_fan_ins,
        stale_default_writer_sets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_contract::MAX_AST_WALK_DEPTH;

    /// Parse a multi-rule content set through
    /// `rule_pipeline::split_content_unchecked` — the real production
    /// splitter's `(intrinsic …)`-segregation and `E-LOAD-001` duplicate-
    /// id enforcement, WITHOUT the same-tick-ordering gate — discarding
    /// the `(intrinsic …)` half, since the test adapters need only the paired
    /// `(rule_id, form)` list before assigning fixture ranks.
    ///
    /// W2 fix round 1 (review finding, discovered while building the
    /// corpus-wide inventory): an earlier version of this helper called
    /// `canonical_ast::rule_id` directly over every top-level form, which
    /// panicked on `decomposition.bsl`/`territory.bsl` — both declare a
    /// top-level `(intrinsic floor …)` form (§2.2), which is legal
    /// content the real splitter already knows to segregate.
    ///
    /// **W2 fix round 2 (review finding NEW-1): calling the GATED
    /// `split_content` from here, as the round-1 fix did, was self-
    /// refuting.** This module's own tests exist to MEASURE what the
    /// same-tick-ordering refusals say about the landed corpus —
    /// including, deliberately, cases the refusals reject (the RED
    /// fixtures). Aggregate enforcement now belongs after phase compilation;
    /// a `rules()` built on the gated per-source splitter would still analyze
    /// the wrong causal boundary and could reject before these audit assertions
    /// run. `split_content_unchecked` has no such gate. This is the ONE test
    /// helper in this module that constructs a content set from raw source, so
    /// it is the ONE place this distinction matters.
    fn rules(source: &str) -> Vec<(String, SExpr)> {
        crate::rule_pipeline::split_content_unchecked(source)
            .expect("test fixture must be a legal content set")
            .1
            .into_iter()
            .map(|rule| (rule.rule_id, rule.form))
            .collect()
    }

    fn diagnose_source_with<F>(source: &str, rank_for: F) -> Diagnosis
    where
        F: Fn(&str) -> u16,
    {
        let parsed = rules(source);
        let ranked: Vec<RankedRule<'_>> = parsed
            .iter()
            .map(|(id, form)| RankedRule {
                rule_id: id,
                execution_rank: rank_for(id),
                form,
            })
            .collect();
        diagnose_ranked(&ranked)
    }

    fn diagnose_same_rank(source: &str) -> Diagnosis {
        diagnose_source_with(source, |_| 0)
    }

    #[test]
    fn ranked_diagnosis_refuses_a_supplied_id_that_disagrees_with_the_form() {
        let parsed = rules(
            "(rule actual/id :material-basis \"identity fixture\" :fuel 10 \
             (bindings (seen :field ns/f :optional :default 0)) (effects)) \
             (rule actual/writer :material-basis \"writer fixture\" :fuel 10 \
             (bindings) (effects (update-node self ns/f (set 1))))",
        );
        let ranked = [
            RankedRule {
                rule_id: "forged/id",
                execution_rank: 0,
                form: &parsed[0].1,
            },
            RankedRule {
                rule_id: "actual/writer",
                execution_rank: 1,
                form: &parsed[1].1,
            },
        ];
        let diagnosis = diagnose_ranked(&ranked);
        let expected =
            SameTickOrderError::RankedRuleInput(RankedRuleInputError::IdentityMismatch {
                supplied_rule_id: "forged/id".to_owned(),
                form_rule_id: "actual/id".to_owned(),
            });

        assert!(diagnosis.stale_default_reads.is_empty());
        assert!(diagnosis.unreset_fan_ins.is_empty());
        assert_eq!(diagnosis.clone().into_result().unwrap_err(), expected);
        assert_eq!(diagnosis.into_enforced_result().unwrap_err(), expected);
    }

    #[test]
    fn ranked_diagnosis_refuses_duplicate_ids_independent_of_input_order() {
        let alpha = crate::reader::read(
            "(rule alpha/id :material-basis \"alpha fixture\" :fuel 1 \
             (bindings) (effects))",
        )
        .expect("alpha rule fixture must parse")
        .0;
        let zeta = crate::reader::read(
            "(rule zeta/id :material-basis \"zeta fixture\" :fuel 1 \
             (bindings) (effects))",
        )
        .expect("zeta rule fixture must parse")
        .0;
        let forward = [
            RankedRule {
                rule_id: "zeta/id",
                execution_rank: 1,
                form: &zeta,
            },
            RankedRule {
                rule_id: "alpha/id",
                execution_rank: 2,
                form: &alpha,
            },
            RankedRule {
                rule_id: "zeta/id",
                execution_rank: 3,
                form: &zeta,
            },
            RankedRule {
                rule_id: "alpha/id",
                execution_rank: 4,
                form: &alpha,
            },
        ];
        let reverse = [forward[3], forward[2], forward[1], forward[0]];
        let expected = SameTickOrderError::RankedRuleInput(RankedRuleInputError::DuplicateRuleId {
            rule_id: "alpha/id".to_owned(),
        });

        assert_eq!(
            diagnose_ranked(&forward).into_result().unwrap_err(),
            expected
        );
        assert_eq!(
            diagnose_ranked(&reverse).into_result().unwrap_err(),
            expected
        );
    }

    #[test]
    fn ranked_diagnosis_refuses_a_non_rule_form() {
        let form = SExpr::Atom(Atom::Int(0));
        let ranked = [RankedRule {
            rule_id: "invalid/form",
            execution_rank: 0,
            form: &form,
        }];

        assert_eq!(
            diagnose_ranked(&ranked).into_result().unwrap_err(),
            SameTickOrderError::RankedRuleInput(RankedRuleInputError::InvalidRuleForm {
                supplied_rule_id: "invalid/form".to_owned(),
            })
        );
    }

    #[test]
    fn aggregate_diagnosis_refuses_write_nesting_beyond_the_static_depth_bound() {
        let mut effect = "(update-node self ns/f (set 0))".to_owned();
        for _ in 0..=MAX_AST_WALK_DEPTH {
            effect = format!("(guard #t {effect})");
        }
        let source = format!(
            "(rule a/deep :material-basis \"depth refusal\" :fuel 1000000 \
             (bindings) (effects {effect}))"
        );

        let diagnosis = diagnose_same_rank(&source);
        assert!(!diagnosis.is_clean());
        assert_eq!(
            diagnosis.into_result().unwrap_err(),
            SameTickOrderError::AstWalkLimit(AstWalkError::new(
                "same-tick field writes",
                AstWalkLimit::Depth,
                MAX_AST_WALK_DEPTH,
            ))
        );
    }

    #[test]
    fn same_tick_walker_reports_each_bound_without_truncation() {
        let ast = crate::reader::read("(root (child) sibling)")
            .expect("fixture must parse")
            .0;
        for (limits, expected) in [
            (
                AstWalkLimits::new(0, 16, 16),
                AstWalkError::new("same-tick field writes", AstWalkLimit::Depth, 0),
            ),
            (
                AstWalkLimits::new(16, 16, 1),
                AstWalkError::new("same-tick field writes", AstWalkLimit::Stack, 1),
            ),
            (
                AstWalkLimits::new(16, 1, 16),
                AstWalkError::new("same-tick field writes", AstWalkLimit::Nodes, 1),
            ),
        ] {
            assert_eq!(
                walk_for_writes_with_limits(&ast, limits).unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn nested_control_flow_sets_cannot_discharge_e059() {
        let set = "(update-node self ns/f (set 0))";
        for nested in [
            format!("(guard #f {set})"),
            format!("(guard should-reset {set})"),
            format!("(for-each it (nodes NodeType/TERRITORY) {set})"),
        ] {
            let source = format!(
                "(rule a/reset :role mechanic :evidence derived \
                   :material-basis \"nested reset\" :fuel 64 \
                   (bindings (binding should-reset :expr #f)) (effects {nested}))
                 (rule b/add :role mechanic :evidence derived \
                   :material-basis \"fan in\" :fuel 64 \
                   (bindings) (effects (update-node self ns/f (add 1))))"
            );

            let diagnosis = diagnose_same_rank(&source);
            assert_eq!(diagnosis.unreset_fan_ins.len(), 1, "{diagnosis:?}");
            assert!(
                !diagnosis.unreset_fan_ins[0]
                    .earliest_writer_semantics
                    .has_set
            );
            assert_eq!(
                diagnosis.into_result().unwrap_err().spec_code(),
                Some("E-LOAD-059")
            );
        }
    }

    // ---- W2.1(a): reader-before-writer refuses -------------------------

    #[test]
    fn a_reader_before_writer_optional_default_pair_refuses() {
        // `a/reader` sorts before `b/writer` — the writer does NOT sort
        // strictly before the reader, so refusal 1 must fire.
        let source = r#"
(rule a/reader :material-basis "x" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set 1))))
(rule b/writer :material-basis "y" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 1))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert_eq!(diagnosis.stale_default_reads.len(), 1, "{diagnosis:?}");
        let finding = &diagnosis.stale_default_reads[0];
        assert_eq!(finding.reader_rule, "a/reader");
        assert_eq!(finding.writer_rule, "b/writer");
        assert_eq!(finding.field, "ns/f");
        assert_eq!(finding.binding_name, "v");
        let err = diagnosis.into_result().unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-058"));
        assert!(err.to_string().contains("a/reader"), "{err}");
        assert!(err.to_string().contains("b/writer"), "{err}");
        assert!(err.to_string().contains("ns/f"), "{err}");
    }

    /// The dual: a writer that DOES sort strictly before the reader must
    /// load clean under refusal 1.
    #[test]
    fn a_writer_sorting_before_the_reader_loads_clean() {
        let source = r#"
(rule a/writer :material-basis "x" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 1))))
(rule b/reader :material-basis "y" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set 1))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert!(diagnosis.stale_default_reads.is_empty(), "{diagnosis:?}");
        assert!(diagnosis.into_result().is_ok());
    }

    #[test]
    fn an_earlier_rank_writer_loads_clean_despite_a_later_lexical_id() {
        let source = r#"
(rule a/reader :material-basis "x" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set 1))))
(rule z/writer :material-basis "y" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 1))))
"#;
        let diagnosis = diagnose_source_with(source, |id| match id {
            "a/reader" => 20,
            "z/writer" => 10,
            other => panic!("unexpected rule {other}"),
        });

        assert!(diagnosis.stale_default_reads.is_empty(), "{diagnosis:?}");
    }

    #[test]
    fn a_later_rank_writer_refuses_despite_an_earlier_lexical_id() {
        let source = r#"
(rule a/writer :material-basis "x" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 1))))
(rule z/reader :material-basis "y" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set 1))))
"#;
        let diagnosis = diagnose_source_with(source, |id| match id {
            "a/writer" => 20,
            "z/reader" => 10,
            other => panic!("unexpected rule {other}"),
        });

        assert_eq!(diagnosis.stale_default_reads.len(), 1, "{diagnosis:?}");
        assert_eq!(diagnosis.stale_default_reads[0].writer_rule, "a/writer");
    }

    #[test]
    fn ranked_diagnosis_is_invariant_to_source_permutation() {
        let reader = r#"(rule a/reader :material-basis "x" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set 1))))"#;
        let writer = r#"(rule z/writer :material-basis "y" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 1))))"#;
        let rank = |id: &str| match id {
            "a/reader" => 10,
            "z/writer" => 20,
            other => panic!("unexpected rule {other}"),
        };

        let forward = diagnose_source_with(&format!("{reader}\n{writer}"), rank);
        let reversed = diagnose_source_with(&format!("{writer}\n{reader}"), rank);

        assert_eq!(forward, reversed);
    }

    // ---- W2.1(b): multi-writer, no earlier unconditional set, refuses --

    #[test]
    fn a_multi_writer_field_with_no_unconditional_set_refuses() {
        let source = r#"
(rule a/first :material-basis "x" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self ns/f (set 1))))
(rule b/second :material-basis "y" :fuel 10
  (bindings)
  (when (> 2 0))
  (effects (update-node self ns/f (add 1))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert_eq!(diagnosis.unreset_fan_ins.len(), 1, "{diagnosis:?}");
        let finding = &diagnosis.unreset_fan_ins[0];
        assert_eq!(finding.field, "ns/f");
        assert_eq!(
            finding.writers,
            vec!["a/first".to_owned(), "b/second".to_owned()]
        );
        let err = diagnosis.into_result().unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-059"));
        assert!(err.to_string().contains("ns/f"), "{err}");
    }

    /// The dual: an EARLIER unconditional `set` discharges refusal 2.
    #[test]
    fn an_earlier_unconditional_set_discharges_the_fan_in_refusal() {
        let source = r#"
(rule a/reset :material-basis "x" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 0))))
(rule b/accumulate :material-basis "y" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self ns/f (add 1))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert!(diagnosis.unreset_fan_ins.is_empty(), "{diagnosis:?}");
    }

    #[test]
    fn an_earlier_rank_reset_discharges_fan_in_despite_a_later_lexical_id() {
        let source = r#"
(rule a/accumulate :material-basis "x" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self ns/f (add 1))))
(rule z/reset :material-basis "y" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 0))))
"#;
        let diagnosis = diagnose_source_with(source, |id| match id {
            "a/accumulate" => 20,
            "z/reset" => 10,
            other => panic!("unexpected rule {other}"),
        });

        assert!(diagnosis.unreset_fan_ins.is_empty(), "{diagnosis:?}");
    }

    #[test]
    fn a_later_rank_reset_does_not_discharge_fan_in_despite_an_earlier_lexical_id() {
        let source = r#"
(rule a/reset :material-basis "x" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 0))))
(rule z/accumulate :material-basis "y" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self ns/f (add 1))))
"#;
        let diagnosis = diagnose_source_with(source, |id| match id {
            "a/reset" => 20,
            "z/accumulate" => 10,
            other => panic!("unexpected rule {other}"),
        });

        assert_eq!(diagnosis.unreset_fan_ins.len(), 1, "{diagnosis:?}");
        assert_eq!(
            diagnosis.unreset_fan_ins[0].writers,
            ["z/accumulate", "a/reset"]
        );
    }

    /// A rule with NO `(when …)` at all is unconditional too (not only a
    /// literal `(when #t)`).
    #[test]
    fn a_writer_with_no_when_clause_at_all_is_unconditional() {
        let source = r#"
(rule a/reset :material-basis "x" :fuel 10
  (bindings)
  (effects (update-node self ns/f (set 0))))
(rule b/accumulate :material-basis "y" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self ns/f (add 1))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert!(diagnosis.unreset_fan_ins.is_empty(), "{diagnosis:?}");
    }

    /// The D127-marked shape discharges refusal 2 even though its op is
    /// `add`, not `set` — the citation is the whole discharge, per this
    /// module's own doc.
    #[test]
    fn a_d127_cited_unconditional_writer_discharges_the_fan_in_refusal() {
        let source = r#"
(rule a/recompute :material-basis "D127 unconditional recompute" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (add 0))))
(rule b/also-writes :material-basis "y" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self ns/f (add 1))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert!(diagnosis.unreset_fan_ins.is_empty(), "{diagnosis:?}");
    }

    // ---- W2.1(c): row-37 self-exclusion, keyed on rule identity --------

    /// The row-37 shape: ONE rule reads field `f` on `self` and writes `f`
    /// on `it` — must LOAD, proving the self-exclusion is keyed on rule
    /// identity, not on write target (`self` vs `it`).
    #[test]
    fn the_row_37_shape_reads_self_writes_it_same_rule_loads_clean() {
        let source = r#"
(rule ns/only-rule :material-basis "x" :fuel 10
  (bindings (binding r :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node it ns/f (set r))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert!(
            diagnosis.stale_default_reads.is_empty(),
            "self-exclusion must be keyed on rule identity, not write target: {diagnosis:?}"
        );
        assert!(diagnosis.into_result().is_ok());
    }

    /// The mutation-check adjudication §(c) itself asks for: WITHOUT the
    /// self-exclusion, a rule that reads a field via `:optional :default`
    /// and also writes that SAME field to `self` would trip refusal 1 on
    /// almost every read-modify-write rule in the corpus. This fixture
    /// proves the positive control — a rule whose only "writer" of `f` is
    /// itself must load clean, matching `consciousness/p6-route`'s own
    /// shape (reads `r/l/f`, writes `r/l/f`, no OTHER writer).
    #[test]
    fn a_rule_that_only_writes_its_own_read_field_to_self_loads_clean() {
        let source = r#"
(rule ns/read-modify-write :material-basis "x" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/f (set v))))
"#;
        let diagnosis = diagnose_same_rank(source);
        assert!(diagnosis.stale_default_reads.is_empty(), "{diagnosis:?}");
    }

    #[test]
    fn an_optional_default_read_with_no_writer_loads_clean() {
        let source = r#"
(rule ns/reader :material-basis "x" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set v))))
"#;
        let diagnosis = diagnose_same_rank(source);

        assert!(diagnosis.stale_default_reads.is_empty(), "{diagnosis:?}");
        assert!(diagnosis.into_result().is_ok());
    }

    // ---- W2.4: the audit, against the real landed corpus -----------------

    const CONSCIOUSNESS_BSL: &str =
        include_str!("../../babylon-tick/content/rules/consciousness.bsl");
    const SOLIDARITY_BSL: &str = include_str!("../../babylon-tick/content/rules/solidarity.bsl");
    const DECOMPOSITION_BSL: &str =
        include_str!("../../babylon-tick/content/rules/decomposition.bsl");
    const CONTROL_RATIO_BSL: &str =
        include_str!("../../babylon-tick/content/rules/control-ratio.bsl");
    const PRODUCTION_BSL: &str = include_str!("../../babylon-tick/content/rules/production.bsl");
    const TERRITORY_BSL: &str = include_str!("../../babylon-tick/content/rules/territory.bsl");
    const VITALITY_BSL: &str = include_str!("../../babylon-tick/content/rules/vitality.bsl");
    const LIFECYCLE_BSL: &str = include_str!("../../babylon-tick/content/rules/lifecycle.bsl");
    const DISPOSSESSION_BSL: &str =
        include_str!("../../babylon-tick/content/rules/dispossession.bsl");
    const METABOLISM_BSL: &str = include_str!("../../babylon-tick/content/rules/metabolism.bsl");
    const ORGANIZATION_BSL: &str =
        include_str!("../../babylon-tick/content/rules/organization.bsl");
    const WORLDVIEW_BSL: &str = include_str!("../../babylon-tick/content/rules/worldview.bsl");
    const FUNDAMENTAL_THEOREM_BSL: &str =
        include_str!("../../babylon-tick/content/rules/fundamental-theorem.bsl");
    const COMMUNITY_BSL: &str = include_str!("../../babylon-tick/content/rules/community.bsl");
    const IMPERIAL_RENT_BSL: &str =
        include_str!("../../babylon-tick/content/rules/imperial-rent.bsl");
    const VITALITY_ATTRITION_BSL: &str =
        include_str!("../../babylon-tick/content/rules/vitality-attrition.bsl");
    const AUDITED_SOLO_PACKS: &[(&str, &str)] = &[
        ("consciousness", CONSCIOUSNESS_BSL),
        ("solidarity", SOLIDARITY_BSL),
        ("decomposition", DECOMPOSITION_BSL),
        ("control-ratio", CONTROL_RATIO_BSL),
        ("production", PRODUCTION_BSL),
        ("territory", TERRITORY_BSL),
        ("vitality", VITALITY_BSL),
        ("lifecycle", LIFECYCLE_BSL),
        ("dispossession", DISPOSSESSION_BSL),
        ("metabolism", METABOLISM_BSL),
        ("organization", ORGANIZATION_BSL),
        ("worldview", WORLDVIEW_BSL),
        ("fundamental-theorem", FUNDAMENTAL_THEOREM_BSL),
        ("imperial-rent", IMPERIAL_RENT_BSL),
        ("vitality-attrition", VITALITY_ATTRITION_BSL),
        ("community", COMMUNITY_BSL),
    ];
    const GOVERNED_DISPOSITION_PACKS: &[&str] = &[
        CONSCIOUSNESS_BSL,
        DECOMPOSITION_BSL,
        PRODUCTION_BSL,
        TERRITORY_BSL,
        COMMUNITY_BSL,
        IMPERIAL_RENT_BSL,
    ];

    fn append_fan_ins(
        findings: &mut Vec<(&'static str, String)>,
        pack: &'static str,
        source: &str,
    ) {
        findings.extend(
            diagnose_same_rank(source)
                .unreset_fan_ins
                .into_iter()
                .map(|finding| (pack, finding.field)),
        );
    }

    fn expected_corpus_fan_ins() -> Vec<(&'static str, String)> {
        vec![
            ("consciousness", "social-class/agitation"),
            ("consciousness", "social-class/fascist"),
            ("consciousness", "social-class/liberal"),
            ("consciousness", "social-class/revolutionary"),
            ("decomposition", "social-class/active"),
            ("decomposition", "social-class/population"),
            ("decomposition", "social-class/wealth"),
            ("decomposition+control-ratio", "social-class/active"),
            ("decomposition+control-ratio", "social-class/population"),
            ("decomposition+control-ratio", "social-class/wealth"),
            ("production", "social-class/production-value"),
            ("production", "social-class/wealth"),
            ("territory", "territory/population"),
            ("community", "social-class/community-cost-modifier"),
            (
                "community+control-ratio",
                "social-class/community-cost-modifier",
            ),
            (
                "community+solidarity",
                "social-class/community-cost-modifier",
            ),
            ("imperial-rent", "institution/rent-pool"),
            ("imperial-rent", "social-class/wealth"),
        ]
        .into_iter()
        .map(|(pack, field)| (pack, field.to_owned()))
        .collect()
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct E058InventoryRow {
        reader_rule: String,
        binding_name: String,
        field: String,
        writers: Vec<String>,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct E059InventoryRow {
        field: String,
        writers: Vec<String>,
        earliest: Vec<(String, bool, bool, bool)>,
    }

    type E059EarliestInventory = (String, bool, bool, bool);
    type E059Accumulator = (HashSet<String>, HashSet<E059EarliestInventory>);

    fn observed_e058_inventory() -> Vec<E058InventoryRow> {
        let mut inventory = Vec::new();
        for source in GOVERNED_DISPOSITION_PACKS {
            let diagnosis = diagnose_same_rank(source);
            inventory.extend(diagnosis.stale_default_writer_sets.into_iter().map(
                |mut evidence| {
                    evidence.writer_rules.sort();
                    E058InventoryRow {
                        reader_rule: evidence.reader_rule,
                        binding_name: evidence.binding_name,
                        field: evidence.field,
                        writers: evidence.writer_rules,
                    }
                },
            ));
        }
        inventory.sort();
        inventory
    }

    fn governed_e058_inventory(rows: &[GovernedE058Disposition]) -> Vec<E058InventoryRow> {
        let mut inventory: Vec<E058InventoryRow> = rows
            .iter()
            .map(|row| {
                let mut writers: Vec<String> = row
                    .allowed_writers
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                writers.sort();
                E058InventoryRow {
                    reader_rule: row.reader_rule.to_owned(),
                    binding_name: row.binding_name.to_owned(),
                    field: row.field.to_owned(),
                    writers,
                }
            })
            .collect();
        inventory.sort();
        inventory
    }

    fn observed_e059_inventory() -> Vec<E059InventoryRow> {
        let mut by_field: HashMap<String, E059Accumulator> = HashMap::new();
        for source in GOVERNED_DISPOSITION_PACKS {
            for finding in diagnose_same_rank(source).unreset_fan_ins {
                let semantics = finding.earliest_writer_semantics;
                let earliest_rule = finding
                    .writers
                    .first()
                    .expect("every E059 finding has at least two writers")
                    .clone();
                let entry = by_field.entry(finding.field).or_default();
                entry.0.extend(finding.writers);
                entry.1.insert((
                    earliest_rule,
                    semantics.unconditional,
                    semantics.has_set,
                    semantics.d127_recompute,
                ));
            }
        }
        e059_accumulator_inventory(by_field)
    }

    fn e059_accumulator_inventory(
        by_field: HashMap<String, E059Accumulator>,
    ) -> Vec<E059InventoryRow> {
        let mut inventory: Vec<E059InventoryRow> = by_field
            .into_iter()
            .map(|(field, (writers, earliest))| {
                let mut writers: Vec<String> = writers.into_iter().collect();
                let mut earliest: Vec<(String, bool, bool, bool)> = earliest.into_iter().collect();
                writers.sort();
                earliest.sort();
                E059InventoryRow {
                    field,
                    writers,
                    earliest,
                }
            })
            .collect();
        inventory.sort();
        inventory
    }

    fn governed_e059_inventory(rows: &[GovernedE059Disposition]) -> Vec<E059InventoryRow> {
        let mut inventory: Vec<E059InventoryRow> = rows
            .iter()
            .map(|row| {
                let mut writers: Vec<String> = row
                    .allowed_writers
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                let mut earliest: Vec<(String, bool, bool, bool)> = row
                    .allowed_earliest_writers
                    .iter()
                    .map(|allowed| {
                        (
                            allowed.rule_id.to_owned(),
                            allowed.semantics.unconditional,
                            allowed.semantics.has_set,
                            allowed.semantics.d127_recompute,
                        )
                    })
                    .collect();
                writers.sort();
                earliest.sort();
                E059InventoryRow {
                    field: row.field.to_owned(),
                    writers,
                    earliest,
                }
            })
            .collect();
        inventory.sort();
        inventory
    }

    /// Refusal 1, gate forced ON, against `consciousness.bsl` loaded SOLO
    /// (its own content set — the W2 pre-audit's own finding: every
    /// committed load path loads this pack alone). Must name EXACTLY the
    /// 13 EXPOSED rows of the pre-audit table (rows 3-9, 14-16, 18, 20,
    /// 22) — not 23, not 24: this simultaneously proves the self-exclusion
    /// (which would otherwise flip rows 23-26 too, per adjudication §(c))
    /// and pins the audit's own headline number.
    #[test]
    fn refusal_1_fires_on_exactly_the_13_exposed_bindings_of_consciousness_bsl() {
        let diagnosis = diagnose_same_rank(CONSCIOUSNESS_BSL);
        let mut got: Vec<(String, String)> = diagnosis
            .stale_default_reads
            .iter()
            .map(|f| (f.reader_rule.clone(), f.binding_name.clone()))
            .collect();
        got.sort();
        let mut want: Vec<(String, String)> = vec![
            ("consciousness/p0-position", "r"),
            ("consciousness/p0-position", "l"),
            ("consciousness/p0-position", "f"),
            ("consciousness/p1-inbox-reset", "r"),
            ("consciousness/p1-inbox-reset", "l"),
            ("consciousness/p1-inbox-reset", "f"),
            ("consciousness/p3-class-solidarity-push", "r"),
            ("consciousness/p5-agitation", "r"),
            ("consciousness/p5-agitation", "l"),
            ("consciousness/p5-agitation", "f"),
            ("consciousness/p5-agitation", "prev-wages"),
            ("consciousness/p5-agitation", "prev-wealth"),
            ("consciousness/p5-agitation", "agitation"),
        ]
        .into_iter()
        .map(|(r, b)| (r.to_owned(), b.to_owned()))
        .collect();
        want.sort();
        assert_eq!(got.len(), 13, "{got:#?}");
        assert_eq!(got, want);
    }

    /// `solidarity.bsl` loaded SOLO: refusal 1 must find ZERO violations —
    /// its two bindings are both NO-WRITER (one true, one self-excluded,
    /// row 37).
    #[test]
    fn refusal_1_is_silent_on_solidarity_bsl_loaded_solo() {
        let diagnosis = diagnose_same_rank(SOLIDARITY_BSL);
        assert!(diagnosis.stale_default_reads.is_empty(), "{diagnosis:?}");
    }

    /// Refusal 2 against `consciousness.bsl` (this crate's own checkout,
    /// POST the W2.5 repair — `p1-inbox-reset`'s guard is `(when #t)`):
    /// exactly the r/l/f/agitation complementary-guard class — four
    /// fields, per the W2.4 adjudicated expectation — and NOTHING on
    /// either inbox field. Before W2.5's repair this assertion FAILS (six
    /// fields fire, including `wages-inbox` — the real latent defect
    /// adjudication §(d) found — and `solidarity-inbox`, a false
    /// positive); this is the RED signal W2.5's content repair turns
    /// GREEN, not a check this module's own logic satisfies unassisted.
    #[test]
    fn refusal_2_fires_on_exactly_the_r_l_f_agitation_complementary_guard_class() {
        let diagnosis = diagnose_same_rank(CONSCIOUSNESS_BSL);
        let mut got: Vec<String> = diagnosis
            .unreset_fan_ins
            .iter()
            .map(|f| f.field.clone())
            .collect();
        got.sort();
        assert_eq!(
            got,
            vec![
                "social-class/agitation",
                "social-class/fascist",
                "social-class/liberal",
                "social-class/revolutionary",
            ],
            "{got:#?} — expected exactly the complementary-guard class; a \
             solidarity-inbox/wages-inbox appearance here means W2.5's \
             (when #t) repair on p1-inbox-reset has not landed (or regressed)"
        );
    }

    /// `solidarity.bsl` loaded SOLO: refusal 2 must find zero violations —
    /// its one multi-writer-eligible field (`revolutionary`) has only ONE
    /// writer rule within this solo content set (`solidarity/p0-transmit`
    /// itself), so it never reaches the 2-distinct-writers threshold.
    #[test]
    fn refusal_2_is_silent_on_solidarity_bsl_loaded_solo() {
        let diagnosis = diagnose_same_rank(SOLIDARITY_BSL);
        assert!(diagnosis.unreset_fan_ins.is_empty(), "{diagnosis:?}");
    }

    /// Refusal 2 can fire on any multi-writer field in any pack. This audit
    /// started with W2's 13 solo packs and two co-loads, then grew with the
    /// Imperial Rent, Vitality Attrition, and Community packs plus Community's
    /// two committed co-loads. The header-only `class-dynamics.bsl` is not a
    /// legal content set until it gains a rule. The current findings are the
    /// exact input to ADR224's governed E059 table: complementary or
    /// role-exclusive branches, conditional reset-then-accumulate, and durable
    /// economic or spatial stocks. Any inventory drift requires re-triage.
    #[test]
    fn refusal_2_inventory_over_the_whole_landed_corpus() {
        assert_eq!(
            AUDITED_SOLO_PACKS.len(),
            16,
            "sixteen landed rule-bearing packs are auditable as solo content sets"
        );

        let mut got: Vec<(&str, String)> = Vec::new();
        for (name, source) in AUDITED_SOLO_PACKS {
            append_fan_ins(&mut got, name, source);
        }
        let deco_cr = format!("{DECOMPOSITION_BSL}\n{CONTROL_RATIO_BSL}");
        append_fan_ins(&mut got, "decomposition+control-ratio", &deco_cr);
        let vit_life = format!("{VITALITY_BSL}\n{LIFECYCLE_BSL}");
        append_fan_ins(&mut got, "vitality+lifecycle", &vit_life);
        let community_solidarity = format!("{COMMUNITY_BSL}\n{SOLIDARITY_BSL}");
        append_fan_ins(&mut got, "community+solidarity", &community_solidarity);
        let community_control = format!("{COMMUNITY_BSL}\n{CONTROL_RATIO_BSL}");
        append_fan_ins(&mut got, "community+control-ratio", &community_control);
        got.sort();

        let mut want = expected_corpus_fan_ins();
        want.sort();

        assert_eq!(
            got, want,
            "corpus-wide refusal-2 inventory drifted from the W2 fix round 1 \
             classification — re-triage any new/missing row before trusting \
             this test again (report.md's Fix-round-1 section)"
        );
    }

    // ---- PER-19 / ADR224: governed aggregate enforcement --------------

    #[test]
    fn rank_aware_aggregate_enforcement_is_ready_for_the_post_compile_hook() {
        let enabled = std::hint::black_box(ENFORCE_RANK_AWARE_AGGREGATE_ORDERING);
        assert!(enabled);
    }

    #[test]
    fn governed_tables_have_exact_cardinality_unique_keys_and_provenance() {
        assert_eq!(GOVERNED_E058_DISPOSITIONS.len(), 13);
        assert_eq!(GOVERNED_E059_DISPOSITIONS.len(), 11);
        assert!(e058_table_is_valid(GOVERNED_E058_DISPOSITIONS));
        assert!(e059_table_is_valid(GOVERNED_E059_DISPOSITIONS));

        let e058_keys: HashSet<(&str, &str, &str)> = GOVERNED_E058_DISPOSITIONS
            .iter()
            .map(|row| (row.reader_rule, row.binding_name, row.field))
            .collect();
        let e059_keys: HashSet<&str> = GOVERNED_E059_DISPOSITIONS
            .iter()
            .map(|row| row.field)
            .collect();
        assert_eq!(e058_keys.len(), GOVERNED_E058_CARDINALITY);
        assert_eq!(e059_keys.len(), GOVERNED_E059_CARDINALITY);

        for row in GOVERNED_E058_DISPOSITIONS {
            assert!(!row.reason.trim().is_empty());
            assert_eq!(
                (row.owner, row.date, row.adr),
                ("Director", "2026-08-23", "ADR224")
            );
        }
        for row in GOVERNED_E059_DISPOSITIONS {
            assert!(!row.reason.trim().is_empty());
            assert_eq!(
                (row.owner, row.date, row.adr),
                ("Director", "2026-08-23", "ADR224")
            );
        }
    }

    #[test]
    fn governed_tables_are_bidirectionally_complete_for_landed_findings() {
        let observed_e058 = observed_e058_inventory();
        let governed_e058 = governed_e058_inventory(GOVERNED_E058_DISPOSITIONS);
        assert_eq!(observed_e058.len(), GOVERNED_E058_CARDINALITY);
        assert_eq!(
            observed_e058, governed_e058,
            "E058 keys and complete offender sets drifted"
        );

        let observed_e059 = observed_e059_inventory();
        let governed_e059 = governed_e059_inventory(GOVERNED_E059_DISPOSITIONS);
        assert_eq!(observed_e059.len(), GOVERNED_E059_CARDINALITY);
        assert_eq!(
            observed_e059, governed_e059,
            "E059 fields, unioned writers, or allowed earliest semantics drifted"
        );
    }

    #[test]
    fn every_current_landed_finding_has_an_exact_governed_disposition() {
        for source in GOVERNED_DISPOSITION_PACKS {
            let diagnosis = diagnose_same_rank(source);
            assert!(!diagnosis.is_clean(), "fixture must exercise a disposition");
            assert!(diagnosis.into_enforced_result().is_ok());
        }
    }

    #[test]
    fn duplicate_governed_keys_make_exact_lookup_default_deny() {
        let e058_row = GOVERNED_E058_DISPOSITIONS[0];
        let e058_finding = StaleDefaultRead {
            reader_rule: e058_row.reader_rule.to_owned(),
            binding_name: e058_row.binding_name.to_owned(),
            field: e058_row.field.to_owned(),
            writer_rule: e058_row.allowed_writers[0].to_owned(),
        };
        let e058_writers: Vec<String> = e058_row
            .allowed_writers
            .iter()
            .map(ToString::to_string)
            .collect();
        let mut duplicate_e058 = GOVERNED_E058_DISPOSITIONS.to_vec();
        duplicate_e058[GOVERNED_E058_CARDINALITY - 1] = e058_row;
        assert!(!e058_table_is_valid(&duplicate_e058));
        assert!(!e058_is_disposed_with(
            &duplicate_e058,
            &e058_finding,
            &e058_writers
        ));

        let rent_pool = diagnose_same_rank(IMPERIAL_RENT_BSL)
            .unreset_fan_ins
            .into_iter()
            .find(|finding| finding.field == "institution/rent-pool")
            .expect("the governed rent-pool finding must remain present");
        let mut duplicate_e059 = GOVERNED_E059_DISPOSITIONS.to_vec();
        duplicate_e059[GOVERNED_E059_CARDINALITY - 1] = GOVERNED_E059_DISPOSITIONS[0];
        assert!(!e059_table_is_valid(&duplicate_e059));
        assert!(!e059_is_disposed_with(&duplicate_e059, &rent_pool));
    }

    #[test]
    fn stale_governed_rows_fail_reverse_completeness_and_exact_lookup() {
        let original_e058 = GOVERNED_E058_DISPOSITIONS[0];
        let e058_finding = StaleDefaultRead {
            reader_rule: original_e058.reader_rule.to_owned(),
            binding_name: original_e058.binding_name.to_owned(),
            field: original_e058.field.to_owned(),
            writer_rule: original_e058.allowed_writers[0].to_owned(),
        };
        let e058_writers: Vec<String> = original_e058
            .allowed_writers
            .iter()
            .map(ToString::to_string)
            .collect();
        let mut stale_e058 = GOVERNED_E058_DISPOSITIONS.to_vec();
        stale_e058[0] = GovernedE058Disposition {
            reader_rule: "retired/reader",
            ..original_e058
        };
        assert!(e058_table_is_valid(&stale_e058));
        assert_ne!(
            observed_e058_inventory(),
            governed_e058_inventory(&stale_e058)
        );
        assert!(!e058_is_disposed_with(
            &stale_e058,
            &e058_finding,
            &e058_writers
        ));

        let original_e059 = GOVERNED_E059_DISPOSITIONS[0];
        let rent_pool = diagnose_same_rank(IMPERIAL_RENT_BSL)
            .unreset_fan_ins
            .into_iter()
            .find(|finding| finding.field == original_e059.field)
            .expect("the governed rent-pool finding must remain present");
        let mut stale_e059 = GOVERNED_E059_DISPOSITIONS.to_vec();
        stale_e059[0] = GovernedE059Disposition {
            field: "retired/field",
            ..original_e059
        };
        assert!(e059_table_is_valid(&stale_e059));
        assert_ne!(
            observed_e059_inventory(),
            governed_e059_inventory(&stale_e059)
        );
        assert!(!e059_is_disposed_with(&stale_e059, &rent_pool));
    }

    #[test]
    fn stale_governance_provenance_invalidates_the_whole_table() {
        let mut stale_e058 = GOVERNED_E058_DISPOSITIONS.to_vec();
        let original_e058 = stale_e058[0];
        stale_e058[0] = GovernedE058Disposition {
            reason: "",
            ..original_e058
        };
        assert!(!e058_table_is_valid(&stale_e058));

        let mut stale_e059 = GOVERNED_E059_DISPOSITIONS.to_vec();
        let original_e059 = stale_e059[0];
        stale_e059[0] = GovernedE059Disposition {
            adr: "ADR223",
            ..original_e059
        };
        assert!(!e059_table_is_valid(&stale_e059));
    }

    #[test]
    fn an_allowed_writer_subset_keeps_the_shared_wealth_disposition_stable() {
        for source in [DECOMPOSITION_BSL, PRODUCTION_BSL, IMPERIAL_RENT_BSL] {
            let diagnosis = diagnose_same_rank(source);
            let wealth = diagnosis
                .unreset_fan_ins
                .iter()
                .find(|finding| finding.field == "social-class/wealth")
                .expect("each pack must exercise its governed wealth writer subset");
            assert!(wealth.writers.len() < 7, "the row is a strict superset");
            assert!(diagnosis.into_enforced_result().is_ok());
        }
    }

    #[test]
    fn a_new_e058_writer_cannot_hide_behind_the_governed_display_writer() {
        let new_writer = r#"
(rule consciousness/p9-new-route :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self social-class/revolutionary (set 0.0p))))
"#;
        let source = format!("{CONSCIOUSNESS_BSL}\n{new_writer}");
        let diagnosis = diagnose_same_rank(&source);

        assert_eq!(
            diagnosis.stale_default_reads[0].writer_rule, "consciousness/p6-route",
            "the existing display writer remains byte-first"
        );
        let error = diagnosis.into_enforced_result().unwrap_err();
        assert_eq!(error.spec_code(), Some("E-LOAD-058"));
    }

    #[test]
    fn e058_writer_sets_cannot_drop_one_governed_writer() {
        let one = vec!["writer/a".to_owned()];
        let both = vec!["writer/a".to_owned(), "writer/b".to_owned()];
        let governed = ["writer/a", "writer/b"];

        assert!(!writer_sets_equal(&one, &governed));
        assert!(writer_sets_equal(&both, &governed));
    }

    #[test]
    fn a_new_e059_writer_is_not_absorbed_by_an_allowed_writer_superset() {
        let new_writer = r#"
(rule production/p9-new-wealth-flow :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self social-class/wealth (add 1))))
"#;
        let source = format!("{PRODUCTION_BSL}\n{new_writer}");
        let error = diagnose_same_rank(&source)
            .into_enforced_result()
            .unwrap_err();

        assert_eq!(error.spec_code(), Some("E-LOAD-059"));
        let SameTickOrderError::UnresetFanIn(finding) = error else {
            panic!("expected the undisposed wealth fan-in")
        };
        assert_eq!(finding.field, "social-class/wealth");
    }

    #[test]
    fn a_governed_e059_row_refuses_changed_earliest_writer_semantics() {
        let source = r#"
(rule community/c09-cost-modifier-reset :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self social-class/community-cost-modifier (add 1))))
(rule community/c10-cost-modifier-accumulate :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self social-class/community-cost-modifier (scale 1))))
"#;
        let error = diagnose_same_rank(source)
            .into_enforced_result()
            .unwrap_err();

        assert_eq!(error.spec_code(), Some("E-LOAD-059"));
    }

    #[test]
    fn a_governed_e059_row_refuses_a_different_execution_earliest_writer() {
        let source = r#"
(rule consciousness/p0-position :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self social-class/agitation (set 0))))
(rule consciousness/p6-route :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when (> 1 0))
  (effects (update-node self social-class/agitation (set 1))))
"#;
        let diagnosis = diagnose_source_with(source, |id| {
            if id == "consciousness/p6-route" {
                0
            } else {
                10
            }
        });
        let error = diagnosis.into_enforced_result().unwrap_err();

        assert_eq!(error.spec_code(), Some("E-LOAD-059"));
        let SameTickOrderError::UnresetFanIn(finding) = error else {
            panic!("the fixture has no optional-default reads")
        };
        assert_eq!(finding.field, "social-class/agitation");
        assert_eq!(finding.writers[0], "consciousness/p6-route");
    }

    #[test]
    fn an_e058_binding_near_miss_remains_default_deny() {
        let source = r#"
(rule consciousness/p5-agitation :material-basis "synthetic mutation" :fuel 10
  (bindings (binding renamed-r :field social-class/revolutionary :optional :default 0.0p))
  (when #t)
  (effects (update-node self social-class/other (set 1))))
(rule consciousness/p6-route :material-basis "synthetic mutation" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self social-class/revolutionary (set 0.0p))))
"#;
        let error = diagnose_same_rank(source)
            .into_enforced_result()
            .unwrap_err();

        assert_eq!(error.spec_code(), Some("E-LOAD-058"));
        let SameTickOrderError::StaleDefaultRead(finding) = error else {
            panic!("the binding near-miss must remain E058")
        };
        assert_eq!(finding.binding_name, "renamed-r");
    }

    #[test]
    fn enforced_error_selection_is_permutation_stable_and_byte_first() {
        let reader = r#"(rule zz/reader :material-basis "x" :fuel 10
  (bindings (binding v :field zz/f :optional :default 0))
  (when #t)
  (effects (update-node self zz/other (set 1))))"#;
        let writer = r#"(rule zz/writer :material-basis "x" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self zz/f (set 1))))"#;
        let forward = format!("{CONSCIOUSNESS_BSL}\n{reader}\n{writer}");
        let reverse = format!("{writer}\n{reader}\n{CONSCIOUSNESS_BSL}");

        let first = diagnose_same_rank(&forward)
            .into_enforced_result()
            .unwrap_err();
        let second = diagnose_same_rank(&reverse)
            .into_enforced_result()
            .unwrap_err();
        assert_eq!(first, second);
        let SameTickOrderError::StaleDefaultRead(finding) = first else {
            panic!("E058 retains precedence over E059")
        };
        assert_eq!(finding.reader_rule, "zz/reader");
    }
}
