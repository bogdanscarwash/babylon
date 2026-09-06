//! Presentation identity for the already-authorized production observation.
//!
//! V3 uses a fixed domain/version, big-endian u64 quantities and lengths,
//! length-prefixed UTF-8, and explicit 0/1 option tags. Unordered rows sort by
//! their complete typed fields (including exact good/unit identities), after
//! nested collections have been sorted. Duplicates retain their multiplicity.
//! Events retain their supplied sequence; their subject sets and provenance
//! declarations are unordered. V3 appends optional typed delivery evidence to
//! each event (stage tags 1 arrival, 2 delivery, 3 quantity realization), and an
//! optional completed material balance after provenance. Each balance encodes
//! week, row count, then site/good/unit identities and labels followed by
//! opening/arrivals/produced/consumed/dispatched/closing quantities. V1 and V2
//! identities retain their historical meaning; V3 is the sole live encoder.
//! Changing this layout requires a new version.
//!
//! This is neither a world hash nor an authorization proof. It commits to what
//! the reader disclosed, including labels and assumptions, without fetching
//! additional data or performing economic calculations. Camera, lens and UI
//! preferences are not production-observation inputs in the current client.

use sha2::{Digest as _, Sha256};

use crate::{
    ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionEventV1, ProductionFreightV1,
    ProductionInputV1, ProductionRouteV1, ProductionSiteV1, ProductionSnapshotV1,
    ProductionStockV1,
};

const DOMAIN: &[u8] = b"babylon.production-observation-evidence.v3\0";

/// SHA-256 of one scope-bound production presentation, distinct from world identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProductionEvidenceDigestV3([u8; 32]);

impl ProductionEvidenceDigestV3 {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lowercase hexadecimal suitable for the observation's evidence details.
    #[must_use]
    pub fn to_hex(self) -> String {
        crate::michigan_economy::digest_hex(&self.0)
    }
}

impl ObserverEconomySnapshotV1 {
    /// Bind the exact disclosed production rows to their campaign and committed scope.
    ///
    /// Absent production returns `None`, including today's known-only preview.
    /// Call only after validating that this observation belongs to the current
    /// session; the digest does not validate provenance or confer read authority.
    /// Compute on observation installation or evidence disclosure, not per frame.
    #[must_use]
    pub fn production_evidence_digest(&self) -> Option<ProductionEvidenceDigestV3> {
        let production = canonical_production(self.production.as_ref()?);
        let mut encoder = EvidenceEncoder(Sha256::new());
        encoder.0.update(DOMAIN);
        encoder.0.update(3_u32.to_be_bytes());
        encoder.text(&self.campaign_id);
        encoder.number(self.resolve_tick);
        encoder.text(&self.foundation_digest);
        encoder.optional_text(self.tick_content_hash.as_deref());
        encoder.optional_text(self.envelope_digest.as_deref());
        encoder.optional_text(self.nominal_world_hash.as_deref());
        encoder.0.update([match self.visibility {
            ObserverVisibilityV1::FullObserver => 0,
            ObserverVisibilityV1::KnownPreview => 1,
        }]);
        encoder.production(&production);
        Some(ProductionEvidenceDigestV3(encoder.0.finalize().into()))
    }
}

fn canonical_production(source: &ProductionSnapshotV1) -> ProductionSnapshotV1 {
    let mut rows = source.clone();
    for site in &mut rows.sites {
        site.inventory.sort_unstable();
        for input in &mut site.inputs {
            input.supplier_site_ids.sort_unstable();
        }
        site.inputs.sort_unstable();
        site.labor.sort_unstable();
    }
    rows.sites.sort_unstable();
    rows.labor_accounts.sort_unstable();
    if let Some(balance) = &mut rows.material_balance {
        balance.rows.sort_unstable();
    }
    rows.observed_contexts.sort_unstable();
    rows.process_attributions.sort_unstable();
    rows.routes.sort_unstable();
    rows.freight.sort_unstable();
    for event in &mut rows.events {
        event.subject_site_ids.sort_unstable();
    }
    rows.provenance.sort_unstable();
    rows
}

