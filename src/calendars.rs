//! Simple structural calendars.

use std::collections::BTreeSet;

use jiff::civil::{Date, Weekday};
use jiff::tz::TimeZone;

use crate::{HolidayCalendar, OutOfRangeError, RangeBound, check_range};

/// A calendar with no holidays — every day is a business day.
///
/// Useful as a baseline for time series consistency checking, or when
/// holiday adjustments are not required.
pub struct NoHolidays {
    tz: TimeZone,
}

impl NoHolidays {
    /// Creates a new `NoHolidays` calendar with UTC time zone.
    pub fn new() -> Self {
        Self { tz: TimeZone::UTC }
    }

    /// Creates a new `NoHolidays` calendar with the given time zone.
    pub fn with_tz(tz: TimeZone) -> Self {
        Self { tz }
    }
}

impl Default for NoHolidays {
    fn default() -> Self {
        Self::new()
    }
}

impl HolidayCalendar for NoHolidays {
    fn name(&self) -> &str {
        "NoHolidays"
    }

    fn tz(&self) -> &TimeZone {
        &self.tz
    }

    fn valid_range(&self) -> (RangeBound, RangeBound) {
        (RangeBound::Unbounded, RangeBound::Unbounded)
    }

    fn is_holiday(&self, _date: Date) -> Result<bool, OutOfRangeError> {
        Ok(false)
    }

    fn is_business_day(&self, _date: Date) -> Result<bool, OutOfRangeError> {
        Ok(true)
    }
}

/// A calendar where Saturday and Sunday are non-business days, with no holidays.
///
/// Useful as a baseline that accounts for weekends but ignores market-specific holidays.
pub struct WeekendsOnly {
    tz: TimeZone,
}

impl WeekendsOnly {
    /// Creates a new `WeekendsOnly` calendar with UTC time zone.
    pub fn new() -> Self {
        Self { tz: TimeZone::UTC }
    }

    /// Creates a new `WeekendsOnly` calendar with the given time zone.
    pub fn with_tz(tz: TimeZone) -> Self {
        Self { tz }
    }

    fn is_weekend(weekday: Weekday) -> bool {
        matches!(weekday, Weekday::Saturday | Weekday::Sunday)
    }
}

impl Default for WeekendsOnly {
    fn default() -> Self {
        Self::new()
    }
}

impl HolidayCalendar for WeekendsOnly {
    fn name(&self) -> &str {
        "WeekendsOnly"
    }

    fn tz(&self) -> &TimeZone {
        &self.tz
    }

    fn valid_range(&self) -> (RangeBound, RangeBound) {
        (RangeBound::Unbounded, RangeBound::Unbounded)
    }

    fn is_holiday(&self, _date: Date) -> Result<bool, OutOfRangeError> {
        Ok(false)
    }

    fn is_business_day(&self, date: Date) -> Result<bool, OutOfRangeError> {
        Ok(!Self::is_weekend(date.weekday()))
    }
}

/// A calendar built from an explicit set of holiday dates and a weekend pattern.
///
/// Unlike rule-based calendars, `BespokeCalendar` takes raw dates rather than
/// computing them from rules. It always has a bounded valid range `[from, to]`;
/// with `from > to` the range is empty and every query returns
/// [`OutOfRangeError`]. Holiday dates outside the range are kept but can
/// never be queried.
///
/// As on every [`HolidayCalendar`], a holiday dated on one of the calendar's
/// weekend days is not reported by `is_holiday` — the weekend already makes
/// it a non-business day.
///
/// Use cases: external data providers, ad-hoc calendars, user-supplied holiday lists.
pub struct BespokeCalendar {
    name: String,
    tz: TimeZone,
    from: Date,
    to: Date,
    holidays: BTreeSet<Date>,
    /// Weekend pattern indexed by [`Weekday::to_monday_zero_offset`] (Mon=0 .. Sun=6).
    weekends: [bool; 7],
}

impl BespokeCalendar {
    /// Creates a new `BespokeCalendar` with Sat/Sun weekends, no holidays, and UTC.
    pub fn new(name: impl Into<String>, from: Date, to: Date) -> Self {
        let mut weekends = [false; 7];
        weekends[Weekday::Saturday.to_monday_zero_offset() as usize] = true;
        weekends[Weekday::Sunday.to_monday_zero_offset() as usize] = true;
        Self {
            name: name.into(),
            tz: TimeZone::UTC,
            from,
            to,
            holidays: BTreeSet::new(),
            weekends,
        }
    }

