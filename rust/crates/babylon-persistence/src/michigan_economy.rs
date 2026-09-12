//! Exact observed Michigan county economics and the shared observer foundation.

use std::fmt::Write as _;
use std::io::Read as _;
use std::sync::OnceLock;

use babylon_bsl::{canonical_ast::rules_hash_of, rule_pipeline::split_content};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::{
    clock::DAYS_PER_TICK,
    content_digest::sha256_of,
    content_digest::ContentDigest,
    replay::{ReplaySeed, ReplaySessionId},
    tick_content_hash::RefDigest,
};
use babylon_tick::{material_state::MaterialState, replay_session::ReplayTickSession};
use serde::{Deserialize, Serialize};

use crate::{michigan_dynamic_hex_foundation, FoundationContentBundle};

/// Immutable scenario identity shared by the runtime and window.
pub const MICHIGAN_OBSERVER_SCENARIO: &str = "production/michigan-observer-v1";
/// Exact public-record source vintage; these values are not simulated flows.
pub const QCEW_ECONOMICS_VINTAGE: u16 = 2024;
/// Source identifier shared by baseline and Archive citations.
pub const QCEW_ECONOMICS_SOURCE_ID: &str = "qcew-county-economics-v1";
/// Exact artifact identity from `QcewCountyEconomicsV1`.
pub const QCEW_ECONOMICS_ARTIFACT_SHA256: &str =
    "116affb2998c6c0259d5bf14840f99f835d7e0733aa0b4f4c60a257b2723cd16";
/// Exact observed field keys. Wages are USD whole units, not engine money.
pub const QCEW_ECONOMICS_FIELD_KEYS: [&str; 4] = [
    "qcew-establishments",
    "qcew-employment",
    "qcew-total-annual-wages",
    "qcew-average-weekly-wage",
];
const ARTIFACT: &[u8] = include_bytes!(
    "../../../../src/babylon/data/reference/economy/qcew_county_economics_mi_2024.csv.gz"
);
const HEADER: &str = "county_geoid,annual_avg_estabs_count,annual_avg_emplvl,total_annual_wages,annual_avg_wkly_wage";
const MAX_DECODED_BYTES: u64 = 33_554_432;
const REFERENCE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";

/// One unrounded, unsuppressed, exact row from the pinned public artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MichiganCountyEconomy {
    pub county_geoid: String,
    pub annual_avg_estabs_count: u64,
    pub annual_avg_emplvl: u64,
    pub total_annual_wages: u64,
    pub annual_avg_wkly_wage: u64,
}

/// Checked immutable source and deterministic scenario text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganEconomy {
    counties: Vec<MichiganCountyEconomy>,
    scenario_source: String,
}
impl MichiganEconomy {
    #[must_use]
    pub fn counties(&self) -> &[MichiganCountyEconomy] {
        &self.counties
    }
    #[must_use]
    pub fn scenario_source(&self) -> &str {
        &self.scenario_source
    }
}

/// Closed construction failures, without input or credential disclosure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganEconomyError {
    ArtifactDigest,
    ArtifactDecode,
    ArtifactShape,
    ArtifactValue,
    Scenario,
    Foundation,
    Reference,
}
impl std::fmt::Display for MichiganEconomyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan economics refused: {self:?}")
    }
}
impl std::error::Error for MichiganEconomyError {}

pub(crate) fn digest_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("String write");
    }
    output
}

