//! Readings of authenticated shared freight capacity, separate from material arrivals.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use babylon_persistence::{
    ProductionFreightCapacityAccountV1, ProductionFreightCapacityOrderV1,
    ProductionFreightReservationV1, ProductionRouteV1, ProductionSiteV1, ProductionSnapshotV1,
};

fn participating_routes<'a>(
    account: &ProductionFreightCapacityAccountV1,
    snapshot: &'a ProductionSnapshotV1,
) -> Vec<&'a ProductionRouteV1> {
    let mut routes: Vec<_> = snapshot
        .routes
        .iter()
        .filter(|route| {
            account.route_ids.contains(&route.id)
                && route.unit_id == account.unit_id
                && route
                    .corridor_legs
                    .iter()
                    .any(|leg| leg.corridor_id == account.corridor_id)
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
    snapshot: &'a ProductionSnapshotV1,
    selected_site: Option<&str>,
) -> Vec<&'a ProductionFreightCapacityAccountV1> {
    let mut accounts: Vec<_> = snapshot
        .freight_capacity_accounts
        .iter()
        .filter(|account| {
            let routes = participating_routes(account, snapshot);
            routes.len() > 1
                && selected_site.is_none_or(|selected| {
                    routes.iter().any(|route| {
                        route.supplier_site_id == selected || route.buyer_site_id == selected
                    })
                })
        })
        .collect();
    accounts.sort_by(|a, b| (&a.corridor_id, &a.unit_id).cmp(&(&b.corridor_id, &b.unit_id)));
    accounts
}

fn route_label(route: &ProductionRouteV1, snapshot: &ProductionSnapshotV1) -> String {
    let name = |id: &str| {
        snapshot
            .sites
            .iter()
            .find(|site| site.id == id)
            .map(|site| site.name.as_str())
    };
    match (name(&route.supplier_site_id), name(&route.buyer_site_id)) {
        (Some(supplier), Some(buyer)) => format!("{supplier} -> {buyer} / {}", route.good),
        _ => "Route endpoints unavailable in this observation".into(),
    }
}

