//! Immutable staffing inputs, conserved stocks and completed evidence.

use std::collections::BTreeSet;

use crate::{LaborCapacityRowV1, ProcessIdV1, SiteIdV1, UnitIdV1, MAX_MATERIAL_CIRCUIT_ROWS_V1};

crate::model::identity_type!(StaffingPoolIdV1);

/// Closed refusals; no partial staffing transition is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum StaffingErrorV1 {
    RowLimit = 1,
    EmptyProcesses = 2,
    ZeroSchedule = 3,
    DuplicatePool = 4,
    DuplicateSiteUnit = 5,
    DuplicateProcess = 6,
    PopulationInvariant = 7,
    PeriodInvariant = 8,
    Arithmetic = 9,
    UnknownRequest = 10,
    RequestBinding = 11,
    DuplicateRequest = 12,
    MissingRequest = 13,
    Allocation = 14,
}

impl std::fmt::Display for StaffingErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "staffing refused: {self:?}")
    }
}
impl std::error::Error for StaffingErrorV1 {}

/// An explicit schedule under V1's one-period retention rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffingPolicyV1 {
    hours_per_person: u64,
}
impl StaffingPolicyV1 {
    /// Retain the larger of current and preceding unretained work requests.
    /// # Errors
    /// Refuses a zero work schedule; no default schedule is inferred.
    pub const fn one_period(hours_per_person: u64) -> Result<Self, StaffingErrorV1> {
        if hours_per_person == 0 {
            return Err(StaffingErrorV1::ZeroSchedule);
        }
        Ok(Self { hours_per_person })
    }
    #[must_use]
    pub const fn hours_per_person(self) -> u64 {
        self.hours_per_person
    }
}

/// One nonduplicated person principal serving a single site and labor unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingPoolBindingV1 {
    pool_id: StaffingPoolIdV1,
    site_id: SiteIdV1,
    unit_id: UnitIdV1,
    labor_force: u64,
    policy: StaffingPolicyV1,
    processes: Vec<ProcessIdV1>,
}
impl StaffingPoolBindingV1 {
    /// Declare the complete process membership and labor force explicitly.
    /// # Errors
    /// Refuses empty, duplicate or over-bound process membership.
    pub fn try_new(
        pool_id: StaffingPoolIdV1,
        site_id: SiteIdV1,
        unit_id: UnitIdV1,
        labor_force: u64,
        policy: StaffingPolicyV1,
        mut processes: Vec<ProcessIdV1>,
    ) -> Result<Self, StaffingErrorV1> {
        if processes.is_empty() {
            return Err(StaffingErrorV1::EmptyProcesses);
        }
        if processes.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
            return Err(StaffingErrorV1::RowLimit);
        }
        processes.sort_unstable();
        if processes.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(StaffingErrorV1::DuplicateProcess);
        }
        Ok(Self {
            pool_id,
            site_id,
            unit_id,
            labor_force,
            policy,
            processes,
        })
    }
    #[must_use]
    pub const fn pool_id(&self) -> StaffingPoolIdV1 {
        self.pool_id
    }
    #[must_use]
    pub const fn site_id(&self) -> SiteIdV1 {
        self.site_id
    }
    #[must_use]
    pub const fn unit_id(&self) -> UnitIdV1 {
        self.unit_id
    }
    #[must_use]
    pub const fn labor_force(&self) -> u64 {
        self.labor_force
    }
    #[must_use]
    pub const fn policy(&self) -> StaffingPolicyV1 {
        self.policy
    }
    #[must_use]
    pub fn processes(&self) -> &[ProcessIdV1] {
        &self.processes
    }
}

