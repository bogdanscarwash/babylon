//! The graph STATE hash — Constitution III.7's "every tick produces a
//! deterministic hash", for the substrate half.
//!
//! [`babylon_kernel::ContentDigest`] fingerprints what the engine was *told*
//! (defines + rules). This fingerprints what the engine *did*: the graph
//! after a tick. Two runs of the same content over the same starting state
//! must produce the same bytes here, and any real change to the world must
//! change them.
//!
//! # The canonical byte layout — normative
//!
//! This is a **cross-language contract**, so the layout is specified rather
//! than implied by one implementation (the rewrite test: another engine in
//! another language must be able to reproduce these bytes from this
//! description alone). Every integer is **big-endian**. Every string is
//! length-prefixed `u32` + raw UTF-8 bytes, never NUL-terminated.
//!
//! ```text
//! 0x01 ‖ u32 count ‖ per node,      ascending id:   u64 id ‖ str type
//! 0x02 ‖ u32 count ‖ per attribute, ascending (id, name):
//!                                    u64 id ‖ str name ‖ u64 value-bits
//! 0x03 ‖ u32 count ‖ per edge,      ascending (type, from, to):
//!                                    str type ‖ u64 from ‖ u64 to ‖ u64 strength-bits
//! 0x04 ‖ u32 count ‖ per hyperedge, ascending id:
//!                                    u64 id ‖ str type ‖ u32 member-count
//!                                           ‖ u64 member id, ascending
//! 0x05 ‖ u32 count ‖ per edge attribute, ascending (type, from, to, qname):
//!                                    str type ‖ u64 from ‖ u64 to ‖ str qname
//!                                           ‖ u64 value-bits
//! 0x06 ‖ u32 count ‖ per Currency node attribute, ascending (id, name):
//!                                    u64 id ‖ str name ‖ i128 value (16 bytes, big-endian)
//! ```
//!
//! **An edge's strength is NOT an edge attribute.** The `add_edge` operand
//! hashes in section `0x03`; section `0x05` holds every OTHER edge-resident
//! field (the deffield rows ADR198 R1 rules in). A write to
//! `<edge-type>/strength` MUST land in the `0x03` slot, never mint a `0x05`
//! row shadowing it — one datum, one hashed home (the double-storage
//! ruling, D143; `GraphSubstrate::update_edge`'s suffix fork enforces it).
//!
//! **Section `0x06` is a SEPARATE lane from `0x02`, not a widened row
//! shape.** A `currency`-declared node attribute never appears in section
//! `0x02` at all (`babylon-bsl`'s executor routes it through
//! [`crate::substrate::GraphSubstrate::update_node_currency`], never
//! `update_node`) — the two sections partition the node-attribute space by
//! declared type, they do not compete for the same row (T3 #491, OQ-J —
//! Half 2 of the typed-attribute-seeding design). The i128 value is written
//! via `i128::to_be_bytes()` — 16 raw bytes, big-endian — the same "every
//! integer is big-endian" rule this module's other sections hold, extended
//! to 128 bits rather than inventing a second integer convention.
//!
//! # Layout versions
//!
//! The contract is **versioned** (ADR198 R2 ordered the elision rule
//! "documented and versioned"; there was no version anchor before T3, so
//! T3 minted the convention — [`CANONICAL_LAYOUT_VERSION`] carries it in
//! code). The version number is never written into the encoding: the bytes
//! stay self-describing by tag, and a stored hash remains comparable across
//! versions exactly as far as the versions are byte-compatible.
//!
//! - **Version 1** (2026-08-11, the hypergraph storage swap): sections
//!   `0x01`–`0x04`, each written UNCONDITIONALLY — a graph with zero
//!   hyperedges still writes `0x04 ‖ 0x00000000`.
//! - **Version 2** (T3, issue #560, ADR198 R1/R2): section `0x05` — edge
//!   attributes — exists, and is **elided when empty**: a state holding no
//!   edge attributes writes NO `0x05` bytes at all. Consequence: every
//!   version-1 state encodes byte-identically under version 2, so every
//!   existing golden, baseline and save survives the widening unmoved — no
//!   rebless ceremony.
//! - **Version 3** (T3 #491, OQ-J, Currency typed-attribute storage): section
//!   `0x06` — Currency node attributes — exists, and is **elided when
//!   empty**, the same rule `0x05` set as precedent. Every version-2 state
//!   (equivalently, every version-1 state with an empty `0x05`) encodes
//!   byte-identically under version 3: nothing in the tree declares a
//!   `currency`-typed `deffield` yet, so this landing moves zero existing
//!   goldens — the design doc's own recommended shape (conditional
//!   emission over the unconditional variant, chosen exactly so Half 2's
//!   landing PR is byte-free until content opts in). Collision note: the
//!   design doc that specced this train assumed section `0x05` for
//!   Currency; `0x05` was independently claimed by the T3/#560 edge-
//!   attribute widening before this train landed, so Currency takes the
//!   next-free tag, `0x06`, instead — a deviation from the design doc's
//!   literal number, not from its shape.
//!
//! **The deliberate asymmetry, recorded as ADR198 R2's own text
//! orders.** Section `0x04`'s empty section IS written (version-1 behavior,
//! unchanged); sections `0x05`/`0x06`'s are NOT. The conventions genuinely
//! differ and the difference is deliberate: elision is what makes a
//! widening byte-compatible, and retrofitting elision onto `0x04` would
//! itself move every existing hash — the cure would be the disease. Every
//! FUTURE section follows `0x05`/`0x06`'s rule: added at the next tag,
//! elided when empty, so a widening no state yet exercises never moves an
//! existing hash. A change to an EXISTING section's layout is never a
//! version bump; it is a contract change that invalidates every stored
//! state hash and every other implementation (see the pinned byte vectors
//! in this module's tests).
//!
//! - **Version 4** (Community port train, Task 5/E2a, hyperedge own-field
//!   attributes): section `0x07` — `(hyperedge-id, qname, f64)` rows —
//!   exists, and is **elided when empty**, exactly the rule
//!   `0x05`/`0x06` set. Every version-3 state encodes byte-identically
//!   under version 4: nothing in the tree declares a hyperedge field yet
//!   (that surface is Task 6's), so this landing moves zero existing
//!   goldens. Collision note, same class as version 3's: the Community
//!   plan's literal text assumed section `0x06` for this lane; T3's
//!   Currency widening claimed `0x06` before this train landed, so the
//!   lane takes the next-free tag, `0x07` — a deviation from the plan's
//!   literal number, not from its shape.
//!
//! **Section tags are not decoration.** Without them a graph with one node
//! and no edges could serialize identically to one with no nodes and one
//! edge under some count arrangement; the tag makes each section's identity
//! explicit.
//!
//! **Sorting is the whole determinism argument.** `MemoryGraph` stores in
//! `HashMap`s, whose iteration order varies per process. Every section sorts
//! before writing, so storage order is unobservable here exactly as it is
//! unobservable through the trait's ranged accessors.
//!
//! # Floats
//!
//! Attribute values and edge strengths are `f64`, and two IEEE-754 facts
//! would otherwise break the contract:
//!
//! - **`-0.0` and `+0.0` compare equal but do not serialize equal.** A sign
//!   bit arriving from upstream arithmetic (`-1.0 * 0.0`) would change the
//!   hash without changing the world. They are canonicalized to `+0.0`.
//! - **NaN is not a value.** It is refused loudly rather than hashed, so a
//!   non-finite can never enter the tick hash and make a run irreproducible.
//!   The write path already refuses non-finites; this is the second gate,
//!   because a hash that silently accepted one would launder it.
//!
//! Finite values are hashed by `to_bits()`, which is exact — no formatting,
//! no rounding, no locale.

