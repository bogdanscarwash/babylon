//! The typed structural verb algebra (`bsl-language.rst` §2.8): the seven
//! graph verbs plus `emit`, executed against any
//! [`babylon_graph::substrate::GraphSubstrate`] — generic over the store by
//! design, so this module needed no change when the production store
//! swapped from `MemoryGraph` to `HypergraphStore` at the Phase 1/2
//! boundary (ADR179 T3, executed by ADR193, 2026-08-11). This is the
//! crate-DAG edge Task 11 planned: `babylon-bsl` depends on `babylon-graph`.
//!
//! **No clique expansion exists in this module** and none may be added: a
//! member list is handed to `GraphSubstrate::add_hyperedge` whole — that is
//! Anti-Pattern VIII.9 enforced where the verbs live. There is deliberately
//! no `add-member`/`remove-member`: membership change is whole-hyperedge
//! replacement, `remove-hyperedge` then `add-hyperedge` in one effect list
//! (§2.8 draft ruling, D26 — whose member-list half **stands**).
//!
//! **`update-edge` is served (T3, ADR198 R1-R3, issue #560);
//! `update-hyperedge` is served too (Community port train, Task 6 E2b —
//! the charter T3's doc said did not exist).** T3 PR A gave
//! `GraphSubstrate` full symmetric edge-attribute storage (deffield rows per
//! edge type, the empty-elided fifth canonical section), and this module's
//! collect-then-apply machinery widened to match: `update-edge` defers
//! through the same [`PendingWrite`] batch as `update-node` (the target sum
//! type [`WriteTarget`]), with `set`/`add`/`sub`/`scale` parity, enum set
//! included, and the strength fork (D143) routing `<edge-type>/strength`
//! writes to the edge's existing 0x03-slot strength. The Community port
//! train then chartered the hyperedge half outright: Task 5 (E2a) minted
//! `GraphSubstrate::update_hyperedge_attribute` (the seventh canonical
//! section, `0x07`), and Task 6 wired the verb to it through BOTH dispatch
//! sites — `update-hyperedge` defers through `WriteTarget::Hyperedge` with
//! the same `set`/`add`/`sub`/`scale` parity, enum set included, the
//! §2.10 discipline-1 referent check (`evaluator::check_hyperedge_referent_type`)
//! shared with `field-of`'s new `HyperedgeRef` arm. What STAYS refused:
//! per-membership payloads (Amendment AG(i)'s ceremony, #653) and
//! mint-time `<field-init>`s on `add-hyperedge` (a distinct unserved lane —
//! the storage exists; the init sugar is not routed to it in this train).
//! The grammar, the §3.7 cost rows, the §2.8 static
//! checks and the error codes landed with the R9 chapters; the storage and
//! the apply path are what T3 added for edges and Task 5+6 added for
//! hyperedges.
//!
//! **I.15 stays a declared Phase-2 gap, named here rather than silently
//! absorbed** (the dossier's scope tension, recorded): nothing in this
//! module enforces the edge-mode transition law, so no E-EVAL-030 can fire
//! — the §6.2 chapter-C2 vector family's I.15 leg is unserved until the
//! machine itself is chartered.
//!
//! **`update-node` against a `currency`-declared field writes the i128
//! lane** (T3 #491, OQ-J — Half 2 of the typed-attribute-seeding design):
//! `update_node`/`collect_update_node`/[`EffectExecutor::apply_pending_write`]
//! each fork on the field's declared type BEFORE reaching
//! `numeric_write_value`'s f64 lane, routing a `Value::Currency` through
//! `GraphSubstrate::update_node_currency` instead — a SEPARATE store map,
//! never a lossy cast. Only `set` is licensed on the Currency lane;
//! `add`/`sub`/`scale` would need to pick which of Currency's five legal
//! operators (§3.2) applies, which this train does not license (mirrors
//! `refuse_arithmetic_on_enum_field`'s identical narrowness for `Enum<T>`).
//! **`update-edge` against a `currency`-declared
//! field is still refused** — there is no edge-scoped Currency lane in
//! this train, the same declared gap it always was on that side.
//!
//! **Id operands are effect-list-scoped names** (draft ruling recorded in
//! §2.8, implementation-discovered): `add-node`/`add-hyperedge`'s id
//! operand is read as a symbol naming the minted object for the rest of the
//! effect list — the substrate mints the actual identity; roster
//! replacement referencing the new hyperedge needs exactly this.
//!
//! Discipline held here, not invented here:
//! - Effects apply in **source order** (§2.8); `guard` evaluates only the
//!   taken branch (§4.1) and charges accordingly.
//! - Substrate failures surface as `E-EVAL-031` — removing what does not
//!   exist, adding what exists, unknown/duplicate members: absence is never
//!   success, and nothing is silently deduplicated.
//! - The **store boundary** runs §3.3's one range check: a written value
//!   outside the target field's declared `[0,1]` domain is `E-EVAL-020`, a
//!   loud failure, never a clamp.
//! - Fuel: verbs charge their §3.7 base cost (3), update-ops theirs (1),
//!   operand expressions charge through the Task 14 evaluator — one §4.5
//!   meter end to end.

use crate::evaluator::{
    charge, check_edge_referent_type, check_node_referent_type, evaluate, require_graph, EvalCode,
    EvalEnv, EvalError, Value,
};
use crate::fuel::cost;
use crate::intrinsic_host::IntrinsicHost;
use crate::query::EdgeKey;
use crate::reader::{Atom, SExpr};
use crate::typecheck::TypeEnv;
use crate::types::{BslType, EnumRegistry, EnumTypeId};
use crate::vocabulary::ClosedVocabulary;
use crate::write_log::{Write, WriteObserver, WriteRecord};
use babylon_graph::substrate::{GraphError, GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::Currency;
use std::collections::HashMap;

/// Where `emit` lands (§2.8): an event sink the engine wires to the kernel
/// event bus (Phase 3). Payload values are already evaluated.
pub trait EventSink {
    /// Record one emitted event.
    fn emit(&mut self, event_type: &str, payload: Vec<(String, Value)>);
}

/// A sink that simply collects, for tests and the conformance corpus.
#[derive(Debug, Default)]
pub struct CollectingSink {
    /// Every event emitted, in source order.
    pub events: Vec<(String, Vec<(String, Value)>)>,
}

impl EventSink for CollectingSink {
    fn emit(&mut self, event_type: &str, payload: Vec<(String, Value)>) {
        self.events.push((event_type.to_owned(), payload));
    }
}

/// `update-node`'s four update-ops (§2.8), carried by a [`PendingWrite`]
/// rather than pre-combined: `add`/`sub`/`scale` need the target's CURRENT
/// value as of APPLY time, not collect time (Task 12, D-row Q2), so the
/// combine cannot happen until [`EffectExecutor::apply_pending_write`] runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOp {
    /// `(set <expr>)` — the operand IS the final value.
    Set,
    /// `(add <expr>)` — combine by addition at apply time.
    Add,
    /// `(sub <expr>)` — combine by subtraction at apply time.
    Sub,
    /// `(scale <expr>)` — combine by multiplication at apply time.
    Scale,
}

/// One collected, not-yet-applied `update-node`/`update-edge` mutation
/// (Task 12, P27 Phase 2 query-evaluation plan, §4.2 chapter C4 + §2.8
/// chapter C6; widened to edge targets by T3, ADR198 R3, issue #560). The
/// evaluator has ALREADY reduced the operand expression against the rule's
/// PRE-STATE during collection (`EffectExecutor::collect_effects`); the
/// accumulating ops read the target's CURRENT value at APPLY time (D-row
/// Q2) — §4.2's carrier-accumulation clause is only satisfiable that way:
/// reading the target at collect time would make three subjects each
/// adding to one carrier lose two of the three contributions.
///
/// **Scope.** `update-node` and `update-edge` defer via this type — the
/// target is the sum type [`WriteTarget`], so a single flat batch carries
/// node and edge writes INTERLEAVED in collection order (the application
/// law below forbids any reordering, which rules out a parallel
/// edge-write batch: two batches cannot represent "node write, then edge
/// write, then node write"). Every other effect
/// kind is unaffected by Task 12: `emit` never touched the graph and still
/// fires during collection (its payload evaluates against the SAME
/// pre-state, matching §2.8's own worked `for-each` example, whose `emit`
/// reads the PRE-scale `solidarity/strength`); the six graph-shape verbs
/// (`add-node`, `remove-node`, `add-edge`, `remove-edge`, `add-hyperedge`,
/// `remove-hyperedge`) remain served only through
/// [`EffectExecutor::execute_effects`], the immediate-apply path this
/// module keeps unchanged — see [`EffectExecutor::collect_effects`]'s own
/// doc for why deferring them is out of this task's scope.
///
/// **The algebra, named (CT4P B1, issue #525).** The collected batch —
/// `Vec<PendingWrite>`, `tick.rs`'s `all_pending` — is the **free monoid**
/// on writes: list concatenation, associative, the empty batch its unit,
/// order and multiplicity both meaningful data. Its **application**
/// ([`EffectExecutor::apply_pending_write`]) is a DIFFERENT structure — a
/// **monoid action on graph state**, not a fold in that monoid: because
/// `add`/`sub`/`scale` read the target's CURRENT value at APPLY time (D-row
/// Q2, above), the batch acts as a sequence of **endomorphisms composed
/// left-to-right**, and `Add`/`Scale` do not commute. Reordering a batch
/// changes the result even though the batch itself — the free monoid
/// element — is unchanged. Consequence for any future optimiser: it **may**
/// re-chunk the COLLECTION phase (concatenation is associative); it **may
/// NOT** reorder the APPLICATION phase (the action is not commutative).
/// "Monoid" alone is exactly the word that would license the reordering
/// this distinction forbids.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingWrite {
    /// The write's target, already resolved (a computed `NodeRef`/`EdgeRef`
    /// resolves the same way whether the write applies immediately or is
    /// collected).
    pub target: WriteTarget,
    /// The declared field qname.
    pub field: String,
    /// Which of the four update-ops.
    pub op: UpdateOp,
    /// The reduced operand (T3 #491, OQ-J widened this from a bare `f64`):
    /// `set`'s final value, or the amount `add`/`sub`/`scale` combine with
    /// the target's CURRENT value at apply time, in EITHER the binary64
    /// lane or the i128 Currency lane.
    pub operand: WriteOperand,
}

/// The reduced operand a [`PendingWrite`] carries forward from collect to
/// apply (T3 #491, OQ-J). One flat `Vec<PendingWrite>` batch still carries
/// BOTH lanes in collection order — a Currency `set` sits in the identical
/// position an f64 `set` would, so [`PendingWrite`]'s own documented
/// ordering law is unaffected: this widens WHAT a write carries, never the
/// sequence it carries in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WriteOperand {
    /// The binary64 lane — every declared type except `currency`.
    Real(f64),
    /// The i128 lane — `currency`-declared node fields only, `set` only
    /// (no read-modify-write; see `update_node_currency_op`).
    Currency(Currency),
}

/// What a [`PendingWrite`] writes to (T3, ADR198 R3, issue #560). A sum
/// type rather than a widened id, so one flat `Vec<PendingWrite>` batch
/// preserves collection order across node and edge writes — the batch's
/// documented application law (above) forbids reordering, and only a
/// single interleaved sequence can honor it.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteTarget {
    /// A node's declared field (`update-node`).
    Node(NodeId),
    /// A dyadic edge's declared field (`update-edge`) — T2's `EdgeKey`
    /// (issue #559), shared unmodified exactly as D36's "the two trains
    /// share the type" intends.
    Edge(EdgeKey),
    /// A hyperedge's own field (`update-hyperedge`) — the Community port
    /// train's Task 6 (E2b); the id alone is the identity (VIII.9).
    Hyperedge(HyperedgeId),
}

fn plain(message: impl Into<String>) -> EvalError {
    EvalError::plain(message)
}

fn from_graph(e: GraphError) -> EvalError {
    // Every GraphSubstrate failure is the §2.8 existence discipline:
    // absence is never success, presence is never overwritten, a member
    // list is a set (E-EVAL-031).
    EvalError::coded(EvalCode::ExistenceDiscipline, e.message)
}

/// Canonicalizes −0.0 to +0.0 at the store boundary: the canonical state
/// hash encodes raw binary64 bits, so −0.0 and +0.0 would be OBSERVABLY
/// different states for the same number — III.7's decidable equality
/// admits no such fork. (`v == 0.0` holds for both zeros; the arm then
/// yields the canonical +0.0 bit pattern.) Every write lane funnels
/// through here: the direct update paths and the collect-then-apply
/// APPLY phase must agree bit-for-bit.
fn canonical_zero(v: f64) -> f64 {
    if v == 0.0 {
        0.0
    } else {
        v
    }
}

/// The `add`/`sub`/`scale` combine step [`EffectExecutor::apply_pending_write`]'s
/// Node and Edge arms both perform, extracted here because the two arms
/// were byte-identical apart from which verb name they cited (T3 #491,
/// OQ-J pulled this out incidentally while keeping `apply_pending_write`
/// under the ≤100-line bound, but the duplication removal stands on its
/// own). Never called for `UpdateOp::Set` — the caller handles `set`
/// before reaching here.
fn combine_and_check_finite(
    op: UpdateOp,
    current: f64,
    operand: f64,
    field: &str,
    verb: &str,
) -> Result<f64, EvalError> {
    let combined = match op {
        UpdateOp::Add => current + operand,
        UpdateOp::Sub => current - operand,
        UpdateOp::Scale => current * operand,
        UpdateOp::Set => unreachable!("Set is handled by the caller before combining"),
    };
    if !combined.is_finite() {
        return Err(EvalError::coded(
            EvalCode::NonFinite,
            format!("{verb} on {field} produced a non-finite value"),
        ));
    }
    Ok(combined)
}

/// Executes one rule's effect list against a substrate, carrying the
/// effect-list-scoped names `add-node`/`add-hyperedge` introduce.
pub struct EffectExecutor<'a> {
    types: &'a TypeEnv,
    /// **§2.13 addendum (D101).** The content-declared closed-enum
    /// registry — `numeric_write_value`'s enum branch resolves a written
    /// `<enum-ref>`'s ordinal through this, exactly as `bind_subject`
    /// (`tick.rs`) resolves the read-path inverse. An empty registry
    /// (`EnumRegistry::default()`) is the honest "no `defenum`s in scope"
    /// input for content with none — every enum-typed `store_range_check`/
    /// `numeric_write_value` site is unreachable without a
    /// `BslType::Enum`-declared field in `types` first, so an empty
    /// registry never silently under-serves real content.
    enums: &'a EnumRegistry,
    /// Task 8 (Organization foundation plan): the closed graph vocabulary,
    /// threaded exactly as `enums` above (Task 5's precedent). `None` is
    /// today's unchecked behavior for every EXISTING caller — and, in
    /// production, for `tick.rs::run_tick`'s two construction sites
    /// unconditionally: the three MINTING verbs this field gates
    /// (`add_node`/`add_edge`/`add_hyperedge`, via
    /// [`Self::enum_member_checked`]) are refused at LOAD TIME,
    /// unconditionally, by `rule_pipeline::check_no_deferred_shape_verbs`
    /// for every rule reaching `run_tick`, so no rule can ever exercise
    /// this field there regardless of what it is threaded to. It exists
    /// for the crate's own direct-execution callers
    /// (`execute_effects`/`execute_item`, this module's unit tests,
    /// `conformance_corpus.rs`) and for whenever that gate lifts.
    vocabulary_registry: Option<&'a ClosedVocabulary>,
    declared_nodes: HashMap<String, NodeId>,
    declared_hyperedges: HashMap<String, HyperedgeId>,
    /// The ADR182 R1 interception point. `None` is the unobserved path and
    /// does no observer work at all.
    observer: Option<&'a mut dyn WriteObserver>,
    /// The rule id every record from this executor is attributed to.
    attribution: String,
    /// Source-order position of the NEXT write. The wider internal lane lets
    /// the executor detect the public `u32` boundary before mutating the
    /// substrate, rather than panicking in debug builds or wrapping in
    /// release builds. Counts writes performed, not effect items considered.
    next_write_ordinal: u64,
}

impl<'a> EffectExecutor<'a> {
    /// A fresh executor for one effect list. `types` supplies the declared
    /// field types the §3.3 store-boundary range check needs; `enums`
    /// supplies the §2.13 enum-ordinal registry a `BslType::Enum`-declared
    /// field's write path resolves against; `vocabulary_registry` is the
    /// §3.6 closed graph vocabulary a minting verb's type-operand is
    /// checked against, when one is threaded (Task 8, Organization
    /// foundation plan — see this struct's own field doc for why `None`
    /// changes nothing observable in production today).
    #[must_use]
    pub fn new(
        types: &'a TypeEnv,
        enums: &'a EnumRegistry,
        vocabulary_registry: Option<&'a ClosedVocabulary>,
    ) -> Self {
        Self {
            types,
            enums,
            vocabulary_registry,
            declared_nodes: HashMap::new(),
            declared_hyperedges: HashMap::new(),
            observer: None,
            attribution: String::new(),
            next_write_ordinal: 0,
        }
    }

    /// The same executor with the ADR182 R1 write log installed, attributing
    /// every record to `rule` (the `<system>/<rule-name>` qname). One
    /// executor runs one rule's effect list, so attribution is fixed at
    /// construction rather than carried per write.
    ///
    /// Observation MUST NOT change what the executor does: an observed and
    /// an unobserved run of the same effect list leave identical graph state
    /// and consume identical fuel. See `write_log`'s module doc.
    #[must_use]
    pub fn observed(
        types: &'a TypeEnv,
        enums: &'a EnumRegistry,
        vocabulary_registry: Option<&'a ClosedVocabulary>,
        rule: impl Into<String>,
        observer: &'a mut dyn WriteObserver,
    ) -> Self {
        Self {
            types,
            enums,
            vocabulary_registry,
            declared_nodes: HashMap::new(),
            declared_hyperedges: HashMap::new(),
            observer: Some(observer),
            attribution: rule.into(),
            next_write_ordinal: 0,
        }
    }

    /// Refuse a write whose observation ordinal cannot be represented.
    /// Every mutation site calls this before touching the substrate; the
    /// recorder repeats the check defensively before publishing the record.
    fn ensure_record_capacity(&self) -> Result<(), EvalError> {
        u32::try_from(self.next_write_ordinal)
            .map(|_| ())
            .map_err(|_| {
                EvalError::coded(
                    EvalCode::Overflow,
                    "write observation ordinal exceeds the u32 domain",
                )
            })
    }

    /// Hand one completed mutation to the observer, if any. Call sites are
    /// AFTER the substrate accepted the write, never before: a write that
    /// failed leaves no record.
    fn record(&mut self, write: Write) -> Result<(), EvalError> {
        self.ensure_record_capacity()?;
        let ordinal = u32::try_from(self.next_write_ordinal).map_err(|_| {
            EvalError::coded(
                EvalCode::Overflow,
                "write observation ordinal exceeds the u32 domain",
            )
        })?;
        if let Some(observer) = self.observer.as_mut() {
            observer.record(WriteRecord {
                rule: self.attribution.clone(),
                ordinal,
                write,
            });
        }
        self.next_write_ordinal = self.next_write_ordinal.checked_add(1).ok_or_else(|| {
            EvalError::coded(
                EvalCode::Overflow,
                "write observation ordinal counter exceeded u64",
            )
        })?;
        Ok(())
    }

    /// The prior value of a field, for the log only — a failed probe means
    /// the field held nothing, which is exactly what `None` records. Never
    /// call this to make a decision (see `write_log`'s discipline 3).
    fn probe_previous(&self, graph: &dyn GraphSubstrate, id: NodeId, field: &str) -> Option<f64> {
        self.observer.as_ref()?;
        graph.node_attribute(id, field).ok()
    }

    /// The edge half of [`Self::probe_previous`] (T3, ADR198 R3) — same
    /// discipline through `edge_attribute`: a never-written edge field (or a
    /// never-minted edge) records `None`, and this is still never called to
    /// make a decision.
    fn probe_previous_edge(
        &self,
        graph: &dyn GraphSubstrate,
        key: &EdgeKey,
        field: &str,
    ) -> Option<f64> {
        self.observer.as_ref()?;
        graph
            .edge_attribute(&key.edge_type, key.source, key.target, field)
            .ok()
    }

    /// The Currency-lane half of [`Self::probe_previous`] (T3 #491, OQ-J) —
    /// same discipline through `node_attribute_currency`: a never-written
    /// currency field records `None`, and this is still never called to
    /// make a decision.
    fn probe_previous_currency(
        &self,
        graph: &dyn GraphSubstrate,
        id: NodeId,
        field: &str,
    ) -> Option<Currency> {
        self.observer.as_ref()?;
        graph.node_attribute_currency(id, field).ok()
    }

    /// Execute the items of an `(effects …)` form in source order (§2.8),
    /// applying each write IMMEDIATELY — the single-pass model.
    ///
    /// **NOT a production path (#519 fix round, fix 7).** No production
    /// driver has called this method since Task 12: `run_tick`
    /// (`tick.rs`) calls [`Self::collect_effects`] then
    /// [`Self::apply_pending_write`], the collect-then-apply split §4.2
    /// chapter C4's pre-state law requires. `execute_effects` survives
    /// because two things still legitimately need the immediate-apply
    /// model rather than the deferred one: this crate's OWN unit tests
    /// (verb-level correctness, write-log discipline, error messaging —
    /// none of which depend on collect-vs-apply staging) and
    /// `babylon-bsl/tests/conformance_corpus.rs`'s
    /// `bifurcation_routes_by_solidarity_density`, which needs no
    /// `env.graph` and applies one subject's effects once. A test meaning
    /// to prove something about `run_tick`'s ACTUAL pre-state/subject-order
    /// guarantees must not use this method or `Self::for_each` below —
    /// see `structural_verbs::tests::collect_then_apply`, or drive
    /// `run_tick` directly.
    ///
    /// # Errors
    ///
    /// Any [`EvalError`] an operand expression raises, `E-EVAL-031` from
    /// the substrate's existence discipline, `E-EVAL-020` from the store
    /// boundary, `E-EVAL-040` from the shared fuel meter, and loud uncoded
    /// errors for shapes off the §2.8 grammar.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_effects(
        &mut self,
        effect_items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        for item in effect_items {
            self.execute_item(item, env, host, graph, sink, fuel)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_item(
        &mut self,
        item: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        let SExpr::List(items) = item else {
            return Err(plain(format!(
                "an effect item must be a form, found {item:?}"
            )));
        };
        let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
            return Err(plain(format!(
                "an effect item must be a verb or guard form, found {:?}",
                items.first()
            )));
        };
        match head.as_str() {
            "guard" => {
                // cost(guard) base, then only the taken branch (§4.1).
                charge(fuel, cost::GUARD_BASE)?;
                let [_, cond, nested @ ..] = items.as_slice() else {
                    return Err(plain("(guard <cond> <effect-item>+) — missing condition"));
                };
                if nested.is_empty() {
                    return Err(plain("(guard …) requires at least one effect item"));
                }
                let taken = crate::evaluator::as_bool(evaluate(cond, env, host, fuel)?)?;
                if taken {
                    for nested_item in nested {
                        self.execute_item(nested_item, env, host, graph, sink, fuel)?;
                    }
                }
                Ok(())
            }
            "update-node" => self.update_node(items, env, host, graph, fuel),
            "add-node" => self.add_node(items, env, host, graph, fuel),
            "remove-node" => {
                charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
                let [_, node] = items.as_slice() else {
                    return Err(plain("(remove-node <expr>) takes exactly one operand"));
                };
                let id = self.resolve_node(node, env, host, fuel)?;
                self.ensure_record_capacity()?;
                graph.remove_node(id).map_err(from_graph)?;
                self.record(Write::NodeRemoved { id })?;
                Ok(())
            }
            "add-edge" => self.add_edge(items, env, host, graph, fuel),
            "remove-edge" => self.remove_edge(items, env, host, graph, fuel),
            "add-hyperedge" => self.add_hyperedge(items, env, host, graph, fuel),
            "remove-hyperedge" => {
                charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
                let [_, h] = items.as_slice() else {
                    return Err(plain("(remove-hyperedge <expr>) takes exactly one operand"));
                };
                let id = self.resolve_hyperedge(h, env, host, fuel)?;
                self.ensure_record_capacity()?;
                graph.remove_hyperedge(id).map_err(from_graph)?;
                self.record(Write::HyperedgeRemoved { id })?;
                Ok(())
            }
            "emit" => Self::emit(items, env, host, sink, fuel),
            "for-each" => self.for_each(items, env, host, graph, sink, fuel),
            "update-edge" => self.update_edge(items, env, host, graph, fuel),
            "update-hyperedge" => self.update_hyperedge(items, env, host, graph, fuel),
            other => Err(plain(format!(
                "unknown effect head ({other} …) — the §2.8 verb set is closed"
            ))),
        }
    }

