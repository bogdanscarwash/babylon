//! Comparable campaign totals from the same disclosed owners and material principals.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use babylon_persistence::{
    observer_reader::ObserverEconomySnapshot, production_observation::ProductionFinalDemandAccount,
    production_observation::ProductionSite, production_observation::ProductionSiteRole,
    production_observation::ProductionSnapshot,
};

use crate::map_economy_lens::{
    project_map_lens, CountyLensReading, MapLens, MaterialGoodKey, MaterialLensKind,
};

use super::staffing_difference;
use crate::workforce::{
    validate_staffing_balance, validate_staffing_period, StaffingError, StaffingIdentity,
};

fn owners(snapshot: &ProductionSnapshot) -> Result<BTreeMap<&str, &ProductionSite>, &'static str> {
    let mut owners = BTreeMap::new();
    for site in &snapshot.sites {
        if owners.insert(site.id.as_str(), site).is_some() {
            return Err("duplicate owner identity");
        }
    }
    if owners.is_empty() {
        return Err("no modeled owners disclosed");
    }
    Ok(owners)
}

fn compatible_owners(
    current: &ProductionSnapshot,
    compared: &ProductionSnapshot,
) -> Result<(), &'static str> {
    let current = owners(current)?;
    let compared = owners(compared)?;
    if current.len() != compared.len()
        || current.iter().any(|(id, site)| {
            compared.get(id).is_none_or(|other| {
                (
                    site.county_geoid.as_str(),
                    site.sector_code.as_str(),
                    site.role,
                    site.industry_code.as_str(),
                ) != (
                    other.county_geoid.as_str(),
                    other.sector_code.as_str(),
                    other.role,
                    other.industry_code.as_str(),
                )
            })
        })
    {
        return Err("owner, county, sector or role coverage differs");
    }
    Ok(())
}

struct WorkforceTotals<'a> {
    principals: BTreeSet<StaffingIdentity<'a>>,
    employed: u64,
    reserve: u64,
}

fn workforce(
    snapshot: &ProductionSnapshot,
    tick: u64,
) -> Result<WorkforceTotals<'_>, &'static str> {
    let owners = owners(snapshot)?;
    let mut identities = BTreeSet::new();
    let mut pools = BTreeSet::new();
    let mut staffed = BTreeSet::new();
    let (mut employed, mut reserve) = (0_u64, 0_u64);
    for account in &snapshot.staffing_accounts {
        if !identities.insert(StaffingIdentity::from(account)) || !pools.insert(&account.pool_id) {
            return Err("duplicate workforce principal");
        }
        if !owners.contains_key(account.site_id.as_str()) {
            return Err("workforce owner is not disclosed");
        }
        validate_staffing_period(account, tick).map_err(StaffingError::message)?;
        validate_staffing_balance(account).map_err(StaffingError::message)?;
        staffed.insert(account.site_id.as_str());
        employed = employed
            .checked_add(account.employed)
            .ok_or("workforce quantity overflow")?;
        reserve = reserve
            .checked_add(account.reserve)
            .ok_or("workforce quantity overflow")?;
    }
    if staffed.len() != owners.len() {
        return Err("staffing accounts are missing for modeled owners");
    }
    Ok(WorkforceTotals {
        principals: identities,
        employed,
        reserve,
    })
}

fn write_workforce(
    output: &mut String,
    current: &ProductionSnapshot,
    compared: &ProductionSnapshot,
    tick: u64,
) -> Result<(), &'static str> {
    compatible_owners(current, compared)?;
    let left = workforce(current, tick)?;
    let right = workforce(compared, tick)?;
    if left.principals != right.principals {
        return Err("workforce principal, owner or labor unit coverage differs");
    }
    output.push_str(if tick == 0 {
        "Foundation workforce / Designed\n"
    } else {
        "Closing workforce / Derived\n"
    });
    staffing_difference(
        output,
        "All modeled employed",
        left.employed,
        right.employed,
    );
    staffing_difference(output, "All modeled reserve", left.reserve, right.reserve);
    Ok(())
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MaterialPrincipal<'a> {
    Process(&'a str, &'a str),
    Stock(&'a str),
    Route(&'a str, &'a str, &'a str),
}

fn material_principals<'a>(
    snapshot: &'a ProductionSnapshot,
    kind: MaterialLensKind,
    good: &MaterialGoodKey,
) -> Result<BTreeSet<MaterialPrincipal<'a>>, &'static str> {
    let mut rows = Vec::new();
    match kind {
        MaterialLensKind::ProducedThisPeriod => {
            for owner in &snapshot.sites {
                rows.extend(
                    owner
                        .processes
                        .iter()
                        .filter(|row| {
                            row.output_good_id == good.good_id && row.output_unit_id == good.unit_id
                        })
                        .map(|row| MaterialPrincipal::Process(&owner.id, &row.id)),
                );
            }
        }
        MaterialLensKind::OnHand => {
            for owner in &snapshot.sites {
                rows.extend(
                    owner
                        .inventory
                        .iter()
                        .filter(|row| row.good_id == good.good_id && row.unit_id == good.unit_id)
                        .map(|_| MaterialPrincipal::Stock(&owner.id)),
                );
            }
        }
        MaterialLensKind::InboundInTransit => {
            rows.extend(
                snapshot
                    .routes
                    .iter()
                    .filter(|row| row.good_id == good.good_id && row.unit_id == good.unit_id)
                    .map(|row| {
                        MaterialPrincipal::Route(&row.id, &row.supplier_site_id, &row.buyer_site_id)
                    }),
            );
            let mut lots = BTreeSet::new();
            for lot in snapshot
                .freight
                .iter()
                .filter(|row| row.good_id == good.good_id && row.unit_id == good.unit_id)
            {
                if !lots.insert(&lot.id) {
                    return Err("duplicate freight lot identity");
                }
            }
        }
    }
    let mut identities = BTreeSet::new();
    for row in rows {
        if !identities.insert(row) {
            return Err("duplicate selected-material principal");
        }
    }
    if identities.is_empty() {
        return Err("no account for this exact good and unit");
    }
    Ok(identities)
}

