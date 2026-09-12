//! Immutable staffing inputs, conserved stocks and completed evidence.

use std::collections::BTreeSet;

use crate::{LaborCapacityRow, ProcessId, SiteId, UnitId, MAX_MATERIAL_CIRCUIT_ROWS};

crate::model::identity_type!(StaffingPoolId);

/// The material activity served by a staffing pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StaffingWorkSource {
    Production(ProcessId),
    MerchantHandling(SiteId),
}

/// Closed refusals; no partial staffing transition is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum StaffingError {
    RowLimit = 1,
    EmptyWorkSources = 2,
    ZeroSchedule = 3,
    DuplicatePool = 4,
    DuplicateSiteUnit = 5,
    DuplicateWorkSource = 6,
    PopulationInvariant = 7,
    PeriodInvariant = 8,
    Arithmetic = 9,
    UnknownRequest = 10,
    RequestBinding = 11,
    DuplicateRequest = 12,
    MissingRequest = 13,
    Allocation = 14,
}

impl std::fmt::Display for StaffingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "staffing refused: {self:?}")
    }
}
impl std::error::Error for StaffingError {}

/// An explicit schedule under V1's one-period retention rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffingPolicy {
    hours_per_person: u64,
}
impl StaffingPolicy {
    /// Retain the larger of current and preceding unretained work requests.
    /// # Errors
    /// Refuses a zero work schedule; no default schedule is inferred.
    pub const fn one_period(hours_per_person: u64) -> Result<Self, StaffingError> {
        if hours_per_person == 0 {
            return Err(StaffingError::ZeroSchedule);
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
pub struct StaffingPoolBinding {
    pool_id: StaffingPoolId,
    site_id: SiteId,
    unit_id: UnitId,
    labor_force: u64,
    policy: StaffingPolicy,
    work_sources: Vec<StaffingWorkSource>,
}
impl StaffingPoolBinding {
    /// Declare the complete work-source membership and labor force explicitly.
    /// # Errors
    /// Refuses empty, duplicate or over-bound work-source membership.
    pub fn try_new(
        pool_id: StaffingPoolId,
        site_id: SiteId,
        unit_id: UnitId,
        labor_force: u64,
        policy: StaffingPolicy,
        mut work_sources: Vec<StaffingWorkSource>,
    ) -> Result<Self, StaffingError> {
        if work_sources.is_empty() {
            return Err(StaffingError::EmptyWorkSources);
        }
        if work_sources.len() > MAX_MATERIAL_CIRCUIT_ROWS {
            return Err(StaffingError::RowLimit);
        }
        work_sources.sort_unstable();
        if work_sources.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(StaffingError::DuplicateWorkSource);
        }
        Ok(Self {
            pool_id,
            site_id,
            unit_id,
            labor_force,
            policy,
            work_sources,
        })
    }
    #[must_use]
    pub const fn pool_id(&self) -> StaffingPoolId {
        self.pool_id
    }
    #[must_use]
    pub const fn site_id(&self) -> SiteId {
        self.site_id
    }
    #[must_use]
    pub const fn unit_id(&self) -> UnitId {
        self.unit_id
    }
    #[must_use]
    pub const fn labor_force(&self) -> u64 {
        self.labor_force
    }
    #[must_use]
    pub const fn policy(&self) -> StaffingPolicy {
        self.policy
    }
    #[must_use]
    pub fn work_sources(&self) -> &[StaffingWorkSource] {
        &self.work_sources
    }
}

/// Opening person stocks and the previous unretained labor-time request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingPoolState {
    binding: StaffingPoolBinding,
    employed: u64,
    reserve: u64,
    previous_unretained_hours: u64,
}
impl StaffingPoolState {
    /// Capture exact stocks; retention memory is an explicit caller input.
    /// # Errors
    /// Refuses a nonconserved population or an unrepresentable opening schedule.
    pub fn try_new(
        binding: StaffingPoolBinding,
        employed: u64,
        reserve: u64,
        previous_unretained_hours: u64,
    ) -> Result<Self, StaffingError> {
        if employed
            .checked_add(reserve)
            .ok_or(StaffingError::Arithmetic)?
            != binding.labor_force
        {
            return Err(StaffingError::PopulationInvariant);
        }
        employed
            .checked_mul(binding.policy.hours_per_person)
            .ok_or(StaffingError::Arithmetic)?;
        Ok(Self {
            binding,
            employed,
            reserve,
            previous_unretained_hours,
        })
    }
    #[must_use]
    pub const fn binding(&self) -> &StaffingPoolBinding {
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
pub struct StaffingState {
    period: u64,
    pools: Vec<StaffingPoolState>,
}
impl StaffingState {
    /// Validate one owner per pool, site/unit and work source across the state.
    /// # Errors
    /// Refuses period zero, row bounds or overlapping ownership.
    pub fn try_new(period: u64, mut pools: Vec<StaffingPoolState>) -> Result<Self, StaffingError> {
        if period == 0 {
            return Err(StaffingError::PeriodInvariant);
        }
        if pools.len() > MAX_MATERIAL_CIRCUIT_ROWS {
            return Err(StaffingError::RowLimit);
        }
        pools.sort_unstable_by_key(|pool| pool.binding.pool_id);
        let mut site_units = BTreeSet::new();
        let mut work_sources = BTreeSet::new();
        let mut previous_pool = None;
        for pool in &pools {
            let binding = &pool.binding;
            if previous_pool == Some(binding.pool_id) {
                return Err(StaffingError::DuplicatePool);
            }
            previous_pool = Some(binding.pool_id);
            if !site_units.insert((binding.site_id, binding.unit_id)) {
                return Err(StaffingError::DuplicateSiteUnit);
            }
            for source in &binding.work_sources {
                if !work_sources.insert(*source) {
                    return Err(StaffingError::DuplicateWorkSource);
                }
                if work_sources.len() > MAX_MATERIAL_CIRCUIT_ROWS {
                    return Err(StaffingError::RowLimit);
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
    pub fn pools(&self) -> &[StaffingPoolState] {
        &self.pools
    }
}

/// One explicit material work request, including an explicit zero for no work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffingWorkRequest {
    period: u64,
    pool_id: StaffingPoolId,
    work_source: StaffingWorkSource,
    site_id: SiteId,
    unit_id: UnitId,
    hours: u64,
}
impl StaffingWorkRequest {
    #[must_use]
    pub const fn new(
        period: u64,
        pool_id: StaffingPoolId,
        work_source: StaffingWorkSource,
        site_id: SiteId,
        unit_id: UnitId,
        hours: u64,
    ) -> Self {
        Self {
            period,
            pool_id,
            work_source,
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
    pub const fn pool_id(self) -> StaffingPoolId {
        self.pool_id
    }
    #[must_use]
    pub const fn work_source(self) -> StaffingWorkSource {
        self.work_source
    }
    #[must_use]
    pub const fn site_id(self) -> SiteId {
        self.site_id
    }
    #[must_use]
    pub const fn unit_id(self) -> UnitId {
        self.unit_id
    }
    #[must_use]
    pub const fn hours(self) -> u64 {
        self.hours
    }
}

/// Exact completed account; V1 has no mortality, migration or inactivity flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffingReceipt {
    period: u64,
    binding: StaffingPoolBinding,
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
impl StaffingReceipt {
    pub(super) fn from_transition(
        period: u64,
        opening: &StaffingPoolState,
        closing: &StaffingPoolState,
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
    pub const fn binding(&self) -> &StaffingPoolBinding {
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
pub struct StaffingTransition {
    state: StaffingState,
    receipts: Vec<StaffingReceipt>,
    labor: Vec<LaborCapacityRow>,
}
impl StaffingTransition {
    pub(super) fn new(
        state: StaffingState,
        receipts: Vec<StaffingReceipt>,
        labor: Vec<LaborCapacityRow>,
    ) -> Self {
        Self {
            state,
            receipts,
            labor,
        }
    }
    #[must_use]
    pub const fn state(&self) -> &StaffingState {
        &self.state
    }
    #[must_use]
    pub fn receipts(&self) -> &[StaffingReceipt] {
        &self.receipts
    }
    /// Proposed exact budgets, ordered by site/unit, for the following period.
    #[must_use]
    pub fn next_labor(&self) -> &[LaborCapacityRow] {
        &self.labor
    }
    #[must_use]
    pub fn into_state(self) -> StaffingState {
        self.state
    }
}
