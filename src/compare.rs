//! Calendar comparison — comparing two holiday calendars over a date range.
//!
//! The core primitive is [`compare_calendars`], which iterates over every day in
//! a range and classifies each into one of four buckets: business day in both,
//! non-business day in both, or business day in only one of the two calendars.
//!
//! Two convenience wrappers convert a slice of dates into a [`BespokeCalendar`]
//! and delegate to `compare_calendars`:
//! - [`compare_dates`] — input dates are **business days**
//!   (via [`BespokeCalendar::from_business_days`]).
//! - [`compare_holidays`] — input dates are **holidays**
//!   (via [`BespokeCalendar::with_holidays`]).
//!
//! [`explain_dates`] and [`explain_holidays`] rank multiple candidate calendars
//! by how well they explain the input.

use std::fmt;

use jiff::civil::{Date, Weekday};

use crate::calendars::BespokeCalendar;
use crate::{HolidayCalendar, OutOfRangeError};

/// Result of comparing two holiday calendars over a date range.
///
/// Each day in the comparison range is classified into exactly one of four
/// buckets based on whether calendar A and calendar B consider it a business day.
#[derive(Debug, Clone)]
pub struct CalendarComparison {
    /// Days where calendar A is a business day but calendar B is not.
    pub a_only_bdays: Vec<Date>,
    /// Days where calendar B is a business day but calendar A is not.
    pub b_only_bdays: Vec<Date>,
    /// Days where both calendars agree it is a business day.
    pub agreed_bdays: Vec<Date>,
    /// Count of days where both calendars agree it is a non-business day.
    ///
    /// Only the count is stored because the individual dates are rarely useful.
    pub agreed_non_bdays: usize,
}

impl CalendarComparison {
    /// Returns `true` if the two calendars agree on every day in the range.
    pub fn is_identical(&self) -> bool {
        self.a_only_bdays.is_empty() && self.b_only_bdays.is_empty()
    }

    /// Fraction of days that both calendars classify as business days, out of
    /// all days that *either* calendar considers a business day.
    ///
    /// Returns `1.0` when both calendars agree on every business day (including
    /// when neither calendar has any business days in the range).
    pub fn match_rate(&self) -> f64 {
        let total = self.agreed_bdays.len() + self.a_only_bdays.len() + self.b_only_bdays.len();
        if total == 0 {
            return 1.0;
        }
        self.agreed_bdays.len() as f64 / total as f64
    }
}

impl fmt::Display for CalendarComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "agreed: {} bdays, {} non-bdays | a-only: {} | b-only: {}",
            self.agreed_bdays.len(),
            self.agreed_non_bdays,
            self.a_only_bdays.len(),
            self.b_only_bdays.len(),
        )?;
        if !self.a_only_bdays.is_empty() {
            write!(f, "\n  a-only examples:")?;
            for d in self.a_only_bdays.iter().take(5) {
                write!(f, " {d}")?;
            }
            if self.a_only_bdays.len() > 5 {
                write!(f, " ...")?;
            }
        }
        if !self.b_only_bdays.is_empty() {
            write!(f, "\n  b-only examples:")?;
            for d in self.b_only_bdays.iter().take(5) {
                write!(f, " {d}")?;
            }
            if self.b_only_bdays.len() > 5 {
                write!(f, " ...")?;
            }
        }
        Ok(())
    }
}

/// Compares two holiday calendars day-by-day over the inclusive range `[from, to]`.
///
/// Each day is classified by whether calendar `a` and calendar `b` consider it a
/// business day. Returns [`OutOfRangeError`] if any date in the range falls
/// outside either calendar's valid range.
///
/// # Errors
///
/// Returns [`OutOfRangeError`] if any date in `[from, to]` is outside the valid
/// range of either calendar.
pub fn compare_calendars(
    a: &dyn HolidayCalendar,
    b: &dyn HolidayCalendar,
    from: Date,
    to: Date,
) -> Result<CalendarComparison, OutOfRangeError> {
    let mut a_only_bdays = Vec::new();
    let mut b_only_bdays = Vec::new();
    let mut agreed_bdays = Vec::new();
    let mut agreed_non_bdays: usize = 0;

    for d in crate::days_inclusive(from, to) {
        let a_bday = a.is_business_day(d)?;
        let b_bday = b.is_business_day(d)?;
        match (a_bday, b_bday) {
            (true, true) => agreed_bdays.push(d),
            (true, false) => a_only_bdays.push(d),
            (false, true) => b_only_bdays.push(d),
            (false, false) => agreed_non_bdays += 1,
        }
    }

    Ok(CalendarComparison {
        a_only_bdays,
        b_only_bdays,
        agreed_bdays,
        agreed_non_bdays,
    })
}

