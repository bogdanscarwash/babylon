//! Readings of authenticated shared freight capacity, separate from material arrivals.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use babylon_persistence::{
    production_observation::ProductionFreightCapacityAccount,
    production_observation::ProductionFreightCapacityOrder,
    production_observation::ProductionFreightReservation, production_observation::ProductionRoute,
    production_observation::ProductionSite, production_observation::ProductionSnapshot,
};

/// Capacity mass is displayed in kilograms without rounding away gram residuals.
pub(crate) fn format_freight_mass(grams: u64) -> String {
    let whole = grams / 1_000;
    let fraction = grams % 1_000;
    if fraction == 0 {
        format!("{whole} kg")
    } else {
        let fraction = format!("{fraction:03}");
        format!("{whole}.{} kg", fraction.trim_end_matches('0'))
    }
}

fn participating_routes<'a>(
    account: &ProductionFreightCapacityAccount,
    snapshot: &'a ProductionSnapshot,
) -> Vec<&'a ProductionRoute> {
    let mut routes: Vec<_> = snapshot
        .routes
        .iter()
        .filter(|route| {
            account.route_ids.contains(&route.id)
                && route
                    .stages
                    .iter()
                    .any(|leg| leg.capacity_ids.contains(&account.corridor_id))
                && snapshot
                    .sites
                    .iter()
                    .any(|site| site.id == route.supplier_site_id)
                && snapshot
                    .sites
                    .iter()
                    .any(|site| site.id == route.buyer_site_id)
        })
        .collect();
    routes.sort_by(|a, b| a.id.cmp(&b.id));
    routes
}

/// A shared principal is shown once, independent of how many routes use it.
/// Both endpoints must belong to this observation before a route is named.
pub(crate) fn shared_accounts<'a>(
    snapshot: &'a ProductionSnapshot,
    selected_site: Option<&str>,
) -> Vec<&'a ProductionFreightCapacityAccount> {
    let disclosed: BTreeSet<_> = snapshot.sites.iter().map(|site| site.id.as_str()).collect();
    let routes: BTreeMap<_, _> = snapshot
        .routes
        .iter()
        .filter(|route| {
            disclosed.contains(route.supplier_site_id.as_str())
                && disclosed.contains(route.buyer_site_id.as_str())
        })
        .map(|route| (route.id.as_str(), route))
        .collect();
    let mut accounts: Vec<_> = snapshot
        .freight_capacity_accounts
        .iter()
        .filter(|account| {
            if account.kind
                != babylon_persistence::production_observation::ProductionCapacityKind::Transport
            {
                return false;
            }
            let participants: Vec<_> = account
                .route_ids
                .iter()
                .filter_map(|id| routes.get(id.as_str()))
                .filter(|route| {
                    route
                        .stages
                        .iter()
                        .any(|stage| stage.capacity_ids.contains(&account.corridor_id))
                })
                .collect();
            participants.len() > 1
                && selected_site.is_none_or(|id| {
                    participants
                        .iter()
                        .any(|route| route.supplier_site_id == id || route.buyer_site_id == id)
                })
        })
        .collect();
    accounts.sort_by(|a, b| {
        (a.next_opening_available_grams, &a.corridor_id)
            .cmp(&(b.next_opening_available_grams, &b.corridor_id))
    });
    accounts
}

fn route_label(route: &ProductionRoute, snapshot: &ProductionSnapshot) -> String {
    let name = |id: &str| {
        snapshot
            .sites
            .iter()
            .find(|site| site.id == id)
            .map(|site| site.name.as_str())
    };
    match (name(&route.supplier_site_id), name(&route.buyer_site_id)) {
        (Some(supplier), Some(buyer)) => format!(
            "{} -> {} / {}",
            supplier.trim_end_matches(" cohort"),
            buyer.trim_end_matches(" cohort"),
            route.good
        ),
        _ => "Route endpoints unavailable in this observation".into(),
    }
}

