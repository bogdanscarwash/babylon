//! Deterministic schematic placement of disclosed production relationships.
//! Coordinates, structure sizes and link widths are presentation, not quantities
//! or geographic locations. Strongly connected sites share a stage.

use std::collections::{BTreeMap, BTreeSet};

use babylon_persistence::ProductionSnapshotV1;
use bevy::prelude::{Rect, Vec2, Vec3};

use crate::production_brief::{dependency_sites, DependencyDirection};

pub(crate) const RAIL_HEIGHT: f32 = 42.0;

pub(crate) struct ProductionLayout {
    pub positions: BTreeMap<String, Vec3>,
    pub links: Vec<(String, String)>,
    pub platforms: Vec<(Vec3, Vec2)>,
}

impl ProductionLayout {
    pub(crate) fn new(snapshot: &ProductionSnapshotV1) -> Self {
        let ids: Vec<_> = snapshot
            .sites
            .iter()
            .map(|site| site.id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let indices: BTreeMap<_, _> = ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.as_str(), index))
            .collect();
        let mut edges = BTreeSet::new();
        for site in &snapshot.sites {
            for (direction, buyer) in dependency_sites(site, snapshot) {
                if direction == DependencyDirection::Downstream {
                    edges.insert((indices[site.id.as_str()], indices[buyer.id.as_str()]));
                }
            }
        }
        let ranks = stages(ids.len(), &edges);
        let groups = weak_components(ids.len(), &edges);
        let (positions, platforms) = place_groups(&ids, &ranks, &groups);
        Self {
            positions,
            links: edges
                .into_iter()
                .map(|(from, to)| (ids[from].clone(), ids[to].clone()))
                .collect(),
            platforms,
        }
    }
}

fn reachable(start: usize, edges: &BTreeSet<(usize, usize)>, undirected: bool) -> BTreeSet<usize> {
    let mut visited = BTreeSet::new();
    let mut pending = vec![start];
    while let Some(node) = pending.pop() {
        if !visited.insert(node) {
            continue;
        }
        for &(from, to) in edges {
            if from == node {
                pending.push(to);
            }
            if undirected && to == node {
                pending.push(from);
            }
        }
    }
    visited
}

fn weak_components(count: usize, edges: &BTreeSet<(usize, usize)>) -> Vec<BTreeSet<usize>> {
    let mut seen = BTreeSet::<usize>::new();
    let mut groups = Vec::new();
    for node in 0..count {
        if !seen.contains(&node) {
            let group = reachable(node, edges, true);
            seen.extend(&group);
            groups.push(group);
        }
    }
    groups
}

/// Condense mutual reachability before assigning longest-path stages. Cycles
/// never stall the layout or masquerade as a supplier-before-buyer ordering.
fn stages(count: usize, edges: &BTreeSet<(usize, usize)>) -> Vec<usize> {
    let reaches: Vec<_> = (0..count)
        .map(|node| reachable(node, edges, false))
        .collect();
    let roots: Vec<_> = (0..count)
        .map(|node| {
            (0..=node)
                .find(|other| reaches[node].contains(other) && reaches[*other].contains(&node))
                .unwrap_or(node)
        })
        .collect();
    let condensed: BTreeSet<_> = edges
        .iter()
        .filter_map(|&(from, to)| (roots[from] != roots[to]).then_some((roots[from], roots[to])))
        .collect();
    let mut indegree = vec![0_usize; count];
    for &(_, to) in &condensed {
        indegree[to] += 1;
    }
    let mut ready: BTreeSet<_> = roots
        .iter()
        .copied()
        .filter(|node| indegree[*node] == 0)
        .collect();
    let mut rank = vec![0_usize; count];
    while let Some(node) = ready.pop_first() {
        for &(_, to) in condensed.iter().filter(|(from, _)| *from == node) {
            rank[to] = rank[to].max(rank[node] + 1);
            indegree[to] -= 1;
            if indegree[to] == 0 {
                ready.insert(to);
            }
        }
    }
    roots.iter().map(|root| rank[*root]).collect()
}

type PlacedGroups = (BTreeMap<String, Vec3>, Vec<(Vec3, Vec2)>);

// Counts only determine schematic spacing; no quantity is converted here.
#[allow(clippy::cast_precision_loss)]
fn place_groups(ids: &[String], ranks: &[usize], groups: &[BTreeSet<usize>]) -> PlacedGroups {
    let mut positions = BTreeMap::new();
    let mut platforms = Vec::new();
    let mut offset = 0.0;
    for group in groups {
        let mut layers: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for &node in group {
            layers.entry(ranks[node]).or_default().push(node);
        }
        let longest = layers.values().map(Vec::len).max().unwrap_or(1);
        let depth = longest as f32 * 180.0 + 150.0;
        let last_stage = layers.keys().next_back().copied().unwrap_or(0) as f32;
        let center = offset + depth * 0.5;
        for (rank, nodes) in &layers {
            for (lane, &node) in nodes.iter().enumerate() {
                positions.insert(
                    ids[node].clone(),
                    Vec3::new(
                        (*rank as f32 - last_stage * 0.5) * 300.0,
                        0.0,
                        center + (lane as f32 - (nodes.len() as f32 - 1.0) * 0.5) * 180.0,
                    ),
                );
            }
        }
        platforms.push((
            Vec3::new(0.0, -10.0, center),
            Vec2::new(last_stage * 300.0 + 260.0, depth - 60.0),
        ));
        offset += depth;
    }
    for position in positions.values_mut() {
        position.z -= offset * 0.5;
    }
    for (center, _) in &mut platforms {
        center.z -= offset * 0.5;
    }
    (positions, platforms)
}

