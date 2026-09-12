//! Exact textual readings from one disclosed production snapshot.

use babylon_persistence::{
    production_observation::ProductionSite, production_observation::ProductionSnapshot,
};
use std::fmt::Write as _;

use super::ProductionReadingSection;
use crate::observer_ui::grouped;
use crate::production_brief::committed_plan_status;
use crate::production_freight::{account_reading, format_freight_mass, shared_accounts};

pub(super) fn describe(
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
    section: ProductionReadingSection,
) -> String {
    match section {
        ProductionReadingSection::Flow => describe_flow(site, snapshot),
        ProductionReadingSection::Freight => describe_freight(site, snapshot),
        ProductionReadingSection::Work => describe_work(site, snapshot),
        ProductionReadingSection::Sources => describe_sources(site, snapshot),
    }
}

pub(super) fn reading_headline(
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
    period: u64,
) -> String {
    let mut value = if period == 0 {
        "Foundation / Designed\n".to_owned()
    } else {
        format!("Period {period} / committed reading\n")
    };
    let mut outputs = std::collections::BTreeSet::new();
    for process in &site.processes {
        if !outputs.insert((&process.output_good_id, &process.output_unit_id)) {
            continue;
        }
        let output = snapshot.material_balance.as_ref().and_then(|balance| {
            balance.rows.iter().find(|row| {
                row.site_id == site.id
                    && row.good_id == process.output_good_id
                    && row.unit_id == process.output_unit_id
            })
        });
        if let Some(output) = output {
            writeln!(
                value,
                "{} {} produced / Derived · {}",
                grouped(output.produced),
                output.unit,
                process.output_good
            )
            .expect("String write");
        } else {
            writeln!(
                value,
                "{}: {}",
                process.output_good,
                crate::production_brief::process_plan_status(process)
            )
            .expect("String write");
        }
    }
    if site.processes.is_empty() {
        writeln!(value, "{}", committed_plan_status(site)).expect("String write");
    }
    let accounts: Vec<_> = snapshot
        .staffing_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
        .collect();
    if accounts.is_empty() {
        value.push_str("Modeled workforce not disclosed");
    }
    for account in accounts {
        writeln!(
            value,
            "{} employed · {} reserve / {}",
            grouped(account.employed),
            grouped(account.reserve),
            if account.completed.is_some() {
                "Derived"
            } else {
                "Designed"
            }
        )
        .expect("String write");
    }
    value.trim_end().to_owned()
}

pub(super) fn describe_flow(site: &ProductionSite, snapshot: &ProductionSnapshot) -> String {
    let mut value = String::new();
    for process in &site.processes {
        writeln!(
            value,
            "{} / {}\n{}",
            process.name,
            process.output_good,
            crate::production_brief::process_plan_status(process)
        )
        .expect("String write");
        if let (Some(done), Some(plan)) = (process.produced_batches, process.planned_batches) {
            writeln!(
                value,
                "COMMITTED PRODUCTION\n{done} of {plan} planned batches"
            )
            .expect("String write");
        }
        writeln!(
            value,
            "{} {} / batch (Designed)\nNext-period capacity: {} batches\n\nINPUTS / ON HAND",
            grouped(process.output_per_batch),
            process.output_unit,
            grouped(process.available_batches)
        )
        .expect("String write");
        for input in &process.inputs {
            writeln!(
                value,
                "{}: {} {}",
                input.good,
                grouped(input.on_hand),
                input.unit
            )
            .expect("String write");
        }
        if process.inputs.is_empty() {
            value.push_str("No material inputs in this recipe.\n");
        }
        value.push('\n');
    }
    if site.processes.is_empty() {
        writeln!(value, "{}", committed_plan_status(site)).expect("String write");
    }
    for account in snapshot
        .final_demand_accounts
        .iter()
        .filter(|account| account.retailer_site_ids.contains(&site.id))
    {
        writeln!(value, "COUNTY FINAL DEMAND / {}\n{} {} ordered · {} fulfilled · {} outstanding\nDelivery to end buyers; consumption is not recorded.", account.good, grouped(account.ordered), account.unit, grouped(account.fulfilled), grouped(account.outstanding)).expect("String write");
    }
    describe_material_balance(&mut value, site, snapshot);
    value.push_str("\nINVENTORY\n");
    for stock in &site.inventory {
        writeln!(
            value,
            "{}: {} {}",
            stock.good,
            grouped(stock.quantity),
            stock.unit
        )
        .expect("String write");
    }
    value
}

