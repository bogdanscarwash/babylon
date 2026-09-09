//! The `GraphSubstrate` trait: the typed-verb surface BSL's structural
//! effects compile against (`docs/reference/bsl-language.rst` §2.6 queries,
//! §2.8 verbs), independent of the underlying storage.
//!
//! **Two typed halves, one substrate (Amendment D, sub-ruling D-2).** The
//! dyadic half (`add_edge`/`remove_edge`) is II.9's strictly dyadic morphism
//! layer. The hyperedge half (`add_hyperedge`/`remove_hyperedge`/
//! `members_of`/`hyperedges_of`) is Amendment D's first-class membership
//! layer. A dyadic caller cannot be handed a hyperedge and vice versa —
//! II.7's "MUST remain separate" is enforced by the type system rather than
//! by two libraries.
//!
//! **Silent on representation, loud on shape.** No adjacency iteration order
//! and no storage type is exposed; a Levi/incidence bipartite store is
//! permitted and unobservable. What IS exposed, because the ruling fixes it:
//! a hyperedge has an identity and a member list, and there is no method
//! anywhere that expands a member list into pairwise edges (VIII.9).
//!
//! **Task 16 revisions (recorded, not silent):** type/attribute names are
//! `&str` (the drafted `&'static str` forced every runtime-string caller
//! through `Box::leak` — the exact smell the Phase-1 plan's own TODO said
//! not to ship); `add_edge` carries the `:strength` operand §2.8's grammar
//! makes mandatory; and `node_attribute` exists because §2.8's four
//! update-ops (`add`/`sub`/`scale` read-modify-write) are unimplementable
//! without a read point. Attribute values are `f64` — the binary64 lane
//! only; a SECOND, parallel i128 lane exists for `currency`-declared node
//! attributes ([`GraphSubstrate::update_node_currency`]/
//! [`GraphSubstrate::node_attribute_currency`] — Half 2 of the
//! typed-attribute-seeding design, T3 #491/OQ-J). The two lanes never
//! share a map: `f64` cannot carry `Currency`'s i128 exactness or its
//! half-even rounding law across a round trip (the design doc's §A), so
//! the binary64 attribute lane still refuses a `Currency`-typed write
//! loudly (`babylon-bsl`'s executor) rather than casting it — the gap this
//! doc used to call "a declared Phase-2 gap" now has its own typed
//! storage, node-scoped only (no edge/hyperedge Currency lane in this
//! train).

use babylon_kernel::Currency;

/// Opaque node identity — a newtype so no caller depends on it being an
/// integer index vs. a UUID vs. anything else the concrete shape picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// Opaque hyperedge identity. **Distinct from [`NodeId`] on purpose**: a
/// hyperedge is a first-class object, not a node with a member set stashed in
/// its attributes (the shape D-4 explicitly declines to ratify for
/// `ECONOMIC_SECTOR`). Two id types is what makes the dyadic/hyperedge
/// separation type-level rather than conventional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HyperedgeId(pub u64);

/// A structural-verb failure. Loud by construction (III.11): every fallible
/// method on [`GraphSubstrate`] returns one rather than silently coercing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphError {
    /// Human-readable cause, suitable for surfacing to a BSL diagnostic.
    pub message: String,
}

/// §2.6 `<direction>` for the `neighbors` query: `:out` follows
/// source→target, `:in` the reverse, `:any` their union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// `:out` — nodes this node points at across the edge type.
    Out,
    /// `:in` — nodes pointing at this node across the edge type.
    In,
    /// `:any` — the set union of both directions.
    Any,
}

/// The typed structural-verb surface a `GraphSubstrate` implementation
/// provides. `node_type`/`edge_type`/`hyperedge_type` are plain `&str` (the
/// closed `NodeType`/`EdgeType`/`HyperedgeType` enums are a `babylon-domain`
/// concern, Phase 2/3; this trait is domain-agnostic on purpose so it
/// compiles before those enums port).
///
/// **Absence is never success** (§2.8): removing what does not exist and
/// adding what already exists are errors on every method below — a
/// substrate that silently no-ops either is non-conforming.
pub trait GraphSubstrate {
    // ---- dyadic half (II.9 morphism layer) ----

