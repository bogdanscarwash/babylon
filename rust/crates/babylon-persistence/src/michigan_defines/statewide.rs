//! Authored commodity quantities; source observations only qualify participation.

use std::collections::BTreeMap;

use babylon_kernel::clock::WEEKS_PER_TICK;
use serde::{Deserialize, Serialize};

use super::{MichiganDefinesError, MAX_EXACT_INTEGER};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) enum DesignedEvidence {
    Designed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) enum CommodityUnit {
    #[serde(rename = "kg")]
    Kilogram,
    #[serde(rename = "item")]
    Item,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommodityDisposition {
    FiniteOpening,
    Traded,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct CommodityDefines {
    pub unit: CommodityUnit,
    pub grams_per_unit: u64,
    pub disposition: CommodityDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct TemplateDefines {
    pub output_good: String,
    pub output_units_per_batch: u64,
    pub input_units_per_batch: BTreeMap<String, u64>,
    pub opening_input_units: BTreeMap<String, u64>,
    pub batches_per_week: u64,
    pub labor_hours_per_batch: u64,
    pub employed_people: u64,
    pub reserve_people: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct StatewideDefines {
    pub evidence_class: DesignedEvidence,
    pub finite_order_periods: u64,
    pub terminal_attachment_limit_meters: u64,
    pub experiment: Option<StatewideExperimentDefines>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct StatewideExperimentDefines {
    pub freight_capacity_key: String,
    pub constrained_grams_per_period: u64,
    pub food_process_key: String,
    pub packaging_good_key: String,
    pub shortage_opening_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct TransportDefines {
    pub evidence_class: DesignedEvidence,
    pub road_travel_periods: u16,
    pub truck_gross_weight_kg: u64,
    pub truck_height_mm: u64,
    pub truck_width_mm: u64,
    pub truck_length_mm: u64,
    pub default_maxheight_mm: u64,
    pub road_capacity_grams_per_period: u64,
    pub extraction_buffer_degrees_e7: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct MerchantDefines {
    pub evidence_class: DesignedEvidence,
    pub handling_grams_per_period: u64,
    pub labor_hours_per_kg: u64,
    pub labor_hours_per_item: u64,
    pub employed_people: u64,
    pub reserve_people: u64,
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.as_bytes()[0].is_ascii_lowercase()
        && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

fn positive(value: u64) -> bool {
    (1..=MAX_EXACT_INTEGER).contains(&value)
}

fn product(left: u64, right: u64) -> bool {
    left.checked_mul(right)
        .is_some_and(|n| n <= MAX_EXACT_INTEGER)
}

fn workforce(employed: u64, reserve: u64, hours_per_period: u64) -> bool {
    employed
        .checked_add(reserve)
        .is_some_and(|people| positive(people) && product(people, hours_per_period))
}

pub(super) fn validate(defines: &super::MichiganDefines) -> Result<(), MichiganDefinesError> {
    use MichiganDefinesError::Value;
    if defines.commodity.is_empty()
        || defines.commodity.len() > 64
        || defines.template.is_empty()
        || defines.template.len() > 32
        || !defines.commodity.keys().all(|name| valid_name(name))
        || !defines.template.keys().all(|name| valid_name(name))
    {
        return Err(Value("bounded commodity and template identities"));
    }
    for good in defines.commodity.values() {
        if !positive(good.grams_per_unit)
            || (good.unit == CommodityUnit::Kilogram && good.grams_per_unit != 1000)
        {
            return Err(Value("positive exact commodity grams per native unit"));
        }
    }
    for (name, template) in &defines.template {
        validate_template(defines, name, template)?;
    }
    if defines.commodity.iter().any(|(id, good)| {
        good.disposition == CommodityDisposition::Traded && !defines.template.contains_key(id)
    }) {
        return Err(Value("unresolved traded commodity template coverage"));
    }
    if !(1..=defines.horizon_periods).contains(&defines.statewide.finite_order_periods)
        || !positive(defines.statewide.terminal_attachment_limit_meters)
    {
        return Err(Value(
            "statewide finite orders or terminal attachment bounds",
        ));
    }
    validate_transport_and_handling(defines)?;
    Ok(())
}

fn validate_template(
    defines: &super::MichiganDefines,
    name: &str,
    template: &TemplateDefines,
) -> Result<(), MichiganDefinesError> {
    use MichiganDefinesError::Value;
    let Some(output) = defines.commodity.get(&template.output_good) else {
        return Err(Value("template output must identify a commodity"));
    };
    if name != template.output_good.as_str() || output.disposition != CommodityDisposition::Traded {
        return Err(Value("one productive template per traded output"));
    }
    let Some(batches) = template.batches_per_week.checked_mul(WEEKS_PER_TICK) else {
        return Err(Value("template batches overflow"));
    };
    if !positive(batches)
        || !positive(template.output_units_per_batch)
        || !positive(template.labor_hours_per_batch)
        || !product(batches, template.labor_hours_per_batch)
        || !product(batches, template.output_units_per_batch)
        || !product(template.output_units_per_batch, output.grams_per_unit)
        || !workforce(
            template.employed_people,
            template.reserve_people,
            defines.hours_per_period(),
        )
    {
        return Err(Value(
            "template capacity, labor, output, or workforce bounds",
        ));
    }
    if template.input_units_per_batch.is_empty()
        || template.input_units_per_batch.len() > 16
        || !template
            .input_units_per_batch
            .keys()
            .eq(template.opening_input_units.keys())
    {
        return Err(Value("each template input needs exactly one opening stock"));
    }
    for (input, coefficient) in &template.input_units_per_batch {
        let Some(good) = defines.commodity.get(input) else {
            return Err(Value("template input must identify a commodity"));
        };
        if input == name
            || !positive(*coefficient)
            || !product(batches, *coefficient)
            || !product(*coefficient, good.grams_per_unit)
            || !product(template.opening_input_units[input], good.grams_per_unit)
        {
            return Err(Value("template physical input or stock bounds"));
        }
    }
    Ok(())
}

fn validate_transport_and_handling(
    defines: &super::MichiganDefines,
) -> Result<(), MichiganDefinesError> {
    use MichiganDefinesError::Value;
    let transport = &defines.transport;
    if let Some(experiment) = &defines.statewide.experiment {
        if [
            &experiment.freight_capacity_key,
            &experiment.food_process_key,
        ]
        .iter()
        .any(|key| {
            key.is_empty()
                || key.len() > 128
                || !key.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_')
                })
        }) || experiment.packaging_good_key != "paper_packaging"
            || !positive(experiment.constrained_grams_per_period)
            || experiment.constrained_grams_per_period >= transport.road_capacity_grams_per_period
            || defines
                .template
                .get("prepared_food")
                .and_then(|template| template.opening_input_units.get("paper_packaging"))
                .is_none_or(|opening| experiment.shortage_opening_units >= *opening)
        {
            return Err(Value(
                "statewide interventions require a capacity reduction and a packaging shortage",
            ));
        }
    }
    if transport.road_travel_periods != 1
        || ![
            transport.truck_gross_weight_kg,
            transport.truck_height_mm,
            transport.truck_width_mm,
            transport.truck_length_mm,
            transport.default_maxheight_mm,
            transport.road_capacity_grams_per_period,
            transport.extraction_buffer_degrees_e7,
        ]
        .into_iter()
        .all(positive)
    {
        return Err(Value("Designed road journey and transport profile bounds"));
    }
    let merchant = &defines.merchant;
    if !positive(merchant.handling_grams_per_period)
        || !positive(merchant.labor_hours_per_kg)
        || !positive(merchant.labor_hours_per_item)
        || !workforce(
            merchant.employed_people,
            merchant.reserve_people,
            defines.hours_per_period(),
        )
        || !defines.commodity.values().all(|good| {
            let units = merchant.handling_grams_per_period / good.grams_per_unit;
            let hours = match good.unit {
                CommodityUnit::Kilogram => merchant.labor_hours_per_kg,
                CommodityUnit::Item => merchant.labor_hours_per_item,
            };
            product(units, hours)
        })
    {
        return Err(Value("merchant handling and labor bounds"));
    }

    Ok(())
}
