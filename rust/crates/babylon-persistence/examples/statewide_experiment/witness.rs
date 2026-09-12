//! Compare committed counterfactuals; static reachability selects evidence, never shipments.
use super::{
    refused,
    report::{
        CompletedPeriod, FreightWitness, OutputDifference, Report, RouteDifference, Witnesses,
        WorkforceDifference,
    },
    run::{BASELINE, FREIGHT, PACKAGING},
    Result,
};
use babylon_persistence::michigan_material::MichiganMaterialPath;
use std::collections::{BTreeMap, VecDeque};

type Reachable = BTreeMap<String, Vec<String>>;
fn downstream(report: &Report, changed_routes: &[RouteDifference]) -> Reachable {
    let mut reached = Reachable::new();
    let mut queue = VecDeque::new();
    for route in changed_routes {
        let identity = &report.routes[&route.route];
        if let std::collections::btree_map::Entry::Vacant(entry) =
            reached.entry(identity.buyer.clone())
        {
            entry.insert(vec![route.route.clone()]);
            queue.push_back(identity.buyer.clone());
        }
    }
    while let Some(owner) = queue.pop_front() {
        for (key, route) in report
            .routes
            .iter()
            .filter(|(_, route)| route.supplier == owner)
        {
            if !reached.contains_key(&route.buyer) {
                let mut path = reached[&owner].clone();
                path.push(key.clone());
                reached.insert(route.buyer.clone(), path);
                queue.push_back(route.buyer.clone());
            }
        }
    }
    reached
}
fn output(
    report: &Report,
    baseline: &CompletedPeriod,
    constrained: &CompletedPeriod,
    reachable: &Reachable,
) -> Option<OutputDifference> {
    report.processes.iter().find_map(|(key, identity)| {
        let path = reachable.get(&identity.owner)?;
        let base = baseline.processes.get(key)?.output_units;
        let changed = constrained.processes.get(key)?.output_units;
        (base != changed).then(|| OutputDifference {
            period: baseline.period,
            process: key.clone(),
            owner: identity.owner.clone(),
            baseline_output_units: base,
            constrained_output_units: changed,
            supply_chain_routes: path.clone(),
        })
    })
}
fn workforce(
    baseline: &CompletedPeriod,
    constrained: &CompletedPeriod,
    reachable: &Reachable,
) -> Result<Option<WorkforceDifference>> {
    for (owner, path) in reachable {
        let base = &baseline
            .owners
            .get(owner)
            .ok_or_else(|| refused("witness owner absent in baseline"))?
            .staffing;
        let changed = &constrained
            .owners
            .get(owner)
            .ok_or_else(|| refused("witness owner absent in counterfactual"))?
            .staffing;
        let value = |fields: &BTreeMap<String, u64>, key: &str| {
            fields
                .get(key)
                .copied()
                .ok_or_else(|| refused(format!("staffing witness field absent: {key}")))
        };
        let (be, br) = (
            value(base, "closing-employed")?,
            value(base, "closing-reserve")?,
        );
        let (ce, cr) = (
            value(changed, "closing-employed")?,
            value(changed, "closing-reserve")?,
        );
        if (be, br) != (ce, cr) {
            return Ok(Some(WorkforceDifference {
                period: baseline.period,
                owner: owner.clone(),
                baseline_employed: be,
                constrained_employed: ce,
                baseline_reserve: br,
                constrained_reserve: cr,
                supply_chain_routes: path.clone(),
            }));
        }
    }
    Ok(None)
}
fn freight(report: &Report) -> Result<Option<FreightWitness>> {
    let baseline = &report.cases[BASELINE].periods;
    let constrained = &report.cases[FREIGHT].periods;
    for (index, (base, changed)) in baseline.iter().zip(constrained).enumerate() {
        if base.selected_capacity_reserved_grams <= changed.selected_capacity_reserved_grams {
            continue;
        }
        let changed_routes: Vec<_> = report
            .routes
            .iter()
            .filter_map(|(key, route)| {
                let MichiganMaterialPath::Routed { capacity_keys, .. } = &route.path else {
                    return None;
                };
                if !capacity_keys.contains(&report.candidate.capacity_key) {
                    return None;
                }
                let before = base.routes.get(key)?.dispatched;
                let after = changed.routes.get(key)?.dispatched;
                (before != after).then(|| RouteDifference {
                    route: key.clone(),
                    baseline_dispatched: before,
                    constrained_dispatched: after,
                })
            })
            .collect();
        let reached = downstream(report, &changed_routes);
        let mut output_difference = None;
        let mut workforce_difference = None;
        for (base, changed) in baseline.iter().zip(constrained).skip(index + 1) {
            if output_difference.is_none() {
                output_difference = output(report, base, changed, &reached);
            }
            if workforce_difference.is_none() {
                workforce_difference = workforce(base, changed, &reached)?;
            }
        }
        if let (Some(downstream_output), Some(downstream_workforce)) =
            (output_difference, workforce_difference)
        {
            return Ok(Some(FreightWitness {
                dispatch_period: base.period,
                capacity_key: report.candidate.capacity_key.clone(),
                baseline_reserved_grams: base.selected_capacity_reserved_grams,
                constrained_reserved_grams: changed.selected_capacity_reserved_grams,
                changed_routes,
                downstream_output,
                downstream_workforce,
            }));
        }
    }
    Ok(None)
}
pub fn find(report: &Report) -> Result<Witnesses> {
    let freight = freight(report)?;
    let identity = report
        .processes
        .get(&report.candidate.food_process)
        .ok_or_else(|| refused("selected food process missing from report"))?;
    let packaging = report.cases[BASELINE]
        .periods
        .iter()
        .zip(&report.cases[PACKAGING].periods)
        .find_map(|(base, changed)| {
            let before = base
                .processes
                .get(&report.candidate.food_process)?
                .output_units;
            let after = changed
                .processes
                .get(&report.candidate.food_process)?
                .output_units;
            (before != after).then(|| OutputDifference {
                period: base.period,
                process: report.candidate.food_process.clone(),
                owner: identity.owner.clone(),
                baseline_output_units: before,
                constrained_output_units: after,
                supply_chain_routes: Vec::new(),
            })
        });
    let mut missing = Vec::new();
    if freight.is_none() {
        missing.push("freight-only requires a selected-principal dispatch reduction and both later downstream production and employed/reserve differences");
    }
    if packaging.is_none() {
        missing.push("packaging-only requires a selected-food committed output difference");
    }
    Ok(Witnesses {
        freight,
        packaging,
        missing,
    })
}
