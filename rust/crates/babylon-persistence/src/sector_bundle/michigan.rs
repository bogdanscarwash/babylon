//! Authored bundle capture and executable composition; no weekly adjudication here.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use babylon_material_circuit::{
    BacklogRowV1, CapacityRowV1, CorridorCapacityV2, InputOutputCoefficientV1, InventoryRowV1,
    LaborCapacityRowV1, LaborCoefficientV1, OrderAccessModeV1, OrderRowV2, ProcessOutputV1,
    ProductionCommitmentV1, RouteLegV2, SiteLogisticsNodeV2, SupplierRouteV2,
};

use super::{
    decode_material_circuit_state_v2, encode_material_circuit_state_v2, sha256_of, validate,
    MaterialCircuitStateV2, SectorBundleErrorV1, SectorBundleGoodV1, SectorBundleOwnerV1,
    SectorBundleProcessV1, SectorBundleSourcesV1, SectorBundleV1, UnitIdV1, HORIZON_TICKS,
};
use crate::michigan_cohorts::michigan_business_subject_v2;
use crate::michigan_material::{
    michigan_material_catalog_v1, MichiganDeliveryPresetV1, MichiganMaterialCatalogV1,
    MichiganMaterialProcessV1, MichiganMaterialRouteV1, MICHIGAN_INDUSTRY_BASELINE_SHA256_V1,
    MICHIGAN_MATERIAL_SCENARIO_SHA256_V1,
};
use crate::michigan_sectors::{
    michigan_county_sectors_v1, MichiganCountySectorV1, QCEW_SECTORS_ARTIFACT_SHA256_V1,
    QCEW_SECTORS_SEMANTIC_SHA256_V1,
};

fn catalog() -> Result<&'static MichiganMaterialCatalogV1, SectorBundleErrorV1> {
    michigan_material_catalog_v1().map_err(|_| SectorBundleErrorV1::Source)
}

fn source(county: &str) -> Result<&'static MichiganCountySectorV1, SectorBundleErrorV1> {
    michigan_county_sectors_v1()
        .map_err(|_| SectorBundleErrorV1::Source)?
        .rows()
        .iter()
        .find(|row| row.county_geoid() == county && row.sector_code().as_str() == "31-33")
        .ok_or(SectorBundleErrorV1::Owner)
}

fn source_proof(
    row: &MichiganCountySectorV1,
) -> Result<SectorBundleSourcesV1, SectorBundleErrorV1> {
    Ok(SectorBundleSourcesV1 {
        county_source_file: row.source_file().to_owned(),
        county_source_sha256: digest(row.source_sha256())?,
        sector_artifact_sha256: digest(QCEW_SECTORS_ARTIFACT_SHA256_V1)?,
        sector_semantic_sha256: digest(QCEW_SECTORS_SEMANTIC_SHA256_V1)?,
        industry_artifact_sha256: digest(MICHIGAN_INDUSTRY_BASELINE_SHA256_V1)?,
        designed_scenario_sha256: digest(MICHIGAN_MATERIAL_SCENARIO_SHA256_V1)?,
    })
}

fn digest(text: &str) -> Result<[u8; 32], SectorBundleErrorV1> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SectorBundleErrorV1::Source);
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| SectorBundleErrorV1::Source)?;
    }
    Ok(bytes)
}

pub(super) fn validate_sources(
    owner: &SectorBundleOwnerV1,
    evidence: &SectorBundleSourcesV1,
) -> Result<(), SectorBundleErrorV1> {
    if owner.sector_code != "31-33"
        || !matches!(
            owner.county_geoid.as_str(),
            "26099" | "26125" | "26161" | "26163"
        )
    {
        return Err(SectorBundleErrorV1::Owner);
    }
    let row = source(&owner.county_geoid)?;
    if owner.subject != michigan_business_subject_v2(row) {
        return Err(SectorBundleErrorV1::Owner);
    }
    if *evidence != source_proof(row)? {
        return Err(SectorBundleErrorV1::Source);
    }
    Ok(())
}