/// Opening person stocks and the previous unretained labor-time request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingPoolStateV1 {
    binding: StaffingPoolBindingV1,
    employed: u64,
    reserve: u64,
    previous_unretained_hours: u64,
}
impl StaffingPoolStateV1 {
    /// Capture exact stocks; retention memory is an explicit caller input.
    /// # Errors
    /// Refuses a nonconserved population or an unrepresentable opening schedule.
    pub fn try_new(
        binding: StaffingPoolBindingV1,
        employed: u64,
        reserve: u64,
        previous_unretained_hours: u64,
    ) -> Result<Self, StaffingErrorV1> {
        if employed
            .checked_add(reserve)
            .ok_or(StaffingErrorV1::Arithmetic)?
            != binding.labor_force
        {
            return Err(StaffingErrorV1::PopulationInvariant);
        }
        employed
            .checked_mul(binding.policy.hours_per_person)
            .ok_or(StaffingErrorV1::Arithmetic)?;
        Ok(Self {
            binding,
            employed,
            reserve,
            previous_unretained_hours,
        })
    }
    #[must_use]
    pub const fn binding(&self) -> &StaffingPoolBindingV1 {
        &self.binding
    }
    #[must_use]
    pub const fn employed(&self) -> u64 {
        self.employed
    }
    #[must_use]
    pub const fn reserve(&self) -> u64 {
        self.reserve
    }
    #[must_use]
    pub const fn previous_unretained_hours(&self) -> u64 {
        self.previous_unretained_hours
    }
}

/// Complete immutable opening staffing state, ordered by pool identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingStateV1 {
    period: u64,
    pools: Vec<StaffingPoolStateV1>,
}
impl StaffingStateV1 {
    /// Validate one owner per pool, site/unit and process across the state.
    /// # Errors
    /// Refuses period zero, row bounds or overlapping ownership.
    pub fn try_new(
        period: u64,
        mut pools: Vec<StaffingPoolStateV1>,
    ) -> Result<Self, StaffingErrorV1> {
        if period == 0 {
            return Err(StaffingErrorV1::PeriodInvariant);
        }
        if pools.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
            return Err(StaffingErrorV1::RowLimit);
        }
        pools.sort_unstable_by_key(|pool| pool.binding.pool_id);
        let mut site_units = BTreeSet::new();
        let mut processes = BTreeSet::new();
        let mut previous_pool = None;
        for pool in &pools {
            let binding = &pool.binding;
            if previous_pool == Some(binding.pool_id) {
                return Err(StaffingErrorV1::DuplicatePool);
            }
            previous_pool = Some(binding.pool_id);
            if !site_units.insert((binding.site_id, binding.unit_id)) {
                return Err(StaffingErrorV1::DuplicateSiteUnit);
            }
            for process in &binding.processes {
                if !processes.insert(*process) {
                    return Err(StaffingErrorV1::DuplicateProcess);
                }
                if processes.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
                    return Err(StaffingErrorV1::RowLimit);
                }
            }
        }
        Ok(Self { period, pools })
    }
    #[must_use]
    pub const fn period(&self) -> u64 {
        self.period
    }
    #[must_use]
    pub fn pools(&self) -> &[StaffingPoolStateV1] {
        &self.pools
    }
}

/// One explicit process request, including an explicit zero for no work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffingWorkRequestV1 {
    period: u64,
    pool_id: StaffingPoolIdV1,
    process_id: ProcessIdV1,
    site_id: SiteIdV1,
    unit_id: UnitIdV1,
    hours: u64,
}
impl StaffingWorkRequestV1 {
    #[must_use]
    pub const fn new(
        period: u64,
        pool_id: StaffingPoolIdV1,
        process_id: ProcessIdV1,
        site_id: SiteIdV1,
        unit_id: UnitIdV1,
        hours: u64,
    ) -> Self {
        Self {
            period,
            pool_id,
            process_id,
            site_id,
            unit_id,
            hours,
        }
    }
    #[must_use]
    pub const fn period(self) -> u64 {
        self.period
    }
    #[must_use]
    pub const fn pool_id(self) -> StaffingPoolIdV1 {
        self.pool_id
    }
    #[must_use]
    pub const fn process_id(self) -> ProcessIdV1 {
        self.process_id
    }
    #[must_use]
    pub const fn site_id(self) -> SiteIdV1 {
        self.site_id
    }
    #[must_use]
    pub const fn unit_id(self) -> UnitIdV1 {
        self.unit_id
    }
    #[must_use]
    pub const fn hours(self) -> u64 {
        self.hours
    }
}