use crate::substrate::{GraphError, HyperedgeId, NodeId};
use babylon_kernel::{sha256_of, Currency};

const TAG_NODES: u8 = 0x01;
const TAG_ATTRIBUTES: u8 = 0x02;
const TAG_EDGES: u8 = 0x03;
const TAG_HYPEREDGES: u8 = 0x04;
const TAG_EDGE_ATTRIBUTES: u8 = 0x05;
const TAG_CURRENCY_ATTRIBUTES: u8 = 0x06;
const TAG_HYPEREDGE_ATTRIBUTES: u8 = 0x07;

/// The section tags are the whole basis for `TAG_NODES`/`TAG_EDGES`/… being
/// distinguishable on the wire (module doc, "Section tags are not
/// decoration") — a future edit that let two of them collide would silently
/// merge two sections' bytes into one ambiguous stream. `bsl-lint
/// namespace-unique`'s (c) arm (the BSL hygiene knock-out train, W1) is this
/// check's repo-relationship half; this is its content half, kept next to
/// the constants it guards rather than in the external tool.
#[test]
fn tag_constants_are_pairwise_distinct() {
    let tags = [
        TAG_NODES,
        TAG_ATTRIBUTES,
        TAG_EDGES,
        TAG_HYPEREDGES,
        TAG_EDGE_ATTRIBUTES,
        TAG_CURRENCY_ATTRIBUTES,
        TAG_HYPEREDGE_ATTRIBUTES,
    ];
    let mut seen = std::collections::HashSet::new();
    for tag in tags {
        assert!(
            seen.insert(tag),
            "duplicate state_hash section tag: {tag:#04x}"
        );
    }
}

/// The canonical-layout version (module doc, "Layout versions"). **4** =
/// version 3's six sections plus section `0x07` (hyperedge own-field
/// attributes, Community port train Task 5/E2a — T3's Currency lane took
/// `0x06` first, so this section is `0x07`), the seventh **elided when
/// empty** — a byte-compatible widening: every version-3 state (so every
/// earlier one, since `0x05`/`0x06` were already elidable) hashes
/// identically under version 4. Never written into the encoding itself;
/// this versions the CONTRACT for cross-language reimplementations, not the
/// payload. Bumping it is a contract event — the module doc's "Layout
/// versions" section says what each version means and what may never be
/// one.
pub const CANONICAL_LAYOUT_VERSION: u32 = 4;

/// Accumulates the canonical encoding described in the module docs.
///
/// Sections must be written in tag order; the encoder does not reorder them,
/// because a caller that emitted them out of order has a bug the hash should
/// expose rather than paper over.
#[derive(Debug, Default)]
pub struct StateEncoder {
    bytes: Vec<u8>,
}