fn material_total(
    snapshot: &ObserverEconomySnapshot,
    lens: &MapLens,
) -> Result<(String, u64), &'static str> {
    let projection = project_map_lens(Some(snapshot), lens);
    if projection.counties.is_empty() {
        return Err(projection.unavailable.label());
    }
    let total = projection
        .counties
        .values()
        .try_fold(0_u64, |total, row| match row {
            CountyLensReading::Available(value) => total
                .checked_add(*value)
                .ok_or("material quantity overflow"),
            CountyLensReading::Unavailable(_) => {
                Err("no completed production receipt for the selected period")
            }
        })?;
    Ok((projection.unit, total))
}

fn write_material(
    output: &mut String,
    current: &ObserverEconomySnapshot,
    compared: &ObserverEconomySnapshot,
    lens: &MapLens,
) -> Result<(), &'static str> {
    let MapLens::Material {
        kind,
        good: Some(good),
    } = lens
    else {
        return Err("Select an exact good and unit in World's material lens");
    };
    let left = current
        .production
        .as_ref()
        .ok_or("current production is missing")?;
    let right = compared
        .production
        .as_ref()
        .ok_or("compared production is missing")?;
    compatible_owners(left, right)?;
    if material_principals(left, *kind, good)? != material_principals(right, *kind, good)? {
        return Err("selected-material owner or principal coverage differs");
    }
    if current.resolve_tick == 0 && *kind == MaterialLensKind::ProducedThisPeriod {
        return Err("foundation; no completed production period");
    }
    let (unit, value) = material_total(current, lens)?;
    let (_, other) = material_total(compared, lens)?;
    write_quantity(output, kind.label(), value, other, &unit);
    Ok(())
}

fn write_quantity(output: &mut String, label: &str, current: u64, compared: u64, unit: &str) {
    writeln!(output, "{label}: {current} / {compared} {unit}").expect("String write");
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct RetailIdentity<'a> {
    county: &'a str,
    principal: &'a str,
}

fn retail_accounts<'a>(
    snapshot: &'a ProductionSnapshot,
    good: &MaterialGoodKey,
) -> Result<BTreeMap<RetailIdentity<'a>, &'a ProductionFinalDemandAccount>, &'static str> {
    let owners = owners(snapshot)?;
    let mut rows = BTreeMap::new();
    let mut counties = BTreeSet::new();
    let mut order_ids = BTreeSet::new();
    for row in snapshot
        .final_demand_accounts
        .iter()
        .filter(|row| row.good_id == good.good_id && row.unit_id == good.unit_id)
    {
        if rows
            .insert(
                RetailIdentity {
                    county: &row.county_geoid,
                    principal: &row.demand_principal_id,
                },
                row,
            )
            .is_some()
            || !counties.insert(&row.county_geoid)
        {
            return Err("duplicate county or final-demand principal");
        }
        let retailers: BTreeSet<_> = row.retailer_site_ids.iter().collect();
        if retailers.is_empty()
            || retailers.len() != row.retailer_site_ids.len()
            || retailers.iter().any(|id| {
                owners.get(id.as_str()).is_none_or(|owner| {
                    owner.role != ProductionSiteRole::Retail
                        || owner.county_geoid != row.county_geoid
                })
            })
        {
            return Err("retail owner coverage is missing or inconsistent");
        }
        let mut ordered_retailers = BTreeSet::new();
        let (mut fulfilled, mut outstanding) = (0_u64, 0_u64);
        for order in &row.orders {
            if !order_ids.insert(&order.order_id) {
                return Err("duplicate final-demand order principal");
            }
            ordered_retailers.insert(&order.retailer_site_id);
            if order.fulfilled.checked_add(order.outstanding) != Some(order.ordered) {
                return Err("final-demand order quantities do not conserve");
            }
            fulfilled = fulfilled
                .checked_add(order.fulfilled)
                .ok_or("retail quantity overflow")?;
            outstanding = outstanding
                .checked_add(order.outstanding)
                .ok_or("retail quantity overflow")?;
        }
        if retailers != ordered_retailers
            || fulfilled != row.fulfilled
            || outstanding != row.outstanding
            || fulfilled.checked_add(outstanding) != Some(row.ordered)
        {
            return Err("county final-demand account does not match its orders");
        }
    }
    if rows.is_empty() {
        return Err("no final-demand account for this exact good and unit");
    }
    Ok(rows)
}

