//! Identity, period, and conservation checks for disclosed staffing accounts.

use babylon_persistence::production_observation::ProductionStaffingAccount;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StaffingIdentity<'a> {
    pool: &'a str,
    site: &'a str,
    unit: &'a str,
}

impl<'a> From<&'a ProductionStaffingAccount> for StaffingIdentity<'a> {
    fn from(account: &'a ProductionStaffingAccount) -> Self {
        Self {
            pool: &account.pool_id,
            site: &account.site_id,
            unit: &account.unit_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StaffingError {
    OpeningPeriod,
    FoundationReceipt,
    MissingReceipt,
    ReceiptPeriod,
    Conservation,
}

impl StaffingError {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::OpeningPeriod => "workforce account does not match the selected period",
            Self::FoundationReceipt => "foundation unexpectedly has a completed staffing receipt",
            Self::MissingReceipt => "no completed staffing receipt for the selected period",
            Self::ReceiptPeriod => "staffing receipt does not match the selected period",
            Self::Conservation => "employed and reserve do not conserve the workforce",
        }
    }
}

impl std::fmt::Display for StaffingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

pub(crate) fn validate_staffing_period(
    account: &ProductionStaffingAccount,
    tick: u64,
) -> Result<(), StaffingError> {
    if tick.checked_add(1) != Some(account.next_opening_period) {
        return Err(StaffingError::OpeningPeriod);
    }
    match (&account.completed, tick) {
        (None, 0) => Ok(()),
        (Some(_), 0) => Err(StaffingError::FoundationReceipt),
        (None, _) => Err(StaffingError::MissingReceipt),
        (Some(receipt), _) if receipt.period != tick => Err(StaffingError::ReceiptPeriod),
        (Some(_), _) => Ok(()),
    }
}

pub(crate) fn validate_staffing_balance(
    account: &ProductionStaffingAccount,
) -> Result<(), StaffingError> {
    if account.employed.checked_add(account.reserve) == Some(account.labor_force) {
        Ok(())
    } else {
        Err(StaffingError::Conservation)
    }
}
