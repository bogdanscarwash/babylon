//! Failed staffing ownership admission must leave no newly founded campaign.

use super::DisposableTarget;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_persistence::{
    identity::CampaignId,
    material_runtime::{DurableMaterialRuntime, MaterialRuntimeError, MaterialRuntimeFoundation},
    michigan_content::MichiganContentPreset,
    michigan_dynamic_hex_foundation, FoundationContentBundle,
};
use babylon_tick::{
    material_replay::{MaterialBaseError, MaterialReplayError},
    material_staffing::EMPLOYED_POPULATION,
    material_state::MaterialState,
    replay_session::{ReplayTickError, ReplayTickSession},
};
use postgres::NoTls;
use uuid::Uuid;

const FOREIGN_WRITER_ID: &str = "fixture/foreign-staffing-writer";
const FOREIGN_WRITER: &str = r#"
(rule fixture/foreign-staffing-writer
  :role mechanic :evidence designed
  :material-basis "fixture proves that staffing ownership refuses before durable founding"
  :fuel 64
  (anchor :after metabolism)
  (bindings (binding employed :field social-class/employed-population))
  (when #t)
  (effects (update-node self social-class/employed-population (set 0))))
"#;

fn captured_foreign_writer() -> MaterialRuntimeFoundation {
    let original = MichiganContentPreset::FourWeekStandard
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let foundation = original.graph_foundation();
    let content = foundation.content_bundle();
    let scenario = std::str::from_utf8(content.scenario_source_bytes()).unwrap();
    let prelude = content
        .prelude_source_bytes()
        .map(|bytes| std::str::from_utf8(bytes).unwrap());
    let bundle = FoundationContentBundle::try_new(
        scenario,
        prelude,
        FOREIGN_WRITER,
        content.defines_bytes(),
        content.reference_bundle_manifest_bytes(),
    )
    .unwrap();
    let graph = ReplayTickSession::new(
        scenario,
        prelude,
        FOREIGN_WRITER,
        HypergraphStore::new(),
        foundation.replay_session_identity().clone(),
        foundation.rng_seed(),
        bundle.content_digest().clone(),
        bundle.reference_digest(),
        MaterialState::try_new(michigan_dynamic_hex_foundation().unwrap()).unwrap(),
    )
    .expect("the ordinary BSL loader admits this Mechanic field write");
    MaterialRuntimeFoundation::capture(
        graph,
        bundle,
        original.initial_register().state().clone(),
        original.spec().clone(),
    )
    .expect("source capture succeeds; live Staffed admission must reject the foreign owner")
}

fn campaign_row_counts(client: &mut postgres::Client, campaign: CampaignId) -> Vec<(String, i64)> {
    client
        .query(
            "SELECT 'state campaign', count(*) FROM babylon_state.campaign WHERE campaign_id=$1 \
             UNION ALL SELECT 'catalog campaign', count(*) FROM babylon_meta.campaign WHERE campaign_id=$1 \
             UNION ALL SELECT 'graph foundation', count(*) FROM babylon_state.campaign_foundation WHERE campaign_id=$1 \
             UNION ALL SELECT 'material foundation', count(*) FROM babylon_state.material_campaign_foundation_v2 WHERE campaign_id=$1 \
             UNION ALL SELECT 'county map', count(*) FROM babylon_meta.territory_county_map_v1 WHERE campaign_id=$1 \
             UNION ALL SELECT 'knowledge grants', count(*) FROM babylon_meta.archive_knowledge_grant_v1 WHERE campaign_id=$1 \
             UNION ALL SELECT 'commit marker', count(*) FROM babylon_state.tick_commit WHERE campaign_id=$1",
            &[campaign.as_uuid()],
        )
        .unwrap()
        .into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect()
}

#[test]
#[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
fn live_staffing_owner_refusal_rolls_back_foundation_and_grants() {
    let target = DisposableTarget::create();
    let campaign = CampaignId::from_uuid(Uuid::from_u128(32_001));
    let invalid = captured_foreign_writer();
    let error = DurableMaterialRuntime::create(&target.writer, campaign, invalid)
        .err()
        .expect("the foreign staffing writer must refuse runtime creation");
    assert!(
        matches!(
            &error,
            MaterialRuntimeError::Replay(MaterialReplayError::Graph(
                ReplayTickError::MaterialBase(MaterialBaseError::StaffingFieldOwner {
                    rule_id,
                    field,
                })
            )) if rule_id == FOREIGN_WRITER_ID && field == EMPLOYED_POPULATION
        ),
        "expected the exact staffing ownership refusal: {error:?}"
    );
    let mut client = target.writer.connect(NoTls).unwrap();
    let counts = campaign_row_counts(&mut client, campaign);
    assert!(
        counts.iter().all(|(_, count)| *count == 0),
        "failed founding must leave no campaign-owned rows: {counts:?}"
    );

    // A valid retry uses the same UUID, proving the refused attempt retained no owner.
    let valid = MichiganContentPreset::FourWeekStandard
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let digest = valid.digest();
    let runtime = DurableMaterialRuntime::create(&target.writer, campaign, valid).unwrap();
    assert_eq!(runtime.session().completed_tick(), 0);
    assert_eq!(
        DurableMaterialRuntime::open(&target.writer, campaign, digest)
            .unwrap()
            .session()
            .completed_tick(),
        0
    );
    let counts = campaign_row_counts(&mut client, campaign);
    assert!(counts.iter().all(|(name, count)| {
        if name == "commit marker" {
            *count == 0
        } else {
            *count > 0
        }
    }));
}
