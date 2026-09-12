//! Admission checks consume captured authority only, including on restart.
use super::{
    MichiganDeliveryPreset, MichiganIntervention, MichiganMaterialCorridor, MichiganMaterialError,
    MichiganMaterialGood, MichiganMaterialPath, MichiganMaterialSite, MichiganNormalizedContent,
    MichiganOwnerSource, MichiganSiteRole,
};
use std::collections::{BTreeMap, BTreeSet};
mod physical;
fn key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.'))
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn county(value: &str) -> bool {
    value.len() == 5 && value.starts_with("26") && value.bytes().all(|b| b.is_ascii_digit())
}
fn unique<'a>(mut values: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    values.all(|v| key(v) && seen.insert(v))
}
pub(super) fn canonicalize(
    c: &mut MichiganNormalizedContent,
    interventions: &mut [MichiganIntervention],
) {
    c.sites.sort_by(|a, b| a.key.cmp(&b.key));
    c.goods.sort_by(|a, b| a.key.cmp(&b.key));
    c.processes.sort_by(|a, b| a.key.cmp(&b.key));
    for p in &mut c.processes {
        p.inputs.sort_by(|a, b| a.good_key.cmp(&b.good_key));
    }
    c.routes.sort_by(|a, b| a.key.cmp(&b.key));
    c.corridors.sort_by(|a, b| a.key.cmp(&b.key));
    for r in &mut c.routes {
        path(&mut r.path);
    }
    c.staffing.pools.sort_by(|a, b| a.key.cmp(&b.key));
    for p in &mut c.staffing.pools {
        p.process_keys.sort();
    }
    c.merchants.sort_by(|a, b| a.site_key.cmp(&b.site_key));
    c.final_demands.sort_by(|a, b| a.key.cmp(&b.key));
    c.owners
        .sort_by(|a, b| (&a.county_geoid, &a.sector_code).cmp(&(&b.county_geoid, &b.sector_code)));
    c.industry
        .sort_by(|a, b| (&a.area_fips, &a.industry_code).cmp(&(&b.area_fips, &b.industry_code)));
    if let Some(n) = &mut c.physical_network {
        n.terminals
            .sort_by(|a, b| a.county_geoid.cmp(&b.county_geoid));
        n.edges.sort_by(|a, b| a.id.cmp(&b.id));
        n.capacity_groups.sort_by(|a, b| a.key.cmp(&b.key));
        for g in &mut n.capacity_groups {
            g.edge_keys.sort();
        }
    }
    interventions.sort_by_key(|i| i.preset);
    for i in interventions {
        i.capacities
            .sort_by(|a, b| a.capacity_key.cmp(&b.capacity_key));
        i.opening_stocks
            .sort_by(|a, b| (&a.process_key, &a.good_key).cmp(&(&b.process_key, &b.good_key)));
        i.routes.sort_by(|a, b| a.route_key.cmp(&b.route_key));
        for r in &mut i.routes {
            path(&mut r.path);
        }
    }
}
fn path(path: &mut MichiganMaterialPath) {
    if let MichiganMaterialPath::Routed { capacity_keys, .. } = path {
        capacity_keys.sort();
    }
}
pub(super) fn content(c: &MichiganNormalizedContent) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::{ArtifactShape, ContentReference, ContentValue};
    if c.schema != "MichiganNormalizedContentV2"
        || c.evidence_class != "Designed"
        || c.tick_duration_days != babylon_kernel::clock::DAYS_PER_TICK
        || !(1..=16).contains(&c.horizon_ticks)
        || c.sites.is_empty()
        || c.sites.len() > 1024
        || c.goods.is_empty()
        || c.goods.len() > 64
        || c.processes.len() > 4096
        || c.routes.len() > 8192
        || c.corridors.len() > 4096
    {
        return Err(ArtifactShape);
    }
    if !unique(c.sites.iter().map(|r| r.key.as_str()))
        || !unique(c.goods.iter().map(|r| r.key.as_str()))
        || !unique(c.processes.iter().map(|r| r.key.as_str()))
        || !unique(c.routes.iter().map(|r| r.key.as_str()))
        || !unique(c.corridors.iter().map(|r| r.key.as_str()))
        || !unique(c.final_demands.iter().map(|r| r.key.as_str()))
    {
        return Err(ContentReference);
    }
    let sites: BTreeMap<_, _> = c.sites.iter().map(|s| (s.key.as_str(), s)).collect();
    let goods: BTreeMap<_, _> = c.goods.iter().map(|g| (g.key.as_str(), g)).collect();
    let corridors: BTreeMap<_, _> = c.corridors.iter().map(|g| (g.key.as_str(), g)).collect();
    source_authority(c)?;
    if c.goods.iter().any(|g| {
        g.grams_per_unit == 0
            || g.label.is_empty()
            || !key(&g.unit_key)
            || (g.unit_key == "kg" && g.grams_per_unit != 1000)
    }) {
        return Err(ContentValue);
    }
    let mut ceilings: BTreeMap<(&str, &str), u64> = BTreeMap::new();
    production_ceilings(c, &sites, &goods, &mut ceilings)?;
    route_ceilings(c, &sites, &goods, &corridors, &mut ceilings)?;
    for ((_, good), quantity) in ceilings {
        quantity
            .checked_mul(goods.get(good).ok_or(ContentReference)?.grams_per_unit)
            .ok_or(ContentValue)?;
    }
    workforce(c)?;
    merchants(c)?;
    physical::validate(c)?;
    Ok(())
}
fn workforce(c: &MichiganNormalizedContent) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::ContentValue;
    let d = &c.staffing;
    if d.composition_id != "g4-workforce-staffing"
        || d.role != "Mechanic"
        || d.evidence_class != "Designed"
        || d.placement != "after-metabolism-material-base"
        || d.retention_periods != 1
        || d.hours_per_worker_period == 0
        || d.hours_per_worker_period > 672
        || d.pools.len() != c.sites.len()
        || !unique(d.pools.iter().map(|p| p.key.as_str()))
    {
        return Err(ContentValue);
    }
    let mut sites = BTreeSet::new();
    for pool in &d.pools {
        if !sites.insert(pool.site_key.as_str())
            || pool
                .employed
                .checked_add(pool.reserve)
                .is_none_or(|n| n > (1 << 53))
        {
            return Err(ContentValue);
        }
        let site = c
            .sites
            .iter()
            .find(|s| s.key == pool.site_key)
            .ok_or(ContentValue)?;
        let expected: BTreeSet<_> = c
            .processes
            .iter()
            .filter(|p| p.site_key == pool.site_key)
            .map(|p| &p.key)
            .collect();
        if pool.process_keys.iter().collect::<BTreeSet<_>>() != expected
            || pool.process_keys.len() != expected.len()
            || pool.merchant_handling != (site.role != MichiganSiteRole::Production)
            || (expected.is_empty() && !pool.merchant_handling)
        {
            return Err(ContentValue);
        }
        if pool
            .employed
            .checked_add(pool.reserve)
            .and_then(|n| n.checked_mul(d.hours_per_worker_period))
            .is_none_or(|n| n > (1 << 53))
            || pool.previous_unretained_hours
                != pool
                    .employed
                    .checked_mul(d.hours_per_worker_period)
                    .ok_or(ContentValue)?
        {
            return Err(ContentValue);
        }
    }
    Ok(())
}
fn merchants(c: &MichiganNormalizedContent) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::ContentReference;
    let expected: BTreeSet<_> = c
        .sites
        .iter()
        .filter(|s| s.role != MichiganSiteRole::Production)
        .map(|s| s.key.as_str())
        .collect();
    if c.merchants
        .iter()
        .map(|m| m.site_key.as_str())
        .collect::<BTreeSet<_>>()
        != expected
        || c.merchants.len() != expected.len()
    {
        return Err(ContentReference);
    }
    let mut capacity_keys = BTreeSet::new();
    for merchant in &c.merchants {
        if !capacity_keys.insert(&merchant.capacity_key)
            || !c.corridors.iter().any(|r| r.key == merchant.capacity_key)
        {
            return Err(ContentReference);
        }
        let goods: BTreeSet<_> = c
            .routes
            .iter()
            .filter(|r| r.supplier_site_key == merchant.site_key)
            .map(|r| &r.good_key)
            .chain(
                c.final_demands
                    .iter()
                    .filter(|d| d.retailer_site_key == merchant.site_key)
                    .map(|d| &d.good_key),
            )
            .collect();
        if goods.is_empty()
            || merchant
                .handling_hours_per_unit
                .keys()
                .collect::<BTreeSet<_>>()
                != goods
            || merchant.handling_hours_per_unit.values().any(|v| *v == 0)
        {
            return Err(ContentReference);
        }
    }
    for route in &c.routes {
        if let MichiganMaterialPath::Routed {
            capacity_keys: keys,
            ..
        } = &route.path
        {
            if keys.iter().any(|key| capacity_keys.contains(key)) {
                return Err(ContentReference);
            }
        }
    }
    for demand in &c.final_demands {
        if demand.ordered_quantity == 0
            || !c.goods.iter().any(|g| g.key == demand.good_key)
            || !c.sites.iter().any(|s| {
                s.key == demand.retailer_site_key
                    && s.county_geoid == demand.county_geoid
                    && s.role == MichiganSiteRole::Retail
            })
        {
            return Err(ContentReference);
        }
    }
    Ok(())
}
pub(super) fn interventions(
    c: &MichiganNormalizedContent,
    base: MichiganDeliveryPreset,
    rows: &[MichiganIntervention],
) -> Result<(), MichiganMaterialError> {
    let mut seen = BTreeSet::new();
    for row in rows {
        if row.preset == base
            || !seen.insert(row.preset)
            || !unique(row.capacities.iter().map(|r| r.capacity_key.as_str()))
            || !unique(row.routes.iter().map(|r| r.route_key.as_str()))
        {
            return Err(MichiganMaterialError::Preset);
        }
        let mut keys = BTreeSet::new();
        if row
            .opening_stocks
            .iter()
            .any(|r| !keys.insert((&r.process_key, &r.good_key)))
        {
            return Err(MichiganMaterialError::Preset);
        }
        let mut modified = c.clone();
        apply(&mut modified, row)?;
        content(&modified)?;
    }
    Ok(())
}
pub(super) fn apply(
    c: &mut MichiganNormalizedContent,
    row: &MichiganIntervention,
) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::ContentReference;
    for item in &row.capacities {
        c.corridors
            .iter_mut()
            .find(|r| r.key == item.capacity_key)
            .ok_or(ContentReference)?
            .capacity_grams_per_period = item.grams_per_period;
    }
    for item in &row.opening_stocks {
        c.processes
            .iter_mut()
            .find(|r| r.key == item.process_key)
            .ok_or(ContentReference)?
            .inputs
            .iter_mut()
            .find(|i| i.good_key == item.good_key)
            .ok_or(ContentReference)?
            .opening_quantity = item.quantity;
    }
    for item in &row.routes {
        c.routes
            .iter_mut()
            .find(|r| r.key == item.route_key)
            .ok_or(ContentReference)?
            .path
            .clone_from(&item.path);
    }
    Ok(())
}

