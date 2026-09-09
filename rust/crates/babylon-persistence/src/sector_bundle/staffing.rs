//! Closed staffing authority stored with the executable sector definitions.

use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    ProcessIdV1, SiteIdV1, StaffingPolicyV1, StaffingPoolBindingV2, StaffingPoolIdV1,
    StaffingWorkSourceV2, UnitIdV1,
};
use babylon_tick::material_staffing::{StaffingCompositionV1, StaffingNodeBindingV1};
use serde::{Deserialize, Serialize};

use super::SectorBundleErrorV2;
use crate::michigan_cohorts::MICHIGAN_COHORT_SCENARIO_V2;
use crate::michigan_material::{MichiganMaterialCatalogV1, MichiganStaffingDesignV1};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
enum StoredWorkSourceV2 {
    Production([u8; 32]),
    MerchantHandling([u8; 32]),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredPoolV2 {
    scenario: String,
    local_name: String,
    pool_id: [u8; 32],
    site_id: [u8; 32],
    unit_id: [u8; 32],
    work_sources: Vec<StoredWorkSourceV2>,
    labor_force: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredStaffingV2 {
    authority: String,
    design: MichiganStaffingDesignV1,
    bindings: Vec<StoredPoolV2>,
}

impl StoredStaffingV2 {
    pub(super) fn authored(
        catalog: &MichiganMaterialCatalogV1,
    ) -> Result<Self, SectorBundleErrorV2> {
        let mut bindings = Vec::new();
        for seed in &catalog.staffing().pools {
            let site = catalog
                .site(&seed.site_key)
                .ok_or(SectorBundleErrorV2::ProcessOwnership)?;
            let mut work_sources = seed
                .process_keys
                .iter()
                .map(|key| {
                    catalog
                        .processes()
                        .iter()
                        .find(|p| p.key == *key)
                        .map(|p| StoredWorkSourceV2::Production(p.id().as_bytes()))
                        .ok_or(SectorBundleErrorV2::ProcessOwnership)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if seed.merchant_handling {
                work_sources.push(StoredWorkSourceV2::MerchantHandling(site.id().as_bytes()));
            }
            work_sources.sort();
            bindings.push(StoredPoolV2 {
                scenario: MICHIGAN_COHORT_SCENARIO_V2.to_owned(),
                local_name: seed.local_name(),
                pool_id: sha256_of(
                    format!("babylon.michigan-staffing.v1\0pool\0{}", seed.key).as_bytes(),
                ),
                site_id: site.id().as_bytes(),
                unit_id: sha256_of(b"babylon.michigan-material.v1\0unit\0labor-hour"),
                work_sources,
                labor_force: seed
                    .employed
                    .checked_add(seed.reserve)
                    .ok_or(SectorBundleErrorV2::Arithmetic)?,
            });
        }
        bindings.sort_unstable_by_key(|binding| binding.pool_id);
        Ok(Self {
            authority: "Staffed".to_owned(),
            design: catalog.staffing().clone(),
            bindings,
        })
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, SectorBundleErrorV2> {
        serde_json::to_vec(self).map_err(|_| SectorBundleErrorV2::WireNoncanonical)
    }

    pub(super) fn decode(
        bytes: &[u8],
        catalog: &MichiganMaterialCatalogV1,
    ) -> Result<Self, SectorBundleErrorV2> {
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| SectorBundleErrorV2::WireNoncanonical)?;
        if value.encode()? != bytes {
            return Err(SectorBundleErrorV2::WireNoncanonical);
        }
        // Exact source admission also proves seeds, policy, placement, principal
        // bindings and canonical ordering. A self-hash cannot grant authority.
        if value != Self::authored(catalog)? {
            return Err(SectorBundleErrorV2::Source);
        }
        value.composition()?;
        Ok(value)
    }

    #[cfg(test)]
    pub(super) fn design(&self) -> &MichiganStaffingDesignV1 {
        &self.design
    }

    pub(super) fn composition(&self) -> Result<StaffingCompositionV1, SectorBundleErrorV2> {
        let bindings = self
            .bindings
            .iter()
            .map(|binding| {
                let pool = StaffingPoolBindingV2::try_new(
                    StaffingPoolIdV1::from_bytes(binding.pool_id),
                    SiteIdV1::from_bytes(binding.site_id),
                    UnitIdV1::from_bytes(binding.unit_id),
                    binding.labor_force,
                    StaffingPolicyV1::one_period(self.design.hours_per_worker_period)
                        .map_err(|_| SectorBundleErrorV2::Resource)?,
                    binding
                        .work_sources
                        .iter()
                        .map(|source| match source {
                            StoredWorkSourceV2::Production(id) => {
                                StaffingWorkSourceV2::Production(ProcessIdV1::from_bytes(*id))
                            }
                            StoredWorkSourceV2::MerchantHandling(id) => {
                                StaffingWorkSourceV2::MerchantHandling(SiteIdV1::from_bytes(*id))
                            }
                        })
                        .collect(),
                )
                .map_err(|_| SectorBundleErrorV2::Resource)?;
                StaffingNodeBindingV1::try_new(
                    StableElementKeyV1::Node {
                        scenario: binding.scenario.clone(),
                        local_name: binding.local_name.clone(),
                    },
                    pool,
                )
                .map_err(|_| SectorBundleErrorV2::Resource)
            })
            .collect::<Result<Vec<_>, _>>()?;
        StaffingCompositionV1::try_new(bindings).map_err(|_| SectorBundleErrorV2::Resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_authority_refuses_policy_placement_seed_and_every_principal_mutation() {
        let original = StoredStaffingV2::authored(&crate::test_support::catalog()).unwrap();
        let bytes = original.encode().unwrap();
        let decoded = StoredStaffingV2::decode(&bytes, &crate::test_support::catalog()).unwrap();
        assert_eq!(decoded, original);
        assert_eq!(decoded.composition().unwrap().bindings().len(), 5);
        for change in 0..11 {
            let mut changed = original.clone();
            match change {
                0 => changed.authority = "Scheduled".to_owned(),
                1 => changed.design.hours_per_worker_period = 39,
                2 => changed.design.retention_periods = 2,
                3 => changed.design.placement = "before-metabolism".to_owned(),
                4 => changed.design.pools[0].previous_unretained_hours += 1,
                5 => changed.bindings[0].pool_id[0] ^= 1,
                6 => changed.bindings[0].site_id[0] ^= 1,
                7 => changed.bindings[0].unit_id[0] ^= 1,
                8 => match &mut changed.bindings[0].work_sources[0] {
                    StoredWorkSourceV2::Production(id)
                    | StoredWorkSourceV2::MerchantHandling(id) => id[0] ^= 1,
                },
                9 => changed.bindings[0].local_name.push('x'),
                _ => changed.design.composition_id.push('x'),
            }
            assert_eq!(
                StoredStaffingV2::decode(
                    &changed.encode().unwrap(),
                    &crate::test_support::catalog()
                ),
                Err(SectorBundleErrorV2::Source)
            );
        }
        assert!(StoredStaffingV2::decode(b"{}", &crate::test_support::catalog()).is_err());
    }
}
