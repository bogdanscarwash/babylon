//! The single compiler from captured normalized owners into V3 material rows.
use super::{
    sha256_of, validate, SectorBundle, SectorBundleError, SectorBundleGood, SectorBundleOwner,
    SectorBundleProcess, SectorBundleSources,
};
use crate::michigan_cohorts::michigan_business_subject_for_owner;
use crate::michigan_material::{
    MichiganDeliveryPreset, MichiganMaterialCatalog, MichiganMaterialCorridor,
    MichiganMaterialPath, MichiganMaterialRoute, MichiganMaterialSite, MichiganSiteRole,
    MICHIGAN_MAX_HORIZON_PERIODS,
};
use babylon_material_circuit::{
    decode_material_circuit_state, encode_material_circuit_state, BacklogRow, CapacityRow,
    CorridorCapacity, FinalDemandOrder, FinalDemandPrincipal, FreightMassCoefficient, GoodId,
    InputOutputCoefficient, InventoryRow, LaborCapacityRow, LaborCoefficient, MaterialCircuitState,
    MerchantHandling, MerchantHandlingCoefficient, MerchantRole, OrderAccessMode, OrderRow,
    ProcessOutput, ProductionCommitment, RouteStage, RouteStageCapacity, SiteId, SiteLogisticsNode,
    SupplierRoute, SupplierTransport, UnitId,
};
use std::collections::{BTreeMap, BTreeSet};

