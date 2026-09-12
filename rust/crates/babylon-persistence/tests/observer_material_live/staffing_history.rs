//! Exact historical staffing and complete-envelope refusal in an owned clone.

use super::{
    advance_material_period, assert_known_material_absence, install_reader_role,
    provision_observer_role, CampaignId, Config, DisposableTarget, DurableMaterialRuntime,
    MichiganContentPreset, NoTls, ObserverEconomyError, ObserverEconomyReader, ObserverVisibility,
    Uuid,
};
use babylon_graph::stable_element::StableElementKey;
use babylon_persistence::observer_reader::ObserverEconomySnapshot;
use postgres::{
    types::{FromSqlOwned, ToSql},
    Client,
};

struct Fixture {
    target: DisposableTarget,
    campaign: CampaignId,
    runtime: DurableMaterialRuntime,
    observer: ObserverEconomyReader,
    preview: ObserverEconomyReader,
    observer_config: Config,
    foundation_digest: [u8; 32],
}

impl Fixture {
    fn new() -> Self {
        let mut target = DisposableTarget::create();
        let campaign = CampaignId::from_uuid(Uuid::from_u128(41_101));
        let foundation = MichiganContentPreset::FourWeekDelayed
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let foundation_digest = foundation.digest();
        let runtime = DurableMaterialRuntime::create(&target.writer, campaign, foundation).unwrap();
        install_reader_role(&target.writer).unwrap();
        provision_observer_role(&target.writer).unwrap();
        let observer_config = target.login("babylon_observer", "staffinghistory");
        let observer =
            ObserverEconomyReader::connect(&observer_config, ObserverVisibility::FullObserver)
                .unwrap();
        let preview = ObserverEconomyReader::connect(
            &target.login("babylon_reader", "staffingpreview"),
            ObserverVisibility::KnownPreview,
        )
        .unwrap();
        Self {
            target,
            campaign,
            runtime,
            observer,
            preview,
            observer_config,
            foundation_digest,
        }
    }

    fn read(&self, tick: u64) -> ObserverEconomySnapshot {
        self.observer.snapshot(self.campaign, tick).unwrap()
    }

    fn advance_to(&mut self, tick: u64) {
        while self.runtime.session().completed_tick() < tick {
            advance_material_period(&mut self.runtime);
        }
    }
}

