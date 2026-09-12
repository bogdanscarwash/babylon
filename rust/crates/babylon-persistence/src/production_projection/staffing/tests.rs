use std::sync::OnceLock;

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::stable_state::{
    compose_stable_graph_state_from_rows, StableGraphStateRowsInput,
};
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::replay_session::ReplayCommitDisposition;

use super::*;
use crate::michigan_content::MichiganContentPreset;

#[derive(Clone)]
struct Window {
    opening: Option<StableGraphState>,
    graph: StableGraphState,
    register: MaterialWorldRegister,
    events: Vec<StoredEvent>,
}

impl Window {
    fn project(&self) -> Result<Vec<ProductionStaffingAccount>> {
        project_staffing_accounts(
            &fixture().composition,
            &self.graph,
            &self.register,
            self.opening.as_ref(),
            &self.events,
        )
    }
}

struct PublishedFixture {
    composition: StaffingComposition,
    windows: Vec<Window>,
}

/// Actual engine publications through an in-memory commit sink. Database
/// authentication is a separate live test; these tests begin at its typed seam.
fn fixture() -> &'static PublishedFixture {
    static FIXTURE: OnceLock<PublishedFixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let foundation = MichiganContentPreset::FourWeekStandard
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let composition = foundation.labor().clone();
        let mut session = foundation.into_session().unwrap();
        let mut windows = vec![Window {
            opening: None,
            graph: session.graph_session().stable_graph_state().unwrap(),
            register: session.material().clone(),
            events: vec![],
        }];
        let mut sink = CollectingSink::default();
        for tick in 1..=8 {
            let opening = session.graph_session().stable_graph_state().unwrap();
            let actions = OrderedPracticeActionBatch::empty(
                session.graph_session().session_identity().clone(),
                tick,
            )
            .unwrap();
            let prepared = session.prepare_advance(&actions).unwrap();
            let window = Window {
                opening: Some(opening),
                graph: prepared.graph_report().result_stable_graph().clone(),
                register: prepared.material().register().clone(),
                events: prepared
                    .graph_report()
                    .successful_event_batch()
                    .events()
                    .iter()
                    .map(|event| StoredEvent {
                        emitting_rule: event.emitting_rule().to_owned(),
                        choice_receipt_ordinal: event
                            .choice_receipt()
                            .map(babylon_tick::choice_receipt::ChoiceReceiptRef::encounter_ordinal),
                        event_type: event.event_type().to_owned(),
                        fields: event.fields().to_vec(),
                    })
                    .collect(),
            };
            session
                .commit_prepared_and_publish(&mut sink, prepared, |_| {
                    Ok::<_, ()>(ReplayCommitDisposition::Committed)
                })
                .unwrap();
            assert_eq!(window.register, *session.material());
            assert_eq!(
                window.graph,
                session.graph_session().stable_graph_state().unwrap()
            );
            windows.push(window);
        }
        PublishedFixture {
            composition,
            windows,
        }
    })
}

fn committed() -> Window {
    fixture().windows[1].clone()
}

fn field_mut<'a>(event: &'a mut StoredEvent, name: &str) -> &'a mut StableBslValue {
    &mut event
        .fields
        .iter_mut()
        .find(|(key, _)| key == name)
        .unwrap()
        .1
}

fn event_integer(event: &StoredEvent, name: &str) -> u64 {
    let value = &event.fields.iter().find(|(key, _)| key == name).unwrap().1;
    let StableBslValue::Int(value) = value else {
        panic!("integer event fixture");
    };
    u64::try_from(*value).unwrap()
}

fn local_name() -> &'static str {
    let StableElementKey::Node { local_name, .. } = fixture().composition.bindings()[0].subject()
    else {
        panic!("node binding");
    };
    local_name
}

fn graph_rows(graph: &StableGraphState) -> StableGraphStateRowsInput {
    let rows = graph.rows();
    StableGraphStateRowsInput {
        nodes: rows.nodes().to_vec(),
        node_f64: rows.node_f64().to_vec(),
        edges: rows.edges().to_vec(),
        hyperedges: rows.hyperedges().to_vec(),
        edge_f64: rows.edge_f64().to_vec(),
        node_currency: rows.node_currency().to_vec(),
        hyperedge_f64: rows.hyperedge_f64().to_vec(),
    }
}

fn change_graph(
    graph: &StableGraphState,
    edit: impl FnOnce(&mut StableGraphStateRowsInput),
) -> StableGraphState {
    let mut rows = graph_rows(graph);
    edit(&mut rows);
    compose_stable_graph_state_from_rows(graph.scenario_scope(), rows).unwrap()
}

