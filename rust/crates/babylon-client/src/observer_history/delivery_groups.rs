//! Delivery summaries over one already-scoped committed observation.
//! Individual events remain evidence. A summary represents an order's activity
//! during a week, never a particular freight lot, a whole completed order, or money.

use std::collections::BTreeMap;

use babylon_persistence::{
    ProductionDeliveryEvidenceV1, ProductionDeliveryStageV1, ProductionEventV1, ProductionRouteV1,
    ProductionSiteV1, ProductionSnapshotV1,
};

/// Wrap this key in the current `ObservationContext` before storing expansion.
/// A key identifies evidence inside an observation, not a read capability.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct DeliveryGroupKey {
    pub week: u64,
    pub receipt_digest: String,
    pub order_id: String,
    pub route_id: String,
    pub good_id: String,
    pub unit_id: String,
}

impl DeliveryGroupKey {
    fn new(event: &ProductionEventV1, evidence: &ProductionDeliveryEvidenceV1) -> Self {
        Self {
            week: event.week,
            receipt_digest: event.receipt_digest.clone(),
            order_id: evidence.order_id.clone(),
            route_id: evidence.route_id.clone(),
            good_id: evidence.good_id.clone(),
            unit_id: evidence.unit_id.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeliveryStageTotal {
    pub quantity: u64,
    pub evidence_entries: usize,
}

/// Every member remains in its original sequence, including equal quantities.
/// Separate stage totals prevent counting one delivery three times.
#[derive(Debug)]
pub(super) struct DeliveryGroup<'a> {
    pub key: DeliveryGroupKey,
    pub route: &'a ProductionRouteV1,
    pub supplier: &'a ProductionSiteV1,
    pub buyer: &'a ProductionSiteV1,
    pub events: Vec<&'a ProductionEventV1>,
    pub arrivals: Option<DeliveryStageTotal>,
    pub deliveries: Option<DeliveryStageTotal>,
    pub realizations: Option<DeliveryStageTotal>,
}

impl<'a> DeliveryGroup<'a> {
    fn new(key: DeliveryGroupKey, route: DisclosedRoute<'a>) -> Self {
        Self {
            key,
            route: route.route,
            supplier: route.supplier,
            buyer: route.buyer,
            events: Vec::new(),
            arrivals: None,
            deliveries: None,
            realizations: None,
        }
    }

    fn push(
        &mut self,
        event: &'a ProductionEventV1,
        evidence: &ProductionDeliveryEvidenceV1,
    ) -> Result<(), DeliveryGroupingError> {
        let total = match evidence.stage {
            ProductionDeliveryStageV1::Arrival => &mut self.arrivals,
            ProductionDeliveryStageV1::Delivery => &mut self.deliveries,
            ProductionDeliveryStageV1::QuantityRealization => &mut self.realizations,
        };
        let prior = total.unwrap_or(DeliveryStageTotal {
            quantity: 0,
            evidence_entries: 0,
        });
        *total = Some(DeliveryStageTotal {
            quantity: prior
                .quantity
                .checked_add(evidence.quantity)
                .ok_or(DeliveryGroupingError::QuantityRange)?,
            evidence_entries: prior
                .evidence_entries
                .checked_add(1)
                .ok_or(DeliveryGroupingError::QuantityRange)?,
        });
        self.events.push(event);
        Ok(())
    }

    pub(super) fn headline(&self) -> String {
        if self.deliveries.is_some() {
            format!("{} delivered to {}", self.route.good, self.buyer.name)
        } else if self.arrivals.is_some() {
            format!("{} arrived at {}", self.route.good, self.buyer.name)
        } else {
            format!(
                "Quantity realization recorded for {} at {}",
                self.route.good, self.buyer.name
            )
        }
    }

    /// Exact quantities are optional detail. Missing evidence is not zero.
    /// Each digest belongs to the complete tick receipt family, not one entry.
    pub(super) fn details(&self) -> String {
        let stage = |label: &str, total: Option<DeliveryStageTotal>| match total {
            Some(total) => format!("{label}: {} {}", total.quantity, self.route.unit),
            None => format!("{label}: no evidence entry"),
        };
        format!(
            "Week {} / {} -> {}\n{}\n{}\n{}\n{} evidence entries",
            self.key.week,
            self.supplier.name,
            self.buyer.name,
            stage("Arrived", self.arrivals),
            stage("Delivered", self.deliveries),
            stage("Quantity realized", self.realizations),
            self.events.len()
        )
    }
}

#[derive(Debug)]
pub(super) enum DeliveryLogEntry<'a> {
    Event(&'a ProductionEventV1),
    Delivery(Box<DeliveryGroup<'a>>),
}

impl DeliveryLogEntry<'_> {
    pub(super) fn week(&self) -> u64 {
        match self {
            Self::Event(event) => event.week,
            Self::Delivery(group) => group.key.week,
        }
    }
}

#[derive(Debug)]
pub(super) struct DeliveryLog<'a> {
    /// Newest first, capped only after complete groups have formed.
    pub entries: Vec<DeliveryLogEntry<'a>>,
    pub total_entries: usize,
    pub evidence_entries: usize,
}

impl DeliveryLog<'_> {
    pub(super) fn contains_group(&self, key: &DeliveryGroupKey) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry, DeliveryLogEntry::Delivery(group) if group.key == *key))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeliveryGroupingError {
    RouteUnavailable,
    SiteUnavailable,
    IdentityMismatch,
    QuantityRange,
}

