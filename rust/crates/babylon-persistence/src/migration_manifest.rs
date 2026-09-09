//! Bounded, language-neutral migration-set framing.

use babylon_kernel::sha256_of;

use crate::MigrationSetDigest;

/// Shared `PostgreSQL` session advisory-lock key inherited from the Python writer.
pub const SCHEMA_ADVISORY_LOCK_KEY: i64 = 0xBAB1_0537;
/// Hard ceiling on chunks in one manifest.
pub const MAX_MANIFEST_CHUNKS: usize = 256;
/// Hard ceiling on the full NUL-framed byte sequence.
pub const MAX_MANIFEST_BYTES: usize = 1_048_576;

/// Invalid migration manifest input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The stable manifest name was empty.
    EmptyName,
    /// The manifest carried no chunks.
    EmptySet,
    /// The manifest exceeded its fixed chunk ceiling.
    TooManyChunks { actual: usize, max: usize },
    /// The framed representation exceeded its fixed byte ceiling.
    TooManyBytes { actual: usize, max: usize },
    /// One source chunk was empty.
    EmptyChunk { index: usize },
    /// One source chunk contained the framing delimiter.
    EmbeddedNul { index: usize },
    /// A serialized manifest omitted the final delimiter.
    MissingTrailingNul,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid migration manifest: {self:?}")
    }
}

impl std::error::Error for ManifestError {}

/// Validated identity of one ordered migration sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationManifest {
    name: &'static str,
    chunk_count: usize,
    digest: MigrationSetDigest,
}

impl MigrationManifest {
    /// Validate and hash ordered raw chunks.
    ///
    /// # Errors
    /// Returns [`ManifestError`] for an empty, malformed, or unbounded set.
    pub fn from_chunks(name: &'static str, chunks: &[&[u8]]) -> Result<Self, ManifestError> {
        validate_header(name, chunks.len())?;
        let framed_len = framed_len(chunks)?;
        let mut framed = Vec::with_capacity(framed_len);
        for chunk in chunks.iter().take(MAX_MANIFEST_CHUNKS) {
            framed.extend_from_slice(chunk);
            framed.push(0);
        }
        Ok(Self {
            name,
            chunk_count: chunks.len(),
            digest: MigrationSetDigest::from_bytes(sha256_of(&framed)),
        })
    }

    /// Parse a sequence in which every chunk, including the last, ends in NUL.
    ///
    /// # Errors
    /// Returns [`ManifestError`] for missing framing or an unbounded set.
    pub fn from_nul_framed(name: &'static str, bytes: &[u8]) -> Result<Self, ManifestError> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(ManifestError::TooManyBytes {
                actual: bytes.len(),
                max: MAX_MANIFEST_BYTES,
            });
        }
        if bytes.last() != Some(&0) {
            return Err(ManifestError::MissingTrailingNul);
        }
        let chunks: Vec<&[u8]> = bytes[..bytes.len() - 1]
            .split(|byte| *byte == 0)
            .take(MAX_MANIFEST_CHUNKS + 1)
            .collect();
        Self::from_chunks(name, &chunks)
    }

    /// Stable manifest name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Number of ordered chunks.
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Ordered-NUL SHA-256 digest.
    #[must_use]
    pub fn digest(&self) -> MigrationSetDigest {
        self.digest
    }
}

fn validate_header(name: &str, chunk_count: usize) -> Result<(), ManifestError> {
    if name.is_empty() {
        return Err(ManifestError::EmptyName);
    }
    if chunk_count == 0 {
        return Err(ManifestError::EmptySet);
    }
    if chunk_count > MAX_MANIFEST_CHUNKS {
        return Err(ManifestError::TooManyChunks {
            actual: chunk_count,
            max: MAX_MANIFEST_CHUNKS,
        });
    }
    Ok(())
}

fn framed_len(chunks: &[&[u8]]) -> Result<usize, ManifestError> {
    let mut total = 0_usize;
    for (index, chunk) in chunks.iter().enumerate().take(MAX_MANIFEST_CHUNKS) {
        if chunk.is_empty() {
            return Err(ManifestError::EmptyChunk { index });
        }
        total = total
            .checked_add(chunk.len())
            .and_then(|length| length.checked_add(1))
            .ok_or(ManifestError::TooManyBytes {
                actual: usize::MAX,
                max: MAX_MANIFEST_BYTES,
            })?;
        if total > MAX_MANIFEST_BYTES {
            return Err(ManifestError::TooManyBytes {
                actual: total,
                max: MAX_MANIFEST_BYTES,
            });
        }
        if chunk.contains(&0) {
            return Err(ManifestError::EmbeddedNul { index });
        }
    }
    Ok(total)
}