    /// `(for-each <query> <elem-name>? <effect-item>+)` (§2.8 chapter C6).
    /// This is the EXECUTE path's copy — production has not called it since
    /// Task 12 (`run_tick` calls `collect_item`'s own `"for-each"` arm
    /// instead); it survives as the single-pass immediate-apply harness
    /// `EffectExecutor::execute_effects`'s own callers use (this crate's
    /// unit tests, the conformance corpus — see that method's own doc).
    ///
    /// The query materializes through `env.graph` — the caller's pre-state
    /// reference — exactly once, before any of this `for-each`'s own
    /// per-element effects apply, mirroring `evaluator::eval_fold`/
    /// `eval_exists_forall`/`eval_selection`'s identical query-then-iterate
    /// shape in expression position.
    ///
    /// **Corrected (#519 fix round):** this doc used to claim `env.graph`
    /// is NEVER the same object a verb write path mutates, as though the
    /// TYPE SYSTEM enforced that. It does not: `env: &EvalEnv<'_>` and
    /// `graph: &mut dyn GraphSubstrate` are independent parameters, and
    /// nothing in this method's signature stops a caller from constructing
    /// both from the SAME underlying graph via sequential, non-overlapping
    /// reborrows — exactly the technique `tick.rs::run_tick` uses across
    /// its own Pass 1/Pass 2 split (NLL re-acquires a fresh reborrow per
    /// subject; the verifier compiled a Pass-1 mutation that built cleanly,
    /// proving the whole-pass guarantee is not type-level either). What the
    /// type system DOES guarantee, scoped to exactly this one call: nothing
    /// this method calls performs a write through `env.graph` (it is `&`,
    /// never `&mut`) — every write goes through the separate `graph`
    /// parameter below. That callers keep pre-state reads and live writes
    /// from OVERLAPPING in time is their own discipline (`run_tick`'s
    /// two-pass split, guarded by this crate's own pre-state tests), not a
    /// fact this method's signature forces on every caller.
    ///
    /// Application order is total: the body runs once per element in
    /// iteration order (outer), and its own items apply in source order
    /// (inner, via the ordinary `execute_item` recursion) — nested
    /// `for-each` composes the same way. An empty query applies nothing and
    /// is not an error: an iteration is a command, and "do it to none" is
    /// completely determined.
    #[allow(clippy::too_many_arguments)]
    fn for_each(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        let (elem_name, effect_items, elements) = Self::for_each_prelude(items, env, host, fuel)?;
        for element in elements {
            let child = crate::evaluator::with_element(env, elem_name.clone(), element);
            for effect_item in effect_items {
                self.execute_item(effect_item, &child, host, graph, sink, fuel)?;
            }
        }
        Ok(())
    }