    /// Sets the time zone.
    pub fn with_tz(mut self, tz: TimeZone) -> Self {
        self.tz = tz;
        self
    }

    /// Sets the weekend days, replacing the default Sat/Sun pattern.
    pub fn with_weekends(mut self, days: &[Weekday]) -> Self {
        self.weekends = [false; 7];
        for &day in days {
            self.weekends[day.to_monday_zero_offset() as usize] = true;
        }
        self
    }

    /// Sets the initial holiday dates, replacing any previously added holidays.
    pub fn with_holidays(mut self, dates: impl IntoIterator<Item = Date>) -> Self {
        self.holidays = dates.into_iter().collect();
        self
    }

    /// Adds a single holiday date.
    pub fn add_holiday(&mut self, date: Date) {
        self.holidays.insert(date);
    }

    /// Removes a single holiday date.
    pub fn remove_holiday(&mut self, date: Date) {
        self.holidays.remove(&date);
    }

    /// Constructs a calendar where the given dates are the only business days.
    ///
    /// All other days in `[from, to]` are treated as non-business days. The
    /// weekend pattern is cleared (set to no weekends) because the date list
    /// is authoritative — any day not in `business_days` is a holiday.
    ///
    /// Dates in `business_days` that fall outside `[from, to]` are silently
    /// ignored.
    pub fn from_business_days(
        name: impl Into<String>,
        from: Date,
        to: Date,
        business_days: &[Date],
    ) -> Self {
        let bday_set: BTreeSet<Date> = business_days
            .iter()
            .copied()
            .filter(|&d| d >= from && d <= to)
            .collect();
        let holidays: BTreeSet<Date> = crate::days_inclusive(from, to)
            .filter(|d| !bday_set.contains(d))
            .collect();
        Self {
            name: name.into(),
            tz: TimeZone::UTC,
            from,
            to,
            holidays,
            weekends: [false; 7], // no weekend pattern — date list is authoritative
        }
    }

    fn check(&self, date: Date) -> Result<(), OutOfRangeError> {
        check_range(date, &self.name, self.valid_range())
    }

    /// Returns `true` if `date` falls on one of this calendar's weekend days.
    fn is_weekend(&self, date: Date) -> bool {
        self.weekends[date.weekday().to_monday_zero_offset() as usize]
    }
}

impl HolidayCalendar for BespokeCalendar {
    fn name(&self) -> &str {
        &self.name
    }

    fn tz(&self) -> &TimeZone {
        &self.tz
    }

    fn valid_range(&self) -> (RangeBound, RangeBound) {
        (RangeBound::Date(self.from), RangeBound::Date(self.to))
    }

    fn is_holiday(&self, date: Date) -> Result<bool, OutOfRangeError> {
        self.check(date)?;
        Ok(!self.is_weekend(date) && self.holidays.contains(&date))
    }