fn source_authority(c: &MichiganNormalizedContent) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::SourceValue;
    let owners: BTreeMap<_, _> = c
        .owners
        .iter()
        .map(|o| ((o.county_geoid.as_str(), o.sector_code.as_str()), o))
        .collect();
    if owners.len() != c.owners.len()
        || owners.keys().copied().collect::<BTreeSet<_>>()
            != c.sites
                .iter()
                .map(|s| (s.county_geoid.as_str(), s.sector_code.as_str()))
                .collect()
    {
        return Err(SourceValue);
    }
    for o in &c.owners {
        let metrics = [
            o.annual_avg_emplvl,
            o.total_annual_wages,
            o.annual_avg_wkly_wage,
        ];
        if !matches!(o.disclosure_code.as_str(), "" | "N")
            || (o.disclosure_code == "N" && metrics.iter().any(Option::is_some))
            || (o.disclosure_code.is_empty() && metrics.iter().any(Option::is_none))
            || o.annual_avg_estabs_count == 0
            || o.sector_title.is_empty()
        {
            return Err(SourceValue);
        }
        if !county(&o.county_geoid)
            || !matches!(
                o.sector_code.as_str(),
                "11" | "21" | "31-33" | "42" | "44-45"
            )
            || o.county_source_file.is_empty()
            || [
                &o.county_source_sha256,
                &o.sector_artifact_sha256,
                &o.sector_semantic_sha256,
                &o.industry_artifact_sha256,
            ]
            .into_iter()
            .any(|h| !digest(h))
        {
            return Err(SourceValue);
        }
    }

    industry_authority(c, &owners)
}

