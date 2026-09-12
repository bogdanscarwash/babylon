//! Source qualification is reference authoring; no allocation or tick runs here.
use super::{
    regional, sha256_of, MichiganDefines, MichiganDefinesError, MichiganDeliveryPreset,
    MichiganFinalDemand, MichiganIndustryBaselineRow, MichiganIntervention,
    MichiganMaterialCatalog, MichiganMaterialCorridor, MichiganMaterialError, MichiganMaterialGood,
    MichiganMaterialInput, MichiganMaterialPath, MichiganMaterialProcess, MichiganMaterialRoute,
    MichiganMaterialSite, MichiganMerchant, MichiganNormalizedContent, MichiganPhysicalNetwork,
    MichiganSiteRole, MichiganWorkforceSeed, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES,
};
use crate::michigan_defines::{CommodityDisposition, CommodityUnit};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
const ROSTER: &[u8] = include_bytes!(
    "../../../../../src/babylon/data/reference/economy/michigan_commodity_roster_mi_2024.json.gz"
);
const ROSTER_SHA256: &str = "a56de97bf8a0ba59a41cdfb2fc237781faf3a89b8c0a68bdb7e852ddd170389d";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Qualification {
    schema: String,
    evidence_class: String,
    qualified: bool,
    owners: Vec<Owner>,
    processes: Vec<Process>,
    orders: Vec<Order>,
    retail_final_demands: Vec<Demand>,
    diagnostics: Vec<()>,
    defines_sha256: String,
    roster_sha256: String,
    paths_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    county_geoid: String,
    sector_code: String,
    role: String,
    primary_family: Option<String>,
    eligible_families: Vec<String>,
    source_file: String,
    source_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Process {
    county_geoid: String,
    sector_code: String,
    family: String,
    enrollment: String,
    source_industry_code: String,
    source_file: String,
    source_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Order {
    supplier_county_geoid: String,
    supplier_sector_code: String,
    buyer_county_geoid: String,
    buyer_sector_code: String,
    good: String,
    unit: String,
    units: u64,
    supplier_family: Option<String>,
    buyer_families: Vec<String>,
    purposes: Vec<String>,
    local: bool,
    distance_mm: u64,
    edge_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Demand {
    county_geoid: String,
    sector_code: String,
    good: String,
    unit: String,
    units: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Roster {
    schema: String,
    vintage: u16,
    source_manifest_sha256: String,
    sector_context: SectorContext,
    sources: Vec<Source>,
    industries: Vec<IndustryObservation>,
    actors: Vec<Actor>,
    excluded_cohorts: Vec<Excluded>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SectorContext {
    path: String,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    county_geoid: String,
    file: String,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    county_geoid: String,
    sector_code: String,
    role: String,
    primary_family: Option<String>,
    eligible_families: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Excluded {
    county_geoid: String,
    sector_code: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndustryObservation {
    county_geoid: String,
    sector_code: String,
    industry_code: String,
    agglvl_code: String,
    industry_title: String,
    disclosure_code: String,
    annual_avg_estabs_count: u64,
    annual_avg_emplvl: Option<u64>,
    total_annual_wages: Option<u64>,
    annual_avg_wkly_wage: Option<u64>,
}
fn site_key(county: &str, sector: &str) -> String {
    format!("owner-{county}-{sector}")
}
fn process_key(county: &str, sector: &str, family: &str) -> String {
    format!("{county}-{sector}-{family}")
}
fn error() -> MichiganDefinesError {
    MichiganDefinesError::Material(MichiganMaterialError::ContentReference)
}
fn role(value: &str) -> Result<MichiganSiteRole, MichiganDefinesError> {
    match value {
        "producer" => Ok(MichiganSiteRole::Production),
        "wholesaler" => Ok(MichiganSiteRole::Wholesale),
        "retailer" => Ok(MichiganSiteRole::Retail),
        _ => Err(error()),
    }
}
fn load_roster() -> Result<Roster, MichiganDefinesError> {
    if crate::michigan_economy::digest_hex(&sha256_of(ROSTER)) != ROSTER_SHA256 {
        return Err(error());
    }
    let mut decoded = Vec::new();
    flate2::read::GzDecoder::new(ROSTER)
        .take(4_194_305)
        .read_to_end(&mut decoded)
        .map_err(|_| error())?;
    if decoded.len() > 4_194_304 {
        return Err(error());
    }
    let r: Roster = serde_json::from_slice(&decoded).map_err(|_| error())?;
    if r.schema != "MichiganCommodityRosterV1"
        || r.vintage != 2024
        || r.actors.len() != 397
        || r.sources.len() != 83
        || r.source_manifest_sha256.len() != 64
        || r.sector_context.path
            != "src/babylon/data/reference/economy/qcew_county_sectors_mi_2024.csv.gz"
        || r.sector_context.sha256 != crate::michigan_sectors::QCEW_SECTORS_ARTIFACT_SHA256
        || r.excluded_cohorts.len() != 5
        || r.excluded_cohorts.iter().any(|e| {
            e.sector_code != "21"
                || e.reason != "support-only"
                || !matches!(
                    e.county_geoid.as_str(),
                    "26011" | "26019" | "26039" | "26089" | "26119"
                )
        })
    {
        return Err(error());
    }
    Ok(r)
}
pub(super) fn compile(
    text: &str,
    bytes: &[u8],
    physical: MichiganPhysicalNetwork,
    interventions: Vec<MichiganIntervention>,
) -> Result<MichiganMaterialCatalog, MichiganDefinesError> {
    if bytes.len() > MAX_MICHIGAN_CAPTURED_CONTENT_BYTES {
        return Err(error());
    }
    let defines = MichiganDefines::parse(text)?;
    let qualification: Qualification = serde_json::from_slice(bytes).map_err(|_| error())?;
    if qualification.schema != "MichiganCommodityCircuitV1"
        || qualification.evidence_class != "Designed"
        || !qualification.qualified
        || !qualification.diagnostics.is_empty()
        || qualification.defines_sha256
            != crate::michigan_economy::digest_hex(&sha256_of(text.as_bytes()))
        || qualification.roster_sha256 != ROSTER_SHA256
        || qualification.paths_sha256.len() != 64
    {
        return Err(error());
    }
    let roster = load_roster()?;
    let mut c = regional::blank(&defines);
    "finite_county_final_demand".clone_into(&mut c.terminal_output_disposition);
    append_owners(&mut c, &defines, &qualification, &roster)?;
    append_industry(&mut c, roster, &qualification.owners)?;
    for (name, g) in &defines.commodity {
        c.goods.push(MichiganMaterialGood {
            key: name.clone(),
            label: name.replace('_', " "),
            unit_key: match g.unit {
                CommodityUnit::Kilogram => "kg",
                CommodityUnit::Item => "item",
            }
            .to_owned(),
            grams_per_unit: g.grams_per_unit,
        });
    }
    append_processes(&mut c, &defines, &qualification)?;
    for group in &physical.capacity_groups {
        c.corridors.push(MichiganMaterialCorridor {
            key: group.key.clone(),
            label: group.label.clone(),
            capacity_grams_per_period: defines.transport.road_capacity_grams_per_period,
        });
    }
    append_routes(&mut c, &defines, &qualification.orders, &physical)?;
    append_merchants_and_final_demand(&mut c, &defines, &qualification.retail_final_demands)?;
    c.physical_network = Some(physical);
    MichiganMaterialCatalog::from_normalized(
        defines,
        c,
        MichiganDeliveryPreset::StatewideBaseline,
        interventions,
    )
}

fn append_owners(
    c: &mut MichiganNormalizedContent,
    defines: &MichiganDefines,
    qualification: &Qualification,
    roster: &Roster,
) -> Result<(), MichiganDefinesError> {
    if qualification.owners.len() != roster.actors.len() {
        return Err(error());
    }
    let mut seen = BTreeSet::new();
    for owner in &qualification.owners {
        if !seen.insert((&owner.county_geoid, &owner.sector_code)) {
            return Err(error());
        }
        let actor = roster
            .actors
            .iter()
            .find(|a| a.county_geoid == owner.county_geoid && a.sector_code == owner.sector_code)
            .ok_or_else(error)?;
        let source = roster
            .sources
            .iter()
            .find(|s| s.county_geoid == owner.county_geoid)
            .ok_or_else(error)?;
        if owner.role != actor.role
            || owner.primary_family != actor.primary_family
            || owner.eligible_families != actor.eligible_families
            || owner.source_file != source.file
            || owner.source_sha256 != source.sha256
        {
            return Err(error());
        }
        let owner_source =
            regional::owner_source(&owner.county_geoid, &owner.sector_code, ROSTER_SHA256)?;
        if owner_source.county_source_file != source.file
            || owner_source.county_source_sha256 != source.sha256
        {
            return Err(error());
        }
        c.owners.push(owner_source);
        append_owner_material(c, defines, qualification, roster, owner, source)?;
    }

    Ok(())
}

fn append_owner_material(
    c: &mut MichiganNormalizedContent,
    defines: &MichiganDefines,
    qualification: &Qualification,
    roster: &Roster,
    owner: &Owner,
    source: &Source,
) -> Result<(), MichiganDefinesError> {
    let industry = if let Some(family) = &owner.primary_family {
        qualification
            .processes
            .iter()
            .find(|p| {
                p.county_geoid == owner.county_geoid
                    && p.sector_code == owner.sector_code
                    && p.family == *family
                    && p.enrollment == "primary"
            })
            .map(|p| p.source_industry_code.as_str())
            .ok_or_else(error)?
    } else {
        roster
            .industries
            .iter()
            .filter(|r| {
                r.county_geoid == owner.county_geoid
                    && r.sector_code == owner.sector_code
                    && r.industry_code != "425"
            })
            .max_by(|a, b| {
                a.annual_avg_estabs_count
                    .cmp(&b.annual_avg_estabs_count)
                    .then_with(|| b.industry_code.cmp(&a.industry_code))
            })
            .map(|r| r.industry_code.as_str())
            .ok_or_else(error)?
    };
    let site = site_key(&owner.county_geoid, &owner.sector_code);
    c.sites.push(MichiganMaterialSite {
        key: site.clone(),
        label: format!(
            "{} · {}",
            source
                .file
                .strip_prefix(&format!("2024.annual {} ", owner.county_geoid))
                .and_then(|v| v.strip_suffix(", Michigan.csv"))
                .ok_or_else(error)?,
            owner
                .primary_family
                .as_deref()
                .unwrap_or(&owner.role)
                .replace('_', " ")
        ),
        county_geoid: owner.county_geoid.clone(),
        naics: industry.to_owned(),
        sector_code: owner.sector_code.clone(),
        role: role(&owner.role)?,
    });
    let (employed, reserve) = if let Some(family) = &owner.primary_family {
        let t = defines.template.get(family).ok_or_else(error)?;
        (t.employed_people, t.reserve_people)
    } else {
        (
            defines.merchant.employed_people,
            defines.merchant.reserve_people,
        )
    };
    c.staffing.pools.push(MichiganWorkforceSeed {
        key: site.clone(),
        site_key: site,
        process_keys: Vec::new(),
        merchant_handling: owner.primary_family.is_none(),
        employed,
        reserve,
        previous_unretained_hours: employed
            .checked_mul(defines.hours_per_period())
            .ok_or_else(error)?,
    });

    Ok(())
}

fn append_industry(
    c: &mut MichiganNormalizedContent,
    roster: Roster,
    owners: &[Owner],
) -> Result<(), MichiganDefinesError> {
    let seen: BTreeSet<_> = owners
        .iter()
        .map(|owner| (&owner.county_geoid, &owner.sector_code))
        .collect();
    for r in roster.industries {
        if !seen.contains(&(&r.county_geoid, &r.sector_code)) {
            continue;
        }
        let source = roster
            .sources
            .iter()
            .find(|s| s.county_geoid == r.county_geoid)
            .ok_or_else(error)?;
        c.industry.push(MichiganIndustryBaselineRow {
            area_fips: r.county_geoid,
            area_title: source.file.clone(),
            industry_code: r.industry_code,
            industry_title: r.industry_title,
            own_code: "5".to_owned(),
            agglvl_code: r.agglvl_code,
            disclosure_code: r.disclosure_code,
            annual_avg_estabs_count: r.annual_avg_estabs_count,
            annual_avg_emplvl: r.annual_avg_emplvl,
            total_annual_wages: r.total_annual_wages,
            annual_avg_wkly_wage: r.annual_avg_wkly_wage,
            source_file: source.file.clone(),
            source_sha256: source.sha256.clone(),
        });
    }

    Ok(())
}

fn append_processes(
    c: &mut MichiganNormalizedContent,
    defines: &MichiganDefines,
    qualification: &Qualification,
) -> Result<(), MichiganDefinesError> {
    for p in &qualification.processes {
        let owner = qualification
            .owners
            .iter()
            .find(|o| o.county_geoid == p.county_geoid && o.sector_code == p.sector_code)
            .ok_or_else(error)?;
        if !owner.eligible_families.contains(&p.family)
            || p.source_file != owner.source_file
            || p.source_sha256 != owner.source_sha256
            || !matches!(p.enrollment.as_str(), "primary" | "upstream_completion")
        {
            return Err(error());
        }
        let t = defines.template.get(&p.family).ok_or_else(error)?;
        let site = site_key(&p.county_geoid, &p.sector_code);
        let key = process_key(&p.county_geoid, &p.sector_code, &p.family);
        let capacity = t
            .batches_per_week
            .checked_mul(babylon_kernel::clock::WEEKS_PER_TICK)
            .ok_or_else(error)?;
        c.processes.push(MichiganMaterialProcess {
            key: key.clone(),
            site_key: site.clone(),
            industry_code: p.source_industry_code.clone(),
            inputs: t
                .input_units_per_batch
                .iter()
                .map(|(good, n)| MichiganMaterialInput {
                    good_key: good.clone(),
                    quantity_per_batch: *n,
                    opening_quantity: t.opening_input_units[good],
                })
                .collect(),
            output_good_key: t.output_good.clone(),
            output_quantity_per_batch: t.output_units_per_batch,
            capacity_batches_per_period: capacity,
            labor_hours_per_batch: t.labor_hours_per_batch,
            opening_planned_batches: capacity,
        });
        c.staffing
            .pools
            .iter_mut()
            .find(|pool| pool.site_key == site)
            .ok_or_else(error)?
            .process_keys
            .push(key);
    }

    Ok(())
}

fn append_routes(
    c: &mut MichiganNormalizedContent,
    defines: &MichiganDefines,
    orders: &[Order],
    physical: &MichiganPhysicalNetwork,
) -> Result<(), MichiganDefinesError> {
    for order in orders {
        let supplier = site_key(&order.supplier_county_geoid, &order.supplier_sector_code);
        let buyer = site_key(&order.buyer_county_geoid, &order.buyer_sector_code);
        if order.purposes.is_empty()
            || order.purposes.iter().any(|p| {
                !matches!(
                    p.as_str(),
                    "production_input" | "merchant_purchase" | "merchant_resale"
                )
            })
            || (order.supplier_family.is_some()
                && !c
                    .processes
                    .iter()
                    .any(|p| p.site_key == supplier && p.output_good_key == order.good))
            || order
                .buyer_families
                .iter()
                .any(|f| !defines.template.contains_key(f))
        {
            return Err(error());
        }
        if defines
            .commodity
            .get(&order.good)
            .is_none_or(|g| g.disposition != CommodityDisposition::Traded)
            || !c
                .goods
                .iter()
                .any(|g| g.key == order.good && g.unit_key == order.unit)
            || order.local != (order.supplier_county_geoid == order.buyer_county_geoid)
        {
            return Err(error());
        }
        if supplier == buyer {
            continue;
        }
        let path = if order.local {
            if order.distance_mm != 0 || !order.edge_ids.is_empty() {
                return Err(error());
            }
            MichiganMaterialPath::Local
        } else {
            let keys: BTreeSet<_> = order.edge_ids.iter().collect();
            MichiganMaterialPath::Routed {
                travel_periods: defines.transport.road_travel_periods,
                capacity_keys: physical
                    .capacity_groups
                    .iter()
                    .filter(|g| g.edge_keys.iter().any(|k| keys.contains(k)))
                    .map(|g| g.key.clone())
                    .collect(),
                physical_edge_keys: order.edge_ids.clone(),
                distance_mm: Some(order.distance_mm),
            }
        };
        c.routes.push(MichiganMaterialRoute {
            key: format!("{supplier}:{buyer}:{}", order.good),
            supplier_site_key: supplier,
            buyer_site_key: buyer,
            good_key: order.good.clone(),
            ordered_quantity: order.units,
            path,
        });
    }

    Ok(())
}

fn append_merchants_and_final_demand(
    c: &mut MichiganNormalizedContent,
    defines: &MichiganDefines,
    demands: &[Demand],
) -> Result<(), MichiganDefinesError> {
    for d in demands {
        if !c
            .goods
            .iter()
            .any(|g| g.key == d.good && g.unit_key == d.unit)
        {
            return Err(error());
        }
        c.final_demands.push(MichiganFinalDemand {
            key: format!("{}:{}:{}", d.county_geoid, d.sector_code, d.good),
            retailer_site_key: site_key(&d.county_geoid, &d.sector_code),
            county_geoid: d.county_geoid.clone(),
            good_key: d.good.clone(),
            ordered_quantity: d.units,
        });
    }
    for site in c
        .sites
        .iter()
        .filter(|s| s.role != MichiganSiteRole::Production)
    {
        let capacity_key = format!("handling-{}", site.key);
        let good_keys: BTreeSet<_> = c
            .routes
            .iter()
            .filter(|r| r.supplier_site_key == site.key)
            .map(|r| &r.good_key)
            .chain(
                c.final_demands
                    .iter()
                    .filter(|d| d.retailer_site_key == site.key)
                    .map(|d| &d.good_key),
            )
            .collect();
        let handling_hours_per_unit = good_keys
            .into_iter()
            .map(|key| {
                Ok((
                    key.clone(),
                    match defines.commodity.get(key).ok_or_else(error)?.unit {
                        CommodityUnit::Kilogram => defines.merchant.labor_hours_per_kg,
                        CommodityUnit::Item => defines.merchant.labor_hours_per_item,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, MichiganDefinesError>>()?;
        c.corridors.push(MichiganMaterialCorridor {
            key: capacity_key.clone(),
            label: format!("{} handling", site.label),
            capacity_grams_per_period: defines.merchant.handling_grams_per_period,
        });
        c.merchants.push(MichiganMerchant {
            site_key: site.key.clone(),
            capacity_key,
            handling_hours_per_unit,
        });
    }

    Ok(())
}