    fn is_business_day(&self, date: Date) -> Result<bool, OutOfRangeError> {
        self.check(date)?;
        Ok(!self.is_weekend(date) && !self.holidays.contains(&date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BusinessDayConvention;
    use jiff::civil::date;

    // --- NoHolidays tests ---

    #[test]
    fn no_holidays_every_day_is_business_day() {
        let cal = NoHolidays::new();
        // Weekday
        assert!(cal.is_business_day(date(2025, 1, 6)).unwrap());
        // Saturday
        assert!(cal.is_business_day(date(2025, 1, 4)).unwrap());
        // Sunday
        assert!(cal.is_business_day(date(2025, 1, 5)).unwrap());
        // Christmas
        assert!(cal.is_business_day(date(2025, 12, 25)).unwrap());
    }

    #[test]
    fn no_holidays_is_never_holiday() {
        let cal = NoHolidays::new();
        assert!(!cal.is_holiday(date(2025, 12, 25)).unwrap());
    }

    #[test]
    fn no_holidays_navigation() {
        let cal = NoHolidays::new();
        assert_eq!(
            cal.next_business_day(date(2025, 1, 3)).unwrap(),
            date(2025, 1, 4)
        );
        assert_eq!(
            cal.prev_business_day(date(2025, 1, 3)).unwrap(),
            date(2025, 1, 2)
        );
    }

    #[test]
    fn no_holidays_business_days_between() {
        let cal = NoHolidays::new();
        assert_eq!(
            cal.business_days_between(date(2025, 1, 1), date(2025, 1, 11))
                .unwrap(),
            10
        );
    }

    #[test]
    fn no_holidays_end_of_month() {
        let cal = NoHolidays::new();
        assert!(cal.is_end_of_month(date(2025, 1, 31)).unwrap());
        assert!(!cal.is_end_of_month(date(2025, 1, 30)).unwrap());
        assert_eq!(
            cal.end_of_month(date(2025, 2, 3)).unwrap(),
            date(2025, 2, 28)
        );
    }

    #[test]
    fn no_holidays_valid_range_unbounded() {
        let cal = NoHolidays::new();
        assert_eq!(
            cal.valid_range(),
            (RangeBound::Unbounded, RangeBound::Unbounded)
        );
    }

    #[test]
    fn no_holidays_with_tz() {
        let tz = TimeZone::get("America/New_York").unwrap();
        let cal = NoHolidays::with_tz(tz.clone());
        assert_eq!(*cal.tz(), tz);
    }

    // --- WeekendsOnly tests ---

    #[test]
    fn weekends_only_weekday_is_business_day() {
        let cal = WeekendsOnly::new();
        // Monday 2025-01-06
        assert!(cal.is_business_day(date(2025, 1, 6)).unwrap());
        // Friday 2025-01-10
        assert!(cal.is_business_day(date(2025, 1, 10)).unwrap());
    }

    #[test]
    fn weekends_only_weekend_is_not_business_day() {
        let cal = WeekendsOnly::new();
        // Saturday 2025-01-04
        assert!(!cal.is_business_day(date(2025, 1, 4)).unwrap());
        // Sunday 2025-01-05
        assert!(!cal.is_business_day(date(2025, 1, 5)).unwrap());
    }

    #[test]
    fn weekends_only_no_holidays() {
        let cal = WeekendsOnly::new();
        assert!(!cal.is_holiday(date(2025, 12, 25)).unwrap());
    }

    #[test]
    fn weekends_only_next_business_day() {
        let cal = WeekendsOnly::new();
        // Friday -> Monday
        assert_eq!(
            cal.next_business_day(date(2025, 1, 3)).unwrap(),
            date(2025, 1, 6)
        );
        // Saturday -> Monday
        assert_eq!(
            cal.next_business_day(date(2025, 1, 4)).unwrap(),
            date(2025, 1, 6)
        );
        // Sunday -> Monday
        assert_eq!(
            cal.next_business_day(date(2025, 1, 5)).unwrap(),
            date(2025, 1, 6)
        );
        // Monday -> Tuesday
        assert_eq!(
            cal.next_business_day(date(2025, 1, 6)).unwrap(),
            date(2025, 1, 7)
        );
    }

    #[test]
    fn weekends_only_prev_business_day() {
        let cal = WeekendsOnly::new();
        // Monday -> Friday
        assert_eq!(
            cal.prev_business_day(date(2025, 1, 6)).unwrap(),
            date(2025, 1, 3)
        );
        // Sunday -> Friday
        assert_eq!(
            cal.prev_business_day(date(2025, 1, 5)).unwrap(),
            date(2025, 1, 3)
        );
        // Saturday -> Friday
        assert_eq!(
            cal.prev_business_day(date(2025, 1, 4)).unwrap(),
            date(2025, 1, 3)
        );
    }

    #[test]
    fn weekends_only_adjust_following() {
        let cal = WeekendsOnly::new();
        // Saturday -> Monday
        assert_eq!(
            cal.adjust(date(2025, 1, 4), BusinessDayConvention::Following)
                .unwrap(),
            date(2025, 1, 6),
        );
        // Weekday unchanged
        assert_eq!(
            cal.adjust(date(2025, 1, 3), BusinessDayConvention::Following)
                .unwrap(),
            date(2025, 1, 3),
        );
    }

    #[test]
    fn weekends_only_adjust_modified_following() {
        let cal = WeekendsOnly::new();
        // Sunday Aug 31 2025 -> Following would be Sep 1 (crosses month)
        // -> falls back to Friday Aug 29
        assert_eq!(
            cal.adjust(date(2025, 8, 31), BusinessDayConvention::ModifiedFollowing)
                .unwrap(),
            date(2025, 8, 29),
        );
    }

    #[test]
    fn weekends_only_adjust_preceding() {
        let cal = WeekendsOnly::new();
        // Sunday -> Friday
        assert_eq!(
            cal.adjust(date(2025, 1, 5), BusinessDayConvention::Preceding)
                .unwrap(),
            date(2025, 1, 3),
        );
    }

    #[test]
    fn weekends_only_adjust_modified_preceding() {
        let cal = WeekendsOnly::new();
        // Saturday Mar 1 2025 -> Preceding would be Feb 28 (crosses month)
        // -> falls forward to Monday Mar 3
        assert_eq!(
            cal.adjust(date(2025, 3, 1), BusinessDayConvention::ModifiedPreceding)
                .unwrap(),
            date(2025, 3, 3),
        );
    }

    #[test]
    fn weekends_only_adjust_nearest() {
        let cal = WeekendsOnly::new();
        // Saturday -> closer to Friday (1 day back) than Monday (2 days forward)
        assert_eq!(
            cal.adjust(date(2025, 1, 4), BusinessDayConvention::Nearest)
                .unwrap(),
            date(2025, 1, 3),
        );
        // Sunday -> closer to Monday (1 day forward) than Friday (2 days back)
        assert_eq!(
            cal.adjust(date(2025, 1, 5), BusinessDayConvention::Nearest)
                .unwrap(),
            date(2025, 1, 6),
        );
    }

    #[test]
    fn weekends_only_business_days_between() {
        let cal = WeekendsOnly::new();
        // Mon Jan 6 to Fri Jan 10 = 4 business days (Mon, Tue, Wed, Thu)
        assert_eq!(
            cal.business_days_between(date(2025, 1, 6), date(2025, 1, 10))
                .unwrap(),
            4
        );
        // Full week Mon-Mon = 5
        assert_eq!(
            cal.business_days_between(date(2025, 1, 6), date(2025, 1, 13))
                .unwrap(),
            5
        );
        // Reverse direction is negative
        assert_eq!(
            cal.business_days_between(date(2025, 1, 10), date(2025, 1, 6))
                .unwrap(),
            -4
        );
    }

    #[test]
    fn weekends_only_end_of_month() {
        let cal = WeekendsOnly::new();
        // Jan 2025: 31st is Friday -> last biz day
        assert_eq!(
            cal.end_of_month(date(2025, 1, 15)).unwrap(),
            date(2025, 1, 31)
        );
        // Aug 2025: 31st is Sunday -> last biz day is Friday 29th
        assert_eq!(
            cal.end_of_month(date(2025, 8, 15)).unwrap(),
            date(2025, 8, 29)
        );
    }

    #[test]
    fn weekends_only_is_end_of_month() {
        let cal = WeekendsOnly::new();
        // Aug 29 2025 (Fri) is last biz day (31st is Sun)
        assert!(cal.is_end_of_month(date(2025, 8, 29)).unwrap());
        assert!(!cal.is_end_of_month(date(2025, 8, 28)).unwrap());
    }

    // --- BespokeCalendar tests ---

    #[test]
    fn bespoke_holiday_is_not_business_day() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31))
            .with_holidays([date(2025, 12, 25)]);
        assert!(cal.is_holiday(date(2025, 12, 25)).unwrap());
        assert!(!cal.is_business_day(date(2025, 12, 25)).unwrap());
        // Regular weekday is fine
        assert!(cal.is_business_day(date(2025, 12, 24)).unwrap());
    }