fn set_graph_number(graph: &StableGraphState, field: &str, value: f64) -> StableGraphState {
    change_graph(graph, |rows| {
        rows.node_f64
            .iter_mut()
            .find(|(node, key, _)| node == local_name() && key == field)
            .unwrap()
            .2 = value.to_bits();
    })
}

#[test]
fn foundation_reports_modeled_people_without_a_completed_staffing_event() {
    let foundation = &fixture().windows[0];
    let accounts = foundation.project().unwrap();
    assert_eq!(accounts.len(), 5);
    assert_eq!(accounts.iter().map(|row| row.employed).sum::<u64>(), 31);
    assert!(accounts
        .iter()
        .all(|row| row.reserve == 0 && row.completed.is_none()));
    let pools: std::collections::BTreeSet<_> = accounts.iter().map(|row| &row.pool_id).collect();
    assert_eq!(pools.len(), 5);
    for binding in fixture().composition.bindings() {
        let pool = binding.pool();
        let account = accounts
            .iter()
            .find(|row| row.pool_id == digest_hex(&pool.pool_id().as_bytes()))
            .unwrap();
        assert_eq!(account.site_id, digest_hex(&pool.site_id().as_bytes()));
        assert_eq!(account.unit_id, digest_hex(&pool.unit_id().as_bytes()));
        assert_eq!(account.labor_force, pool.labor_force());
    }
    for account in accounts {
        assert_eq!(account.hours_per_person, 160);
        assert_eq!(account.next_opening_period, 1);
        assert_eq!(account.next_opening_hours, account.employed * 160);
        assert_eq!(account.labor_force, account.employed + account.reserve);
    }
    let mut invented = foundation.clone();
    invented.events = committed().events;
    assert_eq!(invented.project(), Err(ProductionProjectionError::History));
    invented.events.clear();
    invented.opening = Some(invented.graph.clone());
    assert_eq!(invented.project(), Err(ProductionProjectionError::History));
}

#[test]
fn published_windows_keep_all_pools_exact_and_disclose_retention_release_and_rehire() {
    let mut saw_retention = false;
    let mut saw_release = false;
    let mut saw_rehire = false;
    let mut saw_quiet = false;
    for window in &fixture().windows[1..] {
        let accounts = window.project().unwrap();
        assert_eq!(accounts.len(), 5);
        for account in accounts {
            assert_eq!(account.hours_per_person, 160);
            assert_eq!(account.employed + account.reserve, account.labor_force);
            assert_eq!(
                account.next_opening_period,
                window.register.completed_tick() + 1
            );
            assert_eq!(account.next_opening_hours, account.employed * 160);
            let completed = account.completed.unwrap();
            assert_eq!(completed.period, window.register.completed_tick());
            assert_eq!(
                account.previous_unretained_hours,
                completed.current_unretained_hours
            );
            let event = window.events.iter().find(|event| {
                matches!(event.fields.iter().find(|(name, _)| name == "subject"),
                    Some((_, StableBslValue::Node(StableElementKey::Node { scenario, local_name })))
                        if *scenario == account.subject.scenario && *local_name == account.subject.local_name)
            }).unwrap();
            assert_eq!(
                completed.retained_hours,
                event_integer(event, "retained-hours")
            );
            assert_eq!(completed.hires, event_integer(event, "hires"));
            assert_eq!(completed.separations, event_integer(event, "separations"));
            saw_retention |= completed.retained_hours > completed.current_unretained_hours;
            saw_release |= completed.separations > 0;
            saw_rehire |= completed.hires > 0;
            saw_quiet |= completed.hires == 0 && completed.separations == 0;
        }
    }
    assert!(
        saw_retention,
        "published windows include a retained work request"
    );
    assert!(saw_release, "published windows include separations");
    assert!(
        saw_rehire,
        "published windows include hires from the closed reserve"
    );
    assert!(
        saw_quiet,
        "completed zero movement remains a present account"
    );
}

#[test]
fn receipt_order_is_irrelevant_but_missing_duplicate_and_extra_subjects_refuse() {
    let original = committed();
    let expected = original.project().unwrap();
    let mut reordered = original.clone();
    reordered.events.reverse();
    assert_eq!(reordered.project().unwrap(), expected);
    let mut missing = original.clone();
    missing.events.remove(0);
    assert_eq!(missing.project(), Err(ProductionProjectionError::History));
    let mut duplicate = original.clone();
    duplicate.events.push(duplicate.events[0].clone());
    assert_eq!(duplicate.project(), Err(ProductionProjectionError::History));
    let mut extra = original.clone();
    let mut event = extra.events[0].clone();
    let StableBslValue::Node(StableElementKey::Node { local_name, .. }) =
        field_mut(&mut event, "subject")
    else {
        panic!("node subject");
    };
    local_name.push_str("-foreign");
    extra.events.push(event);
    assert_eq!(extra.project(), Err(ProductionProjectionError::History));
    let mut no_opening = original;
    no_opening.opening = None;
    assert_eq!(
        no_opening.project(),
        Err(ProductionProjectionError::History)
    );
}