    /// Mint one typed node.
    ///
    /// # Errors
    /// Returns [`GraphError`] if the implementation cannot allocate the node.
    /// `u64::MAX` is the reserved exhausted-cursor sentinel, so the last
    /// mintable identity is `NodeId(u64::MAX - 1)`.
    fn add_node(&mut self, node_type: &str) -> Result<NodeId, GraphError>;

    /// Remove a node, **cascading** to its incident structure (ADR185 R2).
    ///
    /// Removal is whole: every incident dyadic edge goes, and the node is
    /// dropped from every hyperedge's member list. A member list is therefore
    /// always a set of LIVE nodes — that is what makes [`Self::members_of`]
    /// mean the same thing to every reader.
    ///
    /// A hyperedge whose last member is removed is itself removed: an empty
    /// hyperedge is unrepresentable here ([`Self::add_hyperedge`] rejects an
    /// empty member list), so leaving one behind would create by deletion a
    /// state that cannot be created directly.
    ///
    /// **The cascade must be OBSERVABLE.** The substrate performs it; the
    /// caller is responsible for recording it. `remove-node` is a BSL
    /// structural verb, so the effect executor emits one write-log record per
    /// cascaded edge and membership — otherwise a class dissolving would
    /// quietly shrink its industry's membership with no event anybody wrote.
    /// Cascade is the semantics; silence is not (ADR185 R2, ADR182 R1).
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no node in this substrate.
    fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError>;

