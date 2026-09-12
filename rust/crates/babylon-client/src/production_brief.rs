//! Briefs from committed production readings and visible dependency identities.
//! This presentation does not infer past causes from closing stocks, predict
//! output, convert hours to people, or adjudicate production requirements.

use std::collections::{BTreeMap, BTreeSet};

use babylon_persistence::{
    production_observation::ProductionSite, production_observation::ProductionSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DependencyDirection {
    Upstream,
    Downstream,
}
impl DependencyDirection {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Upstream => "UPSTREAM / SUPPLIERS",
            Self::Downstream => "DOWNSTREAM / BUYERS",
        }
    }
}

/// Recipe supplier declarations and committed route accounts identify real
/// relations. Missing capability-scoped endpoints never become named links.
pub(crate) fn dependency_sites<'a>(
    site: &ProductionSite,
    snapshot: &'a ProductionSnapshot,
) -> Vec<(DependencyDirection, &'a ProductionSite)> {
    let mut links = BTreeSet::new();
    for relation in crate::material_relations::declared_material_relations(snapshot) {
        if relation.buyer == site.id {
            links.insert((DependencyDirection::Upstream, relation.supplier));
        }
        if relation.supplier == site.id {
            links.insert((DependencyDirection::Downstream, relation.buyer));
        }
    }
    links
        .into_iter()
        .filter_map(|(direction, id)| {
            snapshot
                .sites
                .iter()
                .find(|candidate| candidate.id == id)
                .map(|site| (direction, site))
        })
        .collect()
}

fn unfinished_plan(site: &ProductionSite) -> bool {
    site.processes.iter().any(|process| matches!((process.produced_batches, process.planned_batches), (Some(done), Some(plan)) if done < plan))
}

/// Deterministic entry into an existing relation, without a strategic score.
/// An unfulfilled committed plan comes first; otherwise choose a visible
/// link between suppliers and buyers, then the first stable site identity.
pub(crate) fn opening_site(snapshot: &ProductionSnapshot) -> Option<&ProductionSite> {
    let first = |predicate: &dyn Fn(&ProductionSite) -> bool| {
        snapshot
            .sites
            .iter()
            .filter(|site| predicate(site))
            .min_by(|a, b| a.id.cmp(&b.id))
    };
    first(&unfinished_plan)
        .or_else(|| {
            first(&|site| {
                let links = dependency_sites(site, snapshot);
                links
                    .iter()
                    .any(|(direction, _)| *direction == DependencyDirection::Upstream)
                    && links
                        .iter()
                        .any(|(direction, _)| *direction == DependencyDirection::Downstream)
            })
        })
        .or_else(|| snapshot.sites.iter().min_by(|a, b| a.id.cmp(&b.id)))
}

/// Only the committed plan and output determine this label, never closing stock.
pub(crate) fn committed_plan_status(site: &ProductionSite) -> &'static str {
    if site.processes.is_empty() {
        return match site.role {
            babylon_persistence::production_observation::ProductionSiteRole::Wholesale => {
                "Wholesale / handling and onward distribution"
            }
            babylon_persistence::production_observation::ProductionSiteRole::Retail => {
                "Retail / delivery to final demand"
            }
            babylon_persistence::production_observation::ProductionSiteRole::Production => {
                "No productive process disclosed"
            }
        };
    }
    if site.processes.len() > 1 {
        return if unfinished_plan(site) {
            "One or more process plans partly completed"
        } else if site
            .processes
            .iter()
            .all(|process| process.produced_batches.is_none())
        {
            "Opening state; no committed production yet"
        } else {
            "Committed processes / read each material separately"
        };
    }
    process_plan_status(&site.processes[0])
}