struct EvidenceEncoder(Sha256);

impl EvidenceEncoder {
    fn number(&mut self, value: u64) {
        self.0.update(value.to_be_bytes());
    }

    fn count(&mut self, length: usize) {
        // Supported Rust targets have at most 64-bit pointers; no truncation
        // or saturating fallback is permitted in the identity layout.
        self.number(u64::try_from(length).expect("collection length fits u64"));
    }

    fn text(&mut self, value: &str) {
        self.count(value.len());
        self.0.update(value.as_bytes());
    }

    fn optional_text(&mut self, value: Option<&str>) {
        self.0.update([u8::from(value.is_some())]);
        if let Some(value) = value {
            self.text(value);
        }
    }

    fn optional_number(&mut self, value: Option<u64>) {
        self.0.update([u8::from(value.is_some())]);
        if let Some(value) = value {
            self.number(value);
        }
    }

    fn strings(&mut self, values: &[String]) {
        self.count(values.len());
        for value in values {
            self.text(value);
        }
    }

    fn production(&mut self, rows: &ProductionSnapshotV1) {
        self.text(&rows.scenario_label);
        self.number(rows.horizon_week);
        self.count(rows.sites.len());
        for site in &rows.sites {
            self.site(site);
        }
        self.count(rows.routes.len());
        for route in &rows.routes {
            self.route(route);
        }
        self.count(rows.freight.len());
        for lot in &rows.freight {
            self.freight(lot);
        }
        self.count(rows.events.len());
        for event in &rows.events {
            self.event(event);
        }
        self.count(rows.labor_accounts.len());
        for account in &rows.labor_accounts {
            self.text(&account.site_id);
            self.text(&account.unit_id);
            self.text(&account.unit);
            self.number(account.next_opening_week);
            self.number(account.next_opening_available);
            self.0.update([u8::from(account.completed.is_some())]);
            if let Some(completed) = &account.completed {
                self.number(completed.week);
                self.number(completed.opening);
                self.number(completed.planned);
                self.number(completed.used);
                self.number(completed.unused);
            }
        }
        self.count(rows.observed_contexts.len());
        for context in &rows.observed_contexts {
            self.observed_context(context);
        }
        self.count(rows.process_attributions.len());
        for link in &rows.process_attributions {
            self.text(&link.process_id);
            self.text(&link.site_id);
            self.text(&link.industry_code);
            self.business_subject(&link.cohort_subject);
            self.text(&link.scenario_artifact_sha256);
            self.text(&link.industry_artifact_sha256);
            self.text(link.evidence_class.as_str());
        }
        self.strings(&rows.provenance);
        self.material_balance(rows.material_balance.as_ref());
    }

    fn material_balance(&mut self, balance: Option<&crate::CompletedMaterialBalanceV1>) {
        self.0.update([u8::from(balance.is_some())]);
        if let Some(balance) = balance {
            self.number(balance.week);
            self.count(balance.rows.len());
            for row in &balance.rows {
                for value in [
                    &row.site_id,
                    &row.good_id,
                    &row.unit_id,
                    &row.good,
                    &row.unit,
                ] {
                    self.text(value);
                }
                for value in [
                    row.opening,
                    row.arrivals,
                    row.produced,
                    row.consumed,
                    row.dispatched,
                    row.closing,
                ] {
                    self.number(value);
                }
            }
        }
    }

    fn business_subject(&mut self, subject: &crate::ProductionBusinessSubjectV1) {
        self.text(&subject.scenario);
        self.text(&subject.local_name);
    }

