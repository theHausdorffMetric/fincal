//! Rule-based holiday calendar.
//!
//! A [`RuleBasedCalendar`] is composed of [`HolidayRule`]s
//! that compute holiday dates algorithmically. Unlike a
//! [`BespokeCalendar`](crate::calendars::BespokeCalendar), which stores explicit
//! dates, a rule-based calendar can extrapolate holidays to any year.
//!
//! Calendars are constructed via [`RuleBasedCalendarBuilder`], obtained from
//! [`RuleBasedCalendar::builder`]. The builder's [`build()`](RuleBasedCalendarBuilder::build)
//! step validates that the calendar's valid range does not exceed the coverage
//! of its rules.

use std::sync::Arc;

use jiff::civil::{Date, Weekday};
use jiff::tz::TimeZone;

use crate::rules::HolidayRule;
use crate::{CalendarBuildError, HolidayCalendar, OutOfRangeError, RangeBound, check_range};

/// A holiday calendar defined by a set of [`HolidayRule`]s.
///
/// Each rule returns all holiday dates that fall within a given year —
/// including cross-year observance shifts (e.g. New Year's Day on Saturday →
/// observed Friday December 31 of the previous year). The calendar combines
/// all rules to determine which dates are holidays. Rules are shared via
/// [`Arc`] so they can also belong to a [`RuleRegistry`](crate::registry::RuleRegistry).
///
/// Construct via [`RuleBasedCalendar::builder`].
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use fincal::rule_calendar::RuleBasedCalendar;
/// use fincal::rules::FixedDate;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = RuleBasedCalendar::builder("Simple")
///     .with_rules(vec![
///         Arc::new(FixedDate::new("Christmas Day", 12, 25)),
///     ])
///     .build()
///     .unwrap();
/// assert!(cal.is_holiday(date(2025, 12, 25)).unwrap());
/// assert!(cal.is_business_day(date(2025, 12, 24)).unwrap());
/// ```
pub struct RuleBasedCalendar {
    name: String,
    description: Option<String>,
    tz: TimeZone,
    weekends: [bool; 7],
    rules: Vec<Arc<dyn HolidayRule>>,
    from: Option<Date>,
    to: Option<Date>,
}

impl std::fmt::Debug for RuleBasedCalendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("RuleBasedCalendar");
        s.field("name", &self.name);
        if let Some(desc) = &self.description {
            s.field("description", desc);
        }
        s.field("rules", &self.rules.len())
            .field("from", &self.from)
            .field("to", &self.to)
            .finish()
    }
}

impl RuleBasedCalendar {
    /// Returns a new builder for a rule-based calendar with the given name.
    pub fn builder(name: impl Into<String>) -> RuleBasedCalendarBuilder {
        RuleBasedCalendarBuilder::new(name)
    }

    /// Returns the calendar's description, if set.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Checks that `date` falls within this calendar's valid range.
    fn check(&self, date: Date) -> Result<(), OutOfRangeError> {
        check_range(date, &self.name, self.valid_range())
    }

    /// Returns the rules that produce the given date as a holiday.
    ///
    /// This is the primary "explain" mechanism: given a date that is a holiday,
    /// which rule(s) caused it?
    ///
    /// The returned rules are `Arc`-cloned references to the same rule objects
    /// held by this calendar.
    ///
    /// This is a **rule-level** answer: on an implicit weekend day it still
    /// names the rules dated there (e.g. Christmas Day on a Saturday), while
    /// [`is_holiday`](HolidayCalendar::is_holiday) is `false` and
    /// [`holiday_name`](HolidayCalendar::holiday_name) is `None` for that day
    /// — the weekend, not the holiday, makes it a non-business day.
    ///
    /// Returns an empty vec if the date is outside this calendar's valid range.
    pub fn explain_date(&self, date: Date) -> Vec<Arc<dyn HolidayRule>> {
        if self.check(date).is_err() {
            return Vec::new();
        }
        self.rules
            .iter()
            .filter(|r| r.holidays_in(date.year()).contains(&date))
            .cloned()
            .collect()
    }

    /// Returns a reference to the calendar's rules.
    pub fn rules(&self) -> &[Arc<dyn HolidayRule>] {
        &self.rules
    }

