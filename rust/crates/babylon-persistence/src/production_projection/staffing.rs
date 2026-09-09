//! Worker accounts from authenticated adjacent graph states and native event rows.
//! This projection checks receipt/end-point agreement; it never runs hiring policy.

use std::collections::BTreeMap;

use babylon_bsl::{
    identity_codec::{project_stored_field_value_v1, StableBslValueV1},
    types::{BslType, EnumRegistry, FieldDecl, FieldKind},
};
use babylon_graph::{stable_element::StableElementKeyV1, stable_state::StableGraphStateV1};
use babylon_tick::{
    material_staffing::{
        StaffingCompositionV1, StaffingNodeBindingV1, EMPLOYED_POPULATION,
        PREVIOUS_UNRETAINED_HOURS, RESERVE_POPULATION, STAFFING_COMPOSITION_ID_V1,
    },
    material_world::MaterialWorldRegisterV3,
};

use super::ProductionProjectionErrorV1;
use crate::{
    michigan_economy::digest_hex, stored_tick::StoredEventV2, CompletedProductionStaffingV1,
    ProductionStaffingAccountV1, ProductionStaffingSubjectV1,
};

type Result<T> = std::result::Result<T, ProductionProjectionErrorV1>;
const EVENT: &str = "WORKFORCE_STAFFING";
const INTEGER_FIELDS: [&str; 12] = [
    "period",
    "opening-employed",
    "opening-reserve",
    "previous-unretained-hours",
    "current-unretained-hours",
    "retained-hours",
    "target-employed",
    "hires",
    "separations",
    "closing-employed",
    "closing-reserve",
    "next-opening-hours",
];

pub(crate) fn project_staffing_accounts_v1(
    composition: &StaffingCompositionV1,
    graph: &StableGraphStateV1,
    register: &MaterialWorldRegisterV3,
    opening: Option<&StableGraphStateV1>,
    events: &[StoredEventV2],
) -> Result<Vec<ProductionStaffingAccountV1>> {
    let tick = register.completed_tick();
    if (tick == 0) != opening.is_none()
        || (tick == 0 && !events.is_empty())
        || tick.checked_add(1) != Some(register.state().period)
        || opening.is_some_and(|prior| prior.scenario_scope() != graph.scenario_scope())
    {
        return Err(ProductionProjectionErrorV1::History);
    }
    let mut receipts = BTreeMap::new();
    for event in events {
        if event.event_type != EVENT && event.emitting_rule != STAFFING_COMPOSITION_ID_V1 {
            continue;
        }
        if event.event_type != EVENT
            || event.emitting_rule != STAFFING_COMPOSITION_ID_V1
            || event.choice_receipt_ordinal.is_some()
        {
            return Err(ProductionProjectionErrorV1::History);
        }
        let (subject, values) = event_fields(event)?;
        if receipts
            .insert(
                subject
                    .canonical_bytes()
                    .map_err(|_| ProductionProjectionErrorV1::State)?,
                values,
            )
            .is_some()
        {
            return Err(ProductionProjectionErrorV1::History);
        }
    }
    let mut accounts = Vec::with_capacity(composition.bindings().len());
    for binding in composition.bindings() {
        let StableElementKeyV1::Node {
            scenario,
            local_name,
        } = binding.subject()
        else {
            return Err(ProductionProjectionErrorV1::Content);
        };
        if scenario != graph.scenario_scope() {
            return Err(ProductionProjectionErrorV1::State);
        }
        let closing = stocks(graph, binding)?;
        let pool = binding.pool();
        let next_hours = closing
            .employed
            .checked_mul(pool.policy().hours_per_person())
            .ok_or(ProductionProjectionErrorV1::Arithmetic)?;
        let mut labor = register.state().labor.iter().filter(|row| {
            row.site_id == pool.site_id()
                && row.unit_id == pool.unit_id()
                && row.period == register.state().period
        });
        if labor.next().map(|row| row.available) != Some(next_hours) || labor.next().is_some() {
            return Err(ProductionProjectionErrorV1::State);
        }
        let completed = if let Some(prior) = opening {
            let key = binding
                .subject()
                .canonical_bytes()
                .map_err(|_| ProductionProjectionErrorV1::State)?;
            let values = receipts
                .remove(&key)
                .ok_or(ProductionProjectionErrorV1::History)?;
            Some(completed_account(
                tick,
                stocks(prior, binding)?,
                closing,
                next_hours,
                values,
            )?)
        } else {
            None
        };
        accounts.push(ProductionStaffingAccountV1 {
            pool_id: digest_hex(&pool.pool_id().as_bytes()),
            site_id: digest_hex(&pool.site_id().as_bytes()),
            unit_id: digest_hex(&pool.unit_id().as_bytes()),
            subject: ProductionStaffingSubjectV1 {
                scenario: scenario.clone(),
                local_name: local_name.clone(),
            },
            hours_per_person: pool.policy().hours_per_person(),
            labor_force: pool.labor_force(),
            employed: closing.employed,
            reserve: closing.reserve,
            previous_unretained_hours: closing.previous,
            next_opening_period: register.state().period,
            next_opening_hours: next_hours,
            completed,
        });
    }
    if !receipts.is_empty() {
        return Err(ProductionProjectionErrorV1::History);
    }
    Ok(accounts)
}