pub(super) fn validate_process_bindings(
    bundle: &SectorBundleV1,
) -> Result<(), SectorBundleErrorV1> {
    let catalog = catalog()?;
    for good in &bundle.goods {
        if !catalog
            .goods()
            .iter()
            .any(|row| row.id() == good.good_id && row.unit_id() == good.unit_id)
        {
            return Err(SectorBundleErrorV1::GoodUnit);
        }
    }
    if bundle.labor_unit
        != UnitIdV1::from_bytes(sha256_of(b"babylon.michigan-material.v1\0unit\0labor-hour"))
    {
        return Err(SectorBundleErrorV1::Resource);
    }
    for binding in &bundle.processes {
        let process = catalog
            .processes()
            .iter()
            .find(|process| process.id() == binding.process_id)
            .ok_or(SectorBundleErrorV1::ProcessOwnership)?;
        let site = catalog
            .site(&process.site_key)
            .ok_or(SectorBundleErrorV1::ProcessOwnership)?;
        let output = bundle
            .rows
            .process_outputs
            .iter()
            .find(|row| row.process_id == binding.process_id)
            .ok_or(SectorBundleErrorV1::ProcessOwnership)?;
        let industry = catalog
            .industry_for_site(site)
            .ok_or(SectorBundleErrorV1::Source)?;
        let node = bundle
            .rows
            .site_logistics_nodes
            .iter()
            .find(|row| row.site_id == site.id())
            .ok_or(SectorBundleErrorV1::ProcessOwnership)?;
        if site.county_geoid != bundle.owner.county_geoid
            || site.naics != binding.industry_code
            || output.site_id != site.id()
            || node.node_id != site.node_id()
            || industry.source_file != bundle.sources.county_source_file
            || digest(&industry.source_sha256)? != bundle.sources.county_source_sha256
        {
            return Err(SectorBundleErrorV1::ProcessOwnership);
        }
    }
    Ok(())
}

/// Capture four nonempty bundles solely from the already admitted Designed content.
/// # Errors
/// Refuses observed source drift, ownership ambiguity or invalid material rows.
pub fn michigan_sector_bundles_v1() -> Result<&'static [SectorBundleV1], SectorBundleErrorV1> {
    static BUNDLES: OnceLock<Result<Vec<SectorBundleV1>, SectorBundleErrorV1>> = OnceLock::new();
    BUNDLES
        .get_or_init(build_bundles)
        .as_ref()
        .map(Vec::as_slice)
        .map_err(|error| *error)
}

fn build_bundles() -> Result<Vec<SectorBundleV1>, SectorBundleErrorV1> {
    let catalog = catalog()?;
    let mut by_county = BTreeMap::<&str, Vec<&MichiganMaterialProcessV1>>::new();
    for process in catalog.processes() {
        let site = catalog
            .site(&process.site_key)
            .ok_or(SectorBundleErrorV1::Owner)?;
        by_county
            .entry(&site.county_geoid)
            .or_default()
            .push(process);
    }
    if by_county.len() != 4 {
        return Err(SectorBundleErrorV1::Coverage);
    }
    by_county
        .into_iter()
        .map(|(county, processes)| build_bundle(catalog, county, &processes))
        .collect()
}

fn build_bundle(
    catalog: &MichiganMaterialCatalogV1,
    county: &str,
    processes: &[&MichiganMaterialProcessV1],
) -> Result<SectorBundleV1, SectorBundleErrorV1> {
    let row = source(county)?;
    let owner = SectorBundleOwnerV1 {
        subject: michigan_business_subject_v2(row),
        county_geoid: county.to_owned(),
        sector_code: row.sector_code().as_str().to_owned(),
    };
    let mut rows = empty_state();
    let mut goods = BTreeSet::new();
    let mut bindings = Vec::new();
    // Preserve the original resource identity; no conversion from observed jobs.
    let labor_unit =
        UnitIdV1::from_bytes(sha256_of(b"babylon.michigan-material.v1\0unit\0labor-hour"));
    for process in processes {
        let site = catalog
            .site(&process.site_key)
            .ok_or(SectorBundleErrorV1::Owner)?;
        rows.site_logistics_nodes.push(SiteLogisticsNodeV2 {
            site_id: site.id(),
            node_id: site.node_id(),
        });
        for key in [&process.input_good_key, &process.output_good_key] {
            let good = catalog.good(key).ok_or(SectorBundleErrorV1::GoodUnit)?;
            goods.insert(SectorBundleGoodV1 {
                good_id: good.id(),
                unit_id: good.unit_id(),
            });
        }
        bindings.push(SectorBundleProcessV1 {
            process_id: process.id(),
            industry_code: site.naics.clone(),
        });
        append_process(&mut rows, catalog, process, labor_unit)?;
    }
    SectorBundleV1::from_parts(
        owner,
        source_proof(row)?,
        goods.into_iter().collect(),
        bindings,
        labor_unit,
        &rows,
    )
}

