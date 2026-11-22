//! Rule-based financial holiday calendar framework.
//!
//! `fincal` answers the core questions of market-data scheduling — *is this a
//! business day?*, *what is the next one?*, *which holiday is this?* — and the
//! inverse problem: *given a list of dates, which market calendar do they
//! follow?* It is built in three layers:
//!
//! 1. **Rules** ([`rules`]) — [`HolidayRule`] atoms that compute the date(s)
//!    of a single holiday for any year.
//! 2. **Calendars** ([`calendars`], [`rule_calendar`], [`builtin`]) — compose
//!    rules (or explicit date sets) into complete market calendars
//!    implementing [`HolidayCalendar`].
//! 3. **Analysis** ([`compare`], [`fit`], [`registry`]) — compare calendars,
//!    rank candidates against observed dates, and fit rules to gaps in data.
//!
//! # Cargo features
//!
//! | Feature | Default | Effect |
//! |---------|---------|--------|
//! | `cli` | yes | the `fincal` CLI binary (pulls `clap`) |
//!
//! With `default-features = false` the library depends on `jiff` alone.

pub mod builtin;
pub mod calendars;
pub mod compare;
pub mod easter;
pub mod fit;
pub mod registry;
pub mod rule_calendar;
pub mod rules;

use std::fmt;

use jiff::civil::Date;
use jiff::tz::TimeZone;

/// One end of a calendar's valid date range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeBound {
    /// No limit in this direction.
    Unbounded,
    /// An inclusive date boundary.
    Date(Date),
}

impl RangeBound {
    /// Returns the boundary date, or `None` if this end is unbounded.
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::RangeBound;
    /// use jiff::civil::date;
    ///
    /// assert_eq!(RangeBound::Date(date(2027, 12, 31)).date(), Some(date(2027, 12, 31)));
    /// assert_eq!(RangeBound::Unbounded.date(), None);
    /// ```
    pub fn date(self) -> Option<Date> {
        match self {
            RangeBound::Unbounded => None,
            RangeBound::Date(d) => Some(d),
        }
    }
}

/// Error returned when a date falls outside a calendar's valid range.
///
/// Also returned when a walk (e.g. [`HolidayCalendar::next_business_day`])
/// would have to step past the limits of the representable date range
/// (years −9999..=9999) — `date` is then the last date examined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutOfRangeError {
    pub date: Date,
    pub calendar: String,
    pub from: RangeBound,
    pub to: RangeBound,
}

/// Error returned by [`RuleBasedCalendarBuilder::build`] when the calendar's
/// valid range cannot be honoured.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CalendarBuildError {
    /// The requested valid range exceeds the data coverage of a rule.
    RuleCoverage {
        /// Name of the calendar being built.
        calendar: String,
        /// Name of the first rule whose coverage is too narrow.
        rule: String,
        /// The calendar's requested `(from, to)` range.
        calendar_range: (RangeBound, RangeBound),
        /// The offending rule's `(from, to)` range.
        rule_range: (RangeBound, RangeBound),
    },
    /// The valid range is empty — it starts after it ends. Either the
    /// requested range is inverted, or the coverages of the rules do not
    /// overlap (the range inferred from them has no day in it). Such a
    /// calendar would reject every query.
    EmptyRange {
        /// Name of the calendar being built.
        calendar: String,
        /// First day of the (empty) range.
        from: Date,
        /// Last day of the (empty) range.
        to: Date,
    },
}

impl fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "date {} is outside valid range of calendar '{}'",
            self.date, self.calendar
        )?;
        match (self.from, self.to) {
            (RangeBound::Date(from), RangeBound::Date(to)) => write!(f, " [{from} .. {to}]"),
            (RangeBound::Date(from), RangeBound::Unbounded) => write!(f, " [{from} ..]"),
            (RangeBound::Unbounded, RangeBound::Date(to)) => write!(f, " [.. {to}]"),
            (RangeBound::Unbounded, RangeBound::Unbounded) => Ok(()),
        }
    }
}

impl std::error::Error for OutOfRangeError {}

fn format_range_bound(rb: RangeBound) -> String {
    match rb {
        RangeBound::Unbounded => "..".to_string(),
        RangeBound::Date(d) => d.to_string(),
    }
}