    fn observed_context(&mut self, context: &crate::ObservedManufacturingContextV1) {
        self.business_subject(&context.subject);
        self.text(&context.county_geoid);
        self.text(&context.sector_code);
        self.text(&context.sector_title);
        self.number(u64::from(context.vintage));
        self.number(context.annual_avg_estabs_count);
        self.optional_number(context.annual_avg_emplvl);
        self.optional_number(context.total_annual_wages);
        self.optional_number(context.annual_avg_wkly_wage);
        self.text(&context.source_url);
        self.text(&context.source_file);
        self.text(&context.source_sha256);
        self.text(&context.artifact_sha256);
        self.text(context.evidence_class.as_str());
    }

    fn site(&mut self, site: &ProductionSiteV1) {
        self.text(&site.id);
        self.text(&site.county_geoid);
        self.text(&site.name);
        self.text(&site.industry_code);
        self.optional_number(site.observed_employment);
        self.text(&site.output_good_id);
        self.text(&site.output_unit_id);
        self.text(&site.output_good);
        self.text(&site.output_unit);
        self.number(site.output_per_batch);
        self.number(site.available_batches);
        self.optional_number(site.planned_batches);
        self.optional_number(site.produced_batches);
        self.count(site.inventory.len());
        for stock in &site.inventory {
            self.stock(stock);
        }
        self.count(site.inputs.len());
        for input in &site.inputs {
            self.input(input);
        }
        self.count(site.labor.len());
        for labor in &site.labor {
            self.text(&labor.unit);
            self.number(labor.available);
            self.number(labor.quantity_per_batch);
        }
    }

    fn stock(&mut self, stock: &ProductionStockV1) {
        self.text(&stock.good_id);
        self.text(&stock.unit_id);
        self.text(&stock.good);
        self.text(&stock.unit);
        self.number(stock.quantity);
    }

    fn input(&mut self, input: &ProductionInputV1) {
        self.text(&input.good_id);
        self.text(&input.unit_id);
        self.text(&input.good);
        self.text(&input.unit);
        self.number(input.quantity_per_batch);
        self.number(input.on_hand);
        self.strings(&input.supplier_site_ids);
    }

    fn route(&mut self, route: &ProductionRouteV1) {
        for value in [
            &route.id,
            &route.supplier_site_id,
            &route.buyer_site_id,
            &route.good_id,
            &route.unit_id,
            &route.good,
            &route.unit,
        ] {
            self.text(value);
        }
        for value in [
            route.travel_weeks,
            route.ordered,
            route.shipped,
            route.delivered,
            route.lost,
            route.realized,
            route.backlog,
        ] {
            self.number(value);
        }
    }

    fn freight(&mut self, lot: &ProductionFreightV1) {
        for value in [
            &lot.id,
            &lot.route_id,
            &lot.source_site_id,
            &lot.destination_site_id,
            &lot.good_id,
            &lot.unit_id,
            &lot.good,
            &lot.unit,
        ] {
            self.text(value);
        }
        self.number(lot.quantity);
        self.number(lot.dispatch_week);
        self.number(lot.arrival_week);
    }

    fn event(&mut self, event: &ProductionEventV1) {
        self.text(&event.id);
        self.number(event.week);
        self.strings(&event.subject_site_ids);
        self.text(&event.kind);
        self.text(&event.description);
        self.text(&event.receipt_digest);
        self.0.update([u8::from(event.delivery_evidence.is_some())]);
        if let Some(delivery) = &event.delivery_evidence {
            self.0.update([match delivery.stage {
                crate::ProductionDeliveryStageV1::Arrival => 1,
                crate::ProductionDeliveryStageV1::Delivery => 2,
                crate::ProductionDeliveryStageV1::QuantityRealization => 3,
            }]);
            for value in [
                &delivery.order_id,
                &delivery.route_id,
                &delivery.good_id,
                &delivery.unit_id,
            ] {
                self.text(value);
            }
            self.number(delivery.quantity);
        }
    }
}

#[cfg(test)]
mod tests;