    /// Returns `true` if the given weekday is an implicit weekend day in this
    /// calendar (false for every weekday on explicit-weekend calendars built
    /// with `with_weekends(&[])`, e.g. `ICISChinaChemicals`).
    pub fn is_weekend_day(&self, weekday: Weekday) -> bool {
        self.is_weekend(weekday)
    }

    /// Returns `true` if the given weekday is a weekend day in this calendar.
    fn is_weekend(&self, weekday: Weekday) -> bool {
        self.weekends[weekday.to_monday_zero_offset() as usize]
    }

    /// Collects all holiday dates produced by all rules for the given year.
    fn holidays_for_year(&self, year: i16) -> Vec<Date> {
        let mut dates = Vec::new();
        for rule in &self.rules {
            dates.extend(rule.holidays_in(year));
        }
        dates
    }
}

impl HolidayCalendar for RuleBasedCalendar {
    fn name(&self) -> &str {
        &self.name
    }

    fn tz(&self) -> &TimeZone {
        &self.tz
    }

    fn valid_range(&self) -> (RangeBound, RangeBound) {
        let from = self.from.map_or(RangeBound::Unbounded, RangeBound::Date);
        let to = self.to.map_or(RangeBound::Unbounded, RangeBound::Date);
        (from, to)
    }

    fn is_holiday(&self, date: Date) -> Result<bool, OutOfRangeError> {
        self.check(date)?;
        if self.is_weekend(date.weekday()) {
            return Ok(false);
        }
        let holidays = self.holidays_for_year(date.year());
        Ok(holidays.contains(&date))
    }

    fn is_business_day(&self, date: Date) -> Result<bool, OutOfRangeError> {
        self.check(date)?;
        if self.is_weekend(date.weekday()) {
            return Ok(false);
        }
        let holidays = self.holidays_for_year(date.year());
        Ok(!holidays.contains(&date))
    }

    fn holiday_name(&self, date: Date) -> Result<Option<&str>, OutOfRangeError> {
        self.check(date)?;
        if self.is_weekend(date.weekday()) {
            return Ok(None);
        }
        for rule in &self.rules {
            if rule.holidays_in(date.year()).contains(&date) {
                return Ok(Some(rule.name()));
            }
        }
        Ok(None)
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`RuleBasedCalendar`].
///
/// Obtained via [`RuleBasedCalendar::builder`]. The [`build`](Self::build)
/// step validates that the calendar's valid range is within the intersection
/// of all rules' [`valid_range()`](HolidayRule::valid_range). If no explicit
/// range is set, the range is inferred as the tightest envelope of the rules.
pub struct RuleBasedCalendarBuilder {
    name: String,
    description: Option<String>,
    tz: TimeZone,
    weekends: [bool; 7],
    rules: Vec<Arc<dyn HolidayRule>>,
    from: Option<Date>,
    to: Option<Date>,
}

impl RuleBasedCalendarBuilder {
    /// Creates a new builder with default settings (UTC, Sat/Sun weekends, no rules).
    fn new(name: impl Into<String>) -> Self {
        let mut weekends = [false; 7];
        weekends[Weekday::Saturday.to_monday_zero_offset() as usize] = true;
        weekends[Weekday::Sunday.to_monday_zero_offset() as usize] = true;
        Self {
            name: name.into(),
            description: None,
            tz: TimeZone::UTC,
            weekends,
            rules: Vec::new(),
            from: None,
            to: None,
        }
    }

    /// Sets the time zone.
    pub fn with_tz(mut self, tz: TimeZone) -> Self {
        self.tz = tz;
        self
    }

    /// Sets a human-readable description for the calendar.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the weekend days, replacing the default Saturday/Sunday pattern.
    pub fn with_weekends(mut self, days: &[Weekday]) -> Self {
        self.weekends = [false; 7];
        for &day in days {
            self.weekends[day.to_monday_zero_offset() as usize] = true;
        }
        self
    }

    /// Sets the holiday rules.
    pub fn with_rules(mut self, rules: Vec<Arc<dyn HolidayRule>>) -> Self {
        self.rules = rules;
        self
    }

