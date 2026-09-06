//! Sole production command for Rust-owned `PostgreSQL` persistence.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[cfg(test)]
use babylon_bsl::rule_pipeline::split_content;
#[cfg(test)]
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_state::StableGraphStateV1;
#[cfg(test)]
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
#[cfg(test)]
use babylon_kernel::tick_content_hash::RefDigestV1;
#[cfg(test)]
use babylon_kernel::ContentDigest;
#[cfg(test)]
use babylon_persistence::michigan_dynamic_hex_foundation_v1;
use babylon_persistence::{
    activate_rust_persistence_v2, preflight_schema_epoch, representative_h3_reference_cohort_v1,
    ArchiveSchemaDispositionV1, CampaignFoundationV1, CampaignId, CommittedResolveTickV1,
    CommittedTickReceiptV2, CompositeArchiveDossierProducerV1, CountyDossierProducerV1,
    DurableReplayRuntimeV2, FoundationContentBundleV1, PlaceDossierProducerV1,
    PostgresDiagnosticV1, RustPersistenceRuntimeErrorV2, SemanticArchiveStoreV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::choice_receipt::ChoiceReceiptV1;
#[cfg(test)]
use babylon_tick::material_state::MaterialStateV1;
use babylon_tick::replay_session::{ReplayCommitDispositionV1, ReplayTickSession};
use postgres::{Config, NoTls};
use uuid::Uuid;

const DSN_ENV: &str = "BABYLON_RUNTIME_DSN";
const CAMPAIGN_ENV: &str = "BABYLON_CAMPAIGN_ID";
const DEFAULT_CAMPAIGN_UUID: u128 = 0x2810_0000_0000_0000_0000_0000_0000_0001;
const MICHIGAN_SMOKE_TICKS: u64 = 60;
const MICHIGAN_SMOKE_RESTART_TICKS: &[u64] = &[1, 51, 52, 60];
const TICK_REPORT_SCHEMA_V2: &str = "babylon.simulation.tick-report.v2";
const CHOICE_RECEIPT_REPORT_SCHEMA_V1: &str = "babylon.simulation.choice-receipts.v1";
const TICK_REPORT_SLICE_ID: &str = "michigan-persistence-slice";
#[cfg(test)]
const FIXED_REPLAY_SEED: i64 = 281;
#[cfg(test)]
const OBSERVED_ENTITY: &str = "wayne";
#[cfg(test)]
const OBSERVABLE_ALLOWLIST_V2: &[(&str, &str)] = &[
    ("territory/median-wage", "configured_input"),
    ("territory/phi-hour", "configured_input"),
    ("territory/phi-savings-adjustment", "dynamic"),
    ("territory/rate-accumulation", "dynamic"),
    ("territory/dist-year", "dynamic"),
];
#[cfg(test)]
const DEFINES: &[u8] = br#"{"alpha":1}"#;
#[cfg(test)]
const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
#[cfg(test)]
const SCENARIO: &str = r"
(scenario production/michigan-rust-runtime
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/median-wage real intensive)
  (deffield territory/phi-hour real intensive)
  (deffield territory/phi-savings-adjustment coefficient intensive)
  (deffield territory/rate-accumulation probability intensive)
  (deffield territory/dist-year int extensive)
  ; Governed geography identity (PER-22 ruling D1): write-prohibited to every
  ; rule role at load (GOVERNED_WRITE_PROHIBITED_NODE_FIELDS); county identity
  ; is frozen at campaign foundation.
  (deffield territory/county-fips int extensive)
  (defconst class-dynamics/hours-per-year 2080)
  (defconst class-dynamics/v-reproduction 12)
  (defconst class-dynamics/accumulation-halt-floor-ratio 0.8c)
  (defconst class-dynamics/phi-cap 0.05c)
  (defconst class-dynamics/savings-proletariat 0.03c)
  (defconst class-dynamics/wealth-threshold 142000)
  (defconst class-dynamics/max-accumulation-rate 0.08c)
  (node wayne NodeType/TERRITORY
    (territory/median-wage 21.0r)
    (territory/phi-hour 1.0r)
    (territory/phi-savings-adjustment 0.0c)
    (territory/rate-accumulation 0.0p)
    (territory/dist-year 2010)
    (territory/county-fips 26163)))
";
#[cfg(test)]
const RULE: &str = r#"
(rule class-dynamics/a01-rollover-accumulation-smoke
  :role mechanic
  :evidence derived
  :material-basis "At the annual boundary, wage and imperial-rent-supported savings change class accumulation"
  :fuel 128
  (bindings
    (binding phase-of-year :tick-in-cycle 52)
    (binding median-wage :field territory/median-wage)
    (binding phi-adjustment :field territory/phi-savings-adjustment)
    (binding hours-per-year :const class-dynamics/hours-per-year)
    (binding v-reproduction :const class-dynamics/v-reproduction)
    (binding halt-floor-ratio :const class-dynamics/accumulation-halt-floor-ratio)
    (binding savings-proletariat :const class-dynamics/savings-proletariat)
    (binding wealth-threshold :const class-dynamics/wealth-threshold)
    (binding max-rate :const class-dynamics/max-accumulation-rate)
    (binding wage-floor :expr (* (- v-reproduction 0c) halt-floor-ratio))
    (binding raw-annual-wage :expr (* median-wage hours-per-year))
    (binding effective-wage :expr (if (< median-wage wage-floor)
                                      (- 0 0c)
                                      raw-annual-wage))
    (binding savings-raw :expr (+ savings-proletariat phi-adjustment))
    (binding savings :expr (if (< savings-raw 1) savings-raw (- 1 0c)))
    (binding annual-accumulation :expr (* effective-wage savings))
    (binding accumulation-ratio :expr (if (> annual-accumulation 0)
                                          (/ annual-accumulation wealth-threshold)
                                          (- 0 0c)))
    (binding rate :expr (if (< accumulation-ratio max-rate)
                            accumulation-ratio
                            max-rate)))
  (when (= phase-of-year 0))
  (effects
    (update-node self territory/rate-accumulation (set rate))
    (update-node self territory/dist-year (add 1))
    (emit EventType/MICHIGAN_YEAR_ROLLOVER
      (subject self)
      (phi-adjustment phi-adjustment)
      (accumulation-rate rate))))

