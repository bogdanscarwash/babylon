//! Pinned glossary concept fixture corpus (ADR249 R12).
//!
//! The eight-concept fixture seeds foundation knowledge grants for glossary
//! subjects and pins the canonical display labels the dossier surface
//! renders. The exact file bytes are SHA-256 pinned like every other
//! reference artifact; the semantic digest covers the canonical
//! per-concept `(concept_id, display_label, definition)` encoding pinned by
//! `contracts/glossary_concepts_v1.yaml`.

use std::sync::OnceLock;

use babylon_kernel::sha256_of;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use crate::archive::ArchiveCitationV1;

/// Repository-relative fixture path pinned by the contract.
pub const GLOSSARY_CONCEPTS_FIXTURE_PATH_V1: &str = "contracts/fixtures/glossary_concepts_v1.jsonl";
/// Contract-pinned SHA-256 of the exact glossary fixture bytes
/// (`contracts/glossary_concepts_v1.yaml`).
pub const PINNED_GLOSSARY_CONCEPTS_SHA256_V1: [u8; 32] = [
    0xf4, 0x7e, 0x28, 0x9d, 0xc4, 0xe7, 0xa1, 0x1c, 0x59, 0x5f, 0x0e, 0x42, 0x64, 0x3e, 0x35, 0x2e,
    0x25, 0x57, 0x75, 0xc7, 0x7d, 0xde, 0x3a, 0x7e, 0xd3, 0x5a, 0x91, 0xde, 0x8d, 0x84, 0xd8, 0x5a,
];

const GLOSSARY_CONCEPTS_DOMAIN_V1: &[u8] = b"babylon.glossary-concepts.v1\0";
const MAX_CONCEPT_ROWS: usize = 64;
const MAX_CONCEPT_ID_BYTES: usize = 128;
const MAX_DISPLAY_LABEL_BYTES: usize = 256;
const MAX_DEFINITION_BYTES: usize = 4_096;
const MAX_CITATION_BYTES: usize = 4_096;

const FIXTURE: &str = include_str!("../../../../contracts/fixtures/glossary_concepts_v1.jsonl");

#[derive(Deserialize)]
struct RawGlossaryConceptV1 {
    concept_id: String,
    term: String,
    display_label: String,
    definition: String,
    #[allow(dead_code)]
    evidence_class: String,
    citation: RawGlossaryCitationV1,
}

#[derive(Deserialize)]
struct RawGlossaryCitationV1 {
    source_id: String,
    locator: String,
}

/// One bounded, validated glossary concept row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlossaryConceptV1 {
    concept_id: String,
    display_label: String,
    definition: String,
    citation: ArchiveCitationV1,
}

impl GlossaryConceptV1 {
    /// Borrow the stable concept key.
    #[must_use]
    pub fn concept_id(&self) -> &str {
        &self.concept_id
    }

    /// Borrow the canonical player-facing display label.
    #[must_use]
    pub fn display_label(&self) -> &str {
        &self.display_label
    }

    /// Borrow the bounded glossary definition.
    #[must_use]
    pub fn definition(&self) -> &str {
        &self.definition
    }

    /// Borrow the pinned fixture citation.
    #[must_use]
    pub const fn citation(&self) -> &ArchiveCitationV1 {
        &self.citation
    }
}

/// Closed refusal taxonomy for the pinned glossary concept corpus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlossaryConceptsErrorV1 {
    /// The checked-in fixture bytes drifted from the contract pin.
    FixtureDigest,
    /// One fixture line was empty, malformed JSON, or the wrong shape.
    FixtureShape {
        /// One-based fixture line number.
        line: usize,
    },
    /// One concept id was empty, unbounded, or outside the key grammar.
    ConceptIdShape {
        /// One-based fixture line number.
        line: usize,
    },
    /// One bounded text field was empty, NUL-containing, or oversized.
    TextBounds {
        /// One-based fixture line number.
        line: usize,
        /// Stable field identity.
        field: &'static str,
    },
    /// One citation failed Archive citation validation.
    InvalidCitation {
        /// One-based fixture line number.
        line: usize,
    },
    /// Two fixture rows repeated one concept id.
    DuplicateConceptId {
        /// One-based fixture line number.
        line: usize,
    },
    /// Fixture rows were not sorted by concept id in ascending order.
    Unsorted,
}

impl std::fmt::Display for GlossaryConceptsErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "glossary concepts refusal: {self:?}")
    }
}

impl std::error::Error for GlossaryConceptsErrorV1 {}

/// The validated, immutable glossary concept corpus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlossaryConceptsV1 {
    concepts: Box<[GlossaryConceptV1]>,
}

impl GlossaryConceptsV1 {
    /// Borrow the ordered concept rows.
    #[must_use]
    pub fn concepts(&self) -> &[GlossaryConceptV1] {
        &self.concepts
    }