/// Corners are schematic routing geometry, never intermediate suppliers.
pub(crate) fn relation_path(from: Vec3, to: Vec3) -> Vec<Vec3> {
    let start = from + Vec3::new(0.0, RAIL_HEIGHT, 68.0);
    let end = to + Vec3::new(0.0, RAIL_HEIGHT, 68.0);
    let mut path = if from == to {
        vec![
            start,
            start + Vec3::X * 130.0,
            start + Vec3::new(130.0, 0.0, 90.0),
            start + Vec3::Z * 90.0,
            end,
        ]
    } else {
        let bend = if to.x > from.x {
            (from.x + to.x) * 0.5
        } else if to.z > from.z {
            from.x.max(to.x) + 135.0
        } else {
            from.x.min(to.x) - 135.0
        };
        vec![
            start,
            Vec3::new(bend, RAIL_HEIGHT, start.z),
            Vec3::new(bend, RAIL_HEIGHT, end.z),
            end,
        ]
    };
    path.dedup();
    path
}

pub(crate) fn path_point(path: &[Vec3], fraction: f32) -> Option<Vec3> {
    let length: f32 = path.windows(2).map(|pair| pair[0].distance(pair[1])).sum();
    let mut remaining = length * fraction.clamp(0.0, 1.0);
    for pair in path.windows(2) {
        let segment = pair[0].distance(pair[1]);
        if segment > 0.0 && remaining <= segment {
            return Some(pair[0].lerp(pair[1], remaining / segment));
        }
        remaining -= segment;
    }
    path.last().copied()
}

/// Reserve the selected label first, then stable identities. Every visible
/// label is placed without overlap; a close leader can identify displaced tags.
pub(crate) fn place_label(
    anchor: Vec2,
    bounds: Rect,
    size: Vec2,
    occupied: &[Rect],
) -> Option<Rect> {
    if !bounds.contains(anchor) || size.x > bounds.width() || size.y > bounds.height() {
        return None;
    }
    let desired = anchor - Vec2::new(size.x * 0.5, size.y + 10.0);
    let valid = |position: Vec2| {
        let rect = Rect::from_corners(
            position.clamp(bounds.min, bounds.max - size),
            position.clamp(bounds.min, bounds.max - size) + size,
        );
        (!occupied
            .iter()
            .any(|other| !rect.intersect(other.inflate(5.0)).is_empty()))
        .then_some(rect)
    };
    for row in [0.0, -1.0, 1.0, -2.0, 2.0, -3.0, 3.0] {
        for column in [0.0, -1.0, 1.0] {
            if let Some(rect) =
                valid(desired + Vec2::new(column * (size.x + 8.0), row * (size.y + 8.0)))
            {
                return Some(rect);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_preserve_chain_direction_and_condense_cycles() {
        let edges = BTreeSet::from([(0, 1), (1, 2), (2, 1), (2, 3), (4, 5)]);
        assert_eq!(stages(6, &edges), [0, 1, 1, 2, 0, 1]);
        assert_eq!(
            weak_components(6, &edges),
            [BTreeSet::from([0, 1, 2, 3]), BTreeSet::from([4, 5])]
        );
        let (positions, _) = place_groups(
            &["a", "b", "c", "d", "e", "f"].map(str::to_owned),
            &stages(6, &edges),
            &weak_components(6, &edges),
        );
        assert!(positions["a"].x < positions["b"].x && positions["b"].x < positions["d"].x);
        assert_ne!(positions["b"], positions["c"]);
        assert!(positions["e"].z > positions["d"].z);
    }

    #[test]
    fn rails_run_from_supplier_to_buyer_including_reverse_and_self_links() {
        for to in [
            Vec3::new(300.0, 0.0, 0.0),
            Vec3::new(-300.0, 0.0, 180.0),
            Vec3::ZERO,
        ] {
            let path = relation_path(Vec3::ZERO, to);
            assert_eq!(
                path_point(&path, 0.0),
                Some(Vec3::new(0.0, RAIL_HEIGHT, 68.0))
            );
            assert_eq!(
                path_point(&path, 1.0),
                Some(to + Vec3::new(0.0, RAIL_HEIGHT, 68.0))
            );
            assert!(path.windows(2).all(|pair| pair[0].distance(pair[1]) > 0.0));
            assert!(path_point(&path, 0.5).is_some());
        }
    }

    #[test]
    fn overlapping_anchors_get_separate_labels_inside_a_small_scene() {
        let bounds = Rect::new(16.0, 124.0, 950.0, 580.0);
        let size = Vec2::new(170.0, 58.0);
        let mut occupied = Vec::new();
        for _ in 0..5 {
            let rect = place_label(Vec2::new(480.0, 350.0), bounds, size, &occupied)
                .expect("five labels fit");
            assert!(bounds.contains(rect.min) && bounds.contains(rect.max));
            assert!(occupied
                .iter()
                .all(|other| rect.intersect(*other).is_empty()));
            occupied.push(rect);
        }
        assert!(place_label(Vec2::ZERO, bounds, size, &[]).is_none());
    }
}