/// Compares a list of observed business-day dates against a holiday calendar.
///
/// Converts `dates` into a [`BespokeCalendar`] using
/// [`BespokeCalendar::from_business_days`], then calls [`compare_calendars`].
///
/// In the result:
/// - `a_only_bdays`: dates present in the input but *not* business days per the
///   calendar (e.g. data on weekends/holidays — "extra" dates).
/// - `b_only_bdays`: business days per the calendar that are *absent* from the
///   input (e.g. missing data — "gaps").
///
/// # Errors
///
/// Returns [`OutOfRangeError`] if any date in `[from, to]` is outside the valid
/// range of `calendar`.
pub fn compare_dates(
    dates: &[Date],
    calendar: &dyn HolidayCalendar,
    from: Date,
    to: Date,
) -> Result<CalendarComparison, OutOfRangeError> {
    let bespoke = BespokeCalendar::from_business_days("observed", from, to, dates);
    compare_calendars(&bespoke, calendar, from, to)
}

/// Compares a list of observed holiday dates against a holiday calendar.
///
/// Converts `holidays` into a [`BespokeCalendar`] using
/// [`BespokeCalendar::with_holidays`], then calls [`compare_calendars`].
///
/// In the result (bespoke = calendar A, reference = calendar B):
/// - `a_only_bdays`: days that are business days in the bespoke calendar (i.e.
///   *not* in the input holiday list) but non-business days in the reference
///   calendar — these are holidays **missing** from the input.
/// - `b_only_bdays`: days that are business days in the reference calendar but
///   non-business days in the bespoke calendar (i.e. *in* the input holiday
///   list) — these are **extra** holidays in the input that the reference
///   calendar does not recognise.
///
/// # Errors
///
/// Returns [`OutOfRangeError`] if any date in `[from, to]` is outside the valid
/// range of `calendar`.
pub fn compare_holidays(
    holidays: &[Date],
    calendar: &dyn HolidayCalendar,
    from: Date,
    to: Date,
) -> Result<CalendarComparison, OutOfRangeError> {
    compare_holidays_with_weekends(
        holidays,
        &[Weekday::Saturday, Weekday::Sunday],
        calendar,
        from,
        to,
    )
}

/// [`compare_holidays`] with an explicit weekend convention for the subject.
///
/// The plain [`compare_holidays`] assumes the subject's weekends are the
/// default Saturday/Sunday — appropriate for lists that only enumerate
/// weekday holidays. For **explicit-weekend** holiday lists (every
/// non-working weekend day is itself a listed holiday, e.g. a vendor holiday list
/// mirroring [`crate::rules::WeekendsExcept`]-based calendars like
/// `ICISChinaChemicals`), pass `subject_weekends = &[]`: the subject then
/// has no implicit weekends, so a weekend day is a business day unless it
/// appears in `holidays` — and *missing* weekend entries surface as
/// disagreements instead of being silently absorbed by the weekend
/// convention.
///
/// # Errors
///
/// Returns [`OutOfRangeError`] if any date in `[from, to]` is outside the
/// valid range of `calendar`.
pub fn compare_holidays_with_weekends(
    holidays: &[Date],
    subject_weekends: &[Weekday],
    calendar: &dyn HolidayCalendar,
    from: Date,
    to: Date,
) -> Result<CalendarComparison, OutOfRangeError> {
    let bespoke = BespokeCalendar::new("observed", from, to)
        .with_weekends(subject_weekends)
        .with_holidays(holidays.iter().copied());
    compare_calendars(&bespoke, calendar, from, to)
}