impl fmt::Display for CalendarBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarBuildError::RuleCoverage {
                calendar,
                rule,
                calendar_range,
                rule_range,
            } => write!(
                f,
                "calendar '{calendar}' range [{} .. {}] exceeds rule '{rule}' coverage [{} .. {}]",
                format_range_bound(calendar_range.0),
                format_range_bound(calendar_range.1),
                format_range_bound(rule_range.0),
                format_range_bound(rule_range.1),
            ),
            CalendarBuildError::EmptyRange { calendar, from, to } => write!(
                f,
                "calendar '{calendar}' has an empty valid range [{from} .. {to}]"
            ),
        }
    }
}

impl std::error::Error for CalendarBuildError {}

/// Checks that `date` falls within the given inclusive range, returning
/// an [`OutOfRangeError`] if it does not.
pub fn check_range(
    date: Date,
    calendar: &str,
    range: (RangeBound, RangeBound),
) -> Result<(), OutOfRangeError> {
    let (from, to) = range;
    let below = match from {
        RangeBound::Unbounded => false,
        RangeBound::Date(f) => date < f,
    };
    let above = match to {
        RangeBound::Unbounded => false,
        RangeBound::Date(t) => date > t,
    };
    if below || above {
        return Err(OutOfRangeError {
            date,
            calendar: calendar.to_string(),
            from,
            to,
        });
    }
    Ok(())
}

/// Every day of the inclusive range `[from, to]`, ascending; empty if
/// `from > to`. Never steps past `to`, so it is safe up to the last
/// representable date.
pub(crate) fn days_inclusive(from: Date, to: Date) -> impl Iterator<Item = Date> {
    std::iter::successors((from <= to).then_some(from), move |d| {
        d.tomorrow().ok().filter(|next| *next <= to)
    })
}

/// The error for a walk that would have to step past `date`, the first or
/// last representable date.
fn unrepresentable(
    calendar: &str,
    (from, to): (RangeBound, RangeBound),
    date: Date,
) -> OutOfRangeError {
    OutOfRangeError {
        date,
        calendar: calendar.to_string(),
        from,
        to,
    }
}

/// Business day adjustment conventions (ISDA).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusinessDayConvention {
    /// Keep date as-is.
    NoAdjustment,
    /// Next business day.
    Following,
    /// Next business day, but stay in same month.
    ModifiedFollowing,
    /// Previous business day.
    Preceding,
    /// Previous business day, but stay in same month.
    ModifiedPreceding,
    /// Closest business day.
    Nearest,
}

/// A calendar of holidays and business days for a specific financial market or jurisdiction.
///
/// Methods that take a date return [`OutOfRangeError`] if the date (or any date
/// examined during the operation) falls outside the calendar's [`valid_range`](Self::valid_range).
/// Calendars with no natural bound (e.g. rule-based calendars) use
/// [`RangeBound::Unbounded`]; their methods fail only where a walk would leave
/// the range of representable dates (years −9999..=9999). No method panics.
///
/// # Holidays and weekends
///
/// Every day is exactly one of *business day*, *holiday* or *regular
/// non-business day* (the calendar's implicit weekend):
///
/// - [`is_holiday`](Self::is_holiday) implies not
///   [`is_business_day`](Self::is_business_day).
/// - A holiday dated on an implicit weekend day is **not** reported by
///   `is_holiday` — the day is non-business either way, and the holiday
///   reaches the calendar through its observed substitute, if it has one.
///   Calendars without an implicit weekend (explicit-weekend models,
///   [`BespokeCalendar::from_business_days`]) report every non-business day
///   as a holiday.
///
/// # Interval conventions
///
/// | Methods | Interval |
/// |---------|----------|
/// | [`business_days_between`](Self::business_days_between), [`holidays_between`](Self::holidays_between) | half-open `[start, end)` — counts and lists chain without double-counting |
/// | [`valid_range`](Self::valid_range), [`covers`](Self::covers), [`holidays_in_year`](Self::holidays_in_year), and the [`compare`] / [`fit`] functions | inclusive `[from, to]` |
///
/// To list the holidays of whole years use `holidays_in_year`; to pass an
/// inclusive end to a `*_between` method, pass the day after it.
pub trait HolidayCalendar: Send + Sync {
    /// Human-readable calendar name (e.g. "USSettlement", "TARGET").
    fn name(&self) -> &str;

    /// Time zone associated with this calendar.
    fn tz(&self) -> &TimeZone;

    /// Inclusive date range for which this calendar is well-defined.
    fn valid_range(&self) -> (RangeBound, RangeBound);