pub(crate) fn process_plan_status(
    process: &babylon_persistence::production_observation::ProductionProcess,
) -> &'static str {
    match (process.produced_batches, process.planned_batches) {
        (None, None) => "Opening state; no committed production yet",
        (Some(0), Some(0)) => "No production planned this period",
        (Some(0), Some(_)) => "No work committed against the plan",
        (Some(done), Some(plan)) if done < plan => "Committed plan partly completed",
        (Some(done), Some(plan)) if done == plan => "Committed plan completed",
        (Some(_), Some(_)) => "Committed output exceeds recorded plan",
        _ => "Committed production reading unavailable",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FlowFact {
    Shipped,
    InTransit,
    Delivered,
    Backlog,
    Loss,
}

type RelationKey<'a> = (&'a str, &'a str, &'a str, &'a str);

struct MaterialRelation<'a> {
    supplier: &'a ProductionSite,
    buyer: &'a ProductionSite,
    labels: BTreeSet<(&'a str, &'a str)>,
    requirement: bool,
    route_ids: BTreeSet<&'a str>,
    facts: BTreeSet<FlowFact>,
}

impl<'a> MaterialRelation<'a> {
    fn new(supplier: &'a ProductionSite, buyer: &'a ProductionSite) -> Self {
        Self {
            supplier,
            buyer,
            labels: BTreeSet::new(),
            requirement: false,
            route_ids: BTreeSet::new(),
            facts: BTreeSet::new(),
        }
    }

    fn material(&self) -> String {
        if self.labels.len() == 1 {
            let (good, unit) = self.labels.first().expect("one material label");
            format!("{good} ({unit})")
        } else {
            "Material label unavailable".into()
        }
    }

    fn flow(&self) -> String {
        if self.route_ids.is_empty() {
            return "requirement only; shipment evidence unavailable".into();
        }
        let mut labels = vec![if self.requirement {
            "requirement + route"
        } else {
            "route"
        }];
        if !self.facts.contains(&FlowFact::Shipped) {
            labels.push("no shipment recorded");
        }
        if self.facts.contains(&FlowFact::InTransit) {
            labels.push("goods in transit");
        }
        if self.facts.contains(&FlowFact::Delivered) {
            labels.push("delivery recorded to date");
        }
        if self.facts.contains(&FlowFact::Backlog) {
            labels.push("backlog remains");
        } else {
            labels.push("no backlog");
        }
        if self.facts.contains(&FlowFact::Loss) {
            labels.push("loss recorded to date");
        }
        labels.join("; ")
    }
}

fn material_relations(
    snapshot: &ProductionSnapshot,
) -> BTreeMap<RelationKey<'_>, MaterialRelation<'_>> {
    let mut relations = BTreeMap::new();
    for declared in crate::material_relations::declared_material_relations(snapshot) {
        let supplier = snapshot
            .sites
            .iter()
            .find(|site| site.id == declared.supplier);
        let buyer = snapshot.sites.iter().find(|site| site.id == declared.buyer);
        let (Some(supplier), Some(buyer)) = (supplier, buyer) else {
            continue;
        };
        let key = (
            declared.supplier,
            declared.buyer,
            declared.good_id,
            declared.unit_id,
        );
        let relation = relations
            .entry(key)
            .or_insert_with(|| MaterialRelation::new(supplier, buyer));
        relation.labels.insert((declared.good, declared.unit));
        let Some(route) = declared.route else {
            relation.requirement = true;
            continue;
        };
        relation.route_ids.insert(route.id.as_str());
        for (quantity, fact) in [
            (route.shipped, FlowFact::Shipped),
            (route.delivered, FlowFact::Delivered),
            (route.backlog, FlowFact::Backlog),
            (route.lost, FlowFact::Loss),
        ] {
            if quantity > 0 {
                relation.facts.insert(fact);
            }
        }
    }
    for ((supplier, buyer, good, unit), relation) in &mut relations {
        if snapshot.freight.iter().any(|lot| {
            lot.quantity > 0
                && relation.route_ids.contains(lot.route_id.as_str())
                && lot.source_site_id == *supplier
                && lot.destination_site_id == *buyer
                && lot.good_id == *good
                && lot.unit_id == *unit
        }) {
            relation.facts.insert(FlowFact::InTransit);
        }
    }
    relations
}