pub(super) fn describe_freight(site: &ProductionSite, snapshot: &ProductionSnapshot) -> String {
    let mut value = String::new();
    let accounts = shared_accounts(snapshot, Some(&site.id));
    if accounts.is_empty() {
        value.push_str("No shared freight pool disclosed for this subject.\n");
    }
    if accounts.len() > 3 {
        writeln!(value, "{} shared capacity accounts; showing the three with least next-opening availability.\n", accounts.len()).expect("String write");
    }
    for account in accounts.into_iter().take(3) {
        value.push_str(&account_reading(account, snapshot));
        value.push('\n');
    }
    value.push_str("\nPHYSICAL DELIVERIES / TO DATE\n");
    for route in snapshot
        .routes
        .iter()
        .filter(|route| route.buyer_site_id == site.id || route.supplier_site_id == site.id)
    {
        let other = if route.buyer_site_id == site.id {
            &route.supplier_site_id
        } else {
            &route.buyer_site_id
        };
        let name = snapshot
            .sites
            .iter()
            .find(|site| site.id == *other)
            .map_or(other.as_str(), |site| site.name.as_str());
        writeln!(
            value,
            "{} | {}\n{} / {} {} delivered | {} unshipped\n",
            name,
            match route.transport_kind {
                babylon_persistence::production_observation::ProductionRouteTransport::Local =>
                    "Local inter-owner transfer".into(),
                babylon_persistence::production_observation::ProductionRouteTransport::Staged =>
                    format!("{} periods travel", route.travel_periods),
            },
            grouped(route.delivered),
            grouped(route.ordered),
            route.unit,
            route
                .ordered
                .checked_sub(route.shipped)
                .map_or_else(|| "unavailable".into(), grouped)
        )
        .expect("String write");
    }
    value.push_str("Deliveries record quantities, not payments.\n");
    value
}

pub(super) fn describe_work(site: &ProductionSite, snapshot: &ProductionSnapshot) -> String {
    let mut value = String::new();
    describe_staffing_accounts(&mut value, site, snapshot);
    describe_labor_accounts(&mut value, site, snapshot);
    value.push_str("\nLABOR BUDGET / DERIVED\n");
    for labor in site.processes.iter().flat_map(|process| &process.labor) {
        writeln!(
            value,
            "{} {} available | {} / batch (Designed)",
            grouped(labor.available),
            labor.unit,
            grouped(labor.quantity_per_batch)
        )
        .expect("String write");
    }
    for handling in snapshot
        .merchant_handling_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
    {
        value.push_str("\nMERCHANT HANDLING\n");
        if let Some(done) = &handling.completed {
            writeln!(
                value,
                "Period {}: {} handled · {} / {} labor-hours used / needed",
                done.period,
                format_freight_mass(done.handled_grams),
                grouped(done.used_hours),
                grouped(done.needed_hours)
            )
            .expect("String write");
        } else {
            value.push_str("Foundation; no completed handling work.\n");
        }
        value.push_str("Handling moves existing goods; it does not create productive output.\n");
    }
    value.trim_start().to_owned()
}