/// The relationship rail keeps the capacity principal visible; full route
/// accounts live in the Freight reading.
pub(crate) fn account_brief(account: &ProductionFreightCapacityAccount) -> String {
    let mut output = format!("{}\n", account.corridor_label);
    if let Some(completed) = &account.completed {
        for reservation in &completed.reservations {
            writeln!(
                output,
                "Reservation period {}\n{} opening · {} reserved · {} remaining",
                reservation.reservation_period,
                format_freight_mass(reservation.opening_available_grams),
                format_freight_mass(reservation.newly_reserved_grams),
                format_freight_mass(reservation.remaining_available_grams)
            )
            .expect("String write");
        }
        if completed.reservations.is_empty() {
            output.push_str("No new reservations this period.\n");
        }
    } else {
        output.push_str("Foundation; no completed reservations.\n");
    }
    writeln!(
        output,
        "Period {} opening: {}\nCapacity reservations; arrivals are separate.",
        account.next_opening_period,
        format_freight_mass(account.next_opening_available_grams)
    )
    .expect("String write");
    output
}

pub(crate) fn account_reading(
    account: &ProductionFreightCapacityAccount,
    snapshot: &ProductionSnapshot,
) -> String {
    let mut output = format!("{}\n", account.corridor_label);
    let routes = participating_routes(account, snapshot);
    if let Some(completed) = &account.completed {
        writeln!(output, "COMMITTED DISPATCH / PERIOD {}", completed.period).expect("String write");
        for reservation in &completed.reservations {
            writeln!(
                output,
                "Reservation period {} / 28 days\nOpening {}\nNewly reserved {}\nRemaining {}\n",
                reservation.reservation_period,
                format_freight_mass(reservation.opening_available_grams),
                format_freight_mass(reservation.newly_reserved_grams),
                format_freight_mass(reservation.remaining_available_grams),
            )
            .expect("String write");
            if reservation.orders.len() > 6 {
                writeln!(output, "{} order accounts; showing six. Other participants are available in the circuit rail.", reservation.orders.len()).expect("String write");
            }
            for order in reservation.orders.iter().take(6) {
                let Some(route) = routes.iter().find(|route| {
                    Some(route.id.as_str()) == order.route_id.as_deref()
                        && route.good_id == order.good_id
                        && route.unit_id == order.unit_id
                }) else {
                    output.push_str("Reservation route detail unavailable in this observation.\n");
                    continue;
                };
                writeln!(
                    output,
                    "{}\nRequested {} {} | dispatched {} {}\nUnshipped {} {}\n",
                    route_label(route, snapshot),
                    order.requested,
                    route.unit,
                    order.dispatched,
                    route.unit,
                    order.remaining_unshipped,
                    route.unit,
                )
                .expect("String write");
            }
        }
        if completed.reservations.is_empty() {
            output.push_str("No new capacity reservations in this completed period.\n");
        }
    } else {
        output.push_str("No completed freight reservations at foundation.\n");
    }
    // Keep membership visible even when a route has no new request this period.
    let named: BTreeSet<_> = account
        .completed
        .iter()
        .flat_map(|completed| &completed.reservations)
        .flat_map(|reservation| &reservation.orders)
        .filter_map(|order| order.route_id.as_deref())
        .collect();
    for route in routes
        .iter()
        .filter(|route| !named.contains(route.id.as_str()))
        .take(6)
    {
        writeln!(output, "Participant: {}\n", route_label(route, snapshot)).expect("String write");
    }
    writeln!(output, "Next opening (period {}): {} available\nReservations use capacity; goods arrive after travel. Follow arrivals in Flow and staffing in Work.",
        account.next_opening_period, format_freight_mass(account.next_opening_available_grams),
    ).expect("String write");
    output
}

/// Competition links are separate from the supplier/buyer relation graph.
pub(crate) fn competitor_sites<'a>(
    site_id: &str,
    snapshot: &'a ProductionSnapshot,
) -> Vec<&'a ProductionSite> {
    let route_ids: BTreeSet<_> = shared_accounts(snapshot, Some(site_id))
        .into_iter()
        .flat_map(|account| &account.route_ids)
        .collect();
    let disclosed: BTreeSet<_> = snapshot.sites.iter().map(|site| site.id.as_str()).collect();
    let ids: BTreeSet<_> = snapshot
        .routes
        .iter()
        .filter(|route| {
            route_ids.contains(&route.id)
                && disclosed.contains(route.supplier_site_id.as_str())
                && disclosed.contains(route.buyer_site_id.as_str())
        })
        .filter(|route| route.supplier_site_id != site_id && route.buyer_site_id != site_id)
        .flat_map(|route| [&route.supplier_site_id, &route.buyer_site_id])
        .collect();
    ids.into_iter()
        .filter_map(|id| snapshot.sites.iter().find(|site| site.id == *id))
        .collect()
}

