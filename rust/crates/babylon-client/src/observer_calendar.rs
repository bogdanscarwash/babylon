//! Relative planning periods over completed weekly commits, not civil dates.
//!
//! Twelve campaign months partition a 52-week campaign year. A month ends
//! at the first weekly commit at or beyond its exact fractional boundary:
//! `ceil(month * 52 / 12)`. Thus month lengths repeat 5, 4, 4 weeks.
//! This presentation convention supplies neither an epoch nor calendar inputs
//! to the engine. A scenario horizon can interrupt a month before its end.

/// One relative campaign month, bounded by completed weekly commits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CampaignMonth {
    pub number: u64,
    pub opening_week: u64,
    pub closing_week: u64,
}

impl CampaignMonth {
    /// The month containing the next weekly commit after `week`.
    ///
    /// Returns `None` when its full closing boundary exceeds `u64`.
    #[must_use]
    pub fn after_week(week: u64) -> Option<Self> {
        let number = u128::from(week) * 12 / 52 + 1;
        Some(Self {
            number: u64::try_from(number).ok()?,
            opening_week: u64::try_from(((number - 1) * 52).div_ceil(12)).ok()?,
            closing_week: u64::try_from((number * 52).div_ceil(12)).ok()?,
        })
    }

    /// The month containing a completed week; opening week zero belongs to one.
    ///
    /// At the integer limit the displayed closing boundary is truncated to
    /// that limit. `after_week` still refuses to schedule an overflowing month.
    ///
    /// # Panics
    ///
    /// Panics on a violation of the internal partition bounds. The month number
    /// and opening boundary fit for every `u64` input under this partition.
    #[must_use]
    pub fn at_week(week: u64) -> Self {
        let number = u128::from(week.saturating_sub(1)) * 12 / 52 + 1;
        Self {
            number: u64::try_from(number).expect("month number cannot exceed week count"),
            opening_week: u64::try_from(((number - 1) * 52).div_ceil(12))
                .expect("opening precedes the represented week"),
            closing_week: u64::try_from((number * 52).div_ceil(12)).unwrap_or(u64::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_relative_months_cover_fifty_two_weekly_commits_without_drift() {
        let mut week = 0;
        for (index, boundary) in [5, 9, 13, 18, 22, 26, 31, 35, 39, 44, 48, 52]
            .into_iter()
            .enumerate()
        {
            let month = CampaignMonth::after_week(week).unwrap();
            assert_eq!(month.number, u64::try_from(index).unwrap() + 1);
            assert_eq!(month.opening_week, week);
            assert_eq!(month.closing_week, boundary);
            for committed in week + 1..=boundary {
                assert_eq!(CampaignMonth::at_week(committed), month);
                assert_eq!(CampaignMonth::after_week(committed - 1), Some(month));
            }
            week = boundary;
        }
        assert_eq!(CampaignMonth::at_week(0).number, 1);
        assert_eq!(CampaignMonth::after_week(52).unwrap().closing_week, 57);
        assert_eq!(CampaignMonth::after_week(520).unwrap().number, 121);
    }

    #[test]
    fn incomplete_month_retains_its_boundary_and_overflow_never_wraps() {
        assert_eq!(CampaignMonth::after_week(2), CampaignMonth::after_week(0));
        assert_eq!(CampaignMonth::after_week(8), CampaignMonth::after_week(5));
        assert!(CampaignMonth::after_week(u64::MAX).is_none());
        let last = CampaignMonth::at_week(u64::MAX);
        assert!(last.opening_week < u64::MAX);
        assert_eq!(last.closing_week, u64::MAX);
    }
}