impl StateEncoder {
    /// A new, empty encoding.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn push_str(&mut self, value: &str) -> Result<(), GraphError> {
        let len = u32::try_from(value.len()).map_err(|_| GraphError {
            message: format!(
                "string of {} bytes exceeds the u32 length prefix",
                value.len()
            ),
        })?;
        self.bytes.extend_from_slice(&len.to_be_bytes());
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }

    fn push_count(&mut self, tag: u8, count: usize) -> Result<(), GraphError> {
        let count = u32::try_from(count).map_err(|_| GraphError {
            message: format!("section {tag:#04x} holds {count} entries, past the u32 count"),
        })?;
        self.bytes.push(tag);
        self.bytes.extend_from_slice(&count.to_be_bytes());
        Ok(())
    }

    /// Canonicalize a stored float and append its bits.
    ///
    /// # Errors
    /// Returns [`GraphError`] on a non-finite value — it must never reach the
    /// tick hash.
    fn push_f64(&mut self, value: f64, what: &str) -> Result<(), GraphError> {
        if !value.is_finite() {
            return Err(GraphError {
                message: format!(
                    "{what} is {value} — a non-finite value cannot enter the state hash, \
                     because a run that hashed one could not be reproduced"
                ),
            });
        }
        // `-0.0 == 0.0` is true, so this normalizes the sign bit without a
        // branch on `is_sign_negative`.
        let canonical = if value == 0.0 { 0.0_f64 } else { value };
        self.bytes
            .extend_from_slice(&canonical.to_bits().to_be_bytes());
        Ok(())
    }

    /// Section `0x01`. `nodes` must already be sorted ascending by id.
    ///
    /// # Errors
    /// Returns [`GraphError`] if a count or string length overflows its prefix.
    pub fn write_nodes(&mut self, nodes: &[(NodeId, String)]) -> Result<(), GraphError> {
        self.push_count(TAG_NODES, nodes.len())?;
        for (id, node_type) in nodes {
            self.bytes.extend_from_slice(&id.0.to_be_bytes());
            self.push_str(node_type)?;
        }
        Ok(())
    }

    /// Section `0x02`. `attributes` must already be sorted ascending by
    /// `(id, name)`.
    ///
    /// # Errors
    /// Returns [`GraphError`] on overflow or a non-finite value.
    pub fn write_attributes(
        &mut self,
        attributes: &[(NodeId, String, f64)],
    ) -> Result<(), GraphError> {
        self.push_count(TAG_ATTRIBUTES, attributes.len())?;
        for (id, name, value) in attributes {
            self.bytes.extend_from_slice(&id.0.to_be_bytes());
            self.push_str(name)?;
            self.push_f64(*value, &format!("attribute {name} on {id:?}"))?;
        }
        Ok(())
    }

    /// Section `0x03`. `edges` must already be sorted ascending by
    /// `(type, from, to)`.
    ///
    /// # Errors
    /// Returns [`GraphError`] on overflow or a non-finite strength.
    pub fn write_edges(
        &mut self,
        edges: &[(String, NodeId, NodeId, f64)],
    ) -> Result<(), GraphError> {
        self.push_count(TAG_EDGES, edges.len())?;
        for (edge_type, from, to, strength) in edges {
            self.push_str(edge_type)?;
            self.bytes.extend_from_slice(&from.0.to_be_bytes());
            self.bytes.extend_from_slice(&to.0.to_be_bytes());
            self.push_f64(*strength, &format!("strength of {edge_type} edge"))?;
        }
        Ok(())
    }

    /// Section `0x04`. `hyperedges` must already be sorted ascending by id,
    /// each member list ascending.
    ///
    /// # Errors
    /// Returns [`GraphError`] if a count or string length overflows.
    pub fn write_hyperedges(
        &mut self,
        hyperedges: &[(HyperedgeId, String, Vec<NodeId>)],
    ) -> Result<(), GraphError> {
        self.push_count(TAG_HYPEREDGES, hyperedges.len())?;
        for (id, hyperedge_type, members) in hyperedges {
            self.bytes.extend_from_slice(&id.0.to_be_bytes());
            self.push_str(hyperedge_type)?;
            let count = u32::try_from(members.len()).map_err(|_| GraphError {
                message: format!("hyperedge {id:?} holds {} members", members.len()),
            })?;
            self.bytes.extend_from_slice(&count.to_be_bytes());
            for member in members {
                self.bytes.extend_from_slice(&member.0.to_be_bytes());
            }
        }
        Ok(())
    }

    /// Section `0x05` (layout version 2 — the module doc's "Layout
    /// versions"). `edge_attributes` must already be sorted ascending by
    /// `(type, from, to, qname)`. **The elision decision is the CALLER's**:
    /// this method writes what it is given, unconditionally;
    /// [`CanonicalState::encode_state`] is the one place that decides an
    /// empty fifth listing contributes zero bytes (ADR198 R2).
    ///
    /// # Errors
    /// Returns [`GraphError`] on overflow or a non-finite value.
    pub fn write_edge_attributes(
        &mut self,
        edge_attributes: &[(String, NodeId, NodeId, String, f64)],
    ) -> Result<(), GraphError> {
        self.push_count(TAG_EDGE_ATTRIBUTES, edge_attributes.len())?;
        for (edge_type, from, to, qname, value) in edge_attributes {
            self.push_str(edge_type)?;
            self.bytes.extend_from_slice(&from.0.to_be_bytes());
            self.bytes.extend_from_slice(&to.0.to_be_bytes());
            self.push_str(qname)?;
            self.push_f64(
                *value,
                &format!("attribute {qname} on {edge_type} edge ({from:?}, {to:?})"),
            )?;
        }
        Ok(())
    }

    /// Section `0x06` (layout version 3 — the module doc's "Layout
    /// versions"). `currency_attributes` must already be sorted ascending
    /// by `(id, name)` — the SAME key section `0x02` sorts by, since this
    /// is the identical node-attribute space, partitioned by declared type.
    /// **The elision decision is the CALLER's**: this method writes what it
    /// is given, unconditionally; [`CanonicalState::encode_state`] is the
    /// one place that decides an empty sixth listing contributes zero
    /// bytes (mirroring ADR198 R2's `0x05` precedent).
    ///
    /// # Errors
    /// Returns [`GraphError`] if a count or string length overflows its
    /// prefix. Currency has no non-finite/negative-zero hazard — `i128` is
    /// exact by construction — so this method has no value-level refusal
    /// the way [`Self::write_attributes`] does for `f64`.
    pub fn write_currency_attributes(
        &mut self,
        currency_attributes: &[(NodeId, String, Currency)],
    ) -> Result<(), GraphError> {
        self.push_count(TAG_CURRENCY_ATTRIBUTES, currency_attributes.len())?;
        for (id, name, value) in currency_attributes {
            self.bytes.extend_from_slice(&id.0.to_be_bytes());
            self.push_str(name)?;
            self.bytes
                .extend_from_slice(&value.micro_units().to_be_bytes());
        }
        Ok(())
    }

    /// Section `0x07` (layout version 4 — the module doc's "Layout
    /// versions"; Community port train, Task 5, E2a).
    /// `hyperedge_attributes` must already be sorted ascending by
    /// `(id, qname)` — the same key sections `0x02`/`0x06` sort by.
    /// **The elision decision is the CALLER's**: this method writes what it
    /// is given, unconditionally; [`CanonicalState::encode_state`] is the
    /// one place that decides an empty seventh listing contributes zero
    /// bytes (ADR198 R2's precedent, restated per section).
    ///
    /// # Errors
    /// Returns [`GraphError`] on overflow or a non-finite value.
    pub fn write_hyperedge_attributes(
        &mut self,
        hyperedge_attributes: &[(HyperedgeId, String, f64)],
    ) -> Result<(), GraphError> {
        self.push_count(TAG_HYPEREDGE_ATTRIBUTES, hyperedge_attributes.len())?;
        for (id, qname, value) in hyperedge_attributes {
            self.bytes.extend_from_slice(&id.0.to_be_bytes());
            self.push_str(qname)?;
            self.push_f64(*value, &format!("attribute {qname} on hyperedge {id:?}"))?;
        }
        Ok(())
    }

    /// The canonical bytes, for a differential when two hashes disagree.
    ///
    /// A bare hash says only *that* two states differ; the bytes say where.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 over the canonical encoding.
    #[must_use]
    pub fn finish(&self) -> [u8; 32] {
        sha256_of(&self.bytes)
    }
}