fn parse_csv(source: &str) -> Result<MichiganEconomy, MichiganEconomyError> {
    let mut lines = source.lines();
    if lines.next() != Some(HEADER) || !source.ends_with('\n') {
        return Err(MichiganEconomyError::ArtifactShape);
    }
    let mut counties = Vec::with_capacity(83);
    for (position, line) in lines.enumerate() {
        if position >= 83 {
            return Err(MichiganEconomyError::ArtifactShape);
        }
        let columns: Vec<&str> = line.split(',').collect();
        if columns.len() != 5 || columns[0] != format!("{}", 26_001 + position * 2) {
            return Err(MichiganEconomyError::ArtifactShape);
        }
        let mut values = [0_u64; 4];
        for (index, text) in columns[1..].iter().enumerate() {
            if text.is_empty()
                || !text.bytes().all(|byte| byte.is_ascii_digit())
                || (text.len() > 1 && text.starts_with('0'))
            {
                return Err(MichiganEconomyError::ArtifactValue);
            }
            let value = text
                .parse::<u64>()
                .map_err(|_| MichiganEconomyError::ArtifactValue)?;
            // BSL's int principal is signed 64-bit; refuse any lossy bridge.
            i64::try_from(value).map_err(|_| MichiganEconomyError::ArtifactValue)?;
            if value > 9_007_199_254_740_992 {
                return Err(MichiganEconomyError::ArtifactValue);
            }
            values[index] = value;
        }
        counties.push(MichiganCountyEconomy {
            county_geoid: columns[0].to_owned(),
            annual_avg_estabs_count: values[0],
            annual_avg_emplvl: values[1],
            total_annual_wages: values[2],
            annual_avg_wkly_wage: values[3],
        });
    }
    if counties.len() != 83 {
        return Err(MichiganEconomyError::ArtifactShape);
    }
    let mut scenario_source = format!("(scenario {MICHIGAN_OBSERVER_SCENARIO}\n  (defvocabulary NodeType (TERRITORY))\n  (deffield territory/county-fips int extensive)\n");
    append_county_observations(&mut scenario_source, &counties);
    scenario_source.push_str(")\n");
    Ok(MichiganEconomy {
        counties,
        scenario_source,
    })
}

/// Append the same observed county declarations and rows to a versioned scenario.
pub(crate) fn append_county_observations(
    scenario_source: &mut String,
    counties: &[MichiganCountyEconomy],
) {
    for (index, key) in QCEW_ECONOMICS_FIELD_KEYS.iter().enumerate() {
        writeln!(
            scenario_source,
            "  (deffield territory/{key} int {})",
            if index == 3 { "intensive" } else { "extensive" }
        )
        .expect("String write");
    }
    for county in counties {
        writeln!(
            scenario_source,
            "  (node county-{} NodeType/TERRITORY\n    (territory/county-fips {})",
            county.county_geoid, county.county_geoid
        )
        .expect("String write");
        let values = [
            county.annual_avg_estabs_count,
            county.annual_avg_emplvl,
            county.total_annual_wages,
            county.annual_avg_wkly_wage,
        ];
        for (key, value) in QCEW_ECONOMICS_FIELD_KEYS.iter().zip(values) {
            writeln!(scenario_source, "    (territory/{key} {value})").expect("String write");
        }
        scenario_source.push_str("  )\n");
    }
}

/// Decode the bounded artifact once and verify its exact governed digest.
/// # Errors
/// Refuses altered gzip bytes, excess expansion, malformed CSV or invalid values.
pub fn michigan_economy() -> Result<&'static MichiganEconomy, MichiganEconomyError> {
    static ECONOMY: OnceLock<Result<MichiganEconomy, MichiganEconomyError>> = OnceLock::new();
    ECONOMY
        .get_or_init(|| {
            if digest_hex(&sha256_of(ARTIFACT)) != QCEW_ECONOMICS_ARTIFACT_SHA256 {
                return Err(MichiganEconomyError::ArtifactDigest);
            }
            let mut decoded = String::new();
            flate2::read::GzDecoder::new(ARTIFACT)
                .take(MAX_DECODED_BYTES + 1)
                .read_to_string(&mut decoded)
                .map_err(|_| MichiganEconomyError::ArtifactDecode)?;
            if u64::try_from(decoded.len()).map_err(|_| MichiganEconomyError::ArtifactDecode)?
                > MAX_DECODED_BYTES
            {
                return Err(MichiganEconomyError::ArtifactDecode);
            }
            parse_csv(&decoded)
        })
        .as_ref()
        .map_err(|error| *error)
}

/// Build the sole Michigan observer foundation. The baseline has no economy rules yet.
/// # Errors
/// Refuses invalid reference identity, content or tick-zero construction.
pub fn michigan_observer_foundation(
) -> Result<(ReplayTickSession<HypergraphStore>, FoundationContentBundle), MichiganEconomyError> {
    build_observer_foundation(michigan_economy()?)
}

