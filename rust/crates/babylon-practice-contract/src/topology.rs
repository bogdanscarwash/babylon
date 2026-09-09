//! Pure detached organization-practice topology validation.

use crate::{
    read_action_budget, OrganizationPracticeTopologyV1, PracticeContractError,
    PracticeTargetDomainV1, MAX_ORGANIZATIONS, MAX_ORG_SOLIDARITY_EDGES_PER_ORG,
};
use std::collections::{HashMap, HashSet};

/// Bounded validation-local counts for a practice topology load.
#[derive(Debug, Default)]
pub struct PracticeTopologyLoadCounter {
    organizations: HashSet<u64>,
    edge_counts: HashMap<u64, usize>,
    solidarity_edges: HashSet<(u64, u64)>,
}

impl PracticeTopologyLoadCounter {
    /// Construct an empty detached counter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe one exact organization row in constant time.
    ///
    /// # Errors
    ///
    /// Returns the organization bound/duplicate, missing-budget, or exact
    /// storage refusal.
    pub fn observe_organization(
        &mut self,
        organization_key: u64,
        active: bool,
        action_budget_storage: Option<f64>,
    ) -> Result<(), PracticeContractError> {
        if self.organizations.contains(&organization_key) {
            return Err(PracticeContractError::PracticeTopologyOrganizationDuplicate);
        }
        if self.organizations.len() == MAX_ORGANIZATIONS {
            return Err(PracticeContractError::PracticeTopologyOrganizationLimit);
        }
        if let Some(storage) = action_budget_storage {
            read_action_budget(storage)?;
        } else if active {
            return Err(PracticeContractError::PracticeTopologyBudgetMissing);
        }
        self.organizations.insert(organization_key);
        Ok(())
    }

    /// Observe one exact organization-to-social-class solidarity edge.
    ///
    /// # Errors
    ///
    /// Returns the duplicate or per-organization edge bound refusal.
    pub fn observe_solidarity_edge(
        &mut self,
        source_organization_key: u64,
        target_domain: PracticeTargetDomainV1,
        target_key: u64,
    ) -> Result<(), PracticeContractError> {
        match target_domain {
            PracticeTargetDomainV1::SocialClass => {}
        }
        let identity = (source_organization_key, target_key);
        if self.solidarity_edges.contains(&identity) {
            return Err(PracticeContractError::PracticeTopologyEdgeDuplicate);
        }
        let count = self.edge_counts.entry(source_organization_key).or_insert(0);
        if *count == MAX_ORG_SOLIDARITY_EDGES_PER_ORG {
            return Err(PracticeContractError::PracticeFootprintLimit);
        }
        *count += 1;
        self.solidarity_edges.insert(identity);
        Ok(())
    }

    /// Complete the bounded detached counter walk.
    ///
    /// # Errors
    ///
    /// Returns the organization bound if internal counts ever exceed it.
    pub fn finish(self) -> Result<(), PracticeContractError> {
        let mut observed = 0_usize;
        for _key in self.organizations.iter().take(MAX_ORGANIZATIONS + 1) {
            observed += 1;
        }
        if observed > MAX_ORGANIZATIONS {
            return Err(PracticeContractError::PracticeTopologyOrganizationLimit);
        }
        Ok(())
    }
}

/// Validate one already-qualified detached topology without graph authority.
///
/// # Errors
///
/// Returns the exact bounded identity, budget, or edge refusal.
pub fn validate_topology(
    topology: &OrganizationPracticeTopologyV1,
) -> Result<(), PracticeContractError> {
    if topology.organizations.len() > MAX_ORGANIZATIONS {
        return Err(PracticeContractError::PracticeTopologyOrganizationLimit);
    }
    let mut counter = PracticeTopologyLoadCounter::new();
    let mut previous_organization: Option<u64> = None;
    for row in topology.organizations.iter().take(MAX_ORGANIZATIONS + 1) {
        if previous_organization == Some(row.node_id_u64) {
            return Err(PracticeContractError::PracticeTopologyOrganizationDuplicate);
        }
        if previous_organization.is_some_and(|previous| row.node_id_u64 < previous) {
            return Err(PracticeContractError::PracticeTopologyOrganizationOrder);
        }
        let storage = row.action_budget_storage_f64_bits_u64.map(f64::from_bits);
        counter.observe_organization(row.node_id_u64, row.active_bool, storage)?;
        if row.edges.len() > MAX_ORG_SOLIDARITY_EDGES_PER_ORG {
            return Err(PracticeContractError::PracticeFootprintLimit);
        }
        let mut previous_target: Option<u64> = None;
        for edge in row.edges.iter().take(MAX_ORG_SOLIDARITY_EDGES_PER_ORG + 1) {
            if previous_target == Some(edge.target_class_node_id_u64) {
                return Err(PracticeContractError::PracticeTopologyEdgeDuplicate);
            }
            if previous_target.is_some_and(|previous| edge.target_class_node_id_u64 < previous) {
                return Err(PracticeContractError::PracticeTopologyEdgeOrder);
            }
            counter.observe_solidarity_edge(
                row.node_id_u64,
                edge.target_domain,
                edge.target_class_node_id_u64,
            )?;
            previous_target = Some(edge.target_class_node_id_u64);
        }
        previous_organization = Some(row.node_id_u64);
    }
    counter.finish()
}