impl std::fmt::Display for DeliveryGroupingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::RouteUnavailable => "Delivery route evidence is unavailable or ambiguous.",
            Self::SiteUnavailable => "Delivery endpoint evidence is unavailable or ambiguous.",
            Self::IdentityMismatch => "Delivery evidence identities do not agree.",
            Self::QuantityRange => "Delivery quantities exceed their exact positive integer range.",
        })
    }
}

struct DisclosedRoute<'a> {
    route: &'a ProductionRouteV1,
    supplier: &'a ProductionSiteV1,
    buyer: &'a ProductionSiteV1,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OrderReceiptKey<'a> {
    week: u64,
    receipt_digest: &'a str,
    order_id: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct OrderPrincipal<'a> {
    route: &'a str,
    good: &'a str,
    unit: &'a str,
    supplier: &'a str,
    buyer: &'a str,
}

/// A contradictory principal cannot become a second apparently valid group.
#[derive(Default)]
struct OrderBindings<'a> {
    principals: BTreeMap<OrderReceiptKey<'a>, OrderPrincipal<'a>>,
}

impl<'a> OrderBindings<'a> {
    fn record(
        &mut self,
        event: &'a ProductionEventV1,
        evidence: &'a ProductionDeliveryEvidenceV1,
        disclosed: &DisclosedRoute<'a>,
    ) -> Result<(), DeliveryGroupingError> {
        let key = OrderReceiptKey {
            week: event.week,
            receipt_digest: &event.receipt_digest,
            order_id: &evidence.order_id,
        };
        let principal = OrderPrincipal {
            route: &disclosed.route.id,
            good: &disclosed.route.good_id,
            unit: &disclosed.route.unit_id,
            supplier: &disclosed.supplier.id,
            buyer: &disclosed.buyer.id,
        };
        if *self.principals.entry(key).or_insert(principal) != principal {
            return Err(DeliveryGroupingError::IdentityMismatch);
        }
        Ok(())
    }
}

struct DisclosedRoutes<'a> {
    routes: BTreeMap<&'a str, &'a ProductionRouteV1>,
    sites: BTreeMap<&'a str, &'a ProductionSiteV1>,
}

impl<'a> DisclosedRoutes<'a> {
    fn new(snapshot: &'a ProductionSnapshotV1) -> Result<Self, DeliveryGroupingError> {
        let mut routes = BTreeMap::new();
        for route in &snapshot.routes {
            if routes.insert(route.id.as_str(), route).is_some() {
                return Err(DeliveryGroupingError::RouteUnavailable);
            }
        }
        let mut sites = BTreeMap::new();
        for site in &snapshot.sites {
            if sites.insert(site.id.as_str(), site).is_some() {
                return Err(DeliveryGroupingError::SiteUnavailable);
            }
        }
        Ok(Self { routes, sites })
    }

