//! Captured selected-path continuity and mass-membership checks, not a router.
use super::{county, digest};
use crate::michigan_material::{
    MichiganCountyTerminal, MichiganMaterialError, MichiganMaterialPath, MichiganNormalizedContent,
    MichiganPhysicalEdge, MichiganPhysicalNetwork,
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn validate(c: &MichiganNormalizedContent) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::PhysicalPath;
    let Some(network) = &c.physical_network else {
        if c.routes.iter().any(|r| matches!(&r.path, MichiganMaterialPath::Routed {physical_edge_keys,distance_mm,..} if !physical_edge_keys.is_empty() || distance_mm.is_some())) { return Err(PhysicalPath); }
        return Ok(());
    };
    network_authority(network)?;
    let edges: BTreeMap<_, _> = network.edges.iter().map(|e| (e.id.as_str(), e)).collect();
    let terminals: BTreeMap<_, _> = network
        .terminals
        .iter()
        .map(|t| (t.county_geoid.as_str(), t))
        .collect();
    if edges.len() != network.edges.len() || terminals.len() != network.terminals.len() {
        return Err(PhysicalPath);
    }
    selected_geometry(c, network, &edges)?;
    directed_paths(c, network, &edges, &terminals)?;
    Ok(())
}

fn network_authority(network: &MichiganPhysicalNetwork) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::PhysicalPath;
    if !digest(&network.source.pbf_sha256)
        || !digest(&network.source.footprint_sha256)
        || !digest(&network.source.graph_sha256)
        || network.source.pbf_bytes == 0
        || network.source.pbf_url.is_empty()
        || network.source.routing_profile_version != "michigan-freight-routing-v1"
        || network.profile.evidence_class != "Designed"
        || network.profile.gross_weight_kg == 0
        || network.profile.height_mm == 0
        || network.profile.width_mm == 0
        || network.profile.length_mm == 0
        || network.profile.default_maxheight_mm == 0
        || network.profile.axle_load_kg == Some(0)
        || network.terminal_source_pins.values().any(|v| !digest(v))
        || network.terminal_attachment_limit_meters == 0
    {
        return Err(PhysicalPath);
    }
    if network.terminal_policy.terminal_evidence_class != "Designed"
        || network.terminal_policy.attachment_limit_mm
            != network
                .terminal_attachment_limit_meters
                .checked_mul(1000)
                .ok_or(PhysicalPath)?
        || network
            .terminal_source_pins
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != [
                "atlas_pin_sha256",
                "atlas_sha256",
                "defines_sha256",
                "graph_sha256",
            ]
        || network.terminal_source_pins.get("graph_sha256") != Some(&network.source.graph_sha256)
    {
        return Err(PhysicalPath);
    }

    Ok(())
}

fn selected_geometry(
    c: &MichiganNormalizedContent,
    network: &MichiganPhysicalNetwork,
    edges: &BTreeMap<&str, &MichiganPhysicalEdge>,
) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::PhysicalPath;
    for t in &network.terminals {
        if !county(&t.county_geoid)
            || t.evidence_class != "Designed"
            || t.attachment_distance_mm
                > network
                    .terminal_attachment_limit_meters
                    .checked_mul(1000)
                    .ok_or(PhysicalPath)?
        {
            return Err(PhysicalPath);
        }
    }
    for edge in &network.edges {
        if edge.distance_mm == 0
            || edge.shape_e7.len() < 2
            || edge
                .shape_e7
                .iter()
                .any(|p| p[0].unsigned_abs() > 1_800_000_000 || p[1].unsigned_abs() > 900_000_000)
        {
            return Err(PhysicalPath);
        }
    }
    let mut group_keys = BTreeSet::new();
    for group in &network.capacity_groups {
        if !group_keys.insert(&group.key)
            || group.edge_keys.is_empty()
            || group.edge_keys.iter().collect::<BTreeSet<_>>().len() != group.edge_keys.len()
            || group
                .edge_keys
                .iter()
                .any(|key| !edges.contains_key(key.as_str()))
            || !c
                .corridors
                .iter()
                .any(|r| r.key == group.key && r.label == group.label)
        {
            return Err(PhysicalPath);
        }
    }

    Ok(())
}

fn directed_paths(
    c: &MichiganNormalizedContent,
    network: &MichiganPhysicalNetwork,
    edges: &BTreeMap<&str, &MichiganPhysicalEdge>,
    terminals: &BTreeMap<&str, &MichiganCountyTerminal>,
) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::PhysicalPath;
    let mut used = BTreeSet::new();
    let mut memberships: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for group in &network.capacity_groups {
        for key in &group.edge_keys {
            memberships.entry(key).or_default().insert(&group.key);
        }
    }
    for route in &c.routes {
        let MichiganMaterialPath::Routed {
            physical_edge_keys,
            distance_mm,
            capacity_keys,
            ..
        } = &route.path
        else {
            continue;
        };
        let supplier = c
            .sites
            .iter()
            .find(|s| s.key == route.supplier_site_key)
            .ok_or(PhysicalPath)?;
        let buyer = c
            .sites
            .iter()
            .find(|s| s.key == route.buyer_site_key)
            .ok_or(PhysicalPath)?;
        let source = terminals
            .get(supplier.county_geoid.as_str())
            .ok_or(PhysicalPath)?;
        let destination = terminals
            .get(buyer.county_geoid.as_str())
            .ok_or(PhysicalPath)?;
        if physical_edge_keys.is_empty() {
            return Err(PhysicalPath);
        }
        let mut previous = source.node_id;
        let mut distance = 0_u64;
        let mut groups = BTreeSet::new();
        for key in physical_edge_keys {
            let edge = edges.get(key.as_str()).ok_or(PhysicalPath)?;
            if edge.from_node != previous {
                return Err(PhysicalPath);
            }
            previous = edge.to_node;
            distance = distance.checked_add(edge.distance_mm).ok_or(PhysicalPath)?;
            used.insert(key.as_str());
            groups.extend(
                memberships
                    .get(key.as_str())
                    .ok_or(PhysicalPath)?
                    .iter()
                    .copied(),
            );
        }
        if previous != destination.node_id || *distance_mm != Some(distance) {
            return Err(PhysicalPath);
        }
        if groups != capacity_keys.iter().map(String::as_str).collect() {
            return Err(PhysicalPath);
        }
    }
    if used != edges.keys().copied().collect() {
        return Err(PhysicalPath);
    }

    Ok(())
}