    /// Returns `true` if every date of the inclusive range `[from, to]` lies
    /// within this calendar's [`valid_range`](Self::valid_range).
    ///
    /// Use it to decide up front whether a range query can succeed — e.g. to
    /// select the candidates of a ranking, or to report that a calendar's
    /// horizon cuts a requested range — instead of provoking an
    /// [`OutOfRangeError`].
    fn covers(&self, from: Date, to: Date) -> bool {
        let (lo, hi) = self.valid_range();
        lo.date().is_none_or(|d| d <= from) && hi.date().is_none_or(|d| to <= d)
    }

    // --- Required methods (must be implemented) ---

    /// Returns `true` if the date is a holiday in this calendar: a day that
    /// is not a business day *because of a holiday*, as opposed to the
    /// calendar's implicit weekend (see [Holidays and weekends](Self#holidays-and-weekends)).
    fn is_holiday(&self, date: Date) -> Result<bool, OutOfRangeError>;

    /// Returns `true` if the date is a business day (not a weekend and not a holiday).
    fn is_business_day(&self, date: Date) -> Result<bool, OutOfRangeError>;

    // --- Default methods (derived from the above) ---

    /// Returns the next business day strictly after `date`.
    fn next_business_day(&self, date: Date) -> Result<Date, OutOfRangeError> {
        let mut d = date;
        loop {
            d = d
                .tomorrow()
                .map_err(|_| unrepresentable(self.name(), self.valid_range(), d))?;
            if self.is_business_day(d)? {
                return Ok(d);
            }
        }
    }

    /// Returns the previous business day strictly before `date`.
    fn prev_business_day(&self, date: Date) -> Result<Date, OutOfRangeError> {
        let mut d = date;
        loop {
            d = d
                .yesterday()
                .map_err(|_| unrepresentable(self.name(), self.valid_range(), d))?;
            if self.is_business_day(d)? {
                return Ok(d);
            }
        }
    }

    /// Steps `n` business days from `date`: forward for positive `n`,
    /// backward for negative `n`.
    ///
    /// `date` itself is never counted, whether or not it is a business day:
    /// `add_business_days(d, 1)` is [`next_business_day`](Self::next_business_day)
    /// and `add_business_days(d, -1)` is
    /// [`prev_business_day`](Self::prev_business_day). `n = 0` returns `date`
    /// unchanged, without a range check.
    ///
    /// This is the building block of contract rules such as "three business
    /// days before the 25th" or "five business days after expiry".
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::HolidayCalendar;
    /// use fincal::builtin::calendars::nymex;
    /// use jiff::civil::date;
    ///
    /// let cal = nymex();
    /// // Henry Hub (NG) January 2026: third-last business day of December 2025
    /// assert_eq!(cal.add_business_days(date(2026, 1, 1), -3).unwrap(), date(2025, 12, 29));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`OutOfRangeError`] if the walk leaves the calendar's valid
    /// range or the representable date range.
    fn add_business_days(&self, date: Date, n: i32) -> Result<Date, OutOfRangeError> {
        let mut d = date;
        for _ in 0..n.unsigned_abs() {
            d = if n > 0 {
                self.next_business_day(d)?
            } else {
                self.prev_business_day(d)?
            };
        }
        Ok(d)
    }

    /// Adjusts a date according to an ISDA business day convention.
    ///
    /// If the date is already a business day, it is returned unchanged
    /// (except for [`BusinessDayConvention::NoAdjustment`] which always returns
    /// the date as-is).
    ///
    /// Near the end of a bounded calendar the adjustment can fail although
    /// its result would lie in range: `ModifiedFollowing`, `ModifiedPreceding`
    /// and `Nearest` must look at the business day on *both* sides of `date`,
    /// and an [`OutOfRangeError`] on either side is returned rather than
    /// guessed around.
    fn adjust(&self, date: Date, conv: BusinessDayConvention) -> Result<Date, OutOfRangeError> {
        if self.is_business_day(date)? {
            return Ok(date);
        }
        match conv {
            BusinessDayConvention::NoAdjustment => Ok(date),
            BusinessDayConvention::Following => self.next_business_day(date),
            BusinessDayConvention::ModifiedFollowing => {
                let next = self.next_business_day(date)?;
                if next.month() != date.month() {
                    self.prev_business_day(date)
                } else {
                    Ok(next)
                }
            }
            BusinessDayConvention::Preceding => self.prev_business_day(date),
            BusinessDayConvention::ModifiedPreceding => {
                let prev = self.prev_business_day(date)?;
                if prev.month() != date.month() {
                    self.next_business_day(date)
                } else {
                    Ok(prev)
                }
            }
            BusinessDayConvention::Nearest => {
                let next = self.next_business_day(date)?;
                let prev = self.prev_business_day(date)?;
                let to_next = next
                    .since(date)
                    .expect("span of two civil dates")
                    .get_days();
                let from_prev = date
                    .since(prev)
                    .expect("span of two civil dates")
                    .get_days();
                if from_prev < to_next {
                    Ok(prev)
                } else {
                    Ok(next)
                }
            }
        }
    }