#[derive(Default)]
struct RetailTotals {
    fulfilled: u64,
    newly_fulfilled: u64,
    stock: u64,
}

impl RetailTotals {
    fn add(&mut self, row: &ProductionFinalDemandAccount, tick: u64) -> Result<(), &'static str> {
        let newly_fulfilled = match (&row.completed, tick) {
            (None, 0) if row.fulfilled == 0 => 0,
            (Some(done), tick)
                if tick > 0
                    && done.period == tick
                    && done.closing_fulfilled == row.fulfilled
                    && done.opening_fulfilled.checked_add(done.newly_fulfilled)
                        == Some(done.closing_fulfilled) =>
            {
                done.newly_fulfilled
            }
            _ => {
                return Err("final-demand receipt is missing or does not match the selected period")
            }
        };
        self.fulfilled = self
            .fulfilled
            .checked_add(row.fulfilled)
            .ok_or("retail quantity overflow")?;
        self.newly_fulfilled = self
            .newly_fulfilled
            .checked_add(newly_fulfilled)
            .ok_or("retail quantity overflow")?;
        self.stock = self
            .stock
            .checked_add(row.retail_stock_on_hand)
            .ok_or("retail quantity overflow")?;
        Ok(())
    }
}

fn retail_order_principals(row: &ProductionFinalDemandAccount) -> BTreeSet<(&str, &str)> {
    row.orders
        .iter()
        .map(|order| (order.order_id.as_str(), order.retailer_site_id.as_str()))
        .collect()
}

fn write_retail(
    output: &mut String,
    current: &ProductionSnapshot,
    compared: &ProductionSnapshot,
    good: &MaterialGoodKey,
    tick: u64,
    unit: &str,
) -> Result<(), &'static str> {
    compatible_owners(current, compared)?;
    let left = retail_accounts(current, good)?;
    let right = retail_accounts(compared, good)?;
    if left.keys().ne(right.keys()) {
        return Err("county or final-demand principal coverage differs");
    }
    let (mut totals, mut other_totals) = (RetailTotals::default(), RetailTotals::default());
    for (key, row) in left {
        let other = right[&key];
        if retail_order_principals(row) != retail_order_principals(other) {
            return Err("final-demand order or retailer principal coverage differs");
        }
        totals.add(row, tick)?;
        other_totals.add(other, tick)?;
    }
    write_quantity(
        output,
        "Delivered to end buyers to date",
        totals.fulfilled,
        other_totals.fulfilled,
        unit,
    );
    if tick == 0 {
        output.push_str("Foundation; no completed retail deliveries.\n");
    } else {
        write_quantity(
            output,
            "Delivered to end buyers this period",
            totals.newly_fulfilled,
            other_totals.newly_fulfilled,
            unit,
        );
    }
    write_quantity(
        output,
        "Unsold retail stock",
        totals.stock,
        other_totals.stock,
        unit,
    );
    Ok(())
}

pub(super) fn write(
    output: &mut String,
    current: &ObserverEconomySnapshot,
    compared: &ObserverEconomySnapshot,
    lens: &MapLens,
) {
    let (Some(left), Some(right)) = (&current.production, &compared.production) else {
        return;
    };
    let counties: BTreeSet<_> = left.sites.iter().map(|site| &site.county_geoid).collect();
    writeln!(
        output,
        "MODELED CAMPAIGN TOTALS / {} owners / {} counties",
        left.sites.len(),
        counties.len()
    )
    .expect("String write");
    if let Err(error) = write_workforce(output, left, right, current.resolve_tick) {
        writeln!(output, "Aggregate workforce unavailable: {error}.").expect("String write");
    }
    let MapLens::Material {
        kind,
        good: Some(good),
    } = lens
    else {
        output.push_str("Select an exact good and unit in World's material lens to compare physical totals.\n\n");
        return;
    };
    let choice = crate::map_economy_lens::material_choices(current, *kind)
        .ok()
        .and_then(|choices| choices.into_iter().find(|choice| choice.key == *good));
    let Some(choice) = choice else {
        output
            .push_str("Selected material unavailable: exact good and unit are not disclosed.\n\n");
        return;
    };
    writeln!(
        output,
        "Selected material / {} / {}",
        choice.label, choice.unit
    )
    .expect("String write");
    if let Err(error) = write_material(output, current, compared, lens) {
        writeln!(output, "Selected material unavailable: {error}.").expect("String write");
    }
    if let Err(error) = write_retail(
        output,
        left,
        right,
        good,
        current.resolve_tick,
        &choice.unit,
    ) {
        writeln!(output, "Retail totals unavailable: {error}.").expect("String write");
    }
    output.push('\n');
}