    fn resolve(
        &self,
        event: &ProductionEventV1,
        evidence: &ProductionDeliveryEvidenceV1,
    ) -> Result<DisclosedRoute<'a>, DeliveryGroupingError> {
        if evidence.quantity == 0 {
            return Err(DeliveryGroupingError::QuantityRange);
        }
        let route = *self
            .routes
            .get(evidence.route_id.as_str())
            .ok_or(DeliveryGroupingError::RouteUnavailable)?;
        if event.week == 0
            || evidence.order_id.is_empty()
            || event.receipt_digest.is_empty()
            || route.good_id != evidence.good_id
            || route.unit_id != evidence.unit_id
        {
            return Err(DeliveryGroupingError::IdentityMismatch);
        }
        let supplier = *self
            .sites
            .get(route.supplier_site_id.as_str())
            .ok_or(DeliveryGroupingError::SiteUnavailable)?;
        let buyer = *self
            .sites
            .get(route.buyer_site_id.as_str())
            .ok_or(DeliveryGroupingError::SiteUnavailable)?;
        let mut subjects = event
            .subject_site_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        subjects.sort_unstable();
        let mut endpoints = [supplier.id.as_str(), buyer.id.as_str()];
        endpoints.sort_unstable();
        if subjects != endpoints {
            return Err(DeliveryGroupingError::IdentityMismatch);
        }
        Ok(DisclosedRoute {
            route,
            supplier,
            buyer,
        })
    }
}