#[test]
fn staffing_emitter_choice_and_subject_contracts_are_closed() {
    type Edit = fn(&mut StoredEvent);
    let edits: [(&str, Edit); 5] = [
        ("foreign emitter", |event| {
            event.emitting_rule = "foreign-rule".into();
        }),
        ("wrong event type", |event| {
            event.event_type = "OTHER_EVENT".into();
        }),
        ("choice evidence", |event| {
            event.choice_receipt_ordinal = Some(0);
        }),
        ("non-node subject", |event| {
            *field_mut(event, "subject") = StableBslValue::Int(1);
        }),
        ("foreign subject", |event| {
            let StableBslValue::Node(StableElementKey::Node { scenario, .. }) =
                field_mut(event, "subject")
            else {
                panic!("node subject");
            };
            *scenario = "foreign-scenario".into();
        }),
    ];
    for (label, edit) in edits {
        let mut window = committed();
        edit(&mut window.events[0]);
        assert_eq!(
            window.project(),
            Err(ProductionProjectionError::History),
            "{label}"
        );
    }
    let mut window = committed();
    let expected = window.project().unwrap();
    let mut unrelated = window.events[0].clone();
    unrelated.emitting_rule = "unrelated-rule".into();
    unrelated.event_type = "UNRELATED_EVENT".into();
    window.events.push(unrelated);
    assert_eq!(window.project().unwrap(), expected);
}

#[test]
fn event_fields_reject_missing_unknown_duplicate_and_noninteger_values() {
    type Edit = fn(&mut StoredEvent);
    let edits: [(&str, Edit); 4] = [
        ("missing", |event| {
            event.fields.pop();
        }),
        ("extra", |event| {
            event
                .fields
                .push(("unexpected".into(), StableBslValue::Int(1)));
        }),
        ("unknown", |event| event.fields[0].0 = "unexpected".into()),
        ("duplicate", |event| {
            event.fields[0] = event.fields[1].clone();
        }),
    ];
    for (label, edit) in edits {
        let mut window = committed();
        edit(&mut window.events[0]);
        assert_eq!(
            window.project(),
            Err(ProductionProjectionError::History),
            "{label}"
        );
    }
    for value in [
        StableBslValue::Int(-1),
        StableBslValue::RealBits(40.0_f64.to_bits()),
        StableBslValue::CurrencyMicroUnits(40),
        StableBslValue::Bool(true),
    ] {
        let mut window = committed();
        *field_mut(&mut window.events[0], "current-unretained-hours") = value;
        assert_eq!(window.project(), Err(ProductionProjectionError::State));
    }
}

#[test]
fn all_receipt_endpoints_and_flow_counts_must_agree_with_the_graph_window() {
    for field in INTEGER_FIELDS
        .into_iter()
        .filter(|field| *field != "retained-hours")
    {
        let mut window = committed();
        let value = field_mut(&mut window.events[0], field);
        let StableBslValue::Int(number) = value else {
            panic!("integer");
        };
        *number += 1;
        assert_eq!(
            window.project(),
            Err(ProductionProjectionError::History),
            "{field}"
        );
    }
}

#[test]
fn opening_and_closing_stocks_memory_owner_and_scope_are_bound() {
    for opening in [false, true] {
        for field in [
            EMPLOYED_POPULATION,
            RESERVE_POPULATION,
            PREVIOUS_UNRETAINED_HOURS,
        ] {
            let mut window = committed();
            let graph = if opening {
                window.opening.as_ref().unwrap()
            } else {
                &window.graph
            };
            let changed = change_graph(graph, |rows| {
                let row = rows
                    .node_f64
                    .iter_mut()
                    .find(|(node, key, _)| node == local_name() && key == field)
                    .unwrap();
                row.2 = (f64::from_bits(row.2) + 1.0).to_bits();
            });
            if opening {
                window.opening = Some(changed);
            } else {
                window.graph = changed;
            }
            assert!(window.project().is_err(), "opening={opening} {field}");
        }
        let mut window = committed();
        let graph = if opening {
            window.opening.as_ref().unwrap()
        } else {
            &window.graph
        };
        let changed = change_graph(graph, |rows| {
            rows.nodes
                .iter_mut()
                .find(|(node, _)| node == local_name())
                .unwrap()
                .1 = "ORGANIZATION".into();
        });
        if opening {
            window.opening = Some(changed);
        } else {
            window.graph = changed;
        }
        assert_eq!(window.project(), Err(ProductionProjectionError::State));
        let mut window = committed();
        let graph = if opening {
            window.opening.as_ref().unwrap()
        } else {
            &window.graph
        };
        let changed =
            compose_stable_graph_state_from_rows("foreign-scope", graph_rows(graph)).unwrap();
        if opening {
            window.opening = Some(changed);
        } else {
            window.graph = changed;
        }
        assert_eq!(window.project(), Err(ProductionProjectionError::History));
    }
}