    /// Sets the valid range to January 1 of `from_year` through December 31 of `to_year`.
    ///
    /// If not set, the range is inferred from the intersection of all rules'
    /// [`valid_range()`](HolidayRule::valid_range). An inverted range
    /// (`from_year > to_year`) is rejected by [`build`](Self::build).
    ///
    /// # Panics
    ///
    /// Panics if either year is outside the representable date range
    /// (−9999..=9999).
    pub fn with_valid_range(mut self, from_year: i16, to_year: i16) -> Self {
        self.from = Some(Date::new(from_year, 1, 1).expect("valid from_year"));
        self.to = Some(Date::new(to_year, 12, 31).expect("valid to_year"));
        self
    }

    /// Builds the [`RuleBasedCalendar`], validating the valid range.
    ///
    /// If an explicit range was set via [`with_valid_range`](Self::with_valid_range),
    /// it must fall within the intersection of all rules'
    /// [`valid_range()`](HolidayRule::valid_range). If not, returns
    /// [`CalendarBuildError::RuleCoverage`] naming the first offending rule.
    ///
    /// If no explicit range was set, the range is inferred as the intersection
    /// (tightest envelope) of all rules' ranges.
    ///
    /// Either way the resulting range must contain at least one day:
    /// an inverted explicit range, or rule coverages that do not overlap,
    /// yield [`CalendarBuildError::EmptyRange`] — such a calendar would
    /// reject every query.
    pub fn build(self) -> Result<RuleBasedCalendar, CalendarBuildError> {
        if let (Some(from), Some(to)) = (self.from, self.to)
            && from > to
        {
            return Err(CalendarBuildError::EmptyRange {
                calendar: self.name,
                from,
                to,
            });
        }

        // Compute the intersection of all rules' valid ranges.
        let mut inferred_from = RangeBound::Unbounded;
        let mut inferred_to = RangeBound::Unbounded;
        for rule in &self.rules {
            let (rf, rt) = rule.valid_range();
            inferred_from = range_bound_max(inferred_from, rf);
            inferred_to = range_bound_min(inferred_to, rt);
        }

        let (cal_from, cal_to) = if self.from.is_some() || self.to.is_some() {
            // Explicit range — validate against each rule.
            let cal_from = self.from.map_or(RangeBound::Unbounded, RangeBound::Date);
            let cal_to = self.to.map_or(RangeBound::Unbounded, RangeBound::Date);

            for rule in &self.rules {
                let (rf, rt) = rule.valid_range();
                if !range_bound_gte(cal_from, rf) || !range_bound_lte(cal_to, rt) {
                    return Err(CalendarBuildError::RuleCoverage {
                        calendar: self.name,
                        rule: rule.name().to_string(),
                        calendar_range: (cal_from, cal_to),
                        rule_range: (rf, rt),
                    });
                }
            }
            (self.from, self.to)
        } else {
            // No explicit range — use the inferred intersection.
            let from = match inferred_from {
                RangeBound::Date(d) => Some(d),
                RangeBound::Unbounded => None,
            };
            let to = match inferred_to {
                RangeBound::Date(d) => Some(d),
                RangeBound::Unbounded => None,
            };
            (from, to)
        };

        if let (Some(from), Some(to)) = (cal_from, cal_to)
            && from > to
        {
            return Err(CalendarBuildError::EmptyRange {
                calendar: self.name,
                from,
                to,
            });
        }

        Ok(RuleBasedCalendar {
            name: self.name,
            description: self.description,
            tz: self.tz,
            weekends: self.weekends,
            rules: self.rules,
            from: cal_from,
            to: cal_to,
        })
    }
}

/// Returns the later (tighter) of two lower bounds.
///
/// `Unbounded` is less than any `Date`.
fn range_bound_max(a: RangeBound, b: RangeBound) -> RangeBound {
    match (a, b) {
        (RangeBound::Unbounded, other) | (other, RangeBound::Unbounded) => other,
        (RangeBound::Date(da), RangeBound::Date(db)) => {
            RangeBound::Date(if da >= db { da } else { db })
        }
    }
}

/// Returns the earlier (tighter) of two upper bounds.
///
/// `Unbounded` is greater than any `Date`.
fn range_bound_min(a: RangeBound, b: RangeBound) -> RangeBound {
    match (a, b) {
        (RangeBound::Unbounded, other) | (other, RangeBound::Unbounded) => other,
        (RangeBound::Date(da), RangeBound::Date(db)) => {
            RangeBound::Date(if da <= db { da } else { db })
        }
    }
}

/// Returns `true` if `a >= b` as lower bounds (`Unbounded` < any Date).
fn range_bound_gte(a: RangeBound, b: RangeBound) -> bool {
    match (a, b) {
        (_, RangeBound::Unbounded) => true,
        (RangeBound::Unbounded, RangeBound::Date(_)) => false,
        (RangeBound::Date(da), RangeBound::Date(db)) => da >= db,
    }
}

/// Returns `true` if `a <= b` as upper bounds (`Unbounded` > any Date).
fn range_bound_lte(a: RangeBound, b: RangeBound) -> bool {
    match (a, b) {
        (_, RangeBound::Unbounded) => true,
        (RangeBound::Unbounded, RangeBound::Date(_)) => false,
        (RangeBound::Date(da), RangeBound::Date(db)) => da <= db,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{EasterRelative, FixedDate, ObservancePolicy, fixed_date_with_observed};
    use jiff::civil::date;

    fn simple_calendar() -> RuleBasedCalendar {
        RuleBasedCalendar::builder("Test")
            .with_rules(vec![
                Arc::new(FixedDate::new("Christmas Day", 12, 25)),
                Arc::new(FixedDate::new("New Year's Day", 1, 1)),
                Arc::new(EasterRelative::new("Good Friday", -2)),
            ])
            .build()
            .unwrap()
    }

    #[test]
    fn is_holiday_fixed_date() {
        let cal = simple_calendar();
        assert!(cal.is_holiday(date(2025, 12, 25)).unwrap());
        assert!(cal.is_holiday(date(2025, 1, 1)).unwrap());
    }

    #[test]
    fn is_business_day() {
        let cal = simple_calendar();
        // Weekday, not a holiday
        assert!(cal.is_business_day(date(2025, 12, 24)).unwrap());
        // Holiday
        assert!(!cal.is_business_day(date(2025, 12, 25)).unwrap());
        // Weekend
        assert!(!cal.is_business_day(date(2025, 12, 27)).unwrap()); // Saturday
    }

    #[test]
    fn weekend_is_not_holiday() {
        let cal = simple_calendar();
        // Dec 25, 2021 is Saturday — it's a weekend, not classified as a holiday
        assert!(!cal.is_holiday(date(2021, 12, 25)).unwrap());
        assert!(!cal.is_business_day(date(2021, 12, 25)).unwrap());
    }

    #[test]
    fn explain_date_returns_matching_rules() {
        let cal = simple_calendar();
        let rules = cal.explain_date(date(2025, 12, 25));
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name(), "Christmas Day");
    }

    #[test]
    fn explain_date_no_match() {
        let cal = simple_calendar();
        let rules = cal.explain_date(date(2025, 6, 15));
        assert!(rules.is_empty());
    }

    #[test]
    fn explain_date_easter_relative() {
        let cal = simple_calendar();
        // 2025: Easter = Apr 20, Good Friday = Apr 18
        let rules = cal.explain_date(date(2025, 4, 18));
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name(), "Good Friday");
    }