    #[test]
    fn bespoke_weekends() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31));
        // Saturday 2025-01-04
        assert!(!cal.is_business_day(date(2025, 1, 4)).unwrap());
        // Weekends are not holidays
        assert!(!cal.is_holiday(date(2025, 1, 4)).unwrap());
    }

    #[test]
    fn bespoke_weekend_dated_holiday_is_not_reported() {
        // Same contract as RuleBasedCalendar: the weekend, not the holiday,
        // makes Saturday 2021-12-25 a non-business day.
        let cal = BespokeCalendar::new("Test", date(2021, 1, 1), date(2021, 12, 31))
            .with_holidays([date(2021, 12, 25), date(2021, 12, 27)]);
        assert!(!cal.is_holiday(date(2021, 12, 25)).unwrap());
        assert!(!cal.is_business_day(date(2021, 12, 25)).unwrap());
        assert_eq!(
            cal.holidays_in_year(2021).unwrap(),
            vec![date(2021, 12, 27)]
        );
        // Without implicit weekends every listed date is a holiday.
        let explicit = BespokeCalendar::new("Test", date(2021, 1, 1), date(2021, 12, 31))
            .with_weekends(&[])
            .with_holidays([date(2021, 12, 25)]);
        assert!(explicit.is_holiday(date(2021, 12, 25)).unwrap());
    }

    #[test]
    fn bespoke_empty_range_rejects_every_query() {
        let cal = BespokeCalendar::new("Empty", date(2025, 12, 31), date(2025, 1, 1));
        assert!(cal.is_business_day(date(2025, 6, 2)).is_err());
        assert!(!cal.covers(date(2025, 6, 2), date(2025, 6, 2)));
    }

    #[test]
    fn end_of_month_of_a_closed_month_is_in_an_earlier_month() {
        let february: Vec<_> = (1..=28).map(|d| date(2025, 2, d)).collect();
        let cal = BespokeCalendar::new("Gap", date(2025, 1, 1), date(2025, 12, 31))
            .with_holidays(february);
        assert_eq!(
            cal.end_of_month(date(2025, 2, 10)).unwrap(),
            date(2025, 1, 31)
        );
    }

    #[test]
    fn range_methods_are_safe_at_the_last_representable_date() {
        let cal = WeekendsOnly::new();
        let last = date(9999, 12, 31); // Friday
        assert_eq!(
            cal.business_days_between(date(9999, 12, 27), last).unwrap(),
            4
        );
        assert!(cal.next_business_day(last).is_err());
        assert_eq!(cal.end_of_month(date(9999, 12, 1)).unwrap(), last);
        assert_eq!(cal.holidays_in_year(9999).unwrap(), vec![]);
        let bespoke = BespokeCalendar::from_business_days("b", date(9999, 12, 30), last, &[last]);
        assert!(bespoke.is_business_day(last).unwrap());
    }

    #[test]
    fn bespoke_custom_weekends() {
        // Middle East: Fri/Sat weekends
        let cal = BespokeCalendar::new("ME", date(2025, 1, 1), date(2025, 12, 31))
            .with_weekends(&[Weekday::Friday, Weekday::Saturday]);
        // Friday is a weekend
        assert!(!cal.is_business_day(date(2025, 1, 3)).unwrap());
        // Sunday is a business day
        assert!(cal.is_business_day(date(2025, 1, 5)).unwrap());
    }

    #[test]
    fn bespoke_out_of_range() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31));
        assert!(cal.is_business_day(date(2024, 12, 31)).is_err());
        assert!(cal.is_business_day(date(2026, 1, 1)).is_err());
        // Boundaries are inclusive
        assert!(cal.is_business_day(date(2025, 1, 1)).is_ok());
        assert!(cal.is_business_day(date(2025, 12, 31)).is_ok());
    }

    #[test]
    fn bespoke_add_remove_holiday() {
        let mut cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31));
        // June 16 2025 is a Monday
        assert!(cal.is_business_day(date(2025, 6, 16)).unwrap());
        cal.add_holiday(date(2025, 6, 16));
        assert!(!cal.is_business_day(date(2025, 6, 16)).unwrap());
        cal.remove_holiday(date(2025, 6, 16));
        assert!(cal.is_business_day(date(2025, 6, 16)).unwrap());
    }

    #[test]
    fn bespoke_navigation_skips_holidays() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31))
            .with_holidays([date(2025, 1, 2), date(2025, 1, 3)]);
        // Wed Jan 1 -> next biz day skips Thu 2 and Fri 3 (holidays) + Sat 4, Sun 5 -> Mon 6
        assert_eq!(
            cal.next_business_day(date(2025, 1, 1)).unwrap(),
            date(2025, 1, 6)
        );
    }

    #[test]
    fn bespoke_business_days_between() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 1, 31))
            .with_holidays([date(2025, 1, 1)]);
        // Jan 1 (Wed, holiday) to Jan 3 (Fri) -> count Jan 2 (Thu) = 1
        assert_eq!(
            cal.business_days_between(date(2025, 1, 1), date(2025, 1, 3))
                .unwrap(),
            1
        );
    }

    #[test]
    fn bespoke_holidays_between() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31))
            .with_holidays([date(2025, 1, 1), date(2025, 7, 4), date(2025, 12, 25)]);
        let hols = cal
            .holidays_between(date(2025, 1, 1), date(2025, 8, 1))
            .unwrap();
        assert_eq!(hols, vec![date(2025, 1, 1), date(2025, 7, 4)]);
    }

    #[test]
    fn bespoke_valid_range_is_bounded() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31));
        assert_eq!(
            cal.valid_range(),
            (
                RangeBound::Date(date(2025, 1, 1)),
                RangeBound::Date(date(2025, 12, 31))
            ),
        );
    }

    #[test]
    fn bespoke_out_of_range_error_message() {
        let cal = BespokeCalendar::new("MyCalendar", date(2025, 1, 1), date(2025, 12, 31));
        let err = cal.is_business_day(date(2024, 6, 1)).unwrap_err();
        assert_eq!(
            err.to_string(),
            "date 2024-06-01 is outside valid range of calendar 'MyCalendar' [2025-01-01 .. 2025-12-31]",
        );
    }

    // --- add_business_days ---

    #[test]
    fn add_business_days_steps_over_weekends() {
        let cal = WeekendsOnly::new();
        // Fri 2025-01-03
        assert_eq!(
            cal.add_business_days(date(2025, 1, 3), 1).unwrap(),
            date(2025, 1, 6)
        );
        assert_eq!(
            cal.add_business_days(date(2025, 1, 3), 5).unwrap(),
            date(2025, 1, 10)
        );
        assert_eq!(
            cal.add_business_days(date(2025, 1, 6), -1).unwrap(),
            date(2025, 1, 3)
        );
        assert_eq!(
            cal.add_business_days(date(2025, 1, 6), -6).unwrap(),
            date(2024, 12, 27)
        );
    }

    #[test]
    fn add_business_days_never_counts_the_start_date() {
        let cal = WeekendsOnly::new();
        // From a Saturday: the first step lands on Monday / Friday.
        assert_eq!(
            cal.add_business_days(date(2025, 1, 4), 1).unwrap(),
            date(2025, 1, 6)
        );
        assert_eq!(
            cal.add_business_days(date(2025, 1, 4), -1).unwrap(),
            date(2025, 1, 3)
        );
        // Zero steps: unchanged, business day or not.
        assert_eq!(
            cal.add_business_days(date(2025, 1, 4), 0).unwrap(),
            date(2025, 1, 4)
        );
    }

    #[test]
    fn add_business_days_agrees_with_single_steps_and_counts() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31))
            .with_holidays([date(2025, 4, 18), date(2025, 4, 21)]);
        let start = date(2025, 4, 16);
        assert_eq!(
            cal.add_business_days(start, 1).unwrap(),
            cal.next_business_day(start).unwrap()
        );
        assert_eq!(
            cal.add_business_days(start, -1).unwrap(),
            cal.prev_business_day(start).unwrap()
        );
        // Wed 16 → Thu 17 → (Good Friday, weekend, Easter Monday) → Tue 22 → Wed 23
        let end = cal.add_business_days(start, 3).unwrap();
        assert_eq!(end, date(2025, 4, 23));
        assert_eq!(cal.business_days_between(start, end).unwrap(), 3);
        assert_eq!(cal.add_business_days(end, -3).unwrap(), start);
    }

    #[test]
    fn add_business_days_out_of_range_is_an_error() {
        let cal = BespokeCalendar::new("Test", date(2025, 1, 1), date(2025, 12, 31));
        assert!(cal.add_business_days(date(2025, 12, 30), 5).is_err());
        assert!(cal.add_business_days(date(2025, 1, 3), -5).is_err());
    }
}
