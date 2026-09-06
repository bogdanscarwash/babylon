//! Tessellation (B1 Task 5): turn a `CountyAtlas`'s rings into triangles.
//!
//! One county's rings run in polygon order (`atlas.rs`'s AS-BUILT ring
//! storage note): every `is_hole == false` ring opens a new polygon, and
//! the `is_hole == true` rings after it belong to that polygon — a county
//! may hold more than one polygon (islands, exclaves). This module groups
//! rings back into polygons and hands each one to `earcutr`, exterior
//! first then holes, per the earcut convention.

// File-scoped (Task 1's pedantic-debt fix pass, plan §Task 1 Step 1.2's
// named sanctioned escape for this file): every flagged cast here is a
// vertex/ring/triangle INDEX (`usize` -> `u32`/`u16`) bounded by the
// committed atlas's real geometry — 360,064 vertices, 3,386 rings, at most
// a few hundred thousand triangles, all far under `u32::MAX` — or the
// deliberate f64->f32 handoff from earcutr's f64 working precision into
// Bevy's f32 vertex format (same handoff `atlas.rs` allows inline), or a
// usize->i64 diagnostic-only delta in a test assertion. None of the ~19
// sites is a computation whose overflow would be silently wrong; each
// would instead show up immediately as visibly broken geometry (this
// module's own area-matching tests, `every_real_county_tessellates_to_its_
// own_shoelace_area` in particular, would fail first). A follow-up hygiene
// item (tracked alongside babylon-tick's own latent pedantic debt, Task 0's
// finding) can tighten these to checked `u32::try_from` conversions later;
// that is a systemic sweep across this file's ~19 sites, not a defect any
// one of them carries alone.
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use crate::atlas::{CountyAtlas, Ring};

/// One county's worth of triangles is a contiguous slice of `positions`
/// (recorded in `county_vertex_range`), so a per-tick recolor is an
/// `O(vertices)` write across one buffer instead of a mesh rebuild.
pub struct Tessellation {
    /// Every triangle vertex position, in world metres, z = 0.0.
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices into `positions`, three per triangle.
    pub indices: Vec<u32>,
    /// Per-vertex county index, parallel to `positions`.
    pub vertex_county: Vec<u32>,
    /// Per-county `[start, end)` range into `positions`.
    pub county_vertex_range: Vec<(u32, u32)>,
}

/// Tessellate every county in `atlas` into one merged triangle set.
///
/// # Panics
/// If `earcutr` fails to triangulate a county's polygon — a malformed atlas
/// (a build-time defect, since this parse runs once at Startup) is the
/// loud-failure posture this crate takes throughout, not a silent hole in
/// the map (see `tessellate_polygon`'s own comment).
#[must_use]
pub fn tessellate(atlas: &CountyAtlas) -> Tessellation {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_county = Vec::new();
    let mut county_vertex_range = Vec::with_capacity(atlas.len());

    for county_index in 0..atlas.len() {
        let county = atlas
            .county(county_index)
            .expect("county_index is within 0..atlas.len()");
        let range_start = positions.len() as u32;

        let mut polygon_start = 0usize;
        while polygon_start < county.rings.len() {
            // F8 (adversarial verification of PR #490): the grouping
            // below RELIES on every polygon opening with a non-hole ring
            // (`atlas.rs`'s AS-BUILT note: "every is_hole == false ring
            // opens a new polygon"). Task 1's encoder guarantees this by
            // construction; assert it here as defense in depth rather
            // than trusting a comment two files away.
            debug_assert!(
                !county.rings[polygon_start].is_hole,
                "a polygon group must open with a non-hole ring (county index {county_index})"
            );
            let mut polygon_end = polygon_start + 1;
            while polygon_end < county.rings.len() && county.rings[polygon_end].is_hole {
                polygon_end += 1;
            }
            tessellate_polygon(
                atlas,
                county_index as u32,
                &county.rings[polygon_start..polygon_end],
                &mut positions,
                &mut indices,
                &mut vertex_county,
            );
            polygon_start = polygon_end;
        }

        let range_end = positions.len() as u32;
        county_vertex_range.push((range_start, range_end));
    }

    Tessellation {
        positions,
        indices,
        vertex_county,
        county_vertex_range,
    }
}

