use super::{
    arguments::Inputs,
    facts, hex, refused,
    report::{Candidate, Case, OwnerIdentity, ProcessIdentity, Report, RouteIdentity, Witnesses},
    witness, Result,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_kernel::content_digest::sha256_of;
use babylon_persistence::{
    michigan_content::MichiganContentPreset,
    michigan_material::{
        MichiganCapacityOverride, MichiganDeliveryPreset, MichiganIntervention,
        MichiganMaterialCatalog, MichiganOpeningStockOverride,
    },
};
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::replay_session::ReplayCommitDisposition;
use std::{collections::BTreeMap, time::Instant};

pub const BASELINE: &str = "baseline";
pub const FREIGHT: &str = "freight_constraint";
pub const PACKAGING: &str = "packaging_shortage";
pub const BOTH: &str = "both";
pub const PERIODS: u64 = 16;

fn catalog(inputs: &Inputs, candidate: &Candidate) -> Result<(MichiganMaterialCatalog, u64, u64)> {
    if inputs.physical.terminal_source_pins.get("defines_sha256")
        != Some(&hex(&sha256_of(inputs.defines.as_bytes())))
    {
        return Err(refused(
            "physical terminal authority uses different defines bytes",
        ));
    }
    let base = MichiganMaterialCatalog::from_statewide_qualification(
        &inputs.defines,
        &inputs.qualification,
        inputs.physical.clone(),
        Vec::new(),
    )?;
    let capacity = base
        .corridors()
        .iter()
        .find(|row| row.key == candidate.capacity_key)
        .ok_or_else(|| refused("selected capacity key is absent"))?
        .capacity_grams_per_period;
    if !inputs
        .physical
        .capacity_groups
        .iter()
        .any(|group| group.key == candidate.capacity_key)
    {
        return Err(refused(
            "selected capacity must be an authored physical network group",
        ));
    }
    let process = base
        .processes()
        .iter()
        .find(|row| row.key == candidate.food_process)
        .ok_or_else(|| refused("selected food process is absent"))?;
    if process.output_good_key != "prepared_food" {
        return Err(refused("selected process must produce prepared_food"));
    }
    let opening = process
        .inputs
        .iter()
        .find(|input| input.good_key == "paper_packaging")
        .ok_or_else(|| refused("selected food process does not use paper_packaging"))?
        .opening_quantity;
    if candidate.constrained_grams == 0 || candidate.constrained_grams >= capacity {
        return Err(refused(
            "constrained capacity must be positive and strictly below baseline",
        ));
    }
    if candidate.shortage_opening >= opening {
        return Err(refused(
            "shortage opening stock must be strictly below baseline; zero is allowed",
        ));
    }
    if base.horizon_ticks() != PERIODS {
        return Err(refused(
            "candidate must have the current sixteen-period horizon",
        ));
    }
    let freight = MichiganCapacityOverride {
        capacity_key: candidate.capacity_key.clone(),
        grams_per_period: candidate.constrained_grams,
    };
    let packaging = MichiganOpeningStockOverride {
        process_key: candidate.food_process.clone(),
        good_key: "paper_packaging".to_owned(),
        quantity: candidate.shortage_opening,
    };
    let interventions = vec![
        MichiganIntervention {
            preset: MichiganDeliveryPreset::StatewideFreightConstraint,
            capacities: vec![freight.clone()],
            opening_stocks: vec![],
            routes: vec![],
        },
        MichiganIntervention {
            preset: MichiganDeliveryPreset::StatewidePackagingShortage,
            capacities: vec![],
            opening_stocks: vec![packaging.clone()],
            routes: vec![],
        },
        MichiganIntervention {
            preset: MichiganDeliveryPreset::StatewideBoth,
            capacities: vec![freight],
            opening_stocks: vec![packaging],
            routes: vec![],
        },
    ];
    let catalog = MichiganMaterialCatalog::from_statewide_qualification(
        &inputs.defines,
        &inputs.qualification,
        inputs.physical.clone(),
        interventions,
    )?;
    Ok((catalog, capacity, opening))
}
fn run_case(
    catalog: &MichiganMaterialCatalog,
    delivery: MichiganDeliveryPreset,
    selected_capacity: &str,
) -> Result<Case> {
    let begin = Instant::now();
    let selected = catalog.with_preset(delivery)?;
    let preset = MichiganContentPreset::new_campaign(delivery);
    let foundation = preset.create_foundation(catalog)?;
    let foundation_hash = hex(&foundation.digest());
    let foundation_bytes = foundation.canonical_bytes().len();
    let mut session = foundation.into_session()?;
    if session
        .material()
        .state()
        .route_stages
        .iter()
        .any(|stage| stage.stage_index != 0 || stage.travel_periods != 1)
    {
        return Err(refused(
            "road qualification expects one 28-day journey stage per routed relationship",
        ));
    }
    let compile_ms = begin.elapsed().as_millis();
    let advancing = Instant::now();
    let mut periods = Vec::with_capacity(16);
    for period in 1..=PERIODS {
        let actions = OrderedPracticeActionBatch::empty(
            session.graph_session().session_identity().clone(),
            period,
        )
        .map_err(|error| {
            refused(format!(
                "{} period {period} actions: {error:?}",
                preset.id()
            ))
        })?;
        let prepared = session
            .prepare_advance(&actions)
            .map_err(|error| refused(format!("{} period {period}: {error}", preset.id())))?;
        if sha256_of(prepared.material().receipt_bytes()) != prepared.identity().receipt_digest() {
            return Err(refused("material receipt identity mismatch"));
        }
        let row = facts::period(
            &selected,
            session.material().state(),
            &prepared,
            selected_capacity,
        )?;
        session
            .commit_prepared_and_publish(&mut CollectingSink::default(), prepared, |_| {
                Ok::<_, ()>(ReplayCommitDisposition::Committed)
            })
            .map_err(|error| {
                refused(format!(
                    "{} period {period} publication: {error:?}",
                    preset.id()
                ))
            })?;
        if hex(&session.current_world_hash()?) != row.world_sha256 {
            return Err(refused("published world differs from committed report"));
        }
        periods.push(row);
    }
    Ok(Case {
        preset: preset.id(),
        captured_content_sha256: hex(&selected.defines_hash()),
        foundation_sha256: foundation_hash,
        captured_bytes: selected.defines_bytes().len(),
        foundation_bytes,
        compile_ms,
        advance_ms: advancing.elapsed().as_millis(),
        periods,
    })
}
pub fn experiment(inputs: &Inputs, candidate: &Candidate) -> Result<Report> {
    let (catalog, baseline_capacity, baseline_opening) = catalog(inputs, candidate)?;
    let mut cases = BTreeMap::new();
    for (name, preset) in [
        (BASELINE, MichiganDeliveryPreset::StatewideBaseline),
        (FREIGHT, MichiganDeliveryPreset::StatewideFreightConstraint),
        (
            PACKAGING,
            MichiganDeliveryPreset::StatewidePackagingShortage,
        ),
        (BOTH, MichiganDeliveryPreset::StatewideBoth),
    ] {
        eprintln!("Running {name}: {PERIODS} periods");
        cases.insert(name, run_case(&catalog, preset, &candidate.capacity_key)?);
    }
    let owners = catalog
        .sites()
        .iter()
        .map(|site| {
            (
                site.key.clone(),
                OwnerIdentity {
                    county_geoid: site.county_geoid.clone(),
                    sector_code: site.sector_code.clone(),
                    role: site.role,
                },
            )
        })
        .collect();
    let processes = catalog
        .processes()
        .iter()
        .map(|process| {
            let good = catalog
                .good(&process.output_good_key)
                .ok_or_else(|| refused("process output good missing"))?;
            Ok((
                process.key.clone(),
                ProcessIdentity {
                    owner: process.site_key.clone(),
                    output_good: good.key.clone(),
                    output_unit: good.unit_key.clone(),
                },
            ))
        })
        .collect::<Result<_>>()?;
    let routes = catalog
        .routes()
        .iter()
        .map(|route| {
            let good = catalog
                .good(&route.good_key)
                .ok_or_else(|| refused("route good missing"))?;
            Ok((
                route.key.clone(),
                RouteIdentity {
                    supplier: route.supplier_site_key.clone(),
                    buyer: route.buyer_site_key.clone(),
                    good: good.key.clone(),
                    unit: good.unit_key.clone(),
                    grams_per_unit: good.grams_per_unit,
                    path: route.path.clone(),
                },
            ))
        })
        .collect::<Result<_>>()?;
    let mut report = Report {
        schema: "MichiganStatewideExperimentV1",
        evidence_scope: if inputs.physical.source.pbf_url.starts_with("synthetic://") {
            "synthetic focused test only"
        } else {
            "candidate runtime qualification; not PostgreSQL or native acceptance"
        },
        qualified: false,
        persisted: false,
        parameter_evidence: "Designed",
        outcome_evidence: "Derived",
        candidate: candidate.clone(),
        baseline_capacity_grams: baseline_capacity,
        baseline_packaging_opening: baseline_opening,
        input_sha256: inputs.hashes.clone(),
        qualification_source_pins: serde_json::from_slice(&inputs.qualification)?,
        road_source: inputs.physical.source.clone(),
        terminal_source_pins: inputs.physical.terminal_source_pins.clone(),
        owners,
        processes,
        routes,
        cases,
        witnesses: Witnesses::default(),
    };
    report.witnesses = witness::find(&report)?;
    report.qualified = report.witnesses.missing.is_empty();
    Ok(report)
}