/// Result of fitting a list of dates against a candidate calendar.
///
/// Pairs a calendar reference with its [`CalendarComparison`], allowing
/// [`explain_dates`] results to be inspected or sorted by fit quality.
pub struct CalendarFit<'a> {
    /// The candidate calendar that was compared.
    pub calendar: &'a dyn HolidayCalendar,
    /// The detailed comparison result.
    pub comparison: CalendarComparison,
}

/// Ranks candidate calendars by how well they explain a list of observed dates.
///
/// For each candidate, calls [`compare_dates`] over the range `[from, to]` and
/// returns the results sorted by [`CalendarComparison::match_rate`] (best fit
/// first, then fewest discrepancies as tiebreaker).
///
/// Candidates whose valid range does not [cover](HolidayCalendar::covers)
/// `[from, to]` are **left out of the ranking** — a match rate over a
/// different window would not be comparable, and one narrow calendar must not
/// void the ranking of all the others. The result can therefore be shorter
/// than `candidates` (or empty); filter with `covers` to report what was
/// skipped.
///
/// # Errors
///
/// Returns [`OutOfRangeError`] only if a candidate that claims to cover the
/// range rejects a date inside it (a faulty [`HolidayCalendar`]
/// implementation); the calendars of this crate never do.
///
/// # Examples
///
/// ```
/// use fincal::compare::explain_dates;
/// use fincal::calendars::WeekendsOnly;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let wo = WeekendsOnly::new();
/// let dates = vec![date(2025, 1, 6), date(2025, 1, 7), date(2025, 1, 8),
///                  date(2025, 1, 9), date(2025, 1, 10)];
/// let fits = explain_dates(&dates, date(2025, 1, 6), date(2025, 1, 10),
///                          &[&wo as &dyn HolidayCalendar]).unwrap();
/// assert_eq!(fits[0].comparison.match_rate(), 1.0);
/// ```
pub fn explain_dates<'a>(
    dates: &[Date],
    from: Date,
    to: Date,
    candidates: &[&'a dyn HolidayCalendar],
) -> Result<Vec<CalendarFit<'a>>, OutOfRangeError> {
    let mut fits: Vec<CalendarFit<'a>> = Vec::with_capacity(candidates.len());
    for &calendar in candidates {
        if !calendar.covers(from, to) {
            continue;
        }
        let comparison = compare_dates(dates, calendar, from, to)?;
        fits.push(CalendarFit {
            calendar,
            comparison,
        });
    }
    sort_fits(&mut fits);
    Ok(fits)
}