// The public entry point admits only the digest-pinned artifact above. Keeping
// preparation private lets tests qualify source changes without a runtime bypass.
fn build_observer_foundation(
    economy: &MichiganEconomy,
) -> Result<(ReplayTickSession<HypergraphStore>, FoundationContentBundle), MichiganEconomyError> {
    let defines = format!(
        "{{\"qcew_vintage\":{QCEW_ECONOMICS_VINTAGE},\"tick_duration_days\":{DAYS_PER_TICK}}}"
    );
    observer_foundation_from_source(
        economy.scenario_source(),
        "g4/michigan-observer-v1",
        defines.as_bytes(),
        FoundationContentBundle::try_new,
    )
}

/// Shared construction; each pinned factory explicitly selects its content codec.
pub(crate) fn observer_foundation_from_source<B>(
    scenario_source: &str,
    session_identity: &str,
    defines: &[u8],
    encode_bundle: impl FnOnce(
        &str,
        Option<&str>,
        &str,
        &[u8],
        &[u8],
    ) -> Result<B, crate::RustPersistenceRuntimeError>,
) -> Result<(ReplayTickSession<HypergraphStore>, B), MichiganEconomyError> {
    let (_, rules) = split_content("").map_err(|_| MichiganEconomyError::Scenario)?;
    let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    let content = ContentDigest {
        defines_hash: sha256_of(defines),
        rules_hash: rules_hash_of(&forms).map_err(|_| MichiganEconomyError::Scenario)?,
    };
    let foundation =
        michigan_dynamic_hex_foundation().map_err(|_| MichiganEconomyError::Reference)?;
    let mut manifest = REFERENCE_DOMAIN.to_vec();
    manifest.extend_from_slice(&foundation.base_reference_cohort_digest());
    manifest.extend_from_slice(&foundation.r8_section_digest());
    if sha256_of(&manifest) != foundation.reference_bundle_digest() {
        return Err(MichiganEconomyError::Reference);
    }
    // Scenario bytes carry every exact economics input, so the foundation digest
    // covers the QCEW rows while the existing H3 reference identity stays intact.
    let session = ReplayTickSession::new(
        scenario_source,
        None,
        "",
        HypergraphStore::new(),
        ReplaySessionId::try_from(session_identity).map_err(|_| MichiganEconomyError::Scenario)?,
        ReplaySeed::new(319),
        content,
        RefDigest::from_bytes(foundation.reference_bundle_digest()),
        MaterialState::try_new(foundation).map_err(|_| MichiganEconomyError::Foundation)?,
    )
    .map_err(|_| MichiganEconomyError::Foundation)?;
    let bundle = encode_bundle(scenario_source, None, "", defines, &manifest)
        .map_err(|_| MichiganEconomyError::Foundation)?;
    Ok((session, bundle))
}