    /// Shared `for-each` prelude (§2.8 chapter C6): charge, destructure
    /// `(for-each <query> <elem-name>? <effect-item>+)`, strip an optional
    /// `:as` name, refuse an empty body, and materialize the query through
    /// `env.graph` exactly once — everything both [`Self::for_each`] (the
    /// execute path) and [`Self::collect_item`]'s `"for-each"` arm (the
    /// collect path) do IDENTICALLY before diverging on how they run the
    /// body over each element (mutate immediately vs. collect a
    /// [`PendingWrite`]).
    ///
    /// Extracted after the M4 mutation-verification gap this duplication
    /// caused (#519 fix round, fix 7): the two copies had already drifted
    /// — a mutation deleting the collect-path copy's iteration loop
    /// (materializing the query but never running the body) flipped ZERO
    /// tests, because every for-each test drove the execute path only.
    /// One prelude now means a shape bug here can only exist once, not
    /// twice with one copy silently stale.
    ///
    /// # Errors
    ///
    /// The missing-query and empty-body shape errors, or whatever
    /// [`crate::query::materialize`] raises.
    #[allow(clippy::type_complexity)]
    fn for_each_prelude<'e>(
        items: &'e [SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<(Option<String>, &'e [SExpr], Vec<crate::query::Element>), EvalError> {
        charge(fuel, cost::FOR_EACH_BASE)?;
        let [_, query, rest @ ..] = items else {
            return Err(plain(
                "(for-each <query> <elem-name>? <effect-item>+) — missing query",
            ));
        };
        let (elem_name, effect_items) = crate::evaluator::strip_as_name(rest);
        if effect_items.is_empty() {
            return Err(plain(
                "(for-each …) requires at least one effect item (§2.8)",
            ));
        }
        let elements = crate::query::materialize(query, env, host, fuel)?;
        Ok((elem_name, effect_items, elements))
    }

    /// `(update-node <expr> <qname> <update-op>)` — read-modify-write under
    /// the §3.3 store-boundary range check.
    fn update_node(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, node, SExpr::Atom(Atom::QName(field)), op_form] = items else {
            return Err(plain(
                "(update-node <expr> <qname> <update-op>) — unrecognized shape",
            ));
        };
        let id = self.resolve_node(node, env, host, fuel)?;
        // §2.10 discipline 1's runtime half (R9 chapter C2): `node`'s
        // static type is a reference (§3.1 gives it none), so the
        // field-owner-vs-referent disagreement `add-node` catches at LOAD
        // as `E-TYPE-014` can only be caught HERE, at evaluation, as
        // `E-EVAL-033` — before Task 11 this write succeeded silently.
        check_node_referent_type(&*graph, id, field, "update-node")?;
        let SExpr::List(op_items) = op_form else {
            return Err(plain(
                "update-op must be a form: (add|sub|set|scale <expr>)",
            ));
        };
        let [SExpr::Atom(Atom::Symbol(op)), operand] = op_items.as_slice() else {
            return Err(plain("update-op must be (add|sub|set|scale <expr>)"));
        };
        charge(fuel, cost::UPDATE_OP_BASE)?;
        // T3 #491, OQ-J: a `currency`-declared field forks BEFORE
        // `numeric_write_value`'s f64 lane — it never reaches that lane at
        // all, the same "check the declared type first" shape §2.13's enum
        // fork already models (`enum_write_value`).
        if matches!(
            self.types.fields.get(field).map(|decl| &decl.ty),
            Some(BslType::Currency)
        ) {
            return self.update_node_currency_op(id, field, op, operand, env, host, graph, fuel);
        }
        let operand_value =
            self.numeric_write_value(operand, env, host, fuel, field, "update-node")?;
        // `previous` is for the write log only. `set` does not otherwise read
        // the field, so it PROBES (a never-written field is `None`, not an
        // error — write_log discipline 3); the read-modify-write ops already
        // hold the value they need.
        let (new_value, previous) = match op.as_str() {
            "set" => (operand_value, self.probe_previous(&*graph, id, field)),
            "add" | "sub" | "scale" => {
                self.refuse_arithmetic_on_enum_field(field, "update-node")?;
                let current = graph.node_attribute(id, field).map_err(from_graph)?;
                let combined = match op.as_str() {
                    "add" => current + operand_value,
                    "sub" => current - operand_value,
                    _ => current * operand_value,
                };
                if !combined.is_finite() {
                    return Err(EvalError::coded(
                        EvalCode::NonFinite,
                        format!("({op} …) on {field} produced a non-finite value"),
                    ));
                }
                (combined, Some(current))
            }
            other => {
                return Err(plain(format!(
                    "unknown update-op ({other} …) — the set is add|sub|set|scale (§2.8)"
                )))
            }
        };
        let new_value = canonical_zero(new_value);
        self.store_range_check(field, new_value)?;
        self.ensure_record_capacity()?;
        graph
            .update_node(id, field, new_value)
            .map_err(from_graph)?;
        self.record(Write::NodeAttribute {
            id,
            field: field.clone(),
            previous,
            value: new_value,
        })?;
        Ok(())
    }

    /// The `currency`-declared field fork of [`Self::update_node`]'s
    /// read-modify-write (T3 #491, OQ-J — Currency's i128 typed storage).
    /// Only `set` is licensed: `add`/`sub`/`scale` over Currency would need
    /// to pick which of Currency's five legal operators (`bsl-language.rst`
    /// §3.2) applies, and nothing in this train's brief asks for that —
    /// narrower is correct here, mirroring
    /// [`Self::refuse_arithmetic_on_enum_field`]'s identical discipline for
    /// `Enum<T>`. Shared by [`Self::update_node`] (this immediate-apply
    /// call) and — via [`WriteOperand::Currency`] — the collect-then-apply
    /// path's `set` case in [`Self::apply_pending_write`].
    #[allow(clippy::too_many_arguments)]
    fn update_node_currency_op(
        &mut self,
        id: NodeId,
        field: &str,
        op: &str,
        operand: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        if op != "set" {
            return Err(plain(format!(
                "update-node {field}: only `set` is licensed for a currency-declared \
                 field — add/sub/scale would need to pick one of Currency's five legal \
                 operators (§3.2), which this typed-storage train does not license"
            )));
        }
        let value = evaluate(operand, env, host, fuel)?;
        let currency = currency_write_value(value, field, "update-node")?;
        store_range_check_currency(field, currency)?;
        let previous = self.probe_previous_currency(&*graph, id, field);
        self.ensure_record_capacity()?;
        graph
            .update_node_currency(id, field, currency)
            .map_err(from_graph)?;
        self.record(Write::NodeCurrencyAttribute {
            id,
            field: field.to_owned(),
            previous,
            value: currency,
        })?;
        Ok(())
    }

    /// `(update-edge <expr> <qname> <update-op>)` (§2.8 chapter C2, D36) —
    /// T3 (ADR198 R3, issue #560). Mirrors [`Self::update_node`] operand for
    /// operand, read-modify-write under the §3.3 store-boundary range check,
    /// on the IMMEDIATE execute path (this crate's test/corpus harness —
    /// production defers through [`Self::collect_update_edge`] +
    /// [`Self::apply_pending_write`]). The referent is an `EdgeRef` (T2's
    /// `EdgeKey`), never a type-and-endpoints triple (D36); the write routes
    /// through `GraphSubstrate::update_edge`, whose suffix fork lands a
    /// `<edge-type>/strength` write in the edge's existing 0x03-slot
    /// strength, never a fifth-section shadow row (D143).
    fn update_edge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, edge, SExpr::Atom(Atom::QName(field)), op_form] = items else {
            return Err(plain(
                "(update-edge <expr> <qname> <update-op>) — unrecognized shape",
            ));
        };
        let key = Self::resolve_edge(edge, env, host, fuel)?;
        // §2.10 discipline 1's runtime half, the edge form: the qname's
        // owner segment must name the referent's declared edge type, or the
        // write would land on the wrong field — E-EVAL-033, the same law
        // `field-of` over an EdgeRef already holds (T2).
        check_edge_referent_type(&key, field, "update-edge")?;
        let SExpr::List(op_items) = op_form else {
            return Err(plain(
                "update-op must be a form: (add|sub|set|scale <expr>)",
            ));
        };
        let [SExpr::Atom(Atom::Symbol(op)), operand] = op_items.as_slice() else {
            return Err(plain("update-op must be (add|sub|set|scale <expr>)"));
        };
        charge(fuel, cost::UPDATE_OP_BASE)?;
        let operand_value =
            self.numeric_write_value(operand, env, host, fuel, field, "update-edge")?;
        // `previous` is for the write log only — the same probe discipline
        // as the node side (write_log discipline 3), through edge_attribute.
        let (new_value, previous) = match op.as_str() {
            "set" => (
                operand_value,
                self.probe_previous_edge(&*graph, &key, field),
            ),
            "add" | "sub" | "scale" => {
                self.refuse_arithmetic_on_enum_field(field, "update-edge")?;
                let current = graph
                    .edge_attribute(&key.edge_type, key.source, key.target, field)
                    .map_err(from_graph)?;
                let combined = match op.as_str() {
                    "add" => current + operand_value,
                    "sub" => current - operand_value,
                    _ => current * operand_value,
                };
                if !combined.is_finite() {
                    return Err(EvalError::coded(
                        EvalCode::NonFinite,
                        format!("({op} …) on {field} produced a non-finite value"),
                    ));
                }
                (combined, Some(current))
            }
            other => {
                return Err(plain(format!(
                    "unknown update-op ({other} …) — the set is add|sub|set|scale (§2.8)"
                )))
            }
        };
        let new_value = canonical_zero(new_value);
        self.store_range_check(field, new_value)?;
        self.ensure_record_capacity()?;
        graph
            .update_edge(&key.edge_type, key.source, key.target, field, new_value)
            .map_err(from_graph)?;
        self.record(Write::EdgeAttribute {
            edge_type: key.edge_type.clone(),
            from: key.source,
            to: key.target,
            field: field.clone(),
            previous,
            value: new_value,
        })?;
        Ok(())
    }

    /// `(update-hyperedge <expr> <qname> <update-op>)` — the hyperedge
    /// own-field write (Community port train, Task 6, E2b), the execute
    /// path. Mirrors [`Self::update_edge`]'s discipline exactly: the
    /// referent's declared type is checked against the qname's owner
    /// segment (E-EVAL-033's hyperedge half), `set` never reads the prior
    /// value for the combine (only the log probe), `add`/`sub`/`scale`
    /// read-combine through `hyperedge_attribute`, non-finite combines
    /// refuse (E-EVAL-014), the write is range-checked at the store
    /// boundary (E-EVAL-020) and canonical-zeroed, and the substrate
    /// errors map to E-EVAL-031.
    fn update_hyperedge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, hyperedge, SExpr::Atom(Atom::QName(field)), op_form] = items else {
            return Err(plain(
                "(update-hyperedge <expr> <qname> <update-op>) — unrecognized shape",
            ));
        };
        let id = self.resolve_hyperedge(hyperedge, env, host, fuel)?;
        crate::evaluator::check_hyperedge_referent_type(id, field, "update-hyperedge", &*graph)?;
        let SExpr::List(op_items) = op_form else {
            return Err(plain(
                "update-op must be a form: (add|sub|set|scale <expr>)",
            ));
        };
        let [SExpr::Atom(Atom::Symbol(op)), operand] = op_items.as_slice() else {
            return Err(plain("update-op must be (add|sub|set|scale <expr>)"));
        };
        charge(fuel, cost::UPDATE_OP_BASE)?;
        let operand_value =
            self.numeric_write_value(operand, env, host, fuel, field, "update-hyperedge")?;
        let (new_value, previous) = match op.as_str() {
            "set" => (
                operand_value,
                self.probe_previous_hyperedge(&*graph, id, field),
            ),
            "add" | "sub" | "scale" => {
                self.refuse_arithmetic_on_enum_field(field, "update-hyperedge")?;
                let current = graph.hyperedge_attribute(id, field).map_err(from_graph)?;
                let combined = match op.as_str() {
                    "add" => current + operand_value,
                    "sub" => current - operand_value,
                    _ => current * operand_value,
                };
                if !combined.is_finite() {
                    return Err(EvalError::coded(
                        EvalCode::NonFinite,
                        format!("({op} …) on {field} produced a non-finite value"),
                    ));
                }
                (combined, Some(current))
            }
            other => {
                return Err(plain(format!(
                    "unknown update-op ({other} …) — the set is add|sub|set|scale (§2.8)"
                )))
            }
        };
        let new_value = canonical_zero(new_value);
        self.store_range_check(field, new_value)?;
        self.ensure_record_capacity()?;
        graph
            .update_hyperedge_attribute(id, field, new_value)
            .map_err(from_graph)?;
        self.record(Write::HyperedgeAttribute {
            id,
            field: field.clone(),
            previous,
            value: new_value,
        })?;
        Ok(())
    }

    /// The collect side's previous-value probe for the hyperedge lane —
    /// `write_log` discipline 3, through `hyperedge_attribute` exactly as
    /// [`Self::probe_previous_edge`] reads through `edge_attribute`.
    fn probe_previous_hyperedge(
        &self,
        graph: &dyn GraphSubstrate,
        id: HyperedgeId,
        field: &str,
    ) -> Option<f64> {
        self.observer.as_ref()?;
        graph.hyperedge_attribute(id, field).ok()
    }

    // ---- Task 12: the pre-state law — collect-then-apply ----

    /// COLLECT phase (§2.8 chapter C6 + §4.2 chapter C4): evaluate
    /// `effect_items` against `env`'s pre-state, returning the
    /// `update-node` writes they would perform WITHOUT applying any of
    /// them. This method takes no mutable graph at all — that is what
    /// makes "every firing observes the same pre-state" a property of the
    /// TYPE, not a convention a caller could violate by forgetting to
    /// re-read: nothing this method calls CAN mutate a graph.
    ///
    /// `emit` fires immediately even here (it never touched a graph, and
    /// its payload evaluates against the same frozen `env`, matching §2.8's
    /// own worked `for-each` example, whose `emit` reads the PRE-scale
    /// `solidarity/strength`). `guard` and `for-each` recurse the same way
    /// `Self::execute_item` does, over this collecting path instead.
    ///
    /// **Scope.** The six graph-shape verbs (`add-node`, `remove-node`,
    /// `add-edge`, `remove-edge`, `add-hyperedge`, `remove-hyperedge`)
    /// refuse loudly here, naming this gap: verified by grep over
    /// `rust/crates/babylon-tick/content/rules/*.bsl`, nothing landed uses
    /// them, and correctly deferring a MINTING verb needs a placeholder-id
    /// scheme neither this plan's `PendingWrite` sketch nor its two
    /// required tests specify — inventing one would be exactly the silent
    /// invention this crate's discipline forbids (Constitution, escalation
    /// clause). They stay fully served through [`Self::execute_effects`],
    /// unchanged, for callers that need them today.
    ///
    /// # Errors
    ///
    /// Any [`EvalError`] an operand or query raises, and a named refusal
    /// for a graph-shape verb this phase does not serve.
    pub fn collect_effects(
        &mut self,
        effect_items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
    ) -> Result<Vec<PendingWrite>, EvalError> {
        let mut pending = Vec::new();
        self.collect_items(effect_items, env, host, sink, fuel, &mut pending)?;
        Ok(pending)
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_items(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
        pending: &mut Vec<PendingWrite>,
    ) -> Result<(), EvalError> {
        for item in items {
            self.collect_item(item, env, host, sink, fuel, pending)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_item(
        &mut self,
        item: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
        pending: &mut Vec<PendingWrite>,
    ) -> Result<(), EvalError> {
        let SExpr::List(items) = item else {
            return Err(plain(format!(
                "an effect item must be a form, found {item:?}"
            )));
        };
        let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
            return Err(plain(format!(
                "an effect item must be a verb or guard form, found {:?}",
                items.first()
            )));
        };
        match head.as_str() {
            "guard" => {
                charge(fuel, cost::GUARD_BASE)?;
                let [_, cond, nested @ ..] = items.as_slice() else {
                    return Err(plain("(guard <cond> <effect-item>+) — missing condition"));
                };
                if nested.is_empty() {
                    return Err(plain("(guard …) requires at least one effect item"));
                }
                let taken = crate::evaluator::as_bool(evaluate(cond, env, host, fuel)?)?;
                if taken {
                    self.collect_items(nested, env, host, sink, fuel, pending)?;
                }
                Ok(())
            }
            "update-node" => {
                let write = self.collect_update_node(items, env, host, fuel)?;
                pending.push(write);
                Ok(())
            }
            "emit" => Self::emit(items, env, host, sink, fuel),
            "for-each" => {
                let (elem_name, effect_items, elements) =
                    Self::for_each_prelude(items.as_slice(), env, host, fuel)?;
                for element in elements {
                    let child = crate::evaluator::with_element(env, elem_name.clone(), element);
                    self.collect_items(effect_items, &child, host, sink, fuel, pending)?;
                }
                Ok(())
            }
            verb @ ("add-node" | "remove-node" | "add-edge" | "remove-edge" | "add-hyperedge"
            | "remove-hyperedge") => Err(plain(format!(
                "({verb} …) needs a mutable graph — Task 12's pre-state \
                 collection phase (§4.2 chapter C4) does not serve the \
                 graph-shape verbs, only update-node/update-edge/emit/guard/\
                 for-each. \
                 Every rule `rule_pipeline::load_rule_form` accepts is \
                 already refused, BY NAME, before it ever reaches this arm \
                 (`check_no_deferred_shape_verbs`, the LOAD-time gate — \
                 §3's own law: every check in this chapter runs at content \
                 load, before any tick executes). Reaching this defense-in- \
                 depth arm at all means a caller invoked collect_effects \
                 directly, bypassing that gate. The follow-on that will \
                 serve {verb} is the placeholder-id design this module's \
                 own collect_effects doc escalates — never \
                 EffectExecutor::execute_effects, which is retired from \
                 production (Task 12) and stays only as a test/corpus \
                 harness (see its own doc)"
            ))),
            "update-edge" => {
                let write = self.collect_update_edge(items, env, host, fuel)?;
                pending.push(write);
                Ok(())
            }
            "update-hyperedge" => {
                let write = self.collect_update_hyperedge(items, env, host, fuel)?;
                pending.push(write);
                Ok(())
            }
            other => Err(plain(format!(
                "unknown effect head ({other} …) — the §2.8 verb set is closed"
            ))),
        }
    }

    /// The collect half of `update-node`: parse, resolve the referent, run
    /// §2.10 discipline 1's type check, and reduce the operand — everything
    /// [`Self::update_node`] does EXCEPT the read-modify-write itself, which
    /// [`Self::apply_pending_write`] performs later, at apply time. The
    /// referent-type check reads through `env.graph` (there is no mutable
    /// graph here to reborrow from, unlike [`Self::update_node`]'s), which
    /// is the SAME pre-state reference the operand's own query, if any,
    /// resolves against.
    fn collect_update_node(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<PendingWrite, EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, node, SExpr::Atom(Atom::QName(field)), op_form] = items else {
            return Err(plain(
                "(update-node <expr> <qname> <update-op>) — unrecognized shape",
            ));
        };
        let id = self.resolve_node(node, env, host, fuel)?;
        let graph = require_graph(env, "update-node")?;
        check_node_referent_type(graph, id, field, "update-node")?;
        let SExpr::List(op_items) = op_form else {
            return Err(plain(
                "update-op must be a form: (add|sub|set|scale <expr>)",
            ));
        };
        let [SExpr::Atom(Atom::Symbol(op)), operand] = op_items.as_slice() else {
            return Err(plain("update-op must be (add|sub|set|scale <expr>)"));
        };
        charge(fuel, cost::UPDATE_OP_BASE)?;
        // T3 #491, OQ-J: the SAME collect-time fork `update_node`'s own
        // immediate-apply path takes, before `numeric_write_value`'s f64
        // lane. The domain check (`store_range_check_currency`) is
        // deliberately deferred to `apply_pending_write`, exactly as the
        // f64 lane's own `store_range_check` is — one check point, not two.
        if matches!(
            self.types.fields.get(field).map(|decl| &decl.ty),
            Some(BslType::Currency)
        ) {
            if op != "set" {
                return Err(plain(format!(
                    "update-node {field}: only `set` is licensed for a currency-declared \
                     field — add/sub/scale would need to pick one of Currency's five legal \
                     operators (§3.2), which this typed-storage train does not license"
                )));
            }
            let value = evaluate(operand, env, host, fuel)?;
            let currency = currency_write_value(value, field, "update-node")?;
            return Ok(PendingWrite {
                target: WriteTarget::Node(id),
                field: field.clone(),
                op: UpdateOp::Set,
                operand: WriteOperand::Currency(currency),
            });
        }
        let operand_value =
            self.numeric_write_value(operand, env, host, fuel, field, "update-node")?;
        let update_op = match op.as_str() {
            "set" => UpdateOp::Set,
            "add" => {
                self.refuse_arithmetic_on_enum_field(field, "update-node")?;
                UpdateOp::Add
            }
            "sub" => {
                self.refuse_arithmetic_on_enum_field(field, "update-node")?;
                UpdateOp::Sub
            }
            "scale" => {
                self.refuse_arithmetic_on_enum_field(field, "update-node")?;
                UpdateOp::Scale
            }
            other => {
                return Err(plain(format!(
                    "unknown update-op ({other} …) — the set is add|sub|set|scale (§2.8)"
                )))
            }
        };
        Ok(PendingWrite {
            target: WriteTarget::Node(id),
            field: field.clone(),
            op: update_op,
            operand: WriteOperand::Real(operand_value),
        })
    }

    /// The collect half of `update-edge` (T3, ADR198 R3, issue #560) — the
    /// production path's half of D36's verb. Mirrors
    /// [`Self::collect_update_node`] line for line: parse, resolve the
    /// `EdgeRef` referent, run §2.10 discipline 1's edge-form check, reduce
    /// the operand against the PRE-STATE — everything except the
    /// read-modify-write itself, which [`Self::apply_pending_write`]
    /// performs at apply time. The referent check needs no graph round-trip
    /// (an `EdgeKey` carries its type inline), unlike the node side's.
    fn collect_update_edge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<PendingWrite, EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, edge, SExpr::Atom(Atom::QName(field)), op_form] = items else {
            return Err(plain(
                "(update-edge <expr> <qname> <update-op>) — unrecognized shape",
            ));
        };
        let key = Self::resolve_edge(edge, env, host, fuel)?;
        check_edge_referent_type(&key, field, "update-edge")?;
        let SExpr::List(op_items) = op_form else {
            return Err(plain(
                "update-op must be a form: (add|sub|set|scale <expr>)",
            ));
        };
        let [SExpr::Atom(Atom::Symbol(op)), operand] = op_items.as_slice() else {
            return Err(plain("update-op must be (add|sub|set|scale <expr>)"));
        };
        charge(fuel, cost::UPDATE_OP_BASE)?;
        let operand_value =
            self.numeric_write_value(operand, env, host, fuel, field, "update-edge")?;
        let update_op = match op.as_str() {
            "set" => UpdateOp::Set,
            "add" => {
                self.refuse_arithmetic_on_enum_field(field, "update-edge")?;
                UpdateOp::Add
            }
            "sub" => {
                self.refuse_arithmetic_on_enum_field(field, "update-edge")?;
                UpdateOp::Sub
            }
            "scale" => {
                self.refuse_arithmetic_on_enum_field(field, "update-edge")?;
                UpdateOp::Scale
            }
            other => {
                return Err(plain(format!(
                    "unknown update-op ({other} …) — the set is add|sub|set|scale (§2.8)"
                )))
            }
        };
        Ok(PendingWrite {
            target: WriteTarget::Edge(key),
            field: field.clone(),
            op: update_op,
            operand: WriteOperand::Real(operand_value),
        })
    }

    /// `(update-hyperedge <expr> <qname> <update-op>)`, the collect half —
    /// mirrors [`Self::collect_update_edge`] exactly; the write applies in
    /// the APPLY phase against `WriteTarget::Hyperedge` (the pre-state law,
    /// §4.2 chapter C4, holds for this lane exactly as for the other two).
    fn collect_update_hyperedge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<PendingWrite, EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, hyperedge, SExpr::Atom(Atom::QName(field)), op_form] = items else {
            return Err(plain(
                "(update-hyperedge <expr> <qname> <update-op>) — unrecognized shape",
            ));
        };
        let id = self.resolve_hyperedge(hyperedge, env, host, fuel)?;
        // The referent check reads the pre-state (collect holds an
        // immutable reborrow) — the same law as every other collect_*.
        crate::evaluator::check_hyperedge_referent_type(
            id,
            field,
            "update-hyperedge",
            env.graph
                .expect("collect needs the pre-state graph (the driver always passes one)"),
        )?;
        let SExpr::List(op_items) = op_form else {
            return Err(plain(
                "update-op must be a form: (add|sub|set|scale <expr>)",
            ));
        };
        let [SExpr::Atom(Atom::Symbol(op)), operand] = op_items.as_slice() else {
            return Err(plain("update-op must be (add|sub|set|scale <expr>)"));
        };
        charge(fuel, cost::UPDATE_OP_BASE)?;
        let operand_value =
            self.numeric_write_value(operand, env, host, fuel, field, "update-hyperedge")?;
        let update_op = match op.as_str() {
            "set" => UpdateOp::Set,
            "add" | "sub" | "scale" => {
                self.refuse_arithmetic_on_enum_field(field, "update-hyperedge")?;
                match op.as_str() {
                    "add" => UpdateOp::Add,
                    "sub" => UpdateOp::Sub,
                    _ => UpdateOp::Scale,
                }
            }
            other => {
                return Err(plain(format!(
                    "unknown update-op ({other} …) — the set is add|sub|set|scale (§2.8)"
                )))
            }
        };
        Ok(PendingWrite {
            target: WriteTarget::Hyperedge(id),
            field: field.clone(),
            op: update_op,
            operand: WriteOperand::Real(operand_value),
        })
    }

    /// The Currency-lane half of [`Self::apply_pending_write`]'s Node arm
    /// (T3 #491, OQ-J), extracted so that function stays under the
    /// ≤100-line bound (Power-of-10 rule 3). Only `UpdateOp::Set` is
    /// legitimate here — `collect_update_node`'s own fork already refuses
    /// `add`/`sub`/`scale` for a currency-declared field, so `op` arriving
    /// as anything else is a collect/apply wiring bug, named rather than
    /// panicked.
    fn apply_pending_currency_write(
        &mut self,
        id: NodeId,
        field: &str,
        op: UpdateOp,
        currency: Currency,
        graph: &mut dyn GraphSubstrate,
    ) -> Result<(), EvalError> {
        if op != UpdateOp::Set {
            return Err(plain(format!(
                "update-node {field}: a Currency operand reached apply with a non-Set \
                 op — collect_update_node should have refused this already (wiring bug \
                 between collect and apply, not content)"
            )));
        }
        let previous = self.probe_previous_currency(&*graph, id, field);
        store_range_check_currency(field, currency)?;
        self.ensure_record_capacity()?;
        graph
            .update_node_currency(id, field, currency)
            .map_err(from_graph)?;
        self.record(Write::NodeCurrencyAttribute {
            id,
            field: field.to_owned(),
            previous,
            value: currency,
        })?;
        Ok(())
    }

    /// APPLY phase (Task 12): perform ONE collected write against the LIVE
    /// graph. `add`/`sub`/`scale` read the target's CURRENT value HERE, at
    /// apply time (D-row Q2) — this is what lets several subjects each
    /// contribute to one shared carrier without losing any contribution:
    /// each apply sees every PRIOR apply's result, in whatever order the
    /// caller applies the collected `Vec<PendingWrite>` (subject order
    /// outer, source order inner — this method performs exactly one write
    /// and does not itself impose an order over a batch). Node and edge
    /// targets share this one law (T3, ADR198 R3): the `WriteTarget::Edge`
    /// arm is the node arm's exact mirror over `edge_attribute`/`update_edge`.
    ///
    /// # Errors
    ///
    /// `E-EVAL-020` (store-boundary range violation), `E-EVAL-014`
    /// (non-finite combine), or the substrate's own existence error mapped
    /// to `E-EVAL-031`.
    pub fn apply_pending_write(
        &mut self,
        write: &PendingWrite,
        graph: &mut dyn GraphSubstrate,
    ) -> Result<(), EvalError> {
        match &write.target {
            WriteTarget::Node(id) => {
                // T3 #491, OQ-J: the Currency lane forks first, extracted
                // to its own method to keep this function under the
                // ≤100-line bound (Power-of-10 rule 3).
                if let WriteOperand::Currency(currency) = write.operand {
                    return self.apply_pending_currency_write(
                        *id,
                        &write.field,
                        write.op,
                        currency,
                        graph,
                    );
                }
                let WriteOperand::Real(operand) = write.operand else {
                    unreachable!("the Currency arm above returns before reaching here");
                };
                let (new_value, previous) = match write.op {
                    UpdateOp::Set => (operand, self.probe_previous(&*graph, *id, &write.field)),
                    UpdateOp::Add | UpdateOp::Sub | UpdateOp::Scale => {
                        self.refuse_arithmetic_on_enum_field(&write.field, "update-node")?;
                        let current = graph
                            .node_attribute(*id, &write.field)
                            .map_err(from_graph)?;
                        let combined = combine_and_check_finite(
                            write.op,
                            current,
                            operand,
                            &write.field,
                            "update-node",
                        )?;
                        (combined, Some(current))
                    }
                };
                let new_value = canonical_zero(new_value);
                self.store_range_check(&write.field, new_value)?;
                self.ensure_record_capacity()?;
                graph
                    .update_node(*id, &write.field, new_value)
                    .map_err(from_graph)?;
                self.record(Write::NodeAttribute {
                    id: *id,
                    field: write.field.clone(),
                    previous,
                    value: new_value,
                })?;
                Ok(())
            }
            WriteTarget::Edge(key) => {
                // There is no edge-scoped Currency lane (T3 #491, OQ-J) —
                // `collect_update_edge` never produces `WriteOperand::Currency`,
                // so reaching one here would be the same collect/apply
                // wiring-bug shape the node arm names above.
                let WriteOperand::Real(operand) = write.operand else {
                    return Err(plain(format!(
                        "update-edge {}: a Currency operand reached apply — there is no \
                         edge-scoped Currency lane; collect_update_edge should never have \
                         produced this (wiring bug, not content)",
                        write.field
                    )));
                };
                let (new_value, previous) = match write.op {
                    UpdateOp::Set => (
                        operand,
                        self.probe_previous_edge(&*graph, key, &write.field),
                    ),
                    UpdateOp::Add | UpdateOp::Sub | UpdateOp::Scale => {
                        self.refuse_arithmetic_on_enum_field(&write.field, "update-edge")?;
                        let current = graph
                            .edge_attribute(&key.edge_type, key.source, key.target, &write.field)
                            .map_err(from_graph)?;
                        let combined = combine_and_check_finite(
                            write.op,
                            current,
                            operand,
                            &write.field,
                            "update-edge",
                        )?;
                        (combined, Some(current))
                    }
                };
                let new_value = canonical_zero(new_value);
                self.store_range_check(&write.field, new_value)?;
                self.ensure_record_capacity()?;
                graph
                    .update_edge(
                        &key.edge_type,
                        key.source,
                        key.target,
                        &write.field,
                        new_value,
                    )
                    .map_err(from_graph)?;
                self.record(Write::EdgeAttribute {
                    edge_type: key.edge_type.clone(),
                    from: key.source,
                    to: key.target,
                    field: write.field.clone(),
                    previous,
                    value: new_value,
                })?;
                Ok(())
            }
            WriteTarget::Hyperedge(id) => self.apply_pending_hyperedge_write(*id, write, graph),
        }
    }

    /// The hyperedge half of [`Self::apply_pending_write`] (Community port
    /// train, Task 6, E2b), extracted so that function stays under the
    /// ≤100-line bound (Power-of-10 rule 3) — the same reason
    /// [`Self::apply_pending_currency_write`] gives for its own extraction.
    /// Mirrors the Edge arm exactly: no hyperedge-scoped Currency lane
    /// exists, so a Currency operand here is the same collect/apply
    /// wiring-bug shape the edge arm names.
    fn apply_pending_hyperedge_write(
        &mut self,
        id: HyperedgeId,
        write: &PendingWrite,
        graph: &mut dyn GraphSubstrate,
    ) -> Result<(), EvalError> {
        let WriteOperand::Real(operand) = write.operand else {
            return Err(plain(format!(
                "update-hyperedge {}: a Currency operand reached apply — there is no \
                 hyperedge-scoped Currency lane; collect_update_hyperedge should never \
                 have produced this (wiring bug, not content)",
                write.field
            )));
        };
        let (new_value, previous) = match write.op {
            UpdateOp::Set => (
                operand,
                self.probe_previous_hyperedge(&*graph, id, &write.field),
            ),
            UpdateOp::Add | UpdateOp::Sub | UpdateOp::Scale => {
                self.refuse_arithmetic_on_enum_field(&write.field, "update-hyperedge")?;
                let current = graph
                    .hyperedge_attribute(id, &write.field)
                    .map_err(from_graph)?;
                let combined = combine_and_check_finite(
                    write.op,
                    current,
                    operand,
                    &write.field,
                    "update-hyperedge",
                )?;
                (combined, Some(current))
            }
        };
        let new_value = canonical_zero(new_value);
        self.store_range_check(&write.field, new_value)?;
        self.ensure_record_capacity()?;
        graph
            .update_hyperedge_attribute(id, &write.field, new_value)
            .map_err(from_graph)?;
        self.record(Write::HyperedgeAttribute {
            id,
            field: write.field.clone(),
            previous,
            value: new_value,
        })?;
        Ok(())
    }

    /// `(add-node <enum-ref> <expr> <field-init>*)`.
    fn add_node(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, type_ref, id_expr, field_inits @ ..] = items else {
            return Err(plain(
                "(add-node <enum-ref> <expr> <field-init>*) — too few operands",
            ));
        };
        let node_type = self.enum_member_checked(type_ref, "add-node")?;
        let name = self.fresh_declared_name(id_expr, env)?;
        self.ensure_record_capacity()?;
        let id = graph.add_node(node_type).map_err(from_graph)?;
        self.declared_nodes.insert(name, id);
        self.record(Write::NodeAdded {
            id,
            node_type: node_type.to_owned(),
        })?;
        for init in field_inits {
            let SExpr::List(pair) = init else {
                return Err(plain(format!(
                    "a field-init must be (<qname> <expr>), found {init:?}"
                )));
            };
            let [SExpr::Atom(Atom::QName(field)), value_expr] = pair.as_slice() else {
                return Err(plain(format!(
                    "a field-init must be (<qname> <expr>), found {pair:?}"
                )));
            };
            let value = self.numeric_write_value(value_expr, env, host, fuel, field, "add-node")?;
            self.store_range_check(field, value)?;
            // A field-init on a freshly minted node normally has no prior
            // value; probing rather than assuming keeps a repeated init
            // honest.
            let previous = self.probe_previous(&*graph, id, field);
            self.ensure_record_capacity()?;
            graph.update_node(id, field, value).map_err(from_graph)?;
            self.record(Write::NodeAttribute {
                id,
                field: field.clone(),
                previous,
                value,
            })?;
        }
        Ok(())
    }

    /// `(add-edge <enum-ref> <expr> <expr> :strength <expr> <field-init>*)`
    /// — the `<field-init>*` tail is R9 chapter C2's addition (D37). Its
    /// static checks (`E-PARSE-041` on a `strength` init, `E-TYPE-014` on a
    /// foreign owner) are [`crate::grammar`]'s, at load; the tail's
    /// *execution* landed with T3 (ADR198 R1/R3, issue #560) — each init
    /// crosses the same funnel an `update-edge` write does
    /// (`numeric_write_value` + §3.3's range check + the write log), against
    /// the freshly minted edge. A `strength` init never reaches here (the
    /// static check owns it; the `:strength` operand is that field's only
    /// writer at mint time) — if one somehow did, `update_edge`'s suffix
    /// fork would silently double-write the 0x03 slot, so this path refuses
    /// it again, defensively.
    fn add_edge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, type_ref, from, to, SExpr::Atom(Atom::Keyword(kw)), strength_expr, field_inits @ ..] =
            items
        else {
            return Err(plain(
                "(add-edge <enum-ref> <expr> <expr> :strength <expr> <field-init>*) \
                 — unrecognized shape",
            ));
        };
        if kw != "strength" {
            return Err(plain(format!("add-edge requires :strength, found :{kw}")));
        }
        let edge_type = self.enum_member_checked(type_ref, "add-edge")?;
        let from_id = self.resolve_node(from, env, host, fuel)?;
        let to_id = self.resolve_node(to, env, host, fuel)?;
        let strength = match evaluate(strength_expr, env, host, fuel)? {
            Value::Real(r) => r,
            other => {
                return Err(plain(format!(
                    ":strength must evaluate in the binary64 lane, got {other:?}"
                )))
            }
        };
        // The evaluator guards its own arithmetic, so a non-finite can only
        // arrive from across the intrinsic seam — refuse it HERE, at the
        // substrate boundary (numeric_write_value's defense-in-depth
        // rationale), never later inside the hash.
        if !strength.is_finite() {
            return Err(EvalError::coded(
                EvalCode::NonFinite,
                format!("add-edge :strength must be finite, got {strength}"),
            ));
        }
        self.ensure_record_capacity()?;
        graph
            .add_edge(edge_type, from_id, to_id, strength)
            .map_err(from_graph)?;
        self.record(Write::EdgeAdded {
            edge_type: edge_type.to_owned(),
            from: from_id,
            to: to_id,
            strength,
        })?;
        for init in field_inits {
            let SExpr::List(pair) = init else {
                return Err(plain(format!(
                    "a field-init must be (<qname> <expr>), found {init:?}"
                )));
            };
            let [SExpr::Atom(Atom::QName(field)), value_expr] = pair.as_slice() else {
                return Err(plain(format!(
                    "a field-init must be (<qname> <expr>), found {pair:?}"
                )));
            };
            if field.ends_with("/strength") {
                // E-PARSE-041's runtime echo (direct-harness defense in
                // depth; load-time grammar owns the check): the `:strength`
                // operand is that field's only writer at mint time.
                return Err(plain(format!(
                    "an add-edge <field-init> naming {field} is E-PARSE-041 at load — the \
                     :strength operand is that field's only writer at mint time"
                )));
            }
            let value = self.numeric_write_value(value_expr, env, host, fuel, field, "add-edge")?;
            self.store_range_check(field, value)?;
            // A field-init on a freshly minted edge normally has no prior
            // value; probing rather than assuming keeps a repeated init
            // honest (write_log discipline 3, the edge half).
            let previous = self.probe_previous_edge(
                &*graph,
                &EdgeKey {
                    source: from_id,
                    target: to_id,
                    edge_type: edge_type.to_owned(),
                },
                field,
            );
            self.ensure_record_capacity()?;
            graph
                .update_edge(edge_type, from_id, to_id, field, value)
                .map_err(from_graph)?;
            self.record(Write::EdgeAttribute {
                edge_type: edge_type.to_owned(),
                from: from_id,
                to: to_id,
                field: field.clone(),
                previous,
                value,
            })?;
        }
        Ok(())
    }

    /// `(remove-edge <enum-ref> <expr> <expr>)`.
    fn remove_edge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, type_ref, from, to] = items else {
            return Err(plain(
                "(remove-edge <enum-ref> <expr> <expr>) — unrecognized shape",
            ));
        };
        let edge_type = Self::enum_member(type_ref)?;
        let from_id = self.resolve_node(from, env, host, fuel)?;
        let to_id = self.resolve_node(to, env, host, fuel)?;
        self.ensure_record_capacity()?;
        graph
            .remove_edge(edge_type, from_id, to_id)
            .map_err(from_graph)?;
        self.record(Write::EdgeRemoved {
            edge_type: edge_type.to_owned(),
            from: from_id,
            to: to_id,
        })?;
        Ok(())
    }

    /// `(add-hyperedge <enum-ref> <expr> <members> <field-init>*)` — the
    /// member list crosses WHOLE; no path here expands it (VIII.9).
    fn add_hyperedge(
        &mut self,
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        graph: &mut dyn GraphSubstrate,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, type_ref, id_expr, members_form, field_inits @ ..] = items else {
            return Err(plain(
                "(add-hyperedge <enum-ref> <expr> <members> <field-init>*) — too few operands",
            ));
        };
        if !field_inits.is_empty() {
            // §2.8's ruling already names the adjacent gap (per-membership
            // payload, hyperedge field mutation); mint-time field-inits are
            // a DISTINCT unserved lane — the own-field storage exists since
            // Task 5 (E2a) and `update-hyperedge`/`(hyperedge-attr …)` are
            // its two landed writers (Task 6), but the init sugar on THIS
            // verb is not routed to it in this train. Loud, not dropped.
            return Err(plain(
                "hyperedge <field-init> is not served at mint time — write the field \
                 after minting through update-hyperedge (or seed it in the scenario \
                 with (hyperedge-attr …)); never silently dropped",
            ));
        }
        let hyperedge_type = self.enum_member_checked(type_ref, "add-hyperedge")?;
        let name = self.fresh_declared_name(id_expr, env)?;
        let SExpr::List(member_items) = members_form else {
            return Err(plain("expected a (members <expr>+) form"));
        };
        let [SExpr::Atom(Atom::Symbol(head)), member_exprs @ ..] = member_items.as_slice() else {
            return Err(plain("expected a (members <expr>+) form"));
        };
        if head != "members" || member_exprs.is_empty() {
            // The grammar's <expr>+ makes a zero-member hyperedge
            // unexpressible; meeting one here is a shape error.
            return Err(plain("(members <expr>+) requires at least one member"));
        }
        let mut members = Vec::with_capacity(member_exprs.len());
        for member in member_exprs {
            members.push(self.resolve_node(member, env, host, fuel)?);
        }
        // Membership is a SET and declared member order is never observable
        // (§2.6 draft ruling D25; `members_of` returns ascending NodeId).
        // Canonicalize HERE so the write log cannot become the one surface
        // that leaks source order back. Duplicates stay the substrate's
        // error to raise — sorting does not mask them.
        members.sort_unstable();
        self.ensure_record_capacity()?;
        let id = graph
            .add_hyperedge(hyperedge_type, &members)
            .map_err(from_graph)?;
        self.declared_hyperedges.insert(name, id);
        // The member list is recorded WHOLE — the log expands it into pairs
        // no more than the executor does (VIII.9) — and in the canonical
        // ascending order established above, never as declared (D25).
        self.record(Write::HyperedgeAdded {
            id,
            hyperedge_type: hyperedge_type.to_owned(),
            members,
        })?;
        Ok(())
    }

    /// `(emit <enum-ref> <payload-item>*)` — payload names are labels;
    /// there is no string interpolation in a payload (§2.8).
    fn emit(
        items: &[SExpr],
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        sink: &mut dyn EventSink,
        fuel: &mut u64,
    ) -> Result<(), EvalError> {
        charge(fuel, cost::STRUCTURAL_VERB_BASE)?;
        let [_, type_ref, payload_items @ ..] = items else {
            return Err(plain(
                "(emit <enum-ref> <payload-item>*) — missing event type",
            ));
        };
        let event_type = Self::enum_member(type_ref)?;
        let mut payload = Vec::with_capacity(payload_items.len());
        for item in payload_items {
            let SExpr::List(pair) = item else {
                return Err(plain(format!(
                    "a payload item must be (<symbol> <expr>), found {item:?}"
                )));
            };
            let [SExpr::Atom(Atom::Symbol(name)), value_expr] = pair.as_slice() else {
                return Err(plain(format!(
                    "a payload item must be (<symbol> <expr>), found {pair:?}"
                )));
            };
            payload.push((name.clone(), evaluate(value_expr, env, host, fuel)?));
        }
        sink.emit(event_type, payload);
        Ok(())
    }

    /// An enum-ref operand's member name — the substrate's type string.
    fn enum_member(expr: &SExpr) -> Result<&str, EvalError> {
        match expr {
            SExpr::Atom(Atom::EnumRef { member, .. }) => Ok(member),
            other => Err(plain(format!(
                "expected an enum-ref where the grammar requires one, found {other:?}"
            ))),
        }
    }

    /// [`Self::enum_member`] plus the runtime half of Task 8's (Organization
    /// foundation plan) closed-vocabulary enforcement (§3.6): when a
    /// registry is threaded, the member must be REGISTERED, not merely
    /// well-shaped. Used only by the three MINTING verbs
    /// (`add-node`/`add-edge`/`add-hyperedge`) — Scout 3's "three
    /// producers" are the ones that mint a graph element the vocabulary is
    /// closed over; the non-minting verbs (`remove-edge`, `emit`) are
    /// unchanged by this task and keep calling [`Self::enum_member`]
    /// directly. `verb` is the calling verb's own name (F6, #534 fix round
    /// item 6) — §4.6's house style names the offending form; this is the
    /// one producer of the three where that form is a runtime call-site
    /// fact, not something the checked value itself carries.
    fn enum_member_checked<'e>(&self, expr: &'e SExpr, verb: &str) -> Result<&'e str, EvalError> {
        let SExpr::Atom(Atom::EnumRef { enum_type, member }) = expr else {
            // Reuses `enum_member`'s exact refusal for a non-enum-ref
            // operand — the same message either way.
            return Self::enum_member(expr);
        };
        if let Some(vocabulary) = self.vocabulary_registry {
            vocabulary
                .check_enum_ref(enum_type, member)
                .map_err(|e| plain(format!("({verb} …): {e}")))?;
        }
        Ok(member)
    }

    /// The id operand of `add-node`/`add-hyperedge`: a symbol introducing a
    /// fresh effect-list-scoped name (the §2.8 draft ruling this task
    /// records). Shadowing a binding, a reserved symbol, or an earlier
    /// declared id is loud.
    fn fresh_declared_name(&self, id_expr: &SExpr, env: &EvalEnv<'_>) -> Result<String, EvalError> {
        let SExpr::Atom(Atom::Symbol(name)) = id_expr else {
            return Err(plain(format!(
                "an add-node/add-hyperedge id operand must be a symbol naming \
                 the minted object for this effect list (§2.8 draft ruling), \
                 found {id_expr:?}"
            )));
        };
        let taken = crate::bindings::RESERVED_NAMES.contains(&name.as_str())
            || env.bindings.contains_key(name)
            || self.declared_nodes.contains_key(name)
            || self.declared_hyperedges.contains_key(name);
        if taken {
            return Err(plain(format!(
                "declared id {name} shadows an existing name — E-PARSE-022's \
                 no-shadowing discipline applies to effect-scoped ids too"
            )));
        }
        Ok(name.clone())
    }

    /// Resolve a node-ref operand: an effect-scoped declared id, or any
    /// expression evaluating to a `NodeRef` (`self`, a bound ref).
    fn resolve_node(
        &self,
        expr: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<NodeId, EvalError> {
        if let SExpr::Atom(Atom::Symbol(name)) = expr {
            if let Some(id) = self.declared_nodes.get(name) {
                charge(fuel, cost::VARIABLE_REF)?;
                return Ok(*id);
            }
        }
        match evaluate(expr, env, host, fuel)? {
            Value::NodeRef(id) => Ok(id),
            other => Err(plain(format!(
                "expected a NodeRef operand, got {other:?} (§3.1)"
            ))),
        }
    }

    /// Resolve a hyperedge-ref operand, symmetrically.
    fn resolve_hyperedge(
        &self,
        expr: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<HyperedgeId, EvalError> {
        if let SExpr::Atom(Atom::Symbol(name)) = expr {
            if let Some(id) = self.declared_hyperedges.get(name) {
                charge(fuel, cost::VARIABLE_REF)?;
                return Ok(*id);
            }
        }
        match evaluate(expr, env, host, fuel)? {
            Value::HyperedgeRef(id) => Ok(id),
            other => Err(plain(format!(
                "expected a HyperedgeRef operand, got {other:?} (§3.1)"
            ))),
        }
    }

    /// Resolve an edge-ref operand (T3, ADR198 R3, issue #560). Deliberately
    /// NO effect-list-scoped-name table half: `add-edge` mints no nameable id
    /// (D36 — the verb's referent is an `EdgeRef`, produced by §2.10's
    /// `edge-between`, by `it` inside a `for-each` over `edges`, or by any
    /// other `EdgeRef`-valued expression), so there is nothing to look up —
    /// an associated function, not a method (no `declared_*` table to read,
    /// unlike `resolve_node`/`resolve_hyperedge`).
    fn resolve_edge(
        expr: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
    ) -> Result<EdgeKey, EvalError> {
        match evaluate(expr, env, host, fuel)? {
            Value::EdgeRef(key) => Ok(key),
            other => Err(plain(format!(
                "(update-edge …)'s first operand must evaluate to an EdgeRef, got {other:?} \
                 (§2.10 — endpoint-holding rules reach the edge through edge-between, D36)"
            ))),
        }
    }

    /// Evaluate a value that will be WRITTEN to `field`, in the binary64
    /// lane the trait's attribute storage carries. **`update_node`'s OWN
    /// call site forks to the i128 lane before calling here (T3 #491,
    /// OQ-J)** — a `currency`-declared field reaching THIS function via
    /// `update-node` is therefore impossible. Three OTHER call sites carry
    /// no such fork and still reach the `Value::Currency` arm below
    /// unconditionally: `update-edge`/`collect_update_edge` (no edge-scoped
    /// Currency lane exists) and the `add-node`/`add-edge` field-init tails
    /// (field-init never routes through the typed lane, even for a
    /// node-scoped field). The match below therefore distinguishes the two
    /// REASONS a `Value::Currency` can arrive: `field` declared something
    /// else entirely (a genuine kind mismatch), or `field` genuinely
    /// declared `currency` but reached via one of those three paths (a
    /// real, named scope gap — not a kind mismatch, and not "no Currency
    /// storage exists" the way it was before T3 landed the node-scoped
    /// lane). Either way this refuses rather than casting lossily — i128
    /// exactness does not survive an f64 attribute regardless of the
    /// reason.
    ///
    /// **§2.13 addendum (D101).** When `field` is declared `BslType::Enum`
    /// (`self.types`), the write funnels through [`Self::enum_write_value`]
    /// instead: the ordinal is never a surface value, so a bare `Real`/`Int`
    /// here is exactly as illegal as it is at load
    /// (`scenario.rs::attribute_value_enum`, `E-LOAD-056`'s runtime
    /// re-check, `E-EVAL-042`). For every OTHER field this arm is a no-op —
    /// the existing catch-all below still refuses `Value::Enum` reaching a
    /// non-enum field with its ORIGINAL message, unchanged.
    ///
    /// `verb` names the calling form (`update-node`, `update-edge`,
    /// `add-node`/`add-edge` field-init) for diagnostics only — the write
    /// law is identical on both target kinds (T3, ADR198 R3).
    fn numeric_write_value(
        &self,
        expr: &SExpr,
        env: &EvalEnv<'_>,
        host: &dyn IntrinsicHost,
        fuel: &mut u64,
        field: &str,
        verb: &str,
    ) -> Result<f64, EvalError> {
        let value = evaluate(expr, env, host, fuel)?;
        if let Some(BslType::Enum(ty)) = self.types.fields.get(field).map(|decl| &decl.ty) {
            return self.enum_write_value(value, field, *ty, verb);
        }
        match value {
            // The evaluator guards its own arithmetic (E-EVAL-014), but a
            // value can reach the store boundary without passing through it
            // — an intrinsic host's return is the trait's named
            // defence-in-depth case. Guard HERE, the one funnel every write
            // value crosses, because §3.3's range check only rejects
            // non-finites on unit-interval fields: an Int or unbounded Real
            // field would otherwise take a NaN straight into the tick hash.
            Value::Real(r) if !r.is_finite() => Err(EvalError::coded(
                EvalCode::NonFinite,
                format!("refusing to store a non-finite value to {field}"),
            )),
            Value::Real(r) => Ok(r),
            // i64 -> f64 is deterministic; exactness beyond 2^53 belongs to
            // the typed-attribute-storage gap named in the module doc.
            #[allow(clippy::cast_precision_loss)]
            Value::Int(n) => Ok(n as f64),
            // L-3 (#491 T3 review): two DIFFERENT reasons, two DIFFERENT
            // messages — a field genuinely declared `currency` reaching
            // here (via `update-edge` or an `add-node`/`add-edge`
            // field-init, none of which fork to the typed lane) is not a
            // kind mismatch, and telling its author "needs a
            // currency-declared field" when it already IS one would be
            // false.
            Value::Currency(_)
                if matches!(
                    self.types.fields.get(field).map(|decl| &decl.ty),
                    Some(BslType::Currency)
                ) =>
            {
                Err(plain(format!(
                    "{verb} {field}: currency-declared fields have typed i128 storage for \
                     `update-node`'s runtime `set` write ONLY (T3 #491, OQ-J) — {verb} does not \
                     route through that lane, so this refuses rather than casting lossily into \
                     the f64 attribute store"
                )))
            }
            Value::Currency(_) => Err(plain(format!(
                "writing a Currency value to {field} needs a currency-declared field \
                 — this field is declared something else, and f64 cannot hold i128 \
                 micro-units without lying about what it stores, so this refuses rather \
                 than casting lossily"
            ))),
            other => Err(plain(format!(
                "cannot store {other:?} as a numeric attribute for {field} ({verb})"
            ))),
        }
    }

    /// The `:enum-type`-declared half of [`Self::numeric_write_value`]
    /// (§2.13, D101): accepts ONLY a `Value::Enum` of the field's exact
    /// declared type, resolves its ordinal through `self.enums`, and
    /// returns `ordinal as f64` — the SAME binary64 lane every other
    /// declared type's write already uses. Everything else — a bare
    /// `Value::Real`/`Value::Int` (the eval-time form of the load-time
    /// bare-number mistake), a `Value::Enum` of a DIFFERENT declared type,
    /// or any other value — is `E-EVAL-042`.
    fn enum_write_value(
        &self,
        value: Value,
        field: &str,
        ty: EnumTypeId,
        verb: &str,
    ) -> Result<f64, EvalError> {
        let Value::Enum { enum_type, member } = value else {
            return Err(EvalError::coded(
                EvalCode::EnumWriteShapeViolation,
                format!(
                    "{verb} {field}: an enum-typed field is written \
                     ONLY as <EnumType>/<MEMBER> — the ordinal is never a \
                     surface value; found {value:?} (§2.13)"
                ),
            ));
        };
        let declared_type = self.enums.name(ty);
        if enum_type != declared_type {
            return Err(EvalError::coded(
                EvalCode::EnumWriteShapeViolation,
                format!(
                    "{verb} {field}: declared enum type is \
                     {declared_type}, found {enum_type}/{member} (§2.13)"
                ),
            ));
        }
        let Some(ordinal) = self.enums.ordinal(ty, &member) else {
            return Err(EvalError::coded(
                EvalCode::EnumWriteShapeViolation,
                format!(
                    "{verb} {field}: {declared_type} has no member \
                     {member} — never a default (§2.13)"
                ),
            ));
        };
        Ok(f64::from(ordinal))
    }

    /// `E-EVAL-042` (§2.13, D101 — "no aggregation kind": `Enum<T>`
    /// supports no arithmetic) — the `add`/`sub`/`scale` half of the write
    /// law [`Self::enum_write_value`] does not itself cover. Reading the
    /// stored ordinal, combining it with an operand's ordinal, and writing
    /// the result back would silently reinterpret the combination as
    /// whatever DIFFERENT member happens to share that ordinal (`add`), or
    /// write a value with NO member at all — `store_range_check` (§3.3)
    /// bounds only the three unit-interval types, so an enum field's
    /// ordinal is otherwise unchecked at the store boundary (`sub`,
    /// `scale`). `set` is the only coherent op and is unaffected: it never
    /// reads the current value.
    ///
    /// Called from all FIVE sites that would otherwise perform this
    /// combine: [`Self::update_node`]'s and [`Self::update_edge`]'s
    /// immediate execute paths, [`Self::collect_update_node`]'s and
    /// [`Self::collect_update_edge`]'s collect paths (`run_tick`'s own —
    /// refusing here means the write never even reaches
    /// [`Self::apply_pending_write`]), and apply itself, which guards
    /// independently as defense in depth (the same two-site discipline
    /// `numeric_write_value`'s own doc names for the load-time/eval-time
    /// enum-shape check). T3 (ADR198 R3, issue #560) reuses this exact
    /// combine shape for the storage-bearing `update-edge`, as this doc
    /// always anticipated. `verb` names the calling form in the diagnostic.
    fn refuse_arithmetic_on_enum_field(&self, field: &str, verb: &str) -> Result<(), EvalError> {
        if let Some(BslType::Enum(_)) = self.types.fields.get(field).map(|decl| &decl.ty) {
            return Err(EvalError::coded(
                EvalCode::EnumWriteShapeViolation,
                format!(
                    "{verb} {field}: add/sub/scale is not a coherent \
                     operation on an enum-typed field — Enum<T> supports no \
                     arithmetic (§2.13); only `set` may write it"
                ),
            ));
        }
        Ok(())
    }

    /// §3.3's one range check, at the store boundary: a value outside the
    /// target field's declared domain is `E-EVAL-020` — never a clamp. A
    /// field absent from the registry is loud (closed vocabulary, §3.6).
    fn store_range_check(&self, field: &str, value: f64) -> Result<(), EvalError> {
        let Some(decl) = self.types.fields.get(field) else {
            return Err(plain(format!(
                "unknown field {field} — the vocabulary is closed (§3.6); the \
                 loader should have rejected this content"
            )));
        };
        let unit_interval = matches!(
            decl.ty,
            BslType::Probability | BslType::Intensity | BslType::Coefficient
        );
        if unit_interval && !(0.0..=1.0).contains(&value) {
            return Err(EvalError::coded(
                EvalCode::StoreRangeViolation,
                format!(
                    "storing {value} to {field} ({:?}) leaves its declared \
                     [0,1] domain — a loud failure, never a clamp (§3.3)",
                    decl.ty
                ),
            ));
        }
        Ok(())
    }
}

/// The Currency-lane half of write-value evaluation (T3 #491, OQ-J) — the
/// free-function counterpart of [`EffectExecutor::numeric_write_value`] for
/// a `currency`-declared field's `set` operand (an associated function, not
/// a method: it consults no `EffectExecutor` state). Accepts ONLY
/// `Value::Currency`; anything else is refused, naming what was found — the
/// same "the entry point never decides the type system" discipline the
/// sibling `*_write_value` functions hold.
fn currency_write_value(value: Value, field: &str, verb: &str) -> Result<Currency, EvalError> {
    match value {
        Value::Currency(c) => Ok(c),
        other => Err(plain(format!(
            "{verb} {field}: a currency-declared field is written ONLY a Currency \
             value — found {other:?}"
        ))),
    }
}

/// The Currency-lane counterpart of [`EffectExecutor::store_range_check`]
/// (a free function for the same "consults no executor state" reason
/// [`currency_write_value`] is one): the BSL spec's own declared domain for
/// the type is `[0, ∞)` (`bsl-language.rst` §1.5's Currency row) — enforced
/// HERE at the store boundary because a `$`-suffixed LITERAL is already
/// lex-bound non-negative (`E-LEX-022`), but a rule-COMPUTED
/// `Value::Currency` (e.g. a subtraction) is not checked anywhere upstream
/// of this write.
fn store_range_check_currency(field: &str, value: Currency) -> Result<(), EvalError> {
    if value.micro_units() < 0 {
        return Err(EvalError::coded(
            EvalCode::StoreRangeViolation,
            format!(
                "storing a negative Currency value to {field} leaves its declared \
                 [0, ∞) domain (bsl-language.rst §1.5) — a loud failure, never a clamp"
            ),
        ));
    }
    Ok(())
}

/// The six graph-shape verbs Task 12's collect-then-apply pre-state split
/// (§4.2 chapter C4) does not defer: deferring a MINTING verb needs a
/// placeholder-id scheme that repair does not specify (see
/// [`EffectExecutor::collect_effects`]'s own doc). The single source of
/// truth [`check_no_deferred_shape_verbs`] (the LOAD-time gate) walks
/// against.
pub(crate) const DEFERRED_SHAPE_VERBS: [&str; 6] = [
    "add-node",
    "remove-node",
    "add-edge",
    "remove-edge",
    "add-hyperedge",
    "remove-hyperedge",
];

/// Refuse, **at load**, a rule whose `<when>`/`<effects>` use any of the
/// `DEFERRED_SHAPE_VERBS` (#519 fix round, fix 4 — the regression this
/// repairs). Before this gate such a rule loaded clean and only failed at
/// RUNTIME, the first tick whose guard admitted a subject — the exact
/// load-passes/execute-dies shape `tick.rs::check_sources_servable`
/// exists to prevent for bindings, and a violation of this chapter's own
/// law: "every check in this chapter runs at content load, before any
/// tick executes" (§3). Walks guard/for-each nesting the same way
/// [`crate::rule_pipeline`]'s fold walk (`typecheck_rule_folds`) does —
/// the whole rule form, since these verbs are legal only in `<when>`
/// (never — they are effect-position-only) or `<effects>`, and walking
/// the header costs nothing extra.
///
/// `EffectExecutor::collect_items`'s own runtime refusal for these six
/// verbs stays live as defense in depth for any caller that reaches
/// `collect_effects` directly, bypassing this gate (as this module's own
/// `the_collect_path_refuses_a_shape_verb_naming_it` test does) — but
/// every rule `rule_pipeline::load_rule_form` accepts is now refused HERE
/// first, before that arm can ever fire in production.
///
/// # Errors
///
/// An uncoded message (no §2 grammar production is violated — every one
/// of these verbs IS legal content; this is Task 12's own composition
/// limit, the same no-invented-codes precedent `LoadError::Content` and
/// `evaluator.rs`'s `EFFECT_POSITION_ONLY`/`UNSERVED_EXPRESSION_HEADS`
/// tables use) naming the verb and the follow-on that will serve it.
pub fn check_no_deferred_shape_verbs(rule: &SExpr) -> Result<(), String> {
    if let Some(verb) = find_deferred_shape_verb(rule) {
        return Err(format!(
            "({verb} …) is one of the six graph-shape verbs Task 12's \
             collect-then-apply pre-state repair (§4.2 chapter C4) does \
             not yet serve — deferring a MINTING verb needs a \
             placeholder-id scheme this repair does not specify, so \
             run_tick's two-pass split cannot defer {verb} the way it \
             defers update-node. Refused HERE, at load (§3's own law: \
             every check in this chapter runs at content load, before any \
             tick executes), rather than letting the rule load clean and \
             abort the first tick whose guard admits a subject. The \
             follow-on that will serve {verb} is the placeholder-id \
             design EffectExecutor::collect_effects's own doc escalates."
        ));
    }
    Ok(())
}

/// Depth-first search for the first [`DEFERRED_SHAPE_VERBS`] head anywhere
/// in `expr`. `Option<&str>` borrows the symbol straight out of the AST —
/// no allocation for a check that runs on every rule at load.
///
/// **Payload-LABEL-only at `emit`, corrected (G1, #534 fix round 2 — one
/// root cause with the sibling fix in `grammar::check_enum_ref_membership`;
/// supersedes F5(b), #534 fix round item 5, which itself mirrored the
/// same label-as-head misreading `grammar::check_type_operands_are_enum_
/// refs` R2 fixed, #528 delta-verify rider).** `emit`'s own trailing
/// operands are its type operand (an `<enum-ref>`, never a form) and zero
/// or more `<payload-item>`s (`(<symbol> <expr>)`, §2.8) — a payload
/// item's LABEL is an unconstrained `Atom::Symbol` that nothing stops
/// content from spelling like one of `DEFERRED_SHAPE_VERBS` (`add-node`,
/// `remove-edge`, …), even though it is a label, never a nested verb
/// invocation. Before F5(b), the unconditional recursion below treated
/// every child list's head as a fresh candidate wherever it sat, so
/// `(emit EventType/RUPTURE (add-node 5) (severity 1))` — a payload item
/// merely LABELED `add-node` — was wrongly refused as if it invoked the
/// verb.
///
/// F5(b)'s own fix over-corrected: `return None` on a matched `emit` head
/// skipped `emit`'s ENTIRE subtree, not just the payload LABELS. Its own
/// justification — "none of `DEFERRED_SHAPE_VERBS` is legal in expression
/// position, so nothing genuinely nested inside `emit`'s own operands
/// could ever be a real match" — reasons about what content SHOULD look
/// like, not what the reader will happily parse: nothing stops a payload
/// item's VALUE from spelling `(add-node NodeType/SOCIAL_CLASS 5)`
/// verbatim, and THIS is exactly the ILLEGAL-but-syntactically-present
/// case [`check_no_deferred_shape_verbs`] exists to catch at load rather
/// than let die mid-tick — stopping at `emit` silently let it back through
/// (a rule loading clean and aborting the first tick whose guard admitted
/// a subject, the exact shape this whole gate exists to prevent). The
/// corrected discipline: once a matched `emit` head is confirmed, recurse
/// into each payload item's element-1 VALUE ONLY — never treating the
/// payload item itself as a form headed by its LABEL (element 0).
/// `guard`/`for-each` are unaffected: neither head matches here, so a form
/// headed by either still falls through to the unconditional recursion,
/// reaching any REAL deferred-shape verb nested in their bodies exactly as
/// before.
///
/// **G1's own fix still assumed two invariants no check here established
/// (H1, #534 fix round 3 — one root cause with `grammar::
/// check_enum_ref_membership`'s own sibling fix): that `items[1]` really
/// is `emit`'s type operand (so `items[2..]` are genuinely payload
/// items), and that a payload item has exactly two elements (so
/// `pair.get(1)` finds its whole value).** Neither holds once a SECOND
/// malformation is present. `(m (emit (add-node NodeType/SOCIAL_CLASS
/// 5)))` is a payload item whose value is a NESTED `emit` MISSING its own
/// type operand — `items[1]` there is `(add-node NodeType/SOCIAL_CLASS
/// 5)` itself (the real verb invocation), not an `<enum-ref>`, and the
/// old unconditional `skip(2)` silently treated it as "the confirmed type
/// operand" and skipped it, so `add-node` was never inspected as a head
/// at all. `(m 1 (add-node NodeType/SOCIAL_CLASS 5))` is an OVER-ARITY
/// payload item — three elements, not two — so `pair.get(1)` (the literal
/// `1`) never reached `pair[2]`, the real invocation. Corrected:
/// `items[1]` is only trusted as `emit`'s type operand when it is
/// genuinely an `<enum-ref>` (`items[2..]` payload, `pair[1..]` every
/// value); otherwise no positional assumption is safe once the
/// type-operand slot itself is broken, so every item after the head is
/// recursed into in full (`items[1..]`, ordinary recursion) —
/// `grammar::check_type_operands_are_enum_refs` is the earlier load-time
/// gate that refuses this exact malformed-nested-`emit` shape outright,
/// but this function must not rely on that gate having run first (it is
/// driven directly by this module's own tests, same as its sibling).
fn find_deferred_shape_verb(expr: &SExpr) -> Option<&str> {
    let SExpr::List(items) = expr else {
        return None;
    };
    if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
        if DEFERRED_SHAPE_VERBS.contains(&head.as_str()) {
            return Some(head.as_str());
        }
        if head == "emit" {
            // Payload-LABEL-only (G1), guarded (H1): see this function's
            // own doc for the full reasoning — `items[1]` is only trusted
            // as emit's type operand when it is genuinely an `<enum-ref>`.
            if matches!(items.get(1), Some(SExpr::Atom(Atom::EnumRef { .. }))) {
                for payload_item in items.iter().skip(2) {
                    if let SExpr::List(pair) = payload_item {
                        for value in pair.iter().skip(1) {
                            if let Some(verb) = find_deferred_shape_verb(value) {
                                return Some(verb);
                            }
                        }
                    }
                }
            } else {
                for item in items.iter().skip(1) {
                    if let Some(verb) = find_deferred_shape_verb(item) {
                        return Some(verb);
                    }
                }
            }
            return None;
        }
    }
    for item in items {
        if let Some(verb) = find_deferred_shape_verb(item) {
            return Some(verb);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel::IntrinsicCosts;
    use crate::intrinsic_host::EmptyIntrinsicHost;
    use crate::reader::read;
    use crate::types::{FieldDecl, FieldKind};
    use crate::write_log::CollectingWriteLog;
    use babylon_graph::memory::MemoryGraph;

    fn types() -> TypeEnv {
        TypeEnv {
            fields: HashMap::from([
                (
                    "social-class/agitation".to_owned(),
                    FieldDecl {
                        ty: BslType::Intensity,
                        kind: FieldKind::Intensive,
                    },
                ),
                (
                    "social-class/head-count".to_owned(),
                    FieldDecl {
                        ty: BslType::Int,
                        kind: FieldKind::Extensive,
                    },
                ),
            ]),
            exemptions: &[],
        }
    }

    /// The `types()` fixture above declares no enum-typed field — an empty
    /// registry is the honest "no `defenum`s in scope" input for every
    /// test in this module that does not build its own (the enum-write
    /// tests below build `organization_types()`/an `OrgKind` registry
    /// instead).
    fn enums() -> EnumRegistry {
        EnumRegistry::default()
    }

    struct Fixture {
        graph: MemoryGraph,
        self_id: NodeId,
        costs: IntrinsicCosts,
    }

    impl Fixture {
        fn new() -> Self {
            let mut graph = MemoryGraph::new();
            let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
            graph
                .update_node(self_id, "social-class/agitation", 0.10)
                .unwrap();
            Self {
                graph,
                self_id,
                costs: IntrinsicCosts::default(),
            }
        }

        /// Execute an `(effects …)` source against the fixture, returning
        /// the collected event stream.
        #[allow(clippy::type_complexity)]
        fn run(
            &mut self,
            effects_source: &str,
            fuel: &mut u64,
        ) -> Result<Vec<(String, Vec<(String, Value)>)>, EvalError> {
            let (form, _) = read(effects_source).expect("effects source must parse");
            let SExpr::List(items) = form else {
                unreachable!()
            };
            let env = EvalEnv {
                bindings: HashMap::from([("self".to_owned(), Value::NodeRef(self.self_id))]),
                intrinsic_costs: &self.costs,
                graph: None,
                types: None,
                enums: None,
                elements: Vec::new(),
                draw_context: None,
            };
            let types = types();
            let enums = enums();
            let mut executor = EffectExecutor::new(&types, &enums, None);
            let mut sink = CollectingSink::default();
            executor.execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut self.graph,
                &mut sink,
                fuel,
            )?;
            Ok(sink.events)
        }

        /// [`Self::run`], with a closed vocabulary threaded (Task 8,
        /// Organization foundation plan) — the runtime enforcement red/green
        /// tests below drive this rather than duplicating `run`'s body.
        #[allow(clippy::type_complexity)]
        fn run_with_vocabulary(
            &mut self,
            effects_source: &str,
            fuel: &mut u64,
            vocabulary: &crate::vocabulary::ClosedVocabulary,
        ) -> Result<Vec<(String, Vec<(String, Value)>)>, EvalError> {
            let (form, _) = read(effects_source).expect("effects source must parse");
            let SExpr::List(items) = form else {
                unreachable!()
            };
            let env = EvalEnv {
                bindings: HashMap::from([("self".to_owned(), Value::NodeRef(self.self_id))]),
                intrinsic_costs: &self.costs,
                graph: None,
                types: None,
                enums: None,
                elements: Vec::new(),
                draw_context: None,
            };
            let types = types();
            let enums = enums();
            let mut executor = EffectExecutor::new(&types, &enums, Some(vocabulary));
            let mut sink = CollectingSink::default();
            executor.execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut self.graph,
                &mut sink,
                fuel,
            )?;
            Ok(sink.events)
        }

        /// The same run with the ADR182 R1 write log installed. Returns the
        /// log ALONGSIDE the result rather than through it: "a failed write
        /// leaves no record" is only assertable if the log survives the
        /// error.
        fn run_observed(
            &mut self,
            effects_source: &str,
            fuel: &mut u64,
        ) -> (Result<(), EvalError>, CollectingWriteLog) {
            let (form, _) = read(effects_source).expect("effects source must parse");
            let SExpr::List(items) = form else {
                unreachable!()
            };
            let env = EvalEnv {
                bindings: HashMap::from([("self".to_owned(), Value::NodeRef(self.self_id))]),
                intrinsic_costs: &self.costs,
                graph: None,
                types: None,
                enums: None,
                elements: Vec::new(),
                draw_context: None,
            };
            let types = types();
            let enums = enums();
            let mut log = CollectingWriteLog::new();
            let mut sink = CollectingSink::default();
            let result = {
                let mut executor =
                    EffectExecutor::observed(&types, &enums, None, "hunger/agitate", &mut log);
                executor.execute_effects(
                    &items[1..],
                    &env,
                    &EmptyIntrinsicHost,
                    &mut self.graph,
                    &mut sink,
                    fuel,
                )
            };
            (result, log)
        }
    }

    #[test]
    fn the_demo_rule_effect_executes_with_pinned_fuel() {
        // The §5.6 effect against agitation = 0.10: update-node charges its
        // static per-node sum exactly — verb(3) + self(1) + qname(0) +
        // update-op(1) + literal(0) = 5.
        let mut fixture = Fixture::new();
        let mut fuel = 64;
        fixture
            .run(
                "(effects (update-node self social-class/agitation (add 0.05i)))",
                &mut fuel,
            )
            .unwrap();
        let stored = fixture
            .graph
            .node_attribute(fixture.self_id, "social-class/agitation")
            .unwrap();
        assert!((stored - 0.15).abs() < 1e-12);
        assert_eq!(fuel, 59, "5 consumed — a conformance-vector quantity");
    }

    #[test]
    fn the_store_boundary_rejects_out_of_range_never_clamps() {
        let mut fixture = Fixture::new();
        let mut fuel = 64;
        let err = fixture
            .run(
                "(effects (update-node self social-class/agitation (add 0.95i)))",
                &mut fuel,
            )
            .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::StoreRangeViolation));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-020");
        // The failed write left the attribute untouched.
        let stored = fixture
            .graph
            .node_attribute(fixture.self_id, "social-class/agitation")
            .unwrap();
        assert!((stored - 0.10).abs() < 1e-12);
    }

    /// Train B item 6 (#591): a `real`-declared field carries NO
    /// store-boundary range law — E-EVAL-020 is the unit-interval types'
    /// check (`store_range_check`'s `matches!` names exactly
    /// Probability/Intensity/Coefficient), and `Real` is not one of them.
    /// A write outside `[0,1]`, and a negative write, both land VERBATIM —
    /// bit-exact, no tolerance — matching what `numeric_write_value`
    /// already does with any computed f64. (No negative FRACTIONAL seed
    /// literal exists — every scaled lane is lex-bounded non-negative —
    /// but a negative `int` literal seeds fine (`attribute_value_real`
    /// takes any `Atom::Int` exact to 2⁵³), and writes may be negative
    /// either way; the store does not care.)
    #[test]
    fn real_field_store_has_no_range_check() {
        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = TypeEnv {
            fields: HashMap::from([(
                "social-class/balance".to_owned(),
                FieldDecl {
                    ty: BslType::Real,
                    kind: FieldKind::Intensive,
                },
            )]),
            exemptions: &[],
        };
        let enums = enums();
        let costs = IntrinsicCosts::default();
        let env = EvalEnv {
            bindings: HashMap::from([("self".to_owned(), Value::NodeRef(self_id))]),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        // The E-EVAL-020 boundary itself, probed directly with the two
        // Train-B-content magnitudes the task brief names: both pass,
        // for Real, where a unit-interval field would refuse loudly.
        let probe = EffectExecutor::new(&types, &enums, None);
        for value in [6_962.099_999_999_999_f64, -0.052_631_578_947_368_42_f64] {
            probe
                .store_range_check("social-class/balance", value)
                .unwrap();
        }
        // End-to-end through `update-node`: the computed binary64 lands
        // bit-verbatim. `(+ 6962 0.1c)` is 6962.1000000000003637… — one
        // ulp ABOVE the 6962.099999999999… decimal, because the sum of
        // the two rounded operands is not the rounding of the decimal
        // sum (the IEEE subtlety this pin exists to state, not absorb).
        // `(- 0 (/ 1.0c 19))` IS exactly -0.05263157894736842.
        for (effects, expected) in [
            (
                "(effects (update-node self social-class/balance (set (+ 6962 0.1c))))",
                6962.0_f64 + 0.1_f64,
            ),
            (
                "(effects (update-node self social-class/balance (set (- 0 (/ 1.0c 19)))))",
                -(1.0_f64 / 19.0_f64),
            ),
        ] {
            let (form, _) = read(effects).expect("effects source must parse");
            let SExpr::List(items) = form else {
                unreachable!()
            };
            let mut executor = EffectExecutor::new(&types, &enums, None);
            let mut sink = CollectingSink::default();
            let mut fuel = 256;
            executor
                .execute_effects(
                    &items[1..],
                    &env,
                    &EmptyIntrinsicHost,
                    &mut graph,
                    &mut sink,
                    &mut fuel,
                )
                .unwrap();
            let stored = graph
                .node_attribute(self_id, "social-class/balance")
                .unwrap();
            assert_eq!(
                stored.to_bits(),
                expected.to_bits(),
                "{effects}: got 0x{:016x}, want 0x{:016x}",
                stored.to_bits(),
                expected.to_bits()
            );
        }
        // The second end-to-end value and the brief's named negative
        // decimal are the same binary64 — pinned, not assumed.
        assert_eq!(
            (-(1.0_f64 / 19.0_f64)).to_bits(),
            (-0.052_631_578_947_368_42_f64).to_bits()
        );
    }

    #[test]
    fn add_node_introduces_a_name_later_effects_can_use() {
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        fixture
            .run(
                "(effects \
                   (add-node NodeType/SOCIAL_CLASS recruit (social-class/agitation 0.2i)) \
                   (add-edge EdgeType/SOLIDARITY recruit self :strength 0.5c))",
                &mut fuel,
            )
            .unwrap();
        assert_eq!(fixture.graph.edge_count(), 1);
    }

    // ---- Task 8 (Organization foundation plan): closed-vocabulary
    // enforcement at verb execution — add-node/add-edge/add-hyperedge's
    // type operand is checked when a registry is threaded ----

    fn probe_vocabulary() -> crate::vocabulary::ClosedVocabulary {
        crate::vocabulary::ClosedVocabulary::new([
            (
                crate::vocabulary::EnumKind::NodeType,
                vec!["SOCIAL_CLASS".to_owned()],
            ),
            (
                crate::vocabulary::EnumKind::EdgeType,
                vec!["SOLIDARITY".to_owned()],
            ),
            (
                crate::vocabulary::EnumKind::HyperedgeType,
                vec!["CELL".to_owned()],
            ),
        ])
        .unwrap()
    }

    #[test]
    fn add_node_with_an_unregistered_type_is_a_loud_eval_error_under_a_declared_vocabulary() {
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        let vocabulary = probe_vocabulary();
        let err = fixture
            .run_with_vocabulary(
                "(effects (add-node NodeType/FOO recruit))",
                &mut fuel,
                &vocabulary,
            )
            .unwrap_err();
        assert!(err.message.contains("E-LOAD-031"), "{}", err.message);
        assert!(err.message.contains("FOO"), "{}", err.message);
        // F6 (#534 fix round item 6): the offending verb is named too.
        assert!(err.message.contains("add-node"), "{}", err.message);
        // The node must never have minted.
        assert_eq!(fixture.graph.nodes("FOO").len(), 0);
    }

    #[test]
    fn add_edge_with_an_unregistered_type_is_a_loud_eval_error_under_a_declared_vocabulary() {
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        let vocabulary = probe_vocabulary();
        let err = fixture
            .run_with_vocabulary(
                "(effects (add-edge EdgeType/NOWHERE self self :strength 0.5c))",
                &mut fuel,
                &vocabulary,
            )
            .unwrap_err();
        assert!(err.message.contains("E-LOAD-031"), "{}", err.message);
        assert_eq!(fixture.graph.edge_count(), 0);
    }

    #[test]
    fn add_hyperedge_with_an_unregistered_type_is_a_loud_eval_error_under_a_declared_vocabulary() {
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        let vocabulary = probe_vocabulary();
        let err = fixture
            .run_with_vocabulary(
                "(effects (add-hyperedge HyperedgeType/NOWHERE nucleus (members self)))",
                &mut fuel,
                &vocabulary,
            )
            .unwrap_err();
        assert!(err.message.contains("E-LOAD-031"), "{}", err.message);
    }

    #[test]
    fn add_hyperedge_under_a_nodetype_only_vocabulary_is_inert_for_hyperedgetype() {
        // G3(a) (#534 fix round 2): the eval-leg per-kind inertness pin,
        // site-isolation style — mirrors F1's own scenario-load pin
        // (`vocabulary::tests::a_kind_absent_from_the_vocabulary_is_inert_
        // not_e_load_031`) one producer down, at verb EXECUTION. A
        // vocabulary that declares NodeType but never HyperedgeType must
        // leave HyperedgeType's own membership checking exactly as inert
        // here as `ClosedVocabulary::check_enum_ref` already proves at the
        // registry level — a kind never opted into checking is not being
        // checked, never a fallback (§3.6).
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        let vocabulary = crate::vocabulary::ClosedVocabulary::new([(
            crate::vocabulary::EnumKind::NodeType,
            vec!["SOCIAL_CLASS".to_owned()],
        )])
        .unwrap();
        fixture
            .run_with_vocabulary(
                "(effects (add-hyperedge HyperedgeType/ANYTHING nucleus (members self)))",
                &mut fuel,
                &vocabulary,
            )
            .expect("HyperedgeType was never declared — its checking must stay inert");
    }

    #[test]
    fn a_registered_type_mints_clean_under_a_declared_vocabulary() {
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        let vocabulary = probe_vocabulary();
        fixture
            .run_with_vocabulary(
                "(effects \
                   (add-node NodeType/SOCIAL_CLASS recruit) \
                   (add-edge EdgeType/SOLIDARITY recruit self :strength 0.5c))",
                &mut fuel,
                &vocabulary,
            )
            .expect("a registered member must mint clean");
        assert_eq!(fixture.graph.edge_count(), 1);
    }

    #[test]
    fn the_same_typo_source_mints_with_no_vocabulary_threaded_backward_compat_pin() {
        // The plan's own backward-compatibility proof, at the THIRD
        // producer (verb execution): `Fixture::run` threads `None` — the
        // same unchecked behavior as every EXISTING test in this module.
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        fixture
            .run("(effects (add-node NodeType/FOO recruit))", &mut fuel)
            .expect("with no threaded vocabulary, membership is unchecked (backward compat)");
        assert_eq!(fixture.graph.nodes("FOO").len(), 1);
    }

    #[test]
    fn roster_replacement_is_remove_then_add_in_one_list() {
        // §2.8 draft ruling: no add-member verb exists; changing a roster
        // is remove-hyperedge + add-hyperedge in one effect list.
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        fixture
            .run(
                "(effects \
                   (add-node NodeType/SOCIAL_CLASS comrade) \
                   (add-hyperedge HyperedgeType/CELL nucleus (members self)) \
                   (remove-hyperedge nucleus) \
                   (add-hyperedge HyperedgeType/CELL grown (members self comrade)))",
                &mut fuel,
            )
            .unwrap();
        let grown = fixture.graph.hyperedges_of(fixture.self_id, "CELL");
        let grown = grown.unwrap();
        assert_eq!(grown.len(), 1);
        assert_eq!(
            fixture.graph.members_of(grown[0]).unwrap().len(),
            2,
            "the replacement roster, whole — never pairwise edges"
        );
    }

    #[test]
    fn substrate_discipline_surfaces_as_e_eval_031() {
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        // Duplicate member: the substrate refuses, nothing deduplicates.
        let err = fixture
            .run(
                "(effects (add-hyperedge HyperedgeType/CELL c (members self self)))",
                &mut fuel,
            )
            .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::ExistenceDiscipline));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-031");
        // Removing an edge that does not exist: absence is never success.
        let mut fuel2 = 128;
        let err2 = fixture
            .run(
                "(effects (remove-edge EdgeType/SOLIDARITY self self))",
                &mut fuel2,
            )
            .unwrap_err();
        assert_eq!(err2.code, Some(EvalCode::ExistenceDiscipline));
    }

    #[test]
    fn a_false_guard_skips_and_does_not_charge_its_effects() {
        let mut fixture = Fixture::new();
        let mut fuel = 64;
        fixture
            .run(
                "(effects (guard (< 1 0) \
                   (update-node self social-class/agitation (add 0.05i))))",
                &mut fuel,
            )
            .unwrap();
        // guard(1) + cmp(1) + two literals(0) = 2; the verb never charged.
        assert_eq!(fuel, 62);
        let stored = fixture
            .graph
            .node_attribute(fixture.self_id, "social-class/agitation")
            .unwrap();
        assert!((stored - 0.10).abs() < 1e-12, "untaken effects never apply");
    }

    #[test]
    fn emit_collects_the_evaluated_payload() {
        let mut fixture = Fixture::new();
        let mut fuel = 64;
        let events = fixture
            .run(
                "(effects (emit EventType/RUPTURE (severity 0.9c) (tick 52)))",
                &mut fuel,
            )
            .unwrap();
        assert_eq!(
            events,
            vec![(
                "RUPTURE".to_owned(),
                vec![
                    ("severity".to_owned(), Value::Real(0.9)),
                    ("tick".to_owned(), Value::Int(52)),
                ]
            )]
        );
    }

    #[test]
    fn currency_writes_into_a_non_currency_field_are_refused_not_cast() {
        // T3 #491, OQ-J: `social-class/head-count` is Int-declared (see
        // `types()` above) — a kind mismatch, not the retired "no Currency
        // storage exists at all" gap.
        let mut fixture = Fixture::new();
        let mut fuel = 64;
        let err = fixture
            .run(
                "(effects (update-node self social-class/head-count (set 100$)))",
                &mut fuel,
            )
            .unwrap_err();
        assert!(
            err.message.contains("needs a currency-declared field"),
            "{err}"
        );
    }

    #[test]
    fn a_declared_id_may_not_shadow_bindings_or_reserved_names() {
        let mut fixture = Fixture::new();
        for id in ["self", "it"] {
            let mut fuel = 64;
            let err = fixture
                .run(
                    &format!("(effects (add-node NodeType/SOCIAL_CLASS {id}))"),
                    &mut fuel,
                )
                .unwrap_err();
            assert!(err.message.contains("shadows"), "{id}: {err}");
        }
    }

    #[test]
    fn hyperedge_field_inits_are_a_loud_phase_2_gap() {
        // The mint-time init lane STAYS refused after Task 6 — with the
        // reason corrected: the own-field storage exists (Task 5) and
        // `update-hyperedge`/`(hyperedge-attr …)` are its writers, but the
        // init sugar on `add-hyperedge` itself is not routed to it.
        let mut fixture = Fixture::new();
        let mut fuel = 128;
        let err = fixture
            .run(
                "(effects (add-hyperedge HyperedgeType/CELL c (members self) \
                   (social-class/agitation 0.5i)))",
                &mut fuel,
            )
            .unwrap_err();
        assert!(err.message.contains("not served at mint time"), "{err}");
    }

    // ---- the ADR182 R1 write log ----

    /// Read the substrate back through the §2.6 query surface, whose
    /// iteration order is contractual — NOT through `Debug`, whose `HashMap`
    /// ordering is not.
    fn snapshot(graph: &MemoryGraph) -> Vec<String> {
        let mut out = Vec::new();
        for id in graph.nodes("SOCIAL_CLASS") {
            let agitation = graph.node_attribute(id, "social-class/agitation").ok();
            let head_count = graph.node_attribute(id, "social-class/head-count").ok();
            out.push(format!("node {id:?} {agitation:?} {head_count:?}"));
        }
        for (from, to) in graph.edges("SOLIDARITY") {
            out.push(format!("edge {from:?}->{to:?}"));
        }
        out
    }

    /// The script the equivalence contract runs: one of every recorded verb.
    const EVERY_VERB: &str = "(effects \
         (update-node self social-class/agitation (add 0.05i)) \
         (add-node NodeType/SOCIAL_CLASS recruit (social-class/agitation 0.2i)) \
         (add-edge EdgeType/SOLIDARITY recruit self :strength 0.5c) \
         (remove-edge EdgeType/SOLIDARITY recruit self) \
         (add-hyperedge HyperedgeType/CELL nucleus (members self recruit)) \
         (remove-hyperedge nucleus))";

    #[test]
    fn observation_changes_neither_state_nor_fuel() {
        // Discipline 1, the reason the log is safe to ship: observing is not
        // a semantic mode. If this ever fails, the write log has become a
        // participant and the engine's determinism hash is at risk.
        let mut unobserved = Fixture::new();
        let mut plain_fuel = 256;
        unobserved.run(EVERY_VERB, &mut plain_fuel).unwrap();

        let mut observed = Fixture::new();
        let mut observed_fuel = 256;
        let (result, _log) = observed.run_observed(EVERY_VERB, &mut observed_fuel);
        result.unwrap();

        assert_eq!(
            snapshot(&unobserved.graph),
            snapshot(&observed.graph),
            "an observed run must leave the substrate exactly as an unobserved one does"
        );
        assert_eq!(
            plain_fuel, observed_fuel,
            "the log must not charge the §4.5 meter — fuel is a conformance quantity"
        );
    }

    #[test]
    fn ordinal_overflow_is_refused_before_any_substrate_mutation() {
        for observed in [false, true] {
            let (form, _) = read("(effects (update-node self social-class/agitation (set 0.9i)))")
                .expect("effects source must parse");
            let SExpr::List(items) = form else {
                unreachable!()
            };
            let mut fixture = Fixture::new();
            let env = EvalEnv {
                bindings: HashMap::from([("self".to_owned(), Value::NodeRef(fixture.self_id))]),
                intrinsic_costs: &fixture.costs,
                graph: None,
                types: None,
                enums: None,
                elements: Vec::new(),
                draw_context: None,
            };
            let types = types();
            let enums = enums();
            let mut log = CollectingWriteLog::new();
            let mut sink = CollectingSink::default();
            let mut fuel = 64;
            let result = {
                let mut executor = if observed {
                    EffectExecutor::observed(&types, &enums, None, "hunger/agitate", &mut log)
                } else {
                    EffectExecutor::new(&types, &enums, None)
                };
                executor.next_write_ordinal = u64::from(u32::MAX) + 1;
                executor.execute_effects(
                    &items[1..],
                    &env,
                    &EmptyIntrinsicHost,
                    &mut fixture.graph,
                    &mut sink,
                    &mut fuel,
                )
            };
            let error = result.expect_err("an unrepresentable write ordinal must fail loudly");
            assert_eq!(error.code, Some(EvalCode::Overflow), "observe={observed}");
            assert_eq!(
                fixture
                    .graph
                    .node_attribute(fixture.self_id, "social-class/agitation")
                    .unwrap()
                    .to_bits(),
                0.10_f64.to_bits(),
                "observe={observed}: the capacity check must precede mutation"
            );
            assert!(
                log.records.is_empty(),
                "observe={observed}: a refused write cannot publish a record"
            );
        }
    }

    #[test]
    fn every_recorded_verb_lands_in_source_order() {
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (result, log) = fixture.run_observed(EVERY_VERB, &mut fuel);
        result.unwrap();

        let self_id = fixture.self_id;
        let recruit = NodeId(self_id.0 + 1);
        assert_eq!(
            log.writes(),
            vec![
                Write::NodeAttribute {
                    id: self_id,
                    field: "social-class/agitation".to_owned(),
                    previous: Some(0.10),
                    value: 0.10 + 0.05,
                },
                Write::NodeAdded {
                    id: recruit,
                    node_type: "SOCIAL_CLASS".to_owned(),
                },
                Write::NodeAttribute {
                    id: recruit,
                    field: "social-class/agitation".to_owned(),
                    previous: None,
                    value: 0.2,
                },
                Write::EdgeAdded {
                    edge_type: "SOLIDARITY".to_owned(),
                    from: recruit,
                    to: self_id,
                    strength: 0.5,
                },
                Write::EdgeRemoved {
                    edge_type: "SOLIDARITY".to_owned(),
                    from: recruit,
                    to: self_id,
                },
                Write::HyperedgeAdded {
                    id: HyperedgeId(0),
                    hyperedge_type: "CELL".to_owned(),
                    members: vec![self_id, recruit],
                },
                Write::HyperedgeRemoved { id: HyperedgeId(0) },
            ],
        );
        assert_eq!(
            log.records.iter().map(|r| r.ordinal).collect::<Vec<_>>(),
            (0..7).collect::<Vec<_>>(),
            "ordinals count writes performed, densely, in source order"
        );
        assert!(
            log.records.iter().all(|r| r.rule == "hunger/agitate"),
            "every record carries the firing rule's id"
        );
    }

    /// A host whose intrinsic returns a non-finite `Real`. The trait doc
    /// names exactly this as the evaluator's defence-in-depth case: the
    /// evaluator guards its own arithmetic, so a non-finite can only reach
    /// the store boundary from across the intrinsic seam.
    struct RogueIntrinsicHost;

    impl IntrinsicHost for RogueIntrinsicHost {
        fn call(
            &self,
            _name: &str,
            _args: &[Value],
            _ctx: crate::intrinsic_host::IntrinsicCallCtx<'_>,
        ) -> Result<Value, EvalError> {
            Ok(Value::Real(f64::NAN))
        }
    }

    #[test]
    fn a_non_finite_never_reaches_the_substrate_on_any_write_path() {
        // §3.3's range check only rejects non-finites on unit-interval
        // fields (`(0.0..=1.0).contains(NaN)` is false). head-count is Int,
        // so without the numeric_write_value guard a NaN would land in the
        // substrate — and in the tick hash. Both write paths are checked:
        // update-node's `set`, and add-node's field-init.
        for source in [
            "(effects (update-node self social-class/head-count (set (rogue))))",
            "(effects (add-node NodeType/SOCIAL_CLASS n (social-class/head-count (rogue))))",
        ] {
            let mut fixture = Fixture::new();
            let mut fuel = 256;
            let (form, _) = read(source).expect("effects source must parse");
            let SExpr::List(items) = form else {
                unreachable!()
            };
            // The intrinsic must be DECLARED or the call fails as a loader
            // bug (E-LOAD-021) before it can return anything at all.
            let costs = IntrinsicCosts::new(HashMap::from([("rogue".to_owned(), 1_u64)]));
            let env = EvalEnv {
                bindings: HashMap::from([("self".to_owned(), Value::NodeRef(fixture.self_id))]),
                intrinsic_costs: &costs,
                graph: None,
                types: None,
                enums: None,
                elements: Vec::new(),
                draw_context: None,
            };
            let types = types();
            let enums = enums();
            let mut executor = EffectExecutor::new(&types, &enums, None);
            let mut sink = CollectingSink::default();
            let err = executor
                .execute_effects(
                    &items[1..],
                    &env,
                    &RogueIntrinsicHost,
                    &mut fixture.graph,
                    &mut sink,
                    &mut fuel,
                )
                .unwrap_err();
            assert_eq!(err.code, Some(EvalCode::NonFinite), "{source}");
            assert!(
                fixture
                    .graph
                    .node_attribute(fixture.self_id, "social-class/head-count")
                    .is_err(),
                "the field must still hold nothing — refused, not stored: {source}"
            );
        }
    }

    /// Copilot review on #585: a combine can yield −0.0 (0.0 scaled by a
    /// negative operand); the canonical hash encodes raw binary64 bits, so
    /// the store boundary canonicalizes — the substrate must never
    /// distinguish −0.0 from +0.0. Node lane here, edge lane next.
    #[test]
    fn apply_pending_write_canonicalizes_negative_zero() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(id, "social-class/agitation", 0.0)
            .unwrap();
        let write = PendingWrite {
            target: WriteTarget::Node(id),
            field: "social-class/agitation".to_owned(),
            op: UpdateOp::Scale,
            operand: WriteOperand::Real(-1.0),
        };
        let types = types();
        let enums = enums();
        let mut applier = EffectExecutor::new(&types, &enums, None);
        applier.apply_pending_write(&write, &mut graph).unwrap();
        let stored = graph.node_attribute(id, "social-class/agitation").unwrap();
        assert_eq!(stored.to_bits(), 0.0_f64.to_bits(), "stored: {stored}");
    }

    #[test]
    fn apply_pending_write_canonicalizes_negative_zero_on_the_edge_lane() {
        let (mut graph, a, b) = edge_fixture();
        graph
            .update_edge("SOLIDARITY", a, b, "solidarity/tension", 0.0)
            .unwrap();
        let write = PendingWrite {
            target: WriteTarget::Edge(EdgeKey {
                source: a,
                target: b,
                edge_type: "SOLIDARITY".to_owned(),
            }),
            field: "solidarity/tension".to_owned(),
            op: UpdateOp::Scale,
            operand: WriteOperand::Real(-1.0),
        };
        let types = edge_types();
        let enums = enums();
        let mut applier = EffectExecutor::new(&types, &enums, None);
        applier.apply_pending_write(&write, &mut graph).unwrap();
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert_eq!(stored.to_bits(), 0.0_f64.to_bits(), "stored: {stored}");
    }

    /// Copilot review on #585: the `:strength` operand's only non-finite
    /// route is the intrinsic seam (source literals are Int/Currency/
    /// unit-interval Scaled; the evaluator guards its own arithmetic), so
    /// `add_edge` refuses it at the substrate boundary, before any mint.
    #[test]
    fn add_edge_refuses_a_non_finite_strength_at_the_substrate_boundary() {
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (form, _) =
            read("(effects (add-edge EdgeType/SOLIDARITY self self :strength (rogue)))")
                .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        // Declared, as the non-finite write-path test above notes: an
        // undeclared intrinsic fails as a loader bug first.
        let costs = IntrinsicCosts::new(HashMap::from([("rogue".to_owned(), 1_u64)]));
        let env = EvalEnv {
            bindings: HashMap::from([("self".to_owned(), Value::NodeRef(fixture.self_id))]),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let types = types();
        let enums = enums();
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let mut sink = CollectingSink::default();
        let err = executor
            .execute_effects(
                &items[1..],
                &env,
                &RogueIntrinsicHost,
                &mut fixture.graph,
                &mut sink,
                &mut fuel,
            )
            .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::NonFinite));
        assert_eq!(fixture.graph.edge_count(), 0, "no edge minted");
    }

    #[test]
    fn hyperedge_members_are_canonicalized_never_logged_as_declared() {
        // D25: declared member order is never observable. The substrate
        // sorts on insert, so the write log is the one surface that could
        // leak source order back — it must not.
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (result, log) = fixture.run_observed(
            "(effects \
               (add-node NodeType/SOCIAL_CLASS a) \
               (add-node NodeType/SOCIAL_CLASS b) \
               (add-hyperedge HyperedgeType/CELL c (members b a self)))",
            &mut fuel,
        );
        result.unwrap();
        let Some(Write::HyperedgeAdded { members, id, .. }) = log
            .writes()
            .into_iter()
            .find(|w| matches!(w, Write::HyperedgeAdded { .. }))
        else {
            panic!("expected a HyperedgeAdded record")
        };
        assert_eq!(
            members,
            vec![
                fixture.self_id,
                NodeId(fixture.self_id.0 + 1),
                NodeId(fixture.self_id.0 + 2)
            ],
            "declared order was b, a, self — the log must show ascending NodeId"
        );
        assert_eq!(
            members,
            fixture.graph.members_of(id).unwrap(),
            "the log must agree with what the substrate reports"
        );
    }

    #[test]
    fn a_hyperedge_member_list_is_logged_whole_never_expanded() {
        // VIII.9 held at the observation seam too: three members are one
        // record with three ids, never three pairwise records.
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (result, log) = fixture.run_observed(
            "(effects \
               (add-node NodeType/SOCIAL_CLASS a) \
               (add-node NodeType/SOCIAL_CLASS b) \
               (add-hyperedge HyperedgeType/CELL c (members self a b)))",
            &mut fuel,
        );
        result.unwrap();
        let hyperedges: Vec<_> = log
            .writes()
            .into_iter()
            .filter(|w| matches!(w, Write::HyperedgeAdded { .. }))
            .collect();
        assert_eq!(hyperedges.len(), 1, "one hyperedge, one record");
        let Write::HyperedgeAdded { members, .. } = &hyperedges[0] else {
            unreachable!()
        };
        assert_eq!(members.len(), 3);
    }

    #[test]
    fn a_write_that_failed_leaves_no_record() {
        // The mirror of §2.8's "absence is never success": the log records
        // what crossed the boundary, not what was attempted. The first
        // effect succeeds, the second trips the §3.3 range check.
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (result, log) = fixture.run_observed(
            "(effects \
               (update-node self social-class/head-count (set 100)) \
               (update-node self social-class/agitation (add 0.95i)))",
            &mut fuel,
        );
        assert_eq!(
            result.unwrap_err().code,
            Some(EvalCode::StoreRangeViolation)
        );
        assert_eq!(
            log.writes(),
            vec![Write::NodeAttribute {
                id: fixture.self_id,
                field: "social-class/head-count".to_owned(),
                previous: None,
                value: 100.0,
            }],
            "only the write that landed is recorded"
        );
    }

    #[test]
    fn an_untaken_guard_branch_records_nothing() {
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (result, log) = fixture.run_observed(
            "(effects (guard (< 1 0) \
               (update-node self social-class/agitation (add 0.05i))))",
            &mut fuel,
        );
        result.unwrap();
        assert!(
            log.records.is_empty(),
            "the ordinal counts writes, not effect items considered"
        );
    }

    #[test]
    fn a_set_probes_its_previous_value_without_inventing_one() {
        // Discipline 3. `set` does not otherwise read the field, so the log
        // probes — and an unwritten field is None, never an error and never
        // a fabricated 0.0 (the §3.5 honest-null discipline).
        let mut fixture = Fixture::new();
        let mut fuel = 256;
        let (result, log) = fixture.run_observed(
            "(effects \
               (update-node self social-class/head-count (set 100)) \
               (update-node self social-class/head-count (set 250)))",
            &mut fuel,
        );
        result.unwrap();
        let previous: Vec<_> = log
            .records
            .iter()
            .map(|r| match &r.write {
                Write::NodeAttribute { previous, .. } => previous.to_owned(),
                other => panic!("expected attribute writes, got {other:?}"),
            })
            .collect();
        assert_eq!(
            previous,
            vec![None, Some(100.0)],
            "never written -> None; then the value the first set stored"
        );
    }

    // ---- Task 10: for-each in effect position (§2.8 chapter C6) ----

    /// Build the rule's PRE-STATE (what `for-each`'s query reads, through
    /// `env.graph`) and the LIVE graph (what effects actually mutate, the
    /// `&mut` parameter every verb writes through) from the SAME
    /// construction sequence, as TWO SEPARATE `MemoryGraph` objects.
    ///
    /// **Scope, narrowed by the PR #519 fix round.** This split exists only
    /// to satisfy the borrow checker for [`Self::execute_effects`], the
    /// single-pass path that holds `env.graph` and a live `&mut` graph
    /// SIMULTANEOUSLY — which can never alias the same object in safe Rust,
    /// so a caller needs two objects or none at all. Production never does
    /// this (Task 12's `collect_effects`/`apply_pending_write` split takes
    /// SEQUENTIAL borrows of ONE graph — see [`Self::collect_then_apply`]
    /// below), and neither should a test that means to exercise production
    /// semantics: because the two objects here can never be the SAME graph,
    /// no test built on this split can observe an aliasing bug — an
    /// implementation that silently read the WRONG one would still see
    /// equal content and pass. That is exactly what the fix round found: a
    /// mutation to `collect_update_node`'s referent-type check, or to the
    /// collect-path `for-each`, flipped none of this module's for-each/
    /// update-node tests, because every one of them drove
    /// `execute_effects` — production's ABANDONED path since Task 12 —
    /// through this fixture. The two callers left on it
    /// (`update_node_against_a_selection_result_writes_the_selected_node`,
    /// `for_each_query_does_not_see_an_earlier_verbs_effect_in_the_same_list`)
    /// are pinned to `execute_effects` ON PURPOSE — the latter needs
    /// `add-node`, which the collect path refuses by design (§4.2 chapter
    /// C4's scope note) — and stay here; every for-each/update-node test
    /// meaning to prove something about `run_tick`'s actual guarantees now
    /// uses `collect_then_apply` instead.
    fn pre_state_and_live(build: impl Fn(&mut MemoryGraph)) -> (MemoryGraph, MemoryGraph) {
        let mut pre_state = MemoryGraph::new();
        build(&mut pre_state);
        let mut live = MemoryGraph::new();
        build(&mut live);
        (pre_state, live)
    }

    /// Run one `(effects …)` list through the PRODUCTION path (Task 12):
    /// collect against an immutable borrow of `graph`, then — after that
    /// borrow ends — apply every collected write against a mutable one.
    /// This is EXACTLY the two passes `tick.rs::run_tick` runs, on ONE
    /// shared graph object, which is what lets a test built on it catch a
    /// bug in `collect_update_node` or the collect-path `for-each` that
    /// [`Self::pre_state_and_live`]'s two-object split structurally cannot
    /// (see that fixture's own doc for why).
    // Same precedent as `Fixture::run`'s own `#[allow]` above: the event
    // stream's shape is spelled out once in this doc rather than named
    // through a second type alias.
    #[allow(clippy::type_complexity)]
    fn collect_then_apply(
        graph: &mut MemoryGraph,
        types: &TypeEnv,
        enums: &EnumRegistry,
        bindings: HashMap<String, Value>,
        effects_source: &str,
        fuel: &mut u64,
    ) -> Result<Vec<(String, Vec<(String, Value)>)>, EvalError> {
        let (form, _) = read(effects_source).expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut sink = CollectingSink::default();
        let pending = {
            let env = EvalEnv {
                bindings,
                intrinsic_costs: &IntrinsicCosts::default(),
                graph: Some(&*graph as &dyn GraphSubstrate),
                // D102 discharge (Task 1, P27 territory-port train): thread
                // the SAME registries `EffectExecutor::new` below already
                // takes, so `field-of` over an enum-declared field renders
                // `Value::Enum` here exactly as the real `run_tick` path
                // does — a helper that silently fell back to `Value::Real`
                // would mask the exact bug D102's discharge exists to fix.
                types: Some(types),
                enums: Some(enums),
                elements: Vec::new(),
                draw_context: None,
            };
            let mut collector = EffectExecutor::new(types, enums, None);
            collector.collect_effects(&items[1..], &env, &EmptyIntrinsicHost, &mut sink, fuel)?
        };
        let mut applier = EffectExecutor::new(types, enums, None);
        for write in &pending {
            applier.apply_pending_write(write, &mut *graph)?;
        }
        Ok(sink.events)
    }

    /// `collect_then_apply`'s COLLECT-ONLY half — stops before
    /// `apply_pending_write` ever runs, never merely stopping at the first
    /// error `collect_then_apply` would ALSO have surfaced. This is the one
    /// way to isolate a collect-site guard from an apply-site one when both
    /// exist as independent defense-in-depth checks (#528 fix round,
    /// blocker 2's mutation-testing finding): `collect_then_apply` cannot
    /// tell the caller which of the two sites actually produced a given
    /// error, so a test built on it that MEANS to pin the collect site
    /// stays green even with that site's own guard deleted, masked by the
    /// apply site's identical backstop.
    fn collect_only(
        graph: &MemoryGraph,
        types: &TypeEnv,
        enums: &EnumRegistry,
        bindings: HashMap<String, Value>,
        effects_source: &str,
        fuel: &mut u64,
    ) -> Result<Vec<PendingWrite>, EvalError> {
        let (form, _) = read(effects_source).expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut sink = CollectingSink::default();
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: Some(graph as &dyn GraphSubstrate),
            // D102 discharge (Task 1, P27 territory-port train): same
            // reasoning as `collect_then_apply`'s own env, above.
            types: Some(types),
            enums: Some(enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let mut collector = EffectExecutor::new(types, enums, None);
        collector.collect_effects(&items[1..], &env, &EmptyIntrinsicHost, &mut sink, fuel)
    }

    #[test]
    fn for_each_over_an_empty_query_applies_nothing_and_does_not_error() {
        // No ORGANIZATION node exists, so (nodes NodeType/ORGANIZATION)
        // materializes empty. §2.8 chapter C6: an iteration is a COMMAND,
        // and "do it to none" is fully determined — this is the one place
        // an empty set is quiet (unlike mean/min/max/select-*, which must
        // PRODUCE a value and have none to produce, E-EVAL-021). Driven
        // through `collect_then_apply` (#519 fix round) — the PRODUCTION
        // path (`tick.rs::run_tick`'s two passes), not the abandoned
        // `execute_effects` single pass.
        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(self_id, "social-class/agitation", 0.10)
            .unwrap();
        let types = types();
        let mut fuel = 64;
        let events = collect_then_apply(
            &mut graph,
            &types,
            &enums(),
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

    #[test]
    fn for_each_applies_the_body_once_per_element_in_iteration_order() {
        // Driven through `collect_then_apply` (#519 fix round) — the
        // PRODUCTION path, on one shared graph.
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let c = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = types();
        let mut fuel = 256;
        let events = collect_then_apply(
            &mut graph,
            &types,
            &enums(),
            HashMap::new(),
            "(effects (for-each (nodes NodeType/SOCIAL_CLASS) \
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
                (
                    "RUPTURE".to_owned(),
                    vec![("who".to_owned(), Value::NodeRef(c))]
                ),
            ],
            "once per element, in §2.6 ascending-id iteration order"
        );
    }

    /// The §6.2 family-15 pre-state vector. §2.8 chapter C6, quoted in the
    /// module: "every expression anywhere in an effects list ... is
    /// evaluated against the pre-state". An EARLIER verb in the SAME
    /// effects list (`add-node`) mutates the LIVE graph only — never
    /// `env.graph`, which no verb write path touches — so `for-each`'s
    /// query, materialized through `env.graph`, must not see it. If it did,
    /// TWO `RUPTURE` events would fire instead of one.
    #[test]
    fn for_each_query_does_not_see_an_earlier_verbs_effect_in_the_same_list() {
        let (pre_state, mut live) = pre_state_and_live(|g| {
            g.add_node("SOCIAL_CLASS").unwrap();
        });
        let self_id = NodeId(0);
        let types = types();
        let enums = enums();
        // PR A verifier fix round (2026-08-12): `types`/`enums` were
        // already built, right here, for `EffectExecutor` below — leaving
        // the sibling `EvalEnv` on `None`/`None` was the exact
        // coincidental-safety shape the fix round closed (harmless only
        // because this rule text never happens to use `field-of` over an
        // enum field; `field_of_node` now refuses loudly rather than
        // trust that).
        let env = EvalEnv {
            bindings: HashMap::from([("self".to_owned(), Value::NodeRef(self_id))]),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: Some(&pre_state as &dyn GraphSubstrate),
            types: Some(&types),
            enums: Some(&enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let mut sink = CollectingSink::default();
        let mut fuel = 256;
        let (form, _) = read(
            "(effects \
               (add-node NodeType/SOCIAL_CLASS extra) \
               (for-each (nodes NodeType/SOCIAL_CLASS) \
                 (emit EventType/RUPTURE (n 1))))",
        )
        .expect("must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut live,
                &mut sink,
                &mut fuel,
            )
            .unwrap();
        assert_eq!(
            sink.events.len(),
            1,
            "for-each's query must read the rule's PRE-state (one \
             SOCIAL_CLASS node), never a live mutation an earlier verb in \
             this same effects list already applied (§2.8 chapter C6): {:?}",
            sink.events
        );
    }

    #[test]
    fn nested_for_each_composes_outer_iteration_then_inner_source_order() {
        // Driven through `collect_then_apply` (#519 fix round) — the
        // PRODUCTION path, on one shared graph.
        let mut graph = MemoryGraph::new();
        let sc1 = graph.add_node("SOCIAL_CLASS").unwrap();
        let sc2 = graph.add_node("SOCIAL_CLASS").unwrap();
        let org1 = graph.add_node("ORGANIZATION").unwrap();
        let org2 = graph.add_node("ORGANIZATION").unwrap();
        let types = types();
        let mut fuel = 8192;
        let events = collect_then_apply(
            &mut graph,
            &types,
            &enums(),
            HashMap::new(),
            "(effects \
               (for-each (nodes NodeType/SOCIAL_CLASS) :as outer-elem \
                 (emit EventType/OUTER_MARKER (who outer-elem)) \
                 (for-each (nodes NodeType/ORGANIZATION) \
                   (emit EventType/PAIR (outer outer-elem) (inner it)))))",
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            events,
            vec![
                (
                    "OUTER_MARKER".to_owned(),
                    vec![("who".to_owned(), Value::NodeRef(sc1))]
                ),
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
                    "OUTER_MARKER".to_owned(),
                    vec![("who".to_owned(), Value::NodeRef(sc2))]
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

    /// Defense in depth (#519 fix round, fix 3): `collect_effects` is
    /// called directly here, bypassing `load_rule_form`'s LOAD-time gate
    /// entirely (the fix round's own fix 4) — the collect path must refuse
    /// a shape verb on its own, loudly, naming it, never silently doing
    /// nothing. Before this test existed `collect_effects` had exactly one
    /// caller in this whole crate (`collect_then_apply`, itself reachable
    /// only from this module's own tests), and nothing exercised this arm
    /// — proven by the verifier's mutation M5 (turn the six-verb refusal
    /// into a silent `Ok(())`), which flipped zero tests in the crate.
    #[test]
    fn the_collect_path_refuses_a_shape_verb_naming_it() {
        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = types();
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums(),
            HashMap::from([("self".to_owned(), Value::NodeRef(self_id))]),
            "(effects (add-node NodeType/SOCIAL_CLASS recruit))",
            &mut fuel,
        )
        .unwrap_err();
        assert!(err.message.contains("add-node"), "{err}");
    }

    /// A `guard` whose condition evaluates to a non-`Bool` refuses loudly
    /// on the COLLECT path — never a silent not-taken. Before the #519
    /// Copilot harvest both guard arms used
    /// `matches!(…, Value::Bool(true))`, which read `(guard 3 …)` as
    /// `false` and skipped the body without a word — masking exactly the
    /// type bugs §2.8's Bool contract exists to surface. Restoring that
    /// `matches!` form flips this test (mutation-checked).
    #[test]
    fn a_non_bool_guard_condition_is_a_loud_error_not_a_silent_skip() {
        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = types();
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums(),
            HashMap::from([("self".to_owned(), Value::NodeRef(self_id))]),
            "(effects (guard 3 (emit event/rupture)))",
            &mut fuel,
        )
        .unwrap_err();
        assert!(err.message.contains("expected Bool"), "{err}");
    }

    // ---- Task 11: update-node against a computed NodeRef ----

    /// The `TypeEnv` for Task 11's own tests: an `ORGANIZATION` field, so
    /// the §2.7 worked example's shape (`update-node` against a
    /// `select-max` result) is exercisable independent of `Fixture`'s
    /// `SOCIAL_CLASS`-only registry.
    fn organization_types() -> TypeEnv {
        TypeEnv {
            fields: HashMap::from([(
                "organization/claim-strength".to_owned(),
                FieldDecl {
                    ty: BslType::Intensity,
                    kind: FieldKind::Intensive,
                },
            )]),
            exemptions: &[],
        }
    }

    /// The §2.7 worked example's SHAPE: `(update-node (select-max …) …)`.
    /// The reference's own literal example writes `#t` to a `Bool` field
    /// (`organization/holds-office`) — `numeric_write_value` (this module)
    /// has no `Bool`-typed store path today (`GraphSubstrate::update_node`
    /// stores `f64` only; boolean field storage is a separate, pre-existing
    /// gap this task does not own), so this proves the SAME property —
    /// a computed `NodeRef` reaches `update-node`'s write path and writes
    /// the SELECTED element, never the wrong one — against a numeric field
    /// instead.
    #[test]
    fn update_node_against_a_selection_result_writes_the_selected_node() {
        let (pre_state, mut live) = pre_state_and_live(|g| {
            let low = g.add_node("ORGANIZATION").unwrap();
            let high = g.add_node("ORGANIZATION").unwrap();
            g.update_node(low, "organization/claim-strength", 0.2)
                .unwrap();
            g.update_node(high, "organization/claim-strength", 0.9)
                .unwrap();
        });
        let [low, high] = [NodeId(0), NodeId(1)];
        let types = organization_types();
        let enums = enums();
        // PR A verifier fix round (2026-08-12): same coincidental-safety
        // shape as `for_each_query_does_not_see_an_earlier_verbs_effect_
        // in_the_same_list` above — `types`/`enums` were already built for
        // `EffectExecutor` below; the sibling `EvalEnv` now carries them
        // too. This test's rule text DOES use `field-of` (over
        // `organization/claim-strength`, a Coefficient field) inside its
        // `select-max` score, so this was reachable, not merely defensive.
        let env = EvalEnv {
            bindings: HashMap::new(),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: Some(&pre_state as &dyn GraphSubstrate),
            types: Some(&types),
            enums: Some(&enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let mut sink = CollectingSink::default();
        let mut fuel = 256;
        let (form, _) = read(
            "(effects (update-node \
               (select-max (nodes NodeType/ORGANIZATION) \
                            (field-of it organization/claim-strength)) \
               organization/claim-strength (set 0.5i)))",
        )
        .expect("must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut live,
                &mut sink,
                &mut fuel,
            )
            .unwrap();
        let selected = live
            .node_attribute(high, "organization/claim-strength")
            .unwrap();
        assert!(
            (selected - 0.5).abs() < 1e-12,
            "the SELECTED (higher-scoring) node was written"
        );
        let untouched = live
            .node_attribute(low, "organization/claim-strength")
            .unwrap();
        assert!(
            (untouched - 0.2).abs() < 1e-12,
            "the non-selected node was left alone"
        );
    }

    /// §2.10 discipline 1's runtime half (R9 chapter C2): a reference has no
    /// static type (§3.1), so the field-owner-vs-referent disagreement that
    /// `add-node`/`add-edge`/`add-hyperedge` catch at LOAD as `E-TYPE-014`
    /// can only be caught HERE, at evaluation, as `E-EVAL-033`. Before this
    /// task `update_node` ran only `store_range_check` on the field — this
    /// write SUCCEEDED SILENTLY.
    #[test]
    fn update_node_whose_referent_is_of_another_type_is_e_eval_033() {
        // Driven through `collect_then_apply` (#519 fix round) — the
        // PRODUCTION path (`collect_update_node`, not `update_node`'s
        // execute-path copy).
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = TypeEnv {
            fields: HashMap::from([(
                "territory/population".to_owned(),
                FieldDecl {
                    ty: BslType::Int,
                    kind: FieldKind::Extensive,
                },
            )]),
            exemptions: &[],
        };
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums(),
            HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            "(effects (update-node self territory/population (set 5)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::AccessorTypeOrValueMismatch));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-033");
        assert!(
            graph
                .node_attribute(subject, "territory/population")
                .is_err(),
            "the refused write must not have landed"
        );
    }

    #[test]
    fn a_set_on_a_never_written_field_still_succeeds_under_observation() {
        // The determinism trap this design exists to avoid: probing the
        // prior value must not propagate `node_attribute`'s honest-null
        // error, or an observed run would fail where an unobserved one
        // succeeds. Both paths are asserted green here.
        for observe in [false, true] {
            let mut fixture = Fixture::new();
            let mut fuel = 256;
            let source = "(effects (update-node self social-class/head-count (set 100)))";
            let result = if observe {
                fixture.run_observed(source, &mut fuel).0
            } else {
                fixture.run(source, &mut fuel).map(|_| ())
            };
            assert!(
                result.is_ok(),
                "observe={observe} must not change the verdict"
            );
        }
    }

    // ==================================================== §2.13 (D101):
    // the runtime enum write path — `update-node` on an `enum`-typed
    // field. Driven through `collect_then_apply` (the SAME two-call
    // sequence — `collect_effects` then `apply_pending_write` — that
    // `tick.rs::run_tick` runs in production; see that helper's own doc),
    // never `execute_effects` (test-harness-only since Task 12).

    /// `OrgKind` in declaration order: `STATE_APPARATUS`=0, `BUSINESS`=1,
    /// `POLITICAL_FACTION`=2, `CIVIL_SOCIETY`=3 — matching the spec's own
    /// worked example (Organization spec §1 Q1/Q15). One `organization/kind`
    /// field declared `BslType::Enum`, `FieldKind::NotApplicable`.
    fn org_kind_types_and_enums() -> (TypeEnv, EnumRegistry) {
        let mut enums = EnumRegistry::default();
        let ty = enums
            .declare(
                "OrgKind",
                &[
                    "STATE_APPARATUS".to_owned(),
                    "BUSINESS".to_owned(),
                    "POLITICAL_FACTION".to_owned(),
                    "CIVIL_SOCIETY".to_owned(),
                ],
            )
            .unwrap();
        let types = TypeEnv {
            fields: HashMap::from([(
                "organization/kind".to_owned(),
                FieldDecl {
                    ty: BslType::Enum(ty),
                    kind: FieldKind::NotApplicable,
                },
            )]),
            exemptions: &[],
        };
        (types, enums)
    }

    #[test]
    fn update_node_writes_an_enum_ref_and_stores_the_declared_ordinal() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 64;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (set OrgKind/POLITICAL_FACTION)))",
            &mut fuel,
        )
        .expect("a matching enum-ref write must succeed");
        let stored = graph.node_attribute(id, "organization/kind").unwrap();
        assert!(
            (stored - 2.0).abs() < 1e-12,
            "POLITICAL_FACTION is declaration-order ordinal 2, stored: {stored}"
        );
    }

    #[test]
    fn a_bare_real_write_into_an_enum_field_is_e_eval_042() {
        // The eval-time form of the load-time bare-number mistake
        // (`scenario.rs::attribute_value_enum`, E-LOAD-056) — the ordinal
        // is never a surface value, checked again HERE because content
        // cannot be checked once and for all at load (§3.7/§4.3's
        // two-site pattern).
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (set 2)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        assert_eq!(err.code.map(EvalCode::spec_code), Some("E-EVAL-042"));
    }

    #[test]
    fn a_cross_enum_type_write_is_e_eval_042() {
        // NodeType/BUSINESS is a perfectly legal <enum-ref> — of the WRONG
        // declared type for this field, but with a MEMBER NAME that
        // collides with a real OrgKind member (BUSINESS, ordinal 1) —
        // deliberately, so a mutant that skipped the type comparison and
        // fell through to the member lookup would find "BUSINESS" and
        // WRONGLY succeed with ordinal 1, rather than failing for a
        // different, equally-plausible-looking reason (an earlier version
        // of this test used SOCIAL_CLASS, whose absence from OrgKind's
        // member list made the member-lookup branch ALSO refuse — masking
        // whether the type check itself ran; caught by mutation testing).
        // No load-time gate catches this either way (update-node's write
        // operand is not one of `grammar.rs`'s typed ENUM_REF_POSITIONS),
        // so this proves the runtime check is load-bearing on its own.
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (set NodeType/BUSINESS)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        assert!(
            err.message.contains("declared enum type is OrgKind"),
            "{}",
            err.message
        );
        assert!(err.message.contains("NodeType/BUSINESS"), "{}", err.message);
    }

    #[test]
    fn an_undeclared_member_of_the_right_type_is_e_eval_042() {
        // OrgKind/NOWHERE lexes and type-checks fine (the reader never
        // validates enum-ref MEMBERSHIP, only shape) — the registry lookup
        // at write time is what catches it.
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (set OrgKind/NOWHERE)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        assert!(err.message.contains("NOWHERE"), "{}", err.message);
    }

    // ==================================================== #528 fix round,
    // blocker 2: `add`/`sub`/`scale` are refused on a `BslType::Enum`
    // field (E-EVAL-042) — `Enum<T>` supports no arithmetic (§2.13's own
    // "no aggregation kind" law). Driven through `collect_then_apply`
    // exactly like the D101 tests above — the collect-path guard (inside
    // `collect_update_node`) refuses before `apply_pending_write` ever
    // runs, so these ALSO exercise the collect site the verifier named
    // invisible to every pre-#528 unit test.

    /// Isolated to the COLLECT site via `collect_only` (never
    /// `collect_then_apply`): `apply_pending_write` guards the SAME class
    /// independently (defense in depth), so a test built on
    /// `collect_then_apply` cannot tell collect's own guard apart from
    /// apply's — proven by mutation testing this fix round's own review
    /// caught (deleting `collect_update_node`'s guard alone left a
    /// `collect_then_apply`-based version of this test green, masked by
    /// apply's identical backstop).
    #[test]
    fn add_on_an_enum_field_is_e_eval_042_at_the_collect_site() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        graph.update_node(id, "organization/kind", 0.0).unwrap(); // STATE_APPARATUS
        let mut fuel = 64;
        let err = collect_only(
            &graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (add OrgKind/BUSINESS)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        assert_eq!(err.code.map(EvalCode::spec_code), Some("E-EVAL-042"));
        // No PendingWrite was ever built, let alone applied — the graph is
        // observably untouched (never re-read `organization/kind` here on
        // purpose: this test's whole point is that apply NEVER RAN).
        let stored = graph.node_attribute(id, "organization/kind").unwrap();
        assert!(
            (stored - 0.0).abs() < 1e-12,
            "the refused write must not have landed: {stored}"
        );
    }

    /// [`Self::update_node`]'s IMMEDIATE execute path — `execute_effects`,
    /// production's abandoned path since Task 12 but still this crate's own
    /// unit-test harness (see that method's doc) — guards independently of
    /// the collect/apply pair above.
    #[test]
    fn add_on_an_enum_field_is_e_eval_042_at_the_execute_site() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        graph.update_node(id, "organization/kind", 0.0).unwrap(); // STATE_APPARATUS
        let mut fuel = 64;
        let (form, _) =
            read("(effects (update-node self organization/kind (add OrgKind/BUSINESS)))")
                .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let env = EvalEnv {
            bindings: HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let mut sink = CollectingSink::default();
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let err = executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        let stored = graph.node_attribute(id, "organization/kind").unwrap();
        assert!(
            (stored - 0.0).abs() < 1e-12,
            "the refused write must not have landed: {stored}"
        );
    }

    #[test]
    fn sub_on_an_enum_field_is_e_eval_042_before_it_corrupts_the_store() {
        // The worst case: STATE_APPARATUS (ordinal 0) minus BUSINESS's
        // ordinal (1) would write -1.0 — no range check bounds an enum
        // field's ordinal at the store boundary (§3.3's unit-interval
        // check does not cover BslType::Enum), so this is the "corrupts
        // the store into a next-tick read error at the wrong site" case
        // the fix round names.
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        graph.update_node(id, "organization/kind", 0.0).unwrap(); // STATE_APPARATUS
        let mut fuel = 64;
        let err = collect_only(
            &graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (sub OrgKind/BUSINESS)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        let stored = graph.node_attribute(id, "organization/kind").unwrap();
        assert!(
            (stored - 0.0).abs() < 1e-12,
            "the refused write must never reach -1.0 (no such member): {stored}"
        );
    }

    #[test]
    fn scale_on_an_enum_field_is_e_eval_042_at_the_collect_site() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        graph.update_node(id, "organization/kind", 0.0).unwrap();
        let mut fuel = 64;
        let err = collect_only(
            &graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (scale OrgKind/BUSINESS)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
    }

    /// `apply_pending_write` guards independently of `collect_update_node`
    /// (defense in depth, matching `numeric_write_value`'s own "the one
    /// funnel every write value crosses" discipline) — proven by building a
    /// `PendingWrite` DIRECTLY, bypassing collect entirely, the only way to
    /// isolate the apply-site guard from the collect-site one.
    #[test]
    fn apply_pending_write_refuses_arithmetic_on_an_enum_field_even_bypassing_collect() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        graph.update_node(id, "organization/kind", 0.0).unwrap();
        let write = PendingWrite {
            target: WriteTarget::Node(id),
            field: "organization/kind".to_owned(),
            op: UpdateOp::Add,
            operand: WriteOperand::Real(1.0),
        };
        let mut applier = EffectExecutor::new(&types, &enums, None);
        let err = applier.apply_pending_write(&write, &mut graph).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        let stored = graph.node_attribute(id, "organization/kind").unwrap();
        assert!((stored - 0.0).abs() < 1e-12);
    }

    /// `set` remains the coherent op on an enum field — the guard is scoped
    /// to `add`/`sub`/`scale` only, never widened to refuse the legitimate
    /// write path.
    #[test]
    fn set_on_an_enum_field_is_unaffected_by_the_arithmetic_guard() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 64;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/kind (set OrgKind/BUSINESS)))",
            &mut fuel,
        )
        .expect("set is the coherent op on an enum field");
    }

    #[test]
    fn an_enum_ref_write_into_a_non_enum_field_refuses_via_the_original_catchall() {
        // The EXISTING catch-all in `numeric_write_value` — unchanged
        // message — still fires for a non-enum field, proving the new
        // branch is scoped to enum-declared fields only (`store field`
        // has no enum in `types` here at all).
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = types();
        let enums = enums();
        let mut fuel = 64;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self social-class/head-count (set OrgKind/BUSINESS)))",
            &mut fuel,
        )
        .unwrap_err();
        assert!(
            err.message.contains("cannot store"),
            "the ORIGINAL non-enum catch-all message must be unchanged: {}",
            err.message
        );
    }

    /// D102 discharge (Task 1, P27 territory-port train): `field-of` over
    /// an enum-declared field now typechecks and evaluates to a real
    /// `Value::Enum` (previously refused at load) — this proves the
    /// relaxation does NOT open an arithmetic hole. The catch-all proven
    /// above for a LITERAL enum-ref operand fires identically when the
    /// `Value::Enum` is sourced through `field-of` instead — same funnel,
    /// same refusal, whichever expression produced the value.
    #[test]
    fn field_of_over_an_enum_field_still_refuses_as_an_add_operand() {
        let mut graph = MemoryGraph::new();
        let id = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(id, "organization/kind", 0.0).unwrap(); // STATE_APPARATUS
        let (mut types, enums) = org_kind_types_and_enums();
        types.fields.insert(
            "organization/budget".to_owned(),
            FieldDecl {
                ty: BslType::Int,
                kind: FieldKind::Extensive,
            },
        );
        graph.update_node(id, "organization/budget", 5.0).unwrap();
        let mut fuel = 64;
        let err = collect_only(
            &graph,
            &types,
            &enums,
            HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            "(effects (update-node self organization/budget \
                        (add (field-of self organization/kind))))",
            &mut fuel,
        )
        .unwrap_err();
        assert!(
            err.message.contains("cannot store"),
            "the ORIGINAL non-enum catch-all message must fire for a \
             field-of-sourced enum value too: {}",
            err.message
        );
    }

    /// PR A verifier fix round (2026-08-12): the read/write ROUND TRIP
    /// `field-of` + `update-node (set …)` makes possible for an
    /// enum-declared field — `(update-node <n> <enum-field> (set
    /// (field-of <m> <enum-field>)))`, the exact shape Territory's
    /// `_find_sink_node` sink-priority write needs (a neighbor's
    /// `territory_type` copied onto the acting node) — existed since D102's
    /// discharge (Task 1) and `enum_write_value`'s own pre-existing type
    /// check, but had no dedicated test. Both halves in one vector: a
    /// same-type copy stores the SOURCE's ordinal (not a re-derivation —
    /// the exact ordinal, proving the round trip is lossless); a
    /// cross-type copy refuses `E-EVAL-042`, exactly as a literal
    /// cross-type enum-ref write already does
    /// (`a_cross_enum_type_write_is_e_eval_042` above).
    #[test]
    fn field_of_copied_into_an_update_node_set_round_trips_same_type_and_refuses_cross_type() {
        // ---- same-type half ----
        let mut graph = MemoryGraph::new();
        let source = graph.add_node("ORGANIZATION").unwrap();
        let target = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(source, "organization/kind", 2.0).unwrap(); // POLITICAL_FACTION, declaration-order ordinal 2
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 64;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([
                ("self".to_owned(), Value::NodeRef(target)),
                ("source".to_owned(), Value::NodeRef(source)),
            ]),
            "(effects (update-node self organization/kind \
                        (set (field-of source organization/kind))))",
            &mut fuel,
        )
        .expect("a same-type field-of copy into an enum field must succeed");
        let stored = graph.node_attribute(target, "organization/kind").unwrap();
        assert!(
            (stored - 2.0).abs() < 1e-12,
            "the copy must land the SOURCE's ordinal, POLITICAL_FACTION=2: {stored}"
        );

        // ---- cross-type half: a second enum-declared field of a
        // DIFFERENT declared type on the SAME node type ----
        let mut cross_graph = MemoryGraph::new();
        let cross_source = cross_graph.add_node("ORGANIZATION").unwrap();
        let cross_target = cross_graph.add_node("ORGANIZATION").unwrap();
        cross_graph
            .update_node(cross_source, "organization/kind", 0.0)
            .unwrap(); // STATE_APPARATUS
        let (mut cross_types, mut cross_enums) = org_kind_types_and_enums();
        let rank_ty = cross_enums
            .declare("PartyRank", &["CADRE".to_owned(), "SYMPATHIZER".to_owned()])
            .unwrap();
        cross_types.fields.insert(
            "organization/rank".to_owned(),
            FieldDecl {
                ty: BslType::Enum(rank_ty),
                kind: FieldKind::NotApplicable,
            },
        );
        let mut fuel2 = 64;
        let err = collect_only(
            &cross_graph,
            &cross_types,
            &cross_enums,
            HashMap::from([
                ("self".to_owned(), Value::NodeRef(cross_target)),
                ("source".to_owned(), Value::NodeRef(cross_source)),
            ]),
            "(effects (update-node self organization/rank \
                        (set (field-of source organization/kind))))",
            &mut fuel2,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation));
        assert!(
            err.message.contains("declared enum type is PartyRank"),
            "{}",
            err.message
        );
        assert!(err.message.contains("OrgKind"), "{}", err.message);
    }

    // ---- F5(b) (#534 fix round item 5): `find_deferred_shape_verb`
    // head-position discipline — the same label-as-head misreading R2
    // (grammar.rs's `check_type_operands_are_enum_refs`) already fixed. ----

    #[test]
    fn a_payload_item_labeled_like_a_deferred_shape_verb_is_never_over_refused() {
        // `emit`'s `<payload-item>` label is an unconstrained `Atom::Symbol`
        // (§2.8's `<payload-item> ::= (<symbol> <expr>)`) — nothing stops
        // content from naming one `add-node`, and that label is not a
        // nested verb invocation. The buggy walk treated every child
        // list's head as a fresh candidate and wrongly refused this exact
        // form.
        let (probe, _) =
            read("(emit EventType/RUPTURE (add-node 5) (severity 1))").expect("probe must parse");
        assert!(
            super::check_no_deferred_shape_verbs(&probe).is_ok(),
            "a payload item's own LABEL must never be mistaken for a nested verb invocation"
        );
        // The same shape with a different label was always Ok — proves the
        // probe isolates the label collision, not some other cause.
        let (control, _) =
            read("(emit EventType/RUPTURE (foo 5) (severity 1))").expect("control must parse");
        assert!(super::check_no_deferred_shape_verbs(&control).is_ok());
    }

    #[test]
    fn a_genuine_verb_nested_inside_guard_or_for_each_is_still_caught() {
        // The regression pair for the probe above: head-position-only
        // stopping at `emit` must not blind the walk to a GENUINE nested
        // deferred-shape verb inside `guard`/`for-each` effect bodies —
        // neither head is `emit` nor a `DEFERRED_SHAPE_VERBS` member, so
        // the walk must still fall through to the generic recursion and
        // find the real verb.
        for source in [
            "(guard #t (remove-node self))",
            "(for-each (edges EdgeType/SOLIDARITY) (remove-edge EdgeType/SOLIDARITY a b))",
        ] {
            let (probe, _) = read(source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
            let err = super::check_no_deferred_shape_verbs(&probe).expect_err(source);
            assert!(
                err.contains("remove-node") || err.contains("remove-edge"),
                "{source}: {err}"
            );
        }
    }

    // ---- H1 (#534 fix round 3, residual of G1 — one root cause with
    // `grammar::check_enum_ref_membership`'s/`check_type_operands_are_
    // enum_refs`'s own sibling fixes): the G1 descent assumed `items[1]`
    // is `emit`'s type operand (`skip(2)`) and that a payload item has
    // exactly 2 elements (`pair.get(1)`). A SECOND malformation — a
    // type-operand-less nested `emit`, or an over-arity payload item —
    // breaks both assumptions at once and hides a genuine deferred-shape
    // verb from this walk. ----

    #[test]
    fn a_deferred_verb_behind_a_type_operand_less_nested_emit_still_refuses() {
        // `(m (emit (add-node NodeType/SOCIAL_CLASS 5)))` is a payload
        // item whose value is a NESTED `emit` MISSING its own type
        // operand — under the OLD `skip(2)`, that nested emit's own
        // `items[1]` (`(add-node NodeType/SOCIAL_CLASS 5)`, the verb
        // invocation itself) was silently treated as "the confirmed type
        // operand" and skipped, so `add-node` was never inspected as a
        // head at all.
        let (probe, _) =
            read("(emit EventType/RUPTURE (m (emit (add-node NodeType/SOCIAL_CLASS 5))))")
                .expect("probe must parse");
        let err = super::check_no_deferred_shape_verbs(&probe)
            .expect_err("add-node behind a malformed nested emit must still be caught");
        assert!(err.contains("add-node"), "{err}");
    }

    #[test]
    fn remove_node_behind_a_type_operand_less_nested_emit_still_refuses() {
        // The sibling probe with a REMOVAL verb (`remove-node`), not a
        // minting one — proves the fix is not scoped to `add-node` alone.
        let (probe, _) = read("(emit EventType/RUPTURE (m (emit (remove-node self))))")
            .expect("probe must parse");
        let err = super::check_no_deferred_shape_verbs(&probe)
            .expect_err("remove-node behind a malformed nested emit must still be caught");
        assert!(err.contains("remove-node"), "{err}");
    }

    #[test]
    fn a_deferred_verb_in_an_over_arity_payload_item_still_refuses() {
        // `(m 1 (add-node NodeType/SOCIAL_CLASS 5))` is an over-arity
        // payload item — THREE elements, not the well-formed
        // `(<symbol> <expr>)` two — so `pair.get(1)` alone (the literal
        // `1`) never reached `pair[2]`, the real `add-node` invocation.
        // The fix descends `pair[1..]`, every element after the label.
        let (probe, _) = read("(emit EventType/RUPTURE (m 1 (add-node NodeType/SOCIAL_CLASS 5)))")
            .expect("probe must parse");
        let err = super::check_no_deferred_shape_verbs(&probe)
            .expect_err("every value after an over-arity payload item's label must be checked");
        assert!(err.contains("add-node"), "{err}");
    }

    // ---------------- T3 PR B (issue #560, ADR198 R3): update-edge parity ----------------

    use babylon_graph::state_hash::CanonicalState;

    /// The edge-field `TypeEnv`: the D32 implicit field declared explicitly
    /// (production seeds it through `prepare_rules`'s `FieldRegistry` wiring;
    /// a unit fixture declares it), plus one deffield-declared intensive
    /// edge field — the two field kinds the §6.2 chapter-C2 family names.
    fn edge_types() -> TypeEnv {
        TypeEnv {
            fields: HashMap::from([
                (
                    "solidarity/strength".to_owned(),
                    FieldDecl {
                        ty: BslType::Coefficient,
                        kind: FieldKind::Extensive,
                    },
                ),
                (
                    "solidarity/tension".to_owned(),
                    FieldDecl {
                        ty: BslType::Intensity,
                        kind: FieldKind::Intensive,
                    },
                ),
            ]),
            exemptions: &[],
        }
    }

    /// Two nodes joined by one SOLIDARITY edge (strength 0.5), plus the
    /// `EdgeRef` binding (`e`) a rule's `it`/`<expr>` operand would carry.
    fn edge_fixture() -> (MemoryGraph, NodeId, NodeId) {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", a, b, 0.5).unwrap();
        (graph, a, b)
    }

    fn edge_binding(a: NodeId, b: NodeId) -> HashMap<String, Value> {
        HashMap::from([(
            "e".to_owned(),
            Value::EdgeRef(EdgeKey {
                source: a,
                target: b,
                edge_type: "SOLIDARITY".to_owned(),
            }),
        )])
    }

    /// R3's core clause, executed: `set` on a deffield-declared edge field
    /// through the SAME collect-then-apply machinery update-node uses —
    /// the T3 storage read back through T2's read path.
    #[test]
    fn update_edge_set_writes_a_declared_edge_field_through_collect_then_apply() {
        let (mut graph, a, b) = edge_fixture();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 128;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/tension (set 0.7i)))",
            &mut fuel,
        )
        .expect("update-edge on a declared edge field must serve");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert!((stored - 0.7).abs() < 1e-12, "stored: {stored}");
    }

    /// The double-storage ruling (D143) at the VERB level: an update-edge
    /// against `<edge-type>/strength` moves the edge's 0x03-slot strength
    /// and mints NO fifth-section row.
    #[test]
    fn update_edge_against_strength_writes_the_slot_never_a_fifth_section_row() {
        let (mut graph, a, b) = edge_fixture();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 128;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/strength (scale 0.5c)))",
            &mut fuel,
        )
        .expect("the §2.10 worked shape must serve");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/strength")
            .unwrap();
        assert!((stored - 0.25).abs() < 1e-12, "0.5 scaled by 0.5: {stored}");
        assert!(
            graph.all_edge_attributes().is_empty(),
            "strength writes never mint a fifth-section row (D143)"
        );
    }

    /// D104's apply-time accumulation, edge half: two subjects each
    /// contributing `(add 0.1i)` to ONE edge's field must both land —
    /// collect reduces both operands against the PRE-state (0.2), apply
    /// reads the CURRENT value at apply time, so 0.2 + 0.1 + 0.1 = 0.4,
    /// never 0.3 (the lost-contribution bug the deferred machinery exists
    /// to prevent).
    #[test]
    fn update_edge_add_accumulates_at_apply_time_across_writes() {
        let (mut graph, a, b) = edge_fixture();
        graph
            .update_edge("SOLIDARITY", a, b, "solidarity/tension", 0.2)
            .unwrap();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 256;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/tension (add 0.1i)) \
                      (update-edge e solidarity/tension (add 0.1i)))",
            &mut fuel,
        )
        .expect("both contributions must land");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert!(
            (stored - 0.4).abs() < 1e-12,
            "apply-time accumulation (D104): 0.2 + 0.1 + 0.1, stored: {stored}"
        );
    }

    /// The batch's application order is load-bearing (the monoid action —
    /// `set` then `scale` is not `scale` then `set`): one effect list, two
    /// writes to one field, applied in collection order.
    #[test]
    fn update_edge_writes_apply_in_collection_order() {
        let (mut graph, a, b) = edge_fixture();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 256;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/tension (set 0.3i)) \
                      (update-edge e solidarity/tension (scale 0.5c)))",
            &mut fuel,
        )
        .expect("in-order application");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert!(
            (stored - 0.15).abs() < 1e-12,
            "set-then-scale, in order: 0.3 * 0.5, stored: {stored}"
        );
    }

    /// §2.8's existence discipline through the new path: a write against a
    /// triple no edge occupies is E-EVAL-031, never a mint.
    #[test]
    fn update_edge_on_a_missing_edge_is_e_eval_031() {
        let (mut graph, _a, _b) = edge_fixture();
        let c = graph.add_node("SOCIAL_CLASS").unwrap();
        let d = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 128;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(c, d),
            "(effects (update-edge e solidarity/tension (set 0.7i)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::ExistenceDiscipline), "{err}");
        assert!(graph.all_edge_attributes().is_empty());
    }

    /// §2.10 discipline 1 on the write side: a qname whose owner segment
    /// names a DIFFERENT edge type than the referent's is E-EVAL-033.
    #[test]
    fn update_edge_with_a_foreign_owner_qname_is_e_eval_033() {
        let (mut graph, a, b) = edge_fixture();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 128;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e tenancy/strength (set 0.7c)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(
            err.code,
            Some(EvalCode::AccessorTypeOrValueMismatch),
            "{err}"
        );
    }

    /// §3.3's store-boundary range law holds on edge fields exactly as on
    /// node fields: a store outside the declared [0,1] domain is
    /// E-EVAL-020, never a clamp.
    #[test]
    fn update_edge_leaving_the_declared_range_is_e_eval_020() {
        let (mut graph, a, b) = edge_fixture();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 128;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/tension (set 2)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::StoreRangeViolation), "{err}");
        assert!(
            graph
                .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
                .is_err(),
            "the refused write must not have landed"
        );
    }

    /// L-3 (#491 T3 review): `numeric_write_value`'s Currency refusal for a
    /// field that IS genuinely `currency`-declared, reached through the
    /// three paths with no typed-lane fork — `update-edge` here, and the
    /// two field-init tails below. The message must say the field has no
    /// route to the typed lane THROUGH THIS FORM, never "needs a
    /// currency-declared field" (which would be false — it already is one).
    #[test]
    fn update_edge_against_a_currency_declared_field_names_the_scope_gap_not_a_kind_mismatch() {
        let (mut graph, a, b) = edge_fixture();
        let types = TypeEnv {
            fields: HashMap::from([(
                "solidarity/subsidy".to_owned(),
                FieldDecl {
                    ty: BslType::Currency,
                    kind: FieldKind::Extensive,
                },
            )]),
            exemptions: &[],
        };
        let enums = enums();
        let mut fuel = 128;
        let err = collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/subsidy (set 5$)))",
            &mut fuel,
        )
        .unwrap_err();
        assert!(
            err.message.contains("update-node")
                && err.message.contains("set")
                && err.message.contains("update-edge"),
            "{err}"
        );
        assert!(
            !err.message.contains("needs a currency-declared field"),
            "the field IS declared currency — this framing would be false: {err}"
        );
    }

    /// The `add-node` field-init half of the same L-3 fix: a node-scoped
    /// `currency`-declared field STILL cannot be seeded via a field-init
    /// (only `update-node`'s runtime `set` reaches the typed lane) — a real,
    /// named scope gap, not a kind mismatch.
    #[test]
    fn add_node_field_init_against_a_currency_declared_field_names_the_scope_gap() {
        let mut graph = MemoryGraph::new();
        let types = TypeEnv {
            fields: HashMap::from([(
                "social-class/treasury".to_owned(),
                FieldDecl {
                    ty: BslType::Currency,
                    kind: FieldKind::Extensive,
                },
            )]),
            exemptions: &[],
        };
        let enums = enums();
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let env = EvalEnv {
            bindings: HashMap::new(),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (form, _) =
            read("(effects (add-node NodeType/SOCIAL_CLASS n (social-class/treasury 5$)))")
                .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut sink = CollectingSink::default();
        let mut fuel = 128;
        let err = executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .unwrap_err();
        assert!(err.message.contains("add-node"), "{err}");
        assert!(
            !err.message.contains("needs a currency-declared field"),
            "the field IS declared currency — this framing would be false: {err}"
        );
    }

    /// R3's "enum set included": an enum-typed edge field takes
    /// `<EnumType>/<MEMBER>` and stores the declaration-order ordinal —
    /// reusing `enum_write_value` unchanged, as its own doc anticipated.
    #[test]
    fn update_edge_set_on_an_enum_field_stores_the_declared_ordinal() {
        let (mut graph, a, b) = edge_fixture();
        let mut enums = EnumRegistry::default();
        let mode = enums
            .declare(
                "EdgeMode",
                &[
                    "DORMANT".to_owned(),
                    "ACTIVE".to_owned(),
                    "MILITANT".to_owned(),
                ],
            )
            .unwrap();
        let types = TypeEnv {
            fields: HashMap::from([
                (
                    "solidarity/strength".to_owned(),
                    FieldDecl {
                        ty: BslType::Coefficient,
                        kind: FieldKind::Extensive,
                    },
                ),
                (
                    "solidarity/mode".to_owned(),
                    FieldDecl {
                        ty: BslType::Enum(mode),
                        kind: FieldKind::NotApplicable,
                    },
                ),
            ]),
            exemptions: &[],
        };
        let mut fuel = 128;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/mode (set EdgeMode/MILITANT)))",
            &mut fuel,
        )
        .expect("enum set on an edge field must serve");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/mode")
            .unwrap();
        assert!(
            (stored - 2.0).abs() < 1e-12,
            "MILITANT is declaration-order ordinal 2, stored: {stored}"
        );
    }

    /// The enum-arithmetic guard rides the edge path too — and at the APPLY
    /// site independently (defense in depth), proven by building the
    /// `PendingWrite` directly, bypassing collect.
    #[test]
    fn update_edge_arithmetic_on_an_enum_field_is_e_eval_042_at_both_sites() {
        let (mut graph, a, b) = edge_fixture();
        graph
            .update_edge("SOLIDARITY", a, b, "solidarity/mode", 0.0)
            .unwrap(); // DORMANT
        let mut enums = EnumRegistry::default();
        let mode = enums
            .declare("EdgeMode", &["DORMANT".to_owned(), "ACTIVE".to_owned()])
            .unwrap();
        let types = TypeEnv {
            fields: HashMap::from([(
                "solidarity/mode".to_owned(),
                FieldDecl {
                    ty: BslType::Enum(mode),
                    kind: FieldKind::NotApplicable,
                },
            )]),
            exemptions: &[],
        };
        // Collect site.
        let mut fuel = 128;
        let err = collect_only(
            &graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/mode (add EdgeMode/ACTIVE)))",
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation), "{err}");
        // Apply site, bypassing collect (the direct-PendingWrite probe the
        // node side's own test established).
        let write = PendingWrite {
            target: WriteTarget::Edge(EdgeKey {
                source: a,
                target: b,
                edge_type: "SOLIDARITY".to_owned(),
            }),
            field: "solidarity/mode".to_owned(),
            op: UpdateOp::Add,
            operand: WriteOperand::Real(1.0),
        };
        let mut applier = EffectExecutor::new(&types, &enums, None);
        let err = applier.apply_pending_write(&write, &mut graph).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::EnumWriteShapeViolation), "{err}");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/mode")
            .unwrap();
        assert!(
            (stored - 0.0).abs() < 1e-12,
            "the refused write must not have landed: {stored}"
        );
    }

    /// The IMMEDIATE execute path (`execute_effects` — retired from
    /// production, still this crate's own harness) serves update-edge too:
    /// the same write, no collect/apply split.
    #[test]
    fn update_edge_serves_on_the_execute_path_immediately() {
        let (mut graph, a, b) = edge_fixture();
        let types = edge_types();
        let enums = enums();
        let mut sink = CollectingSink::default();
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let env = EvalEnv {
            bindings: edge_binding(a, b),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (form, _) = read("(effects (update-edge e solidarity/tension (set 0.4i)))")
            .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut fuel = 128;
        executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .expect("the execute path serves update-edge");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert!((stored - 0.4).abs() < 1e-12, "stored: {stored}");
    }

    /// The add-edge `<field-init>*` tail (D37) executes: mint-time field
    /// writes land through the same funnel (`numeric_write_value` +
    /// `store_range_check` + the write log), strength still minted ONLY by
    /// the `:strength` operand (E-PARSE-041 owns the static half).
    #[test]
    fn add_edge_field_inits_execute_and_are_logged() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = edge_types();
        let enums = enums();
        let mut log = CollectingWriteLog::new();
        let mut sink = CollectingSink::default();
        let mut executor =
            EffectExecutor::observed(&types, &enums, None, "test/add-edge-inits", &mut log);
        let env = EvalEnv {
            bindings: HashMap::from([
                ("self".to_owned(), Value::NodeRef(a)),
                ("other".to_owned(), Value::NodeRef(b)),
            ]),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (form, _) = read(
            "(effects (add-edge EdgeType/SOLIDARITY self other :strength 0.5c \
                (solidarity/tension 0.7i)))",
        )
        .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut fuel = 128;
        executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .expect("add-edge with field-inits must serve");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert!((stored - 0.7).abs() < 1e-12, "stored: {stored}");
        assert!(
            log.writes().iter().any(|w| matches!(
                w,
                Write::EdgeAttribute {
                    field,
                    value,
                    ..
                } if field == "solidarity/tension" && (*value - 0.7).abs() < 1e-12
            )),
            "one EdgeAttribute record per init, after the substrate accepted it"
        );
    }

    /// A COMMUNITY hyperedge over two classes, with its id returned for the
    /// `h` binding — the hyperedge half of [`edge_fixture`]'s pattern.
    fn hyperedge_fixture() -> (MemoryGraph, HyperedgeId) {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let cell = graph.add_hyperedge("COMMUNITY", &[a, b]).unwrap();
        (graph, cell)
    }

    fn hyperedge_types() -> TypeEnv {
        TypeEnv {
            fields: HashMap::from([(
                "community/heat".to_owned(),
                FieldDecl {
                    ty: BslType::Coefficient,
                    kind: FieldKind::Intensive,
                },
            )]),
            exemptions: &[],
        }
    }

    /// Task 6 (Community port train, E2b): `update-hyperedge` WRITES on
    /// BOTH dispatch sites — the refusal this test's predecessor pinned
    /// ("no substrate storage … III.7") was discharged by Task 5's
    /// `GraphSubstrate::update_hyperedge_attribute`, so the pin inverts:
    /// the execute path writes and records `Write::HyperedgeAttribute`,
    /// and the collect-then-apply path lands the combined value through
    /// the APPLY phase. (The M4 lesson: each site owes its own proof.)
    #[test]
    fn update_hyperedge_writes_a_declared_hyperedge_field_on_both_dispatch_sites() {
        // Site 1 — the execute path, observed so the write log is visible.
        let (mut graph, cell) = hyperedge_fixture();
        let types = hyperedge_types();
        let enums = enums();
        let mut sink = CollectingSink::default();
        let mut log = CollectingWriteLog::new();
        let env = EvalEnv {
            bindings: HashMap::from([("h".to_owned(), Value::HyperedgeRef(cell))]),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (form, _) = read("(effects (update-hyperedge h community/heat (set 0.5c)))")
            .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut fuel = 128;
        {
            let mut executor =
                EffectExecutor::observed(&types, &enums, None, "community/probe", &mut log);
            executor
                .execute_effects(
                    &items[1..],
                    &env,
                    &EmptyIntrinsicHost,
                    &mut graph,
                    &mut sink,
                    &mut fuel,
                )
                .expect("update-hyperedge must serve on the execute path");
        }
        let stored = graph.hyperedge_attribute(cell, "community/heat").unwrap();
        assert!((stored - 0.5).abs() < 1e-12, "set to 0.5, stored: {stored}");
        assert!(
            log.writes().iter().any(|w| matches!(
                w,
                Write::HyperedgeAttribute {
                    field,
                    previous: None,
                    value,
                    ..
                } if field == "community/heat" && (*value - 0.5).abs() < 1e-12
            )),
            "one HyperedgeAttribute record with no prior value, after the store accepted it"
        );

        // Site 2 — collect-then-apply: `add` reads the pre-state (0.5 from
        // site 1's write), combines, and lands 0.75 through the APPLY phase.
        let mut fuel = 128;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            HashMap::from([("h".to_owned(), Value::HyperedgeRef(cell))]),
            "(effects (update-hyperedge h community/heat (add 0.25c)))",
            &mut fuel,
        )
        .expect("update-hyperedge must serve on the collect path");
        let stored = graph.hyperedge_attribute(cell, "community/heat").unwrap();
        assert!(
            (stored - 0.75).abs() < 1e-12,
            "0.5 + 0.25 through collect-then-apply, stored: {stored}"
        );
    }

    /// `sub` — the fourth op — on a deffield-declared edge field, completing
    /// the chapter-C2 family's op coverage at the unit level (set/scale/add
    /// have their own rows above).
    #[test]
    fn update_edge_sub_on_a_declared_edge_field() {
        let (mut graph, a, b) = edge_fixture();
        graph
            .update_edge("SOLIDARITY", a, b, "solidarity/tension", 0.7)
            .unwrap();
        let types = edge_types();
        let enums = enums();
        let mut fuel = 128;
        collect_then_apply(
            &mut graph,
            &types,
            &enums,
            edge_binding(a, b),
            "(effects (update-edge e solidarity/tension (sub 0.2i)))",
            &mut fuel,
        )
        .expect("sub must serve");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/tension")
            .unwrap();
        assert!((stored - 0.5).abs() < 1e-12, "0.7 - 0.2, stored: {stored}");
    }

    /// E-PARSE-041's runtime echo (direct-harness defense in depth — the
    /// load-time grammar owns the check, `grammar.rs`'s own test pins that
    /// half): a field-init naming the implicit strength field refuses even
    /// when the load gate was never run.
    #[test]
    fn add_edge_field_init_naming_strength_is_refused_at_execution_too() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let types = edge_types();
        let enums = enums();
        let mut sink = CollectingSink::default();
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let env = EvalEnv {
            bindings: HashMap::from([
                ("self".to_owned(), Value::NodeRef(a)),
                ("other".to_owned(), Value::NodeRef(b)),
            ]),
            intrinsic_costs: &IntrinsicCosts::default(),
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (form, _) = read(
            "(effects (add-edge EdgeType/SOLIDARITY self other :strength 0.5c \
                (solidarity/strength 0.9c)))",
        )
        .expect("effects source must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut fuel = 128;
        let err = executor
            .execute_effects(
                &items[1..],
                &env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .unwrap_err();
        assert!(err.message.contains("E-PARSE-041"), "{err}");
        let stored = graph
            .edge_attribute("SOLIDARITY", a, b, "solidarity/strength")
            .unwrap();
        assert!(
            (stored - 0.5).abs() < 1e-12,
            "the :strength operand's value stands; the second writer was refused: {stored}"
        );
    }
}
