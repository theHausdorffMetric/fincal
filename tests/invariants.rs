//! Invariant sweep over everything the crate ships.
//!
//! These tests assert properties that must hold for **every** built-in rule
//! and calendar on **every** day, rather than spot dates: the rule contract,
//! the consistency of the calendar predicates, the documented equivalences
//! between calendars, and a sanity bound on closure length that catches
//! data-entry slips in the maintained date lists. A new calendar or rule is
//! covered without touching this file.

use std::collections::BTreeSet;

use fincal::builtin::{all_calendars, all_rules, calendars};
use fincal::rule_calendar::RuleBasedCalendar;
use fincal::{HolidayCalendar, RangeBound, compare_calendars};
use jiff::civil::{Date, date};

/// First and last day to sweep: the valid range, or 2000–2099 where unbounded.
fn sweep_range(cal: &dyn HolidayCalendar) -> (Date, Date) {
    let (from, to) = cal.valid_range();
    (
        from.date().unwrap_or(date(2000, 1, 1)),
        to.date().unwrap_or(date(2099, 12, 31)),
    )
}

/// Every day of the inclusive range `[from, to]`.
fn days(from: Date, to: Date) -> impl Iterator<Item = Date> {
    std::iter::successors(Some(from), move |d| d.tomorrow().ok().filter(|n| *n <= to))
}

#[test]
fn rules_return_only_dates_of_the_queried_year_without_duplicates() {
    for rule in all_rules() {
        for year in 1990i16..=2110 {
            let dates = rule.holidays_in(year);
            let unique: BTreeSet<Date> = dates.iter().copied().collect();
            assert_eq!(
                unique.len(),
                dates.len(),
                "rule '{}' returns a duplicate date for {year}",
                rule.name()
            );
            for d in dates {
                assert_eq!(
                    d.year(),
                    year,
                    "rule '{}' returned {d} for year {year}",
                    rule.name()
                );
            }
        }
    }
}

#[test]
fn bounded_rules_are_silent_outside_their_coverage() {
    for rule in all_rules() {
        let (RangeBound::Date(from), RangeBound::Date(to)) = rule.valid_range() else {
            continue;
        };
        for year in (1990..from.year()).chain(to.year() + 1..=2110) {
            assert!(
                rule.holidays_in(year).is_empty(),
                "rule '{}' covers {}..{} but produces dates in {year}",
                rule.name(),
                from.year(),
                to.year()
            );
        }
    }
}

#[test]
fn calendar_predicates_are_consistent_on_every_day() {
    for cal in all_calendars() {
        let (from, to) = sweep_range(&cal);
        for d in days(from, to) {
            let holiday = cal.is_holiday(d).unwrap();
            let business = cal.is_business_day(d).unwrap();
            let name = cal.holiday_name(d).unwrap();
            let weekend = cal.is_weekend_day(d.weekday());
            assert!(
                !(holiday && business),
                "{}: {d} is both a holiday and a business day",
                cal.name()
            );
            assert_eq!(
                business,
                !weekend && !holiday,
                "{}: {d} business day ≠ not weekend and not holiday",
                cal.name()
            );
            assert_eq!(
                name.is_some(),
                holiday,
                "{}: {d} holiday_name disagrees with is_holiday",
                cal.name()
            );
            assert!(
                !(weekend && holiday),
                "{}: {d} is an implicit weekend day reported as a holiday",
                cal.name()
            );
        }
    }
}

#[test]
fn holidays_between_lists_exactly_the_holidays() {
    for cal in all_calendars() {
        let (from, to) = sweep_range(&cal);
        let expected: Vec<Date> = days(from, to)
            .filter(|d| cal.is_holiday(*d).unwrap())
            .collect();
        // Half-open interval: step past the inclusive end.
        let listed = cal.holidays_between(from, to.tomorrow().unwrap()).unwrap();
        assert_eq!(listed, expected, "{}", cal.name());
    }
}

#[test]
fn no_calendar_is_closed_for_more_than_two_weeks() {
    // Longest real closures: Baltic Christmas (10 days with the adjoining
    // weekend and New Year), ICIS Europe year-end (12). A longer run means a
    // slip in a maintained date list (a wrong month, a span typed twice).
    for cal in all_calendars() {
        let (from, to) = sweep_range(&cal);
        let (mut run, mut longest, mut longest_end) = (0, 0, from);
        for d in days(from, to) {
            if cal.is_business_day(d).unwrap() {
                run = 0;
            } else {
                run += 1;
                if run > longest {
                    (longest, longest_end) = (run, d);
                }
            }
        }
        assert!(
            longest <= 14,
            "{}: {longest} consecutive non-business days ending {longest_end}",
            cal.name()
        );
    }
}

#[test]
fn every_bounded_calendar_spans_whole_years() {
    for cal in all_calendars() {
        let (from, to) = cal.valid_range();
        if let RangeBound::Date(d) = from {
            assert_eq!((d.month(), d.day()), (1, 1), "{} starts {d}", cal.name());
        }
        if let RangeBound::Date(d) = to {
            assert_eq!((d.month(), d.day()), (12, 31), "{} ends {d}", cal.name());
        }
        if let (RangeBound::Date(f), RangeBound::Date(t)) = (from, to) {
            assert!(f <= t, "{} has an empty valid range", cal.name());
            assert!(cal.covers(f, t));
            assert!(!cal.covers(f, t.tomorrow().unwrap()));
        }
    }
}

#[test]
fn calendar_names_are_unique() {
    let cals = all_calendars();
    let names: BTreeSet<String> = cals.iter().map(|c| c.name().to_ascii_lowercase()).collect();
    assert_eq!(
        names.len(),
        cals.len(),
        "calendar names collide ignoring case"
    );
}

/// Asserts that two calendars agree on every day of `[from, to]`.
fn assert_equivalent(a: &RuleBasedCalendar, b: &RuleBasedCalendar, from: Date, to: Date) {
    let cmp = compare_calendars(a, b, from, to).unwrap();
    assert!(cmp.is_identical(), "{} vs {}: {cmp}", a.name(), b.name());
}

#[test]
fn documented_calendar_equivalences_hold() {
    // Each pair is documented as date-equivalent in builtin/calendars.rs and
    // kept as separate constructors so the publishers can diverge; this test
    // is what notices when they do.
    assert_equivalent(
        &calendars::argus_asia_lpg(),
        &calendars::platts_asia_marketscan(),
        date(2010, 1, 1),
        date(2027, 12, 31),
    );
    assert_equivalent(
        &calendars::opis_petrochem_wire(),
        &calendars::platts_us_marketscan(),
        date(2025, 1, 1),
        date(2027, 12, 31),
    );
    assert_equivalent(
        &calendars::argus_europe_lpg(),
        &calendars::platts_european_market_scan(),
        date(2000, 1, 1),
        date(2099, 12, 31),
    );
}