fn append_process(
    state: &mut MaterialCircuitStateV2,
    catalog: &MichiganMaterialCatalogV1,
    process: &MichiganMaterialProcessV1,
    labor_unit: UnitIdV1,
) -> Result<(), SectorBundleErrorV1> {
    let input = catalog
        .good(&process.input_good_key)
        .ok_or(SectorBundleErrorV1::GoodUnit)?;
    let output = catalog
        .good(&process.output_good_key)
        .ok_or(SectorBundleErrorV1::GoodUnit)?;
    let site_id = process.site_id();
    let process_id = process.id();
    state.process_outputs.push(ProcessOutputV1 {
        process_id,
        site_id,
        good_id: output.id(),
        unit_id: output.unit_id(),
        quantity_per_batch: process.output_quantity_per_batch,
    });
    state.input_coefficients.push(InputOutputCoefficientV1 {
        process_id,
        good_id: input.id(),
        unit_id: input.unit_id(),
        quantity_per_batch: process.input_quantity_per_batch,
    });
    state.labor_coefficients.push(LaborCoefficientV1 {
        process_id,
        unit_id: labor_unit,
        quantity_per_batch: process.labor_hours_per_batch,
    });
    state.inventory.push(InventoryRowV1 {
        site_id,
        good_id: input.id(),
        unit_id: input.unit_id(),
        quantity: process.opening_input_quantity,
    });
    state.inventory.push(InventoryRowV1 {
        site_id,
        good_id: output.id(),
        unit_id: output.unit_id(),
        quantity: 0,
    });
    for week in 1..=HORIZON_TICKS {
        state.capacities.push(CapacityRowV1 {
            process_id,
            site_id,
            week,
            available_batches: process.capacity_batches_per_week,
        });
        state.labor.push(LaborCapacityRowV1 {
            site_id,
            unit_id: labor_unit,
            week,
            available: process.labor_capacity_hours_per_week,
        });
    }
    if process.opening_planned_batches > 0 {
        state.production_commitments.push(ProductionCommitmentV1 {
            process_id,
            site_id,
            week: 1,
            planned_batches: process.opening_planned_batches,
        });
    }
    Ok(())
}

/// Compile actual opening rows from all four bundles and the existing transfer design.
///
/// This is a content compiler, not admission: the future campaign factory must
/// separately pin these bundle bytes. Reopening a save must use stored content.
/// # Errors
/// Refuses missing/duplicate ownership, mixed units or invalid V2 circuit rows.
pub fn compile_sector_bundles_v1(
    bundles: &[SectorBundleV1],
    preset: MichiganDeliveryPresetV1,
) -> Result<MaterialCircuitStateV2, SectorBundleErrorV1> {
    if bundles.len() != 4 {
        return Err(SectorBundleErrorV1::Coverage);
    }
    let catalog = catalog()?;
    let mut owners = BTreeSet::new();
    let mut processes = BTreeSet::new();
    let mut sites = BTreeSet::new();
    let mut goods = BTreeMap::new();
    let mut state = empty_state();
    for bundle in bundles {
        validate::bundle(bundle)?;
        if !owners.insert(bundle.owner.county_geoid.as_str()) {
            return Err(SectorBundleErrorV1::ProcessOwnership);
        }
        for output in &bundle.rows.process_outputs {
            if !processes.insert(output.process_id) || !sites.insert(output.site_id) {
                return Err(SectorBundleErrorV1::ProcessOwnership);
            }
        }
        for good in &bundle.goods {
            if goods
                .insert(good.good_id, good.unit_id)
                .is_some_and(|unit| unit != good.unit_id)
            {
                return Err(SectorBundleErrorV1::GoodUnit);
            }
        }
        append_rows(&mut state, &bundle.rows);
    }
    if processes
        != catalog
            .processes()
            .iter()
            .map(MichiganMaterialProcessV1::id)
            .collect()
    {
        return Err(SectorBundleErrorV1::Coverage);
    }
    for route in catalog.routes() {
        append_route(&mut state, catalog, route, preset)?;
    }
    let bytes = encode_material_circuit_state_v2(&state)?;
    decode_material_circuit_state_v2(&bytes).map_err(Into::into)
}

