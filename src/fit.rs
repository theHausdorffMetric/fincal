//! Rule fitting — finding which rules best explain observed holiday patterns.
//!
//! Given a set of observed business-day dates, [`fit_rules`] scores each candidate
//! [`HolidayRule`] by how well it explains the gaps (observed holidays) in the data.
//! Perfect-fit rules — those that explain at least one holiday without any
//! false positives — are assembled greedily to form an implied calendar.
//!
//! # Example
//!
//! ```
//! use fincal::builtin;
//! use fincal::fit::fit_rules;
//! use jiff::civil::{date, Weekday};
//!
//! // Simulate observed data: every weekday in Jan 2025 except Jan 1 (New Year's)
//! // and Jan 20 (MLK Day).
//! let dates: Vec<_> = (1..=31)
//!     .map(|d| date(2025, 1, d))
//!     .filter(|d| {
//!         d.weekday() != Weekday::Saturday
//!             && d.weekday() != Weekday::Sunday
//!             && *d != date(2025, 1, 1)
//!             && *d != date(2025, 1, 20)
//!     })
//!     .collect();
//!
//! let rules = builtin::all_rules();
//! let result = fit_rules(
//!     &dates,
//!     date(2025, 1, 1),
//!     date(2025, 1, 31),
//!     &rules,
//!     &[Weekday::Saturday, Weekday::Sunday],
//! );
//!
//! assert_eq!(result.match_rate, 1.0);
//! assert!(result.unexplained.is_empty());
//! ```

use std::collections::BTreeSet;
use std::sync::Arc;

use jiff::civil::{Date, Weekday};

use crate::rules::HolidayRule;

/// Returns `true` if the weekday is in the given weekend list.
fn is_weekend(day: Weekday, weekends: &[Weekday]) -> bool {
    weekends.contains(&day)
}

/// Scoring result for a single [`HolidayRule`] against observed data.
///
/// A rule is scored by resolving its holiday dates across the analysis range
/// and checking them against the observed gaps (weekdays with no data).
///
/// - **Hits**: rule fires on a day that is an observed holiday (gap in data).
/// - **False positives**: rule fires on a day where data actually exists.
///
/// A rule with hits and no false positives is a "perfect fit" — it explains
/// holidays without contradicting the data.
pub struct RuleFit {
    /// The rule being scored.
    pub rule: Arc<dyn HolidayRule>,
    /// Observed holidays that this rule explains (rule fires on a gap day).
    pub hits: Vec<Date>,
    /// Days where the rule fires but data exists (not actually a holiday).
    pub false_positives: Vec<Date>,
}

impl RuleFit {
    /// Returns `true` if this rule has at least one hit and no false positives.
    ///
    /// Perfect-fit rules are used in the greedy assembly step of [`fit_rules`].
    pub fn is_perfect(&self) -> bool {
        !self.hits.is_empty() && self.false_positives.is_empty()
    }
}

/// Result of fitting rules against observed data via [`fit_rules`].
///
/// Contains the scored rules, the set of explained/unexplained holidays, and
/// an overall match rate indicating how well the greedy assembly of perfect-fit
/// rules covers the observed holiday pattern.
pub struct RuleFitResult {
    /// All rules that matched at least one observed holiday, sorted by quality
    /// (perfect fits first, then by hit count descending).
    pub fits: Vec<RuleFit>,
    /// Observed holidays explained by at least one perfect-fit rule.
    pub explained: Vec<Date>,
    /// Observed holidays not explained by any perfect-fit rule.
    pub unexplained: Vec<Date>,
    /// Fraction of observed holidays explained by the greedy assembly of
    /// perfect-fit rules. Returns `1.0` when all holidays are explained or
    /// when no holidays exist in the range.
    pub match_rate: f64,
}