fn describe_sources(site: &ProductionSite, snapshot: &ProductionSnapshot) -> String {
    let mut value = format!(
        "COUNTY-SECTOR OWNER / NAICS {}\nSector {} · {:?}\n",
        site.industry_code, site.sector_code, site.role
    );
    for process in &site.processes {
        writeln!(
            value,
            "\nRECIPE / DESIGNED / {}\n{} {} / batch",
            process.name,
            grouped(process.output_per_batch),
            process.output_unit
        )
        .expect("String write");
        for input in &process.inputs {
            writeln!(
                value,
                "{}: {} {} / batch",
                input.good,
                grouped(input.quantity_per_batch),
                input.unit
            )
            .expect("String write");
        }
    }
    if let Some(jobs) = site.observed_employment {
        writeln!(value, "\nINDUSTRY EMPLOYMENT / OBSERVED 2024\n{} annual-average jobs (QCEW; separate from modeled people and hours)", grouped(jobs)).expect("String write");
    }
    describe_sector_context(&mut value, site, snapshot);
    writeln!(
        value,
        "\nCAPTURED AUTHORITY\n{}",
        snapshot.content_authority_sha256
    )
    .expect("String write");
    if let Some(source) = &snapshot.road_source {
        writeln!(
            value,
            "Road extract: {}\nReplication: {}\nRouting profile: {}\nGraph: {}",
            source.pbf_url,
            source.replication_timestamp,
            source.routing_profile_version,
            source.graph_sha256
        )
        .expect("String write");
        value.push_str("Road data © OpenStreetMap contributors, available under the Open Database License (ODbL).\nhttps://www.openstreetmap.org/copyright\n");
    }
    value.push_str("\nSCENE KEY\nEqual-height structures identify county cohorts; height and spacing carry no quantity or geography. Arrows point from disclosed suppliers to buyers. Cyan links enter the selection; copper links leave it. Packets are actual in-transit lots at static schematic positions.\n");
    value
}

pub(super) fn describe_material_balance(
    value: &mut String,
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
) {
    let Some(balance) = &snapshot.material_balance else {
        value.push_str("\nNo completed stock-movement account at this point.\n");
        return;
    };
    let mut rows = balance
        .rows
        .iter()
        .filter(|row| row.site_id == site.id)
        .peekable();
    if rows.peek().is_none() {
        value.push_str("\nNo stock-movement account disclosed for this subject.\n");
        return;
    }
    writeln!(value, "\nSTOCK MOVEMENT / PERIOD {}", balance.period).expect("String write");
    for row in rows {
        if row.local_received != 0 || row.local_transferred != 0 || row.final_demand_fulfilled != 0
        {
            writeln!(value, "{} / {}\nOpened {} + arrived {} + received locally {} + produced {}\n= consumed {} + dispatched {} + transferred locally {} + final demand {} + closed {}", row.good, row.unit, grouped(row.opening), grouped(row.arrivals), grouped(row.local_received), grouped(row.produced), grouped(row.consumed), grouped(row.dispatched), grouped(row.local_transferred), grouped(row.final_demand_fulfilled), grouped(row.closing)).expect("String write");
            continue;
        }
        writeln!(
            value,
            "{} / {}\nOpened {} + arrived {} + produced {}\n= consumed {} + dispatched {} + closed {}",
            row.good,
            row.unit,
            grouped(row.opening),
            grouped(row.arrivals),
            grouped(row.produced),
            grouped(row.consumed),
            grouped(row.dispatched),
            grouped(row.closing),
        )
        .expect("String write");
    }
}

