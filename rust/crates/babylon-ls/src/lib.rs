//! `babylon-ls` — the agent-first Language Server Protocol front end for
//! `.bsl`/`.bscn` (issue #652, wave 1).
//!
//! **Non-normative, observes-only** (global constraint 1,
//! `docs/superpowers/plans/2026-08-17-652-bsl-ls.md` §1): this crate holds
//! no state the loader (`babylon-bsl`/`babylon-tick`) does not, never
//! writes content, and is never a gate — a file this server likes may
//! still be refused at load, and the load path stays the only door
//! (#533's load-ceremony rule).
//!
//! Task 5 ships the protocol skeleton only: `initialize`/`initialized`
//! lifecycle, document sync (`didOpen`/`didChange`/`didClose`),
//! `shutdown`/`exit`, an in-memory document store, and byte<->UTF-16
//! position mapping. No diagnostics flow yet (Task 6), no manifest
//! discovery yet (Task 5.4), no size-rotated log sink yet (Task 5.5) —
//! this crate logs to stderr via `eprintln!` until that lands.
//!
//! **Layering** (plan §5.1): `babylon-ls` is a leaf ABOVE the whole engine
//! stack — `kernel` < `models`/`formulas` < `topology` < `domain` <
//! `persistence` < `engine`, with `intelligence` (ai + rag) observing
//! separately. Nothing in that stack may depend on `babylon-ls`; Task 7's
//! sentinel 7.5 holds that boundary because Rust has no `lint:imports`
//! equivalent to check it automatically. Task 5 itself depends on nothing
//! from that stack yet — `babylon-bsl`/`babylon-tick`/`babylon-graph`
//! become real dependencies only when Task 5.4 (manifest loading) and
//! Task 6 (diagnostics) need them.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod authoring;
pub mod capabilities;
pub mod content_manifest;
pub mod diagnostics;
pub mod document_store;
pub mod lifecycle;
pub mod line_index;
pub mod locator;
pub mod pass;
pub mod uri;

pub use lifecycle::serve;
