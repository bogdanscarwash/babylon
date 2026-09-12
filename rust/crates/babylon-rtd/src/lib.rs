#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

// Schema literals retain the language-neutral contract spelling.
mod canonical;
#[allow(clippy::unreadable_literal)]
mod schema;
mod validate;

pub use canonical::{
    canonical_draft_bytes, parse_vector_corpus, projection_hash, seal_draft, RtdVectorCase,
};
pub use schema::*;
pub use validate::{append_bounded, parse_draft_json, validate_draft, RtdError};

/// SHA-256 of the exact language-neutral V1 dossier schema bytes.
pub const RTD_CONTRACT_SOURCE_SHA256: [u8; 32] = [
    0xa3, 0x55, 0x34, 0xcd, 0xe9, 0xa1, 0xfe, 0x0e, 0x26, 0x4c, 0x84, 0xfd, 0x58, 0xf5, 0x11, 0x76,
    0x61, 0x29, 0xe1, 0x60, 0x1c, 0xb5, 0xd4, 0xaa, 0xe3, 0xa9, 0xb7, 0xa5, 0xe8, 0x36, 0x74, 0x3a,
];