#[derive(Clone, Copy)]
struct Stocks {
    employed: u64,
    reserve: u64,
    previous: u64,
}

fn stocks(graph: &StableGraphStateV1, binding: &StaffingNodeBindingV1) -> Result<Stocks> {
    let StableElementKeyV1::Node {
        scenario,
        local_name,
    } = binding.subject()
    else {
        return Err(ProductionProjectionErrorV1::Content);
    };
    if scenario != graph.scenario_scope()
        || !graph
            .rows()
            .nodes()
            .iter()
            .any(|(name, owner)| name == local_name && owner == "SOCIAL_CLASS")
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    let result = Stocks {
        employed: population_field(graph, local_name, EMPLOYED_POPULATION)?,
        reserve: population_field(graph, local_name, RESERVE_POPULATION)?,
        previous: population_field(graph, local_name, PREVIOUS_UNRETAINED_HOURS)?,
    };
    if result.employed.checked_add(result.reserve) != Some(binding.pool().labor_force()) {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(result)
}

fn population_field(graph: &StableGraphStateV1, node: &str, field: &str) -> Result<u64> {
    let mut values = graph
        .rows()
        .node_f64()
        .iter()
        .filter(|(name, key, _)| name == node && key == field);
    let bits = values.next().map(|(_, _, bits)| *bits);
    if values.next().is_some()
        || graph
            .rows()
            .node_currency()
            .iter()
            .any(|(name, key, _)| name == node && key == field)
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    let value = project_stored_field_value_v1(
        &FieldDecl {
            ty: BslType::Int,
            kind: FieldKind::Extensive,
        },
        bits,
        None,
        &EnumRegistry::default(),
    )
    .map_err(|_| ProductionProjectionErrorV1::State)?;
    integer(&value)
}

fn integer(value: &StableBslValueV1) -> Result<u64> {
    let StableBslValueV1::Int(value) = value else {
        return Err(ProductionProjectionErrorV1::State);
    };
    let value = u64::try_from(*value).map_err(|_| ProductionProjectionErrorV1::State)?;
    Ok(value)
}

fn event_fields(event: &StoredEventV2) -> Result<(&StableElementKeyV1, [u64; 12])> {
    if event.fields.len() != INTEGER_FIELDS.len() + 1 {
        return Err(ProductionProjectionErrorV1::History);
    }
    let fields: BTreeMap<_, _> = event
        .fields
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect();
    if fields.len() != event.fields.len() {
        return Err(ProductionProjectionErrorV1::History);
    }
    let Some(StableBslValueV1::Node(subject @ StableElementKeyV1::Node { .. })) =
        fields.get("subject")
    else {
        return Err(ProductionProjectionErrorV1::History);
    };
    let mut values = [0; 12];
    for (index, name) in INTEGER_FIELDS.iter().enumerate() {
        values[index] = integer(
            fields
                .get(name)
                .ok_or(ProductionProjectionErrorV1::History)?,
        )?;
    }
    Ok((subject, values))
}

fn completed_account(
    tick: u64,
    opening: Stocks,
    closing: Stocks,
    next_hours: u64,
    values: [u64; 12],
) -> Result<CompletedProductionStaffingV1> {
    let [period, opening_employed, opening_reserve, previous_unretained_hours, current_unretained_hours, retained_hours, target_employed, hires, separations, closing_employed, closing_reserve, next_opening_hours] =
        values;
    if period != tick
        || opening_employed != opening.employed
        || opening_reserve != opening.reserve
        || previous_unretained_hours != opening.previous
        || current_unretained_hours != closing.previous
        || closing_employed != closing.employed
        || closing_reserve != closing.reserve
        || target_employed != closing.employed
        || next_opening_hours != next_hours
        || (hires > 0 && separations > 0)
        || hires > opening.reserve
        || separations > opening.employed
        || opening
            .employed
            .checked_add(hires)
            .and_then(|n| n.checked_sub(separations))
            != Some(closing.employed)
        || opening
            .reserve
            .checked_add(separations)
            .and_then(|n| n.checked_sub(hires))
            != Some(closing.reserve)
    {
        return Err(ProductionProjectionErrorV1::History);
    }
    Ok(CompletedProductionStaffingV1 {
        period,
        opening_employed,
        opening_reserve,
        previous_unretained_hours,
        current_unretained_hours,
        retained_hours,
        target_employed,
        hires,
        separations,
    })
}

#[cfg(test)]
mod tests;
