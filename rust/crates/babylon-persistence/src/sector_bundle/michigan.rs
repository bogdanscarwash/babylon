//! The single compiler from captured normalized owners into V3 material rows.
use super::{
    sha256_of, validate, SectorBundleErrorV2, SectorBundleGoodV2, SectorBundleOwnerV2,
    SectorBundleProcessV2, SectorBundleSourcesV2, SectorBundleV2,
};
use crate::michigan_cohorts::michigan_business_subject_for_owner_v2;
use crate::michigan_material::{
    MichiganDeliveryPresetV1, MichiganMaterialCatalogV1, MichiganMaterialCorridorV1,
    MichiganMaterialPathV2, MichiganMaterialRouteV1, MichiganMaterialSiteV1, MichiganSiteRoleV2,
    MICHIGAN_MAX_HORIZON_PERIODS_V1,
};
use babylon_material_circuit::{
    decode_material_circuit_state_v3, encode_material_circuit_state_v3, BacklogRowV1,
    CapacityRowV1, CorridorCapacityV3, FinalDemandOrderV3, FinalDemandPrincipalV3,
    FreightMassCoefficientV3, GoodIdV1, InputOutputCoefficientV1, InventoryRowV1,
    LaborCapacityRowV1, LaborCoefficientV1, MaterialCircuitStateV3, MerchantHandlingCoefficientV3,
    MerchantHandlingV3, MerchantRoleV3, OrderAccessModeV1, OrderRowV2, ProcessOutputV1,
    ProductionCommitmentV1, RouteStageCapacityV3, RouteStageV3, SiteIdV1, SiteLogisticsNodeV2,
    SupplierRouteV3, SupplierTransportV3, UnitIdV1,
};
use std::collections::{BTreeMap, BTreeSet};

