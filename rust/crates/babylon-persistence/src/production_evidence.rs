//! V6 identity of an already-authorized production presentation.
//!
//! Scope and the complete typed DTO are serialized as canonical JSON after the
//! fixed domain/version. True multisets sort; events, geometry vertices and each
//! route's physical edge sequence retain their semantic order. Serialization
//! streams into the hash with an explicit byte ceiling. This replaces the V5
//! field-by-field encoder; historical digest bytes keep their historical meaning.

use crate::{ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionSnapshotV2};
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

const DOMAIN: &[u8] = b"babylon.production-observation-evidence.v6\0";
const MAX_ROWS: usize = 65_536;
const MAX_PHYSICAL_ROWS: usize = 1_114_112;
const MAX_EVIDENCE_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProductionEvidenceDigestV6([u8; 32]);
impl ProductionEvidenceDigestV6 {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    #[must_use]
    pub fn to_hex(self) -> String {
        crate::michigan_economy::digest_hex(&self.0)
    }
}

/// A malformed disclosure cannot acquire a production evidence identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductionEvidenceErrorV6 {
    InvalidIdentity,
    Bound,
    Serialization,
}
impl std::fmt::Display for ProductionEvidenceErrorV6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "production evidence refused: {self:?}")
    }
}
impl std::error::Error for ProductionEvidenceErrorV6 {}

type Result<T> = std::result::Result<T, ProductionEvidenceErrorV6>;

#[derive(Serialize)]
struct EvidenceScopeV6<'a> {
    campaign_id: &'a str,
    resolve_tick: u64,
    foundation_digest: &'a str,
    tick_content_hash: Option<&'a str>,
    envelope_digest: Option<&'a str>,
    nominal_world_hash: Option<&'a str>,
    visibility: &'static str,
    production: &'a ProductionSnapshotV2,
}

impl ObserverEconomySnapshotV1 {
    /// Hash the complete role-scoped disclosure after observation authentication.
    /// `None` means no production was disclosed, including a restricted preview.
    ///
    /// # Errors
    /// Refuses duplicate identities, malformed preview disclosure, row/byte bounds
    /// and serialization errors; failure is never reported as absent production.
    pub fn production_evidence_digest(&self) -> Result<Option<ProductionEvidenceDigestV6>> {
        let Some(source) = &self.production else {
            return Ok(None);
        };
        if self.visibility != ObserverVisibilityV1::FullObserver {
            return Err(ProductionEvidenceErrorV6::InvalidIdentity);
        }
        validate_identities(source)?;
        let production = canonical_production(source);
        let scope = EvidenceScopeV6 {
            campaign_id: &self.campaign_id,
            resolve_tick: self.resolve_tick,
            foundation_digest: &self.foundation_digest,
            tick_content_hash: self.tick_content_hash.as_deref(),
            envelope_digest: self.envelope_digest.as_deref(),
            nominal_world_hash: self.nominal_world_hash.as_deref(),
            visibility: "full_observer",
            production: &production,
        };
        let mut output = EvidenceWriter {
            hash: Sha256::new(),
            remaining: MAX_EVIDENCE_BYTES,
            bound: false,
        };
        output.hash.update(DOMAIN);
        output.hash.update(6_u32.to_be_bytes());
        if serde_json::to_writer(&mut output, &scope).is_err() {
            return Err(if output.bound {
                ProductionEvidenceErrorV6::Bound
            } else {
                ProductionEvidenceErrorV6::Serialization
            });
        }
        Ok(Some(ProductionEvidenceDigestV6(
            output.hash.finalize().into(),
        )))
    }
}

struct EvidenceWriter {
    hash: Sha256,
    remaining: usize,
    bound: bool,
}
impl Write for EvidenceWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.remaining {
            self.bound = true;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "production evidence byte bound",
            ));
        }
        self.remaining -= bytes.len();
        self.hash.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn unique<T: Ord>(rows: impl IntoIterator<Item = T>) -> Result<()> {
    let mut ids = BTreeSet::new();
    for id in rows {
        if !ids.insert(id) {
            return Err(ProductionEvidenceErrorV6::InvalidIdentity);
        }
    }
    Ok(())
}