    #[test]
    fn cross_year_observed_holiday() {
        // 2022: Jan 1 is Saturday -> US observed = Fri Dec 31, 2021
        let mut rules = fixed_date_with_observed(
            "New Year's Day",
            1,
            1,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        );
        rules.push(Arc::new(FixedDate::new("Christmas Day", 12, 25)));
        let cal = RuleBasedCalendar::builder("US-lite")
            .with_rules(rules)
            .build()
            .unwrap();

        // Dec 31, 2021 is the observed New Year's Day for 2022
        assert!(cal.is_holiday(date(2021, 12, 31)).unwrap());
        assert!(!cal.is_business_day(date(2021, 12, 31)).unwrap());

        // Explain it
        let matching = cal.explain_date(date(2021, 12, 31));
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].name(), "New Year's Day (Observed)");
    }

    #[test]
    fn holiday_name_returns_first_matching() {
        let cal = simple_calendar();
        assert_eq!(
            cal.holiday_name(date(2025, 12, 25)).unwrap(),
            Some("Christmas Day")
        );
        assert_eq!(cal.holiday_name(date(2025, 6, 15)).unwrap(), None);
    }

    #[test]
    fn valid_range_is_unbounded() {
        let cal = simple_calendar();
        assert_eq!(
            cal.valid_range(),
            (RangeBound::Unbounded, RangeBound::Unbounded)
        );
    }

    #[test]
    fn custom_weekends() {
        let cal = RuleBasedCalendar::builder("ME")
            .with_weekends(&[Weekday::Friday, Weekday::Saturday])
            .with_rules(vec![Arc::new(FixedDate::new("Test", 1, 1))])
            .build()
            .unwrap();
        // Friday is weekend
        assert!(!cal.is_business_day(date(2025, 1, 3)).unwrap()); // Friday
        // Sunday is business day
        assert!(cal.is_business_day(date(2025, 1, 5)).unwrap()); // Sunday
    }

    #[test]
    fn next_business_day_skips_holiday_and_weekend() {
        let cal = RuleBasedCalendar::builder("Test")
            .with_rules(vec![
                Arc::new(FixedDate::new("Holiday", 1, 2)),
                Arc::new(FixedDate::new("Holiday2", 1, 3)),
            ])
            .build()
            .unwrap();
        // Wed Jan 1 2025 -> next skips Thu 2 (hol), Fri 3 (hol), Sat 4, Sun 5 -> Mon 6
        assert_eq!(
            cal.next_business_day(date(2025, 1, 1)).unwrap(),
            date(2025, 1, 6)
        );
    }

    #[test]
    fn with_tz_sets_timezone() {
        let tz = TimeZone::get("Europe/Berlin").unwrap();
        let cal = RuleBasedCalendar::builder("TARGET")
            .with_tz(tz.clone())
            .build()
            .unwrap();
        assert_eq!(*cal.tz(), tz);
    }

    #[test]
    fn valid_range_with_builder() {
        let cal = RuleBasedCalendar::builder("Test")
            .with_valid_range(2020, 2025)
            .build()
            .unwrap();
        assert_eq!(
            cal.valid_range(),
            (
                RangeBound::Date(date(2020, 1, 1)),
                RangeBound::Date(date(2025, 12, 31)),
            )
        );
    }

    #[test]
    fn valid_range_rejects_out_of_range() {
        let cal = RuleBasedCalendar::builder("Test")
            .with_rules(vec![
                Arc::new(FixedDate::new("Christmas Day", 12, 25)),
                Arc::new(FixedDate::new("New Year's Day", 1, 1)),
                Arc::new(EasterRelative::new("Good Friday", -2)),
            ])
            .with_valid_range(2020, 2025)
            .build()
            .unwrap();
        // Before range
        assert!(cal.is_holiday(date(2019, 12, 25)).is_err());
        assert!(cal.is_business_day(date(2019, 12, 25)).is_err());
        assert!(cal.holiday_name(date(2019, 12, 25)).is_err());
        // After range
        assert!(cal.is_holiday(date(2026, 1, 1)).is_err());
        // At boundaries — should succeed
        assert!(cal.is_holiday(date(2020, 1, 1)).is_ok());
        assert!(cal.is_holiday(date(2025, 12, 25)).is_ok());
    }

    #[test]
    fn explain_date_returns_empty_outside_range() {
        let cal = RuleBasedCalendar::builder("Test")
            .with_rules(vec![
                Arc::new(FixedDate::new("Christmas Day", 12, 25)),
                Arc::new(FixedDate::new("New Year's Day", 1, 1)),
                Arc::new(EasterRelative::new("Good Friday", -2)),
            ])
            .with_valid_range(2020, 2025)
            .build()
            .unwrap();
        let rules = cal.explain_date(date(2019, 12, 25));
        assert!(rules.is_empty());
    }

    #[test]
    fn build_rejects_range_exceeding_rules() {
        // DateList with dates only in 2024-2025 has unbounded valid_range by default,
        // so we create a rule with a finite valid_range by using a bounded FixedDate-like
        // approach. We'll use DateList and override nothing — but we need a rule with
        // finite valid_range. Let's create a struct for this test.
        struct BoundedRule;
        impl HolidayRule for BoundedRule {
            fn name(&self) -> &str {
                "Bounded Test Rule"
            }
            fn holidays_in(&self, year: i16) -> Vec<Date> {
                if (2020..=2025).contains(&year) {
                    vec![Date::new(year, 6, 15).unwrap()]
                } else {
                    vec![]
                }
            }
            fn valid_range(&self) -> (RangeBound, RangeBound) {
                (
                    RangeBound::Date(date(2020, 1, 1)),
                    RangeBound::Date(date(2025, 12, 31)),
                )
            }
        }

        let result = RuleBasedCalendar::builder("Wide")
            .with_rules(vec![Arc::new(BoundedRule)])
            .with_valid_range(2018, 2030)
            .build();

        let Err(CalendarBuildError::RuleCoverage { calendar, rule, .. }) = result else {
            panic!("expected a RuleCoverage error");
        };
        assert_eq!(calendar, "Wide");
        assert_eq!(rule, "Bounded Test Rule");
    }

    #[test]
    fn build_rejects_range_exceeding_date_list_coverage() {
        // The guard that makes coverage declarations real: extending a
        // calendar's range past a recurring DateList's maintained window
        // must fail at build time, naming the offending list.
        let list = crate::rules::DateList::new("Bounded List", [date(2024, 5, 1)])
            .with_valid_range(2022, 2026);
        let result = RuleBasedCalendar::builder("TooWide")
            .with_rules(vec![Arc::new(list)])
            .with_valid_range(2022, 2028)
            .build();
        let Err(CalendarBuildError::RuleCoverage { calendar, rule, .. }) = result else {
            panic!("expected a RuleCoverage error");
        };
        assert_eq!(calendar, "TooWide");
        assert_eq!(rule, "Bounded List");
    }

    #[test]
    fn build_infers_range_from_rules() {
        struct BoundedFrom;
        impl HolidayRule for BoundedFrom {
            fn name(&self) -> &str {
                "Bounded From"
            }
            fn holidays_in(&self, year: i16) -> Vec<Date> {
                vec![Date::new(year, 3, 1).unwrap()]
            }
            fn valid_range(&self) -> (RangeBound, RangeBound) {
                (RangeBound::Date(date(2015, 1, 1)), RangeBound::Unbounded)
            }
        }

        struct BoundedTo;
        impl HolidayRule for BoundedTo {
            fn name(&self) -> &str {
                "Bounded To"
            }
            fn holidays_in(&self, year: i16) -> Vec<Date> {
                vec![Date::new(year, 7, 1).unwrap()]
            }
            fn valid_range(&self) -> (RangeBound, RangeBound) {
                (
                    RangeBound::Date(date(2010, 1, 1)),
                    RangeBound::Date(date(2027, 12, 31)),
                )
            }
        }

        let cal = RuleBasedCalendar::builder("Inferred")
            .with_rules(vec![Arc::new(BoundedFrom), Arc::new(BoundedTo)])
            .build()
            .unwrap();

        // Inferred from = max(2015-01-01, 2010-01-01) = 2015-01-01
        // Inferred to   = min(Unbounded, 2027-12-31) = 2027-12-31
        assert_eq!(
            cal.valid_range(),
            (
                RangeBound::Date(date(2015, 1, 1)),
                RangeBound::Date(date(2027, 12, 31)),
            )
        );
    }

    #[test]
    fn build_rejects_inverted_explicit_range() {
        let err = RuleBasedCalendar::builder("Inverted")
            .with_valid_range(2030, 2020)
            .build()
            .unwrap_err();
        assert_eq!(
            err,
            CalendarBuildError::EmptyRange {
                calendar: "Inverted".to_string(),
                from: date(2030, 1, 1),
                to: date(2020, 12, 31),
            }
        );
        assert_eq!(
            err.to_string(),
            "calendar 'Inverted' has an empty valid range [2030-01-01 .. 2020-12-31]"
        );
    }

    #[test]
    fn build_rejects_disjoint_rule_coverages() {
        // Two date lists maintained for different eras: no year is covered
        // by both, so the inferred range is empty.
        let early =
            crate::rules::DateList::new("Early", [date(2019, 5, 1)]).with_valid_range(2018, 2020);
        let late =
            crate::rules::DateList::new("Late", [date(2024, 5, 1)]).with_valid_range(2023, 2025);
        let err = RuleBasedCalendar::builder("Disjoint")
            .with_rules(vec![Arc::new(early), Arc::new(late)])
            .build()
            .unwrap_err();
        assert!(
            matches!(err, CalendarBuildError::EmptyRange { .. }),
            "{err}"
        );
    }

    #[test]
    fn unbounded_calendar_never_panics_at_the_date_limits() {
        // Observance rules look at the neighbouring editions; at the first
        // and last representable year those do not exist.
        let cal = RuleBasedCalendar::builder("Unbounded")
            .with_rules(fixed_date_with_observed(
                "New Year's Day",
                1,
                1,
                ObservancePolicy::SaturdayToFridaySundayToMonday,
            ))
            .build()
            .unwrap();
        assert!(cal.is_business_day(date(9999, 6, 1)).is_ok());
        assert!(cal.is_business_day(date(-9999, 6, 1)).is_ok());
        // 9999-12-31 is a Friday and a business day; there is no day after it.
        let err = cal.next_business_day(date(9999, 12, 31)).unwrap_err();
        assert_eq!(err.date, date(9999, 12, 31));
        assert!(cal.holidays_in_year(i16::MAX).is_err());
    }

    #[test]
    fn calendar_without_business_days_errors_instead_of_walking_forever() {
        let closed = RuleBasedCalendar::builder("Closed")
            .with_weekends(&[
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
                Weekday::Saturday,
                Weekday::Sunday,
            ])
            .with_valid_range(2025, 2026)
            .build()
            .unwrap();
        assert!(closed.next_business_day(date(2025, 6, 1)).is_err());
        assert!(closed.prev_business_day(date(2025, 6, 1)).is_err());
    }

    #[test]
    fn holidays_in_year_includes_december_31() {
        let cal = RuleBasedCalendar::builder("NYE")
            .with_rules(vec![
                Arc::new(FixedDate::new("New Year's Day", 1, 1)),
                Arc::new(FixedDate::new("New Year's Eve", 12, 31)),
            ])
            .with_valid_range(2025, 2025)
            .build()
            .unwrap();
        // Both are weekdays in 2025 (Wed); the valid range ends on the 31st.
        assert_eq!(
            cal.holidays_in_year(2025).unwrap(),
            vec![date(2025, 1, 1), date(2025, 12, 31)]
        );
        assert!(cal.holidays_in_year(2026).is_err());
    }

    #[test]
    fn explain_date_is_rule_level_on_weekends() {
        let cal = simple_calendar();
        let saturday = date(2021, 12, 25);
        assert!(!cal.is_holiday(saturday).unwrap());
        assert_eq!(cal.holiday_name(saturday).unwrap(), None);
        assert_eq!(cal.explain_date(saturday)[0].name(), "Christmas Day");
    }

    #[test]
    fn build_accepts_range_within_rules() {
        struct BoundedRule;
        impl HolidayRule for BoundedRule {
            fn name(&self) -> &str {
                "Bounded"
            }
            fn holidays_in(&self, year: i16) -> Vec<Date> {
                vec![Date::new(year, 6, 15).unwrap()]
            }
            fn valid_range(&self) -> (RangeBound, RangeBound) {
                (
                    RangeBound::Date(date(2010, 1, 1)),
                    RangeBound::Date(date(2030, 12, 31)),
                )
            }
        }

        let result = RuleBasedCalendar::builder("Narrow")
            .with_rules(vec![Arc::new(BoundedRule)])
            .with_valid_range(2015, 2025)
            .build();
        assert!(result.is_ok());
    }
}
