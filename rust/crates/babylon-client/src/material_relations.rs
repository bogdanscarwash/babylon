//! Authored input suppliers and route declarations in their captured order.
//! Consumers retain their own endpoint visibility, aggregation, and labeling rules.

use babylon_persistence::{
    production_observation::ProductionRoute, production_observation::ProductionSnapshot,
};

pub(crate) struct MaterialRelationDeclaration<'a> {
    pub(crate) supplier: &'a str,
    pub(crate) buyer: &'a str,
    pub(crate) good_id: &'a str,
    pub(crate) unit_id: &'a str,
    pub(crate) good: &'a str,
    pub(crate) unit: &'a str,
    pub(crate) route: Option<&'a ProductionRoute>,
}

pub(crate) fn declared_material_relations(
    snapshot: &ProductionSnapshot,
) -> impl Iterator<Item = MaterialRelationDeclaration<'_>> {
    let requirements = snapshot.sites.iter().flat_map(|buyer| {
        buyer.processes.iter().flat_map(move |process| {
            process.inputs.iter().flat_map(move |input| {
                input
                    .supplier_site_ids
                    .iter()
                    .map(move |supplier| MaterialRelationDeclaration {
                        supplier,
                        buyer: &buyer.id,
                        good_id: &input.good_id,
                        unit_id: &input.unit_id,
                        good: &input.good,
                        unit: &input.unit,
                        route: None,
                    })
            })
        })
    });
    requirements.chain(
        snapshot
            .routes
            .iter()
            .map(|route| MaterialRelationDeclaration {
                supplier: &route.supplier_site_id,
                buyer: &route.buyer_site_id,
                good_id: &route.good_id,
                unit_id: &route.unit_id,
                good: &route.good,
                unit: &route.unit,
                route: Some(route),
            }),
    )
}