/// The dependency button already names its endpoint. Its second line describes
/// the disclosed material and flow without repeating the name or summing goods.
pub(crate) fn dependency_flow_summary(
    site: &ProductionSite,
    other: &ProductionSite,
    direction: DependencyDirection,
    snapshot: &ProductionSnapshot,
) -> String {
    let relations = material_relations(snapshot);
    let relevant: Vec<_> = relations
        .values()
        .filter(|relation| match direction {
            DependencyDirection::Upstream => {
                relation.buyer.id == site.id && relation.supplier.id == other.id
            }
            DependencyDirection::Downstream => {
                relation.supplier.id == site.id && relation.buyer.id == other.id
            }
        })
        .collect();
    if relevant.is_empty() {
        return "Material relationship unavailable".into();
    }
    let mut lines: Vec<_> = relevant
        .iter()
        .take(2)
        .map(|relation| format!("{}: {}", relation.material(), relation.flow()))
        .collect();
    if relevant.len() > 2 {
        lines.push("More materials in Details.".into());
    }
    lines.join("\n")
}

/// Keep the introduction brief; neighboring buttons carry material and flow.
pub(crate) fn describe_brief(site: &ProductionSite, snapshot: &ProductionSnapshot) -> String {
    let guidance = if dependency_sites(site, snapshot).is_empty() {
        "No material relationships disclosed."
    } else {
        "Follow a supplier or buyer below."
    };
    format!("{}\n{}\n{guidance}", site.name, committed_plan_status(site))
}

