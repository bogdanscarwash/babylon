//! The Babylon Scripting Language (BSL): the reader, typechecker, load-time
//! bound checker, and fuel evaluator (spec §5). No `unsafe`.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod bindings;
pub mod bound_checker;
pub mod canonical_ast;
pub mod causal_contract;
pub mod declarations;
pub mod default_lint;
pub mod domain;
pub mod error_identity;
pub mod evaluator;
pub mod exemptions;
pub mod fuel;
pub mod grammar;
pub mod identity_codec;
pub mod identity_sections;
pub mod intrinsic_host;
pub mod manifest;
pub mod material_basis;
pub mod metrics;
pub mod mod_anchors;
pub mod probability;
pub mod query;
pub mod reader;
pub mod rule_pipeline;
pub mod same_tick_order;
pub mod scenario;
pub mod scope;
pub mod score_class;
pub mod sfs_profile;
pub mod structural_verbs;
pub mod tick;
pub mod typecheck;
pub mod types;
pub mod vocabulary;
pub mod write_log;

// `diagnose` re-exported under a qualified name (W2 fix round 1, review
// Minor 4): `babylon_bsl::diagnose` unqualified reads as "diagnose
// anything" at the crate root, alongside dozens of other checkers this
// crate could plausibly want the bare name for later. No caller outside
// `same_tick_order`'s own module uses the unqualified re-export today
// (checked: `rg -n 'babylon_bsl::diagnose\b'` across `rust/`, zero hits
// besides this file), so the rename is free.