    /// Counts business days in the half-open interval `[start, end)`.
    ///
    /// Returns a negative count if `start > end`.
    fn business_days_between(&self, start: Date, end: Date) -> Result<i32, OutOfRangeError> {
        let mut count = 0i32;
        let (from, to, sign) = if end >= start {
            (start, end, 1)
        } else {
            (end, start, -1)
        };
        // `to` is exclusive: stop at the day before it.
        if let Ok(last) = to.yesterday() {
            for d in days_inclusive(from, last) {
                if self.is_business_day(d)? {
                    count += 1;
                }
            }
        }
        Ok(count * sign)
    }

    /// Returns all holiday dates in the half-open interval `[start, end)`.
    fn holidays_between(&self, start: Date, end: Date) -> Result<Vec<Date>, OutOfRangeError> {
        let mut holidays = Vec::new();
        let (from, to) = if end >= start {
            (start, end)
        } else {
            (end, start)
        };
        // `to` is exclusive: stop at the day before it.
        if let Ok(last) = to.yesterday() {
            for d in days_inclusive(from, last) {
                if self.is_holiday(d)? {
                    holidays.push(d);
                }
            }
        }
        Ok(holidays)
    }

    /// Returns all holiday dates of the calendar year `year` (1 January to
    /// 31 December inclusive), ascending.
    ///
    /// The whole-year counterpart of [`holidays_between`](Self::holidays_between)
    /// — no end-of-year arithmetic for the caller, and a 31 December holiday
    /// is included.
    ///
    /// # Errors
    ///
    /// Returns [`OutOfRangeError`] if any day of the year is outside the
    /// calendar's valid range, or if `year` is outside the representable
    /// date range (the error then carries the nearest representable date).
    fn holidays_in_year(&self, year: i16) -> Result<Vec<Date>, OutOfRangeError> {
        let (Ok(first), Ok(last)) = (Date::new(year, 1, 1), Date::new(year, 12, 31)) else {
            let nearest = if year > 0 { Date::MAX } else { Date::MIN };
            return Err(unrepresentable(self.name(), self.valid_range(), nearest));
        };
        let mut holidays = Vec::new();
        for d in days_inclusive(first, last) {
            if self.is_holiday(d)? {
                holidays.push(d);
            }
        }
        Ok(holidays)
    }

    /// Returns `true` if `date` is the last business day of its month.
    fn is_end_of_month(&self, date: Date) -> Result<bool, OutOfRangeError> {
        Ok(date == self.end_of_month(date)?)
    }

    /// Returns the last business day of the month containing `date`.
    ///
    /// This is the month's last calendar day adjusted *preceding* (the
    /// QuantLib convention). If the month has no business day at all — a
    /// data-derived calendar with a month-long gap, say — the result is
    /// therefore the last business day of an **earlier** month; compare the
    /// result's month with `date`'s where that matters.
    fn end_of_month(&self, date: Date) -> Result<Date, OutOfRangeError> {
        let last = date.last_of_month();
        if self.is_business_day(last)? {
            Ok(last)
        } else {
            self.prev_business_day(last)
        }
    }

    /// Name of the holiday on this date, if any.
    fn holiday_name(&self, date: Date) -> Result<Option<&str>, OutOfRangeError> {
        let _ = date;
        Ok(None)
    }
}

// Re-exports for convenience.
pub use calendars::{BespokeCalendar, NoHolidays, WeekendsOnly};
pub use compare::{
    CalendarComparison, CalendarFit, compare_calendars, compare_dates, compare_holidays,
    compare_holidays_with_weekends, explain_dates, explain_holidays,
};
pub use fit::{RuleFit, RuleFitResult, fit_rules};
pub use registry::RuleRegistry;
pub use rule_calendar::{RuleBasedCalendar, RuleBasedCalendarBuilder};
pub use rules::HolidayRule;