fn industry_authority(
    c: &MichiganNormalizedContent,
    owners: &BTreeMap<(&str, &str), &MichiganOwnerSource>,
) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::SourceValue;
    let mut seen_industry = BTreeSet::new();
    for row in &c.industry {
        let metrics = [
            row.annual_avg_emplvl,
            row.total_annual_wages,
            row.annual_avg_wkly_wage,
        ];
        if !seen_industry.insert((&row.area_fips, &row.industry_code))
            || !county(&row.area_fips)
            || row.annual_avg_estabs_count == 0
            || !digest(&row.source_sha256)
            || !matches!(row.disclosure_code.as_str(), "" | "N")
            || (row.disclosure_code == "N" && metrics.iter().any(Option::is_some))
            || (row.disclosure_code.is_empty() && metrics.iter().any(Option::is_none))
        {
            return Err(SourceValue);
        }
    }
    for site in &c.sites {
        let source = c
            .industry
            .iter()
            .find(|r| r.area_fips == site.county_geoid && r.industry_code == site.naics)
            .ok_or(SourceValue)?;
        let owner = owners
            .get(&(site.county_geoid.as_str(), site.sector_code.as_str()))
            .ok_or(SourceValue)?;
        if source.source_file != owner.county_source_file
            || source.source_sha256 != owner.county_source_sha256
            || site.label.is_empty()
        {
            return Err(SourceValue);
        }
    }

    Ok(())
}