(rule economics/phi-savings-coupling-smoke
  :role mechanic
  :evidence derived
  :material-basis "Imperial rent purchases class entry by increasing the savings rate against annual wage"
  :fuel 128
  (bindings
    (binding median-wage :field territory/median-wage)
    (binding phi-hour :field territory/phi-hour)
    (binding hours-per-year :const class-dynamics/hours-per-year)
    (binding v-reproduction :const class-dynamics/v-reproduction)
    (binding halt-floor-ratio :const class-dynamics/accumulation-halt-floor-ratio)
    (binding phi-cap :const class-dynamics/phi-cap)
    (binding wage-floor :expr (* (- v-reproduction 0c) halt-floor-ratio))
    (binding raw-annual-wage :expr (* median-wage hours-per-year))
    (binding effective-wage :expr (if (< median-wage wage-floor)
                                      (- 0 0c)
                                      raw-annual-wage))
    (binding uncapped-adjustment :expr (if (or (= effective-wage 0) (= phi-hour 0))
                                          (- 0 0c)
                                          (/ (* phi-hour hours-per-year) effective-wage)))
    (binding phi-adjustment :expr (if (< uncapped-adjustment phi-cap)
                                      uncapped-adjustment
                                      phi-cap)))
  (when #t)
  (effects
    (update-node self territory/phi-savings-adjustment (set phi-adjustment))))
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Activate,
    Bootstrap,
    Preflight,
    Run {
        ticks: u64,
        report_jsonl: Option<PathBuf>,
        choice_receipts_jsonl: Option<PathBuf>,
        restart_every: Option<u64>,
    },
    Probe,
    Session {
        preset: Option<babylon_persistence::michigan_material::MichiganDeliveryPresetV1>,
    },
    ObserverSchema,
    Archive,
    ArchiveWorker,
    MichiganSmoke {
        report_jsonl: Option<PathBuf>,
        choice_receipts_jsonl: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let Ok(command) = parse_command(std::env::args_os().skip(1)) else {
        eprintln!(
            "babylon-runtime: expected activate, bootstrap, preflight, diagnostic run --ticks N [--report-jsonl PATH] [--choice-receipts-jsonl PATH] [--restart-every N], probe, archive, archive-worker --once, diagnostic michigan-smoke [--report-jsonl PATH] [--choice-receipts-jsonl PATH], observer-schema, or session --stdio [--preset standard|delayed]"
        );
        return ExitCode::from(2);
    };
    let Some(raw_dsn) = std::env::var_os(DSN_ENV) else {
        eprintln!("babylon-runtime: {DSN_ENV} is required");
        return ExitCode::from(2);
    };
    let Ok(dsn) = raw_dsn.into_string() else {
        eprintln!("babylon-runtime: {DSN_ENV} must be valid UTF-8");
        return ExitCode::from(2);
    };
    let Ok(config) = dsn.parse::<Config>() else {
        eprintln!("babylon-runtime: {DSN_ENV} is not a valid PostgreSQL DSN");
        return ExitCode::from(2);
    };
    if let Err(error) = representative_h3_reference_cohort_v1() {
        eprintln!("babylon-runtime: embedded H3 reference fixture is invalid: {error}");
        return ExitCode::FAILURE;
    }

    match execute(command, &config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("babylon-runtime: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(command: Command, config: &Config) -> Result<(), String> {
    match command {
        Command::Preflight => {
            preflight_schema_epoch(config).map_err(|error| error.to_string())?;
            println!("Rust schema target and owner preflight complete.");
        }
        Command::Activate | Command::Bootstrap => {
            let report = activate_rust_persistence_v2(config).map_err(|error| error.to_string())?;
            println!(
                "Rust persistence authority active (prepared_epoch={}, active_epoch={}).",
                report.prepared_row().activation_epoch(),
                report.active_row().activation_epoch(),
            );
        }
        Command::Run {
            ticks,
            report_jsonl,
            choice_receipts_jsonl,
            restart_every,
        } => {
            let mut report_writer = report_jsonl
                .as_deref()
                .map(TickReportJsonlWriter::create)
                .transpose()?;
            let mut choice_receipt_writer = choice_receipts_jsonl
                .as_deref()
                .map(ChoiceReceiptJsonlWriter::create)
                .transpose()?;
            activate_rust_persistence_v2(config).map_err(|error| error.to_string())?;
            run_to_tick(
                config,
                campaign_id()?,
                ticks,
                &[],
                restart_every,
                report_writer.as_mut(),
                choice_receipt_writer.as_mut(),
            )?;
        }
        Command::MichiganSmoke {
            report_jsonl,
            choice_receipts_jsonl,
        } => {
            let mut report_writer = report_jsonl
                .as_deref()
                .map(TickReportJsonlWriter::create)
                .transpose()?;
            let mut choice_receipt_writer = choice_receipts_jsonl
                .as_deref()
                .map(ChoiceReceiptJsonlWriter::create)
                .transpose()?;
            activate_rust_persistence_v2(config).map_err(|error| error.to_string())?;
            run_to_tick(
                config,
                campaign_id()?,
                MICHIGAN_SMOKE_TICKS,
                MICHIGAN_SMOKE_RESTART_TICKS,
                None,
                report_writer.as_mut(),
                choice_receipt_writer.as_mut(),
            )?;
        }
        Command::Session { preset } => {
            babylon_persistence::run_runtime_session_stdio_v2(config, campaign_id()?, preset)
                .map_err(|error| error.to_string())?;
        }
        Command::ObserverSchema => {
            SemanticArchiveStoreV1::new(config)
                .install_schema()
                .map_err(|error| error.to_string())?;
            babylon_persistence::install_reader_role_v1(config)
                .map_err(|error| error.to_string())?;
            babylon_persistence::install_observer_economy_schema_v1(config)
                .map_err(|error| error.to_string())?;
        }
        Command::Probe => probe(config, configured_campaign_id()?)?,
        Command::Archive => inspect_archive(config)?,
        Command::ArchiveWorker => run_archive_worker_once(config)?,
    }
    Ok(())
}

fn campaign_id() -> Result<CampaignId, String> {
    Ok(configured_campaign_id()?
        .unwrap_or_else(|| CampaignId::from_uuid(Uuid::from_u128(DEFAULT_CAMPAIGN_UUID))))
}

fn configured_campaign_id() -> Result<Option<CampaignId>, String> {
    let uuid = match std::env::var(CAMPAIGN_ENV) {
        Ok(value) => Some(
            Uuid::parse_str(&value)
                .map_err(|_| format!("{CAMPAIGN_ENV} must be a canonical UUID"))?,
        ),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(format!("{CAMPAIGN_ENV} must be valid UTF-8"));
        }
    };
    Ok(uuid.map(CampaignId::from_uuid))
}

fn run_to_tick(
    config: &Config,
    campaign: CampaignId,
    target_tick: u64,
    restart_ticks: &[u64],
    restart_every: Option<u64>,
    mut report_writer: Option<&mut TickReportJsonlWriter>,
    mut choice_receipt_writer: Option<&mut ChoiceReceiptJsonlWriter>,
) -> Result<(), String> {
    let mut runtime = open_or_create_runtime(config, campaign)?;
    let mut completed = runtime
        .last_committed_tick()
        .map_or(0, babylon_persistence::CommittedResolveTickV1::get);
    if completed > target_tick {
        return Err(format!(
            "campaign tail {completed} is beyond requested target {target_tick}"
        ));
    }
    while completed < target_tick {
        let reporting = report_writer.is_some();
        let before = reporting
            .then(|| runtime.observe_current_stable_graph_state_v1())
            .transpose()
            .map_err(|error| error.to_string())?;
        let resolve_tick = completed
            .checked_add(1)
            .ok_or_else(|| "requested tick overflow".to_owned())?;
        let actions = OrderedPracticeActionBatchV1::empty(
            runtime.foundation().replay_session_identity().clone(),
            resolve_tick,
        )
        .map_err(|_| "empty action batch refused".to_owned())?;
        let mut sink = CollectingSink::default();
        let receipt = runtime
            .advance_and_commit(&mut sink, &actions)
            .map_err(|error| error.to_string())?;
        completed = receipt.resolve_tick().get();
        if let Some(writer) = choice_receipt_writer.as_deref_mut() {
            let choices = runtime
                .observe_committed_choice_receipts_v1(&receipt)
                .map_err(|error| error.to_string())?;
            writer.write_receipt(&receipt, choices)?;
        }
        let reopened_after_commit = should_reopen_after_commit_v2(
            completed,
            target_tick,
            restart_ticks,
            restart_every,
            reporting,
        );
        if reopened_after_commit {
            runtime = DurableReplayRuntimeV2::open(config, campaign)
                .map_err(|error| error.to_string())?;
            if runtime
                .last_committed_tick()
                .map(CommittedResolveTickV1::get)
                != Some(completed)
            {
                return Err("restart did not recover the acknowledged tail".to_owned());
            }
        }
        if let Some(writer) = report_writer.as_deref_mut() {
            let after = runtime
                .observe_committed_graph_state_v1(&receipt)
                .map_err(|error| error.to_string())?;
            writer.write_receipt(
                &receipt,
                before
                    .as_ref()
                    .ok_or_else(|| "tick report pre-state is absent".to_owned())?,
                &after,
                &sink,
                reopened_after_commit,
                foundation_identity_v2(runtime.foundation()),
            )?;
        }
        println!(
            "Committed Rust tick {} (content_sha256={}).",
            completed,
            hex_digest(receipt.tick_content_hash().as_bytes()),
        );
    }
    println!("Rust durable campaign is complete at tick {completed}.");
    Ok(())
}

fn should_reopen_after_commit_v2(
    completed: u64,
    target_tick: u64,
    restart_ticks: &[u64],
    restart_every: Option<u64>,
    reporting: bool,
) -> bool {
    restart_ticks.contains(&completed)
        || restart_every.is_some_and(|interval| completed.is_multiple_of(interval))
        || (reporting && completed == target_tick)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuleTickReportV2 {
    rule_id: String,
    considered: usize,
    fired: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventTypeTickReportV2 {
    event_type: String,
    count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservableTickReportV2 {
    name: String,
    entity: String,
    field: String,
    role: &'static str,
    before_value_bits: u64,
    after_value_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FoundationIdentityTickReportV2 {
    foundation: [u8; 32],
    defines: [u8; 32],
    rules: [u8; 32],
    reference: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimulationTickReportV2 {
    scenario: String,
    resolve_tick: u64,
    commit_disposition: &'static str,
    graph_before: [u8; 32],
    graph_after: [u8; 32],
    stable_graph_before: [u8; 32],
    stable_graph_after: [u8; 32],
    world_before: [u8; 32],
    world_after: [u8; 32],
    considered: usize,
    fired: usize,
    per_rule: Vec<RuleTickReportV2>,
    event_count: usize,
    event_digest: [u8; 32],
    choice_receipt_count: usize,
    choice_receipt_digest: [u8; 32],
    event_per_type: Vec<EventTypeTickReportV2>,
    observables: Vec<ObservableTickReportV2>,
    persistence_reopened_after_commit: bool,
    foundation: FoundationIdentityTickReportV2,
    audit_receipt_count: usize,
    material_row_count: usize,
    material_row_digest: [u8; 32],
    tick_content_hash: [u8; 32],
}

impl SimulationTickReportV2 {
    fn try_from_receipt(
        receipt: &CommittedTickReceiptV2,
        before: &StableGraphStateV1,
        after: &StableGraphStateV1,
        sink: &CollectingSink,
        reopened_after_commit: bool,
        foundation: FoundationIdentityTickReportV2,
    ) -> Result<Self, String> {
        if before.digest().as_bytes() != &receipt.prior_stable_graph_digest() {
            return Err("tick report pre-state is not bound to its receipt".to_owned());
        }
        if after.digest().as_bytes() != &receipt.result_stable_graph_digest() {
            return Err("tick report post-state is not bound to its receipt".to_owned());
        }
        if before.scenario_scope() != after.scenario_scope() {
            return Err("tick report stable-state scenarios are misaligned".to_owned());
        }
        let considered = receipt.per_rule_considered();
        let fired = receipt.per_rule_fired();
        if considered.len() != fired.len() {
            return Err("acknowledged tick report rule counts are misaligned".to_owned());
        }
        let mut per_rule = Vec::with_capacity(considered.len());
        for ((considered_id, considered_count), (fired_id, fired_count)) in
            considered.iter().zip(fired)
        {
            if considered_id != fired_id {
                return Err("acknowledged tick report rule identities are misaligned".to_owned());
            }
            per_rule.push(RuleTickReportV2 {
                rule_id: considered_id.clone(),
                considered: *considered_count,
                fired: *fired_count,
            });
        }
        let commit_disposition = match receipt.commit_disposition() {
            ReplayCommitDispositionV1::Committed => "committed",
            ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit => {
                "reconciled_after_ambiguous_commit"
            }
        };
        Ok(Self {
            scenario: after.scenario_scope().to_owned(),
            resolve_tick: receipt.resolve_tick().get(),
            commit_disposition,
            graph_before: receipt.graph_before(),
            graph_after: receipt.graph_after(),
            stable_graph_before: receipt.prior_stable_graph_digest(),
            stable_graph_after: receipt.result_stable_graph_digest(),
            world_before: receipt.world_before(),
            world_after: receipt.world_after(),
            considered: receipt.considered(),
            fired: receipt.fired(),
            per_rule,
            event_count: receipt.event_count(),
            event_digest: receipt.event_digest(),
            choice_receipt_count: receipt.choice_receipt_count(),
            choice_receipt_digest: receipt.choice_receipt_digest(),
            event_per_type: collect_event_type_counts_v2(sink, receipt.event_count())?,
            observables: collect_observable_transitions_v2(before, after)?,
            persistence_reopened_after_commit: reopened_after_commit,
            foundation,
            audit_receipt_count: receipt.audit_receipt_count(),
            material_row_count: receipt.material_row_count(),
            material_row_digest: receipt.material_row_digest(),
            tick_content_hash: *receipt.tick_content_hash().as_bytes(),
        })
    }

    fn json_value(&self) -> Result<serde_json::Value, String> {
        let per_rule = self
            .per_rule
            .iter()
            .map(|rule| {
                serde_json::json!({
                    "rule_id": rule.rule_id.as_str(),
                    "considered": rule.considered,
                    "fired": rule.fired,
                })
            })
            .collect::<Vec<_>>();
        let event_per_type = self
            .event_per_type
            .iter()
            .map(|event| {
                serde_json::json!({
                    "event_type": event.event_type.as_str(),
                    "count": event.count,
                })
            })
            .collect::<Vec<_>>();
        let observables = observable_json_values_v2(&self.observables)?;
        Ok(serde_json::json!({
            "schema": TICK_REPORT_SCHEMA_V2,
            "resolve_tick": self.resolve_tick,
            "commit_disposition": self.commit_disposition,
            "scope": {
                "slice_id": TICK_REPORT_SLICE_ID,
                "scenario": self.scenario.as_str(),
                "fixed_replay_seed": if self.scenario == babylon_persistence::MICHIGAN_OBSERVER_SCENARIO_V1 { 319 } else { 281 },
                "parameter_overrides": false,
                "stochastic_draws": false,
                "dynamic_h3_updates": false,
            },
            "graph": {
                "before_sha256": hex_digest(&self.graph_before),
                "after_sha256": hex_digest(&self.graph_after),
            },
            "stable_graph": {
                "before_sha256": hex_digest(&self.stable_graph_before),
                "after_sha256": hex_digest(&self.stable_graph_after),
            },
            "world": {
                "before_sha256": hex_digest(&self.world_before),
                "after_sha256": hex_digest(&self.world_after),
            },
            "rules": {
                "considered": self.considered,
                "fired": self.fired,
                "per_rule": per_rule,
            },
            "events": {
                "count": self.event_count,
                "digest_sha256": hex_digest(&self.event_digest),
                "per_type": event_per_type,
            },
            "choice_receipts": {
                "count": self.choice_receipt_count,
                "digest_sha256": hex_digest(&self.choice_receipt_digest),
            },
            "observables": observables,
            "persistence": {
                "reopened_after_commit": self.persistence_reopened_after_commit,
            },
            "foundation": {
                "foundation_sha256": hex_digest(&self.foundation.foundation),
                "defines_sha256": hex_digest(&self.foundation.defines),
                "rules_sha256": hex_digest(&self.foundation.rules),
                "reference_sha256": hex_digest(&self.foundation.reference),
            },
            "audit_receipts": {
                "count": self.audit_receipt_count,
            },
            "material_rows": {
                "count": self.material_row_count,
                "digest_sha256": hex_digest(&self.material_row_digest),
            },
            "tick_content_hash": hex_digest(&self.tick_content_hash),
        }))
    }
}

fn observable_json_values_v2(
    observables: &[ObservableTickReportV2],
) -> Result<Vec<serde_json::Value>, String> {
    observables
        .iter()
        .map(|observable| {
            let before_value =
                serde_json::Number::from_f64(f64::from_bits(observable.before_value_bits))
                    .ok_or_else(|| {
                        format!(
                            "tick report observable {} pre-value is not finite",
                            observable.name
                        )
                    })?;
            let after_value =
                serde_json::Number::from_f64(f64::from_bits(observable.after_value_bits))
                    .ok_or_else(|| {
                        format!(
                            "tick report observable {} post-value is not finite",
                            observable.name
                        )
                    })?;
            Ok(serde_json::json!({
                "name": observable.name.as_str(),
                "entity": observable.entity.as_str(),
                "field": observable.field.as_str(),
                "role": observable.role,
                "kind": "f64",
                "before_value": before_value,
                "before_bits_hex": format!("{:016x}", observable.before_value_bits),
                "after_value": after_value,
                "after_bits_hex": format!("{:016x}", observable.after_value_bits),
            }))
        })
        .collect()
}

fn collect_event_type_counts_v2(
    sink: &CollectingSink,
    expected_count: usize,
) -> Result<Vec<EventTypeTickReportV2>, String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for (event_type, _) in &sink.events {
        let count = counts.entry(event_type.clone()).or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| "tick report event-type count overflow".to_owned())?;
    }
    let actual_count = counts.values().try_fold(0_usize, |total, count| {
        total
            .checked_add(*count)
            .ok_or_else(|| "tick report event count overflow".to_owned())
    })?;
    if actual_count != expected_count {
        return Err(format!(
            "acknowledged event count {expected_count} differs from published event count {actual_count}"
        ));
    }
    Ok(counts
        .into_iter()
        .map(|(event_type, count)| EventTypeTickReportV2 { event_type, count })
        .collect())
}

fn collect_observable_transitions_v2(
    before: &StableGraphStateV1,
    after: &StableGraphStateV1,
) -> Result<Vec<ObservableTickReportV2>, String> {
    if before.scenario_scope() != after.scenario_scope() {
        return Err("tick report observable scenarios are misaligned".to_owned());
    }
    if after.scenario_scope() == babylon_persistence::MICHIGAN_OBSERVER_SCENARIO_V1 {
        let mut observables = Vec::with_capacity(83 * 4);
        for county in babylon_persistence::michigan_economy_v1()
            .map_err(|error| error.to_string())?
            .counties()
        {
            let entity = format!("county-{}", county.county_geoid);
            for key in babylon_persistence::QCEW_ECONOMICS_FIELD_KEYS_V1 {
                let field = format!("territory/{key}");
                let find = |state: &StableGraphStateV1| -> Result<u64, String> {
                    state
                        .rows()
                        .node_f64()
                        .iter()
                        .find(|(candidate, candidate_field, _)| {
                            candidate == &entity && candidate_field == &field
                        })
                        .map(|(_, _, bits)| *bits)
                        .ok_or_else(|| {
                            format!("committed QCEW observable {entity}::{field} missing")
                        })
                };
                observables.push(ObservableTickReportV2 {
                    name: format!("{}::{entity}::{field}", after.scenario_scope()),
                    entity: entity.clone(),
                    field: field.clone(),
                    role: "observed_baseline",
                    before_value_bits: find(before)?,
                    after_value_bits: find(after)?,
                });
            }
        }
        return Ok(observables);
    }
    #[cfg(not(test))]
    return Err("tick report scenario is not the Michigan observer foundation".to_owned());
    #[cfg(test)]
    {
        let mut observables = Vec::with_capacity(OBSERVABLE_ALLOWLIST_V2.len());
        for &(field, role) in OBSERVABLE_ALLOWLIST_V2 {
            let (before_entity, before_value_bits) = observable_bits_v2(before, field, "pre")?;
            let (after_entity, after_value_bits) = observable_bits_v2(after, field, "post")?;
            if before_entity != after_entity {
                return Err(format!(
                    "tick report observable {OBSERVED_ENTITY}::{field} identities are misaligned"
                ));
            }
            observables.push(ObservableTickReportV2 {
                name: format!("{}::{OBSERVED_ENTITY}::{field}", after.scenario_scope()),
                entity: after_entity.to_owned(),
                field: field.to_owned(),
                role,
                before_value_bits,
                after_value_bits,
            });
        }
        Ok(observables)
    }
}

#[cfg(test)]
fn observable_bits_v2<'a>(
    state: &'a StableGraphStateV1,
    field: &str,
    boundary: &str,
) -> Result<(&'a str, u64), String> {
    let mut matches = state
        .rows()
        .node_f64()
        .iter()
        .filter(|(entity, candidate_field, _)| {
            entity == OBSERVED_ENTITY && candidate_field == field
        });
    let Some((entity, _, value_bits)) = matches.next() else {
        return Err(format!(
            "tick report {boundary}-observable {OBSERVED_ENTITY}::{field} is missing"
        ));
    };
    if matches.next().is_some() {
        return Err(format!(
            "tick report {boundary}-observable {OBSERVED_ENTITY}::{field} is duplicated"
        ));
    }
    if !f64::from_bits(*value_bits).is_finite() {
        return Err(format!(
            "tick report {boundary}-observable {OBSERVED_ENTITY}::{field} is not finite"
        ));
    }
    Ok((entity.as_str(), *value_bits))
}

fn foundation_identity_v2(foundation: &CampaignFoundationV1) -> FoundationIdentityTickReportV2 {
    FoundationIdentityTickReportV2 {
        foundation: sha256_of(foundation.canonical_bytes()),
        defines: foundation.content_digest().defines_hash,
        rules: foundation.content_digest().rules_hash,
        reference: *foundation.reference_digest().as_bytes(),
    }
}

struct TickReportJsonlWriter {
    output: BufWriter<File>,
}

impl TickReportJsonlWriter {
    fn create(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|_| "tick report path must be new and writable".to_owned())?;
        Ok(Self {
            output: BufWriter::new(file),
        })
    }

    fn write_receipt(
        &mut self,
        receipt: &CommittedTickReceiptV2,
        before: &StableGraphStateV1,
        after: &StableGraphStateV1,
        sink: &CollectingSink,
        reopened_after_commit: bool,
        foundation: FoundationIdentityTickReportV2,
    ) -> Result<(), String> {
        self.write_report(&SimulationTickReportV2::try_from_receipt(
            receipt,
            before,
            after,
            sink,
            reopened_after_commit,
            foundation,
        )?)
    }

    fn write_report(&mut self, report: &SimulationTickReportV2) -> Result<(), String> {
        let value = report.json_value()?;
        serde_json::to_writer(&mut self.output, &value).map_err(|_| {
            format!(
                "tick report JSON serialization failed after durable tick {}",
                report.resolve_tick
            )
        })?;
        self.output
            .write_all(b"\n")
            .and_then(|()| self.output.flush())
            .map_err(|_| {
                format!(
                    "tick report JSONL write failed after durable tick {}",
                    report.resolve_tick
                )
            })
    }
}

/// Optional, non-authoritative detail log for exact realized choices.
///
/// The runtime invokes this writer only after `advance_and_commit` has returned
/// an acknowledged receipt. The file is create-new, flushed per durable tick,
/// and deliberately excludes database coordinates, replay-session bytes, and
/// replay-seed bytes.
struct ChoiceReceiptJsonlWriter {
    output: BufWriter<File>,
}

impl ChoiceReceiptJsonlWriter {
    fn create(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|_| "choice receipt report path must be new and writable".to_owned())?;
        Ok(Self {
            output: BufWriter::new(file),
        })
    }

    fn write_receipt(
        &mut self,
        receipt: &CommittedTickReceiptV2,
        choices: &[ChoiceReceiptV1],
    ) -> Result<(), String> {
        let resolve_tick = receipt.resolve_tick().get();
        if choices.len() != receipt.choice_receipt_count() {
            return Err(format!(
                "choice receipt detail count differs from acknowledged count after durable tick {resolve_tick}"
            ));
        }
        self.write_tick(resolve_tick, receipt.choice_receipt_digest(), choices)
    }

    fn write_tick(
        &mut self,
        resolve_tick: u64,
        choice_receipt_digest: [u8; 32],
        choices: &[ChoiceReceiptV1],
    ) -> Result<(), String> {
        let value = choice_receipt_json_value_v1(resolve_tick, choice_receipt_digest, choices)?;
        serde_json::to_writer(&mut self.output, &value).map_err(|_| {
            format!("choice receipt JSON serialization failed after durable tick {resolve_tick}")
        })?;
        self.output
            .write_all(b"\n")
            .and_then(|()| self.output.flush())
            .map_err(|_| {
                format!("choice receipt JSONL write failed after durable tick {resolve_tick}")
            })
    }
}

fn choice_receipt_json_value_v1(
    resolve_tick: u64,
    choice_receipt_digest: [u8; 32],
    choices: &[ChoiceReceiptV1],
) -> Result<serde_json::Value, String> {
    let mut receipts = Vec::with_capacity(choices.len());
    for choice in choices {
        let stable_carrier = choice.stable_carrier().canonical_bytes().map_err(|_| {
            format!(
                "choice receipt stable carrier serialization failed after durable tick {resolve_tick}"
            )
        })?;
        let active_elements = choice
            .active_elements()
            .iter()
            .map(|element| {
                element.canonical_bytes().map(|bytes| hex_bytes(&bytes)).map_err(|_| {
                    format!(
                        "choice receipt active element serialization failed after durable tick {resolve_tick}"
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let branches = choice
            .branches()
            .iter()
            .enumerate()
            .map(|(position, branch)| {
                serde_json::json!({
                    "position": position,
                    "member": branch.member.as_str(),
                    "mass_nanounits": branch.mass.nanounits().to_string(),
                    "ticket_start": branch.tickets.start.to_string(),
                    "ticket_end": branch.tickets.end.to_string(),
                    "ticket_count": branch.tickets.count.to_string(),
                })
            })
            .collect::<Vec<_>>();
        receipts.push(serde_json::json!({
            "encounter_ordinal": choice.encounter_ordinal(),
            "rule_id": choice.rule_id(),
            "sample": choice.sample(),
            "slot": choice.slot(),
            "outcome_enum": choice.outcome_enum(),
            "stable_carrier_hex": hex_bytes(&stable_carrier),
            "active_elements_hex": active_elements,
            "branches": branches,
            "draw_ticket": choice.draw_ticket().to_string(),
            "selected_outcome": choice.selected_outcome(),
            "allocation_digest_sha256": hex_digest(&choice.allocation_digest()),
            "instance_digest_sha256": hex_digest(&choice.instance_digest()),
        }));
    }
    Ok(serde_json::json!({
        "schema": CHOICE_RECEIPT_REPORT_SCHEMA_V1,
        "authority": "post_commit_operational_observation_only",
        "resolve_tick": resolve_tick,
        "choice_receipt_count": choices.len(),
        "choice_receipt_digest_sha256": hex_digest(&choice_receipt_digest),
        "receipts": receipts,
    }))
}

fn open_or_create_runtime(
    config: &Config,
    campaign: CampaignId,
) -> Result<DurableReplayRuntimeV2<HypergraphStore>, String> {
    match DurableReplayRuntimeV2::open(config, campaign) {
        Ok(runtime) => {
            let economy =
                babylon_persistence::michigan_economy_v1().map_err(|error| error.to_string())?;
            if runtime
                .foundation()
                .content_bundle()
                .scenario_source_bytes()
                != economy.scenario_source().as_bytes()
            {
                return Err("campaign scenario differs from the Michigan observer foundation; choose another campaign identity".to_owned());
            }
            Ok(runtime)
        }
        Err(RustPersistenceRuntimeErrorV2::FoundationAbsent) => {
            let (session, bundle) = runtime_foundation()?;
            DurableReplayRuntimeV2::create(config, campaign, session, bundle)
                .map_err(|error| error.to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

fn runtime_foundation() -> Result<
    (
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ),
    String,
> {
    babylon_persistence::michigan_observer_foundation_v1().map_err(|error| error.to_string())
}

#[cfg(test)]
fn smoke_foundation() -> Result<
    (
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ),
    String,
> {
    let (_, rules) = split_content(RULE).map_err(|_| "runtime rule parse refused".to_owned())?;
    let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    let content = ContentDigest {
        defines_hash: sha256_of(DEFINES),
        rules_hash: rules_hash_of(&forms).map_err(|_| "runtime rule hash refused".to_owned())?,
    };
    let session_id = ReplaySessionIdV1::try_from("per281/rust-runtime")
        .map_err(|_| "runtime replay identity refused".to_owned())?;
    let seed = ReplaySeed::new(FIXED_REPLAY_SEED);
    let foundation = michigan_dynamic_hex_foundation_v1()
        .map_err(|error| format!("Michigan foundation refused: {error}"))?;
    let mut reference_manifest = REFERENCE_BUNDLE_DOMAIN.to_vec();
    reference_manifest.extend_from_slice(&foundation.base_reference_cohort_digest());
    reference_manifest.extend_from_slice(&foundation.r8_section_digest());
    if sha256_of(&reference_manifest) != foundation.reference_bundle_digest() {
        return Err("Michigan reference-bundle digest mismatch".to_owned());
    }
    let reference = RefDigestV1::from_bytes(foundation.reference_bundle_digest());
    let session = ReplayTickSession::new(
        SCENARIO,
        None,
        RULE,
        HypergraphStore::new(),
        session_id,
        seed,
        content,
        reference,
        MaterialStateV1::try_new(foundation)
            .map_err(|_| "Michigan material foundation refused".to_owned())?,
    )
    .map_err(|_| "runtime tick-zero session refused".to_owned())?;
    let bundle =
        FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
            .map_err(|error| error.to_string())?;
    Ok((session, bundle))
}

fn probe(config: &Config, selected_campaign: Option<CampaignId>) -> Result<(), String> {
    let mut client = config
        .connect(NoTls)
        .map_err(|error| postgres_failure("database probe connection", &error))?;
    let authority_row = client
        .query_one(
            "SELECT \
               (SELECT pg_catalog.count(*) \
                FROM babylon_meta.committed_tick_v2_authority_ledger), \
               (SELECT pg_catalog.count(*) \
                FROM babylon_meta.persistence_authority_ledger)",
            &[],
        )
        .map_err(|error| postgres_failure("authority probe", &error))?;
    let v2_authority_rows: i64 = authority_row
        .try_get(0)
        .map_err(|error| postgres_failure("V2 authority probe decode", &error))?;
    let predecessor_authority_rows: i64 = authority_row
        .try_get(1)
        .map_err(|error| postgres_failure("predecessor authority probe decode", &error))?;
    let row = client
        .query_one(
            "SELECT pg_catalog.count(DISTINCT foundation.campaign_id), pg_catalog.max(marker.resolve_tick) \
             FROM babylon_state.campaign_foundation AS foundation \
             LEFT JOIN babylon_state.tick_commit AS marker \
               ON marker.campaign_id = foundation.campaign_id",
            &[],
        )
        .map_err(|error| postgres_failure("campaign-tail probe", &error))?;
    let campaigns: i64 = row
        .try_get(0)
        .map_err(|error| postgres_failure("campaign count decode", &error))?;
    let tail: Option<i64> = row
        .try_get(1)
        .map_err(|error| postgres_failure("campaign tail decode", &error))?;
    let (selected_campaign_state, selected_tail_label) = match selected_campaign {
        Some(campaign) => {
            let selected_tail: Option<i64> = client
                .query_one(
                    "SELECT pg_catalog.max(resolve_tick) \
                     FROM babylon_state.tick_commit \
                     WHERE campaign_id = $1::uuid",
                    &[campaign.as_uuid()],
                )
                .and_then(|row| row.try_get(0))
                .map_err(|error| postgres_failure("selected campaign-tail probe", &error))?;
            (
                "configured",
                selected_tail.map_or_else(|| "none".to_owned(), |value| value.to_string()),
            )
        }
        None => ("unset", "unqueried".to_owned()),
    };
    println!(
        "Rust V2 authority rows={v2_authority_rows}; \
         predecessor epoch-9 authority rows={predecessor_authority_rows}; \
         selected_campaign={selected_campaign_state}; selected_tail={selected_tail_label}; \
         global_durable_campaigns={campaigns}; global_highest_tick={}.",
        tail.map_or_else(|| "none".to_owned(), |value| value.to_string()),
    );
    Ok(())
}

fn run_archive_worker_once(config: &Config) -> Result<(), String> {
    let store = SemanticArchiveStoreV1::new(config);
    store
        .install_schema()
        .map_err(|error| format!("Archive schema refused: {error}"))?;
    let county = CountyDossierProducerV1::try_new(config)
        .map_err(|error| format!("Archive county producer refused: {error}"))?;
    let place = PlaceDossierProducerV1::try_new(config)
        .map_err(|error| format!("Archive place producer refused: {error}"))?;
    let producer = CompositeArchiveDossierProducerV1::new(vec![Box::new(county), Box::new(place)]);
    let mut worker = babylon_persistence::ArchiveWorkerV1::new(config);
    let report = worker
        .sweep_once(campaign_id()?, &producer)
        .map_err(|error| format!("Archive worker sweep refused: {error}"))?;
    println!(
        "Archive worker sweep complete; verified_tick={}; applied={}; \
         already_consumed={}; paged={}.",
        report.verified_tick(),
        report.applied_count(),
        report.already_consumed_count(),
        report.paged_count(),
    );
    Ok(())
}

fn inspect_archive(config: &Config) -> Result<(), String> {
    let schema = SemanticArchiveStoreV1::new(config)
        .install_schema()
        .map_err(|error| format!("Archive schema refused: {error}"))?;
    let mut client = config
        .connect(NoTls)
        .map_err(|error| postgres_failure("Archive probe connection", &error))?;
    let row = client
        .query_one(
            "SELECT pg_catalog.count(*), pg_catalog.min(resolve_tick), pg_catalog.max(resolve_tick) \
             FROM babylon_state.archive_dirty_receipt_v1",
            &[],
        )
        .map_err(|error| postgres_failure("Archive dirty-receipt probe", &error))?;
    let receipts: i64 = row
        .try_get(0)
        .map_err(|error| postgres_failure("Archive receipt count decode", &error))?;
    let first: Option<i64> = row
        .try_get(1)
        .map_err(|error| postgres_failure("Archive first tick decode", &error))?;
    let last: Option<i64> = row
        .try_get(2)
        .map_err(|error| postgres_failure("Archive last tick decode", &error))?;
    let meta = client
        .query_one(
            "SELECT \
               (SELECT pg_catalog.count(*) FROM babylon_meta.archive_knowledge_grant_v1), \
               (SELECT pg_catalog.count(*) FROM babylon_meta.archive_receipt_consumption_v1), \
               (SELECT pg_catalog.count(*) FROM babylon_meta.archive_page_v1)",
            &[],
        )
        .map_err(|error| postgres_failure("semantic Archive probe", &error))?;
    let grants: i64 = meta
        .try_get(0)
        .map_err(|error| postgres_failure("Archive grant count decode", &error))?;
    let consumptions: i64 = meta
        .try_get(1)
        .map_err(|error| postgres_failure("Archive consumption count decode", &error))?;
    let pages: i64 = meta
        .try_get(2)
        .map_err(|error| postgres_failure("Archive page count decode", &error))?;
    let schema = match schema {
        ArchiveSchemaDispositionV1::Installed => "installed",
        ArchiveSchemaDispositionV1::AlreadyCurrent => "current",
    };
    println!(
        "Rust Archive schema={schema}; dirty_receipts={receipts}; tick_range={}..{}; \
         knowledge_grants={grants}; consumed_receipts={consumptions}; pages={pages}.",
        first.map_or_else(|| "none".to_owned(), |value| value.to_string()),
        last.map_or_else(|| "none".to_owned(), |value| value.to_string()),
    );
    Ok(())
}

fn postgres_failure(operation: &'static str, error: &postgres::Error) -> String {
    format!(
        "{operation} failed: {:?}",
        PostgresDiagnosticV1::capture(error)
    )
}

fn hex_digest(bytes: &[u8; 32]) -> String {
    hex_bytes(bytes)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn parse_command(mut args: impl Iterator<Item = OsString>) -> Result<Command, ()> {
    let Some(command) = args.next() else {
        return Err(());
    };
    match command.as_os_str() {
        value if value == OsStr::new("activate") && args.next().is_none() => Ok(Command::Activate),
        value if value == OsStr::new("bootstrap") && args.next().is_none() => {
            Ok(Command::Bootstrap)
        }
        value if value == OsStr::new("preflight") && args.next().is_none() => {
            Ok(Command::Preflight)
        }
        value if value == OsStr::new("session") => {
            if args.next() != Some(OsString::from("--stdio")) {
                return Err(());
            }
            let preset=match args.next() {
                None=>None,
                Some(flag) if flag==OsStr::new("--preset")=>Some(match args.next().as_deref() {
                    Some(value) if value==OsStr::new("standard")=>babylon_persistence::michigan_material::MichiganDeliveryPresetV1::Standard,
                    Some(value) if value==OsStr::new("delayed")=>babylon_persistence::michigan_material::MichiganDeliveryPresetV1::Delayed,
                    _=>return Err(()),
                }),
                _=>return Err(()),
            };
            if args.next().is_some() {
                return Err(());
            }
            Ok(Command::Session { preset })
        }
        value if value == OsStr::new("observer-schema") && args.next().is_none() => {
            Ok(Command::ObserverSchema)
        }
        value if value == OsStr::new("probe") && args.next().is_none() => Ok(Command::Probe),
        value if value == OsStr::new("archive") && args.next().is_none() => Ok(Command::Archive),
        value if value == OsStr::new("archive-worker") => {
            let flag = args.next().ok_or(())?;
            if flag != OsStr::new("--once") || args.next().is_some() {
                return Err(());
            }
            Ok(Command::ArchiveWorker)
        }
        value if value == OsStr::new("michigan-smoke") => {
            let (report_jsonl, choice_receipts_jsonl) = parse_jsonl_options(args)?;
            Ok(Command::MichiganSmoke {
                report_jsonl,
                choice_receipts_jsonl,
            })
        }
        value if value == OsStr::new("run") => {
            let mut ticks = None;
            let mut report_jsonl = None;
            let mut choice_receipts_jsonl = None;
            let mut restart_every = None;
            while let Some(flag) = args.next() {
                if flag == OsStr::new("--ticks") {
                    if ticks.is_some() {
                        return Err(());
                    }
                    let value = args.next().ok_or(())?;
                    ticks = Some(value.to_str().ok_or(())?.parse::<u64>().map_err(|_| ())?);
                } else if flag == OsStr::new("--report-jsonl") {
                    if report_jsonl.is_some() {
                        return Err(());
                    }
                    report_jsonl = Some(parse_report_path(args.next().ok_or(())?)?);
                } else if flag == OsStr::new("--choice-receipts-jsonl") {
                    if choice_receipts_jsonl.is_some() {
                        return Err(());
                    }
                    choice_receipts_jsonl = Some(parse_report_path(args.next().ok_or(())?)?);
                } else if flag == OsStr::new("--restart-every") {
                    if restart_every.is_some() {
                        return Err(());
                    }
                    let value = args.next().ok_or(())?;
                    restart_every = Some(value.to_str().ok_or(())?.parse::<u64>().map_err(|_| ())?);
                } else {
                    return Err(());
                }
            }
            let ticks = ticks.ok_or(())?;
            if ticks == 0 || ticks > i64::MAX as u64 {
                return Err(());
            }
            if restart_every == Some(0) {
                return Err(());
            }
            Ok(Command::Run {
                ticks,
                report_jsonl,
                choice_receipts_jsonl,
                restart_every,
            })
        }
        _ => Err(()),
    }
}

fn parse_jsonl_options(
    mut args: impl Iterator<Item = OsString>,
) -> Result<(Option<PathBuf>, Option<PathBuf>), ()> {
    let mut report_jsonl = None;
    let mut choice_receipts_jsonl = None;
    while let Some(flag) = args.next() {
        if flag == OsStr::new("--report-jsonl") {
            if report_jsonl.is_some() {
                return Err(());
            }
            report_jsonl = Some(parse_report_path(args.next().ok_or(())?)?);
        } else if flag == OsStr::new("--choice-receipts-jsonl") {
            if choice_receipts_jsonl.is_some() {
                return Err(());
            }
            choice_receipts_jsonl = Some(parse_report_path(args.next().ok_or(())?)?);
        } else {
            return Err(());
        }
    }
    Ok((report_jsonl, choice_receipts_jsonl))
}

fn parse_report_path(value: OsString) -> Result<PathBuf, ()> {
    if value.is_empty() {
        Err(())
    } else {
        Ok(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use babylon_bsl::probability::{
        realize_kernel, FiniteKernelV1, KernelBranchV1, KernelInstanceIdentityV1, Mass,
    };
    use babylon_bsl::reader::{Atom, SExpr};
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_bsl::types::EnumTypeId;
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_graph::stable_element::StableElementKeyV1;
    use babylon_graph::substrate::{GraphSubstrate, NodeId};
    use babylon_kernel::SessionId;
    use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
    use babylon_tick::choice_receipt::ChoiceReceiptV1;
    use babylon_tick::TickSession;

    use super::{
        collect_event_type_counts_v2, collect_observable_transitions_v2, foundation_identity_v2,
        parse_command, should_reopen_after_commit_v2, smoke_foundation as runtime_foundation,
        ChoiceReceiptJsonlWriter, Command, EventTypeTickReportV2, FoundationIdentityTickReportV2,
        ObservableTickReportV2, RuleTickReportV2, SimulationTickReportV2, TickReportJsonlWriter,
        MICHIGAN_SMOKE_RESTART_TICKS, MICHIGAN_SMOKE_TICKS, RULE, SCENARIO, TICK_REPORT_SCHEMA_V2,
    };

    static REPORT_PATH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn report_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "babylon-runtime-{label}-{}-{}.jsonl",
            std::process::id(),
            REPORT_PATH_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn report_fixture() -> SimulationTickReportV2 {
        SimulationTickReportV2 {
            scenario: "production/michigan-rust-runtime".to_owned(),
            resolve_tick: 7,
            commit_disposition: "reconciled_after_ambiguous_commit",
            graph_before: [0x11; 32],
            graph_after: [0x22; 32],
            stable_graph_before: [0x23; 32],
            stable_graph_after: [0x24; 32],
            world_before: [0x33; 32],
            world_after: [0x44; 32],
            considered: 5,
            fired: 3,
            per_rule: vec![
                RuleTickReportV2 {
                    rule_id: "vitality/example".to_owned(),
                    considered: 2,
                    fired: 1,
                },
                RuleTickReportV2 {
                    rule_id: "lifecycle/example".to_owned(),
                    considered: 3,
                    fired: 2,
                },
            ],
            event_count: 1,
            event_digest: [0x55; 32],
            choice_receipt_count: 2,
            choice_receipt_digest: [0x56; 32],
            event_per_type: vec![EventTypeTickReportV2 {
                event_type: "EventType/EXAMPLE".to_owned(),
                count: 1,
            }],
            observables: vec![
                ObservableTickReportV2 {
                    name: "production/michigan-rust-runtime::wayne::territory/median-wage"
                        .to_owned(),
                    entity: "wayne".to_owned(),
                    field: "territory/median-wage".to_owned(),
                    role: "configured_input",
                    before_value_bits: 20.0_f64.to_bits(),
                    after_value_bits: 21.0_f64.to_bits(),
                },
                ObservableTickReportV2 {
                    name: "production/michigan-rust-runtime::wayne::territory/phi-hour".to_owned(),
                    entity: "wayne".to_owned(),
                    field: "territory/phi-hour".to_owned(),
                    role: "configured_input",
                    before_value_bits: 1.0_f64.to_bits(),
                    after_value_bits: 1.0_f64.to_bits(),
                },
                ObservableTickReportV2 {
                    name:
                        "production/michigan-rust-runtime::wayne::territory/phi-savings-adjustment"
                            .to_owned(),
                    entity: "wayne".to_owned(),
                    field: "territory/phi-savings-adjustment".to_owned(),
                    role: "dynamic",
                    before_value_bits: 0.0_f64.to_bits(),
                    after_value_bits: 0.04_f64.to_bits(),
                },
                ObservableTickReportV2 {
                    name: "production/michigan-rust-runtime::wayne::territory/rate-accumulation"
                        .to_owned(),
                    entity: "wayne".to_owned(),
                    field: "territory/rate-accumulation".to_owned(),
                    role: "dynamic",
                    before_value_bits: 0.01_f64.to_bits(),
                    after_value_bits: 0.02_f64.to_bits(),
                },
                ObservableTickReportV2 {
                    name: "production/michigan-rust-runtime::wayne::territory/dist-year".to_owned(),
                    entity: "wayne".to_owned(),
                    field: "territory/dist-year".to_owned(),
                    role: "dynamic",
                    before_value_bits: 2010.0_f64.to_bits(),
                    after_value_bits: 2011.0_f64.to_bits(),
                },
            ],
            persistence_reopened_after_commit: true,
            foundation: FoundationIdentityTickReportV2 {
                foundation: [0x25; 32],
                defines: [0x26; 32],
                rules: [0x27; 32],
                reference: [0x28; 32],
            },
            audit_receipt_count: 2,
            material_row_count: 9,
            material_row_digest: [0x66; 32],
            tick_content_hash: [0x77; 32],
        }
    }

    fn choice_receipt_fixture() -> ChoiceReceiptV1 {
        let stable_carrier = StableElementKeyV1::Node {
            scenario: "pilot/struggle".to_owned(),
            local_name: "worker".to_owned(),
        };
        let identity = KernelInstanceIdentityV1 {
            replay_session: b"must-not-appear-in-operational-json".to_vec(),
            replay_seed: 17_i64.to_be_bytes(),
            tick: 7,
            rule_id: "struggle/spark-mechanic".to_owned(),
            subject: stable_carrier,
            active_elements: Vec::new(),
        };
        let kernel = FiniteKernelV1 {
            sample: "struggle/spark".to_owned(),
            sample_path: vec![0, 1, 1],
            slot: 0,
            slot_path: Vec::new(),
            enum_type: EnumTypeId(0),
            enum_type_name: "StruggleSparkOutcome".to_owned(),
            branches: ["EXCESSIVE_FORCE", "NO_INCIDENT"]
                .into_iter()
                .enumerate()
                .map(|(ordinal, member)| KernelBranchV1 {
                    enum_type: "StruggleSparkOutcome".to_owned(),
                    member: member.to_owned(),
                    ordinal: u32::try_from(ordinal).expect("two branches"),
                    mass: SExpr::Atom(Atom::Mass(Mass::from_nanounits(1))),
                    effects: Vec::new(),
                    form_path: vec![0, 1, u32::try_from(ordinal).expect("two branches")],
                    head_path: vec![0, 1, u32::try_from(ordinal).expect("two branches"), 0],
                    mass_path: vec![0, 1, u32::try_from(ordinal).expect("two branches"), 3],
                    mass_literals: Vec::new(),
                    quantize_mass_paths: Vec::new(),
                    static_mass: Some(Mass::from_nanounits(1)),
                })
                .collect(),
            form_path: vec![0, 1],
            head_path: vec![0, 1, 0],
        };
        let realization = realize_kernel(
            &identity,
            &kernel,
            &[Mass::from_nanounits(1), Mass::from_nanounits(3)],
            0,
        )
        .expect("valid finite realization");
        ChoiceReceiptV1::try_new(0, &identity, realization).expect("valid choice receipt")
    }

    fn assert_report_core_json(row: &serde_json::Value) {
        assert_eq!(row["schema"], TICK_REPORT_SCHEMA_V2);
        assert_eq!(row["resolve_tick"], 7);
        assert_eq!(
            row["commit_disposition"],
            "reconciled_after_ambiguous_commit"
        );
        assert_eq!(row["rules"]["considered"], 5);
        assert_eq!(row["rules"]["fired"], 3);
        assert_eq!(row["rules"]["per_rule"][0]["considered"], 2);
        assert_eq!(row["rules"]["per_rule"][0]["fired"], 1);
        assert_eq!(
            row["stable_graph"]["before_sha256"],
            "2323232323232323232323232323232323232323232323232323232323232323"
        );
        assert_eq!(
            row["stable_graph"]["after_sha256"],
            "2424242424242424242424242424242424242424242424242424242424242424"
        );
        assert_eq!(row["scope"]["slice_id"], "michigan-persistence-slice");
        assert_eq!(row["scope"]["scenario"], "production/michigan-rust-runtime");
        assert_eq!(row["scope"]["fixed_replay_seed"], 281);
        assert_eq!(row["scope"]["parameter_overrides"], false);
        assert_eq!(row["scope"]["stochastic_draws"], false);
        assert_eq!(row["scope"]["dynamic_h3_updates"], false);
    }

    fn assert_report_event_and_observable_json(row: &serde_json::Value) {
        assert_eq!(row["events"]["count"], 1);
        assert_eq!(
            row["events"]["per_type"][0]["event_type"],
            "EventType/EXAMPLE"
        );
        assert_eq!(row["events"]["per_type"][0]["count"], 1);
        assert_eq!(row["observables"].as_array().map(Vec::len), Some(5));
        assert_eq!(
            row["observables"][0]["name"],
            "production/michigan-rust-runtime::wayne::territory/median-wage"
        );
        assert_eq!(row["observables"][0]["entity"], "wayne");
        assert_eq!(row["observables"][0]["field"], "territory/median-wage");
        assert_eq!(row["observables"][0]["role"], "configured_input");
        assert_eq!(row["observables"][0]["kind"], "f64");
        assert_eq!(row["observables"][0]["before_value"], 20.0);
        assert_eq!(row["observables"][0]["before_bits_hex"], "4034000000000000");
        assert_eq!(row["observables"][0]["after_value"], 21.0);
        assert_eq!(row["observables"][0]["after_bits_hex"], "4035000000000000");
        assert_eq!(row["observables"][4]["field"], "territory/dist-year");
        assert_eq!(row["observables"][4]["role"], "dynamic");
    }

    fn assert_report_persistence_json(row: &serde_json::Value) {
        assert_eq!(row["persistence"]["reopened_after_commit"], true);
        assert_eq!(
            row["foundation"]["foundation_sha256"],
            "2525252525252525252525252525252525252525252525252525252525252525"
        );
        assert_eq!(
            row["foundation"]["defines_sha256"],
            "2626262626262626262626262626262626262626262626262626262626262626"
        );
        assert_eq!(
            row["foundation"]["rules_sha256"],
            "2727272727272727272727272727272727272727272727272727272727272727"
        );
        assert_eq!(
            row["foundation"]["reference_sha256"],
            "2828282828282828282828282828282828282828282828282828282828282828"
        );
        assert_eq!(row["audit_receipts"]["count"], 2);
        assert_eq!(row["material_rows"]["count"], 9);
    }

    fn assert_report_is_secret_safe(rendered: &str) {
        for forbidden in [
            "BABYLON_RUNTIME_DSN",
            "postgres://",
            "hostname",
            "timestamp",
        ] {
            assert!(!rendered.contains(forbidden), "leaked {forbidden}");
        }
    }

    fn territory_value(session: &TickSession<HypergraphStore>, field: &str) -> f64 {
        session
            .graph()
            .node_attribute(NodeId(0), field)
            .unwrap_or_else(|error| panic!("Wayne {field}: {}", error.message))
    }

    #[test]
    fn michigan_smoke_drives_phi_accumulation_on_the_tick_52_rollover() {
        let session_id = SessionId::new("per281/michigan-rollover-contract")
            .expect("the fixed smoke identity is nonempty");
        let mut session = TickSession::new(SCENARIO, RULE, HypergraphStore::new(), session_id)
            .expect("the production Michigan smoke content must load");

        for tick in 1..=51 {
            let report = session
                .advance(&mut CollectingSink::default())
                .unwrap_or_else(|error| panic!("Michigan pre-rollover tick {tick}: {error}"));
            assert_eq!(
                report
                    .per_rule_fired
                    .iter()
                    .find(|(rule, _)| rule == "class-dynamics/a01-rollover-accumulation-smoke")
                    .map(|(_, fired)| *fired),
                Some(0),
                "the annual mechanics stay inert before tick 52"
            );
            assert_eq!(
                territory_value(&session, "territory/rate-accumulation").to_bits(),
                0.0_f64.to_bits(),
                "the annual accumulation rate stays at its seed before rollover"
            );
        }

        let phi_adjustment = territory_value(&session, "territory/phi-savings-adjustment");
        assert!(
            phi_adjustment > 0.0 && phi_adjustment < 0.05,
            "the uncapped Phi-to-savings gradient must be live before rollover"
        );

        let report = session
            .advance(&mut CollectingSink::default())
            .expect("Michigan tick 52 must cross the annual boundary");
        assert_eq!(
            report
                .per_rule_fired
                .iter()
                .find(|(rule, _)| rule == "class-dynamics/a01-rollover-accumulation-smoke")
                .map(|(_, fired)| *fired),
            Some(1),
            "the real class-dynamics accumulation path fires at tick 52"
        );
        assert_eq!(
            territory_value(&session, "territory/dist-year").to_bits(),
            2011.0_f64.to_bits(),
            "the annual distribution year advances exactly once"
        );
        let rate = territory_value(&session, "territory/rate-accumulation");
        assert!(
            rate > 0.0 && rate < 0.08,
            "wage, Phi, and savings must produce a bounded accumulation rate"
        );
    }

    #[test]
    fn michigan_smoke_restarts_at_the_initial_rollover_and_terminal_boundaries() {
        assert_eq!(MICHIGAN_SMOKE_TICKS, 60);
        assert_eq!(MICHIGAN_SMOKE_RESTART_TICKS, [1, 51, 52, 60]);
    }

    #[test]
    fn report_v2_captures_tick_one_observable_transitions_and_sorted_event_counts() {
        let (mut session, _) = runtime_foundation().expect("Michigan runtime foundation");
        let before = session
            .stable_graph_state()
            .expect("Michigan pre-tick graph state recomposes");
        let actions = OrderedPracticeActionBatchV1::empty(session.session_identity().clone(), 1)
            .expect("tick-one actions");
        session
            .advance(&mut CollectingSink::default(), &actions)
            .expect("Michigan tick one succeeds");
        let after = session
            .stable_graph_state()
            .expect("Michigan post-tick graph state recomposes");
        let observables = collect_observable_transitions_v2(&before, &after)
            .expect("paired observable allowlist is complete");
        let fields = observables
            .iter()
            .map(|observable| observable.field.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            fields,
            [
                "territory/median-wage",
                "territory/phi-hour",
                "territory/phi-savings-adjustment",
                "territory/rate-accumulation",
                "territory/dist-year",
            ]
        );
        assert_eq!(observables[0].before_value_bits, 21.0_f64.to_bits());
        assert_eq!(observables[0].after_value_bits, 21.0_f64.to_bits());
        assert_eq!(
            observables[2].before_value_bits,
            0.0_f64.to_bits(),
            "tick one begins from the configured Phi-adjustment seed"
        );
        assert!(
            f64::from_bits(observables[2].after_value_bits) > 0.0,
            "tick one exposes the live Phi-to-savings change"
        );

        let mut sink = CollectingSink::default();
        sink.events.push(("EventType/ZETA".to_owned(), Vec::new()));
        sink.events.push(("EventType/ALPHA".to_owned(), Vec::new()));
        sink.events.push(("EventType/ZETA".to_owned(), Vec::new()));
        let per_type = collect_event_type_counts_v2(&sink, 3).expect("event total matches");
        assert_eq!(
            per_type,
            [
                EventTypeTickReportV2 {
                    event_type: "EventType/ALPHA".to_owned(),
                    count: 1,
                },
                EventTypeTickReportV2 {
                    event_type: "EventType/ZETA".to_owned(),
                    count: 2,
                },
            ]
        );
        assert!(collect_event_type_counts_v2(&sink, 2).is_err());
    }

    #[test]
    fn persistence_readback_marks_intervals_smoke_boundaries_and_final_reports() {
        assert!(!should_reopen_after_commit_v2(1, 520, &[], Some(52), true));
        assert!(should_reopen_after_commit_v2(52, 520, &[], Some(52), true));
        assert!(should_reopen_after_commit_v2(51, 60, &[51], None, false));
        assert!(should_reopen_after_commit_v2(520, 520, &[], None, true));
        assert!(!should_reopen_after_commit_v2(520, 520, &[], None, false));
    }

    #[test]
    fn report_foundation_identity_is_derived_from_exact_runtime_sources() {
        let (session, bundle) = runtime_foundation().expect("Michigan runtime foundation");
        let foundation = babylon_persistence::CampaignFoundationV1::capture(&session, bundle)
            .expect("tick-zero foundation captures");

        let identity = foundation_identity_v2(&foundation);

        assert_eq!(
            identity.foundation,
            babylon_kernel::sha256_of(foundation.canonical_bytes())
        );
        assert_eq!(identity.defines, foundation.content_digest().defines_hash);
        assert_eq!(identity.rules, foundation.content_digest().rules_hash);
        assert_eq!(
            identity.reference,
            *foundation.reference_digest().as_bytes()
        );
        assert_eq!(identity, foundation_identity_v2(&foundation));
    }

    #[test]
    fn report_jsonl_is_deterministic_flushed_and_secret_safe() {
        let first_path = report_path("first");
        let second_path = report_path("second");
        let report = report_fixture();
        let mut first = TickReportJsonlWriter::create(&first_path).expect("new first report path");
        first.write_report(&report).expect("first report row");
        let first_bytes = std::fs::read(&first_path).expect("flushed first report row");
        let mut second =
            TickReportJsonlWriter::create(&second_path).expect("new second report path");
        second.write_report(&report).expect("second report row");
        let second_bytes = std::fs::read(&second_path).expect("flushed second report row");

        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first_bytes.last(), Some(&b'\n'));
        let row: serde_json::Value =
            serde_json::from_slice(&first_bytes).expect("one valid JSON object");
        assert_report_core_json(&row);
        assert_report_event_and_observable_json(&row);
        assert_report_persistence_json(&row);
        let rendered = String::from_utf8(first_bytes).expect("report is UTF-8");
        assert_report_is_secret_safe(&rendered);

        drop(first);
        drop(second);
        std::fs::remove_file(first_path).expect("remove first report fixture");
        std::fs::remove_file(second_path).expect("remove second report fixture");
    }

    #[test]
    fn report_jsonl_refuses_to_overwrite_an_existing_path() {
        let path = report_path("existing");
        let writer = TickReportJsonlWriter::create(&path).expect("new report path");

        assert!(TickReportJsonlWriter::create(&path).is_err());

        drop(writer);
        std::fs::remove_file(path).expect("remove report fixture");
    }

    #[test]
    fn report_jsonl_write_failure_names_the_already_durable_tick() {
        let path = report_path("read-only");
        std::fs::write(&path, b"").expect("create read-only report fixture");
        let read_only = std::fs::File::open(&path).expect("open fixture without write access");
        let mut writer = TickReportJsonlWriter {
            output: std::io::BufWriter::new(read_only),
        };

        let error = writer
            .write_report(&report_fixture())
            .expect_err("read-only report must refuse the acknowledged row");

        assert_eq!(error, "tick report JSONL write failed after durable tick 7");
        std::fs::remove_file(path).expect("remove read-only report fixture");
    }

    #[test]
    fn choice_receipt_jsonl_is_exact_deterministic_and_secret_safe() {
        let first_path = report_path("choice-first");
        let second_path = report_path("choice-second");
        let choices = [choice_receipt_fixture()];
        let mut first =
            ChoiceReceiptJsonlWriter::create(&first_path).expect("new first choice path");
        first
            .write_tick(7, [0x55; 32], &choices)
            .expect("first choice row");
        let first_bytes = std::fs::read(&first_path).expect("flushed first choice row");
        let mut second =
            ChoiceReceiptJsonlWriter::create(&second_path).expect("new second choice path");
        second
            .write_tick(7, [0x55; 32], &choices)
            .expect("second choice row");
        let second_bytes = std::fs::read(&second_path).expect("flushed second choice row");

        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first_bytes.last(), Some(&b'\n'));
        let row: serde_json::Value =
            serde_json::from_slice(&first_bytes).expect("one valid choice JSON object");
        assert_eq!(
            row["schema"],
            serde_json::Value::String(super::CHOICE_RECEIPT_REPORT_SCHEMA_V1.to_owned())
        );
        assert_eq!(row["authority"], "post_commit_operational_observation_only");
        assert_eq!(row["resolve_tick"], 7);
        assert_eq!(row["choice_receipt_count"], 1);
        assert_eq!(row["receipts"][0]["encounter_ordinal"], 0);
        assert_eq!(row["receipts"][0]["rule_id"], "struggle/spark-mechanic");
        assert_eq!(row["receipts"][0]["sample"], "struggle/spark");
        assert_eq!(row["receipts"][0]["selected_outcome"], "EXCESSIVE_FORCE");
        assert_eq!(row["receipts"][0]["draw_ticket"], "0");
        assert_eq!(row["receipts"][0]["branches"][0]["mass_nanounits"], "1");
        assert_eq!(row["receipts"][0]["branches"][1]["mass_nanounits"], "3");
        let rendered = String::from_utf8(first_bytes).expect("choice report is UTF-8");
        assert!(!rendered.contains("must-not-appear-in-operational-json"));
        assert!(!rendered.contains(super::DSN_ENV));
        assert!(!rendered.contains("replay_seed"));
        assert!(!rendered.contains("replay_session"));

        drop(first);
        drop(second);
        std::fs::remove_file(first_path).expect("remove first choice fixture");
        std::fs::remove_file(second_path).expect("remove second choice fixture");
    }

    #[test]
    fn choice_receipt_jsonl_refuses_overwrite_and_names_post_commit_write_failure() {
        let path = report_path("choice-existing");
        let writer = ChoiceReceiptJsonlWriter::create(&path).expect("new choice report path");
        assert!(ChoiceReceiptJsonlWriter::create(&path).is_err());
        drop(writer);
        std::fs::remove_file(&path).expect("remove create-new fixture");

        std::fs::write(&path, b"").expect("create read-only choice fixture");
        let read_only = std::fs::File::open(&path).expect("open fixture without write access");
        let mut writer = ChoiceReceiptJsonlWriter {
            output: std::io::BufWriter::new(read_only),
        };
        let error = writer
            .write_tick(7, [0x55; 32], &[])
            .expect_err("read-only choice report must refuse the acknowledged row");
        assert_eq!(
            error,
            "choice receipt JSONL write failed after durable tick 7"
        );
        std::fs::remove_file(path).expect("remove read-only choice fixture");
    }

    #[test]
    fn production_run_commands_accept_the_closed_supported_surface() {
        assert_eq!(
            parse_command(vec!["activate".into()].into_iter()),
            Ok(Command::Activate)
        );
        assert_eq!(
            parse_command(vec!["bootstrap".into()].into_iter()),
            Ok(Command::Bootstrap)
        );
        assert_eq!(
            parse_command(vec!["preflight".into()].into_iter()),
            Ok(Command::Preflight)
        );
        assert_eq!(
            parse_command(vec!["run".into(), "--ticks".into(), "5".into()].into_iter()),
            Ok(Command::Run {
                ticks: 5,
                report_jsonl: None,
                choice_receipts_jsonl: None,
                restart_every: None,
            })
        );
        assert_eq!(
            parse_command(
                vec![
                    "run".into(),
                    "--report-jsonl".into(),
                    "report.jsonl".into(),
                    "--ticks".into(),
                    "5".into(),
                ]
                .into_iter()
            ),
            Ok(Command::Run {
                ticks: 5,
                report_jsonl: Some("report.jsonl".into()),
                choice_receipts_jsonl: None,
                restart_every: None,
            })
        );
        assert_eq!(
            parse_command(
                vec![
                    "run".into(),
                    "--ticks".into(),
                    "5".into(),
                    "--report-jsonl".into(),
                    "report.jsonl".into(),
                ]
                .into_iter()
            ),
            Ok(Command::Run {
                ticks: 5,
                report_jsonl: Some("report.jsonl".into()),
                choice_receipts_jsonl: None,
                restart_every: None,
            })
        );
        assert_eq!(
            parse_command(
                vec![
                    "run".into(),
                    "--restart-every".into(),
                    "52".into(),
                    "--ticks".into(),
                    "520".into(),
                    "--report-jsonl".into(),
                    "report.jsonl".into(),
                    "--choice-receipts-jsonl".into(),
                    "choices.jsonl".into(),
                ]
                .into_iter()
            ),
            Ok(Command::Run {
                ticks: 520,
                report_jsonl: Some("report.jsonl".into()),
                choice_receipts_jsonl: Some("choices.jsonl".into()),
                restart_every: Some(52),
            })
        );
    }

    #[test]
    fn production_nonrun_commands_accept_the_closed_supported_surface() {
        assert_eq!(
            parse_command(vec!["probe".into()].into_iter()),
            Ok(Command::Probe)
        );
        assert_eq!(
            parse_command(vec!["archive".into()].into_iter()),
            Ok(Command::Archive)
        );
        assert_eq!(
            parse_command(vec!["archive-worker".into(), "--once".into()].into_iter()),
            Ok(Command::ArchiveWorker)
        );
        assert_eq!(
            parse_command(vec!["michigan-smoke".into()].into_iter()),
            Ok(Command::MichiganSmoke {
                report_jsonl: None,
                choice_receipts_jsonl: None,
            })
        );
        assert_eq!(
            parse_command(
                vec![
                    "michigan-smoke".into(),
                    "--report-jsonl".into(),
                    "report.jsonl".into(),
                ]
                .into_iter()
            ),
            Ok(Command::MichiganSmoke {
                report_jsonl: Some("report.jsonl".into()),
                choice_receipts_jsonl: None,
            })
        );
        assert_eq!(
            parse_command(
                vec![
                    "michigan-smoke".into(),
                    "--choice-receipts-jsonl".into(),
                    "choices.jsonl".into(),
                    "--report-jsonl".into(),
                    "report.jsonl".into(),
                ]
                .into_iter()
            ),
            Ok(Command::MichiganSmoke {
                report_jsonl: Some("report.jsonl".into()),
                choice_receipts_jsonl: Some("choices.jsonl".into()),
            })
        );
    }

    #[test]
    fn production_commands_refuse_invalid_or_ambiguous_arguments() {
        assert_eq!(parse_command(Vec::new().into_iter()), Err(()));
        assert_eq!(parse_command(vec!["run".into()].into_iter()), Err(()));
        assert_eq!(
            parse_command(vec!["run".into(), "--ticks".into(), "0".into()].into_iter()),
            Err(())
        );
        assert_eq!(
            parse_command(
                vec![
                    "run".into(),
                    "--ticks".into(),
                    "5".into(),
                    "--restart-every".into(),
                    "0".into(),
                ]
                .into_iter()
            ),
            Err(())
        );
        assert_eq!(
            parse_command(
                vec![
                    "run".into(),
                    "--ticks".into(),
                    "5".into(),
                    "--ticks".into(),
                    "6".into(),
                ]
                .into_iter()
            ),
            Err(())
        );
        assert_eq!(
            parse_command(
                vec![
                    "run".into(),
                    "--ticks".into(),
                    "5".into(),
                    "--restart-every".into(),
                    "2".into(),
                    "--restart-every".into(),
                    "3".into(),
                ]
                .into_iter()
            ),
            Err(())
        );
        assert_eq!(
            parse_command(
                vec![
                    "michigan-smoke".into(),
                    "--report-jsonl".into(),
                    "first.jsonl".into(),
                    "--report-jsonl".into(),
                    "second.jsonl".into(),
                ]
                .into_iter()
            ),
            Err(())
        );
        assert_eq!(
            parse_command(vec!["michigan-smoke".into(), "--unknown".into()].into_iter()),
            Err(())
        );
        assert_eq!(
            parse_command(vec!["archive-worker".into()].into_iter()),
            Err(())
        );
        assert_eq!(
            parse_command(vec!["archive-worker".into(), "--unknown".into()].into_iter()),
            Err(())
        );
        assert_eq!(
            parse_command(
                vec!["archive-worker".into(), "--once".into(), "--poll".into()].into_iter()
            ),
            Err(())
        );
        assert_eq!(parse_command(vec!["unknown".into()].into_iter()), Err(()));
    }
}
