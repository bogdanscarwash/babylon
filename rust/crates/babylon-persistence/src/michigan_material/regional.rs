//! Regional authoring translates to the same normalized rows as statewide content.
use super::{
    sha256_of, MichiganCapacityOverrideV2, MichiganDefinesErrorV1, MichiganDefinesV3,
    MichiganDeliveryPresetV1, MichiganIndustryBaselineRowV1, MichiganInterventionV2,
    MichiganMaterialCatalogV1, MichiganMaterialCorridorV1, MichiganMaterialErrorV1,
    MichiganMaterialGoodV1, MichiganMaterialInputV2, MichiganMaterialPathV2,
    MichiganMaterialProcessV1, MichiganMaterialRouteV1, MichiganMaterialSiteV1,
    MichiganNormalizedContentV2, MichiganOwnerSourceV2, MichiganRouteOverrideV2,
    MichiganSiteRoleV2, MichiganStaffingDesignV1, MichiganWorkforceSeedV1,
    MICHIGAN_INDUSTRY_BASELINE_SHA256_V1, SOURCE_URL,
};
use crate::michigan_sectors::{
    michigan_county_sectors_v1, QCEW_SECTORS_ARTIFACT_SHA256_V1, QCEW_SECTORS_SEMANTIC_SHA256_V1,
};
use serde::Deserialize;
use std::collections::BTreeSet;
const INDUSTRY: &[u8] =
    include_bytes!("../../../../../contracts/fixtures/michigan_industry_baseline_v1.json");