fn append_rows(state: &mut MaterialCircuitStateV2, rows: &MaterialCircuitStateV2) {
    state
        .site_logistics_nodes
        .extend_from_slice(&rows.site_logistics_nodes);
    state
        .process_outputs
        .extend_from_slice(&rows.process_outputs);
    state
        .input_coefficients
        .extend_from_slice(&rows.input_coefficients);
    state
        .labor_coefficients
        .extend_from_slice(&rows.labor_coefficients);
    state.inventory.extend_from_slice(&rows.inventory);
    state.capacities.extend_from_slice(&rows.capacities);
    state.labor.extend_from_slice(&rows.labor);
    state
        .production_commitments
        .extend_from_slice(&rows.production_commitments);
}

fn append_route(
    state: &mut MaterialCircuitStateV2,
    catalog: &MichiganMaterialCatalogV1,
    route: &MichiganMaterialRouteV1,
    preset: MichiganDeliveryPresetV1,
) -> Result<(), SectorBundleErrorV1> {
    let supplier = catalog
        .site(&route.supplier_site_key)
        .ok_or(SectorBundleErrorV1::Owner)?;
    let buyer = catalog
        .site(&route.buyer_site_key)
        .ok_or(SectorBundleErrorV1::Owner)?;
    let good = catalog
        .good(&route.good_key)
        .ok_or(SectorBundleErrorV1::GoodUnit)?;
    state.supplier_routes.push(SupplierRouteV2 {
        buyer_site_id: buyer.id(),
        supplier_site_id: supplier.id(),
        good_id: good.id(),
        unit_id: good.unit_id(),
        route_id: route.id(),
    });
    state.route_legs.push(RouteLegV2 {
        route_id: route.id(),
        leg_index: 0,
        corridor_id: route.corridor_id(),
        from_node_id: supplier.node_id(),
        to_node_id: buyer.node_id(),
        travel_weeks: catalog.travel_weeks(route, preset),
        loss_ppm: 0,
    });
    state.orders.push(OrderRowV2 {
        order_id: route.order_id(),
        access_mode: OrderAccessModeV1::CommoditySale,
        buyer_site_id: buyer.id(),
        supplier_site_id: supplier.id(),
        good_id: good.id(),
        unit_id: good.unit_id(),
        ordered: route.ordered_quantity,
        shipped: 0,
        lost: 0,
        delivered: 0,
        realized: 0,
    });
    state.backlog.push(BacklogRowV1 {
        order_id: route.order_id(),
        quantity: route.ordered_quantity,
    });
    for week in 1..=HORIZON_TICKS {
        state.corridor_capacities.push(CorridorCapacityV2 {
            corridor_id: route.corridor_id(),
            unit_id: good.unit_id(),
            week,
            available: route.capacity_quantity_per_week,
        });
    }
    Ok(())
}

fn empty_state() -> MaterialCircuitStateV2 {
    MaterialCircuitStateV2 {
        week: 1,
        site_logistics_nodes: Vec::new(),
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        supplier_routes: Vec::new(),
        route_legs: Vec::new(),
        inventory: Vec::new(),
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        corridor_capacities: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
    }
}