#[test]
fn graph_numeric_lane_and_exact_integer_boundary_are_enforced() {
    let window = committed();
    for value in [-1.0, 0.5, 9_007_199_254_740_994.0] {
        let mut changed = window.clone();
        changed.graph = set_graph_number(&changed.graph, PREVIOUS_UNRETAINED_HOURS, value);
        assert_eq!(changed.project(), Err(ProductionProjectionError::State));
    }
    let exact = set_graph_number(
        &window.graph,
        PREVIOUS_UNRETAINED_HOURS,
        9_007_199_254_740_992.0,
    );
    assert_eq!(
        population_field(&exact, local_name(), PREVIOUS_UNRETAINED_HOURS).unwrap(),
        1_u64 << 53
    );
    let mut missing = window.clone();
    missing.graph = change_graph(&missing.graph, |rows| {
        rows.node_f64
            .retain(|(node, key, _)| node != local_name() || key != PREVIOUS_UNRETAINED_HOURS);
    });
    assert_eq!(missing.project(), Err(ProductionProjectionError::State));
    let mut wrong_lane = window;
    wrong_lane.graph = change_graph(&wrong_lane.graph, |rows| {
        rows.node_f64
            .retain(|(node, key, _)| node != local_name() || key != PREVIOUS_UNRETAINED_HOURS);
        rows.node_currency
            .push((local_name().into(), PREVIOUS_UNRETAINED_HOURS.into(), 40));
    });
    assert_eq!(wrong_lane.project(), Err(ProductionProjectionError::State));
}

#[test]
fn next_labor_budget_and_pool_conservation_cannot_be_invented() {
    let original = committed();
    let pool = fixture().composition.bindings()[0].pool();
    let mut changed = original.clone();
    let mut state = changed.register.state().clone();
    state
        .labor
        .iter_mut()
        .find(|row| {
            row.site_id == pool.site_id()
                && row.unit_id == pool.unit_id()
                && row.period == state.period
        })
        .unwrap()
        .available += 1;
    changed.register =
        MaterialWorldRegister::try_new(changed.register.completed_tick(), state).unwrap();
    assert_eq!(changed.project(), Err(ProductionProjectionError::State));
    let mut changed = original.clone();
    let mut state = changed.register.state().clone();
    state
        .labor
        .retain(|row| row.site_id != pool.site_id() || row.unit_id != pool.unit_id());
    changed.register =
        MaterialWorldRegister::try_new(changed.register.completed_tick(), state).unwrap();
    assert_eq!(changed.project(), Err(ProductionProjectionError::State));
    let mut changed = original;
    let event = &mut changed.events[0];
    *field_mut(event, "hires") = StableBslValue::Int(1);
    *field_mut(event, "separations") = StableBslValue::Int(1);
    assert_eq!(changed.project(), Err(ProductionProjectionError::History));
}

#[test]
fn full_typed_integer_hours_above_graph_precision_survive_the_complete_account_path() {
    let people = 225_179_981_368_525;
    let request = 9_007_199_254_740_992;
    let next_hours = 9_007_199_254_741_000;
    let mut event = committed().events[0].clone();
    let values = [
        1, people, 0, request, request, request, people, 0, 0, people, 0, next_hours,
    ];
    for (field, value) in INTEGER_FIELDS.iter().zip(values) {
        *field_mut(&mut event, field) = StableBslValue::Int(i64::try_from(value).unwrap());
    }
    let (_, values) = event_fields(&event).unwrap();
    assert_eq!(values[11], (1_u64 << 53) + 8);
    let opening = Stocks {
        employed: people,
        reserve: 0,
        previous: request,
    };
    let result = completed_account(1, opening, opening, next_hours, values).unwrap();
    assert_eq!(result.current_unretained_hours, request);
    assert_eq!(result.retained_hours, request);
    assert_eq!(result.target_employed, people);
    assert_eq!(result.hires, 0);
    assert_eq!(result.separations, 0);
}