fn assert_foundation(fixture: &Fixture) {
    let snapshot = fixture.read(0);
    let production = snapshot.production.as_ref().unwrap();
    let catalog = crate::test_support::catalog();
    assert_eq!(production.staffing_accounts.len(), 5);
    let mut employed = Vec::new();
    for seed in &catalog.staffing().pools {
        let account = production
            .staffing_accounts
            .iter()
            .find(|account| account.subject.local_name == seed.local_name())
            .unwrap();
        assert_eq!(account.employed, seed.employed);
        assert_eq!(account.reserve, seed.reserve);
        assert_eq!(
            account.previous_unretained_hours,
            seed.previous_unretained_hours
        );
        assert_eq!(account.labor_force, seed.employed + seed.reserve);
        assert_eq!(account.hours_per_person, 160);
        assert_eq!(account.next_opening_period, 1);
        assert_eq!(account.next_opening_hours, seed.employed * 160);
        assert!(account.completed.is_none());
        employed.push(account.employed);
    }
    employed.sort_unstable();
    assert_eq!(employed, [1, 2, 4, 4, 20]);
    let count: i64 = fixture
        .target
        .writer
        .connect(NoTls)
        .unwrap()
        .query_one(
            "SELECT count(*) FROM babylon_state.tick_event_v2 WHERE campaign_id=$1::uuid",
            &[fixture.campaign.as_uuid()],
        )
        .unwrap()
        .get(0);
    assert_eq!(count, 0);
    assert_known_material_absence(&fixture.preview.snapshot(fixture.campaign, 0).unwrap());
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL harness; independent clone ownership"]
fn held_staffing_history_survives_advance_reopen_and_does_not_mutate_authority() {
    let mut fixture = Fixture::new();
    assert_foundation(&fixture);
    fixture.advance_to(2);
    let held = fixture.read(2);
    let digest = held.production_evidence_digest().unwrap().unwrap();
    let accounts = &held.production.as_ref().unwrap().staffing_accounts;
    assert_eq!(accounts.len(), 5);
    assert!(accounts.iter().all(|row| row
        .completed
        .as_ref()
        .is_some_and(|period| period.period == 2)));
    fixture.advance_to(4);
    let committed = *fixture.runtime.tail().unwrap();
    let world = fixture.runtime.session().current_world_hash().unwrap();
    fixture.runtime = DurableMaterialRuntime::open(
        &fixture.target.writer,
        fixture.campaign,
        fixture.foundation_digest,
    )
    .unwrap();
    let reader =
        ObserverEconomyReader::connect(&fixture.observer_config, ObserverVisibility::FullObserver)
            .unwrap();
    for tick in [4, 0, 2, 3, 2] {
        let snapshot = reader.snapshot(fixture.campaign, tick).unwrap();
        assert_eq!(snapshot.resolve_tick, tick);
        assert_known_material_absence(&fixture.preview.snapshot(fixture.campaign, tick).unwrap());
        if tick == 2 {
            assert_eq!(snapshot.production_evidence_digest().unwrap(), Some(digest));
            assert_eq!(snapshot, held);
        }
        assert_eq!(fixture.runtime.session().completed_tick(), 4);
        assert_eq!(fixture.runtime.tail(), Some(&committed));
        assert_eq!(
            fixture.runtime.session().current_world_hash().unwrap(),
            world
        );
    }
    let reopened = DurableMaterialRuntime::open(
        &fixture.target.writer,
        fixture.campaign,
        fixture.foundation_digest,
    )
    .unwrap();
    assert_eq!(reopened.tail(), Some(&committed));
    assert_eq!(reopened.session().current_world_hash().unwrap(), world);
}

#[derive(Clone, Copy, Debug)]
struct Fault {
    relation: &'static str,
    column: &'static str,
    predicate: &'static str,
    tick: i64,
}

// Only constants in this test select identifiers. CTIDs identify one owned row
// across each autocommit write; RETURNING captures its changed physical location.
// Hold the reader result until the old typed value is restored, then check refusal.
fn assert_fault<T: FromSqlOwned + ToSql + Sync + PartialEq + std::fmt::Debug>(
    fixture: &Fixture,
    writer: &mut Client,
    healthy: &ObserverEconomySnapshot,
    fault: Fault,
    change: impl FnOnce(&T) -> T,
) {
    let row = writer.query_one(&format!(
        "SELECT ctid::text, {} FROM babylon_state.{} WHERE campaign_id=$1::uuid AND resolve_tick=$2 AND {} ORDER BY ctid LIMIT 1",
        fault.column, fault.relation, fault.predicate,
    ), &[fixture.campaign.as_uuid(), &fault.tick]).unwrap();
    let location: String = row.get(0);
    let original: T = row.get(1);
    let changed = change(&original);
    assert_ne!(changed, original, "{fault:?}");
    let update = format!(
        "UPDATE babylon_state.{} SET {}=$1 WHERE ctid=$2::text::tid RETURNING ctid::text",
        fault.relation, fault.column
    );
    let moved = writer.query(&update, &[&changed, &location]).unwrap();
    assert_eq!(moved.len(), 1, "{fault:?}");
    let location: String = moved[0].get(0);
    let refusal = fixture
        .observer
        .snapshot(fixture.campaign, healthy.resolve_tick);
    let restored = writer.query(&update, &[&original, &location]).unwrap();
    assert_eq!(restored.len(), 1, "{fault:?}");
    assert_eq!(
        refusal,
        Err(ObserverEconomyError::InvalidProjection),
        "{fault:?}"
    );
    assert_eq!(
        fixture.read(healthy.resolve_tick),
        *healthy,
        "restore {fault:?}"
    );
}

fn flipped(bytes: &[u8]) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    *changed.last_mut().unwrap() ^= 1;
    changed
}

fn assert_graph_and_state_faults(
    fixture: &Fixture,
    writer: &mut Client,
    healthy: &ObserverEconomySnapshot,
) {
    for tick in [2, 1] {
        assert_fault(
            fixture,
            writer,
            healthy,
            Fault {
                relation: "graph_node_f64_v1",
                column: "value_bits",
                predicate: "qname='social-class/employed-population'",
                tick,
            },
            |value: &i64| value ^ 1,
        );
    }
    assert_fault(
        fixture,
        writer,
        healthy,
        Fault {
            relation: "graph_node_f64_v1",
            column: "value_bits",
            predicate: "qname NOT LIKE 'social-class/%'",
            tick: 2,
        },
        |value: &i64| value ^ 1,
    );
    assert_fault(
        fixture,
        writer,
        healthy,
        Fault {
            relation: "hex_state_delta_v1",
            column: "c_bits",
            predicate: "true",
            tick: 2,
        },
        |value: &i64| value ^ 1,
    );
}

