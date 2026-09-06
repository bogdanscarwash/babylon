//! The county atlas reader (B1 Task 4).
//!
//! `tools/build_county_atlas.py::encode` is the wire format's one writer;
//! this module is its one reader. The format is documented in that tool's
//! module docstring — this file transcribes it, not re-derives it. Layout,
//! byte for byte:
//!
//! ```text
//! header (fixed 128 bytes)
//!   magic        [u8; 8]   b"BABCTY\0\x01"
//!   version      u32       = 1
//!   flags        u32       = 0
//!   content_hash [u8; 32]  sha256 of every byte AFTER this field (bytes[48:])
//!   origin_x     f64       Albers metres of quantization grid origin
//!   origin_y     f64
//!   scale        f64       metres per quantization unit
//!   county_count u32
//!   ring_count   u32
//!   vertex_count u32
//!   csr_nnz      u32       directed adjacency entries
//!   source_hash  [u8; 32]  county_adjacency.json's content_hash
//!   reserved     [u8; 8]   zero-filled padding to 128
//!
//! county table    (county_count x 28 bytes)
//!   fips        [u8; 5]    ASCII, zero-padded
//!   pad         [u8; 1]
//!   ring_start  u32        index into the ring table
//!   ring_count  u16
//!   flags       u16        bit 0 = has adjacency row
//!   bbox        [u16; 4]   min_x, min_y, max_x, max_y in grid units
//!   centroid    [u16; 2]   grid units
//!   pad         [u8; 2]
//! ring table      (ring_count x 12 bytes)
//!   vertex_start u32,  vertex_count u32,  is_hole u8,  pad [u8; 3]
//! vertex array    (vertex_count x 4 bytes)   x u16, y u16
//! csr_offsets     ((county_count + 1) x u32)
//! csr_neighbors   (csr_nnz x u32)            county indices, ascending per row
//! name blob       u32 length, then UTF-8 "<county_name>, <state_abbrev>\n" per county
//! ```
//!
//! **Check-then-decode.** Every count and offset read from the file is
//! validated against the buffer length (or against a table already
//! validated) before any loop walks it — Power-of-10 rule 2: no loop takes
//! its bound from an unchecked number read out of a file. A reader that
//! accepts a corrupted atlas commits the silent-corruption sin
//! Constitution III.11 forbids.

use babylon_kernel::content_digest::sha256_of;
use bevy::math::{Rect, Vec2};

const MAGIC: [u8; 8] = *b"BABCTY\0\x01";
const FORMAT_VERSION: u32 = 1;
const HEADER_BYTES: usize = 128;
const COUNTY_ENTRY_BYTES: usize = 28;
const RING_ENTRY_BYTES: usize = 12;
const VERTEX_ENTRY_BYTES: usize = 4;
const CSR_ENTRY_BYTES: usize = 4;

/// Every way a `CountyAtlas::parse` call can reject its input. Each variant
/// names the specific check that failed — a reader that folds every failure
/// into one opaque error would make the rejection tests (Task 4 Step 1)
/// unable to prove which check actually fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasError {
    /// The buffer ends before a field or table the header's own counts say
    /// must be present — including the final "no trailing bytes" check.
    Truncated,
    /// The 8-byte magic does not read `b"BABCTY\0\x01"`.
    BadMagic,
    /// The format version is not the one this reader understands.
    BadVersion,
    /// `content_hash` does not match `sha256(bytes[48..])` — the file was
    /// hand-edited or half-regenerated.
    HashMismatch,
    /// A county's `ring_start`/`ring_count` names rings past the ring table.
    RingRangeOutOfBounds,
    /// A ring's `vertex_start`/`vertex_count` names vertices past the vertex
    /// array.
    VertexRangeOutOfBounds,
    /// `csr_offsets[i + 1] < csr_offsets[i]` for some row — an offset table
    /// that runs backwards can never describe a real CSR row.
    CsrOffsetsBackwards,
    /// `csr_offsets[county_count] != csr_nnz` — the offset table's own
    /// final entry disagrees with the header's `csr_nnz`. Combined with
    /// the monotonic check above, this is what guarantees every offset
    /// lands in `[0, csr_nnz]`, so `neighbors()` can safely index
    /// `csr_neighbors` without a bounds check at call time.
    CsrTotalMismatch,
    /// A `csr_neighbors` entry names a county index `>= county_count` — it
    /// cannot be a real neighbor if no such county exists.
    CsrNeighborOutOfRange,
    /// The name blob's `\n`-delimited line count does not equal
    /// `county_count` (including: the name blob is not valid UTF-8, which
    /// can never split into the right number of lines either).
    NameCountMismatch,
    /// A county table entry's `fips` field is not valid ASCII/UTF-8 — a
    /// distinct check from `NameCountMismatch`, since a "each variant
    /// names the specific check" reader should not have to guess which of
    /// two unrelated tables actually failed.
    FipsNotUtf8,
}

