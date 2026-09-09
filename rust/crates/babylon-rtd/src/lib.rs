#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

// Schema literals retain the language-neutral contract spelling.
mod canonical;
#[allow(clippy::unreadable_literal)]
mod schema;
mod validate;

pub use canonical::{
    canonical_draft_bytes, parse_vector_corpus, projection_hash, seal_draft, RtdVectorCaseV1,
};
pub use schema::*;
pub use validate::{append_bounded, parse_draft_json, validate_draft, RtdError};

/// SHA-256 of the exact language-neutral V1 dossier schema bytes.
pub const RTD_CONTRACT_SOURCE_SHA256: [u8; 32] = [
    0x5f, 0x0e, 0x27, 0x1d, 0x46, 0x78, 0x3b, 0xd8, 0x2f, 0xb5, 0xc9, 0x33, 0x6c, 0x46, 0x6f, 0x4c,
    0x36, 0x31, 0xa4, 0x99, 0xb4, 0x3c, 0x83, 0xc1, 0x1b, 0x85, 0x4d, 0xb2, 0x3e, 0xa5, 0x9e, 0x40,
];
