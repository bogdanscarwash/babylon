use babylon_tick::{hex, run_once};
use serde::Deserialize;

const SCENARIO: &str =
    include_str!("../../../../contracts/fixtures/detroit_windsor_rtd_v1_admin_world.bscn");
const RULE: &str =
    include_str!("../../../../contracts/fixtures/detroit_windsor_rtd_v1_admin_noop.bsl");
const IDENTITY_JSON: &str =
    include_str!("../../../../contracts/fixtures/detroit_windsor_rtd_v1_world_identity.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldIdentity {
    verified_tick: u64,
    graph_state_hash: String,
    nominal_world_hash: String,
    scenario_digest: String,
    rule_digest: String,
    definitions_digest: String,
    template_digest: String,
}

#[test]
fn administrative_fixture_recomputes_real_tick_identity() {
    let identity: WorldIdentity = serde_json::from_str(IDENTITY_JSON).expect("closed identity");
    let report = run_once(SCENARIO, RULE).expect("administrative control tick");
    assert_eq!(identity.verified_tick, 1);
    assert_eq!(report.fired, 0);
    assert_eq!(hex(&report.after), identity.graph_state_hash);
    assert_eq!(hex(&report.world_after), identity.nominal_world_hash);
    assert_eq!(
        hex(&babylon_kernel::sha256_of(SCENARIO.as_bytes())),
        identity.scenario_digest
    );
    assert_eq!(
        hex(&babylon_kernel::sha256_of(RULE.as_bytes())),
        identity.rule_digest
    );
    assert_eq!(identity.definitions_digest.len(), 64);
    assert_eq!(identity.template_digest.len(), 64);
}

#[test]
fn scenario_state_mutation_moves_raw_and_tick_identities() {
    let identity: WorldIdentity = serde_json::from_str(IDENTITY_JSON).expect("closed identity");
    let mutated = SCENARIO.replacen("26099", "26098", 1);
    let report = run_once(&mutated, RULE).expect("mutated administrative control tick");
    assert_ne!(
        hex(&babylon_kernel::sha256_of(mutated.as_bytes())),
        identity.scenario_digest
    );
    assert_ne!(hex(&report.after), identity.graph_state_hash);
    assert_ne!(hex(&report.world_after), identity.nominal_world_hash);
}

#[test]
fn raw_rule_mutation_invalidates_rule_digest_without_overclaiming_world_binding() {
    let identity: WorldIdentity = serde_json::from_str(IDENTITY_JSON).expect("closed identity");
    let mutated = RULE.replacen("99999", "99998", 1);
    let report = run_once(SCENARIO, &mutated).expect("still-false mutated rule");
    assert_eq!(report.fired, 0);
    assert_ne!(
        hex(&babylon_kernel::sha256_of(mutated.as_bytes())),
        identity.rule_digest
    );
    assert_eq!(hex(&report.after), identity.graph_state_hash);
    assert_eq!(hex(&report.world_after), identity.nominal_world_hash);
}