/// A bounded map of disclosed material relationships, with no cross-good totals.
pub(crate) fn describe_overview(snapshot: &ProductionSnapshot) -> String {
    if snapshot.sites.is_empty() {
        return "No production cohorts are visible in this observation.".to_owned();
    }
    let relations = material_relations(snapshot);
    let mut lines = vec!["MATERIAL RELATIONSHIPS".to_owned()];
    for relation in relations.values().take(4) {
        lines.push(format!(
            "{} -> {} / {}: {}",
            relation.supplier.name,
            relation.buyer.name,
            relation.material(),
            relation.flow()
        ));
    }
    if relations.is_empty() {
        lines.push("No material relationships disclosed.".to_owned());
    }
    if relations.len() > 4 {
        lines.push("Select a cohort to reveal its other relationships.".to_owned());
    }
    lines.push("Follow inputs, shipments and the work they connect.".to_owned());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_persistence::{
        production_observation::ProductionFreight, production_observation::ProductionInput,
        production_observation::ProductionLabor, production_observation::ProductionRoute,
    };

    fn site(id: &str, suppliers: &[&str]) -> ProductionSite {
        ProductionSite {
            id: id.into(),
            county_geoid: "26163".into(),
            name: format!("Cohort {id}"),
            industry_code: "331".into(),
            observed_employment: Some(999_999),
            inventory: Vec::new(),
            role: babylon_persistence::production_observation::ProductionSiteRole::Production,
            sector_code: "31-33".into(),
            processes: vec![
                babylon_persistence::production_observation::ProductionProcess {
                    id: "fixture-process".into(),
                    name: "Fixture process".into(),
                    output_good_id: "output".into(),
                    output_unit_id: "kg".into(),
                    output_good: "steel".into(),
                    output_unit: "kg".into(),
                    output_per_batch: 10,
                    available_batches: 8,
                    planned_batches: Some(8),
                    produced_batches: Some(8),
                    inputs: suppliers
                        .iter()
                        .map(|supplier| ProductionInput {
                            good_id: (*supplier).into(),
                            unit_id: "kg".into(),
                            good: format!("Input {supplier}"),
                            unit: "kg".into(),
                            quantity_per_batch: 3,
                            on_hand: 5,
                            supplier_site_ids: vec![(*supplier).into()],
                        })
                        .collect(),
                    labor: vec![ProductionLabor {
                        unit: "labor-hours".into(),
                        available: 7,
                        quantity_per_batch: 2,
                    }],
                },
            ],
        }
    }
    fn chain() -> ProductionSnapshot {
        ProductionSnapshot {
            content_authority_sha256: "a".repeat(64),
            road_source: None,
            physical_edges: Vec::new(),
            merchant_handling_accounts: Vec::new(),
            final_demand_accounts: Vec::new(),
            freight_capacity_accounts: Vec::new(),
            material_balance: None,
            labor_accounts: Vec::new(),
            staffing_accounts: Vec::new(),
            scenario_label: "Designed test chain".into(),
            horizon_period: 16,
            sites: vec![site("a", &[]), site("b", &["a"]), site("c", &["b"])],
            routes: Vec::new(),
            freight: Vec::new(),
            events: Vec::new(),
            observed_contexts: Vec::new(),
            process_attributions: Vec::new(),
            provenance: Vec::new(),
        }
    }

    #[test]
    fn opening_uses_unfinished_plan_then_connected_site_then_stable_id() {
        let mut snapshot = chain();
        assert_eq!(opening_site(&snapshot).expect("connected site").id, "b");
        snapshot.sites[2].processes[0].produced_batches = Some(0);
        assert_eq!(opening_site(&snapshot).expect("unfinished plan").id, "c");
        snapshot.sites[0].processes[0].produced_batches = Some(7);
        assert_eq!(
            opening_site(&snapshot).expect("stable unfinished ID").id,
            "a"
        );
        for site in &mut snapshot.sites {
            site.processes[0].planned_batches = None;
            site.processes[0].produced_batches = None;
            site.processes[0].inputs.clear();
        }
        snapshot.sites.reverse();
        assert_eq!(opening_site(&snapshot).expect("stable default").id, "a");
    }

    #[test]
    fn closing_resources_do_not_rewrite_the_committed_work_reading() {
        let mut snapshot = chain();
        snapshot.sites[1].processes[0].produced_batches = Some(3);
        let text = describe_brief(&snapshot.sites[1], &snapshot);
        for fact in [
            "Cohort b",
            "Committed plan partly completed",
            "Follow a supplier or buyer below.",
        ] {
            assert!(text.contains(fact), "missing {fact}: {text}");
        }
        for invented in [
            "bottleneck",
            "caused",
            "workers",
            "support",
            "layoff",
            "999,999",
            "supports 1 batch",
            "CAPACITY",
            "3 of 8",
        ] {
            assert!(!text.contains(invented), "invented {invented}: {text}");
        }
        snapshot.sites[1].processes[0].inputs[0].on_hand = 0;
        snapshot.sites[1].processes[0].labor[0].available = 0;
        snapshot.sites[1].processes[0].available_batches = 0;
        assert_eq!(describe_brief(&snapshot.sites[1], &snapshot), text);
    }

    #[test]
    fn unavailable_endpoints_never_become_names_or_political_claims() {
        let mut snapshot = chain();
        snapshot.sites[1].processes[0].inputs[0].supplier_site_ids =
            vec!["withheld-endpoint".into()];
        snapshot.sites.retain(|site| site.id != "c");
        let text = describe_brief(&snapshot.sites[1], &snapshot);
        assert!(text.contains("No material relationships disclosed."));
        assert!(!text.contains("withheld-endpoint"));
        assert!(!describe_overview(&snapshot).contains("withheld-endpoint"));
    }

    #[test]
    fn foundation_missing_reading_and_committed_zero_stay_distinct() {
        let mut snapshot = chain();
        snapshot.sites[1].processes[0].planned_batches = None;
        snapshot.sites[1].processes[0].produced_batches = None;
        assert!(
            describe_brief(&snapshot.sites[1], &snapshot).contains("no committed production yet")
        );
        snapshot.sites[1].processes[0].planned_batches = Some(0);
        assert!(describe_brief(&snapshot.sites[1], &snapshot).contains("reading unavailable"));
        snapshot.sites[1].processes[0].produced_batches = Some(0);
        assert!(describe_brief(&snapshot.sites[1], &snapshot)
            .contains("No production planned this period"));
        snapshot.sites[1].processes[0].planned_batches = Some(8);
        assert_eq!(
            committed_plan_status(&snapshot.sites[1]),
            "No work committed against the plan"
        );
        snapshot.sites[1].processes[0].produced_batches = Some(8);
        assert_eq!(
            committed_plan_status(&snapshot.sites[1]),
            "Committed plan completed"
        );
        snapshot.sites[1].processes[0].produced_batches = Some(9);
        assert_eq!(
            committed_plan_status(&snapshot.sites[1]),
            "Committed output exceeds recorded plan"
        );
    }

    #[test]
    fn ordering_and_duplicate_supplier_mentions_do_not_change_the_reading() {
        let mut snapshot = chain();
        snapshot.sites[1].processes[0].inputs[0]
            .supplier_site_ids
            .push("a".into());
        let site = snapshot.sites[1].clone();
        let before = describe_brief(&site, &snapshot);
        let overview = describe_overview(&snapshot);
        snapshot.sites.reverse();
        assert_eq!(describe_brief(&site, &snapshot), before);
        assert_eq!(describe_overview(&snapshot), overview);
        assert_eq!(dependency_sites(&site, &snapshot).len(), 2);
        assert_eq!(opening_site(&snapshot).expect("opening site").id, "b");
    }

    #[test]
    fn brief_leads_with_relationships_and_leaves_exact_resources_in_details() {
        let mut snapshot = chain();
        snapshot.sites[1] = site("b", &["a", "d", "e", "f"]);
        let text = describe_brief(&snapshot.sites[1], &snapshot);
        assert_eq!(text.lines().count(), 3);
        assert!(!text.contains("Cohort a"));
        assert!(!text.contains("Input a"));
        let flow = dependency_flow_summary(
            &snapshot.sites[1],
            &snapshot.sites[0],
            DependencyDirection::Upstream,
            &snapshot,
        );
        assert!(flow.contains("Input a (kg)"));
        assert!(!flow.contains("Cohort a"));
        for detail in ["per batch", "labor-hours", "8 batches", "999,999"] {
            assert!(!text.contains(detail), "duplicated detail {detail}: {text}");
        }
        snapshot.sites.clear();
        assert_eq!(opening_site(&snapshot), None);
        assert_eq!(
            describe_overview(&snapshot),
            "No production cohorts are visible in this observation."
        );
    }

    fn route() -> ProductionRoute {
        ProductionRoute {
            physical_edge_ids: Vec::new(),
            distance_mm: None,
            transport_kind:
                babylon_persistence::production_observation::ProductionRouteTransport::Staged,
            grams_per_unit: 1000,
            stages: Vec::new(),
            id: "route-a-b".into(),
            supplier_site_id: "a".into(),
            buyer_site_id: "b".into(),
            good_id: "a".into(),
            unit_id: "kg".into(),
            good: "Input a".into(),
            unit: "kg".into(),
            travel_periods: 2,
            ordered: 120,
            shipped: 100,
            delivered: 80,
            lost: 0,
            realized: 80,
            backlog: 20,
        }
    }

    fn freight() -> ProductionFreight {
        ProductionFreight {
            current_stage_index: 0,
            grams_per_unit: 1000,
            mass_grams: 1000,
            id: "lot-a-b".into(),
            route_id: "route-a-b".into(),
            source_site_id: "a".into(),
            destination_site_id: "b".into(),
            good_id: "a".into(),
            unit_id: "kg".into(),
            good: "Input a".into(),
            unit: "kg".into(),
            quantity: 20,
            dispatch_period: 2,
            arrival_period: 4,
        }
    }

    #[test]
    fn shipment_state_comes_from_exact_routes_and_lots_not_requirements() {
        let mut snapshot = chain();
        let requirement = dependency_flow_summary(
            &snapshot.sites[1],
            &snapshot.sites[0],
            DependencyDirection::Upstream,
            &snapshot,
        );
        assert!(requirement.contains("shipment evidence unavailable"));
        assert!(!requirement.contains("goods in transit"));
        snapshot.routes.push(route());
        snapshot.freight.push(freight());
        let moving = dependency_flow_summary(
            &snapshot.sites[1],
            &snapshot.sites[0],
            DependencyDirection::Upstream,
            &snapshot,
        );
        for fact in [
            "requirement + route",
            "goods in transit",
            "delivery recorded to date",
            "backlog remains",
        ] {
            assert!(moving.contains(fact), "missing {fact}: {moving}");
        }
        // A same-named lot with another canonical material is not this flow.
        snapshot.freight[0].good_id = "another-good".into();
        assert!(!dependency_flow_summary(
            &snapshot.sites[1],
            &snapshot.sites[0],
            DependencyDirection::Upstream,
            &snapshot
        )
        .contains("goods in transit"));
        snapshot.freight.clear();
        snapshot.routes[0].backlog = 0;
        let arrived = dependency_flow_summary(
            &snapshot.sites[1],
            &snapshot.sites[0],
            DependencyDirection::Upstream,
            &snapshot,
        );
        assert!(arrived.contains("delivery recorded to date; no backlog"));
        assert!(!arrived.contains("goods in transit"));
        assert!(!arrived.contains("this period"));
        snapshot.routes[0].shipped = 0;
        snapshot.routes[0].delivered = 0;
        let no_shipment = dependency_flow_summary(
            &snapshot.sites[1],
            &snapshot.sites[0],
            DependencyDirection::Upstream,
            &snapshot,
        );
        assert!(no_shipment.contains("no shipment recorded"));
        assert!(!no_shipment.contains("delivery recorded to date"));
    }

    #[test]
    fn material_identity_prevents_mixed_goods_and_units_from_collapsing() {
        let mut snapshot = chain();
        let mut food = snapshot.sites[1].processes[0].inputs[0].clone();
        food.good_id = "food".into();
        food.good = "Food".into();
        let mut tonnes = snapshot.sites[1].processes[0].inputs[0].clone();
        tonnes.unit_id = "tonne".into();
        tonnes.unit = "tonne".into();
        snapshot.sites[1].processes[0].inputs.extend([food, tonnes]);
        snapshot.routes.extend([route(), route()]);
        snapshot.freight.extend([freight(), freight()]);
        let before = describe_overview(&snapshot);
        assert_eq!(before.matches("Cohort a -> Cohort b").count(), 3);
        for material in ["Input a (kg)", "Input a (tonne)", "Food (kg)"] {
            assert!(before.contains(material), "missing {material}: {before}");
        }
        assert_eq!(before.matches("goods in transit").count(), 1);
        assert!(!before.contains("100"));
        snapshot.sites[1].processes[0].inputs.reverse();
        snapshot.sites.reverse();
        snapshot.routes.reverse();
        snapshot.freight.reverse();
        assert_eq!(describe_overview(&snapshot), before);
    }

    #[test]
    fn undisclosed_route_endpoints_cannot_supply_names_or_flow_facts() {
        let mut snapshot = chain();
        snapshot.routes.push(route());
        snapshot.freight.push(freight());
        snapshot.sites.retain(|site| site.id == "b");
        let text = describe_brief(&snapshot.sites[0], &snapshot);
        for absent in [
            "Cohort a",
            "Cohort c",
            "Input a",
            "goods in transit",
            "delivery recorded",
        ] {
            assert!(!text.contains(absent), "undisclosed {absent}: {text}");
        }
        assert!(describe_overview(&snapshot).contains("No material relationships disclosed"));
    }
}