    /// Hash the canonical per-concept encoding pinned by the contract.
    #[must_use]
    pub fn semantic_sha256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(GLOSSARY_CONCEPTS_DOMAIN_V1);
        hash_len(&mut hasher, self.concepts.len());
        for concept in &self.concepts {
            hash_text(&mut hasher, &concept.concept_id);
            hash_text(&mut hasher, &concept.display_label);
            hash_text(&mut hasher, &concept.definition);
        }
        hasher.finalize().into()
    }
}

fn hash_len(hasher: &mut Sha256, len: usize) {
    hasher.update((len as u64).to_be_bytes());
}

fn hash_text(hasher: &mut Sha256, text: &str) {
    hash_len(hasher, text.len());
    hasher.update(text.as_bytes());
}

fn bounded_text(
    value: &str,
    line: usize,
    field: &'static str,
    maximum: usize,
) -> Result<(), GlossaryConceptsErrorV1> {
    if value.is_empty() || value.len() > maximum || value.as_bytes().contains(&0) {
        return Err(GlossaryConceptsErrorV1::TextBounds { line, field });
    }
    Ok(())
}

fn valid_concept_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn parse_fixture() -> Result<GlossaryConceptsV1, GlossaryConceptsErrorV1> {
    if sha256_of(FIXTURE.as_bytes()) != PINNED_GLOSSARY_CONCEPTS_SHA256_V1 {
        return Err(GlossaryConceptsErrorV1::FixtureDigest);
    }
    let mut concepts = Vec::new();
    for (index, line) in FIXTURE.lines().enumerate() {
        let line_number = index + 1;
        if line.is_empty() {
            return Err(GlossaryConceptsErrorV1::FixtureShape { line: line_number });
        }
        let raw: RawGlossaryConceptV1 = serde_json::from_str(line)
            .map_err(|_| GlossaryConceptsErrorV1::FixtureShape { line: line_number })?;
        if !valid_concept_id(&raw.concept_id) || raw.concept_id.len() > MAX_CONCEPT_ID_BYTES {
            return Err(GlossaryConceptsErrorV1::ConceptIdShape { line: line_number });
        }
        bounded_text(&raw.term, line_number, "term", MAX_DISPLAY_LABEL_BYTES)?;
        bounded_text(
            &raw.display_label,
            line_number,
            "display_label",
            MAX_DISPLAY_LABEL_BYTES,
        )?;
        bounded_text(
            &raw.definition,
            line_number,
            "definition",
            MAX_DEFINITION_BYTES,
        )?;
        let citation = ArchiveCitationV1::try_new(raw.citation.source_id, raw.citation.locator)
            .map_err(|_| GlossaryConceptsErrorV1::InvalidCitation { line: line_number })?;
        bounded_text(
            citation.source_id(),
            line_number,
            "citation.source_id",
            MAX_CITATION_BYTES,
        )?;
        bounded_text(
            citation.locator(),
            line_number,
            "citation.locator",
            MAX_CITATION_BYTES,
        )?;
        if concepts
            .iter()
            .any(|prior: &GlossaryConceptV1| prior.concept_id == raw.concept_id)
        {
            return Err(GlossaryConceptsErrorV1::DuplicateConceptId { line: line_number });
        }
        match concepts.last() {
            Some(prior) if prior.concept_id > raw.concept_id => {
                return Err(GlossaryConceptsErrorV1::Unsorted);
            }
            _ => {}
        }
        concepts.push(GlossaryConceptV1 {
            concept_id: raw.concept_id,
            display_label: raw.display_label,
            definition: raw.definition,
            citation,
        });
        if concepts.len() > MAX_CONCEPT_ROWS {
            return Err(GlossaryConceptsErrorV1::FixtureShape { line: line_number });
        }
    }
    Ok(GlossaryConceptsV1 {
        concepts: concepts.into_boxed_slice(),
    })
}

/// Return the sole parsed glossary concept corpus after validating its bytes.
///
/// The fixture parses at most once; every caller receives the same immutable
/// corpus and therefore cannot introduce a second parser or source identity.
///
/// # Errors
/// Returns [`GlossaryConceptsErrorV1`] when the pinned digest, row shape,
/// key grammar, text bounds, citation, order, or uniqueness drift.
pub fn glossary_concepts_v1() -> Result<&'static GlossaryConceptsV1, GlossaryConceptsErrorV1> {
    static CONCEPTS: OnceLock<Result<GlossaryConceptsV1, GlossaryConceptsErrorV1>> =
        OnceLock::new();
    CONCEPTS
        .get_or_init(parse_fixture)
        .as_ref()
        .map_err(Clone::clone)
}