    /// Mint one typed dyadic edge with its mandatory `:strength` (§2.8).
    ///
    /// # Errors
    /// Returns [`GraphError`] if either endpoint does not exist, or if the
    /// `(edge_type, from, to)` edge already exists (`E-EVAL-031` — never a
    /// silent overwrite).
    fn add_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        strength: f64,
    ) -> Result<(), GraphError>;

    /// Remove one typed dyadic edge.
    ///
    /// # Errors
    /// Returns [`GraphError`] if the edge does not exist — absence is never
    /// treated as success (`E-EVAL-031`).
    fn remove_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId) -> Result<(), GraphError>;

    /// Update a single attribute on a node under the I.15 edge-mode state
    /// machine's constraints — this trait method does NOT itself enforce
    /// I.15 (that is a `babylon-domain` law over the concrete shape); it is
    /// the mechanical write point I.15's checker wraps.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no node in this substrate.
    fn update_node(&mut self, id: NodeId, attribute: &str, value: f64) -> Result<(), GraphError>;

    /// Write one dyadic edge's attribute — T3 (ADR198 R1/R3, issue #560):
    /// full symmetric with [`Self::update_node`]. `attribute` is the FULL
    /// QNAME (e.g. `"solidarity/tension"`), mirroring every other method
    /// here — never a bare segment. As with `update_node`, this is the
    /// mechanical write point: the `add`/`sub`/`scale` read-modify-write
    /// happens in the CALLER (`babylon-bsl`'s apply path reads the current
    /// value through [`Self::edge_attribute`], combines, and writes the
    /// result here), and this trait does NOT enforce I.15's edge-mode
    /// transition law (a `babylon-domain` concern over the concrete shape).
    ///
    /// **The strength fork (the double-storage ruling, D143).** A qname
    /// ENDING IN `/strength` writes the edge's EXISTING strength slot — the
    /// same datum [`Self::add_edge`]'s mandatory operand mints and
    /// `CanonicalState` section `0x03` already hashes — and NEVER mints a
    /// fifth-section attribute row: one datum, one hashed home. Any OTHER
    /// qname writes the fifth-section edge-attribute store (listed by
    /// `CanonicalState::all_edge_attributes`, hashed in section `0x05`).
    /// The fork keys on the attribute SUFFIX only — the same deliberate
    /// division [`Self::edge_attribute`] documents: whether the qname's
    /// OWNER segment names `edge_type` is the caller's obligation
    /// (`babylon-bsl`'s referent checks), never verified here.
    ///
    /// **Values are `f64`** — the binary64 lane only. Currency STORAGE
    /// stays refused (ADR198 R1's own clause): `babylon-bsl`'s executor
    /// rejects a Currency-typed write before it ever reaches this trait,
    /// exactly as on the node side.
    ///
    /// # Errors
    /// Returns [`GraphError`] if no `(edge_type, from, to)` edge exists —
    /// under EITHER fork: a write never mints an attribute row for a
    /// nonexistent edge, and an absent edge has no strength slot to write
    /// (§2.8's existence discipline; the honest-null mirror of
    /// [`Self::add_edge`]'s duplicate-add refusal).
    fn update_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
        value: f64,
    ) -> Result<(), GraphError>;

    /// Read a single attribute — the read half §2.8's `add`/`sub`/`scale`
    /// update-ops need for their read-modify-write.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no node, or the attribute has
    /// never been written — never a default `0.0` (the honest-null
    /// discipline, §3.5).
    fn node_attribute(&self, id: NodeId, attribute: &str) -> Result<f64, GraphError>;

    /// Write a Currency-typed node attribute — Half 2 of the
    /// typed-attribute-seeding design (T3 #491, OQ-J): the i128 lane,
    /// stored in a map PARALLEL to [`Self::update_node`]'s `f64` one, never
    /// the same map. Node-scoped only — there is no edge/hyperedge Currency
    /// lane in this train.
    ///
    /// Values are non-negative (`bsl-language.rst` §1.5's Currency row,
    /// `[0, ∞)`) — enforced by the caller (`babylon-bsl`'s executor, the
    /// same division [`Self::update_node`] holds for its own `[0,1]`
    /// domain checks), never by this trait.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no node in this substrate.
    fn update_node_currency(
        &mut self,
        id: NodeId,
        attribute: &str,
        value: Currency,
    ) -> Result<(), GraphError>;

    /// Read a single Currency-typed node attribute — the read half of
    /// [`Self::update_node_currency`].
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no node, or the attribute has
    /// never been written — never a default (the honest-null discipline,
    /// §3.5), exactly as [`Self::node_attribute`] holds for its own lane.
    fn node_attribute_currency(&self, id: NodeId, attribute: &str) -> Result<Currency, GraphError>;

    /// Whether `id` names a live node.
    fn node_exists(&self, id: NodeId) -> bool;

    // Removal semantics are RULED (ADR185 R2), not left to the implementor:
    // see [`Self::remove_node`]. A second implementor that orphans, or that
    // rejects instead of cascading, is wrong — not merely different.

    // ---- §2.6 query surface (dyadic half) ----
    //
    // Iteration order is part of the CONTRACT (§2.6 iteration-order ruling):
    // ascending id order / ascending (source-id, target-id) order — never
    // graph-internal storage order. Predicates (`<node-pred>` etc.) are the
    // evaluator's concern; the substrate provides the unfiltered range.

    /// `(nodes <enum-ref>)` — every node of the given type, ascending
    /// [`NodeId`] order. An unknown type is an empty range, not an error:
    /// type validity is BSL's static check (`E-TYPE-011`), not the
    /// substrate's.
    fn nodes(&self, node_type: &str) -> Vec<NodeId>;

    /// `(edges <enum-ref>)` — every dyadic edge of the given type as
    /// `(source, target)`, ascending `(source-id, target-id)` order.
    fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)>;

    /// `(neighbors <expr> <enum-ref> <direction>)` — the `NodeSet` reachable
    /// from `node` across `edge_type` in `direction`, ascending [`NodeId`]
    /// order, deduplicated (a set, so `:any` never yields a node twice).
    ///
    /// # Errors
    /// Returns [`GraphError`] if `node` does not exist — a dangling
    /// `NodeRef` must never read as an empty neighborhood (the honest-null
    /// discipline).
    fn neighbors(
        &self,
        node: NodeId,
        edge_type: &str,
        direction: Direction,
    ) -> Result<Vec<NodeId>, GraphError>;

    // ---- hyperedge half (Amendment D: first-class membership) ----

    /// Mint one typed hyperedge over `members`. The member list is a SET:
    /// a repeated [`NodeId`], an unknown [`NodeId`], or an empty list is a
    /// loud error (BSL `E-EVAL-031`), never deduplicated or ignored. Cost is
    /// `members.len()` incidences — never `C(n,2)` edges.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `members` is empty, contains a duplicate,
    /// names a node that does not exist, or the identity space is exhausted.
    /// `u64::MAX` is the reserved exhausted-cursor sentinel, so the last
    /// mintable identity is `HyperedgeId(u64::MAX - 1)`.
    fn add_hyperedge(
        &mut self,
        hyperedge_type: &str,
        members: &[NodeId],
    ) -> Result<HyperedgeId, GraphError>;

    /// Remove one hyperedge whole. There is deliberately no `remove_member`:
    /// membership change is whole-hyperedge replacement (BSL invariant S-10).
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no hyperedge in this substrate.
    fn remove_hyperedge(&mut self, id: HyperedgeId) -> Result<(), GraphError>;

    /// Write one `f64` attribute on an existing hyperedge — the hyperedge
    /// own-field lane (Community port train, Task 5, E2a; Amendment D's
    /// first-class membership gains first-class state). Stored in a map
    /// keyed `(HyperedgeId, qname)`, exactly as [`Self::update_edge`]'s
    /// fifth-section store is keyed — never inside the member list.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no hyperedge in this substrate,
    /// or `value` is non-finite — a write never mints state for an absent
    /// hyperedge and never stores a NaN.
    fn update_hyperedge_attribute(
        &mut self,
        id: HyperedgeId,
        attribute: &str,
        value: f64,
    ) -> Result<(), GraphError>;

    /// Read one hyperedge attribute — the read half of
    /// [`Self::update_hyperedge_attribute`].
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no hyperedge in this substrate,
    /// or the attribute was never written — the same two-tier honest-null
    /// discipline [`Self::edge_attribute`] holds (§3.5/III.11): absence is
    /// never a default `0.0`.
    fn hyperedge_attribute(&self, id: HyperedgeId, attribute: &str) -> Result<f64, GraphError>;

    /// Members of one hyperedge, in **ascending [`NodeId`] order** — declared
    /// member order is never observable (BSL §2.6 draft ruling D25).
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no hyperedge in this substrate.
    fn members_of(&self, id: HyperedgeId) -> Result<Vec<NodeId>, GraphError>;

    /// The hyperedges of the given type a node belongs to, in ascending
    /// [`HyperedgeId`] order.
    ///
    /// An unknown *type* is an empty range, matching [`Self::nodes`]: type
    /// validity is BSL's static check (`E-TYPE-011`), not the substrate's.
    /// An unknown *node* is an error, matching [`Self::neighbors`].
    ///
    /// The two halves are not symmetric on purpose. "This node belongs to no
    /// community" and "there is no such node" are different facts, and the
    /// first is load-bearing here: a county with no census rows must not
    /// read as a node that belongs to nothing, because a target belonging to
    /// no protective structure is the CHEAPEST one an adversary can reach
    /// ([`crate::backfire`]). Pine Ridge (FIPS 46102) is empty at every
    /// census vintage, so this is a live data shape, not a hypothetical.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `node` does not exist.
    fn hyperedges_of(
        &self,
        node: NodeId,
        hyperedge_type: &str,
    ) -> Result<Vec<HyperedgeId>, GraphError>;

    /// `(hyperedges <enum-ref>)` — every hyperedge of the given type,
    /// ascending, deduplicated [`HyperedgeId`] order (dedup is structural —
    /// each id is minted once — but it is the accessor's stated guarantee,
    /// not the caller's inference) — the accessor a type-scoped
    /// `(hyperedges …)` BSL query iterates. Symmetric with [`Self::nodes`]
    /// and [`Self::edges`]: total order is the accessor's OWN guarantee,
    /// never the caller's, and an unknown type yields an empty `Vec` exactly
    /// as [`Self::nodes`] does for an unpopulated type — type validity is
    /// BSL's static check (`E-TYPE-011`), not the substrate's, so the
    /// loudness for an invalid type lives at the BOUND checker
    /// (`MissingCeiling`), never here.
    ///
    /// Unlike [`Self::hyperedges_of`] this has no NODE argument, so there is
    /// no second fact to be loud about — it is infallible by signature, the
    /// same shape [`Self::nodes`] and [`Self::edges`] already have.
    fn hyperedges(&self, hyperedge_type: &str) -> Vec<HyperedgeId>;

    /// The declared type of a live hyperedge — the runtime half of
    /// `E-EVAL-032` (D24: a `members-of`/`hyperedges-of` whose referent is
    /// not of the annotated `HyperedgeType` is a mismatch, never a silently
    /// empty set). The static bound checker bounds a `members-of` fold by
    /// the annotated type's census-fed max-members; without this accessor a
    /// referent of a DIFFERENT type would iterate past that bound
    /// unchecked (PR #684's review found the gap).
    ///
    /// READ-ONLY, mirroring [`Self::node_type_of`] exactly: it reports a
    /// fact the substrate already stores to satisfy [`Self::hyperedges`].
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no live hyperedge — a dangling
    /// `HyperedgeRef` never reads as an untyped hyperedge (III.11).
    fn hyperedge_type_of(&self, id: HyperedgeId) -> Result<&str, GraphError>;

    /// The declared type of a live node — `(neighbors … <NodeType>)`'s
    /// filter (§2.6, D24: this operand FILTERS) and §2.10 discipline 1's
    /// `E-EVAL-033` referent check both need it, and neither is expressible
    /// without it.
    ///
    /// READ-ONLY: it reports a fact the substrate already stores to satisfy
    /// [`Self::nodes`]. It adds no state, and `CanonicalState`'s four
    /// sections are untouched.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `id` names no live node — a dangling
    /// `NodeRef` never reads as an untyped node (III.11).
    fn node_type_of(&self, id: NodeId) -> Result<&str, GraphError>;

    /// Read one dyadic edge's attribute (§2.10's `edge-between`/`field-of` share this) — the read
    /// half `edge-between`'s existence check and `field-of` over an `EdgeRef` both derive from.
    /// `attribute` is the FULL QNAME (e.g. `"solidarity/strength"`), mirroring
    /// [`Self::node_attribute`]'s own convention exactly — never a bare segment.
    ///
    /// **T3 (ADR198 R1, issue #560) widened the body from T2's strength-only storage to a real
    /// per-`(edge, qname)` lookup; the SIGNATURE and the ownership division are unchanged
    /// (D141's contract, kept).** The lookup routes on the attribute SUFFIX: a qname ending in
    /// `/strength` reads the edge's strength slot — the datum [`Self::add_edge`] mints and
    /// `CanonicalState` section `0x03` hashes — and any OTHER qname reads the fifth-section
    /// edge-attribute store ([`Self::update_edge`]'s write side, section `0x05`). **This method
    /// still does NOT verify that `attribute`'s OWNER segment names `edge_type`** — exactly as
    /// [`Self::node_attribute`] performs no ownership check of its own, that half of §2.10
    /// discipline 1 is the CALLER's obligation (`field_of_edge`'s `check_edge_referent_type`,
    /// upstream of every call this trait receives), pinned as deliberate by the shared
    /// conformance row `edge_attribute_does_not_check_the_owner_segment` so a future reader
    /// does not "fix" the suffix routing into an owner check by surprise.
    ///
    /// # Errors
    /// Returns [`GraphError`] if no `(edge_type, from, to)` edge exists, or if a non-strength
    /// `attribute` was never written on an existing edge — the same two-tier honest-null
    /// discipline [`Self::node_attribute`] holds (III.11): absence is never a default `0.0`.
    fn edge_attribute(
        &self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
    ) -> Result<f64, GraphError>;
}