impl core::fmt::Display for AtlasError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            AtlasError::Truncated => "atlas buffer is shorter than its own header claims",
            AtlasError::BadMagic => "atlas magic bytes do not match BABCTY",
            AtlasError::BadVersion => "atlas format version is not supported by this reader",
            AtlasError::HashMismatch => "atlas content_hash does not match sha256(bytes[48..])",
            AtlasError::RingRangeOutOfBounds => "a county's ring range runs past the ring table",
            AtlasError::VertexRangeOutOfBounds => {
                "a ring's vertex range runs past the vertex array"
            }
            AtlasError::CsrOffsetsBackwards => "a csr_offsets row runs backwards",
            AtlasError::CsrTotalMismatch => "csr_offsets's final entry does not equal csr_nnz",
            AtlasError::CsrNeighborOutOfRange => {
                "a csr_neighbors entry names a county index past county_count"
            }
            AtlasError::NameCountMismatch => {
                "the name blob's line count does not match county_count"
            }
            AtlasError::FipsNotUtf8 => "a county table entry's fips field is not valid UTF-8",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for AtlasError {}

/// A county's bounding box in world metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bbox {
    pub min: Vec2,
    pub max: Vec2,
}

/// One ring (exterior or hole) of a county's polygon, as an index range
/// into `CountyAtlas::vertices()`. Rings carry no lifetime of their own —
/// `CountyAtlas` owns the vertex array they index into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ring {
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub is_hole: bool,
}

/// A borrowed view of one county, built from `CountyAtlas`'s owned arrays.
#[derive(Debug, Clone, Copy)]
pub struct County<'a> {
    pub fips: &'a str,
    pub name: &'a str,
    pub rings: &'a [Ring],
    pub bbox: Bbox,
    pub centroid: Vec2,
}

/// One row of the decoded county table — everything about a county except
/// its rings (which live in `CountyAtlas::rings`, sliced by `ring_start`/
/// `ring_count`) and its vertices (`CountyAtlas::vertices`).
#[derive(Debug, Clone)]
struct CountyRow {
    fips: String,
    name: String,
    ring_start: u32,
    ring_count: u16,
    bbox: Bbox,
    centroid: Vec2,
}

/// The decoded, owned county atlas. `parse` takes `&[u8]` rather than a
/// path so tests can feed it crafted bytes; the returned struct copies
/// everything it needs out of that buffer, so it owns its data and carries
/// no lifetime.
///
/// `Resource` (adversarial-panel fix FB5): `map::mesh::spawn_map_surface`
/// inserts the ONE parse it already does at Startup as this resource, so
/// per-frame `Update` systems (`refresh_hud`, `refresh_state_panel`,
/// `recolor_on_lens_changed`) read it instead of each re-parsing the
/// 1.7 MB embedded atlas — a full SHA-256 hash plus a table decode — on
/// every call. Never re-derive `CountyAtlas::parse(ATLAS_BYTES)` in a
/// system that could instead read `Res<CountyAtlas>`.
#[derive(Debug, bevy::prelude::Resource)]
pub struct CountyAtlas {
    counties: Vec<CountyRow>,
    rings: Vec<Ring>,
    vertices: Vec<Vec2>,
    csr_offsets: Vec<u32>,
    csr_neighbors: Vec<u32>,
    world_bounds: Rect,
}