/// The caller must pass only `ObserverFrame::for_session` production data.
/// No endpoint, total, or label comes from a cache or another observation.
/// Stage metadata, not descriptions or labels, determines membership.
pub(super) fn delivery_log_entries(
    snapshot: &ProductionSnapshotV1,
    limit: usize,
) -> Result<DeliveryLog<'_>, DeliveryGroupingError> {
    let routes = DisclosedRoutes::new(snapshot)?;
    let mut orders = OrderBindings::default();
    let mut groups: BTreeMap<DeliveryGroupKey, (usize, DeliveryGroup<'_>)> = BTreeMap::new();
    let mut entries = Vec::new();
    for (position, event) in snapshot.events.iter().enumerate() {
        let Some(evidence) = &event.delivery_evidence else {
            entries.push((position, DeliveryLogEntry::Event(event)));
            continue;
        };
        let route = routes.resolve(event, evidence)?;
        orders.record(event, evidence, &route)?;
        let key = DeliveryGroupKey::new(event, evidence);
        let (last_position, group) = groups
            .entry(key.clone())
            .or_insert_with(|| (position, DeliveryGroup::new(key, route)));
        group.push(event, evidence)?;
        *last_position = position;
    }
    entries.extend(
        groups
            .into_values()
            .map(|(position, group)| (position, DeliveryLogEntry::Delivery(Box::new(group)))),
    );
    entries.sort_unstable_by_key(|(position, _)| std::cmp::Reverse(*position));
    let total_entries = entries.len();
    entries.truncate(limit);
    Ok(DeliveryLog {
        entries: entries.into_iter().map(|(_, entry)| entry).collect(),
        total_entries,
        evidence_entries: snapshot.events.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(id: &str, name: &str) -> ProductionSiteV1 {
        ProductionSiteV1 {
            id: id.into(),
            county_geoid: "26163".into(),
            name: name.into(),
            industry_code: "331".into(),
            observed_employment: None,
            output_good_id: "sheet".into(),
            output_unit_id: "tonnes".into(),
            output_good: "Sheet metal".into(),
            output_unit: "tonnes".into(),
            output_per_batch: 1,
            available_batches: 1,
            planned_batches: None,
            produced_batches: None,
            inventory: vec![],
            inputs: vec![],
            labor: vec![],
        }
    }

    fn snapshot() -> ProductionSnapshotV1 {
        ProductionSnapshotV1 {
            scenario_label: "Designed delivery evidence fixture".into(),
            horizon_week: 16,
            sites: vec![
                site("supplier", "Wayne metal"),
                site("buyer", "Macomb parts"),
            ],
            routes: vec![ProductionRouteV1 {
                id: "route".into(),
                supplier_site_id: "supplier".into(),
                buyer_site_id: "buyer".into(),
                good_id: "sheet".into(),
                unit_id: "tonnes".into(),
                good: "Sheet metal".into(),
                unit: "tonnes".into(),
                travel_weeks: 1,
                ordered: 1_000,
                shipped: 30,
                delivered: 12,
                lost: 0,
                realized: 12,
                backlog: 970,
            }],
            freight: vec![],
            events: vec![],
            material_balance: None,
            labor_accounts: vec![],
            observed_contexts: vec![],
            process_attributions: vec![],
            provenance: vec![],
        }
    }

    fn event(
        id: &str,
        stage: ProductionDeliveryStageV1,
        quantity: u64,
        order: &str,
        week: u64,
    ) -> ProductionEventV1 {
        ProductionEventV1 {
            id: id.into(),
            week,
            subject_site_ids: vec!["supplier".into(), "buyer".into()],
            kind: "Display text is not an identity".into(),
            description: "Preserve this original committed description.".into(),
            receipt_digest: format!("receipt-{week}"),
            delivery_evidence: Some(ProductionDeliveryEvidenceV1 {
                stage,
                order_id: order.into(),
                route_id: "route".into(),
                good_id: "sheet".into(),
                unit_id: "tonnes".into(),
                quantity,
            }),
        }
    }

    fn triplet(order: &str, week: u64, quantity: u64) -> Vec<ProductionEventV1> {
        [
            ProductionDeliveryStageV1::Arrival,
            ProductionDeliveryStageV1::Delivery,
            ProductionDeliveryStageV1::QuantityRealization,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, stage)| {
            event(
                &format!("{order}-{week}-{index}"),
                stage,
                quantity,
                order,
                week,
            )
        })
        .collect()
    }

    fn group<'a>(log: &'a DeliveryLog<'a>, index: usize) -> &'a DeliveryGroup<'a> {
        let DeliveryLogEntry::Delivery(group) = &log.entries[index] else {
            panic!("expected a delivery group")
        };
        group
    }

    #[test]
    fn one_delivery_keeps_three_original_entries_without_tripling_quantity() {
        let mut snapshot = snapshot();
        snapshot.events = triplet("order", 3, 10);
        let log = delivery_log_entries(&snapshot, 160).unwrap();
        assert_eq!((log.total_entries, log.evidence_entries), (1, 3));
        let group = group(&log, 0);
        assert_eq!(group.events.len(), 3);
        for (actual, original) in group.events.iter().zip(&snapshot.events) {
            assert!(std::ptr::eq(*actual, original));
        }
        for stage in [group.arrivals, group.deliveries, group.realizations] {
            assert_eq!(
                stage,
                Some(DeliveryStageTotal {
                    quantity: 10,
                    evidence_entries: 1
                })
            );
        }
        assert_eq!(group.headline(), "Sheet metal delivered to Macomb parts");
        assert_eq!(
            group.details(),
            "Week 3 / Wayne metal -> Macomb parts\nArrived: 10 tonnes\nDelivered: 10 tonnes\nQuantity realized: 10 tonnes\n3 evidence entries"
        );
    }

    #[test]
    fn multiple_partial_arrivals_preserve_multiplicity_and_ignore_closing_order_totals() {
        let mut snapshot = snapshot();
        snapshot.events = triplet("order", 3, 6);
        let mut second = triplet("order", 3, 6);
        for event in &mut second {
            event.id.push_str("-second");
        }
        snapshot.events.extend(second);
        let log = delivery_log_entries(&snapshot, 160).unwrap();
        let group = group(&log, 0);
        assert_eq!(group.events.len(), 6);
        for stage in [group.arrivals, group.deliveries, group.realizations] {
            assert_eq!(
                stage,
                Some(DeliveryStageTotal {
                    quantity: 12,
                    evidence_entries: 2
                })
            );
        }
        assert_eq!(group.key.order_id, "order");
        assert_eq!(group.headline(), "Sheet metal delivered to Macomb parts");
        assert!(!group.details().contains("1,000"));
        assert!(!group.details().contains("970"));
    }

    #[test]
    fn missing_stages_are_unknown_and_never_promoted_to_delivery() {
        let cases = [
            (
                ProductionDeliveryStageV1::Arrival,
                "Sheet metal arrived at Macomb parts",
            ),
            (
                ProductionDeliveryStageV1::QuantityRealization,
                "Quantity realization recorded for Sheet metal at Macomb parts",
            ),
        ];
        for (stage, headline) in cases {
            let mut snapshot = snapshot();
            snapshot.events = vec![event("only", stage, 4, "order", 2)];
            let log = delivery_log_entries(&snapshot, 160).unwrap();
            let group = group(&log, 0);
            assert_eq!(group.deliveries, None);
            assert_eq!(group.headline(), headline);
            assert!(group.details().contains("Delivered: no evidence entry"));
            assert!(!group.details().contains("Delivered: 0"));
        }
    }

    #[test]
    fn foundation_is_empty_and_unannotated_events_stay_individual() {
        let mut snapshot = snapshot();
        let log = delivery_log_entries(&snapshot, 160).unwrap();
        assert!(log.entries.is_empty());
        assert_eq!((log.total_entries, log.evidence_entries), (0, 0));
        let mut standalone = event(
            "dispatch",
            ProductionDeliveryStageV1::Delivery,
            4,
            "order",
            1,
        );
        standalone.delivery_evidence = None;
        snapshot.events.push(standalone);
        let log = delivery_log_entries(&snapshot, 160).unwrap();
        let DeliveryLogEntry::Event(actual) = &log.entries[0] else {
            panic!("display text must not turn an unannotated event into a group")
        };
        assert!(std::ptr::eq(*actual, &raw const snapshot.events[0]));
        assert_eq!(log.entries[0].week(), 1);
    }

    #[test]
    fn newest_member_positions_order_whole_groups_before_the_cap() {
        let mut snapshot = snapshot();
        let first = triplet("a", 3, 2);
        let second = triplet("b", 3, 5);
        snapshot.events = vec![
            first[0].clone(),
            second[0].clone(),
            second[1].clone(),
            first[1].clone(),
            second[2].clone(),
            first[2].clone(),
        ];
        let mut last = event(
            "production",
            ProductionDeliveryStageV1::Delivery,
            4,
            "unused",
            3,
        );
        last.delivery_evidence = None;
        snapshot.events.push(last);
        let log = delivery_log_entries(&snapshot, 2).unwrap();
        assert_eq!(
            (log.total_entries, log.evidence_entries, log.entries.len()),
            (3, 7, 2)
        );
        assert!(matches!(log.entries[0], DeliveryLogEntry::Event(_)));
        let newest_group = group(&log, 1);
        assert_eq!(newest_group.key.order_id, "a");
        assert_eq!(
            newest_group
                .events
                .iter()
                .map(|event| event.id.as_str())
                .collect::<Vec<_>>(),
            ["a-3-0", "a-3-1", "a-3-2"]
        );
        let all = delivery_log_entries(&snapshot, 160).unwrap();
        assert_eq!(group(&all, 2).key.order_id, "b");
        let none = delivery_log_entries(&snapshot, 0).unwrap();
        assert!(none.entries.is_empty());
        assert_eq!(none.total_entries, 3);
    }

    #[test]
    fn order_week_and_receipt_identity_keep_equal_labels_apart() {
        let mut snapshot = snapshot();
        snapshot.events.extend(triplet("a", 2, 3));
        snapshot.events.extend(triplet("b", 2, 3));
        snapshot.events.extend(triplet("a", 3, 3));
        let mut other_receipt = triplet("a", 3, 3);
        for event in &mut other_receipt {
            event.receipt_digest = "another-receipt-family".into();
        }
        snapshot.events.extend(other_receipt);
        let log = delivery_log_entries(&snapshot, 160).unwrap();
        assert_eq!((log.total_entries, log.evidence_entries), (4, 12));
        assert_eq!(group(&log, 0).key.receipt_digest, "another-receipt-family");
        assert_eq!(group(&log, 1).key.week, 3);
        assert_eq!(group(&log, 2).key.order_id, "b");
        assert_eq!(group(&log, 3).key.week, 2);
    }

    #[test]
    fn route_good_and_unit_ids_separate_groups_despite_identical_display_labels() {
        for (good_id, unit_id) in [
            ("sheet", "tonnes"),
            ("other-good", "tonnes"),
            ("sheet", "other-unit"),
        ] {
            let mut snapshot = snapshot();
            snapshot.events = triplet("order", 2, 3);
            let mut route = snapshot.routes[0].clone();
            route.id = "other-route".into();
            route.good_id = good_id.into();
            route.unit_id = unit_id.into();
            snapshot.routes.push(route);
            let mut other_events = triplet("other-order", 2, 3);
            for event in &mut other_events {
                let evidence = event.delivery_evidence.as_mut().unwrap();
                evidence.route_id = "other-route".into();
                evidence.good_id = good_id.into();
                evidence.unit_id = unit_id.into();
            }
            snapshot.events.extend(other_events);
            let log = delivery_log_entries(&snapshot, 160).unwrap();
            assert_eq!(log.total_entries, 2);
            assert_ne!(group(&log, 0).key, group(&log, 1).key);
            assert_eq!(group(&log, 0).headline(), group(&log, 1).headline());
        }
    }

    #[test]
    fn one_order_receipt_refuses_conflicting_principals_before_the_cap() {
        for changed in ["route", "good", "unit", "supplier", "buyer"] {
            let mut snapshot = snapshot();
            snapshot.events = triplet("order", 2, 3);
            let mut route = snapshot.routes[0].clone();
            route.id = "other-route".into();
            match changed {
                "good" => {
                    route.good_id = "other-good".into();
                }
                "unit" => {
                    route.unit_id = "other-unit".into();
                }
                "supplier" => {
                    snapshot.sites.push(site("other-supplier", "Wayne metal"));
                    route.supplier_site_id = "other-supplier".into();
                }
                "buyer" => {
                    snapshot.sites.push(site("other-buyer", "Macomb parts"));
                    route.buyer_site_id = "other-buyer".into();
                }
                _ => {}
            }
            let mut contradictory = triplet("order", 2, 3);
            for event in &mut contradictory {
                event.id.push_str("-contradictory");
                event.subject_site_ids =
                    vec![route.supplier_site_id.clone(), route.buyer_site_id.clone()];
                let evidence = event.delivery_evidence.as_mut().unwrap();
                evidence.route_id.clone_from(&route.id);
                evidence.good_id.clone_from(&route.good_id);
                evidence.unit_id.clone_from(&route.unit_id);
            }
            snapshot.routes.push(route);
            snapshot.events.extend(contradictory);
            for limit in [0, 160] {
                assert_eq!(
                    delivery_log_entries(&snapshot, limit).unwrap_err(),
                    DeliveryGroupingError::IdentityMismatch,
                    "contradictory {changed} must not become a separate group"
                );
            }
        }
    }

    #[test]
    fn missing_ambiguous_or_mismatched_disclosed_identities_refuse() {
        let mut original = snapshot();
        original.events = triplet("order", 2, 3);
        let mut missing_route = original.clone();
        missing_route.routes.clear();
        assert_eq!(
            delivery_log_entries(&missing_route, 160).unwrap_err(),
            DeliveryGroupingError::RouteUnavailable
        );
        let mut missing_site = original.clone();
        missing_site.sites.pop();
        assert_eq!(
            delivery_log_entries(&missing_site, 160).unwrap_err(),
            DeliveryGroupingError::SiteUnavailable
        );
        let mut duplicate_route = original.clone();
        duplicate_route
            .routes
            .push(duplicate_route.routes[0].clone());
        assert_eq!(
            delivery_log_entries(&duplicate_route, 160).unwrap_err(),
            DeliveryGroupingError::RouteUnavailable
        );
        let mut duplicate_site = original.clone();
        duplicate_site.sites.push(duplicate_site.sites[0].clone());
        assert_eq!(
            delivery_log_entries(&duplicate_site, 160).unwrap_err(),
            DeliveryGroupingError::SiteUnavailable
        );
        for field in ["good", "unit", "subject", "week"] {
            let mut mismatch = original.clone();
            let event = &mut mismatch.events[0];
            match field {
                "good" => {
                    event.delivery_evidence.as_mut().unwrap().good_id = "other".into();
                }
                "unit" => {
                    event.delivery_evidence.as_mut().unwrap().unit_id = "other".into();
                }
                "subject" => {
                    event.subject_site_ids.pop();
                }
                _ => {
                    event.week = 0;
                }
            }
            assert_eq!(
                delivery_log_entries(&mismatch, 160).unwrap_err(),
                DeliveryGroupingError::IdentityMismatch
            );
        }
    }

    #[test]
    fn subject_order_does_not_change_groups_and_stage_overflow_never_wraps() {
        let mut snapshot = snapshot();
        snapshot.events = triplet("order", 2, u64::MAX);
        for event in &mut snapshot.events {
            event.subject_site_ids.reverse();
        }
        let log = delivery_log_entries(&snapshot, 160).unwrap();
        assert_eq!(group(&log, 0).deliveries.unwrap().quantity, u64::MAX);
        snapshot.events.push(event(
            "overflow",
            ProductionDeliveryStageV1::Delivery,
            1,
            "order",
            2,
        ));
        assert_eq!(
            delivery_log_entries(&snapshot, 0).unwrap_err(),
            DeliveryGroupingError::QuantityRange
        );
        snapshot
            .events
            .last_mut()
            .unwrap()
            .delivery_evidence
            .as_mut()
            .unwrap()
            .quantity = 0;
        assert_eq!(
            delivery_log_entries(&snapshot, 160).unwrap_err(),
            DeliveryGroupingError::QuantityRange
        );
    }
}