fn digest(text: &str) -> Result<[u8; 32], SectorBundleError> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(SectorBundleError::Source);
    }
    let mut out = [0; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| SectorBundleError::Source)?;
    }
    Ok(out)
}
/// Capture all admitted owners; an owner may have several processes or handle goods.
/// # Errors
/// Refuses missing sources, invalid native units or incompatible resource rows.
pub fn michigan_sector_bundles(
    catalog: &MichiganMaterialCatalog,
) -> Result<Vec<SectorBundle>, SectorBundleError> {
    let mut bundles = Vec::new();
    for source in catalog.owners() {
        let owner = SectorBundleOwner {
            subject: michigan_business_subject_for_owner(&source.county_geoid, &source.sector_code),
            county_geoid: source.county_geoid.clone(),
            sector_code: source.sector_code.clone(),
        };
        let evidence = SectorBundleSources {
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
fn county_bytes(county: &str) -> Result<[u8; 5], SectorBundleError> {
    county
        .as_bytes()
        .try_into()
        .map_err(|_| SectorBundleError::Owner)
}
fn corridor<'a>(
    catalog: &'a MichiganMaterialCatalog,
    key: &str,
) -> Result<&'a MichiganMaterialCorridor, SectorBundleError> {
    catalog
        .corridors()
        .iter()
        .find(|c| c.key == key)
        .ok_or(SectorBundleError::Resource)
}
/// Compile one selected captured model through the current material codec.
/// # Errors
/// Refuses incomplete or changed bundles and inconsistent selected presets.
pub fn compile_sector_bundles(
    bundles: &[SectorBundle],
    preset: MichiganDeliveryPreset,
    catalog: &MichiganMaterialCatalog,
) -> Result<MaterialCircuitState, SectorBundleError> {
    if catalog.preset() != preset {
        return Err(SectorBundleError::Preset);
    }
    let mut ordered = bundles.to_vec();
    ordered.sort_by(|a, b| a.owner.subject.cmp(&b.owner.subject));
    if ordered != michigan_sector_bundles(catalog)? {
        return Err(SectorBundleError::Source);
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
                return Err(SectorBundleError::GoodUnit);
            }
        }
    }
    state.freight_mass_coefficients = mass
        .into_iter()
        .map(
            |((good_id, unit_id), grams_per_unit)| FreightMassCoefficient {
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
        for period in 1..=MICHIGAN_MAX_HORIZON_PERIODS {
            state.corridor_capacities.push(CorridorCapacity {
                corridor_id: c.id(),
                period,
                available_grams: c.capacity_grams_per_period,
            });
        }
    }
    decode_material_circuit_state(&encode_material_circuit_state(&state)?).map_err(Into::into)
}
fn append_route(
    state: &mut MaterialCircuitState,
    catalog: &MichiganMaterialCatalog,
    route: &MichiganMaterialRoute,
) -> Result<(), SectorBundleError> {
    let supplier = catalog
        .site(&route.supplier_site_key)
        .ok_or(SectorBundleError::Owner)?;
    let buyer = catalog
        .site(&route.buyer_site_key)
        .ok_or(SectorBundleError::Owner)?;
    let good = catalog
        .good(&route.good_key)
        .ok_or(SectorBundleError::GoodUnit)?;
    let transport_kind = match &route.path {
        MichiganMaterialPath::Local => SupplierTransport::Local,
        MichiganMaterialPath::Routed {
            travel_periods,
            capacity_keys,
            ..
        } => {
            state.route_stages.push(RouteStage {
                route_id: route.id(),
                stage_index: 0,
                from_node_id: supplier.node_id(),
                to_node_id: buyer.node_id(),
                travel_periods: *travel_periods,
                loss_ppm: 0,
            });
            for key in capacity_keys {
                state.route_stage_capacities.push(RouteStageCapacity {
                    route_id: route.id(),
                    stage_index: 0,
                    corridor_id: corridor(catalog, key)?.id(),
                });
            }
            SupplierTransport::Staged
        }
    };
    state.supplier_routes.push(SupplierRoute {
        buyer_site_id: buyer.id(),
        supplier_site_id: supplier.id(),
        good_id: good.id(),
        unit_id: good.unit_id(),
        route_id: route.id(),
        transport_kind,
    });
    state.orders.push(OrderRow {
        order_id: route.order_id(),
        access_mode: OrderAccessMode::CommoditySale,
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
    state.backlog.push(BacklogRow {
        order_id: route.order_id(),
        quantity: route.ordered_quantity,
    });
    Ok(())
}
fn empty_state() -> MaterialCircuitState {
    MaterialCircuitState {
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
    rows: MaterialCircuitState,
    goods: BTreeSet<SectorBundleGood>,
    processes: Vec<SectorBundleProcess>,
    labor_unit: UnitId,
    inventory: BTreeMap<(SiteId, GoodId, UnitId), u64>,
}
impl OwnerRows {
    fn new() -> Self {
        Self {
            rows: empty_state(),
            goods: BTreeSet::new(),
            processes: Vec::new(),
            labor_unit: UnitId::from_bytes(sha256_of(
                b"babylon.michigan-material.v1\0unit\0labor-hour",
            )),
            inventory: BTreeMap::new(),
        }
    }

    fn append_site(
        &mut self,
        catalog: &MichiganMaterialCatalog,
        site: &MichiganMaterialSite,
    ) -> Result<(), SectorBundleError> {
        self.rows.site_logistics_nodes.push(SiteLogisticsNode {
            site_id: site.id(),
            node_id: site.node_id(),
        });
        let seed = catalog
            .staffing()
            .pools
            .iter()
            .find(|p| p.site_key == site.key)
            .ok_or(SectorBundleError::Resource)?;
        self.rows.labor.push(LaborCapacityRow {
            site_id: site.id(),
            unit_id: self.labor_unit,
            period: 1,
            available: seed
                .employed
                .checked_mul(catalog.staffing().hours_per_worker_period)
                .ok_or(SectorBundleError::Arithmetic)?,
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
            let good = catalog.good(key).ok_or(SectorBundleError::GoodUnit)?;
            self.goods.insert(SectorBundleGood {
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
        catalog: &'a MichiganMaterialCatalog,
        site: &MichiganMaterialSite,
        good_keys: &mut BTreeSet<&'a str>,
    ) -> Result<(), SectorBundleError> {
        for process in catalog
            .processes()
            .iter()
            .filter(|p| p.site_key == site.key)
        {
            self.processes.push(SectorBundleProcess {
                process_id: process.id(),
                industry_code: process.industry_code.clone(),
            });
            let output = catalog
                .good(&process.output_good_key)
                .ok_or(SectorBundleError::GoodUnit)?;
            good_keys.insert(&process.output_good_key);
            self.rows.process_outputs.push(ProcessOutput {
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
                    .ok_or(SectorBundleError::GoodUnit)?;
                good_keys.insert(&input.good_key);
                self.rows.input_coefficients.push(InputOutputCoefficient {
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
                    .ok_or(SectorBundleError::Arithmetic)?;
            }
            self.rows.labor_coefficients.push(LaborCoefficient {
                process_id: process.id(),
                unit_id: self.labor_unit,
                quantity_per_batch: process.labor_hours_per_batch,
            });
            for period in 1..=MICHIGAN_MAX_HORIZON_PERIODS {
                self.rows.capacities.push(CapacityRow {
                    process_id: process.id(),
                    site_id: site.id(),
                    period,
                    available_batches: process.capacity_batches_per_period,
                });
            }
            if process.opening_planned_batches > 0 {
                self.rows.production_commitments.push(ProductionCommitment {
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
        catalog: &'a MichiganMaterialCatalog,
        site: &MichiganMaterialSite,
        good_keys: &mut BTreeSet<&'a str>,
    ) -> Result<(), SectorBundleError> {
        if let Some(merchant) = catalog.merchants().iter().find(|m| m.site_key == site.key) {
            self.rows.merchants.push(MerchantHandling {
                site_id: site.id(),
                county_geoid: county_bytes(&site.county_geoid)?,
                role: match site.role {
                    MichiganSiteRole::Wholesale => MerchantRole::Wholesale,
                    MichiganSiteRole::Retail => MerchantRole::Retail,
                    MichiganSiteRole::Production => return Err(SectorBundleError::Owner),
                },
                capacity_id: corridor(catalog, &merchant.capacity_key)?.id(),
                labor_unit_id: self.labor_unit,
            });
            for (key, hours) in &merchant.handling_hours_per_unit {
                let good = catalog.good(key).ok_or(SectorBundleError::GoodUnit)?;
                good_keys.insert(key);
                self.rows
                    .handling_coefficients
                    .push(MerchantHandlingCoefficient {
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
        catalog: &MichiganMaterialCatalog,
        owner: SectorBundleOwner,
        evidence: SectorBundleSources,
    ) -> Result<SectorBundle, SectorBundleError> {
        self.rows.inventory = self
            .inventory
            .into_iter()
            .map(|((site_id, good_id, unit_id), quantity)| InventoryRow {
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
                .ok_or(SectorBundleError::GoodUnit)?;
            self.rows
                .freight_mass_coefficients
                .push(FreightMassCoefficient {
                    good_id: good.good_id,
                    unit_id: good.unit_id,
                    grams_per_unit: definition.grams_per_unit,
                });
        }
        SectorBundle::from_parts(
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
    state: &mut MaterialCircuitState,
    catalog: &MichiganMaterialCatalog,
) -> Result<(), SectorBundleError> {
    let mut principals = BTreeSet::new();
    for demand in catalog.final_demands() {
        let site = catalog
            .site(&demand.retailer_site_key)
            .ok_or(SectorBundleError::Owner)?;
        let good = catalog
            .good(&demand.good_key)
            .ok_or(SectorBundleError::GoodUnit)?;
        if principals.insert(demand.principal_id()) {
            state.final_demand_principals.push(FinalDemandPrincipal {
                id: demand.principal_id(),
                county_geoid: county_bytes(&demand.county_geoid)?,
            });
        }
        state.final_demand_orders.push(FinalDemandOrder {
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