/// Exact completed account; V1 has no mortality, migration or inactivity flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingReceiptV1 {
    period: u64,
    binding: StaffingPoolBindingV1,
    opening_employed: u64,
    opening_reserve: u64,
    previous_unretained_hours: u64,
    current_unretained_hours: u64,
    retained_hours: u64,
    target_employed: u64,
    hires: u64,
    separations: u64,
    closing_employed: u64,
    closing_reserve: u64,
    next_opening_hours: u64,
}
impl StaffingReceiptV1 {
    pub(super) fn from_transition(
        period: u64,
        opening: &StaffingPoolStateV1,
        closing: &StaffingPoolStateV1,
        current_unretained_hours: u64,
        retained_hours: u64,
        target_employed: u64,
        next_opening_hours: u64,
    ) -> Self {
        let (hires, separations) = if closing.employed >= opening.employed {
            (closing.employed - opening.employed, 0)
        } else {
            (0, opening.employed - closing.employed)
        };
        Self {
            period,
            binding: opening.binding.clone(),
            opening_employed: opening.employed,
            opening_reserve: opening.reserve,
            previous_unretained_hours: opening.previous_unretained_hours,
            current_unretained_hours,
            retained_hours,
            target_employed,
            hires,
            separations,
            closing_employed: closing.employed,
            closing_reserve: closing.reserve,
            next_opening_hours,
        }
    }
    #[must_use]
    pub const fn period(&self) -> u64 {
        self.period
    }
    #[must_use]
    pub const fn binding(&self) -> &StaffingPoolBindingV1 {
        &self.binding
    }
    #[must_use]
    pub const fn opening_employed(&self) -> u64 {
        self.opening_employed
    }
    #[must_use]
    pub const fn opening_reserve(&self) -> u64 {
        self.opening_reserve
    }
    #[must_use]
    pub const fn previous_unretained_hours(&self) -> u64 {
        self.previous_unretained_hours
    }
    #[must_use]
    pub const fn current_unretained_hours(&self) -> u64 {
        self.current_unretained_hours
    }
    #[must_use]
    pub const fn retained_hours(&self) -> u64 {
        self.retained_hours
    }
    #[must_use]
    pub const fn target_employed(&self) -> u64 {
        self.target_employed
    }
    #[must_use]
    pub const fn hires(&self) -> u64 {
        self.hires
    }
    #[must_use]
    pub const fn separations(&self) -> u64 {
        self.separations
    }
    #[must_use]
    pub const fn closing_employed(&self) -> u64 {
        self.closing_employed
    }
    #[must_use]
    pub const fn closing_reserve(&self) -> u64 {
        self.closing_reserve
    }
    #[must_use]
    pub const fn next_opening_hours(&self) -> u64 {
        self.next_opening_hours
    }
}

/// A complete detached successor; constructing evidence never publishes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingTransitionV1 {
    state: StaffingStateV1,
    receipts: Vec<StaffingReceiptV1>,
    labor: Vec<LaborCapacityRowV1>,
}
impl StaffingTransitionV1 {
    pub(super) fn new(
        state: StaffingStateV1,
        receipts: Vec<StaffingReceiptV1>,
        labor: Vec<LaborCapacityRowV1>,
    ) -> Self {
        Self {
            state,
            receipts,
            labor,
        }
    }
    #[must_use]
    pub const fn state(&self) -> &StaffingStateV1 {
        &self.state
    }
    #[must_use]
    pub fn receipts(&self) -> &[StaffingReceiptV1] {
        &self.receipts
    }
    /// Proposed exact budgets, ordered by site/unit, for the following period.
    #[must_use]
    pub fn next_labor(&self) -> &[LaborCapacityRowV1] {
        &self.labor
    }
    #[must_use]
    pub fn into_state(self) -> StaffingStateV1 {
        self.state
    }
}