fn order_key(order: &ProductionFreightCapacityOrder) -> (&str, Option<&str>, &str, &str) {
    (
        &order.order_id,
        order.route_id.as_deref(),
        &order.good_id,
        &order.unit_id,
    )
}

fn pair(output: &mut String, label: &str, current: u64, compared: u64, unit: &str) {
    writeln!(output, "{label}: {current} / {compared} {unit}").expect("String write");
}

fn capacity_pair(output: &mut String, label: &str, current: u64, compared: u64) {
    writeln!(
        output,
        "{label}: {} / {}",
        format_freight_mass(current),
        format_freight_mass(compared)
    )
    .expect("String write");
}

fn compare_orders(
    output: &mut String,
    r_a: &ProductionFreightReservation,
    r_b: &ProductionFreightReservation,
    current: &ProductionSnapshot,
    compared: &ProductionSnapshot,
    a: &ProductionFreightCapacityAccount,
    b: &ProductionFreightCapacityAccount,
) {
    let orders: BTreeSet<_> = r_a
        .orders
        .iter()
        .chain(&r_b.orders)
        .map(order_key)
        .collect();
    for key in orders.into_iter().take(6) {
        let (Some(o_a), Some(o_b)) = (
            r_a.orders.iter().find(|order| order_key(order) == key),
            r_b.orders.iter().find(|order| order_key(order) == key),
        ) else {
            output.push_str("Comparable route order unavailable.\n");
            continue;
        };
        let route = participating_routes(a, current).into_iter().find(|route| {
            Some(route.id.as_str()) == o_a.route_id.as_deref()
                && route.good_id == o_a.good_id
                && route.unit_id == o_a.unit_id
        });
        let Some(route) = route else {
            output.push_str("Comparable route endpoints unavailable.\n");
            continue;
        };
        let other_route = participating_routes(b, compared).into_iter().find(|other| {
            other.id == route.id && other.good_id == route.good_id && other.unit_id == route.unit_id
        });
        let Some(other_route) = other_route else {
            output.push_str("Comparable route endpoints unavailable.\n");
            continue;
        };
        writeln!(output, "{}", route_label(route, current)).expect("String write");
        pair(
            output,
            "Requested",
            o_a.requested,
            o_b.requested,
            &route.unit,
        );
        pair(
            output,
            "Dispatched",
            o_a.dispatched,
            o_b.dispatched,
            &route.unit,
        );
        pair(
            output,
            "Remaining unshipped",
            o_a.remaining_unshipped,
            o_b.remaining_unshipped,
            &route.unit,
        );
        pair(
            output,
            "Arrived to date",
            route.delivered,
            other_route.delivered,
            &route.unit,
        );
    }
}

fn compare_reservation_capacity(
    output: &mut String,
    r_a: &ProductionFreightReservation,
    r_b: &ProductionFreightReservation,
) {
    for (label, current_value, compared_value) in [
        (
            "Opening capacity",
            r_a.opening_available_grams,
            r_b.opening_available_grams,
        ),
        (
            "Newly reserved",
            r_a.newly_reserved_grams,
            r_b.newly_reserved_grams,
        ),
        (
            "Remaining capacity",
            r_a.remaining_available_grams,
            r_b.remaining_available_grams,
        ),
    ] {
        capacity_pair(output, label, current_value, compared_value);
    }
}

fn capacity_changed(
    period: u64,
    current: &ProductionFreightCapacityAccount,
    compared: &ProductionFreightCapacityAccount,
) -> bool {
    if current.next_opening_period == compared.next_opening_period
        && current.next_opening_available_grams != compared.next_opening_available_grams
    {
        return true;
    }
    let (Some(a), Some(b)) = (&current.completed, &compared.completed) else {
        return false;
    };
    period > 0
        && a.period == period
        && b.period == period
        && a.reservations.iter().any(|left| {
            b.reservations.iter().any(|right| {
                left.reservation_period == right.reservation_period
                    && left.opening_available_grams != right.opening_available_grams
            })
        })
}