/// Sorts fits by match rate descending, tie-broken by fewer discrepancies.
fn sort_fits(fits: &mut [CalendarFit<'_>]) {
    fits.sort_by(|a, b| {
        let rate_cmp = b
            .comparison
            .match_rate()
            .partial_cmp(&a.comparison.match_rate())
            .unwrap_or(std::cmp::Ordering::Equal);
        if rate_cmp != std::cmp::Ordering::Equal {
            return rate_cmp;
        }
        let a_disc = a.comparison.a_only_bdays.len() + a.comparison.b_only_bdays.len();
        let b_disc = b.comparison.a_only_bdays.len() + b.comparison.b_only_bdays.len();
        a_disc.cmp(&b_disc)
    });
}

/// Ranks candidate calendars by how well they explain a list of observed holidays.
///
/// For each candidate, calls [`compare_holidays`] over the range `[from, to]`
/// and returns the results sorted by [`CalendarComparison::match_rate`] (best fit
/// first, then fewest discrepancies as tiebreaker).
///
/// As with [`explain_dates`], candidates whose valid range does not
/// [cover](HolidayCalendar::covers) `[from, to]` are left out of the ranking.
///
/// # Errors
///
/// Returns [`OutOfRangeError`] only if a candidate that claims to cover the
/// range rejects a date inside it (see [`explain_dates`]).
pub fn explain_holidays<'a>(
    holidays: &[Date],
    from: Date,
    to: Date,
    candidates: &[&'a dyn HolidayCalendar],
) -> Result<Vec<CalendarFit<'a>>, OutOfRangeError> {
    let mut fits: Vec<CalendarFit<'a>> = Vec::with_capacity(candidates.len());
    for &calendar in candidates {
        if !calendar.covers(from, to) {
            continue;
        }
        let comparison = compare_holidays(holidays, calendar, from, to)?;
        fits.push(CalendarFit {
            calendar,
            comparison,
        });
    }
    sort_fits(&mut fits);
    Ok(fits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendars::{NoHolidays, WeekendsOnly};
    use jiff::civil::date;

    // --- compare_calendars tests ---

    #[test]
    fn compare_holidays_with_weekends_flags_missing_weekend_entries() {
        use crate::rules::WeekendsExcept;
        use std::sync::Arc;
        // Explicit-weekend reference: weekends are named holidays except one
        // working Saturday.
        let reference = crate::rule_calendar::RuleBasedCalendar::builder("ExplicitWeekend")
            .with_weekends(&[])
            .with_valid_range(2025, 2025)
            .with_rules(vec![
                Arc::new(WeekendsExcept::new("Weekend", [date(2025, 2, 8)]))
                    as Arc<dyn crate::rules::HolidayRule>,
            ])
            .build()
            .unwrap();
        let from = date(2025, 2, 1);
        let to = date(2025, 2, 14);
        // Subject list mirrors the reference but is MISSING the Feb 1 Weekend
        // entry and wrongly contains the Feb 8 working Saturday.
        let subject = [date(2025, 2, 2), date(2025, 2, 8), date(2025, 2, 9)];
        // Plain compare_holidays (implicit Sat/Sun) hides the missing Feb 1:
        let plain = compare_holidays(&subject, &reference, from, to).unwrap();
        assert!(!plain.b_only_bdays.contains(&date(2025, 2, 1)));
        // Explicit-weekend comparison surfaces both defects:
        let strict = compare_holidays_with_weekends(&subject, &[], &reference, from, to).unwrap();
        // Feb 1: subject says business (missing entry), reference says Weekend.
        assert!(strict.a_only_bdays.contains(&date(2025, 2, 1)));
        // Feb 8: subject says holiday (wrong extra), reference says working Saturday.
        assert!(strict.b_only_bdays.contains(&date(2025, 2, 8)));
    }

    #[test]
    fn no_holidays_vs_weekends_only_disagrees_on_weekends() {
        let a = NoHolidays::new();
        let b = WeekendsOnly::new();
        // Mon Jan 6 to Sun Jan 12, 2025 (7 days: Mon-Sun)
        let cmp = compare_calendars(&a, &b, date(2025, 1, 6), date(2025, 1, 12)).unwrap();
        // NoHolidays says every day is bday; WeekendsOnly says Mon-Fri
        // a_only: Sat 11, Sun 12
        assert_eq!(cmp.a_only_bdays, vec![date(2025, 1, 11), date(2025, 1, 12)]);
        assert!(cmp.b_only_bdays.is_empty());
        // agreed bdays: Mon-Fri = 5
        assert_eq!(cmp.agreed_bdays.len(), 5);
        assert_eq!(cmp.agreed_non_bdays, 0);
        assert!(!cmp.is_identical());
    }

    #[test]
    fn weekends_only_vs_itself_is_identical() {
        let a = WeekendsOnly::new();
        let b = WeekendsOnly::new();
        let cmp = compare_calendars(&a, &b, date(2025, 1, 6), date(2025, 1, 19)).unwrap();
        assert!(cmp.is_identical());
        assert_eq!(cmp.match_rate(), 1.0);
        // 2 weeks: 10 bdays, 4 non-bdays
        assert_eq!(cmp.agreed_bdays.len(), 10);
        assert_eq!(cmp.agreed_non_bdays, 4);
    }

    #[test]
    fn two_bespoke_overlapping_holidays() {
        let from = date(2025, 1, 1);
        let to = date(2025, 1, 10);
        // Cal A: holidays on Jan 1 (Wed) and Jan 2 (Thu)
        let a =
            BespokeCalendar::new("A", from, to).with_holidays([date(2025, 1, 1), date(2025, 1, 2)]);
        // Cal B: holidays on Jan 2 (Thu) and Jan 3 (Fri)
        let b =
            BespokeCalendar::new("B", from, to).with_holidays([date(2025, 1, 2), date(2025, 1, 3)]);
        let cmp = compare_calendars(&a, &b, from, to).unwrap();
        // Jan 1: A=non-bday(hol), B=bday -> b_only
        assert!(cmp.b_only_bdays.contains(&date(2025, 1, 1)));
        // Jan 2: both holiday -> agreed non-bday
        assert!(!cmp.a_only_bdays.contains(&date(2025, 1, 2)));
        assert!(!cmp.b_only_bdays.contains(&date(2025, 1, 2)));
        // Jan 3: A=bday, B=non-bday(hol) -> a_only
        assert!(cmp.a_only_bdays.contains(&date(2025, 1, 3)));
        assert!(!cmp.is_identical());
    }

    #[test]
    fn two_bespoke_disjoint_holidays() {
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        let a = BespokeCalendar::new("A", from, to).with_holidays([date(2025, 1, 6)]); // Mon
        let b = BespokeCalendar::new("B", from, to).with_holidays([date(2025, 1, 10)]); // Fri
        let cmp = compare_calendars(&a, &b, from, to).unwrap();
        // Mon: A=non-bday, B=bday -> b_only
        assert_eq!(cmp.b_only_bdays, vec![date(2025, 1, 6)]);
        // Fri: A=bday, B=non-bday -> a_only
        assert_eq!(cmp.a_only_bdays, vec![date(2025, 1, 10)]);
        // Tue-Thu: agreed bdays
        assert_eq!(cmp.agreed_bdays.len(), 3);
    }

    #[test]
    fn match_rate_no_business_days() {
        // Both calendars have no bdays in range (all holidays on weekdays)
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 6); // Just Monday
        let a = BespokeCalendar::new("A", from, to).with_holidays([date(2025, 1, 6)]);
        let b = BespokeCalendar::new("B", from, to).with_holidays([date(2025, 1, 6)]);
        let cmp = compare_calendars(&a, &b, from, to).unwrap();
        assert_eq!(cmp.match_rate(), 1.0);
        assert!(cmp.is_identical());
    }

    // --- compare_dates tests ---

    #[test]
    fn compare_dates_perfect_match() {
        let cal = WeekendsOnly::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // All weekday dates present
        let dates = vec![
            date(2025, 1, 6),
            date(2025, 1, 7),
            date(2025, 1, 8),
            date(2025, 1, 9),
            date(2025, 1, 10),
        ];
        let cmp = compare_dates(&dates, &cal, from, to).unwrap();
        assert!(cmp.is_identical());
        assert_eq!(cmp.match_rate(), 1.0);
    }

    #[test]
    fn compare_dates_extra_dates_on_weekends() {
        let cal = WeekendsOnly::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 12); // Sunday
        // All 7 days present — weekends are "extra"
        let dates: Vec<Date> = (6..=12).map(|d| date(2025, 1, d)).collect();
        let cmp = compare_dates(&dates, &cal, from, to).unwrap();
        // a_only = dates that are bdays in observed but not in cal (Sat, Sun)
        assert_eq!(cmp.a_only_bdays, vec![date(2025, 1, 11), date(2025, 1, 12)]);
        assert!(cmp.b_only_bdays.is_empty());
    }

    #[test]
    fn compare_dates_missing_dates() {
        let cal = WeekendsOnly::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // Missing Wed and Thu
        let dates = vec![date(2025, 1, 6), date(2025, 1, 7), date(2025, 1, 10)];
        let cmp = compare_dates(&dates, &cal, from, to).unwrap();
        assert!(cmp.a_only_bdays.is_empty());
        // b_only = calendar says bday but not in dates (Wed, Thu)
        assert_eq!(cmp.b_only_bdays, vec![date(2025, 1, 8), date(2025, 1, 9)]);
    }

    #[test]
    fn compare_dates_empty_input() {
        let cal = WeekendsOnly::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        let dates: Vec<Date> = vec![];
        let cmp = compare_dates(&dates, &cal, from, to).unwrap();
        // All 5 weekdays are "missing"
        assert_eq!(cmp.b_only_bdays.len(), 5);
        assert!(cmp.a_only_bdays.is_empty());
        assert_eq!(cmp.match_rate(), 0.0);
    }

    #[test]
    fn compare_dates_out_of_range() {
        let cal = BespokeCalendar::new("bounded", date(2025, 3, 1), date(2025, 3, 31));
        let dates = vec![date(2025, 1, 6)];
        // Range extends outside the bounded calendar
        let result = compare_dates(&dates, &cal, date(2025, 1, 1), date(2025, 1, 10));
        assert!(result.is_err());
    }

    // --- from_business_days round-trip ---

    #[test]
    fn from_business_days_round_trip() {
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 12); // Sunday
        let bdays = vec![
            date(2025, 1, 6),
            date(2025, 1, 7),
            date(2025, 1, 8),
            date(2025, 1, 9),
            date(2025, 1, 10),
        ];
        let cal = BespokeCalendar::from_business_days("test", from, to, &bdays);
        // Weekdays are business days
        for &d in &bdays {
            assert!(cal.is_business_day(d).unwrap(), "{d} should be a bday");
        }
        // Weekend days are not
        assert!(!cal.is_business_day(date(2025, 1, 11)).unwrap());
        assert!(!cal.is_business_day(date(2025, 1, 12)).unwrap());
    }

    #[test]
    fn from_business_days_all_days() {
        let from = date(2025, 1, 6);
        let to = date(2025, 1, 8);
        let bdays = vec![date(2025, 1, 6), date(2025, 1, 7), date(2025, 1, 8)];
        let cal = BespokeCalendar::from_business_days("all", from, to, &bdays);
        for &d in &bdays {
            assert!(cal.is_business_day(d).unwrap());
        }
    }

    #[test]
    fn from_business_days_no_days() {
        let from = date(2025, 1, 6);
        let to = date(2025, 1, 8);
        let cal = BespokeCalendar::from_business_days("empty", from, to, &[]);
        assert!(!cal.is_business_day(date(2025, 1, 6)).unwrap());
        assert!(!cal.is_business_day(date(2025, 1, 7)).unwrap());
        assert!(!cal.is_business_day(date(2025, 1, 8)).unwrap());
    }

    // --- Display ---

    #[test]
    fn display_identical() {
        let a = WeekendsOnly::new();
        let b = WeekendsOnly::new();
        let cmp = compare_calendars(&a, &b, date(2025, 1, 6), date(2025, 1, 10)).unwrap();
        let s = cmp.to_string();
        assert!(s.contains("a-only: 0"));
        assert!(s.contains("b-only: 0"));
        // No example lines when identical
        assert!(!s.contains("examples"));
    }

    #[test]
    fn display_with_differences() {
        let a = NoHolidays::new();
        let b = WeekendsOnly::new();
        let cmp = compare_calendars(&a, &b, date(2025, 1, 6), date(2025, 1, 12)).unwrap();
        let s = cmp.to_string();
        assert!(s.contains("a-only: 2"));
        assert!(s.contains("a-only examples:"));
    }

    #[test]
    fn display_truncates_at_five_examples() {
        // Create a bespoke calendar with many holidays to generate >5 differences
        let from = date(2025, 1, 1);
        let to = date(2025, 1, 31);
        let a = BespokeCalendar::new("A", from, to)
            .with_holidays([6, 7, 8, 9, 10, 13, 14, 15].map(|d| date(2025, 1, d)));
        let b = BespokeCalendar::new("B", from, to);
        let cmp = compare_calendars(&a, &b, from, to).unwrap();
        let s = cmp.to_string();
        // b_only should have >5 entries and show "..."
        assert!(cmp.b_only_bdays.len() > 5);
        assert!(s.contains("..."));
    }

    // --- explain_dates tests ---

    #[test]
    fn explain_dates_perfect_match() {
        let wo = WeekendsOnly::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        let dates = vec![
            date(2025, 1, 6),
            date(2025, 1, 7),
            date(2025, 1, 8),
            date(2025, 1, 9),
            date(2025, 1, 10),
        ];
        let fits = explain_dates(&dates, from, to, &[&wo as &dyn HolidayCalendar]).unwrap();
        assert_eq!(fits.len(), 1);
        assert_eq!(fits[0].comparison.match_rate(), 1.0);
        assert_eq!(fits[0].calendar.name(), "WeekendsOnly");
    }

    #[test]
    fn explain_dates_ranks_by_match_rate() {
        let wo = WeekendsOnly::new();
        let nh = NoHolidays::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // All weekday dates — WeekendsOnly is a perfect match,
        // NoHolidays also agrees (every day is a bday).
        let dates = vec![
            date(2025, 1, 6),
            date(2025, 1, 7),
            date(2025, 1, 8),
            date(2025, 1, 9),
            date(2025, 1, 10),
        ];
        let fits = explain_dates(
            &dates,
            from,
            to,
            &[&nh as &dyn HolidayCalendar, &wo as &dyn HolidayCalendar],
        )
        .unwrap();
        // Both should have match_rate 1.0 since there are no weekends in range
        assert_eq!(fits[0].comparison.match_rate(), 1.0);
        assert_eq!(fits[1].comparison.match_rate(), 1.0);
    }

    #[test]
    fn explain_dates_empty_candidates() {
        let dates = vec![date(2025, 1, 6)];
        let fits = explain_dates(&dates, date(2025, 1, 6), date(2025, 1, 6), &[]).unwrap();
        assert!(fits.is_empty());
    }

    #[test]
    fn explain_dates_with_gaps_ranks_correctly() {
        let wo = WeekendsOnly::new();
        let nh = NoHolidays::new();
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 12); // Sunday
        // Include weekend dates — NoHolidays matches better (it has no weekends)
        let dates: Vec<Date> = (6..=12).map(|d| date(2025, 1, d)).collect();
        let fits = explain_dates(
            &dates,
            from,
            to,
            &[&wo as &dyn HolidayCalendar, &nh as &dyn HolidayCalendar],
        )
        .unwrap();
        // NoHolidays should rank first (all 7 days are bdays in both)
        assert_eq!(fits[0].calendar.name(), "NoHolidays");
        assert_eq!(fits[0].comparison.match_rate(), 1.0);
        // WeekendsOnly has 2 extra dates (a_only)
        assert!(fits[1].comparison.match_rate() < 1.0);
    }

    // --- compare_holidays tests ---

    #[test]
    fn compare_holidays_perfect_match() {
        // BespokeCalendar with a known holiday on Wed Jan 1
        let cal = BespokeCalendar::new("ref", date(2025, 1, 1), date(2025, 1, 10))
            .with_holidays([date(2025, 1, 1)]);
        let holidays = vec![date(2025, 1, 1)];
        let cmp = compare_holidays(&holidays, &cal, date(2025, 1, 1), date(2025, 1, 10)).unwrap();
        assert!(cmp.is_identical());
        assert_eq!(cmp.match_rate(), 1.0);
    }

    #[test]
    fn compare_holidays_extra_in_input() {
        // Reference has no holidays; input claims Jan 6 (Mon) is a holiday
        let cal = WeekendsOnly::new();
        let holidays = vec![date(2025, 1, 6)]; // Monday — not a holiday per WeekendsOnly
        let cmp = compare_holidays(&holidays, &cal, date(2025, 1, 6), date(2025, 1, 10)).unwrap();
        // b_only_bdays = extra holidays in input (bday in reference but not in bespoke)
        assert_eq!(cmp.b_only_bdays, vec![date(2025, 1, 6)]);
        assert!(cmp.a_only_bdays.is_empty());
    }

    #[test]
    fn compare_holidays_missing_from_input() {
        // Reference has holiday on Wed Jan 1; input is empty
        let cal = BespokeCalendar::new("ref", date(2025, 1, 1), date(2025, 1, 5))
            .with_holidays([date(2025, 1, 1)]);
        let holidays: Vec<Date> = vec![];
        let cmp = compare_holidays(&holidays, &cal, date(2025, 1, 1), date(2025, 1, 5)).unwrap();
        // a_only_bdays = missing holidays (bday in bespoke but not in reference)
        assert_eq!(cmp.a_only_bdays, vec![date(2025, 1, 1)]);
        assert!(cmp.b_only_bdays.is_empty());
    }

    #[test]
    fn compare_holidays_mixed_extra_and_missing() {
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // Reference: holiday on Mon Jan 6
        let cal = BespokeCalendar::new("ref", from, to).with_holidays([date(2025, 1, 6)]);
        // Input: holiday on Fri Jan 10 (but not Mon Jan 6)
        let holidays = vec![date(2025, 1, 10)];
        let cmp = compare_holidays(&holidays, &cal, from, to).unwrap();
        // Missing: Jan 6 (reference holiday not in input)
        assert_eq!(cmp.a_only_bdays, vec![date(2025, 1, 6)]);
        // Extra: Jan 10 (input holiday not in reference)
        assert_eq!(cmp.b_only_bdays, vec![date(2025, 1, 10)]);
    }

    // --- explain_holidays tests ---

    #[test]
    fn explain_holidays_ranks_best_first() {
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // Calendar A: holiday on Mon Jan 6
        let cal_a = BespokeCalendar::new("A", from, to).with_holidays([date(2025, 1, 6)]);
        // Calendar B: no holidays
        let cal_b = BespokeCalendar::new("B", from, to);
        // Input holidays: Jan 6
        let holidays = vec![date(2025, 1, 6)];
        let fits = explain_holidays(
            &holidays,
            from,
            to,
            &[
                &cal_b as &dyn HolidayCalendar,
                &cal_a as &dyn HolidayCalendar,
            ],
        )
        .unwrap();
        // Calendar A should rank first (perfect match)
        assert_eq!(fits[0].calendar.name(), "A");
        assert_eq!(fits[0].comparison.match_rate(), 1.0);
    }

    #[test]
    fn explain_holidays_empty_candidates() {
        let holidays = vec![date(2025, 1, 6)];
        let fits = explain_holidays(&holidays, date(2025, 1, 6), date(2025, 1, 6), &[]).unwrap();
        assert!(fits.is_empty());
    }

    // --- candidates that do not cover the range ---

    #[test]
    fn explain_skips_candidates_not_covering_the_range() {
        // One narrow candidate must not void the ranking of the others.
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        let wide = WeekendsOnly::new();
        let narrow = BespokeCalendar::new("narrow", date(2025, 1, 8), date(2025, 12, 31));
        let candidates = [
            &narrow as &dyn HolidayCalendar,
            &wide as &dyn HolidayCalendar,
        ];
        let dates: Vec<Date> = (6..=10).map(|d| date(2025, 1, d)).collect();

        let fits = explain_dates(&dates, from, to, &candidates).unwrap();
        assert_eq!(fits.len(), 1);
        assert_eq!(fits[0].calendar.name(), "WeekendsOnly");

        let fits = explain_holidays(&[], from, to, &candidates).unwrap();
        assert_eq!(fits.len(), 1);
        assert_eq!(fits[0].calendar.name(), "WeekendsOnly");
    }

    #[test]
    fn explain_dates_ranks_builtins_over_a_range_some_do_not_cover() {
        // The built-in valid ranges intersect only in 2025–2026; a ranking
        // over an earlier window must still return the calendars covering it.
        let cals = crate::builtin::all_calendars();
        let candidates: Vec<&dyn HolidayCalendar> =
            cals.iter().map(|c| c as &dyn HolidayCalendar).collect();
        let from = date(2015, 1, 1);
        let to = date(2015, 12, 31);
        let target = crate::builtin::calendars::target();
        let mut dates = Vec::new();
        let mut d = from;
        while d <= to {
            if target.is_business_day(d).unwrap() {
                dates.push(d);
            }
            d = d.tomorrow().unwrap();
        }
        let fits = explain_dates(&dates, from, to, &candidates).unwrap();
        assert_eq!(fits[0].calendar.name(), "TARGET");
        assert!(fits[0].comparison.is_identical());
        // e.g. OPISPetroChemWire (2025–2027) is not ranked for 2015.
        assert!(fits.len() < candidates.len());
        assert!(fits.iter().all(|f| f.calendar.covers(from, to)));
    }

    #[test]
    fn covers_respects_both_bounds() {
        let cal = BespokeCalendar::new("b", date(2025, 1, 1), date(2025, 12, 31));
        assert!(cal.covers(date(2025, 1, 1), date(2025, 12, 31)));
        assert!(!cal.covers(date(2024, 12, 31), date(2025, 6, 30)));
        assert!(!cal.covers(date(2025, 6, 30), date(2026, 1, 1)));
        assert!(WeekendsOnly::new().covers(date(1900, 1, 1), date(2200, 1, 1)));
    }
}