fn digest(text: &str) -> Result<[u8; 32], SectorBundleErrorV2> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(SectorBundleErrorV2::Source);
    }
    let mut out = [0; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| SectorBundleErrorV2::Source)?;
    }
    Ok(out)
}
/// Capture all admitted owners; an owner may have several processes or handle goods.
/// # Errors
/// Refuses missing sources, invalid native units or incompatible resource rows.
pub fn michigan_sector_bundles_v2(
    catalog: &MichiganMaterialCatalogV1,
) -> Result<Vec<SectorBundleV2>, SectorBundleErrorV2> {
    let mut bundles = Vec::new();
    for source in catalog.owners() {
        let owner = SectorBundleOwnerV2 {
            subject: michigan_business_subject_for_owner_v2(
                &source.county_geoid,
                &source.sector_code,
            ),
            county_geoid: source.county_geoid.clone(),
            sector_code: source.sector_code.clone(),
        };
        let evidence = SectorBundleSourcesV2 {
            county_source_file: source.county_source_file.clone(),
            county_source_sha256: digest(&source.county_source_sha256)?,
            sector_artifact_sha256: digest(&source.sector_artifact_sha256)?,
            sector_semantic_sha256: digest(&source.sector_semantic_sha256)?,
            industry_artifact_sha256: digest(&source.industry_artifact_sha256)?,
            designed_scenario_sha256: catalog.defines_hash(),
        };
        let mut captured = OwnerRows::new();
        for site in catalog.sites().iter().filter(|s| {
            s.county_geoid == source.county_geoid && s.sector_code == source.sector_code
        }) {
            captured.append_site(catalog, site)?;
        }
        bundles.push(captured.finish(catalog, owner, evidence)?);
    }
    bundles.sort_by(|a, b| a.owner.subject.cmp(&b.owner.subject));
    Ok(bundles)
}
fn county_bytes(county: &str) -> Result<[u8; 5], SectorBundleErrorV2> {
    county
        .as_bytes()
        .try_into()
        .map_err(|_| SectorBundleErrorV2::Owner)
}
fn corridor<'a>(
    catalog: &'a MichiganMaterialCatalogV1,
    key: &str,
) -> Result<&'a MichiganMaterialCorridorV1, SectorBundleErrorV2> {
    catalog
        .corridors()
        .iter()
        .find(|c| c.key == key)
        .ok_or(SectorBundleErrorV2::Resource)
}
/// Compile one selected captured model through the current material codec.
/// # Errors
/// Refuses incomplete or changed bundles and inconsistent selected presets.
pub fn compile_sector_bundles_v2(
    bundles: &[SectorBundleV2],
    preset: MichiganDeliveryPresetV1,
    catalog: &MichiganMaterialCatalogV1,
) -> Result<MaterialCircuitStateV3, SectorBundleErrorV2> {
    if catalog.preset() != preset {
        return Err(SectorBundleErrorV2::Preset);
    }
    let mut ordered = bundles.to_vec();
    ordered.sort_by(|a, b| a.owner.subject.cmp(&b.owner.subject));
    if ordered != michigan_sector_bundles_v2(catalog)? {
        return Err(SectorBundleErrorV2::Source);
    }
    let mut state = empty_state();
    let mut mass = BTreeMap::new();
    for bundle in &ordered {
        validate::bundle(bundle)?;
        let rows = &bundle.rows;
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
        state.merchants.extend_from_slice(&rows.merchants);
        state
            .handling_coefficients
            .extend_from_slice(&rows.handling_coefficients);
        for row in &rows.freight_mass_coefficients {
            if mass
                .insert((row.good_id, row.unit_id), row.grams_per_unit)
                .is_some_and(|n| n != row.grams_per_unit)
            {
                return Err(SectorBundleErrorV2::GoodUnit);
            }
        }
    }
    state.freight_mass_coefficients = mass
        .into_iter()
        .map(
            |((good_id, unit_id), grams_per_unit)| FreightMassCoefficientV3 {
                good_id,
                unit_id,
                grams_per_unit,
            },
        )
        .collect();
    for route in catalog.routes() {
        append_route(&mut state, catalog, route)?;
    }
    append_final_demand(&mut state, catalog)?;
    let active: BTreeSet<_> = state
        .route_stage_capacities
        .iter()
        .map(|r| r.corridor_id)
        .chain(state.merchants.iter().map(|m| m.capacity_id))
        .collect();
    for c in catalog
        .corridors()
        .iter()
        .filter(|c| active.contains(&c.id()))
    {
        for period in 1..=MICHIGAN_MAX_HORIZON_PERIODS_V1 {
            state.corridor_capacities.push(CorridorCapacityV3 {
                corridor_id: c.id(),
                period,
                available_grams: c.capacity_grams_per_period,
            });
        }
    }
    decode_material_circuit_state_v3(&encode_material_circuit_state_v3(&state)?).map_err(Into::into)
}
fn append_route(
    state: &mut MaterialCircuitStateV3,
    catalog: &MichiganMaterialCatalogV1,
    route: &MichiganMaterialRouteV1,
) -> Result<(), SectorBundleErrorV2> {
    let supplier = catalog
        .site(&route.supplier_site_key)
        .ok_or(SectorBundleErrorV2::Owner)?;
    let buyer = catalog
        .site(&route.buyer_site_key)
        .ok_or(SectorBundleErrorV2::Owner)?;
    let good = catalog
        .good(&route.good_key)
        .ok_or(SectorBundleErrorV2::GoodUnit)?;
    let transport_kind = match &route.path {
        MichiganMaterialPathV2::Local => SupplierTransportV3::Local,
        MichiganMaterialPathV2::Routed {
            travel_periods,
            capacity_keys,
            ..
        } => {
            state.route_stages.push(RouteStageV3 {
                route_id: route.id(),
                stage_index: 0,
                from_node_id: supplier.node_id(),
                to_node_id: buyer.node_id(),
                travel_periods: *travel_periods,
                loss_ppm: 0,
            });
            for key in capacity_keys {
                state.route_stage_capacities.push(RouteStageCapacityV3 {
                    route_id: route.id(),
                    stage_index: 0,
                    corridor_id: corridor(catalog, key)?.id(),
                });
            }
            SupplierTransportV3::Staged
        }
    };
    state.supplier_routes.push(SupplierRouteV3 {
        buyer_site_id: buyer.id(),
        supplier_site_id: supplier.id(),
        good_id: good.id(),
        unit_id: good.unit_id(),
        route_id: route.id(),
        transport_kind,
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
    Ok(())
}
fn empty_state() -> MaterialCircuitStateV3 {
    MaterialCircuitStateV3 {
        period: 1,
        site_logistics_nodes: Vec::new(),
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        freight_mass_coefficients: Vec::new(),
        supplier_routes: Vec::new(),
        route_stages: Vec::new(),
        route_stage_capacities: Vec::new(),
        inventory: Vec::new(),
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        corridor_capacities: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
        merchants: Vec::new(),
        handling_coefficients: Vec::new(),
        final_demand_principals: Vec::new(),
        final_demand_orders: Vec::new(),
    }
}

struct OwnerRows {
    rows: MaterialCircuitStateV3,
    goods: BTreeSet<SectorBundleGoodV2>,
    processes: Vec<SectorBundleProcessV2>,
    labor_unit: UnitIdV1,
    inventory: BTreeMap<(SiteIdV1, GoodIdV1, UnitIdV1), u64>,
}
impl OwnerRows {
    fn new() -> Self {
        Self {
            rows: empty_state(),
            goods: BTreeSet::new(),
            processes: Vec::new(),
            labor_unit: UnitIdV1::from_bytes(sha256_of(
                b"babylon.michigan-material.v1\0unit\0labor-hour",
            )),
            inventory: BTreeMap::new(),
        }
    }

    fn append_site(
        &mut self,
        catalog: &MichiganMaterialCatalogV1,
        site: &MichiganMaterialSiteV1,
    ) -> Result<(), SectorBundleErrorV2> {
        self.rows.site_logistics_nodes.push(SiteLogisticsNodeV2 {
            site_id: site.id(),
            node_id: site.node_id(),
        });
        let seed = catalog
            .staffing()
            .pools
            .iter()
            .find(|p| p.site_key == site.key)
            .ok_or(SectorBundleErrorV2::Resource)?;
        self.rows.labor.push(LaborCapacityRowV1 {
            site_id: site.id(),
            unit_id: self.labor_unit,
            period: 1,
            available: seed
                .employed
                .checked_mul(catalog.staffing().hours_per_worker_period)
                .ok_or(SectorBundleErrorV2::Arithmetic)?,
        });
        let mut good_keys: BTreeSet<&str> = BTreeSet::new();
        self.append_production(catalog, site, &mut good_keys)?;
        self.append_merchants(catalog, site, &mut good_keys)?;
        for r in catalog
            .routes()
            .iter()
            .filter(|r| r.supplier_site_key == site.key || r.buyer_site_key == site.key)
        {
            good_keys.insert(&r.good_key);
        }
        for key in good_keys {
            let good = catalog.good(key).ok_or(SectorBundleErrorV2::GoodUnit)?;
            self.goods.insert(SectorBundleGoodV2 {
                good_id: good.id(),
                unit_id: good.unit_id(),
            });
            self.inventory
                .entry((site.id(), good.id(), good.unit_id()))
                .or_default();
        }

        Ok(())
    }

    fn append_production<'a>(
        &mut self,
        catalog: &'a MichiganMaterialCatalogV1,
        site: &MichiganMaterialSiteV1,
        good_keys: &mut BTreeSet<&'a str>,
    ) -> Result<(), SectorBundleErrorV2> {
        for process in catalog
            .processes()
            .iter()
            .filter(|p| p.site_key == site.key)
        {
            self.processes.push(SectorBundleProcessV2 {
                process_id: process.id(),
                industry_code: process.industry_code.clone(),
            });
            let output = catalog
                .good(&process.output_good_key)
                .ok_or(SectorBundleErrorV2::GoodUnit)?;
            good_keys.insert(&process.output_good_key);
            self.rows.process_outputs.push(ProcessOutputV1 {
                process_id: process.id(),
                site_id: site.id(),
                good_id: output.id(),
                unit_id: output.unit_id(),
                quantity_per_batch: process.output_quantity_per_batch,
            });
            self.inventory
                .entry((site.id(), output.id(), output.unit_id()))
                .or_default();
            for input in &process.inputs {
                let good = catalog
                    .good(&input.good_key)
                    .ok_or(SectorBundleErrorV2::GoodUnit)?;
                good_keys.insert(&input.good_key);
                self.rows.input_coefficients.push(InputOutputCoefficientV1 {
                    process_id: process.id(),
                    good_id: good.id(),
                    unit_id: good.unit_id(),
                    quantity_per_batch: input.quantity_per_batch,
                });
                let n = self
                    .inventory
                    .entry((site.id(), good.id(), good.unit_id()))
                    .or_default();
                *n = n
                    .checked_add(input.opening_quantity)
                    .ok_or(SectorBundleErrorV2::Arithmetic)?;
            }
            self.rows.labor_coefficients.push(LaborCoefficientV1 {
                process_id: process.id(),
                unit_id: self.labor_unit,
                quantity_per_batch: process.labor_hours_per_batch,
            });
            for period in 1..=MICHIGAN_MAX_HORIZON_PERIODS_V1 {
                self.rows.capacities.push(CapacityRowV1 {
                    process_id: process.id(),
                    site_id: site.id(),
                    period,
                    available_batches: process.capacity_batches_per_period,
                });
            }
            if process.opening_planned_batches > 0 {
                self.rows
                    .production_commitments
                    .push(ProductionCommitmentV1 {
                        process_id: process.id(),
                        site_id: site.id(),
                        period: 1,
                        planned_batches: process.opening_planned_batches,
                    });
            }
        }

        Ok(())
    }

    fn append_merchants<'a>(
        &mut self,
        catalog: &'a MichiganMaterialCatalogV1,
        site: &MichiganMaterialSiteV1,
        good_keys: &mut BTreeSet<&'a str>,
    ) -> Result<(), SectorBundleErrorV2> {
        if let Some(merchant) = catalog.merchants().iter().find(|m| m.site_key == site.key) {
            self.rows.merchants.push(MerchantHandlingV3 {
                site_id: site.id(),
                county_geoid: county_bytes(&site.county_geoid)?,
                role: match site.role {
                    MichiganSiteRoleV2::Wholesale => MerchantRoleV3::Wholesale,
                    MichiganSiteRoleV2::Retail => MerchantRoleV3::Retail,
                    MichiganSiteRoleV2::Production => return Err(SectorBundleErrorV2::Owner),
                },
                capacity_id: corridor(catalog, &merchant.capacity_key)?.id(),
                labor_unit_id: self.labor_unit,
            });
            for (key, hours) in &merchant.handling_hours_per_unit {
                let good = catalog.good(key).ok_or(SectorBundleErrorV2::GoodUnit)?;
                good_keys.insert(key);
                self.rows
                    .handling_coefficients
                    .push(MerchantHandlingCoefficientV3 {
                        site_id: site.id(),
                        good_id: good.id(),
                        unit_id: good.unit_id(),
                        hours_per_unit: *hours,
                    });
            }
        }

        Ok(())
    }

    fn finish(
        mut self,
        catalog: &MichiganMaterialCatalogV1,
        owner: SectorBundleOwnerV2,
        evidence: SectorBundleSourcesV2,
    ) -> Result<SectorBundleV2, SectorBundleErrorV2> {
        self.rows.inventory = self
            .inventory
            .into_iter()
            .map(|((site_id, good_id, unit_id), quantity)| InventoryRowV1 {
                site_id,
                good_id,
                unit_id,
                quantity,
            })
            .collect();
        for good in &self.goods {
            let definition = catalog
                .goods()
                .iter()
                .find(|g| g.id() == good.good_id)
                .ok_or(SectorBundleErrorV2::GoodUnit)?;
            self.rows
                .freight_mass_coefficients
                .push(FreightMassCoefficientV3 {
                    good_id: good.good_id,
                    unit_id: good.unit_id,
                    grams_per_unit: definition.grams_per_unit,
                });
        }
        SectorBundleV2::from_parts(
            owner,
            evidence,
            self.goods.into_iter().collect(),
            self.processes,
            self.labor_unit,
            &self.rows,
        )
    }
}

fn append_final_demand(
    state: &mut MaterialCircuitStateV3,
    catalog: &MichiganMaterialCatalogV1,
) -> Result<(), SectorBundleErrorV2> {
    let mut principals = BTreeSet::new();
    for demand in catalog.final_demands() {
        let site = catalog
            .site(&demand.retailer_site_key)
            .ok_or(SectorBundleErrorV2::Owner)?;
        let good = catalog
            .good(&demand.good_key)
            .ok_or(SectorBundleErrorV2::GoodUnit)?;
        if principals.insert(demand.principal_id()) {
            state.final_demand_principals.push(FinalDemandPrincipalV3 {
                id: demand.principal_id(),
                county_geoid: county_bytes(&demand.county_geoid)?,
            });
        }
        state.final_demand_orders.push(FinalDemandOrderV3 {
            order_id: demand.order_id(),
            retailer_site_id: site.id(),
            demand_principal_id: demand.principal_id(),
            good_id: good.id(),
            unit_id: good.unit_id(),
            ordered: demand.ordered_quantity,
            fulfilled: 0,
        });
    }
    Ok(())
}