fn assert_event_faults(fixture: &Fixture, writer: &mut Client, healthy: &ObserverEconomySnapshot) {
    assert_fault(
        fixture,
        writer,
        healthy,
        Fault {
            relation: "tick_event_field_v2",
            column: "int_value",
            predicate: "field_name='retained-hours'",
            tick: 2,
        },
        |value: &i64| value + 1,
    );
    assert_fault(
        fixture,
        writer,
        healthy,
        Fault {
            relation: "tick_event_field_v2",
            column: "stable_key",
            predicate: "field_name='subject'",
            tick: 2,
        },
        |bytes: &Vec<u8>| {
            let StableElementKey::Node {
                scenario,
                mut local_name,
            } = StableElementKey::from_canonical_bytes(bytes).unwrap()
            else {
                panic!("native staffing subject is a node");
            };
            local_name.push_str("-foreign");
            StableElementKey::Node {
                scenario,
                local_name,
            }
            .canonical_bytes()
            .unwrap()
        },
    );
    assert_fault(
        fixture,
        writer,
        healthy,
        Fault {
            relation: "tick_event_v2",
            column: "emitting_rule",
            predicate: "event_type='WORKFORCE_STAFFING'",
            tick: 2,
        },
        |value: &String| format!("{value}-foreign"),
    );
    swap_event_fields(writer, fixture.campaign);
    let refusal = fixture.observer.snapshot(fixture.campaign, 2);
    swap_event_fields(writer, fixture.campaign);
    assert_eq!(refusal, Err(ObserverEconomyError::InvalidProjection));
    assert_eq!(fixture.read(2), *healthy);
}

fn swap_event_fields(writer: &mut Client, campaign: CampaignId) {
    let mut tx = writer.transaction().unwrap();
    let ordinal: i64 = tx.query_one(
        "SELECT ordinal FROM babylon_state.tick_event_v2 WHERE campaign_id=$1::uuid AND resolve_tick=2 AND event_type='WORKFORCE_STAFFING' ORDER BY ordinal LIMIT 1",
        &[campaign.as_uuid()],
    ).unwrap().get(0);
    // The deferred continuity trigger sees only the complete, gap-free swap.
    // The immediate primary key stays unique throughout using one spare slot.
    for (from, to) in [(11_i64, 13_i64), (12, 11), (13, 12)] {
        assert_eq!(tx.execute(
            "UPDATE babylon_state.tick_event_field_v2 SET position=$4 WHERE campaign_id=$1::uuid AND resolve_tick=2 AND ordinal=$2 AND position=$3",
            &[campaign.as_uuid(), &ordinal, &from, &to],
        ).unwrap(), 1);
    }
    tx.commit().unwrap();
}

fn assert_commit_faults(fixture: &Fixture, writer: &mut Client, healthy: &ObserverEconomySnapshot) {
    for (relation, column, predicate) in [
        (
            "checkpoint_section_v1",
            "exact_section_bytes",
            "section_tag=2",
        ),
        ("checkpoint_manifest", "manifest_bytes", "true"),
        ("tick_action_batch_v1", "exact_action_batch_bytes", "true"),
        ("archive_dirty_receipt_v1", "tick_content_hash", "true"),
        ("tick_commit", "envelope_digest", "true"),
    ] {
        assert_fault(
            fixture,
            writer,
            healthy,
            Fault {
                relation,
                column,
                predicate,
                tick: 2,
            },
            |bytes: &Vec<u8>| flipped(bytes),
        );
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL harness; independent clone ownership"]
fn complete_marker_authentication_refuses_independent_staffing_and_auxiliary_row_corruption() {
    let mut fixture = Fixture::new();
    fixture.advance_to(4);
    let healthy = fixture.read(2);
    let tail = *fixture.runtime.tail().unwrap();
    let world = fixture.runtime.session().current_world_hash().unwrap();
    let mut writer = fixture.target.writer.connect(NoTls).unwrap();
    assert_graph_and_state_faults(&fixture, &mut writer, &healthy);
    assert_event_faults(&fixture, &mut writer, &healthy);
    assert_commit_faults(&fixture, &mut writer, &healthy);
    assert_known_material_absence(&fixture.preview.snapshot(fixture.campaign, 2).unwrap());
    assert_eq!(fixture.runtime.tail(), Some(&tail));
    assert_eq!(
        fixture.runtime.session().current_world_hash().unwrap(),
        world
    );
    let reopened = DurableMaterialRuntime::open(
        &fixture.target.writer,
        fixture.campaign,
        fixture.foundation_digest,
    )
    .unwrap();
    assert_eq!(reopened.tail(), Some(&tail));
    assert_eq!(reopened.session().current_world_hash().unwrap(), world);
}