const TOPOLOGY: &[u8] = include_bytes!("../../../../../content/scenarios/michigan/topology.json");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Industry {
    schema: String,
    vintage: u16,
    evidence_class: String,
    source_url: String,
    documentation_url: String,
    rows: Vec<MichiganIndustryBaselineRowV1>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Topology {
    sites: Vec<Site>,
    goods: Vec<Good>,
    processes: Vec<Process>,
    routes: Vec<Route>,
    corridors: Vec<Corridor>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Site {
    key: String,
    label: String,
    county_geoid: String,
    naics: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Good {
    key: String,
    label: String,
    unit_key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Process {
    key: String,
    site_key: String,
    input_good_key: String,
    output_good_key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    key: String,
    corridor_key: String,
    shared_corridor_key: String,
    supplier_site_key: String,
    buyer_site_key: String,
    good_key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Corridor {
    key: String,
    label: String,
    unit_key: String,
}

pub(super) fn blank(defines: &MichiganDefinesV3) -> MichiganNormalizedContentV2 {
    MichiganNormalizedContentV2 {
        schema: "MichiganNormalizedContentV2".to_owned(),
        evidence_class: "Designed".to_owned(),
        horizon_ticks: defines.horizon_periods,
        tick_duration_days: defines.tick_duration_days,
        geographic_scale: "county_industry_aggregate".to_owned(),
        terminal_output_disposition: "on_hand_unsold".to_owned(),
        sites: Vec::new(),
        goods: Vec::new(),
        processes: Vec::new(),
        routes: Vec::new(),
        corridors: Vec::new(),
        merchants: Vec::new(),
        final_demands: Vec::new(),
        owners: Vec::new(),
        industry: Vec::new(),
        physical_network: None,
        staffing: MichiganStaffingDesignV1 {
            composition_id: "g4-workforce-staffing".to_owned(),
            role: "Mechanic".to_owned(),
            evidence_class: "Designed".to_owned(),
            placement: "after-metabolism-material-base".to_owned(),
            hours_per_worker_period: defines.hours_per_period(),
            retention_periods: 1,
            pools: Vec::new(),
        },
    }
}
pub(super) fn owner_source(
    county: &str,
    sector: &str,
    industry_hash: &str,
) -> Result<MichiganOwnerSourceV2, MichiganDefinesErrorV1> {
    let source = michigan_county_sectors_v1()
        .map_err(|_| MichiganDefinesErrorV1::Material(MichiganMaterialErrorV1::SourceValue))?;
    let row = source
        .rows()
        .iter()
        .find(|row| row.county_geoid() == county && row.sector_code().as_str() == sector)
        .ok_or(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::ContentReference,
        ))?;
    Ok(MichiganOwnerSourceV2 {
        county_geoid: county.to_owned(),
        sector_code: sector.to_owned(),
        county_source_file: row.source_file().to_owned(),
        county_source_sha256: row.source_sha256().to_owned(),
        sector_title: row.sector_title().to_owned(),
        sector_disposition: row.sector_code().disposition().as_str().to_owned(),
        disclosure_code: row.disclosure().as_str().to_owned(),
        annual_avg_estabs_count: row.annual_avg_estabs_count(),
        annual_avg_emplvl: row.annual_avg_emplvl(),
        total_annual_wages: row.total_annual_wages(),
        annual_avg_wkly_wage: row.annual_avg_wkly_wage(),
        sector_artifact_sha256: QCEW_SECTORS_ARTIFACT_SHA256_V1.to_owned(),
        sector_semantic_sha256: QCEW_SECTORS_SEMANTIC_SHA256_V1.to_owned(),
        industry_artifact_sha256: industry_hash.to_owned(),
    })
}
fn mass(defines: &MichiganDefinesV3, unit: &str) -> Result<u64, MichiganDefinesErrorV1> {
    match unit {
        "kg" => Ok(defines.regional_mass.kilogram_grams_per_unit),
        "panel" => Ok(defines.regional_mass.panel_grams_per_unit),
        "subassembly" => Ok(defines.regional_mass.subassembly_grams_per_unit),
        _ => Err(MichiganDefinesErrorV1::Material(
            MichiganMaterialErrorV1::ContentReference,
        )),
    }
}
pub(super) fn compile(
    defines: MichiganDefinesV3,
) -> Result<MichiganMaterialCatalogV1, MichiganDefinesErrorV1> {
    use MichiganDefinesErrorV1::Material;
    if crate::michigan_economy::digest_hex(&sha256_of(INDUSTRY))
        != MICHIGAN_INDUSTRY_BASELINE_SHA256_V1
    {
        return Err(Material(MichiganMaterialErrorV1::ArtifactDigest));
    }
    let source: Industry = serde_json::from_slice(INDUSTRY)
        .map_err(|_| Material(MichiganMaterialErrorV1::ArtifactDecode))?;
    if source.schema != "MichiganIndustryBaselineV1"
        || source.vintage != 2024
        || source.evidence_class != "Observed"
        || source.source_url != SOURCE_URL
        || source.documentation_url != "https://www.bls.gov/cew/downloadable-data-files.htm"
        || source.rows.len() != 5
    {
        return Err(Material(MichiganMaterialErrorV1::ArtifactShape));
    }
    let topology: Topology = serde_json::from_slice(TOPOLOGY)
        .map_err(|_| Material(MichiganMaterialErrorV1::ArtifactDecode))?;
    let mut normalized = blank(&defines);
    normalized.industry = source.rows;
    normalized.sites = topology
        .sites
        .into_iter()
        .map(|s| MichiganMaterialSiteV1 {
            key: s.key,
            label: s.label,
            county_geoid: s.county_geoid,
            naics: s.naics,
            sector_code: "31-33".to_owned(),
            role: MichiganSiteRoleV2::Production,
        })
        .collect();
    for county in normalized
        .sites
        .iter()
        .map(|site| &site.county_geoid)
        .collect::<BTreeSet<_>>()
    {
        normalized.owners.push(owner_source(
            county,
            "31-33",
            MICHIGAN_INDUSTRY_BASELINE_SHA256_V1,
        )?);
    }
    for good in topology.goods {
        normalized.goods.push(MichiganMaterialGoodV1 {
            grams_per_unit: mass(&defines, &good.unit_key)?,
            key: good.key,
            label: good.label,
            unit_key: good.unit_key,
        });
    }
    append_processes(&mut normalized, &defines, topology.processes)?;
    let interventions = append_transport(
        &mut normalized,
        &defines,
        topology.routes,
        topology.corridors,
    )?;
    MichiganMaterialCatalogV1::from_normalized(
        defines,
        normalized,
        MichiganDeliveryPresetV1::Standard,
        interventions,
    )
}

fn append_processes(
    normalized: &mut MichiganNormalizedContentV2,
    defines: &MichiganDefinesV3,
    processes: Vec<Process>,
) -> Result<(), MichiganDefinesErrorV1> {
    use MichiganDefinesErrorV1::Material;
    for process in processes {
        let value = defines
            .process
            .get(&process.key.replace('-', "_"))
            .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?;
        let site = normalized
            .sites
            .iter()
            .find(|site| site.key == process.site_key)
            .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?;
        normalized.staffing.pools.push(MichiganWorkforceSeedV1 {
            key: process.key.clone(),
            site_key: site.key.clone(),
            process_keys: vec![process.key.clone()],
            merchant_handling: false,
            employed: value.employed_people,
            reserve: value.reserve_people,
            previous_unretained_hours: value
                .employed_people
                .checked_mul(defines.hours_per_period())
                .ok_or(Material(MichiganMaterialErrorV1::ContentValue))?,
        });
        normalized.processes.push(MichiganMaterialProcessV1 {
            key: process.key,
            site_key: site.key.clone(),
            industry_code: site.naics.clone(),
            inputs: vec![MichiganMaterialInputV2 {
                good_key: process.input_good_key,
                quantity_per_batch: value.input_units_per_batch,
                opening_quantity: value.opening_input_units,
            }],
            output_good_key: process.output_good_key,
            output_quantity_per_batch: value.output_units_per_batch,
            capacity_batches_per_period: value.batches_per_week
                * babylon_kernel::clock::WEEKS_PER_TICK,
            labor_hours_per_batch: value.labor_hours_per_batch,
            opening_planned_batches: value.opening_planned_batches,
        });
    }

    Ok(())
}

fn append_transport(
    normalized: &mut MichiganNormalizedContentV2,
    defines: &MichiganDefinesV3,
    routes: Vec<Route>,
    corridors: Vec<Corridor>,
) -> Result<Vec<MichiganInterventionV2>, MichiganDefinesErrorV1> {
    use MichiganDefinesErrorV1::Material;
    let mut delayed = MichiganInterventionV2 {
        preset: MichiganDeliveryPresetV1::Delayed,
        capacities: Vec::new(),
        opening_stocks: Vec::new(),
        routes: Vec::new(),
    };
    let mut ample = MichiganInterventionV2 {
        preset: MichiganDeliveryPresetV1::SharedFreightAmple,
        capacities: Vec::new(),
        opening_stocks: Vec::new(),
        routes: Vec::new(),
    };
    for route in routes {
        let value = defines
            .route
            .get(&route.key.replace('-', "_"))
            .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?;
        let path = |periods, key: String| MichiganMaterialPathV2::Routed {
            travel_periods: periods,
            capacity_keys: vec![key],
            physical_edge_keys: Vec::new(),
            distance_mm: None,
        };
        delayed.routes.push(MichiganRouteOverrideV2 {
            route_key: route.key.clone(),
            path: path(value.delayed_travel_periods, route.corridor_key.clone()),
        });
        ample.routes.push(MichiganRouteOverrideV2 {
            route_key: route.key.clone(),
            path: path(value.travel_periods, route.shared_corridor_key),
        });
        normalized.routes.push(MichiganMaterialRouteV1 {
            key: route.key,
            supplier_site_key: route.supplier_site_key,
            buyer_site_key: route.buyer_site_key,
            good_key: route.good_key,
            ordered_quantity: value.ordered_units,
            path: path(value.travel_periods, route.corridor_key),
        });
    }
    for corridor in corridors {
        let rate = if corridor.key == "shared-freight" {
            defines.shared_freight.ample_units_per_week
        } else {
            defines
                .corridor
                .get(&corridor.key.replace('-', "_"))
                .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?
                .units_per_week
        };
        normalized.corridors.push(MichiganMaterialCorridorV1 {
            key: corridor.key,
            label: corridor.label,
            capacity_grams_per_period: rate
                .checked_mul(babylon_kernel::clock::WEEKS_PER_TICK)
                .and_then(|v| v.checked_mul(mass(defines, &corridor.unit_key).ok()?))
                .ok_or(Material(MichiganMaterialErrorV1::ContentValue))?,
        });
    }
    let mut constrained = ample.clone();
    constrained.preset = MichiganDeliveryPresetV1::SharedFreightConstrained;
    constrained.capacities.push(MichiganCapacityOverrideV2 {
        capacity_key: "shared-freight".to_owned(),
        grams_per_period: defines
            .shared_freight
            .constrained_units_per_week
            .checked_mul(babylon_kernel::clock::WEEKS_PER_TICK)
            .and_then(|v| v.checked_mul(defines.regional_mass.kilogram_grams_per_unit))
            .ok_or(Material(MichiganMaterialErrorV1::ContentValue))?,
    });

    Ok(vec![delayed, ample, constrained])
}