/// A store's seven-way listing of its own contents, plus the ONE canonical
/// encoder built on top of them. (N-way history: five before T3's Currency
/// lane, six before the Community train's hyperedge own-field lane.)
///
/// **Why this trait exists rather than widening [`crate::substrate::GraphSubstrate`].**
/// The substrate trait offers only type-keyed ranges
/// (`nodes(node_type)`, `edges(edge_type)`) and keyed point lookups — no way
/// to list which types exist, no way to list attribute names. It cannot
/// yield the canonical encoding. Listing the whole store is a storage
/// capability a store must declare separately, on a trait about
/// serialization rather than about the structural-verb surface Amendment D
/// ratified.
///
/// **The point is not tidiness — it is that a second store cannot move the
/// bytes by encoding differently, because it does not encode.** A store
/// reports facts through the five required methods; [`Self::encode_state`]
/// sorts them on the ruled key and writes the five sections (the fifth
/// elided when empty, ADR198 R2), and every store shares that one
/// implementation. A swap can change the hash only by
/// reporting a different set of facts, which is a real defect rather than a
/// formatting difference — turning an open-ended "did the bytes move?"
/// question into a closed one.
///
/// **The fifth listing is REQUIRED, not defaulted** (T3, issue #560 — the
/// dossier's design hazard, taken): a default-empty `all_edge_attributes`
/// would let a store silently forget to report edge attributes, which is
/// exactly the "reporting different facts" failure the one-encoder design
/// exists to surface. A required method makes every implementor answer the
/// question out loud, at compile time.
pub trait CanonicalState {
    /// Every node, in any order — [`Self::encode_state`] sorts.
    fn all_nodes(&self) -> Vec<(NodeId, String)>;
    /// Every attribute row, in any order.
    fn all_attributes(&self) -> Vec<(NodeId, String, f64)>;
    /// Every dyadic edge, in any order.
    fn all_edges(&self) -> Vec<(String, NodeId, NodeId, f64)>;
    /// Every hyperedge with its member list, in any order — member lists
    /// need not be pre-sorted either; [`Self::encode_state`] sorts them too.
    fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)>;
    /// Every edge-attribute row as `(edge_type, from, to, qname, value)`, in
    /// any order (T3, ADR198 R1/R2). **An edge's strength is NOT reported
    /// here** — it reports through [`Self::all_edges`] and hashes in section
    /// `0x03`; this listing is section `0x05`'s facts and nothing else (the
    /// double-storage ruling, D143 — one datum, one hashed home).
    fn all_edge_attributes(&self) -> Vec<(String, NodeId, NodeId, String, f64)>;
    /// Every Currency-typed node-attribute row as `(id, name, value)`, in
    /// any order (T3 #491, OQ-J — Half 2 of the typed-attribute-seeding
    /// design). **A `currency`-declared field never reports here AND in
    /// [`Self::all_attributes`]** — the two listings partition the
    /// node-attribute space by declared type, they never overlap on the
    /// same `(id, name)` key, mirroring [`Self::all_edge_attributes`]'s
    /// own disjointness from [`Self::all_edges`] (the double-storage
    /// ruling, D143, restated for the sixth section).
    fn all_currency_attributes(&self) -> Vec<(NodeId, String, Currency)>;
    /// Every hyperedge-attribute row as `(id, qname, value)`, in any order
    /// (Community port train, Task 5, E2a — section `0x07`, layout version
    /// 4). **REQUIRED, never defaulted** — the fifth/sixth listings' own
    /// argument, restated: a default-empty body would let a store silently
    /// forget to report, which is exactly the "reporting different facts"
    /// failure the one-encoder design exists to surface.
    fn all_hyperedge_attributes(&self) -> Vec<(HyperedgeId, String, f64)>;

    /// The canonical encoding (module docs) — the ONLY place the sort and
    /// the `write_*` calls happen, for every store that ever implements
    /// this trait.
    ///
    /// Sorts: nodes by id; attributes by `(id, name)`; edges by
    /// `(type, from, to)`; hyperedges by id, each member list ascending;
    /// edge attributes by `(type, from, to, qname)`; Currency attributes by
    /// `(id, name)` — the same key section `0x02` sorts by. The
    /// member lists are sorted HERE, not only trusted from the listing — a
    /// store reporting them in storage order must still hash correctly,
    /// because the sort contract belongs to the encoder, never to the
    /// store's internal order.
    ///
    /// **Elision (ADR198 R2, extended to `0x06` by T3 #491/OQ-J):** the
    /// fifth section is written ONLY when at least one edge attribute
    /// exists, and the sixth ONLY when at least one Currency attribute
    /// exists — a state holding none emits no `0x05`/`0x06` bytes, so
    /// every version-1 (or version-2) hash stays byte-identical. The four
    /// version-1 sections are written unconditionally (the deliberate
    /// asymmetry the module doc's "Layout versions" records).
    ///
    /// # Errors
    /// Returns [`GraphError`] if a non-finite value is stored or a count
    /// overflows its length prefix — see [`StateEncoder`]'s `write_*`
    /// methods.
    fn encode_state(&self) -> Result<StateEncoder, GraphError> {
        let mut encoder = StateEncoder::new();

        let mut nodes = self.all_nodes();
        nodes.sort_unstable_by_key(|(id, _)| *id);
        encoder.write_nodes(&nodes)?;

        let mut attributes = self.all_attributes();
        attributes.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        encoder.write_attributes(&attributes)?;

        let mut edges = self.all_edges();
        edges.sort_unstable_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
        });
        encoder.write_edges(&edges)?;

        let mut hyperedges = self.all_hyperedges();
        for (_, _, members) in &mut hyperedges {
            members.sort_unstable();
        }
        hyperedges.sort_unstable_by_key(|(id, _, _)| *id);
        encoder.write_hyperedges(&hyperedges)?;

        let mut edge_attributes = self.all_edge_attributes();
        edge_attributes.sort_unstable_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.3.cmp(&b.3))
        });
        // ADR198 R2: elided when empty — no tag, no count, no bytes at all.
        if !edge_attributes.is_empty() {
            encoder.write_edge_attributes(&edge_attributes)?;
        }

        let mut currency_attributes = self.all_currency_attributes();
        currency_attributes.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        // Same elision rule as 0x05 above (module doc, "Layout versions"
        // version 3): no tag, no count, no bytes at all when empty.
        if !currency_attributes.is_empty() {
            encoder.write_currency_attributes(&currency_attributes)?;
        }

        let mut hyperedge_attributes = self.all_hyperedge_attributes();
        hyperedge_attributes.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        // Section 0x07 (layout version 4): the same empty-elision rule as
        // 0x05/0x06 — no tag, no count, no bytes at all when empty (ADR198
        // R2's precedent, restated per section).
        if !hyperedge_attributes.is_empty() {
            encoder.write_hyperedge_attributes(&hyperedge_attributes)?;
        }

        Ok(encoder)
    }

    /// The tick-hash contribution of this store's state (Constitution
    /// III.7).
    ///
    /// # Errors
    /// Returns [`GraphError`] for the reasons [`Self::encode_state`] does.
    fn state_hash(&self) -> Result<[u8; 32], GraphError> {
        Ok(self.encode_state()?.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::{CanonicalState, StateEncoder};
    use crate::substrate::{HyperedgeId, NodeId};
    use babylon_kernel::Currency;
    use std::fmt::Write as _;

    /// A hand-built fixture implementing only the six listings, so
    /// [`CanonicalState::encode_state`]/`state_hash` are exercised as the
    /// PROVIDED methods they are — never re-derived per store.
    struct Facts {
        nodes: Vec<(NodeId, String)>,
        attributes: Vec<(NodeId, String, f64)>,
        edges: Vec<(String, NodeId, NodeId, f64)>,
        hyperedges: Vec<(HyperedgeId, String, Vec<NodeId>)>,
        edge_attributes: Vec<(String, NodeId, NodeId, String, f64)>,
        currency_attributes: Vec<(NodeId, String, Currency)>,
        hyperedge_attributes: Vec<(HyperedgeId, String, f64)>,
    }

    impl CanonicalState for Facts {
        fn all_nodes(&self) -> Vec<(NodeId, String)> {
            self.nodes.clone()
        }
        fn all_attributes(&self) -> Vec<(NodeId, String, f64)> {
            self.attributes.clone()
        }
        fn all_edges(&self) -> Vec<(String, NodeId, NodeId, f64)> {
            self.edges.clone()
        }
        fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)> {
            self.hyperedges.clone()
        }
        fn all_edge_attributes(&self) -> Vec<(String, NodeId, NodeId, String, f64)> {
            self.edge_attributes.clone()
        }
        fn all_currency_attributes(&self) -> Vec<(NodeId, String, Currency)> {
            self.currency_attributes.clone()
        }
        fn all_hyperedge_attributes(&self) -> Vec<(HyperedgeId, String, f64)> {
            self.hyperedge_attributes.clone()
        }
    }

    /// The provided `encode_state` reproduces the exact pinned byte array
    /// that guards the manual `StateEncoder` call sequence — proving the
    /// trait's single implementation is the SAME function, not a lookalike.
    ///
    /// **This test is also ADR198 R2's elision proof at the trait level**
    /// (T3, issue #560): the fixture's `edge_attributes` is EMPTY, so the
    /// version-2 elision rule writes no `0x05` bytes and the pre-T3 pin
    /// stands byte-identical, unmodified. If the fifth section were written
    /// unconditionally, this test and its digest twin below would flip red.
    #[test]
    fn the_provided_encode_state_reproduces_the_pinned_bytes() {
        let facts = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![],
            currency_attributes: vec![],
            hyperedge_attributes: Vec::new(),
        };

        #[rustfmt::skip]
        let expected: &[u8] = &[
            0x01,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'c',
            0x02,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'w',
            0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'E',
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x3F, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            0x00, 0x00, 0x00, 0x01, b'H',
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        ];

        let bytes = facts.encode_state().unwrap();
        assert_eq!(
            bytes.as_bytes(),
            expected,
            "the provided encode_state moved the pinned byte vector"
        );

        let hex = facts
            .state_hash()
            .unwrap()
            .iter()
            .fold(String::new(), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            });
        assert_eq!(
            hex, "5e0041a4948bc52530bdcc3a19e61f94aee5523027e2ed1aee5310109fa1c0d8",
            "the provided state_hash moved the pinned digest"
        );
    }

    /// The provided method sorts — a store reporting its six facts in
    /// deliberately scrambled order must hash identically to one reporting
    /// them already sorted, because the sort contract lives in the provided
    /// method and never depends on the caller's listing order. The
    /// `edge_attributes` rows (T3, ADR198 R1/R2) swap their listing order
    /// between the two fixtures, so the fifth section's own sort —
    /// `(type, from, to, qname)` — is what this test proves, not just the
    /// first four sections'. The `currency_attributes` rows (T3 #491, OQ-J)
    /// swap order too, proving the sixth section's `(id, name)` sort the
    /// same way.
    // The two hand-built `Facts` fixtures are inherently one flat literal
    // each (six fields, several rows apiece) — splitting them into helper
    // functions would obscure the "same facts, deliberately scrambled
    // order" pairing the test's whole point rests on, worse than the length
    // itself.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn the_provided_encode_state_sorts_regardless_of_listing_order() {
        let sorted = Facts {
            nodes: vec![
                (NodeId(1), "social_class".to_owned()),
                (NodeId(2), "social_class".to_owned()),
                (NodeId(3), "territory".to_owned()),
            ],
            attributes: vec![
                (NodeId(1), "a".to_owned(), 1.0),
                (NodeId(1), "b".to_owned(), 2.0),
                (NodeId(2), "a".to_owned(), 3.0),
            ],
            edges: vec![
                ("solidarity".to_owned(), NodeId(1), NodeId(2), 0.5),
                ("wages".to_owned(), NodeId(2), NodeId(3), 0.9),
            ],
            hyperedges: vec![
                (
                    HyperedgeId(0),
                    "sector".to_owned(),
                    vec![NodeId(1), NodeId(2)],
                ),
                (HyperedgeId(1), "sector".to_owned(), vec![NodeId(3)]),
            ],
            edge_attributes: vec![
                (
                    "solidarity".to_owned(),
                    NodeId(1),
                    NodeId(2),
                    "solidarity/tension".to_owned(),
                    0.7,
                ),
                (
                    "solidarity".to_owned(),
                    NodeId(1),
                    NodeId(2),
                    "solidarity/trust".to_owned(),
                    0.2,
                ),
            ],
            currency_attributes: vec![
                (
                    NodeId(1),
                    "social-class/wages".to_owned(),
                    Currency::from_micro_units(500_000),
                ),
                (
                    NodeId(2),
                    "social-class/wages".to_owned(),
                    Currency::from_micro_units(1_000_000),
                ),
            ],
            hyperedge_attributes: Vec::new(),
        };

        let scrambled = Facts {
            nodes: vec![
                (NodeId(3), "territory".to_owned()),
                (NodeId(1), "social_class".to_owned()),
                (NodeId(2), "social_class".to_owned()),
            ],
            attributes: vec![
                (NodeId(2), "a".to_owned(), 3.0),
                (NodeId(1), "b".to_owned(), 2.0),
                (NodeId(1), "a".to_owned(), 1.0),
            ],
            edges: vec![
                ("wages".to_owned(), NodeId(2), NodeId(3), 0.9),
                ("solidarity".to_owned(), NodeId(1), NodeId(2), 0.5),
            ],
            hyperedges: vec![
                (HyperedgeId(1), "sector".to_owned(), vec![NodeId(3)]),
                (
                    HyperedgeId(0),
                    "sector".to_owned(),
                    // member list itself scrambled too
                    vec![NodeId(2), NodeId(1)],
                ),
            ],
            edge_attributes: vec![
                (
                    "solidarity".to_owned(),
                    NodeId(1),
                    NodeId(2),
                    "solidarity/trust".to_owned(),
                    0.2,
                ),
                (
                    "solidarity".to_owned(),
                    NodeId(1),
                    NodeId(2),
                    "solidarity/tension".to_owned(),
                    0.7,
                ),
            ],
            currency_attributes: vec![
                (
                    NodeId(2),
                    "social-class/wages".to_owned(),
                    Currency::from_micro_units(1_000_000),
                ),
                (
                    NodeId(1),
                    "social-class/wages".to_owned(),
                    Currency::from_micro_units(500_000),
                ),
            ],
            hyperedge_attributes: Vec::new(),
        };

        assert_eq!(
            sorted.encode_state().unwrap().as_bytes(),
            scrambled.encode_state().unwrap().as_bytes(),
            "the provided encode_state must be invariant to listing order"
        );
        assert_eq!(
            sorted.state_hash().unwrap(),
            scrambled.state_hash().unwrap()
        );
    }

    fn encoder_with_one_attribute(value: f64) -> [u8; 32] {
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "social_class".to_owned())])
            .unwrap();
        enc.write_attributes(&[(NodeId(1), "wealth".to_owned(), value)])
            .unwrap();
        enc.write_edges(&[]).unwrap();
        enc.write_hyperedges(&[]).unwrap();
        enc.finish()
    }

    /// **The golden byte vector — the actual cross-language contract.**
    ///
    /// Every other test in this module is RELATIONAL: same-in/same-out,
    /// changed-in/changed-out, distinct-shapes-differ. All of them would
    /// still pass if this encoder wrote little-endian integers, or moved
    /// the length prefix after the string, or swapped `from`/`to`. They
    /// pin that the encoding is a *function*; they do not pin *which*
    /// function, and the module docs above claim a normative layout that
    /// another language must reproduce from the description alone.
    ///
    /// So this test writes one entry into every section and asserts the
    /// exact bytes, annotated field by field. A reimplementation in any
    /// language can be checked against this array without running Rust —
    /// which is the whole point of calling the layout normative.
    #[test]
    fn the_canonical_encoding_is_pinned_byte_for_byte() {
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        enc.write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        enc.write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        enc.write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();

        #[rustfmt::skip]
        let expected: &[u8] = &[
            // ── section 0x01: nodes ──────────────────────────────────
            0x01,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 id = 1
            0x00, 0x00, 0x00, 0x01, b'c',                    // str "c"
            // ── section 0x02: attributes ─────────────────────────────
            0x02,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 id = 1
            0x00, 0x00, 0x00, 0x01, b'w',                    // str "w"
            0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // f64 1.0 bits
            // ── section 0x03: edges ──────────────────────────────────
            0x03,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x01, b'E',                    // str "E" FIRST
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 from = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,  // u64 to   = 2
            0x3F, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // f64 0.5 bits
            // ── section 0x04: hyperedges ─────────────────────────────
            0x04,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,  // u64 id = 7
            0x00, 0x00, 0x00, 0x01, b'H',                    // str "H"
            0x00, 0x00, 0x00, 0x02,                          // u32 members = 2
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 member 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,  // u64 member 2
        ];

        assert_eq!(
            enc.as_bytes(),
            expected,
            "the canonical encoding moved — if this was deliberate it is a \
             CONTRACT CHANGE that invalidates every stored state hash and \
             every other implementation, not a test to re-bless casually"
        );
    }

    /// The digest over that exact vector, pinned so a change of hash
    /// FUNCTION is caught too — the byte test above cannot see that.
    /// Generated once from this implementation, byte-pinned thereafter.
    #[test]
    fn the_golden_vector_hashes_to_its_pinned_digest() {
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        enc.write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        enc.write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        enc.write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();

        // `fold` + `write!` rather than `map(format!).collect()`: the latter
        // allocates a String per byte and clippy::pedantic refuses it.
        let hex = enc.finish().iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        });
        assert_eq!(
            hex, "5e0041a4948bc52530bdcc3a19e61f94aee5523027e2ed1aee5310109fa1c0d8",
            "SHA-256 over the golden vector moved: either the encoding changed \
             (see the byte test) or the digest function did"
        );
    }

    /// **The fifth section's layout is normative too — pinned byte for
    /// byte** (T3, ADR198 R1/R2, issue #560). The four-section prefix below
    /// is the version-1 golden vector VERBATIM; the `0x05` block is the new
    /// section appended after it. A reimplementation in any language can be
    /// checked against this array without running Rust.
    #[test]
    fn the_fifth_section_is_pinned_byte_for_byte() {
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        enc.write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        enc.write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        enc.write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();
        enc.write_edge_attributes(&[("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)])
            .unwrap();

        #[rustfmt::skip]
        let expected: &[u8] = &[
            // ── sections 0x01-0x04: the version-1 golden vector, verbatim ──
            0x01,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 id = 1
            0x00, 0x00, 0x00, 0x01, b'c',                    // str "c"
            0x02,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 id = 1
            0x00, 0x00, 0x00, 0x01, b'w',                    // str "w"
            0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // f64 1.0 bits
            0x03,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x01, b'E',                    // str "E" FIRST
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 from = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,  // u64 to   = 2
            0x3F, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // f64 0.5 bits
            0x04,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,  // u64 id = 7
            0x00, 0x00, 0x00, 0x01, b'H',                    // str "H"
            0x00, 0x00, 0x00, 0x02,                          // u32 members = 2
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 member 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,  // u64 member 2
            // ── section 0x05: edge attributes (layout version 2) ─────────
            0x05,                                            // tag
            0x00, 0x00, 0x00, 0x01,                          // u32 count = 1
            0x00, 0x00, 0x00, 0x01, b'E',                    // str edge type "E"
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,  // u64 from = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,  // u64 to   = 2
            0x00, 0x00, 0x00, 0x01, b't',                    // str qname "t"
            0x3F, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // f64 0.25 bits
        ];

        assert_eq!(
            enc.as_bytes(),
            expected,
            "the fifth section's canonical encoding moved — a CONTRACT CHANGE of the same \
             order as the four-section golden above, not a test to re-bless casually"
        );
    }

    /// **The sixth section's layout is normative too — pinned byte for
    /// byte** (T3 #491, OQ-J). The five-section prefix below is the
    /// version-2 golden vector VERBATIM (the fifth section's own pinned
    /// test above, one entry); the `0x06` block is the new section appended
    /// after it. A single 1-micro-unit value keeps the i128 hand-encoding
    /// unambiguous: 15 zero bytes then `0x01`. A reimplementation in any
    /// language can be checked against this array without running Rust.
    #[test]
    fn the_sixth_section_is_pinned_byte_for_byte() {
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        enc.write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        enc.write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        enc.write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();
        enc.write_edge_attributes(&[("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)])
            .unwrap();
        enc.write_currency_attributes(&[(
            NodeId(1),
            "m".to_owned(),
            Currency::from_micro_units(1),
        )])
        .unwrap();

        #[rustfmt::skip]
        let expected: &[u8] = &[
            // ── sections 0x01-0x05: the version-2 golden vector, verbatim ──
            0x01,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'c',
            0x02,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'w',
            0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'E',
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x3F, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            0x00, 0x00, 0x00, 0x01, b'H',
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x05,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, b'E',
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x01, b't',
            0x3F, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ── section 0x06: Currency node attributes (layout version 3) ──
            0x06,                                                              // tag
            0x00, 0x00, 0x00, 0x01,                                            // u32 count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,                    // u64 id = 1
            0x00, 0x00, 0x00, 0x01, b'm',                                      // str "m"
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                    // i128 hi = 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,                    // i128 lo = 1 micro-unit
        ];

        assert_eq!(
            enc.as_bytes(),
            expected,
            "the sixth section's canonical encoding moved — a CONTRACT CHANGE of the same \
             order as the fifth-section golden above, not a test to re-bless casually"
        );
    }

    /// ADR198 R2's elision rule, pinned as behavior rather than prose: the
    /// provided `encode_state` over a store reporting NO edge attributes
    /// produces exactly the bytes a hand-driven four-section encoder
    /// produces — no `0x05` tag, no zero count, nothing. (If the elision
    /// branch is deleted, this flips red.)
    #[test]
    fn an_empty_edge_attribute_listing_writes_no_fifth_section_bytes() {
        let facts = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![],
            currency_attributes: vec![],
            hyperedge_attributes: Vec::new(),
        };
        let mut manual = StateEncoder::new();
        manual.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        manual
            .write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        manual
            .write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        manual
            .write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();
        assert_eq!(
            facts.encode_state().unwrap().as_bytes(),
            manual.as_bytes(),
            "an empty fifth listing must contribute ZERO bytes (ADR198 R2's empty-elision)"
        );
    }

    /// The dual of elision: ONE stored edge attribute must be hash-visible
    /// (a store the hash cannot see is a III.7 hole) — and the fifth
    /// section trails the four version-1 sections rather than inserting
    /// among them.
    #[test]
    fn a_single_edge_attribute_moves_the_hash_and_trails_the_four_sections() {
        let bare = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![],
            currency_attributes: vec![],
            hyperedge_attributes: Vec::new(),
        };
        let with_attribute = Facts {
            edge_attributes: vec![("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)],
            ..Facts {
                nodes: bare.nodes.clone(),
                attributes: bare.attributes.clone(),
                edges: bare.edges.clone(),
                hyperedges: bare.hyperedges.clone(),
                edge_attributes: vec![],
                currency_attributes: vec![],
                hyperedge_attributes: Vec::new(),
            }
        };

        assert_ne!(
            bare.state_hash().unwrap(),
            with_attribute.state_hash().unwrap(),
            "one stored edge attribute must move the hash"
        );
        let bare_encoding = bare.encode_state().unwrap();
        let with_encoding = with_attribute.encode_state().unwrap();
        let bare_bytes = bare_encoding.as_bytes();
        let with_bytes = with_encoding.as_bytes();
        assert_eq!(
            &with_bytes[..bare_bytes.len()],
            bare_bytes,
            "the four version-1 sections are an untouched PREFIX — 0x05 appends, never inserts"
        );
        assert_eq!(
            with_bytes[bare_bytes.len()],
            0x05,
            "the fifth section's tag immediately follows the fourth's last byte"
        );
    }

    /// The `0x06` analogue of `an_empty_edge_attribute_listing_writes_no_fifth_section_bytes`
    /// (T3 #491, OQ-J): the provided `encode_state` over a store reporting
    /// NO Currency attributes produces exactly the bytes a hand-driven
    /// five-section encoder produces — no `0x06` tag, no zero count,
    /// nothing. (If the elision branch is deleted, this flips red.)
    #[test]
    fn an_empty_currency_attribute_listing_writes_no_sixth_section_bytes() {
        let facts = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)],
            currency_attributes: vec![],
            hyperedge_attributes: Vec::new(),
        };
        let mut manual = StateEncoder::new();
        manual.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        manual
            .write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        manual
            .write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        manual
            .write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();
        manual
            .write_edge_attributes(&[("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)])
            .unwrap();
        assert_eq!(
            facts.encode_state().unwrap().as_bytes(),
            manual.as_bytes(),
            "an empty sixth listing must contribute ZERO bytes (the 0x05 elision precedent, \
             extended to 0x06)"
        );
    }

    /// The `0x06` analogue of `a_single_edge_attribute_moves_the_hash_and_trails_the_four_sections`
    /// (T3 #491, OQ-J): ONE stored Currency attribute must be hash-visible,
    /// and the sixth section trails the five prior sections rather than
    /// inserting among them.
    #[test]
    fn a_single_currency_attribute_moves_the_hash_and_trails_the_five_sections() {
        let bare = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)],
            currency_attributes: vec![],
            hyperedge_attributes: Vec::new(),
        };
        let with_attribute = Facts {
            currency_attributes: vec![(NodeId(1), "m".to_owned(), Currency::from_micro_units(1))],
            ..Facts {
                nodes: bare.nodes.clone(),
                attributes: bare.attributes.clone(),
                edges: bare.edges.clone(),
                hyperedges: bare.hyperedges.clone(),
                edge_attributes: bare.edge_attributes.clone(),
                currency_attributes: vec![],
                hyperedge_attributes: Vec::new(),
            }
        };

        assert_ne!(
            bare.state_hash().unwrap(),
            with_attribute.state_hash().unwrap(),
            "one stored Currency attribute must move the hash"
        );
        let bare_encoding = bare.encode_state().unwrap();
        let with_encoding = with_attribute.encode_state().unwrap();
        let bare_bytes = bare_encoding.as_bytes();
        let with_bytes = with_encoding.as_bytes();
        assert_eq!(
            &with_bytes[..bare_bytes.len()],
            bare_bytes,
            "the five prior sections are an untouched PREFIX — 0x06 appends, never inserts"
        );
        assert_eq!(
            with_bytes[bare_bytes.len()],
            0x06,
            "the sixth section's tag immediately follows the fifth's last byte"
        );
    }

    /// The `0x07` analogue of `an_empty_edge_attribute_listing_writes_no_
    /// fifth_section_bytes` / its `0x06` sibling (Community port train,
    /// Task 5, E2a): the provided `encode_state` over a store reporting NO
    /// hyperedge attributes produces exactly the bytes a hand-driven
    /// six-section encoder produces — no `0x07` tag, no zero count,
    /// nothing. (If the elision branch is deleted, this flips red.) The
    /// landed tick pins (19, `tick_goldens.rs`) staying byte-identical are
    /// the third, decisive proof at the whole-estate level.
    #[test]
    fn an_empty_hyperedge_attribute_listing_writes_no_seventh_section_bytes() {
        let facts = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)],
            currency_attributes: vec![(NodeId(1), "m".to_owned(), Currency::from_micro_units(1))],
            hyperedge_attributes: Vec::new(),
        };
        let mut manual = StateEncoder::new();
        manual.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        manual
            .write_attributes(&[(NodeId(1), "w".to_owned(), 1.0)])
            .unwrap();
        manual
            .write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
            .unwrap();
        manual
            .write_hyperedges(&[(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])])
            .unwrap();
        manual
            .write_edge_attributes(&[("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)])
            .unwrap();
        manual
            .write_currency_attributes(&[(
                NodeId(1),
                "m".to_owned(),
                Currency::from_micro_units(1),
            )])
            .unwrap();
        assert_eq!(
            facts.encode_state().unwrap().as_bytes(),
            manual.as_bytes(),
            "an empty seventh listing must contribute ZERO bytes (the 0x05/0x06 elision \
             precedent, extended to 0x07)"
        );
    }

    /// The `0x07` analogue of `a_single_currency_attribute_moves_the_hash_
    /// and_trails_the_five_sections` (Task 5): ONE stored hyperedge
    /// attribute must be hash-visible, and the seventh section trails the
    /// six prior sections rather than inserting among them.
    #[test]
    fn a_single_hyperedge_attribute_moves_the_hash_and_trails_the_six_sections() {
        let bare = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![(NodeId(1), "w".to_owned(), 1.0)],
            edges: vec![("E".to_owned(), NodeId(1), NodeId(2), 0.5)],
            hyperedges: vec![(HyperedgeId(7), "H".to_owned(), vec![NodeId(1), NodeId(2)])],
            edge_attributes: vec![("E".to_owned(), NodeId(1), NodeId(2), "t".to_owned(), 0.25)],
            currency_attributes: vec![],
            hyperedge_attributes: Vec::new(),
        };
        let with_attribute = Facts {
            hyperedge_attributes: vec![(HyperedgeId(7), "community/heat".to_owned(), 0.5)],
            ..Facts {
                nodes: bare.nodes.clone(),
                attributes: bare.attributes.clone(),
                edges: bare.edges.clone(),
                hyperedges: bare.hyperedges.clone(),
                edge_attributes: bare.edge_attributes.clone(),
                currency_attributes: vec![],
                hyperedge_attributes: Vec::new(),
            }
        };

        assert_ne!(
            bare.state_hash().unwrap(),
            with_attribute.state_hash().unwrap(),
            "one stored hyperedge attribute must move the hash"
        );
        let bare_encoding = bare.encode_state().unwrap();
        let with_encoding = with_attribute.encode_state().unwrap();
        let bare_bytes = bare_encoding.as_bytes();
        let with_bytes = with_encoding.as_bytes();
        assert_eq!(
            &with_bytes[..bare_bytes.len()],
            bare_bytes,
            "the six prior sections are an untouched PREFIX — 0x07 appends, never inserts"
        );
        assert_eq!(
            with_bytes[bare_bytes.len()],
            0x07,
            "the seventh section's tag immediately follows the sixth's last byte"
        );
    }

    /// The seventh section sorts by `(id, qname)` inside `encode_state`,
    /// never trusting the listing's own order (Task 5) — two stores
    /// reporting the same rows in different orders hash identically, and
    /// the sort-key mutation (Step 6) reds this.
    #[test]
    fn the_seventh_section_sorts_by_id_then_qname_regardless_of_listing_order() {
        let base = Facts {
            nodes: vec![(NodeId(1), "c".to_owned())],
            attributes: vec![],
            edges: vec![],
            hyperedges: vec![
                (HyperedgeId(3), "H".to_owned(), vec![NodeId(1)]),
                (HyperedgeId(7), "H".to_owned(), vec![NodeId(1)]),
            ],
            edge_attributes: vec![],
            currency_attributes: vec![],
            hyperedge_attributes: vec![
                (HyperedgeId(7), "community/heat".to_owned(), 0.5),
                (HyperedgeId(3), "community/zeta".to_owned(), 0.75),
            ],
        };
        let reversed = Facts {
            hyperedge_attributes: vec![
                (HyperedgeId(3), "community/zeta".to_owned(), 0.75),
                (HyperedgeId(7), "community/heat".to_owned(), 0.5),
            ],
            ..Facts {
                nodes: base.nodes.clone(),
                attributes: vec![],
                edges: vec![],
                hyperedges: base.hyperedges.clone(),
                edge_attributes: vec![],
                currency_attributes: vec![],
                hyperedge_attributes: Vec::new(),
            }
        };
        assert_eq!(
            base.state_hash().unwrap(),
            reversed.state_hash().unwrap(),
            "listing order is never observable — the encoder sorts by (id, qname)"
        );
        // …and the order IS pinned against a hand-driven encoder, so a
        // sort-KEY mutation (not just a dropped sort) is visible: correct
        // is id-then-qname, (3, zeta) before (7, heat); a qname-only sort
        // would write (7, heat) first ("heat" < "zeta") and red this.
        let mut manual = StateEncoder::new();
        manual.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
        manual.write_attributes(&[]).unwrap();
        manual.write_edges(&[]).unwrap();
        manual
            .write_hyperedges(&[
                (HyperedgeId(3), "H".to_owned(), vec![NodeId(1)]),
                (HyperedgeId(7), "H".to_owned(), vec![NodeId(1)]),
            ])
            .unwrap();
        manual
            .write_hyperedge_attributes(&[
                (HyperedgeId(3), "community/zeta".to_owned(), 0.75),
                (HyperedgeId(7), "community/heat".to_owned(), 0.5),
            ])
            .unwrap();
        assert_eq!(
            base.encode_state().unwrap().as_bytes(),
            manual.as_bytes(),
            "the seventh section writes id-then-qname order, pinned"
        );
    }

    /// The layout version constant is the minted versioning convention
    /// (ADR198 R2: the elision rule is "documented and versioned" — there
    /// was no version anchor before T3, so T3 minted one). Pinning the
    /// value is what makes a future bump a deliberate, reviewable act
    /// rather than a silent edit.
    #[test]
    fn the_canonical_layout_version_is_pinned() {
        assert_eq!(
            super::CANONICAL_LAYOUT_VERSION,
            4,
            "layout version 4 = version 3 (sections 0x01-0x06) + section 0x07 (hyperedge \
             own-field attributes, elided when empty, Community port train Task 5/E2a) — \
             bumping this is a contract event, see the module doc's 'Layout versions'"
        );
    }

    #[test]
    fn negative_zero_in_an_edge_attribute_hashes_as_positive_zero() {
        // The fifth section inherits the float discipline wholesale: a sign
        // bit arriving from upstream arithmetic must not move the hash.
        let encode = |value: f64| {
            let mut enc = StateEncoder::new();
            enc.write_nodes(&[(NodeId(1), "c".to_owned())]).unwrap();
            enc.write_attributes(&[]).unwrap();
            enc.write_edges(&[("E".to_owned(), NodeId(1), NodeId(2), 0.5)])
                .unwrap();
            enc.write_hyperedges(&[]).unwrap();
            enc.write_edge_attributes(&[(
                "E".to_owned(),
                NodeId(1),
                NodeId(2),
                "t".to_owned(),
                value,
            )])
            .unwrap();
            enc.finish()
        };
        assert!(
            (-0.0_f64).is_sign_negative(),
            "fixture must carry the sign bit"
        );
        assert_eq!(encode(-0.0), encode(0.0));
    }

    #[test]
    fn a_non_finite_edge_attribute_is_refused_never_hashed() {
        let mut enc = StateEncoder::new();
        let err = enc
            .write_edge_attributes(&[(
                "E".to_owned(),
                NodeId(1),
                NodeId(2),
                "t".to_owned(),
                f64::NAN,
            )])
            .unwrap_err();
        assert!(
            err.message.contains("could not be reproduced"),
            "{}",
            err.message
        );
    }

    #[test]
    fn the_same_state_hashes_identically() {
        assert_eq!(
            encoder_with_one_attribute(42.0),
            encoder_with_one_attribute(42.0)
        );
    }

    #[test]
    fn a_changed_value_changes_the_hash() {
        assert_ne!(
            encoder_with_one_attribute(42.0),
            encoder_with_one_attribute(42.5)
        );
    }

    #[test]
    fn negative_zero_hashes_as_positive_zero() {
        // Otherwise a sign bit from upstream arithmetic (-1.0 * 0.0) would
        // change the tick hash without changing the world.
        assert!(
            (-0.0_f64).is_sign_negative(),
            "fixture must carry the sign bit"
        );
        assert_eq!(
            encoder_with_one_attribute(-0.0),
            encoder_with_one_attribute(0.0)
        );
    }

    #[test]
    fn a_non_finite_value_is_refused_never_hashed() {
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "social_class".to_owned())])
            .unwrap();
        let err = enc
            .write_attributes(&[(NodeId(1), "wealth".to_owned(), f64::NAN)])
            .unwrap_err();
        assert!(
            err.message.contains("could not be reproduced"),
            "{}",
            err.message
        );
    }

    #[test]
    fn section_tags_keep_shapes_distinct() {
        // One node and no edges must not collide with no nodes and one edge.
        let mut a = StateEncoder::new();
        a.write_nodes(&[(NodeId(1), "x".to_owned())]).unwrap();
        a.write_attributes(&[]).unwrap();
        a.write_edges(&[]).unwrap();
        a.write_hyperedges(&[]).unwrap();

        let mut b = StateEncoder::new();
        b.write_nodes(&[]).unwrap();
        b.write_attributes(&[]).unwrap();
        b.write_edges(&[("x".to_owned(), NodeId(1), NodeId(1), 1.0)])
            .unwrap();
        b.write_hyperedges(&[]).unwrap();

        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn a_hyperedge_of_the_same_members_under_a_different_id_differs() {
        // Hyperedge identity is its own, not a function of its membership.
        // Written as a CANONICAL encoding — all four sections, in tag order,
        // empty where empty. A partial encoding would be comparing two
        // shapes the format does not define, so it could pass while the real
        // encoding was broken.
        let nodes = vec![
            (NodeId(1), "social_class".to_owned()),
            (NodeId(2), "social_class".to_owned()),
        ];
        let members = vec![NodeId(1), NodeId(2)];

        let mut a = StateEncoder::new();
        a.write_nodes(&nodes).unwrap();
        a.write_attributes(&[]).unwrap();
        a.write_edges(&[]).unwrap();
        a.write_hyperedges(&[(HyperedgeId(0), "sector".to_owned(), members.clone())])
            .unwrap();

        let mut b = StateEncoder::new();
        b.write_nodes(&nodes).unwrap();
        b.write_attributes(&[]).unwrap();
        b.write_edges(&[]).unwrap();
        b.write_hyperedges(&[(HyperedgeId(1), "sector".to_owned(), members)])
            .unwrap();

        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn the_encoding_is_inspectable_for_a_differential() {
        // A bare hash says only THAT two states differ; the bytes say where.
        // Deliberately partial: this asserts the LEADING bytes of section
        // 0x01, so it stops at the first section on purpose.
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(1), "social_class".to_owned())])
            .unwrap();
        let bytes = enc.as_bytes();
        assert_eq!(bytes[0], 0x01, "section tag leads");
        assert_eq!(
            &bytes[1..5],
            &1_u32.to_be_bytes(),
            "then a big-endian count"
        );
        assert_eq!(&bytes[5..13], &1_u64.to_be_bytes(), "then the node id");
    }
}