fn production_ceilings<'a>(
    c: &'a MichiganNormalizedContent,
    sites: &BTreeMap<&str, &MichiganMaterialSite>,
    goods: &BTreeMap<&str, &MichiganMaterialGood>,
    ceilings: &mut BTreeMap<(&'a str, &'a str), u64>,
) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::{ContentReference, ContentValue, SourceValue};
    // Bound all possible additions independently of allocation.
    for p in &c.processes {
        let site = sites.get(p.site_key.as_str()).ok_or(ContentReference)?;
        if site.role != MichiganSiteRole::Production
            || p.inputs.is_empty()
            || p.inputs.len() > 16
            || !unique(p.inputs.iter().map(|i| i.good_key.as_str()))
            || !goods.contains_key(p.output_good_key.as_str())
            || p.output_quantity_per_batch == 0
            || p.capacity_batches_per_period == 0
            || p.labor_hours_per_batch == 0
            || p.opening_planned_batches > p.capacity_batches_per_period
        {
            return Err(ContentValue);
        }
        if !c
            .industry
            .iter()
            .any(|r| r.area_fips == site.county_geoid && r.industry_code == p.industry_code)
        {
            return Err(SourceValue);
        }
        for input in &p.inputs {
            if input.quantity_per_batch == 0
                || input.good_key == p.output_good_key
                || !goods.contains_key(input.good_key.as_str())
            {
                return Err(ContentValue);
            }
            let n = ceilings.entry((&p.site_key, &input.good_key)).or_default();
            *n = n.checked_add(input.opening_quantity).ok_or(ContentValue)?;
            input
                .quantity_per_batch
                .checked_mul(p.capacity_batches_per_period)
                .ok_or(ContentValue)?;
        }
        let n = ceilings
            .entry((&p.site_key, &p.output_good_key))
            .or_default();
        *n = n
            .checked_add(
                p.output_quantity_per_batch
                    .checked_mul(p.capacity_batches_per_period)
                    .and_then(|n| n.checked_mul(c.horizon_ticks))
                    .ok_or(ContentValue)?,
            )
            .ok_or(ContentValue)?;
        p.labor_hours_per_batch
            .checked_mul(p.capacity_batches_per_period)
            .ok_or(ContentValue)?;
    }

    Ok(())
}

fn route_ceilings<'a>(
    c: &'a MichiganNormalizedContent,
    sites: &BTreeMap<&str, &MichiganMaterialSite>,
    goods: &BTreeMap<&str, &MichiganMaterialGood>,
    corridors: &BTreeMap<&str, &MichiganMaterialCorridor>,
    ceilings: &mut BTreeMap<(&'a str, &'a str), u64>,
) -> Result<(), MichiganMaterialError> {
    use MichiganMaterialError::{ContentReference, ContentValue};
    for r in &c.routes {
        let supplier = sites
            .get(r.supplier_site_key.as_str())
            .ok_or(ContentReference)?;
        let buyer = sites
            .get(r.buyer_site_key.as_str())
            .ok_or(ContentReference)?;
        if supplier.key == buyer.key
            || r.ordered_quantity == 0
            || !goods.contains_key(r.good_key.as_str())
        {
            return Err(ContentValue);
        }
        let n = ceilings
            .entry((&r.buyer_site_key, &r.good_key))
            .or_default();
        *n = n.checked_add(r.ordered_quantity).ok_or(ContentValue)?;
        match &r.path {
            MichiganMaterialPath::Local => {
                if supplier.county_geoid != buyer.county_geoid {
                    return Err(ContentValue);
                }
            }
            MichiganMaterialPath::Routed {
                travel_periods,
                capacity_keys,
                ..
            } => {
                if *travel_periods == 0
                    || capacity_keys.is_empty()
                    || !unique(capacity_keys.iter().map(String::as_str))
                    || capacity_keys
                        .iter()
                        .any(|k| !corridors.contains_key(k.as_str()))
                {
                    return Err(ContentReference);
                }
            }
        }
    }

    Ok(())
}