/// Scores candidate rules against observed business-day dates.
///
/// # Algorithm
///
/// 1. **Derive observed holidays**: weekdays in `[from, to]` absent from `dates`.
/// 2. **Score each rule**: resolve for all years overlapping the range, filter
///    to weekdays in range, compute hits (intersection with
///    observed holidays) and false positives (rule fires on a date where data
///    exists).
/// 3. **Sort**: perfect fits first (hits > 0, false positives = 0), then by
///    hit count descending.
/// 4. **Greedy assembly**: collect all dates explained by perfect-fit rules.
/// 5. **Compute coverage**: `match_rate` = explained / total observed holidays.
///
/// # Rule coverage caveat
///
/// Results are silently scoped to each rule's data coverage
/// ([`HolidayRule::valid_range`]): a bounded rule (e.g. a recurring
/// [`DateList`](crate::rules::DateList)) contributes no dates — neither hits
/// nor false positives — outside its declared range. When the analysis window
/// extends beyond a rule's coverage, gaps there surface as **unexplained**
/// even if the rule would explain its in-coverage editions, and a perfect fit
/// is a claim about the covered sub-window only. Consult `valid_range()` to
/// distinguish "no rule explains this" from "the right rule's data does not
/// reach".
///
/// # Arguments
///
/// * `dates` — Observed business days (dates where data was present).
/// * `from`, `to` — Inclusive date range to analyse.
/// * `rules` — Candidate rules to score.
/// * `weekends` — Days of the week treated as non-business days.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use fincal::fit::fit_rules;
/// use fincal::rules::FixedDate;
/// use jiff::civil::{date, Weekday};
///
/// // One-week range: Mon-Fri, with Wednesday missing (a holiday).
/// let dates = vec![
///     date(2025, 1, 6), date(2025, 1, 7), // Mon, Tue
///     date(2025, 1, 9), date(2025, 1, 10), // Thu, Fri
/// ];
/// let rules = vec![
///     Arc::new(FixedDate::new("Jan 8", 1, 8)) as Arc<dyn fincal::HolidayRule>,
/// ];
/// let result = fit_rules(
///     &dates,
///     date(2025, 1, 6),
///     date(2025, 1, 10),
///     &rules,
///     &[Weekday::Saturday, Weekday::Sunday],
/// );
/// assert_eq!(result.match_rate, 1.0);
/// assert_eq!(result.explained, vec![date(2025, 1, 8)]);
/// ```
pub fn fit_rules(
    dates: &[Date],
    from: Date,
    to: Date,
    rules: &[Arc<dyn HolidayRule>],
    weekends: &[Weekday],
) -> RuleFitResult {
    let date_set: BTreeSet<Date> = dates.iter().copied().collect();

    // Derive observed holidays: weekdays in range with no data.
    let observed_holidays: BTreeSet<Date> = crate::days_inclusive(from, to)
        .filter(|d| !is_weekend(d.weekday(), weekends) && !date_set.contains(d))
        .collect();

    // Determine the year range to resolve rules for.
    let year_from = from.year();
    let year_to = to.year();

    // Score each rule.
    let mut fits: Vec<RuleFit> = Vec::new();
    for rule in rules {
        // Resolve rule dates across the year window.
        let mut rule_dates = BTreeSet::new();
        for year in year_from..=year_to {
            for rd in rule.holidays_in(year) {
                // Only consider dates that are weekdays within the range.
                if rd >= from && rd <= to && !is_weekend(rd.weekday(), weekends) {
                    rule_dates.insert(rd);
                }
            }
        }

        let hits: Vec<Date> = rule_dates
            .intersection(&observed_holidays)
            .copied()
            .collect();
        let false_positives: Vec<Date> = rule_dates
            .iter()
            .filter(|rd| date_set.contains(rd))
            .copied()
            .collect();

        if !hits.is_empty() {
            fits.push(RuleFit {
                rule: Arc::clone(rule),
                hits,
                false_positives,
            });
        }
    }

    // Sort: perfect fits first, then by hit count descending.
    fits.sort_by(|a, b| match (a.is_perfect(), b.is_perfect()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.hits.len().cmp(&a.hits.len()),
    });

    // Greedy assembly: collect dates explained by perfect-fit rules.
    let mut explained_set = BTreeSet::new();
    for fit in &fits {
        if fit.is_perfect() {
            explained_set.extend(&fit.hits);
        }
    }

    let explained: Vec<Date> = explained_set.iter().copied().collect();
    let unexplained: Vec<Date> = observed_holidays
        .iter()
        .filter(|d| !explained_set.contains(d))
        .copied()
        .collect();

    let match_rate = if observed_holidays.is_empty() {
        1.0
    } else {
        explained.len() as f64 / observed_holidays.len() as f64
    };

    RuleFitResult {
        fits,
        explained,
        unexplained,
        match_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin;
    use jiff::civil::date;

    fn standard_weekends() -> Vec<Weekday> {
        vec![Weekday::Saturday, Weekday::Sunday]
    }

    /// Helper: generate all weekdays in [from, to] except the given holidays.
    fn weekdays_except(from: Date, to: Date, holidays: &[Date]) -> Vec<Date> {
        let hol_set: BTreeSet<Date> = holidays.iter().copied().collect();
        let mut dates = Vec::new();
        let mut d = from;
        while d <= to {
            if d.weekday() != Weekday::Saturday
                && d.weekday() != Weekday::Sunday
                && !hol_set.contains(&d)
            {
                dates.push(d);
            }
            d = d.tomorrow().expect("date overflow");
        }
        dates
    }

    #[test]
    fn no_observed_holidays() {
        // All weekdays have data — no gaps to explain.
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        let dates = weekdays_except(from, to, &[]);
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert!(result.fits.is_empty());
        assert!(result.explained.is_empty());
        assert!(result.unexplained.is_empty());
        assert_eq!(result.match_rate, 1.0);
    }

    #[test]
    fn single_rule_perfect_fit() {
        // Jan 2025: only New Year's Day (Jan 1, Wed) is missing.
        let from = date(2025, 1, 1);
        let to = date(2025, 1, 31);
        let dates = weekdays_except(from, to, &[date(2025, 1, 1)]);
        let rules = vec![builtin::holidays::new_years_day()];
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(result.fits.len(), 1);
        assert!(result.fits[0].is_perfect());
        assert_eq!(result.fits[0].hits, vec![date(2025, 1, 1)]);
        assert!(result.fits[0].false_positives.is_empty());
        assert_eq!(result.explained, vec![date(2025, 1, 1)]);
        assert!(result.unexplained.is_empty());
        assert_eq!(result.match_rate, 1.0);
    }

    #[test]
    fn multiple_rules_all_perfect() {
        // Jan 2025: New Year's Day (Jan 1) + MLK Day (Jan 20) missing.
        let from = date(2025, 1, 1);
        let to = date(2025, 1, 31);
        let dates = weekdays_except(from, to, &[date(2025, 1, 1), date(2025, 1, 20)]);
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(result.match_rate, 1.0);
        assert!(result.unexplained.is_empty());
        assert_eq!(result.explained.len(), 2);
        assert!(result.explained.contains(&date(2025, 1, 1)));
        assert!(result.explained.contains(&date(2025, 1, 20)));
    }

    #[test]
    fn unexplained_holidays() {
        // An ad-hoc holiday on Jan 15 that no rule covers.
        let from = date(2025, 1, 1);
        let to = date(2025, 1, 31);
        let dates = weekdays_except(from, to, &[date(2025, 1, 1), date(2025, 1, 15)]);
        let rules = vec![builtin::holidays::new_years_day()];
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(result.explained, vec![date(2025, 1, 1)]);
        assert_eq!(result.unexplained, vec![date(2025, 1, 15)]);
        assert_eq!(result.match_rate, 0.5);
    }

    #[test]
    fn false_positive_prevents_perfect_fit() {
        // Apr 2025: Good Friday (Apr 18) is missing, but Easter Monday (Apr 21)
        // has data. The Easter Monday rule would fire on Apr 21 (a weekday) where
        // data exists — that's a false positive.
        let from = date(2025, 4, 1);
        let to = date(2025, 4, 30);
        let dates = weekdays_except(from, to, &[date(2025, 4, 18)]);
        let rules = vec![
            builtin::holidays::good_friday(),
            builtin::holidays::easter_monday(),
        ];
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        // Good Friday is a perfect fit (hits Apr 18, no false positives).
        let gf = result
            .fits
            .iter()
            .find(|f| f.rule.name() == "Good Friday")
            .unwrap();
        assert!(gf.is_perfect());
        assert_eq!(gf.hits, vec![date(2025, 4, 18)]);

        // Easter Monday fires on Apr 21 where data exists — it has a false positive
        // but also no hits (Apr 21 is not an observed holiday), so it won't appear
        // in fits at all (we only include rules with hits > 0).
        let em = result
            .fits
            .iter()
            .find(|f| f.rule.name() == "Easter Monday");
        assert!(em.is_none());

        assert_eq!(result.match_rate, 1.0);
    }

    #[test]
    fn rule_with_hits_and_false_positives() {
        // Contrived scenario: a rule that fires on both a holiday and a data day.
        // Feb 2025: data missing on Feb 17 (Presidents' Day).
        // Columbus Day rule (2nd Monday of October) won't fire in Feb, so use
        // a broader range where the rule fires on a holiday AND on a non-holiday.
        //
        // Actually, let's use a custom FixedDate rule: fires Jan 6 every year.
        // If Jan 6 is a holiday in one week but data exists on Jan 6 in a
        // different scenario... Better: use a multi-year range where the rule
        // fires in both a gap and a data day (not possible for same year with
        // FixedDate). Instead, use DateList.
        use crate::rules::DateList;

        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // Data on Mon, Tue, Thu, Fri. Wed (Jan 8) is a holiday.
        let dates = vec![
            date(2025, 1, 6),
            date(2025, 1, 7),
            date(2025, 1, 9),
            date(2025, 1, 10),
        ];
        // A rule that fires on Jan 7 (data day) and Jan 8 (holiday).
        let rule: Arc<dyn HolidayRule> = Arc::new(DateList::new(
            "Overlapping",
            [date(2025, 1, 7), date(2025, 1, 8)],
        ));
        let result = fit_rules(&dates, from, to, &[rule], &standard_weekends());

        assert_eq!(result.fits.len(), 1);
        assert!(!result.fits[0].is_perfect());
        assert_eq!(result.fits[0].hits, vec![date(2025, 1, 8)]);
        assert_eq!(result.fits[0].false_positives, vec![date(2025, 1, 7)]);
        // Not a perfect fit, so it doesn't contribute to explained.
        assert!(result.explained.is_empty());
        assert_eq!(result.unexplained, vec![date(2025, 1, 8)]);
        assert_eq!(result.match_rate, 0.0);
    }

    #[test]
    fn perfect_fits_sorted_before_imperfect() {
        use crate::rules::{DateList, FixedDate};

        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        // Data on Mon, Tue, Thu, Fri. Wed (Jan 8) is a holiday.
        let dates = vec![
            date(2025, 1, 6),
            date(2025, 1, 7),
            date(2025, 1, 9),
            date(2025, 1, 10),
        ];

        // Perfect-fit rule: fires only on Jan 8.
        let perfect: Arc<dyn HolidayRule> = Arc::new(FixedDate::new("Jan 8 Holiday", 1, 8));
        // Imperfect rule: fires on Jan 7 and Jan 8.
        let imperfect: Arc<dyn HolidayRule> = Arc::new(DateList::new(
            "Overlapping",
            [date(2025, 1, 7), date(2025, 1, 8)],
        ));

        // Pass imperfect first to verify sorting.
        let result = fit_rules(
            &dates,
            from,
            to,
            &[imperfect, perfect],
            &standard_weekends(),
        );

        assert_eq!(result.fits.len(), 2);
        assert_eq!(result.fits[0].rule.name(), "Jan 8 Holiday");
        assert!(result.fits[0].is_perfect());
        assert_eq!(result.fits[1].rule.name(), "Overlapping");
        assert!(!result.fits[1].is_perfect());
    }

    #[test]
    fn cross_year_observance() {
        // 2022: Jan 1 is Saturday -> US observed Fri Dec 31, 2021.
        // Test that the observed rule fires within a Dec 2021 range.
        let from = date(2021, 12, 27); // Monday
        let to = date(2021, 12, 31); // Friday
        // Data on Mon-Thu. Fri Dec 31 is the observed New Year's holiday.
        let dates = vec![
            date(2021, 12, 27),
            date(2021, 12, 28),
            date(2021, 12, 29),
            date(2021, 12, 30),
        ];
        let rules = vec![builtin::holidays::us_new_years_day_observed()];
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(result.fits.len(), 1);
        assert!(result.fits[0].is_perfect());
        assert_eq!(result.fits[0].hits, vec![date(2021, 12, 31)]);
        assert_eq!(result.match_rate, 1.0);
    }

    #[test]
    fn year_bounded_rule_ignored_before_from_year() {
        // Juneteenth (from 2021) should not fire for 2020.
        let from = date(2020, 6, 1);
        let to = date(2020, 6, 30);
        // Jun 19 2020 is a Friday. Remove it as a holiday.
        let dates = weekdays_except(from, to, &[date(2020, 6, 19)]);
        let rules = vec![builtin::holidays::juneteenth()];
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        // Juneteenth has from_year=2021, so it shouldn't fire for 2020.
        assert!(result.fits.is_empty());
        assert_eq!(result.unexplained, vec![date(2020, 6, 19)]);
        assert_eq!(result.match_rate, 0.0);
    }

    #[test]
    fn full_year_us_settlement() {
        // Verify that all built-in rules can explain a full year of US holidays.
        let from = date(2025, 1, 1);
        let to = date(2025, 12, 31);
        let cal = builtin::calendars::us_settlement();
        let holidays = crate::HolidayCalendar::holidays_between(&cal, from, to).unwrap();
        let dates = weekdays_except(from, to, &holidays);
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(
            result.match_rate, 1.0,
            "all US holidays should be explained; unexplained: {:?}",
            result.unexplained
        );
        assert!(result.unexplained.is_empty());
    }

    #[test]
    fn full_year_target() {
        let from = date(2025, 1, 1);
        let to = date(2025, 12, 31);
        let cal = builtin::calendars::target();
        let holidays = crate::HolidayCalendar::holidays_between(&cal, from, to).unwrap();
        let dates = weekdays_except(from, to, &holidays);
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(
            result.match_rate, 1.0,
            "all TARGET holidays should be explained; unexplained: {:?}",
            result.unexplained
        );
    }

    #[test]
    fn full_year_uk_settlement() {
        let from = date(2025, 1, 1);
        let to = date(2025, 12, 31);
        let cal = builtin::calendars::uk_settlement();
        let holidays = crate::HolidayCalendar::holidays_between(&cal, from, to).unwrap();
        let dates = weekdays_except(from, to, &holidays);
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(
            result.match_rate, 1.0,
            "all UK holidays should be explained; unexplained: {:?}",
            result.unexplained
        );
    }

    #[test]
    fn multi_year_uk_settlement_2020_2025() {
        // This is the scenario that previously failed: the Early May Bank Holiday
        // was moved in 2020 (VE Day) and the Spring Bank Holiday in 2022 (Jubilee),
        // which caused false positives that poisoned the entire rule, leaving
        // holidays in all other years unexplained.
        let from = date(2020, 1, 1);
        let to = date(2025, 12, 31);
        let cal = builtin::calendars::uk_settlement();
        let holidays = crate::HolidayCalendar::holidays_between(&cal, from, to).unwrap();
        let dates = weekdays_except(from, to, &holidays);
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        assert_eq!(
            result.match_rate, 1.0,
            "all UK holidays 2020-2025 should be explained; unexplained: {:?}",
            result.unexplained
        );
        assert!(result.unexplained.is_empty());
    }

    #[test]
    fn empty_dates_all_weekdays_are_holidays() {
        let from = date(2025, 1, 6); // Monday
        let to = date(2025, 1, 10); // Friday
        let dates: Vec<Date> = vec![];
        let rules = builtin::all_rules();
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        // 5 weekdays are all observed holidays; most won't be explained.
        assert_eq!(result.explained.len() + result.unexplained.len(), 5);
    }

    #[test]
    fn observed_rule_no_false_positive_when_base_fires() {
        // 2025: Jan 1 is Wednesday (weekday). The base FixedDate rule fires.
        // The ObservedFixedDate rule returns empty (no shift needed).
        // So the observed rule should NOT appear as a false positive.
        let from = date(2025, 1, 1);
        let to = date(2025, 1, 31);
        let dates = weekdays_except(from, to, &[date(2025, 1, 1), date(2025, 1, 20)]);
        let rules = vec![
            builtin::holidays::new_years_day(),
            builtin::holidays::us_new_years_day_observed(),
            builtin::holidays::mlk_day(),
        ];
        let result = fit_rules(&dates, from, to, &rules, &standard_weekends());

        // Only new_years_day and mlk_day should appear (observed returns empty for 2025).
        let names: Vec<&str> = result.fits.iter().map(|f| f.rule.name()).collect();
        assert!(names.contains(&"New Year's Day"));
        assert!(names.contains(&"Martin Luther King Jr. Day"));
        // The observed rule produces no dates in 2025, so it has no hits.
        assert!(!names.contains(&"New Year's Day (US Observed)"));
        assert_eq!(result.match_rate, 1.0);
    }
}
