//! Required, bounded numeric parameters for a newly created Michigan campaign.
//! Canonical values, not TOML whitespace or a mutable file path, enter identity.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use babylon_kernel::clock::{DAYS_PER_TICK, WEEKS_PER_TICK};
use serde::{Deserialize, Serialize};

pub const MAX_MICHIGAN_DEFINES_BYTES: usize = 32_768;
mod statewide;
pub(crate) use statewide::{
    CommodityDefines, CommodityDisposition, CommodityUnit, MerchantDefines, StatewideDefines,
    TemplateDefines, TransportDefines,
};

const MAX_EXACT_INTEGER: u64 = 1 << 53;

#[derive(Debug)]
pub enum MichiganDefinesError {
    Read(std::io::Error),
    TooLarge,
    Utf8(std::string::FromUtf8Error),
    Toml(toml::de::Error),
    Canonical,
    Value(&'static str),
    Material(super::michigan_material::MichiganMaterialError),
}
impl std::fmt::Display for MichiganDefinesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(error) => write!(f, "defines file read failed: {error}"),
            Self::TooLarge => write!(f, "defines exceeds {MAX_MICHIGAN_DEFINES_BYTES} bytes"),
            Self::Utf8(error) => write!(f, "defines is not UTF-8: {error}"),
            Self::Toml(error) => write!(f, "defines TOML refused: {error}"),
            Self::Canonical => f.write_str("stored defines are not canonical validated values"),
            Self::Value(field) => write!(f, "defines value or unit constraint refused: {field}"),
            Self::Material(error) => write!(f, "defines material composition refused: {error}"),
        }
    }
}
impl std::error::Error for MichiganDefinesError {}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct StaffingDefines {
    pub work_hours_per_person_week: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct ProcessDefines {
    pub batches_per_week: u64,
    pub labor_hours_per_batch: u64,
    pub input_units_per_batch: u64,
    pub output_units_per_batch: u64,
    pub opening_input_units: u64,
    pub opening_planned_batches: u64,
    pub employed_people: u64,
    pub reserve_people: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct CorridorDefines {
    pub units_per_week: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct RouteDefines {
    pub travel_periods: u16,
    pub delayed_travel_periods: u16,
    pub ordered_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct SharedFreightDefines {
    pub ample_units_per_week: u64,
    pub constrained_units_per_week: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct RegionalMassDefines {
    pub evidence_class: statewide::DesignedEvidence,
    pub kilogram_grams_per_unit: u64,
    pub panel_grams_per_unit: u64,
    pub subassembly_grams_per_unit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct MichiganDefines {
    #[serde(rename = "regional_mass")]
    pub regional_mass: RegionalMassDefines,
    pub schema_version: u16,
    pub tick_duration_days: u64,
    pub horizon_periods: u64,
    #[serde(rename = "staffing")]
    pub staffing: StaffingDefines,
    #[serde(rename = "process")]
    pub process: BTreeMap<String, ProcessDefines>,
    #[serde(rename = "corridor")]
    pub corridor: BTreeMap<String, CorridorDefines>,
    #[serde(rename = "route")]
    pub route: BTreeMap<String, RouteDefines>,
    #[serde(rename = "shared_freight")]
    pub shared_freight: SharedFreightDefines,
    #[serde(rename = "statewide")]
    pub statewide: StatewideDefines,
    #[serde(rename = "transport")]
    pub transport: TransportDefines,
    #[serde(rename = "merchant")]
    pub merchant: MerchantDefines,
    #[serde(rename = "commodity")]
    pub commodity: BTreeMap<String, CommodityDefines>,
    #[serde(rename = "template")]
    pub template: BTreeMap<String, TemplateDefines>,
}
impl MichiganDefines {
    pub fn load(path: &Path) -> Result<Self, MichiganDefinesError> {
        let file = std::fs::File::open(path).map_err(MichiganDefinesError::Read)?;
        let mut bytes = Vec::new();
        file.take((MAX_MICHIGAN_DEFINES_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(MichiganDefinesError::Read)?;
        if bytes.len() > MAX_MICHIGAN_DEFINES_BYTES {
            return Err(MichiganDefinesError::TooLarge);
        }
        Self::parse(&String::from_utf8(bytes).map_err(MichiganDefinesError::Utf8)?)
    }
    pub fn parse(text: &str) -> Result<Self, MichiganDefinesError> {
        if text.len() > MAX_MICHIGAN_DEFINES_BYTES {
            return Err(MichiganDefinesError::TooLarge);
        }
        let value: Self = toml::from_str(text).map_err(MichiganDefinesError::Toml)?;
        value.validate()?;
        Ok(value)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, MichiganDefinesError> {
        if bytes.len() > MAX_MICHIGAN_DEFINES_BYTES {
            return Err(MichiganDefinesError::TooLarge);
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| MichiganDefinesError::Canonical)?;
        value.validate()?;
        if value.encode()? != bytes {
            return Err(MichiganDefinesError::Canonical);
        }
        Ok(value)
    }
    pub fn encode(&self) -> Result<Vec<u8>, MichiganDefinesError> {
        serde_json::to_vec(self).map_err(|_| MichiganDefinesError::Canonical)
    }
    pub fn hours_per_period(&self) -> u64 {
        // validate proves the multiplication and physical weekly bound.
        self.staffing.work_hours_per_person_week * WEEKS_PER_TICK
    }
    fn validate(&self) -> Result<(), MichiganDefinesError> {
        use MichiganDefinesError::Value;
        if self.schema_version != 3 {
            return Err(Value("SCHEMA_VERSION must equal 3"));
        }
        if self.tick_duration_days != DAYS_PER_TICK {
            return Err(Value(
                "TICK_DURATION_DAYS must equal the supported 28-day period",
            ));
        }
        if !(1..=super::michigan_material::MICHIGAN_MAX_HORIZON_PERIODS)
            .contains(&self.horizon_periods)
        {
            return Err(Value("HORIZON_PERIODS must be 1..=16"));
        }
        if !(1..=168).contains(&self.staffing.work_hours_per_person_week) {
            return Err(Value("WORK_HOURS_PER_PERSON_WEEK must be 1..=168"));
        }
        let process_keys = [
            "meal_milling",
            "meal_packaging",
            "panel_forming",
            "sheet_rolling",
            "subassembly_making",
        ];
        let corridor_keys = ["food_transfer", "panel_transfer", "sheet_transfer"];
        if !self.process.keys().map(String::as_str).eq(process_keys) {
            return Err(Value(
                "process tables must name exactly the five known processes",
            ));
        }
        if !self.corridor.keys().map(String::as_str).eq(corridor_keys) {
            return Err(Value(
                "corridor tables must name exactly the three known transfers",
            ));
        }
        for value in self.process.values() {
            validate_process(value, self.hours_per_period())?;
        }
        if !self.route.keys().map(String::as_str).eq(corridor_keys) {
            return Err(Value(
                "route tables must name exactly the three known transfers",
            ));
        }
        for value in self.route.values() {
            if value.ordered_units == 0
                || value.travel_periods == 0
                || value.delayed_travel_periods < value.travel_periods
            {
                return Err(Value(
                    "route orders and travel must be positive; delayed travel cannot be shorter",
                ));
            }
        }
        for rate in self.corridor.values().map(|v| v.units_per_week).chain([
            self.shared_freight.ample_units_per_week,
            self.shared_freight.constrained_units_per_week,
        ]) {
            if rate == 0 || rate.checked_mul(WEEKS_PER_TICK).is_none() {
                return Err(Value(
                    "corridor capacities must be positive and fit period units",
                ));
            }
        }
        if self.shared_freight.constrained_units_per_week > self.shared_freight.ample_units_per_week
        {
            return Err(Value(
                "constrained shared freight capacity cannot exceed ample capacity",
            ));
        }
        if self.regional_mass.kilogram_grams_per_unit != 1000
            || !(1..=MAX_EXACT_INTEGER).contains(&self.regional_mass.panel_grams_per_unit)
            || !(1..=MAX_EXACT_INTEGER).contains(&self.regional_mass.subassembly_grams_per_unit)
        {
            return Err(Value(
                "regional mass requires exact kilogram conversion and positive item masses",
            ));
        }
        statewide::validate(self)
    }
}

fn validate_process(
    value: &ProcessDefines,
    hours_per_period: u64,
) -> Result<(), MichiganDefinesError> {
    use MichiganDefinesError::Value;
    if value.batches_per_week == 0
        || value.labor_hours_per_batch == 0
        || value.input_units_per_batch == 0
        || value.output_units_per_batch == 0
        || value.employed_people == 0
    {
        return Err(Value(
            "process throughput, recipe, labor, and employed-person quantities must be positive",
        ));
    }
    let capacity = value
        .batches_per_week
        .checked_mul(WEEKS_PER_TICK)
        .ok_or(Value("BATCHES_PER_WEEK overflows period capacity"))?;
    // A future request is computed from input-feasible batches, before
    // current staffing limits production. Every request enters graph Real.
    if capacity
        .checked_mul(value.labor_hours_per_batch)
        .is_none_or(|hours| hours > MAX_EXACT_INTEGER)
    {
        return Err(Value(
            "maximal period staffing request exceeds the exact integer bound",
        ));
    }
    let people = value
        .employed_people
        .checked_add(value.reserve_people)
        .ok_or(Value("workforce overflows"))?;
    let budget = value
        .employed_people
        .checked_mul(hours_per_period)
        .ok_or(Value("employed labor-hours overflow"))?;
    if people > MAX_EXACT_INTEGER
        || people
            .checked_mul(hours_per_period)
            .is_none_or(|hours| hours > MAX_EXACT_INTEGER)
    {
        return Err(Value(
            "workforce or labor-hours exceed the exact integer observation bound",
        ));
    }
    if value.opening_planned_batches > capacity
        || value
            .opening_planned_batches
            .checked_mul(value.input_units_per_batch)
            .is_none_or(|q| q > value.opening_input_units)
        || value
            .opening_planned_batches
            .checked_mul(value.labor_hours_per_batch)
            .is_none_or(|hours| hours > budget)
        || capacity.checked_mul(value.output_units_per_batch).is_none()
    {
        return Err(Value(
            "opening plan exceeds physical input, labor, throughput, or integer bounds",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../content/scenarios/michigan/defines.toml"
    ));
    #[test]
    fn equivalent_toml_has_one_canonical_identity_and_stored_values_round_trip() {
        let original = MichiganDefines::parse(SOURCE).unwrap();
        let reformatted = MichiganDefines::parse(&format!("# author note\n\n{SOURCE}\n")).unwrap();
        assert_eq!(original.encode().unwrap(), reformatted.encode().unwrap());
        assert_eq!(
            MichiganDefines::decode(&original.encode().unwrap()).unwrap(),
            original
        );
        let mut padded = original.encode().unwrap();
        padded.push(b' ');
        assert!(matches!(
            MichiganDefines::decode(&padded),
            Err(MichiganDefinesError::Canonical)
        ));
    }
    #[test]
    fn missing_unknown_fractional_and_invalid_units_are_refused() {
        for changed in [
            SOURCE.replace("SCHEMA_VERSION = 3", "UNUSED_COEFFICIENT = 1"),
            SOURCE.replace("SCHEMA_VERSION = 3", "SCHEMA_VERSION = 1"),
            SOURCE.replace("[route.sheet_transfer]", "[route.unknown_transfer]"),
            SOURCE.replace(
                "CONSTRAINED_UNITS_PER_WEEK = 40",
                "CONSTRAINED_UNITS_PER_WEEK = 0",
            ),
            SOURCE.replace(
                "CONSTRAINED_UNITS_PER_WEEK = 40",
                "CONSTRAINED_UNITS_PER_WEEK = 201",
            ),
            SOURCE.replace(
                "AMPLE_UNITS_PER_WEEK = 200",
                "AMPLE_UNITS_PER_WEEK = 9223372036854775807",
            ),
            SOURCE.replace(
                "WORK_HOURS_PER_PERSON_WEEK = 40",
                "WORK_HOURS_PER_PERSON_WEEK = 40.5",
            ),
            SOURCE.replace(
                "WORK_HOURS_PER_PERSON_WEEK = 40",
                "WORK_HOURS_PER_PERSON_WEEK = 169",
            ),
            SOURCE.replace("TICK_DURATION_DAYS = 28", "TICK_DURATION_DAYS = 7"),
            SOURCE.replace("HORIZON_PERIODS = 16", "HORIZON_PERIODS = 17"),
            SOURCE.replace("INPUT_UNITS_PER_BATCH = 10", "INPUT_UNITS_PER_BATCH = 0"),
            SOURCE.replace(
                "OPENING_PLANNED_BATCHES = 32",
                "OPENING_PLANNED_BATCHES = 33",
            ),
            SOURCE.replace("[process.sheet_rolling]", "[process.retired_engine]"),
            format!("{SOURCE}\nUNKNOWN = 1\n"),
        ] {
            assert!(
                MichiganDefines::parse(&changed).is_err(),
                "unexpectedly admitted {changed}"
            );
        }
        assert!(matches!(
            MichiganDefines::parse(&" ".repeat(MAX_MICHIGAN_DEFINES_BYTES + 1)),
            Err(MichiganDefinesError::TooLarge)
        ));
    }
    #[test]
    fn future_staffing_demand_must_fit_graph_integers_even_with_no_opening_plan() {
        for coefficient in ["9007199254740993", "9223372036854775807"] {
            let changed = SOURCE
                .replace(
                    "OPENING_PLANNED_BATCHES = 32",
                    "OPENING_PLANNED_BATCHES = 0",
                )
                .replace(
                    "LABOR_HOURS_PER_BATCH = 100",
                    &format!("LABOR_HOURS_PER_BATCH = {coefficient}"),
                );
            assert!(matches!(
                MichiganDefines::parse(&changed),
                Err(MichiganDefinesError::Value(
                    "maximal period staffing request exceeds the exact integer bound"
                ))
            ));
        }
    }

    #[test]
    fn statewide_physical_coefficients_and_source_classification_are_validated() {
        let baseline = MichiganDefines::parse(SOURCE).unwrap();
        assert_eq!(baseline.template.len(), 16);
        assert_eq!(baseline.commodity.len(), 23);
        assert_eq!(baseline.transport.road_travel_periods, 1);
        for changed in [
            SOURCE.replace("GRAMS_PER_UNIT = 1000", "GRAMS_PER_UNIT = 999"),
            SOURCE.replace("GRAMS_PER_UNIT = 50000", "GRAMS_PER_UNIT = 0"),
            SOURCE.replace(
                "EVIDENCE_CLASS = \"Designed\"",
                "EVIDENCE_CLASS = \"Observed\"",
            ),
            SOURCE.replace("ROAD_TRAVEL_PERIODS = 1", "ROAD_TRAVEL_PERIODS = 2"),
            SOURCE.replace("FINITE_ORDER_PERIODS = 4", "FINITE_ORDER_PERIODS = 17"),
            SOURCE.replace("OUTPUT_GOOD = \"machinery\"", "OUTPUT_GOOD = \"unknown\""),
            SOURCE.replace("crop_feedstock = 100", "crop_feedstock = 0"),
            SOURCE.replace("crop_feedstock = 25600", "unknown = 25600"),
            SOURCE.replace(
                "LABOR_HOURS_PER_ITEM = 10",
                "LABOR_HOURS_PER_ITEM = 9007199254740992",
            ),
        ] {
            assert!(
                MichiganDefines::parse(&changed).is_err(),
                "accepted invalid statewide input"
            );
        }
        let no_workers = SOURCE.replace("EMPLOYED_PEOPLE = 8", "EMPLOYED_PEOPLE = 0");
        // A reserve-only producer remains physically admitted so staffing can recover.
        assert!(MichiganDefines::parse(&no_workers).is_ok());
    }
}
