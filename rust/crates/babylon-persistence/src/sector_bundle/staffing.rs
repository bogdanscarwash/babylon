//! Closed staffing authority stored with the executable sector definitions.

use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    ProcessIdV1, SiteIdV1, StaffingPolicyV1, StaffingPoolBindingV1, StaffingPoolIdV1, UnitIdV1,
};
use babylon_tick::material_staffing::{StaffingCompositionV1, StaffingNodeBindingV1};
use serde::{Deserialize, Serialize};

use super::SectorBundleErrorV1;
use crate::michigan_cohorts::MICHIGAN_COHORT_SCENARIO_V2;
use crate::michigan_material::{michigan_material_catalog_v1, MichiganStaffingDesignV1};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredPoolV1 {
    scenario: String,
    local_name: String,
    pool_id: [u8; 32],
    site_id: [u8; 32],
    unit_id: [u8; 32],
    process_ids: Vec<[u8; 32]>,
    labor_force: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredStaffingV1 {
    authority: String,
    design: MichiganStaffingDesignV1,
    bindings: Vec<StoredPoolV1>,
}

impl StoredStaffingV1 {
    pub(super) fn authored() -> Result<Self, SectorBundleErrorV1> {
        let catalog = michigan_material_catalog_v1().map_err(|_| SectorBundleErrorV1::Source)?;
        let mut bindings = Vec::new();
        for seed in &catalog.staffing().pools {
            let process = catalog
                .processes()
                .iter()
                .find(|p| p.key == seed.process_key)
                .ok_or(SectorBundleErrorV1::ProcessOwnership)?;
            bindings.push(StoredPoolV1 {
                scenario: MICHIGAN_COHORT_SCENARIO_V2.to_owned(),
                local_name: seed.local_name(),
                pool_id: sha256_of(
                    format!("babylon.michigan-staffing.v1\0pool\0{}", seed.process_key).as_bytes(),
                ),
                site_id: process.site_id().as_bytes(),
                unit_id: sha256_of(b"babylon.michigan-material.v1\0unit\0labor-hour"),
                process_ids: vec![process.id().as_bytes()],
                labor_force: seed
                    .employed
                    .checked_add(seed.reserve)
                    .ok_or(SectorBundleErrorV1::Arithmetic)?,
            });
        }
        bindings.sort_unstable_by_key(|binding| binding.pool_id);
        Ok(Self {
            authority: "Staffed".to_owned(),
            design: catalog.staffing().clone(),
            bindings,
        })
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, SectorBundleErrorV1> {
        serde_json::to_vec(self).map_err(|_| SectorBundleErrorV1::WireNoncanonical)
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, SectorBundleErrorV1> {
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| SectorBundleErrorV1::WireNoncanonical)?;
        if value.encode()? != bytes {
            return Err(SectorBundleErrorV1::WireNoncanonical);
        }
        // Exact source admission also proves seeds, policy, placement, principal
        // bindings and canonical ordering. A self-hash cannot grant authority.
        if value != Self::authored()? {
            return Err(SectorBundleErrorV1::Source);
        }
        value.composition()?;
        Ok(value)
    }

    pub(super) fn design(&self) -> &MichiganStaffingDesignV1 {
        &self.design
    }

    pub(super) fn composition(&self) -> Result<StaffingCompositionV1, SectorBundleErrorV1> {
        let bindings = self
            .bindings
            .iter()
            .map(|binding| {
                let pool = StaffingPoolBindingV1::try_new(
                    StaffingPoolIdV1::from_bytes(binding.pool_id),
                    SiteIdV1::from_bytes(binding.site_id),
                    UnitIdV1::from_bytes(binding.unit_id),
                    binding.labor_force,
                    StaffingPolicyV1::one_week(self.design.hours_per_worker_week)
                        .map_err(|_| SectorBundleErrorV1::Resource)?,
                    binding
                        .process_ids
                        .iter()
                        .map(|id| ProcessIdV1::from_bytes(*id))
                        .collect(),
                )
                .map_err(|_| SectorBundleErrorV1::Resource)?;
                StaffingNodeBindingV1::try_new(
                    StableElementKeyV1::Node {
                        scenario: binding.scenario.clone(),
                        local_name: binding.local_name.clone(),
                    },
                    pool,
                )
                .map_err(|_| SectorBundleErrorV1::Resource)
            })
            .collect::<Result<Vec<_>, _>>()?;
        StaffingCompositionV1::try_new(bindings).map_err(|_| SectorBundleErrorV1::Resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_authority_refuses_policy_placement_seed_and_every_principal_mutation() {
        let original = StoredStaffingV1::authored().unwrap();
        let bytes = original.encode().unwrap();
        let decoded = StoredStaffingV1::decode(&bytes).unwrap();
        assert_eq!(decoded, original);
        assert_eq!(decoded.composition().unwrap().bindings().len(), 5);
        for change in 0..11 {
            let mut changed = original.clone();
            match change {
                0 => changed.authority = "Scheduled".to_owned(),
                1 => changed.design.hours_per_worker_week = 39,
                2 => changed.design.retention_weeks = 2,
                3 => changed.design.placement = "before-metabolism".to_owned(),
                4 => changed.design.pools[0].previous_unretained_hours += 1,
                5 => changed.bindings[0].pool_id[0] ^= 1,
                6 => changed.bindings[0].site_id[0] ^= 1,
                7 => changed.bindings[0].unit_id[0] ^= 1,
                8 => changed.bindings[0].process_ids[0][0] ^= 1,
                9 => changed.bindings[0].local_name.push('x'),
                _ => changed.design.composition_id.push('x'),
            }
            assert_eq!(
                StoredStaffingV1::decode(&changed.encode().unwrap()),
                Err(SectorBundleErrorV1::Source)
            );
        }
        assert!(StoredStaffingV1::decode(b"{}").is_err());
    }
}