/// A length-checked read of `len` bytes at `offset`. Every field read in
/// this module goes through this function (or `require_len`) first — the
/// check-then-decode discipline this file's docstring commits to.
fn slice(bytes: &[u8], offset: usize, len: usize) -> Result<&[u8], AtlasError> {
    let end = offset.checked_add(len).ok_or(AtlasError::Truncated)?;
    bytes.get(offset..end).ok_or(AtlasError::Truncated)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, AtlasError> {
    let s = slice(bytes, offset, 4)?;
    Ok(u32::from_le_bytes(
        s.try_into().expect("checked 4-byte slice"),
    ))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, AtlasError> {
    let s = slice(bytes, offset, 2)?;
    Ok(u16::from_le_bytes(
        s.try_into().expect("checked 2-byte slice"),
    ))
}

fn read_u8(bytes: &[u8], offset: usize) -> Result<u8, AtlasError> {
    let s = slice(bytes, offset, 1)?;
    Ok(s[0])
}

fn read_f64(bytes: &[u8], offset: usize) -> Result<f64, AtlasError> {
    let s = slice(bytes, offset, 8)?;
    Ok(f64::from_le_bytes(
        s.try_into().expect("checked 8-byte slice"),
    ))
}

impl CountyAtlas {
    /// Parse a `county_atlas.bin` buffer. Every count and offset is
    /// validated against the buffer's actual length (or against a
    /// previously validated table) before it is used to index anything.
    ///
    /// # Errors
    ///
    /// Returns the specific [`AtlasError`] variant naming whichever
    /// check-then-decode validation (this module's docstring) first fails —
    /// truncation, a bad magic/version/hash, or an out-of-range table
    /// reference. See [`AtlasError`]'s variants for the full list.
    // The check-then-decode discipline this file's docstring commits to
    // reads the format's ~10 sequential sections (header, four tables, name
    // blob) in file order with a validation gate per section; splitting
    // this into sub-functions would scatter that single linear proof across
    // functions without reducing it. Same precedent as
    // `babylon-bsl::scenario::load_scenario_inner`'s
    // `#[allow(clippy::too_many_lines)]`.
    #[allow(clippy::too_many_lines)]
    pub fn parse(bytes: &[u8]) -> Result<Self, AtlasError> {
        // --- Header ---
        let magic = slice(bytes, 0, 8)?;
        if magic != MAGIC {
            return Err(AtlasError::BadMagic);
        }
        let version = read_u32(bytes, 8)?;
        if version != FORMAT_VERSION {
            return Err(AtlasError::BadVersion);
        }
        let content_hash = slice(bytes, 16, 32)?;
        let tail_and_body = slice(bytes, 48, bytes.len() - 48)?;
        let computed = sha256_of(tail_and_body);
        if computed.as_slice() != content_hash {
            return Err(AtlasError::HashMismatch);
        }

        let origin_x = read_f64(bytes, 48)?;
        let origin_y = read_f64(bytes, 56)?;
        let scale = read_f64(bytes, 64)?;
        let county_count = read_u32(bytes, 72)? as usize;
        let ring_count = read_u32(bytes, 76)? as usize;
        let vertex_count = read_u32(bytes, 80)? as usize;
        let csr_nnz = read_u32(bytes, 84)? as usize;
        // source_hash (bytes 88..120) is lineage metadata; this reader does
        // not need it, so it is only length-checked, not stored.
        slice(bytes, 88, 32)?;

        // --- Table geometry, computed and length-checked before any walk ---
        let county_table_off = HEADER_BYTES;
        let county_table_len = county_count
            .checked_mul(COUNTY_ENTRY_BYTES)
            .ok_or(AtlasError::Truncated)?;
        let ring_table_off = county_table_off
            .checked_add(county_table_len)
            .ok_or(AtlasError::Truncated)?;
        let ring_table_len = ring_count
            .checked_mul(RING_ENTRY_BYTES)
            .ok_or(AtlasError::Truncated)?;
        let vertex_table_off = ring_table_off
            .checked_add(ring_table_len)
            .ok_or(AtlasError::Truncated)?;
        let vertex_table_len = vertex_count
            .checked_mul(VERTEX_ENTRY_BYTES)
            .ok_or(AtlasError::Truncated)?;
        let csr_offsets_off = vertex_table_off
            .checked_add(vertex_table_len)
            .ok_or(AtlasError::Truncated)?;
        let csr_offsets_len = (county_count + 1)
            .checked_mul(CSR_ENTRY_BYTES)
            .ok_or(AtlasError::Truncated)?;
        let csr_neighbors_off = csr_offsets_off
            .checked_add(csr_offsets_len)
            .ok_or(AtlasError::Truncated)?;
        let csr_neighbors_len = csr_nnz
            .checked_mul(CSR_ENTRY_BYTES)
            .ok_or(AtlasError::Truncated)?;
        let name_len_off = csr_neighbors_off
            .checked_add(csr_neighbors_len)
            .ok_or(AtlasError::Truncated)?;

        // Confirm every table up to (but not including) the name blob
        // actually fits before decoding any of it.
        slice(bytes, county_table_off, county_table_len)?;
        slice(bytes, ring_table_off, ring_table_len)?;
        slice(bytes, vertex_table_off, vertex_table_len)?;
        slice(bytes, csr_offsets_off, csr_offsets_len)?;
        slice(bytes, csr_neighbors_off, csr_neighbors_len)?;

        let name_len = read_u32(bytes, name_len_off)? as usize;
        let name_blob_off = name_len_off.checked_add(4).ok_or(AtlasError::Truncated)?;
        let name_blob = slice(bytes, name_blob_off, name_len)?;

        // No trailing bytes: the header's own counts must account for the
        // entire file.
        let expected_total = name_blob_off
            .checked_add(name_len)
            .ok_or(AtlasError::Truncated)?;
        if expected_total != bytes.len() {
            return Err(AtlasError::Truncated);
        }

        // --- Rings (bounds-checked against vertex_count before any use) ---
        let mut rings = Vec::with_capacity(ring_count);
        for i in 0..ring_count {
            let off = ring_table_off + i * RING_ENTRY_BYTES;
            let vertex_start = read_u32(bytes, off)?;
            let vertex_count_field = read_u32(bytes, off + 4)?;
            let is_hole = read_u8(bytes, off + 8)? != 0;
            let end = u64::from(vertex_start).checked_add(u64::from(vertex_count_field));
            match end {
                Some(end) if end <= vertex_count as u64 => {}
                _ => return Err(AtlasError::VertexRangeOutOfBounds),
            }
            rings.push(Ring {
                vertex_start,
                vertex_count: vertex_count_field,
                is_hole,
            });
        }

        // --- Vertices, converted from quantized grid units to world metres ---
        let mut vertices = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let off = vertex_table_off + i * VERTEX_ENTRY_BYTES;
            let gx = read_u16(bytes, off)?;
            let gy = read_u16(bytes, off + 2)?;
            let wx = origin_x + f64::from(gx) * scale;
            let wy = origin_y + f64::from(gy) * scale;
            // World metres are computed in f64 (grid units are u16, so no
            // precision is lost building wx/wy above); Bevy's Vec2/Transform
            // are f32, so this narrowing is the deliberate, one-time
            // handoff into the renderer's coordinate type, not a
            // computation that itself risks precision loss.
            #[allow(clippy::cast_possible_truncation)]
            vertices.push(Vec2::new(wx as f32, wy as f32));
        }

        // --- CSR offsets, checked monotonic (no row runs backwards) and
        // bounded (the final entry must equal csr_nnz, which — combined
        // with monotonicity — guarantees every offset lands in
        // [0, csr_nnz], so neighbors() can index csr_neighbors without a
        // bounds check at call time) ---
        let mut csr_offsets = Vec::with_capacity(county_count + 1);
        for i in 0..=county_count {
            let off = csr_offsets_off + i * CSR_ENTRY_BYTES;
            csr_offsets.push(read_u32(bytes, off)?);
        }
        for window in csr_offsets.windows(2) {
            if window[1] < window[0] {
                return Err(AtlasError::CsrOffsetsBackwards);
            }
        }
        if csr_offsets[county_count] as usize != csr_nnz {
            return Err(AtlasError::CsrTotalMismatch);
        }

        // --- CSR neighbors (bounds-checked against county_count — a
        // neighbor id naming a county that doesn't exist is exactly the
        // silent-corruption case III.11 forbids: `neighbors()` would
        // return it as though it were real, and a caller indexing back
        // into the county table with it would panic) ---
        let mut csr_neighbors = Vec::with_capacity(csr_nnz);
        for i in 0..csr_nnz {
            let off = csr_neighbors_off + i * CSR_ENTRY_BYTES;
            let neighbor = read_u32(bytes, off)?;
            if neighbor as usize >= county_count {
                return Err(AtlasError::CsrNeighborOutOfRange);
            }
            csr_neighbors.push(neighbor);
        }

        // --- Name blob: one line per county, in county-table order ---
        let name_text =
            core::str::from_utf8(name_blob).map_err(|_| AtlasError::NameCountMismatch)?;
        let names: Vec<&str> = name_text.lines().collect();
        if names.len() != county_count {
            return Err(AtlasError::NameCountMismatch);
        }

        // --- County table (ring ranges checked against ring_count) ---
        let mut counties = Vec::with_capacity(county_count);
        let mut min_corner = Vec2::splat(f32::INFINITY);
        let mut max_corner = Vec2::splat(f32::NEG_INFINITY);
        for (i, &name) in names.iter().enumerate() {
            let off = county_table_off + i * COUNTY_ENTRY_BYTES;
            let fips_bytes = slice(bytes, off, 5)?;
            let fips = core::str::from_utf8(fips_bytes)
                .map_err(|_| AtlasError::FipsNotUtf8)?
                // The format's "[u8; 5] ASCII, zero-padded" allows a
                // shorter FIPS than 5 bytes, trailing-NUL-padded; a FIPS
                // that IS the full 5 digits (every county in the
                // committed atlas) round-trips through this unchanged
                // (F9 — not reachable with committed data, fixed
                // defensively).
                .trim_end_matches('\0')
                .to_string();
            let ring_start = read_u32(bytes, off + 6)?;
            let county_ring_count = read_u16(bytes, off + 10)?;
            // flags (has-adjacency bit) lives at off + 12; this reader
            // exposes it implicitly through `neighbors()` returning an
            // empty slice, so it is not stored separately.
            let bbox_min_x = read_u16(bytes, off + 14)?;
            let bbox_min_y = read_u16(bytes, off + 16)?;
            let bbox_max_x = read_u16(bytes, off + 18)?;
            let bbox_max_y = read_u16(bytes, off + 20)?;
            let centroid_x = read_u16(bytes, off + 22)?;
            let centroid_y = read_u16(bytes, off + 24)?;

            let ring_end = u64::from(ring_start).checked_add(u64::from(county_ring_count));
            match ring_end {
                Some(end) if end <= ring_count as u64 => {}
                _ => return Err(AtlasError::RingRangeOutOfBounds),
            }

            let to_world = |gx: u16, gy: u16| -> Vec2 {
                let wx = origin_x + f64::from(gx) * scale;
                let wy = origin_y + f64::from(gy) * scale;
                // Same deliberate f64->f32 handoff into Bevy's coordinate
                // type as the vertex-array conversion above.
                #[allow(clippy::cast_possible_truncation)]
                Vec2::new(wx as f32, wy as f32)
            };
            let bbox = Bbox {
                min: to_world(bbox_min_x, bbox_min_y),
                max: to_world(bbox_max_x, bbox_max_y),
            };
            min_corner = min_corner.min(bbox.min).min(bbox.max);
            max_corner = max_corner.max(bbox.min).max(bbox.max);

            counties.push(CountyRow {
                fips,
                name: name.to_string(),
                ring_start,
                ring_count: county_ring_count,
                bbox,
                centroid: to_world(centroid_x, centroid_y),
            });
        }

        let world_bounds = if county_count == 0 {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        } else {
            Rect {
                min: min_corner,
                max: max_corner,
            }
        };

        Ok(CountyAtlas {
            counties,
            rings,
            vertices,
            csr_offsets,
            csr_neighbors,
            world_bounds,
        })
    }

    /// The number of counties the atlas describes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.counties.len()
    }

    /// Whether the atlas describes zero counties.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.counties.is_empty()
    }

    /// The county at `index`, or `None` if `index >= self.len()`.
    #[must_use]
    pub fn county(&self, index: usize) -> Option<County<'_>> {
        let row = self.counties.get(index)?;
        let start = row.ring_start as usize;
        let end = start + row.ring_count as usize;
        Some(County {
            fips: &row.fips,
            name: &row.name,
            rings: &self.rings[start..end],
            bbox: row.bbox,
            centroid: row.centroid,
        })
    }

    /// The index of the county whose FIPS code is `fips`, or `None`.
    #[must_use]
    pub fn index_of_fips(&self, fips: &str) -> Option<usize> {
        self.counties.iter().position(|c| c.fips == fips)
    }

    /// The CSR row of county-index neighbors for `index`, ascending.
    /// Panics if `index >= self.len()` — the same contract as slice
    /// indexing, since every caller in this crate only calls it with an
    /// index it already got from `0..self.len()`.
    #[must_use]
    pub fn neighbors(&self, index: usize) -> &[u32] {
        let start = self.csr_offsets[index] as usize;
        let end = self.csr_offsets[index + 1] as usize;
        &self.csr_neighbors[start..end]
    }

    /// Every decoded vertex, in world metres, in file order — the array
    /// `Ring::vertex_start`/`vertex_count` index into.
    #[must_use]
    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices
    }

    /// The bounding rectangle over every county's bounding box, in world
    /// metres — the camera clamp's bound (Task 7).
    #[must_use]
    pub fn world_bounds(&self) -> Rect {
        self.world_bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ATLAS_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/map/county_atlas.bin"
    ));

    /// Re-stamps `content_hash` after a structural mutation elsewhere in
    /// the buffer, so a rejection test isolates the ONE check it targets
    /// instead of also tripping `HashMismatch` first. Mirrors the mutation
    /// strategy `tests/unit/data/test_county_atlas_artifact.py` uses on the
    /// Python side.
    fn recompute_hash(bytes: &mut [u8]) {
        let digest = sha256_of(&bytes[48..]);
        bytes[16..48].copy_from_slice(&digest);
    }

    #[test]
    fn parses_the_committed_atlas() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses");
        assert_eq!(atlas.len(), 3222);
    }

    #[test]
    fn resolves_autauga_by_fips() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("parses");
        let index = atlas
            .index_of_fips("01001")
            .expect("Autauga County resolves");
        let county = atlas.county(index).expect("county exists");
        assert!(
            county.name.starts_with("Autauga County"),
            "unexpected name: {}",
            county.name
        );
    }

    #[test]
    fn neighbors_are_nonempty_ascending_and_symmetric() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("parses");
        let index = atlas.index_of_fips("01001").expect("resolves");
        let row = atlas.neighbors(index);
        assert!(!row.is_empty(), "Autauga County has real neighbors");
        for pair in row.windows(2) {
            assert!(pair[0] < pair[1], "neighbor row must be strictly ascending");
        }
        for &neighbor in row {
            let back = atlas.neighbors(neighbor as usize);
            // `index`/`neighbor` are atlas positions (0..3,222 on the
            // committed atlas) — far under u32::MAX. Cast lifted into its
            // own binding: an `#[allow]` on a macro-invocation statement
            // (`assert!`) is ignored by rustc, only a `let`/item attribute
            // actually scopes the lint.
            #[allow(clippy::cast_possible_truncation)]
            let index_u32 = index as u32;
            assert!(
                back.contains(&index_u32),
                "adjacency must be symmetric: {index} <-> {neighbor}"
            );
        }
    }

    #[test]
    fn world_bounds_are_finite_and_nondegenerate() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("parses");
        let bounds = atlas.world_bounds();
        assert!(bounds.min.x.is_finite() && bounds.min.y.is_finite());
        assert!(bounds.max.x.is_finite() && bounds.max.y.is_finite());
        assert!(bounds.max.x > bounds.min.x);
        assert!(bounds.max.y > bounds.min.y);
    }

    #[test]
    fn rejects_a_truncated_file() {
        // Chop the last 16 bytes off the name blob, then re-stamp the hash
        // so the hash check passes and the length-implied-by-the-header
        // check is what actually catches the truncation — otherwise
        // HashMismatch would fire first (any byte removed from bytes[48..]
        // changes the hash), which would prove the wrong check.
        let mut bytes = ATLAS_BYTES[..ATLAS_BYTES.len() - 16].to_vec();
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::Truncated
        );
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut bytes = ATLAS_BYTES.to_vec();
        bytes[0] = b'X';
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::BadMagic
        );
    }

    #[test]
    fn rejects_wrong_version() {
        let mut bytes = ATLAS_BYTES.to_vec();
        // Version (bytes 8..12) is NOT covered by content_hash (which
        // covers only bytes[48..]), so this mutation alone must not also
        // trip HashMismatch.
        bytes[8..12].copy_from_slice(&99u32.to_le_bytes());
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::BadVersion
        );
    }

    #[test]
    fn rejects_a_tampered_byte() {
        let mut bytes = ATLAS_BYTES.to_vec();
        bytes[200_000] ^= 0xFF;
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::HashMismatch
        );
    }

    #[test]
    fn rejects_ring_start_past_the_ring_table() {
        let mut bytes = ATLAS_BYTES.to_vec();
        // First county's ring_start field: offset 128 (county table start)
        // + 6 (fips[5] + pad[1]) = 134.
        bytes[134..138].copy_from_slice(&4_000_000u32.to_le_bytes());
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::RingRangeOutOfBounds
        );
    }

    #[test]
    fn rejects_vertex_range_past_the_vertex_array() {
        let mut bytes = ATLAS_BYTES.to_vec();
        // Ring table starts at 128 + 3222*28 = 90344; the first ring's
        // vertex_count field is at +4.
        let ring_table_off = 128 + 3222 * 28;
        bytes[ring_table_off + 4..ring_table_off + 8].copy_from_slice(&4_000_000u32.to_le_bytes());
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::VertexRangeOutOfBounds
        );
    }

    #[test]
    fn rejects_csr_offsets_running_backwards() {
        let mut bytes = ATLAS_BYTES.to_vec();
        // csr_offsets starts at 128 + 3222*28 + 3386*12 + 360064*4 =
        // 1,571,232. Set offsets[2] below offsets[1] (originally 5), which
        // makes county index 1's row run backwards.
        let csr_offsets_off = 128 + 3222 * 28 + 3386 * 12 + 360_064 * 4;
        let entry2 = csr_offsets_off + 2 * 4;
        bytes[entry2..entry2 + 4].copy_from_slice(&1u32.to_le_bytes());
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::CsrOffsetsBackwards
        );
    }

    /// F3 (adversarial verification of PR #490): the verifier crafted a
    /// hash-valid atlas with `csr_offsets[county_count] = 4_000_000` and
    /// showed `parse` accepted it, only for `neighbors()` to later panic
    /// with an out-of-range slice. Monotonicity alone does not bound the
    /// LAST offset against `csr_nnz`/`csr_neighbors.len()`.
    #[test]
    fn rejects_csr_offsets_whose_final_entry_disagrees_with_csr_nnz() {
        let mut bytes = ATLAS_BYTES.to_vec();
        let csr_offsets_off = 128 + 3222 * 28 + 3386 * 12 + 360_064 * 4;
        let last_entry = csr_offsets_off + 3222 * 4; // csr_offsets[county_count]
        bytes[last_entry..last_entry + 4].copy_from_slice(&4_000_000u32.to_le_bytes());
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::CsrTotalMismatch
        );
    }

    /// F3: the verifier's second crafted artifact set a `csr_neighbors`
    /// entry to `999999` (no such county) — `parse` accepted it, and a
    /// caller resolving that "neighbor" back into the county table would
    /// panic.
    #[test]
    fn rejects_a_csr_neighbor_naming_a_county_past_county_count() {
        let mut bytes = ATLAS_BYTES.to_vec();
        let csr_offsets_off = 128 + 3222 * 28 + 3386 * 12 + 360_064 * 4;
        let csr_neighbors_off = csr_offsets_off + 3223 * 4;
        bytes[csr_neighbors_off..csr_neighbors_off + 4].copy_from_slice(&999_999u32.to_le_bytes());
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::CsrNeighborOutOfRange
        );
    }

    /// F6: `NameCountMismatch` was the one `AtlasError` variant with no
    /// dedicated test. Swapping one interior `\n` for another byte (same
    /// total length, so this is not also a `Truncated` case) merges two
    /// names into one line, dropping the line count by one.
    #[test]
    fn rejects_a_swapped_newline_in_the_name_blob() {
        let mut bytes = ATLAS_BYTES.to_vec();
        let name_blob_off = 128 + 3222 * 28 + 3386 * 12 + 360_064 * 4 + 3223 * 4 + 18954 * 4 + 4;
        let newline_pos = bytes[name_blob_off..]
            .iter()
            .position(|&b| b == b'\n')
            .expect("name blob has at least one newline")
            + name_blob_off;
        bytes[newline_pos] = b' ';
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::NameCountMismatch
        );
    }

    /// F6: the FIPS UTF-8 failure is a distinct check from the name
    /// blob's — this test proves it fires its OWN variant rather than
    /// `NameCountMismatch` (which is what the pre-fix code returned for
    /// both).
    #[test]
    fn rejects_a_fips_field_that_is_not_valid_utf8() {
        let mut bytes = ATLAS_BYTES.to_vec();
        // First county's fips field starts at byte 128; 0xFF is not a
        // valid UTF-8 lead byte.
        bytes[128] = 0xFF;
        recompute_hash(&mut bytes);
        assert_eq!(
            CountyAtlas::parse(&bytes).unwrap_err(),
            AtlasError::FipsNotUtf8
        );
    }
}