fn validate_identities(rows: &ProductionSnapshotV2) -> Result<()> {
    for count in [
        rows.sites.len(),
        rows.routes.len(),
        rows.freight.len(),
        rows.labor_accounts.len(),
        rows.staffing_accounts.len(),
        rows.freight_capacity_accounts.len(),
        rows.merchant_handling_accounts.len(),
        rows.final_demand_accounts.len(),
        rows.observed_contexts.len(),
        rows.process_attributions.len(),
    ] {
        if count > MAX_ROWS {
            return Err(ProductionEvidenceErrorV6::Bound);
        }
    }
    if rows.physical_edges.len() > MAX_PHYSICAL_ROWS || rows.events.len() > MAX_PHYSICAL_ROWS {
        return Err(ProductionEvidenceErrorV6::Bound);
    }
    unique(rows.sites.iter().map(|row| &row.id))?;
    unique(
        rows.sites
            .iter()
            .flat_map(|row| row.processes.iter().map(|row| &row.id)),
    )?;
    unique(rows.routes.iter().map(|row| &row.id))?;
    unique(rows.freight.iter().map(|row| &row.id))?;
    unique(rows.events.iter().map(|row| &row.id))?;
    unique(rows.physical_edges.iter().map(|row| &row.id))?;
    unique(
        rows.labor_accounts
            .iter()
            .map(|row| (&row.site_id, &row.unit_id)),
    )?;
    unique(rows.staffing_accounts.iter().map(|row| &row.pool_id))?;
    unique(
        rows.staffing_accounts
            .iter()
            .map(|row| (&row.site_id, &row.unit_id)),
    )?;
    unique(
        rows.freight_capacity_accounts
            .iter()
            .map(|row| &row.corridor_id),
    )?;
    unique(
        rows.merchant_handling_accounts
            .iter()
            .map(|row| &row.site_id),
    )?;
    unique(
        rows.final_demand_accounts
            .iter()
            .map(|row| (&row.demand_principal_id, &row.good_id, &row.unit_id)),
    )?;
    for site in &rows.sites {
        if site.processes.len() > MAX_ROWS || site.inventory.len() > MAX_ROWS {
            return Err(ProductionEvidenceErrorV6::Bound);
        }
        unique(
            site.inventory
                .iter()
                .map(|row| (&row.good_id, &row.unit_id)),
        )?;
        for process in &site.processes {
            unique(
                process
                    .inputs
                    .iter()
                    .map(|row| (&row.good_id, &row.unit_id)),
            )?;
        }
    }
    for route in &rows.routes {
        if route.physical_edge_ids.len() > MAX_PHYSICAL_ROWS || route.stages.len() > 16 {
            return Err(ProductionEvidenceErrorV6::Bound);
        }
        unique(route.stages.iter().map(|row| row.stage_index))?;
        for stage in &route.stages {
            unique(&stage.capacity_ids)?;
        }
    }
    validate_account_rows(rows)
}

fn validate_account_rows(rows: &ProductionSnapshotV2) -> Result<()> {
    for account in &rows.freight_capacity_accounts {
        unique(&account.route_ids)?;
        unique(&account.merchant_site_ids)?;
        if let Some(completed) = &account.completed {
            unique(
                completed
                    .reservations
                    .iter()
                    .map(|row| row.reservation_period),
            )?;
            for row in &completed.reservations {
                unique(row.orders.iter().map(|row| (row.kind, &row.order_id)))?;
            }
        }
    }
    for account in &rows.merchant_handling_accounts {
        unique(
            account
                .coefficients
                .iter()
                .map(|row| (&row.good_id, &row.unit_id)),
        )?;
        if let Some(completed) = &account.completed {
            unique(completed.orders.iter().map(|row| (row.kind, &row.order_id)))?;
        }
    }
    for account in &rows.final_demand_accounts {
        unique(&account.retailer_site_ids)?;
        unique(account.orders.iter().map(|row| &row.order_id))?;
    }
    if let Some(balance) = &rows.material_balance {
        unique(
            balance
                .rows
                .iter()
                .map(|row| (&row.site_id, &row.good_id, &row.unit_id)),
        )?;
    }
    Ok(())
}

fn canonical_production(source: &ProductionSnapshotV2) -> ProductionSnapshotV2 {
    let mut rows = source.clone();
    for site in &mut rows.sites {
        site.inventory.sort_unstable();
        for process in &mut site.processes {
            for input in &mut process.inputs {
                input.supplier_site_ids.sort_unstable();
            }
            process.inputs.sort_unstable();
            process.labor.sort_unstable();
        }
        site.processes.sort_unstable();
    }
    rows.sites.sort_unstable();
    rows.labor_accounts.sort_unstable();
    rows.staffing_accounts.sort_unstable();
    if let Some(balance) = &mut rows.material_balance {
        balance.rows.sort_unstable();
    }
    rows.observed_contexts.sort_unstable();
    rows.process_attributions.sort_unstable();
    rows.physical_edges.sort_unstable();
    for route in &mut rows.routes {
        for stage in &mut route.stages {
            stage.capacity_ids.sort_unstable();
        }
        route.stages.sort_unstable();
    }
    rows.routes.sort_unstable();
    for account in &mut rows.freight_capacity_accounts {
        account.route_ids.sort_unstable();
        account.merchant_site_ids.sort_unstable();
        if let Some(completed) = &mut account.completed {
            for reservation in &mut completed.reservations {
                reservation.orders.sort_unstable();
            }
            completed.reservations.sort_unstable();
        }
    }
    rows.freight_capacity_accounts.sort_unstable();
    for account in &mut rows.merchant_handling_accounts {
        account.coefficients.sort_unstable();
        if let Some(completed) = &mut account.completed {
            completed.orders.sort_unstable();
        }
    }
    rows.merchant_handling_accounts.sort_unstable();
    for account in &mut rows.final_demand_accounts {
        account.orders.sort_unstable();
        account.retailer_site_ids.sort_unstable();
    }
    rows.final_demand_accounts.sort_unstable();
    rows.freight.sort_unstable();
    for event in &mut rows.events {
        event.subject_site_ids.sort_unstable();
    }
    rows.provenance.sort_unstable();
    rows
}

#[cfg(test)]
mod tests;