pub(crate) fn account_reading(
    account: &ProductionFreightCapacityAccountV1,
    snapshot: &ProductionSnapshotV1,
) -> String {
    let mut output = format!("{}\n", account.corridor_label);
    let routes = participating_routes(account, snapshot);
    output.push_str("SHARED BY\n");
    for route in &routes {
        writeln!(output, "{}", route_label(route, snapshot)).expect("String write");
    }
    if let Some(completed) = &account.completed {
        writeln!(output, "COMMITTED DISPATCH / PERIOD {}", completed.period).expect("String write");
        for reservation in &completed.reservations {
            writeln!(output,
                "Reservation period {} / 28 days\nOpening {} {} | newly reserved {} {} | remaining {} {}",
                reservation.reservation_period, reservation.opening_available, account.unit,
                reservation.newly_reserved, account.unit, reservation.remaining_available, account.unit,
            ).expect("String write");
            for order in &reservation.orders {
                let Some(route) = routes.iter().find(|route| {
                    route.id == order.route_id
                        && route.good_id == order.good_id
                        && route.unit_id == order.unit_id
                }) else {
                    continue;
                };
                writeln!(
                    output,
                    "{}\nRequested {} {} | dispatched {} {} | unshipped {} {}",
                    route_label(route, snapshot),
                    order.requested,
                    account.unit,
                    order.dispatched,
                    account.unit,
                    order.remaining_unshipped,
                    account.unit,
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
    writeln!(output, "Next opening (period {}): {} {} available\nReservations use capacity; goods arrive after travel. Read arrivals, output and workforce in each participant's Readings.",
        account.next_opening_period, account.next_opening_available, account.unit,
    ).expect("String write");
    output
}

/// Competition links are separate from the supplier/buyer relation graph.
pub(crate) fn competitor_sites<'a>(
    site_id: &str,
    snapshot: &'a ProductionSnapshotV1,
) -> Vec<&'a ProductionSiteV1> {
    let ids: BTreeSet<_> = shared_accounts(snapshot, Some(site_id))
        .into_iter()
        .flat_map(|account| participating_routes(account, snapshot))
        .filter(|route| route.supplier_site_id != site_id && route.buyer_site_id != site_id)
        .flat_map(|route| [&route.supplier_site_id, &route.buyer_site_id])
        .collect();
    ids.into_iter()
        .filter_map(|id| snapshot.sites.iter().find(|site| site.id == *id))
        .collect()
}

fn order_key(order: &ProductionFreightCapacityOrderV1) -> (&str, &str, &str, &str) {
    (
        &order.order_id,
        &order.route_id,
        &order.good_id,
        &order.unit_id,
    )
}

fn pair(output: &mut String, label: &str, current: u64, compared: u64, unit: &str) {
    writeln!(output, "{label}: {current} / {compared} {unit}").expect("String write");
}

fn compare_orders(
    output: &mut String,
    r_a: &ProductionFreightReservationV1,
    r_b: &ProductionFreightReservationV1,
    current: &ProductionSnapshotV1,
    compared: &ProductionSnapshotV1,
    a: &ProductionFreightCapacityAccountV1,
    b: &ProductionFreightCapacityAccountV1,
) {
    let orders: BTreeSet<_> = r_a
        .orders
        .iter()
        .chain(&r_b.orders)
        .map(order_key)
        .collect();
    for key in orders {
        let (Some(o_a), Some(o_b)) = (
            r_a.orders.iter().find(|order| order_key(order) == key),
            r_b.orders.iter().find(|order| order_key(order) == key),
        ) else {
            output.push_str("Comparable route order unavailable.\n");
            continue;
        };
        let route = participating_routes(a, current).into_iter().find(|route| {
            route.id == o_a.route_id && route.good_id == o_a.good_id && route.unit_id == o_a.unit_id
        });
        let Some(route) = route else {
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
        pair(output, "Requested", o_a.requested, o_b.requested, &a.unit);
        pair(
            output,
            "Dispatched",
            o_a.dispatched,
            o_b.dispatched,
            &a.unit,
        );
        pair(
            output,
            "Remaining unshipped",
            o_a.remaining_unshipped,
            o_b.remaining_unshipped,
            &a.unit,
        );
        pair(
            output,
            "Arrived to date",
            route.delivered,
            other_route.delivered,
            &a.unit,
        );
    }
}

pub(crate) fn comparison_reading(
    period: u64,
    current: &ProductionSnapshotV1,
    compared: &ProductionSnapshotV1,
) -> String {
    let left = shared_accounts(current, None);
    let right = shared_accounts(compared, None);
    let keys: BTreeSet<_> = left
        .iter()
        .chain(&right)
        .map(|account| (&account.corridor_id, &account.unit_id))
        .collect();
    let mut output = String::new();
    for (corridor_id, unit_id) in keys {
        let find = |accounts: &[&ProductionFreightCapacityAccountV1]| {
            accounts.iter().position(|account| {
                &account.corridor_id == corridor_id && &account.unit_id == unit_id
            })
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
            pair(
                &mut output,
                &format!("Next opening capacity (period {})", a.next_opening_period),
                a.next_opening_available,
                b.next_opening_available,
                &a.unit,
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
            for (label, current_value, compared_value) in [
                (
                    "Opening capacity",
                    r_a.opening_available,
                    r_b.opening_available,
                ),
                ("Newly reserved", r_a.newly_reserved, r_b.newly_reserved),
                (
                    "Remaining capacity",
                    r_a.remaining_available,
                    r_b.remaining_available,
                ),
            ] {
                pair(&mut output, label, current_value, compared_value, &a.unit);
            }
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
        CompletedProductionFreightCapacityV1, ProductionFreightCapacityAccountV1,
        ProductionFreightCapacityOrderV1, ProductionFreightReservationV1,
        ProductionRouteCorridorLegV1, ProductionRouteV1, ProductionSiteV1, ProductionSnapshotV1,
    };

    pub(crate) fn fixture() -> ProductionSnapshotV1 {
        let sites = ["steel", "panels", "mill", "meals"]
            .into_iter()
            .map(|id| ProductionSiteV1 {
                id: id.into(),
                county_geoid: "26163".into(),
                name: id.into(),
                industry_code: "331".into(),
                observed_employment: None,
                output_good_id: id.into(),
                output_unit_id: "kg".into(),
                output_good: id.into(),
                output_unit: "kg".into(),
                output_per_batch: 1,
                available_batches: 1,
                planned_batches: Some(1),
                produced_batches: Some(1),
                inventory: Vec::new(),
                inputs: Vec::new(),
                labor: Vec::new(),
            })
            .collect();
        let routes = [
            ("sheets", "steel", "panels", 600, 120),
            ("meal", "mill", "meals", 200, 40),
        ]
        .into_iter()
        .map(
            |(id, supplier, buyer, ordered, shipped)| ProductionRouteV1 {
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
                corridor_legs: vec![ProductionRouteCorridorLegV1 {
                    leg_index: 0,
                    corridor_id: "pool".into(),
                    travel_periods: 1,
                }],
            },
        )
        .collect();
        ProductionSnapshotV1 {
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
            freight_capacity_accounts: vec![ProductionFreightCapacityAccountV1 {
                corridor_id: "pool".into(),
                corridor_label: "Designed regional freight pool".into(),
                unit_id: "kg".into(),
                unit: "kg".into(),
                route_ids: vec!["sheets".into(), "meal".into()],
                next_opening_period: 2,
                next_opening_available: 160,
                completed: Some(CompletedProductionFreightCapacityV1 {
                    period: 1,
                    reservations: vec![ProductionFreightReservationV1 {
                        reservation_period: 1,
                        opening_available: 160,
                        newly_reserved: 160,
                        remaining_available: 0,
                        orders: [("sheets", 600, 120), ("meal", 200, 40)]
                            .into_iter()
                            .map(
                                |(id, requested, dispatched)| ProductionFreightCapacityOrderV1 {
                                    order_id: format!("order-{id}"),
                                    route_id: id.into(),
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
            }],
        }
    }

    #[test]
    fn shared_pool_is_counted_once_and_names_both_competing_chains() {
        let snapshot = fixture();
        let accounts = shared_accounts(&snapshot, Some("panels"));
        assert_eq!(accounts.len(), 1);
        let text = account_reading(accounts[0], &snapshot);
        assert_eq!(text.matches("Designed regional freight pool").count(), 1);
        assert_eq!(text.lines().next(), Some("Designed regional freight pool"));
        assert!(text.contains("Opening 160 kg | newly reserved 160 kg | remaining 0 kg"));
        assert!(text.contains("steel -> panels / sheets"));
        assert!(text.contains("mill -> meals / meal"));
        assert!(text.contains("Requested 600 kg | dispatched 120 kg | unshipped 480 kg"));
        assert!(text.contains("Requested 200 kg | dispatched 40 kg | unshipped 160 kg"));
        assert!(text.contains("Reservations use capacity; goods arrive after travel"));
    }

    #[test]
    fn foundation_and_completed_zero_have_different_capacity_readings() {
        let mut snapshot = fixture();
        snapshot.freight_capacity_accounts[0].completed = None;
        snapshot.freight_capacity_accounts[0].next_opening_period = 1;
        let text = account_reading(&snapshot.freight_capacity_accounts[0], &snapshot);
        assert!(text.contains("No completed freight reservations at foundation"));
        assert!(!text.contains("newly reserved 0"));
        let mut snapshot = fixture();
        let reservation = &mut snapshot.freight_capacity_accounts[0]
            .completed
            .as_mut()
            .unwrap()
            .reservations[0];
        reservation.newly_reserved = 0;
        reservation.remaining_available = 160;
        for order in &mut reservation.orders {
            order.dispatched = 0;
            order.remaining_unshipped = order.requested;
        }
        let text = account_reading(&snapshot.freight_capacity_accounts[0], &snapshot);
        assert!(text.contains("newly reserved 0 kg | remaining 160 kg"));
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
        reservation.opening_available = 800;
        reservation.newly_reserved = 400;
        reservation.remaining_available = 400;
        reservation.orders[0].dispatched = 320;
        reservation.orders[0].remaining_unshipped = 280;
        reservation.orders[1].dispatched = 80;
        reservation.orders[1].remaining_unshipped = 120;
        reservation.orders.reverse();
        let text = comparison_reading(1, &current, &other);
        assert_eq!(text.matches("Designed regional freight pool").count(), 1);
        assert!(text.contains("Reservation period 1"));
        assert!(text.contains("Opening capacity: 160 / 800 kg"));
        assert!(text.contains("Dispatched: 120 / 320 kg"));
        assert!(text.contains("Dispatched: 40 / 80 kg"));
        other.freight_capacity_accounts[0]
            .completed
            .as_mut()
            .unwrap()
            .period = 2;
        let text = comparison_reading(1, &current, &other);
        assert!(text.contains("Freight receipt does not match the selected period"));
        assert!(!text.contains("Dispatched:"));
    }
}