fn describe_sector_context(
    value: &mut String,
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
) {
    let subjects: std::collections::BTreeSet<_> = snapshot
        .process_attributions
        .iter()
        .filter(|link| link.site_id == site.id)
        .map(|link| &link.cohort_subject)
        .collect();
    for context in snapshot.observed_contexts.iter().filter(|context| {
        context.county_geoid == site.county_geoid && subjects.contains(&context.subject)
    }) {
        writeln!(
            value,
            "\nSECTOR CONTEXT / OBSERVED {}\n{} | NAICS {}\n{} establishments",
            context.vintage,
            context.sector_title,
            context.sector_code,
            grouped(context.annual_avg_estabs_count),
        )
        .expect("String write");
        for (metric, prefix, suffix, undisclosed) in [
            (
                context.annual_avg_emplvl,
                "",
                " annual-average jobs",
                "Annual-average jobs: not disclosed",
            ),
            (
                context.total_annual_wages,
                "USD ",
                " annual payroll",
                "Annual payroll: not disclosed",
            ),
            (
                context.annual_avg_wkly_wage,
                "USD ",
                " mean weekly wage",
                "Mean weekly wage: not disclosed",
            ),
        ] {
            match metric {
                Some(metric) => writeln!(value, "{prefix}{}{suffix}", grouped(metric)),
                None => writeln!(value, "{undisclosed}"),
            }
            .expect("String write");
        }
        let shared_ids: std::collections::BTreeSet<_> = snapshot
            .process_attributions
            .iter()
            .filter(|link| link.cohort_subject == context.subject)
            .map(|link| link.site_id.as_str())
            .collect();
        let mut names: Vec<_> = snapshot
            .sites
            .iter()
            .filter(|other| shared_ids.contains(other.id.as_str()))
            .map(|other| other.name.as_str())
            .collect();
        names.sort_unstable();
        writeln!(
            value,
            "Modeled processes sharing this context: {}\nThis county-sector total does not assign workers to a process.\nSource: BLS QCEW / {}\n{}",
            names.join("; "),
            context.source_file,
            context.source_url,
        )
        .expect("String write");
    }
}

fn describe_staffing_accounts(
    value: &mut String,
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
) {
    let mut disclosed = false;
    for account in snapshot
        .staffing_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
    {
        disclosed = true;
        value.push_str(if account.completed.is_some() {
            "\nMODELED WORKFORCE / DERIVED\n"
        } else {
            "\nMODELED WORKFORCE / DESIGNED\n"
        });
        writeln!(
            value,
            "{} employed + {} reserve = {} people\n{} hours per person / period (Designed)",
            grouped(account.employed),
            grouped(account.reserve),
            grouped(account.labor_force),
            grouped(account.hours_per_person),
        )
        .expect("String write");
        if let Some(completed) = &account.completed {
            writeln!(
                value,
                "\nSTAFFING / PERIOD {}\nOpening: {} employed, {} reserve\nHires: {} | separations: {} | target: {} employed\nWork request: {} hours | prior period: {} hours\nOne-period retention: {} hours\n",
                completed.period,
                grouped(completed.opening_employed),
                grouped(completed.opening_reserve),
                grouped(completed.hires),
                grouped(completed.separations),
                grouped(completed.target_employed),
                grouped(completed.current_unretained_hours),
                grouped(completed.previous_unretained_hours),
                grouped(completed.retained_hours),
            )
            .expect("String write");
        } else {
            value.push_str("Opening workforce; no completed staffing period.\n");
        }
    }
    if disclosed {
        value.push_str("Observed QCEW jobs are separate; these accounts record no payments.\n");
    } else {
        value.push_str("\nMODELED WORKFORCE\nNo workforce account disclosed for this subject.\n");
    }
}

fn describe_labor_accounts(
    value: &mut String,
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
) {
    for account in snapshot
        .labor_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
    {
        if let Some(completed) = &account.completed {
            writeln!(
                value,
                "\nCOMMITTED WORK TIME / PERIOD {} / DERIVED\n{} used + {} unused = {} available\nProduction planned: {} {}",
                completed.period,
                grouped(completed.used),
                grouped(completed.unused),
                grouped(completed.opening),
                grouped(completed.planned),
                account.unit,
            )
            .expect("String write");
            if site.processes.is_empty()
                || completed.handling_needed != 0
                || completed.handling_used != 0
            {
                writeln!(
                    value,
                    "Handling: {} needed · {} used {}",
                    grouped(completed.handling_needed),
                    grouped(completed.handling_used),
                    account.unit
                )
                .expect("String write");
            }
            value.push_str("Time accounts do not measure job losses.\n");
        }
        writeln!(
            value,
            "Next opening (period {}): {} {} (Derived)",
            account.next_opening_period,
            grouped(account.next_opening_available),
            account.unit,
        )
        .expect("String write");
    }
}
