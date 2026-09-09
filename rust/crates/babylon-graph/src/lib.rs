//! The graph substrate crate (spec §6, crate table). **Amendment D ruled
//! NATIVE HYPEREDGE** (2026-07-29; Amendment AE clause (vi),
//! `ai/_inbox/amendment-d-analysis-p27.md` §9): hyperedges are first-class
//! objects in this crate's exposed model and type system — membership is one
//! typed hyperedge, never a clique expansion, and never *exposed* as a
//! bipartite incidence encoding. Levi/incidence is a permitted INTERNAL
//! storage strategy (what `hypergraph-rs` implements); nothing in the exposed
//! API reveals it. The strictly dyadic morphism API (II.9) lives alongside it
//! in the same trait, separated by type (D-2).
//!
//! This crate exposes the [`substrate::GraphSubstrate`] trait, plus two
//! implementations. **[`hypergraph_store::HypergraphStore`] is the store
//! production logic runs against** (ADR179 T3, executed by ADR193, PR #494,
//! 2026-08-11): `babylon-tick::run_once` — and therefore `babylon-client`'s
//! engine-link path, the one production consumer — constructs it. It
//! delegates the native-hyperedge half to the sibling `hypergraph-rs`
//! library (the Levi/incidence encoding this crate's exposed model permits
//! as an INTERNAL storage strategy and forbids exposing, D-1) behind the
//! adapter covenants `docs/reference/graph-storage-capability-delta.md` §8
//! enumerates; the dyadic half stays native maps. [`memory::MemoryGraph`]
//! (Director ruling 2026-07-31; P27 Phase 2 Slice 1) is kept — not deleted —
//! as the crate's differential oracle (`tests/differential.rs`, byte-level,
//! operation-by-operation) and as the reference implementation
//! [`conformance::run_substrate_conformance`] is written against; both
//! stores pass that same suite.
//!
//! **The trait is the insulation, not either store.** `GraphSubstrate` is
//! unwidened by the swap — 14 methods, exactly as ratified — and a sibling
//! trait, [`state_hash::CanonicalState`], carries the one shared canonical
//! encoding both stores implement. Depend on `GraphSubstrate` (+
//! `CanonicalState` where the canonical byte encoding is needed); construct
//! whichever store the call site needs (production: `HypergraphStore`;
//! tests wanting the reference implementation or an oracle: `MemoryGraph`).
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod allocator_state;
pub mod backfire;
pub mod capacity;
pub mod conformance;
pub mod dossier;
pub mod exposure;
pub mod hypergraph_store;
pub mod induced;
pub mod memory;
pub mod stable_element;
pub mod stable_state;
pub mod state_hash;
pub mod substrate;
pub mod working_copy;