/// Exact complete deterministic foundation identity shared by every observer session.
/// # Errors
/// Refuses any failed tick-zero construction or canonical foundation capture.
pub fn michigan_observer_foundation_digest() -> Result<[u8; 32], MichiganEconomyError> {
    static DIGEST: OnceLock<Result<[u8; 32], MichiganEconomyError>> = OnceLock::new();
    *DIGEST.get_or_init(|| {
        let (session, bundle) = michigan_observer_foundation()?;
        let captured = crate::CampaignFoundation::capture(&session, bundle)
            .map_err(|_| MichiganEconomyError::Foundation)?;
        Ok(sha256_of(captured.canonical_bytes()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use babylon_bsl::{identity_codec::StableBslValue, structural_verbs::CollectingSink};
    use babylon_graph::stable_element::StableElementKey;
    use babylon_practice_contract::OrderedPracticeActionBatch;
    use babylon_tick::replay_session::IdentifiedTickReport;

    use crate::{
        county_committed_signals, CampaignFoundation, CommittedTerritoryFields, CountySignal,
    };

    // Synthetic test inputs, not new observations: the Python source-builder
    // test regenerates these exact CSV bytes from 83 public-schema source files.
    const GENERATED_BASELINE: &str =
        include_str!("../../../../tests/fixtures/qcew_economics/baseline.csv");
    const GENERATED_CHANGED: &str =
        include_str!("../../../../tests/fixtures/qcew_economics/changed.csv");

    #[test]
    fn diagnostic_interval_changes_foundation_identity_without_rescaling_source_facts() {
        let economy = michigan_economy().unwrap();
        let (session, bundle) = build_observer_foundation(economy).unwrap();
        let defines: serde_json::Value = serde_json::from_slice(bundle.defines_bytes()).unwrap();
        assert_eq!(defines["qcew_vintage"], 2024);
        assert_eq!(defines["tick_duration_days"], 28);
        let current = CampaignFoundation::capture(&session, bundle).unwrap();
        assert_eq!(
            sha256_of(current.canonical_bytes()),
            michigan_observer_foundation_digest().unwrap()
        );

        for incompatible in [
            br#"{"qcew_vintage":2024}"#.as_slice(),
            br#"{"qcew_vintage":2024,"tick_duration_days":7}"#.as_slice(),
            br#"{"qcew_vintage":2024,"tick_duration_days":29}"#.as_slice(),
        ] {
            let (old_session, old_bundle) = observer_foundation_from_source(
                economy.scenario_source(),
                "g4/michigan-observer-v1",
                incompatible,
                FoundationContentBundle::try_new,
            )
            .unwrap();
            let old = CampaignFoundation::capture(&old_session, old_bundle).unwrap();
            assert_ne!(
                sha256_of(old.canonical_bytes()),
                sha256_of(current.canonical_bytes()),
                "missing or incompatible time units must not share admission identity"
            );
            assert_eq!(
                old.stable_graph_bytes(),
                current.stable_graph_bytes(),
                "annual observations and weekly wage source values must remain unchanged"
            );
        }
    }

    fn advance_observation(
        session: &mut ReplayTickSession<HypergraphStore>,
    ) -> IdentifiedTickReport {
        let tick = u64::try_from(session.completed_tick() + 1).unwrap();
        let actions =
            OrderedPracticeActionBatch::empty(session.session_identity().clone(), tick).unwrap();
        let mut sink = CollectingSink::default();
        let report = session.advance(&mut sink, &actions).unwrap();
        assert_eq!(report.report().considered, 0);
        assert_eq!(report.report().fired, 0);
        assert!(report.report().audit_receipts.is_empty());
        assert!(sink.events.is_empty());
        report
    }

    fn dossier_signals(report: &IdentifiedTickReport) -> BTreeMap<String, Vec<CountySignal>> {
        let mut result = BTreeMap::new();
        for row in report.material_state_rows().territories().rows() {
            let StableElementKey::Node {
                scenario,
                local_name,
            } = row.territory_id()
            else {
                panic!("a county must retain its stable node identity");
            };
            assert_eq!(scenario, MICHIGAN_OBSERVER_SCENARIO);
            assert_eq!(
                row.ordered_fields().len(),
                5,
                "county FIPS and four exact observed fields only"
            );
            let fields: BTreeMap<_, _> = row
                .ordered_fields()
                .iter()
                .map(|(key, value)| (key.as_str(), value))
                .collect();
            let values = QCEW_ECONOMICS_FIELD_KEYS.map(|key| {
                let StableBslValue::Int(value) = fields[key] else {
                    panic!("QCEW {key} must survive the actual territory projector as an integer");
                };
                Some(*value)
            });
            let input = CommittedTerritoryFields::try_from_qcew(values).unwrap();
            let signals = county_committed_signals(&input).unwrap();
            assert_eq!(signals.len(), 4);
            assert!(result.insert(local_name.clone(), signals).is_none());
        }
        assert_eq!(result.len(), 83);
        result
    }

    #[test]
    fn generated_qcew_source_changes_only_its_county_through_the_real_foundation_and_projection() {
        let baseline = parse_csv(GENERATED_BASELINE).unwrap();
        let changed = parse_csv(GENERATED_CHANGED).unwrap();
        let (mut original, original_bundle) = build_observer_foundation(&baseline).unwrap();
        let (mut revised, revised_bundle) = build_observer_foundation(&changed).unwrap();
        assert!(original_bundle.rule_source_bytes().is_empty());
        assert!(revised_bundle.rule_source_bytes().is_empty());
        let original_foundation = CampaignFoundation::capture(&original, original_bundle).unwrap();
        let revised_foundation = CampaignFoundation::capture(&revised, revised_bundle).unwrap();
        assert_ne!(
            original_foundation.canonical_bytes(),
            revised_foundation.canonical_bytes()
        );
        assert_eq!(
            original.material_state(),
            revised.material_state(),
            "observed payroll cannot author physical output"
        );

        let original_report = advance_observation(&mut original);
        let revised_report = advance_observation(&mut revised);
        let before = dossier_signals(&original_report);
        let after = dossier_signals(&revised_report);
        let changed_counties: Vec<_> = before
            .keys()
            .filter(|county| before[*county] != after[*county])
            .map(String::as_str)
            .collect();
        assert_eq!(changed_counties, ["county-26163"]);
        let wayne = &after["county-26163"];
        let expected = [
            (
                "qcew-establishments",
                "QCEW 2024 annual-average establishments",
                "40001",
            ),
            (
                "qcew-employment",
                "QCEW 2024 annual-average employment (jobs)",
                "812345",
            ),
            (
                "qcew-total-annual-wages",
                "QCEW 2024 total annual wages (USD)",
                "99999999999",
            ),
            (
                "qcew-average-weekly-wage",
                "QCEW 2024 average weekly wage (USD/week)",
                "2049",
            ),
        ];
        for (signal, (key, label, value)) in wayne.iter().zip(expected) {
            assert_eq!(
                (signal.grant_key(), signal.label(), signal.value()),
                (key, label, value)
            );
        }
        assert_eq!(original.material_state(), revised.material_state());
        assert_ne!(
            original_report.tick_content_hash(),
            revised_report.tick_content_hash()
        );
    }

    #[test]
    fn generated_county_observations_resume_with_identical_projection_and_tick_identity() {
        let economy = parse_csv(GENERATED_CHANGED).unwrap();
        let (mut continued, first_bundle) = build_observer_foundation(&economy).unwrap();
        let (mut reopened, second_bundle) = build_observer_foundation(&economy).unwrap();
        assert_eq!(
            CampaignFoundation::capture(&continued, first_bundle)
                .unwrap()
                .canonical_bytes(),
            CampaignFoundation::capture(&reopened, second_bundle)
                .unwrap()
                .canonical_bytes()
        );
        let first = advance_observation(&mut continued);
        reopened
            .restore_full_checkpoint(
                1,
                first.result_stable_graph(),
                first.material_state_rows(),
                first.result_registers().canonical_bytes(),
            )
            .unwrap();
        let next = advance_observation(&mut continued);
        let resumed = advance_observation(&mut reopened);
        assert_eq!(next.tick_content_hash(), resumed.tick_content_hash());
        assert_eq!(
            next.result_stable_graph().canonical_bytes(),
            resumed.result_stable_graph().canonical_bytes()
        );
        assert_eq!(
            next.material_state_rows().canonical_bytes(),
            resumed.material_state_rows().canonical_bytes()
        );
        assert_eq!(dossier_signals(&next), dossier_signals(&resumed));
    }

    #[test]
    fn malformed_county_rows_refuse_instead_of_becoming_zero() {
        assert_eq!(
            parse_csv(&format!("{HEADER}\n26001,1,NaN,3,4\n")),
            Err(MichiganEconomyError::ArtifactValue)
        );
        assert_eq!(
            parse_csv(&format!("{HEADER}\n26003,1,2,3,4\n")),
            Err(MichiganEconomyError::ArtifactShape)
        );
        assert_eq!(
            parse_csv(&format!("{HEADER}\n26001,1,2,9223372036854775808,4\n")),
            Err(MichiganEconomyError::ArtifactValue)
        );
    }
}