fn comparison_keys<'a>(
    period: u64,
    left: &[&'a ProductionFreightCapacityAccount],
    right: &[&'a ProductionFreightCapacityAccount],
) -> Vec<&'a str> {
    let left: BTreeMap<_, _> = left.iter().map(|a| (a.corridor_id.as_str(), *a)).collect();
    let right: BTreeMap<_, _> = right.iter().map(|a| (a.corridor_id.as_str(), *a)).collect();
    let mut keys: Vec<_> = left
        .keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    keys.sort_by_key(|key| {
        let changed = left
            .get(key)
            .zip(right.get(key))
            .is_some_and(|(a, b)| capacity_changed(period, a, b));
        (!changed, *key)
    });
    keys
}

pub(crate) fn comparison_reading(
    period: u64,
    current: &ProductionSnapshot,
    compared: &ProductionSnapshot,
    selected_site: Option<&str>,
) -> String {
    let left = shared_accounts(current, selected_site);
    let right = shared_accounts(compared, selected_site);
    let keys = comparison_keys(period, &left, &right);
    let mut output = String::new();
    if keys.len() > 6 {
        output.push_str("Six shared capacity pools shown, with capacity changes first.\n");
    }
    for corridor_id in keys.into_iter().take(6) {
        let find = |accounts: &[&ProductionFreightCapacityAccount]| {
            accounts
                .iter()
                .position(|account| account.corridor_id == corridor_id)
        };
        let (Some(a), Some(b)) = (find(&left), find(&right)) else {
            output.push_str("Shared freight comparison unavailable: this capacity pool is not disclosed in both campaigns.\n\n");
            continue;
        };
        let (a, b) = (left[a], right[b]);
        writeln!(
            output,
            "{}\nCounts read current / compared; capacity reservations are separate from arrivals.",
            a.corridor_label
        )
        .expect("String write");
        if a.next_opening_period == b.next_opening_period {
            capacity_pair(
                &mut output,
                &format!("Next opening capacity (period {})", a.next_opening_period),
                a.next_opening_available_grams,
                b.next_opening_available_grams,
            );
        } else {
            output.push_str("Next opening capacity periods do not match.\n");
        }
        let (Some(done_a), Some(done_b)) = (&a.completed, &b.completed) else {
            if period == 0 && a.completed.is_none() && b.completed.is_none() {
                output.push_str("No completed freight reservations at foundation.\n\n");
            } else {
                output.push_str("Freight receipt unavailable for the selected period.\n\n");
            }
            continue;
        };
        if period == 0 || done_a.period != period || done_b.period != period {
            output.push_str("Freight receipt does not match the selected period.\n\n");
            continue;
        }
        let reservations: BTreeSet<_> = done_a
            .reservations
            .iter()
            .chain(&done_b.reservations)
            .map(|reservation| reservation.reservation_period)
            .collect();
        if reservations.is_empty() {
            output.push_str("No new capacity reservations in this completed period.\n");
        }
        for reservation_period in reservations {
            writeln!(output, "Reservation period {reservation_period} / 28 days")
                .expect("String write");
            let (Some(r_a), Some(r_b)) = (
                done_a
                    .reservations
                    .iter()
                    .find(|row| row.reservation_period == reservation_period),
                done_b
                    .reservations
                    .iter()
                    .find(|row| row.reservation_period == reservation_period),
            ) else {
                output.push_str("Comparable reservation account unavailable.\n");
                continue;
            };
            compare_reservation_capacity(&mut output, r_a, r_b);
            compare_orders(&mut output, r_a, r_b, current, compared, a, b);
        }
        output.push('\n');
    }
    output
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use babylon_persistence::{
        production_observation::CompletedProductionFreightCapacity,
        production_observation::ProductionFreightCapacityAccount,
        production_observation::ProductionFreightCapacityOrder,
        production_observation::ProductionFreightReservation,
        production_observation::ProductionRoute, production_observation::ProductionRouteStage,
        production_observation::ProductionSite, production_observation::ProductionSnapshot,
    };

    pub(crate) fn fixture() -> ProductionSnapshot {
        let sites = ["steel", "panels", "mill", "meals"]
            .into_iter()
            .map(|id| ProductionSite {
                id: id.into(),
                county_geoid: "26163".into(),
                name: id.into(),
                industry_code: "331".into(),
                observed_employment: None,
                inventory: Vec::new(),
                role: babylon_persistence::production_observation::ProductionSiteRole::Production,
                sector_code: "31-33".into(),
                processes: vec![
                    babylon_persistence::production_observation::ProductionProcess {
                        id: "fixture-process".into(),
                        name: "Fixture process".into(),
                        output_good_id: id.into(),
                        output_unit_id: "kg".into(),
                        output_good: id.into(),
                        output_unit: "kg".into(),
                        output_per_batch: 1,
                        available_batches: 1,
                        planned_batches: Some(1),
                        produced_batches: Some(1),
                        inputs: Vec::new(),
                        labor: Vec::new(),
                    },
                ],
            })
            .collect();
        let routes = [
            ("sheets", "steel", "panels", 600, 120),
            ("meal", "mill", "meals", 200, 40),
        ]
        .into_iter()
        .map(|(id, supplier, buyer, ordered, shipped)| ProductionRoute {
            physical_edge_ids: Vec::new(),
            distance_mm: None,
            transport_kind:
                babylon_persistence::production_observation::ProductionRouteTransport::Staged,
            grams_per_unit: 1000,
            id: id.into(),
            supplier_site_id: supplier.into(),
            buyer_site_id: buyer.into(),
            good_id: id.into(),
            unit_id: "kg".into(),
            good: id.into(),
            unit: "kg".into(),
            travel_periods: 1,
            ordered,
            shipped,
            delivered: 0,
            lost: 0,
            realized: 0,
            backlog: ordered - shipped,
            stages: vec![ProductionRouteStage {
                stage_index: 0,
                capacity_ids: vec!["pool".into()],
                travel_periods: 1,
            }],
        })
        .collect();
        ProductionSnapshot {
            content_authority_sha256: "a".repeat(64),
            road_source: None,
            physical_edges: Vec::new(),
            merchant_handling_accounts: Vec::new(),
            final_demand_accounts: Vec::new(),
            scenario_label: "Shared freight — constrained".into(),
            horizon_period: 16,
            sites,
            routes,
            freight: Vec::new(),
            events: Vec::new(),
            provenance: Vec::new(),
            material_balance: None,
            labor_accounts: Vec::new(),
            staffing_accounts: Vec::new(),
            observed_contexts: Vec::new(),
            process_attributions: Vec::new(),
            freight_capacity_accounts: vec![capacity_fixture()],
        }
    }

    fn capacity_fixture() -> ProductionFreightCapacityAccount {
        ProductionFreightCapacityAccount {
            corridor_id: "pool".into(),
            corridor_label: "Designed regional freight pool".into(),
            kind: babylon_persistence::production_observation::ProductionCapacityKind::Transport,
            merchant_site_ids: Vec::new(),
            route_ids: vec!["sheets".into(), "meal".into()],
            next_opening_period: 2,
            next_opening_available_grams: 160_000,
            completed: Some(CompletedProductionFreightCapacity {
                period: 1,
                reservations: vec![ProductionFreightReservation {
                    reservation_period: 1,
                    opening_available_grams: 160_000,
                    newly_reserved_grams: 160_000,
                    remaining_available_grams: 0,
                    orders: [("sheets", 600, 120), ("meal", 200, 40)]
                        .into_iter()
                        .map(
                            |(id, requested, dispatched)| ProductionFreightCapacityOrder {
                                order_id: format!("order-{id}"),
                                route_id: Some(id.into()),
                                kind: babylon_persistence::production_observation::ProductionOutboundKind::Delivery,
                                supplier_site_id: if id == "sheets" {
                                    "steel".into()
                                } else {
                                    "mill".into()
                                },
                                grams_per_unit: 1000,
                                requested_grams: u128::from(requested) * 1000,
                                reserved_grams: dispatched * 1000,
                                good_id: id.into(),
                                unit_id: "kg".into(),
                                requested,
                                dispatched,
                                remaining_unshipped: requested - dispatched,
                            },
                        )
                        .collect(),
                }],
            }),
        }
    }

    #[test]
    fn capacity_briefs_preserve_every_gram_as_exact_kilograms() {
        let mut account = capacity_fixture();
        account.completed = None;
        for (grams, expected) in [
            (0, "0 kg"),
            (1, "0.001 kg"),
            (10, "0.01 kg"),
            (100, "0.1 kg"),
            (1_001, "1.001 kg"),
            (1_010, "1.01 kg"),
            (1_100, "1.1 kg"),
            (160_000, "160 kg"),
            (800_000, "800 kg"),
            (u64::MAX, "18446744073709551.615 kg"),
        ] {
            account.next_opening_available_grams = grams;
            assert!(account_brief(&account).contains(&format!("Period 2 opening: {expected}")));
        }
    }

    #[test]
    fn capacity_readings_preserve_small_residuals_and_native_commodity_units() {
        let mut snapshot = fixture();
        snapshot.routes[0].unit_id = "panel".into();
        snapshot.routes[0].unit = "panels".into();
        let account = &mut snapshot.freight_capacity_accounts[0];
        account.next_opening_available_grams = 1;
        let reservation = &mut account.completed.as_mut().unwrap().reservations[0];
        reservation.newly_reserved_grams = 159_999;
        reservation.remaining_available_grams = 1;
        reservation.orders[0].unit_id = "panel".into();
        let brief = account_brief(account);
        assert!(brief.contains("160 kg opening · 159.999 kg reserved · 0.001 kg remaining"));
        let text = account_reading(&snapshot.freight_capacity_accounts[0], &snapshot);
        assert!(text.contains("Newly reserved 159.999 kg\nRemaining 0.001 kg"));
        assert!(text.contains("Next opening (period 2): 0.001 kg available"));
        assert!(text.contains("Requested 600 panels | dispatched 120 panels"));
        let comparison = comparison_reading(1, &snapshot, &snapshot, None);
        assert!(comparison.contains("Next opening capacity (period 2): 0.001 kg / 0.001 kg"));
        assert!(comparison.contains("Remaining capacity: 0.001 kg / 0.001 kg"));
        assert!(comparison.contains("Dispatched: 120 / 120 panels"));
    }

    #[test]
    fn shared_pool_is_counted_once_and_names_both_competing_chains() {
        let snapshot = fixture();
        let accounts = shared_accounts(&snapshot, Some("panels"));
        assert_eq!(accounts.len(), 1);
        let text = account_reading(accounts[0], &snapshot);
        assert_eq!(text.matches("Designed regional freight pool").count(), 1);
        assert_eq!(text.lines().next(), Some("Designed regional freight pool"));
        assert!(text.contains("Opening 160 kg\nNewly reserved 160 kg\nRemaining 0 kg"));
        assert_eq!(text.matches("steel -> panels / sheets").count(), 1);
        assert_eq!(text.matches("mill -> meals / meal").count(), 1);
        assert!(text.contains("Requested 600 kg | dispatched 120 kg\nUnshipped 480 kg"));
        assert!(text.contains("Requested 200 kg | dispatched 40 kg\nUnshipped 160 kg"));
        assert!(text.contains("Reservations use capacity; goods arrive after travel"));
    }

    #[test]
    fn unmatched_reservation_reports_unavailable_detail_without_disclosing_order_data() {
        let mut snapshot = fixture();
        let orders = &mut snapshot.freight_capacity_accounts[0]
            .completed
            .as_mut()
            .unwrap()
            .reservations[0]
            .orders;
        let mut unavailable = orders[0].clone();
        unavailable.order_id = "private-order".into();
        unavailable.route_id = Some("private-route".into());
        unavailable.requested = 987_654;
        orders.push(unavailable);
        let reading = account_reading(&snapshot.freight_capacity_accounts[0], &snapshot);
        assert!(reading.contains("Reservation route detail unavailable in this observation."));
        let comparison = comparison_reading(1, &snapshot, &snapshot, None);
        assert!(comparison.contains("Comparable route endpoints unavailable."));
        for text in [reading, comparison] {
            assert!(!text.contains("private-"));
            assert!(!text.contains("987654"));
            assert!(text.contains("steel -> panels / sheets"));
            assert!(text.contains("mill -> meals / meal"));
        }
    }

    #[test]
    fn foundation_and_completed_zero_have_different_capacity_readings() {
        let mut snapshot = fixture();
        snapshot.freight_capacity_accounts[0].completed = None;
        snapshot.freight_capacity_accounts[0].next_opening_period = 1;
        let text = account_reading(&snapshot.freight_capacity_accounts[0], &snapshot);
        assert!(text.contains("No completed freight reservations at foundation"));
        assert!(text.contains("Participant: steel -> panels / sheets"));
        assert!(text.contains("Participant: mill -> meals / meal"));
        assert!(!text.contains("Newly reserved 0"));
        let mut snapshot = fixture();
        let reservation = &mut snapshot.freight_capacity_accounts[0]
            .completed
            .as_mut()
            .unwrap()
            .reservations[0];
        reservation.newly_reserved_grams = 0;
        reservation.remaining_available_grams = 160_000;
        for order in &mut reservation.orders {
            order.dispatched = 0;
            order.remaining_unshipped = order.requested;
        }
        let text = account_reading(&snapshot.freight_capacity_accounts[0], &snapshot);
        assert!(text.contains("Newly reserved 0 kg\nRemaining 160 kg"));
        assert!(!text.contains("at foundation"));
    }

    #[test]
    fn competitor_navigation_exposes_disclosed_peers_without_supplier_edges() {
        let mut snapshot = fixture();
        let peers: Vec<_> = competitor_sites("panels", &snapshot)
            .into_iter()
            .map(|site| site.id.as_str())
            .collect();
        assert_eq!(peers, ["meals", "mill"]);
        assert_eq!(
            crate::production_brief::dependency_sites(&snapshot.sites[1], &snapshot).len(),
            1
        );
        snapshot.sites.retain(|site| site.id != "mill");
        assert!(competitor_sites("panels", &snapshot).is_empty());
        assert!(shared_accounts(&snapshot, Some("panels")).is_empty());
    }

    #[test]
    fn comparison_joins_capacity_and_order_identity_and_preserves_reservation_period() {
        let current = fixture();
        let mut other = fixture();
        other.freight_capacity_accounts[0].corridor_label = "Different display label".into();
        let completed = other.freight_capacity_accounts[0]
            .completed
            .as_mut()
            .unwrap();
        let reservation = &mut completed.reservations[0];
        reservation.opening_available_grams = 800_000;
        reservation.newly_reserved_grams = 400_000;
        reservation.remaining_available_grams = 400_000;
        reservation.orders[0].dispatched = 320;
        reservation.orders[0].remaining_unshipped = 280;
        reservation.orders[1].dispatched = 80;
        reservation.orders[1].remaining_unshipped = 120;
        reservation.orders.reverse();
        let text = comparison_reading(1, &current, &other, None);
        assert_eq!(text.matches("Designed regional freight pool").count(), 1);
        assert!(text.contains("Reservation period 1"));
        assert!(text.contains("Opening capacity: 160 kg / 800 kg"));
        assert!(text.contains("Dispatched: 120 / 320 kg"));
        assert!(text.contains("Dispatched: 40 / 80 kg"));
        other.freight_capacity_accounts[0]
            .completed
            .as_mut()
            .unwrap()
            .period = 2;
        let text = comparison_reading(1, &current, &other, None);
        assert!(text.contains("Freight receipt does not match the selected period"));
        assert!(!text.contains("Dispatched:"));
    }

    #[test]
    fn bounded_comparison_keeps_changed_capacity_visible_among_many_road_pools() {
        for period in [0, 1] {
            let mut current = fixture();
            current.freight_capacity_accounts = (0..8)
                .map(|index| {
                    let mut account = capacity_fixture();
                    account.corridor_id = format!("pool-{index}");
                    account.corridor_label = format!("Road pool {index}");
                    if period == 0 {
                        account.completed = None;
                        account.next_opening_period = 1;
                    }
                    account
                })
                .collect();
            for route in &mut current.routes {
                route.stages[0].capacity_ids = current
                    .freight_capacity_accounts
                    .iter()
                    .map(|account| account.corridor_id.clone())
                    .collect();
            }
            let mut other = current.clone();
            let changed = &mut other.freight_capacity_accounts[7];
            if period == 0 {
                changed.next_opening_available_grams = 800_000;
            } else {
                let reservation = &mut changed.completed.as_mut().unwrap().reservations[0];
                reservation.opening_available_grams = 800_000;
                reservation.remaining_available_grams = 640_000;
            }
            let text = comparison_reading(period, &current, &other, Some("panels"));
            assert!(text.contains("Road pool 7\n"));
            assert!(text.contains("160 kg / 800 kg"));
            assert_eq!(text.matches("Counts read current / compared").count(), 6);
            current.freight_capacity_accounts.reverse();
            other.freight_capacity_accounts.reverse();
            other.routes.reverse();
            assert_eq!(
                text,
                comparison_reading(period, &current, &other, Some("panels"))
            );
        }
    }
}