/// Triangulate one polygon (an exterior ring plus its holes, already
/// grouped by the caller) and append its vertices and triangle indices to
/// the shared buffers.
fn tessellate_polygon(
    atlas: &CountyAtlas,
    county_index: u32,
    rings: &[Ring],
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    vertex_county: &mut Vec<u32>,
) {
    let vertices = atlas.vertices();
    let mut flat: Vec<f64> = Vec::new();
    let mut hole_indices: Vec<usize> = Vec::new();
    let mut point_count = 0usize;

    for (i, ring) in rings.iter().enumerate() {
        if i > 0 {
            // earcutr's hole_indices count POINTS (x,y pairs), not flat
            // coordinate slots.
            hole_indices.push(point_count);
        }
        let start = ring.vertex_start as usize;
        let end = start + ring.vertex_count as usize;
        for v in &vertices[start..end] {
            flat.push(f64::from(v.x));
            flat.push(f64::from(v.y));
            point_count += 1;
        }
    }

    let base = positions.len() as u32;
    for point in flat.chunks_exact(2) {
        positions.push([point[0] as f32, point[1] as f32, 0.0]);
        vertex_county.push(county_index);
    }

    // A malformed polygon here means the atlas (Task 1's build-time
    // simplification/quantization) shipped a defect, and this parse runs
    // once at Startup — panicking is the loud-failure posture Task 6 takes
    // for the whole map (a client that opens with a broken county is the
    // loud-failure case, not a silent hole in the map).
    let triangles = earcutr::earcut(&flat, &hole_indices, 2).unwrap_or_else(|e| {
        panic!("earcut failed to triangulate county index {county_index}: {e:?}")
    });
    for idx in triangles {
        indices.push(base + idx as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_kernel::content_digest::sha256_of;

    /// Assembles a minimal, valid `county_atlas.bin` buffer in memory, byte
    /// for byte per `atlas.rs`'s documented format, so this module's tests
    /// exercise the real reader against hand-built geometry rather than a
    /// mocked `CountyAtlas`. `origin = (0, 0)` and `scale = 1.0`, so grid
    /// units equal world metres exactly — the fixture's coordinates can be
    /// read directly as the world positions the test asserts on.
    struct FixtureCounty {
        fips: &'static str,
        name: &'static str,
        // (is_hole, ring points in grid units)
        rings: Vec<(bool, Vec<(u16, u16)>)>,
    }

    fn build_atlas_bytes(counties: &[FixtureCounty]) -> Vec<u8> {
        let county_count = counties.len() as u32;
        let mut ring_rows: Vec<(u32, u32, bool)> = Vec::new(); // (vertex_start, vertex_count, is_hole)
        let mut vertices: Vec<(u16, u16)> = Vec::new();
        let mut county_ring_start: Vec<u32> = Vec::new();
        let mut county_ring_count: Vec<u16> = Vec::new();

        for county in counties {
            county_ring_start.push(ring_rows.len() as u32);
            county_ring_count.push(county.rings.len() as u16);
            for (is_hole, points) in &county.rings {
                let vertex_start = vertices.len() as u32;
                for &p in points {
                    vertices.push(p);
                }
                ring_rows.push((vertex_start, points.len() as u32, *is_hole));
            }
        }

        let ring_count = ring_rows.len() as u32;
        let vertex_count = vertices.len() as u32;
        let csr_nnz = 0u32; // no adjacency needed for tessellation tests

        // --- county table ---
        let mut county_bytes = Vec::new();
        for (i, county) in counties.iter().enumerate() {
            let fips = county.fips.as_bytes();
            assert_eq!(fips.len(), 5, "fixture FIPS must be exactly 5 ASCII bytes");
            county_bytes.extend_from_slice(fips);
            county_bytes.push(0); // pad
            county_bytes.extend_from_slice(&county_ring_start[i].to_le_bytes());
            county_bytes.extend_from_slice(&county_ring_count[i].to_le_bytes());
            county_bytes.extend_from_slice(&0u16.to_le_bytes()); // flags: no adjacency
                                                                 // bbox: unused by tessellate.rs's tests, zero-filled is fine.
            county_bytes.extend_from_slice(&0u16.to_le_bytes());
            county_bytes.extend_from_slice(&0u16.to_le_bytes());
            county_bytes.extend_from_slice(&0u16.to_le_bytes());
            county_bytes.extend_from_slice(&0u16.to_le_bytes());
            // centroid: likewise unused here.
            county_bytes.extend_from_slice(&0u16.to_le_bytes());
            county_bytes.extend_from_slice(&0u16.to_le_bytes());
            county_bytes.extend_from_slice(&0u16.to_le_bytes()); // pad[2]
        }

        // --- ring table ---
        let mut ring_bytes = Vec::new();
        for (vertex_start, vertex_count, is_hole) in &ring_rows {
            ring_bytes.extend_from_slice(&vertex_start.to_le_bytes());
            ring_bytes.extend_from_slice(&vertex_count.to_le_bytes());
            ring_bytes.push(u8::from(*is_hole));
            ring_bytes.extend_from_slice(&[0u8; 3]);
        }

        // --- vertex array ---
        let mut vertex_bytes = Vec::new();
        for (x, y) in &vertices {
            vertex_bytes.extend_from_slice(&x.to_le_bytes());
            vertex_bytes.extend_from_slice(&y.to_le_bytes());
        }

        // --- csr (empty rows for every county) ---
        let mut csr_offsets_bytes = Vec::new();
        for _ in 0..=counties.len() {
            csr_offsets_bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        let csr_neighbors_bytes: Vec<u8> = Vec::new();

        // --- name blob ---
        let mut name_blob = String::new();
        for county in counties {
            name_blob.push_str(county.name);
            name_blob.push('\n');
        }
        let name_bytes = name_blob.as_bytes();

        let body_without_name = [
            county_bytes.as_slice(),
            ring_bytes.as_slice(),
            vertex_bytes.as_slice(),
            csr_offsets_bytes.as_slice(),
            csr_neighbors_bytes.as_slice(),
        ]
        .concat();
        let mut body = body_without_name;
        body.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        body.extend_from_slice(name_bytes);

        let mut tail = Vec::new();
        tail.extend_from_slice(&0.0f64.to_le_bytes()); // origin_x
        tail.extend_from_slice(&0.0f64.to_le_bytes()); // origin_y
        tail.extend_from_slice(&1.0f64.to_le_bytes()); // scale
        tail.extend_from_slice(&county_count.to_le_bytes());
        tail.extend_from_slice(&ring_count.to_le_bytes());
        tail.extend_from_slice(&vertex_count.to_le_bytes());
        tail.extend_from_slice(&csr_nnz.to_le_bytes());
        tail.extend_from_slice(&[0u8; 32]); // source_hash: unchecked by tessellate tests
        tail.extend_from_slice(&[0u8; 8]); // reserved padding

        let mut tail_and_body = tail.clone();
        tail_and_body.extend_from_slice(&body);
        let content_hash = sha256_of(&tail_and_body);

        let mut out = Vec::new();
        out.extend_from_slice(b"BABCTY\0\x01");
        out.extend_from_slice(&1u32.to_le_bytes()); // version
        out.extend_from_slice(&0u32.to_le_bytes()); // flags
        out.extend_from_slice(&content_hash);
        assert_eq!(out.len(), 48, "header prefix must be 48 bytes before tail");
        out.extend_from_slice(&tail_and_body);
        out
    }

    fn unit_square_fixture() -> Vec<u8> {
        build_atlas_bytes(&[
            FixtureCounty {
                fips: "00001",
                name: "TestCountyA, TS",
                rings: vec![(false, vec![(0, 0), (10, 0), (10, 10), (0, 10)])],
            },
            FixtureCounty {
                fips: "00002",
                name: "TestCountyB, TS",
                rings: vec![
                    (false, vec![(100, 0), (120, 0), (120, 20), (100, 20)]),
                    (true, vec![(105, 5), (105, 15), (115, 15), (115, 5)]),
                ],
            },
        ])
    }

    fn shoelace_area(points: &[(u16, u16)]) -> f64 {
        let mut acc = 0.0;
        for i in 0..points.len() {
            let (x0, y0) = points[i];
            let (x1, y1) = points[(i + 1) % points.len()];
            acc += f64::from(x0) * f64::from(y1) - f64::from(x1) * f64::from(y0);
        }
        acc.abs() / 2.0
    }

    /// The unsigned shoelace area of a ring given as world-metre `Vec2`
    /// points (as opposed to `shoelace_area`'s `u16` grid-unit points,
    /// used by the synthetic fixture tests above).
    fn shoelace_area_vec2(points: &[bevy::math::Vec2]) -> f64 {
        let mut acc = 0.0;
        for i in 0..points.len() {
            let p0 = points[i];
            let p1 = points[(i + 1) % points.len()];
            acc += f64::from(p0.x) * f64::from(p1.y) - f64::from(p1.x) * f64::from(p0.y);
        }
        acc.abs() / 2.0
    }

    /// A county's net polygon area (exterior rings' areas, holes
    /// subtracted) computed directly from the atlas's own ring/vertex
    /// data — independent of tessellation, so comparing it against the
    /// triangulated area sum is a real correctness check, not a
    /// tautology.
    fn county_polygon_area(atlas: &CountyAtlas, county: &crate::atlas::County<'_>) -> f64 {
        let vertices = atlas.vertices();
        let mut area = 0.0;
        for ring in county.rings {
            let start = ring.vertex_start as usize;
            let end = start + ring.vertex_count as usize;
            let ring_area = shoelace_area_vec2(&vertices[start..end]);
            area += if ring.is_hole { -ring_area } else { ring_area };
        }
        area
    }

    fn triangle_area(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f64 {
        let x0 = f64::from(p0[0]);
        let y0 = f64::from(p0[1]);
        let x1 = f64::from(p1[0]);
        let y1 = f64::from(p1[1]);
        let x2 = f64::from(p2[0]);
        let y2 = f64::from(p2[1]);
        ((x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)).abs() / 2.0
    }

    #[test]
    fn unit_square_yields_two_triangles() {
        let bytes = unit_square_fixture();
        let atlas = CountyAtlas::parse(&bytes).expect("fixture atlas parses");
        let tess = tessellate(&atlas);
        let (start, end) = tess.county_vertex_range[0];
        assert_eq!(end - start, 4, "the square contributes its 4 corners");
        let triangle_count = tess
            .indices
            .chunks_exact(3)
            .filter(|tri| tri.iter().all(|&i| (start..end).contains(&i)))
            .count();
        assert_eq!(
            triangle_count, 2,
            "a 4-vertex simple polygon is 2 triangles"
        );
    }

    #[test]
    fn holed_square_triangulates_with_no_gap_and_matching_area() {
        let bytes = unit_square_fixture();
        let atlas = CountyAtlas::parse(&bytes).expect("fixture atlas parses");
        let tess = tessellate(&atlas);

        // county_vertex_range partitions positions contiguously: no gap,
        // no overlap.
        assert_eq!(tess.county_vertex_range[0].0, 0);
        assert_eq!(tess.county_vertex_range[0].1, tess.county_vertex_range[1].0);
        assert_eq!(tess.county_vertex_range[1].1, tess.positions.len() as u32);

        let (start, end) = tess.county_vertex_range[1];
        assert_eq!(end - start, 8, "exterior(4) + hole(4) = 8 vertices");

        // Every index lands in range.
        for &idx in &tess.indices {
            assert!((idx as usize) < tess.positions.len());
        }

        let county_triangles: Vec<[u32; 3]> = tess
            .indices
            .chunks_exact(3)
            .filter(|tri| tri.iter().all(|&i| (start..end).contains(&i)))
            .map(|tri| [tri[0], tri[1], tri[2]])
            .collect();
        assert!(
            !county_triangles.is_empty(),
            "the holed square must not vanish"
        );
        // AS-BUILT deviation from the plan's literal wording: Task 5 Step 1
        // says "the holed square's triangle count equals n - 2 for its
        // combined ring vertex count" (n=8 => 6). That is not what earcutr
        // actually returns for an exterior-plus-hole polygon: bridging the
        // hole into the outer ring adds two point-uses back into the
        // ear-clipping walk, so the real count is n - 2 + 2h = 8 for one
        // hole (confirmed empirically against the compiled earcutr 0.5
        // crate, not re-derived from theory alone). The area-matching
        // assertion below is the property that actually proves correctness
        // regardless of the exact count; this assertion additionally pins
        // the real observed number so a future earcutr upgrade that changes
        // it is visible.
        assert_eq!(county_triangles.len(), 8);

        let triangle_area_sum: f64 = county_triangles
            .iter()
            .map(|tri| {
                triangle_area(
                    tess.positions[tri[0] as usize],
                    tess.positions[tri[1] as usize],
                    tess.positions[tri[2] as usize],
                )
            })
            .sum();
        let exterior_area = shoelace_area(&[(100, 0), (120, 0), (120, 20), (100, 20)]);
        let hole_area = shoelace_area(&[(105, 5), (105, 15), (115, 15), (115, 5)]);
        let expected_area = exterior_area - hole_area;
        assert!(
            (triangle_area_sum - expected_area).abs() < 1e-6,
            "triangle area sum {triangle_area_sum} must match the polygon area {expected_area} \
             (a gap or overlap would move this)"
        );
    }

    #[test]
    fn real_atlas_tessellates_every_county_and_lands_near_the_ring_estimate() {
        let atlas_bytes: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        ));
        let atlas = CountyAtlas::parse(atlas_bytes).expect("committed atlas parses");
        let started = std::time::Instant::now();
        let tess = tessellate(&atlas);
        // This cost lands once at Startup, so slow-but-correct is fine —
        // Task 5 Step 4 asks to say the number out loud rather than assert
        // a budget on it.
        eprintln!(
            "full-atlas tessellation: {:?} (debug/unoptimized build)",
            started.elapsed()
        );

        assert_eq!(tess.county_vertex_range.len(), atlas.len());

        // No county tessellates to zero triangles.
        let mut triangle_count_by_county = vec![0u32; atlas.len()];
        for tri in tess.indices.chunks_exact(3) {
            let county = tess.vertex_county[tri[0] as usize];
            triangle_count_by_county[county as usize] += 1;
        }
        let vanished: Vec<usize> = triangle_count_by_county
            .iter()
            .enumerate()
            .filter(|(_, &count)| count == 0)
            .map(|(i, _)| i)
            .collect();
        assert!(
            vanished.is_empty(),
            "counties with zero triangles (simplification bug): {vanished:?}"
        );

        // The plan's own AS-BUILT note: "the whole atlas to vertex_count -
        // 2 * ring_count" is a rough estimate (it undercounts by 2 per
        // hole, since a hole's bridge does not remove a triangle the way
        // an extra exterior ring boundary does) — assert "near", not
        // exact, as Task 5 Step 4 asks. ring_count is summed from the
        // atlas's own county->rings data (F8: no hardcoded literal —
        // every ring belongs to exactly one county, so this sum equals
        // the atlas's total ring_count without atlas.rs needing to
        // expose that count directly).
        let vertex_count = atlas.vertices().len();
        let ring_count: usize = (0..atlas.len())
            .map(|i| atlas.county(i).expect("index in range").rings.len())
            .sum();
        let total_triangles = tess.indices.len() / 3;
        let rough_estimate = vertex_count as i64 - 2 * ring_count as i64;
        let deviation = (total_triangles as i64 - rough_estimate).unsigned_abs();
        assert!(
            deviation < rough_estimate.unsigned_abs() / 20,
            "total triangles {total_triangles} strayed too far from the rough estimate \
             {rough_estimate} (within 5%)"
        );
    }

    /// F8 (adversarial verification of PR #490): promotes the verifier's
    /// own ad-hoc check — a per-county property comparing the tessellated
    /// triangle-area sum against the county's shoelace polygon area
    /// (exterior minus holes), run over all 3,222 real counties — into
    /// the committed suite, so this correctness property is enforced by
    /// `cargo test`, not left as something only an external review ran
    /// once. The verifier's own run found a worst relative error of
    /// 0.0017 (0.17%) with zero counties over 1%; this test asserts the
    /// 1% band, giving headroom above the measured worst case.
    #[test]
    fn every_real_county_tessellates_to_its_own_shoelace_area() {
        let atlas_bytes: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        ));
        let atlas = CountyAtlas::parse(atlas_bytes).expect("committed atlas parses");
        let tess = tessellate(&atlas);

        // One pass over every triangle, bucketed by owning county —
        // O(triangles), not O(counties x triangles).
        let mut triangle_area_sum_by_county = vec![0.0f64; atlas.len()];
        for tri in tess.indices.chunks_exact(3) {
            let county = tess.vertex_county[tri[0] as usize] as usize;
            triangle_area_sum_by_county[county] += triangle_area(
                tess.positions[tri[0] as usize],
                tess.positions[tri[1] as usize],
                tess.positions[tri[2] as usize],
            );
        }

        let mut worst_rel_error = 0.0f64;
        let mut worst_index = 0usize;
        let mut over_one_percent: Vec<(String, f64)> = Vec::new();

        for (index, &triangulated) in triangle_area_sum_by_county.iter().enumerate() {
            let county = atlas.county(index).expect("index in range");
            let shoelace = county_polygon_area(&atlas, &county);
            let rel_error = if shoelace.abs() > 1.0 {
                (triangulated - shoelace).abs() / shoelace.abs()
            } else {
                (triangulated - shoelace).abs()
            };
            if rel_error > worst_rel_error {
                worst_rel_error = rel_error;
                worst_index = index;
            }
            if rel_error > 0.01 {
                over_one_percent.push((county.fips.to_string(), rel_error));
            }
        }

        assert!(
            over_one_percent.is_empty(),
            "counties over 1% relative area error: {over_one_percent:?}"
        );
        eprintln!(
            "worst per-county area relative error: {worst_rel_error:.6} (county index \
             {worst_index}, fips {})",
            atlas.county(worst_index).expect("in range").fips
        );
    }
}
