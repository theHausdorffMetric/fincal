//! Holiday rules — the building blocks of rule-based calendars.
//!
//! A [`HolidayRule`] produces zero or more holiday dates for a given year. Multiple
//! rules are composed into a [`RuleBasedCalendar`](crate::rule_calendar::RuleBasedCalendar)
//! to define a complete market calendar.
//!
//! # Standard rule types
//!
//! | Type | Example |
//! |------|---------|
//! | [`FixedDate`] | Christmas Day (Dec 25) |
//! | [`NthWeekday`] | Thanksgiving (4th Thursday of November) |
//! | [`EasterRelative`] | Good Friday (Easter − 2) |
//! | [`DateList`] | Ad-hoc one-off holidays |
//! | [`FixedDateRange`] | Baltic Exchange Christmas closure (Dec 25–31) |
//! | [`ReducedWeekdaySchedule`] | BLPG1 weekly/Tue+Thu publication (2026 Hormuz crisis) |
//! | [`ObservedFixedDate`] | New Year's Day observed (US federal policy) |
//! | [`UkChristmasObserved`] | UK Christmas substitute day |
//! | [`UkBoxingDayObserved`] | UK Boxing Day substitute day |

use std::collections::BTreeSet;
use std::sync::Arc;

use jiff::civil::{Date, Weekday};

use crate::RangeBound;
use crate::easter::easter_sunday;

/// A rule that produces holiday dates for a given year.
///
/// Rules are the fundamental building blocks of rule-based calendars. Each rule
/// encapsulates the logic for a single holiday (e.g. "Christmas Day" or
/// "Thanksgiving") and can resolve the concrete date(s) for any year.
///
/// Rules are shared via [`Arc`] so they can belong to both a calendar and a
/// registry simultaneously.
pub trait HolidayRule: Send + Sync {
    /// Human-readable name of this holiday (e.g. "Christmas Day").
    fn name(&self) -> &str;

    /// Returns the holiday date(s) that fall in the given year.
    ///
    /// Returns an empty vec if the rule produces no dates for the year —
    /// e.g. an observed-shift rule when the base date falls on a weekday, a
    /// year-bounded rule outside its enactment period, or a data-driven rule
    /// queried outside its coverage. Rules are infallible; coverage is
    /// declared via [`valid_range`](Self::valid_range) and enforced when a
    /// calendar is built.
    ///
    /// # Contract
    ///
    /// - Every returned date lies in `year`. A rule whose date can shift
    ///   across the year boundary (an observance, a day offset) evaluates the
    ///   neighbouring editions itself and returns only what lands in `year`;
    ///   calendars ask for one year and rely on this.
    /// - The method is total: it must not panic for any `year`. Years that
    ///   [`Date`] cannot represent (outside −9999..=9999) yield no dates.
    fn holidays_in(&self, year: i16) -> Vec<Date>;

    /// Inclusive date range this rule's data reliably covers.
    ///
    /// Algorithmic rules ([`FixedDate`], [`NthWeekday`], etc.) are unbounded —
    /// they can compute holidays for any year. Recurring data-driven rules
    /// ([`DateList`] with [`with_valid_range`](DateList::with_valid_range))
    /// declare finite coverage: outside it `holidays_in` simply returns
    /// nothing, and
    /// [`build`](crate::rule_calendar::RuleBasedCalendarBuilder::build)
    /// rejects a calendar whose valid range exceeds any rule's coverage.
    fn valid_range(&self) -> (RangeBound, RangeBound) {
        (RangeBound::Unbounded, RangeBound::Unbounded)
    }
}

/// The editions that can place a date in `year` when a rule's date may shift
/// across the year boundary by less than a year: `year − 1`, `year`,
/// `year + 1`, without overflowing `i16`.
fn neighbouring_years(year: i16) -> impl Iterator<Item = i16> {
    [year.checked_sub(1), Some(year), year.checked_add(1)]
        .into_iter()
        .flatten()
}

/// `date` shifted by `days`, or `None` at the limits of the representable
/// date range.
fn shifted(date: Date, days: i64) -> Option<Date> {
    date.checked_add(jiff::Span::new().days(days)).ok()
}

/// Returns `true` if `year` is within the optional `[from_year, to_year]` bounds.
fn year_in_bounds(year: i16, from_year: Option<i16>, to_year: Option<i16>) -> bool {
    if let Some(from) = from_year
        && year < from
    {
        return false;
    }
    if let Some(to) = to_year
        && year > to
    {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// FixedDate
// ---------------------------------------------------------------------------

/// A holiday that falls on the same month and day every year.
///
/// Examples: New Year's Day (Jan 1), Christmas Day (Dec 25), Independence Day (Jul 4).
///
/// Optionally bounded by `from_year` and `to_year` to represent holidays that
/// were introduced or abolished in a specific year.
pub struct FixedDate {
    name: String,
    month: i8,
    day: i8,
    from_year: Option<i16>,
    to_year: Option<i16>,
}

impl FixedDate {
    /// Creates a new fixed-date holiday rule.
    ///
    /// # Panics
    ///
    /// Panics if `month`/`day` is not a valid date in every year (this
    /// rejects out-of-range values and Feb 29, which does not exist in
    /// non-leap years).
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::FixedDate;
    ///
    /// let christmas = FixedDate::new("Christmas Day", 12, 25);
    /// ```
    pub fn new(name: impl Into<String>, month: i8, day: i8) -> Self {
        // Validate against a non-leap year so the rule can fire in any year.
        assert!(
            Date::new(2001, month, day).is_ok(),
            "invalid FixedDate month/day: {month}-{day}"
        );
        Self {
            name: name.into(),
            month,
            day,
            from_year: None,
            to_year: None,
        }
    }

    /// Sets the earliest year this rule applies.
    pub fn with_from_year(mut self, year: i16) -> Self {
        self.from_year = Some(year);
        self
    }

    /// Sets the latest year this rule applies.
    pub fn with_to_year(mut self, year: i16) -> Self {
        self.to_year = Some(year);
        self
    }
}

impl HolidayRule for FixedDate {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        if !year_in_bounds(year, self.from_year, self.to_year) {
            return vec![];
        }
        // Month/day are validated at construction; only an unrepresentable
        // year can fail here.
        Date::new(year, self.month, self.day).into_iter().collect()
    }
}

// ---------------------------------------------------------------------------
// NthWeekday
// ---------------------------------------------------------------------------

/// A holiday defined as the Nth occurrence of a weekday in a month.
///
/// Positive `nth` counts from the start of the month (1st, 2nd, 3rd, …).
/// Negative `nth` counts from the end (−1 = last, −2 = second-to-last, …).
///
/// Examples:
/// - Thanksgiving: 4th Thursday of November (`nth = 4`)
/// - Memorial Day: last Monday of May (`nth = -1`)
/// - MLK Day: 3rd Monday of January (`nth = 3`)
pub struct NthWeekday {
    name: String,
    month: i8,
    weekday: Weekday,
    nth: i8,
    offset_days: i8,
    from_year: Option<i16>,
    to_year: Option<i16>,
    exclude_years: Vec<i16>,
}

impl NthWeekday {
    /// Creates a new Nth-weekday holiday rule.
    ///
    /// `nth = 5` (or `-5`) is permitted but only some month/year editions
    /// have a fifth occurrence — [`holidays_in`](HolidayRule::holidays_in)
    /// returns no date for years where it does not exist.
    ///
    /// # Panics
    ///
    /// Panics if `nth` is zero or beyond ±5 (no month has six occurrences of
    /// a weekday), or if `month` is not 1–12.
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::NthWeekday;
    /// use jiff::civil::Weekday;
    ///
    /// // Thanksgiving: 4th Thursday of November
    /// let thanksgiving = NthWeekday::new("Thanksgiving", 11, Weekday::Thursday, 4);
    /// ```
    pub fn new(name: impl Into<String>, month: i8, weekday: Weekday, nth: i8) -> Self {
        assert!(
            nth != 0 && nth.abs() <= 5,
            "nth must be non-zero and within ±5, got {nth}"
        );
        assert!(
            Date::new(2001, month, 1).is_ok(),
            "invalid NthWeekday month: {month}"
        );
        Self {
            name: name.into(),
            month,
            weekday,
            nth,
            offset_days: 0,
            from_year: None,
            to_year: None,
            exclude_years: Vec::new(),
        }
    }

    /// Adds a fixed day offset to the resolved Nth weekday.
    ///
    /// Use for holidays anchored to another Nth-weekday holiday, e.g. the
    /// day after Thanksgiving = 4th Thursday of November **+ 1 day** (not
    /// the 4th Friday — those differ whenever November starts on a Friday,
    /// as in 2019 and 2024).
    ///
    /// An offset that carries the date into the neighbouring year is
    /// handled: the date is reported for the year it falls in. Year bounds
    /// and exclusions keep referring to the year of the Nth weekday itself.
    pub fn with_offset_days(mut self, days: i8) -> Self {
        self.offset_days = days;
        self
    }

    /// Sets the earliest year this rule applies.
    pub fn with_from_year(mut self, year: i16) -> Self {
        self.from_year = Some(year);
        self
    }

    /// Sets the latest year this rule applies.
    pub fn with_to_year(mut self, year: i16) -> Self {
        self.to_year = Some(year);
        self
    }

    /// Excludes a specific year from this rule.
    ///
    /// Use this for years when a recurring bank holiday was officially moved
    /// to a different date (e.g. the UK Early May Bank Holiday was moved from
    /// its usual first-Monday slot to 8 May 2020 for the VE Day anniversary).
    /// The moved date should be modelled as a separate [`DateList`] rule.
    pub fn with_exclude_year(mut self, year: i16) -> Self {
        self.exclude_years.push(year);
        self
    }
}

impl HolidayRule for NthWeekday {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        if self.offset_days == 0 {
            return self.edition(year).into_iter().collect();
        }
        // The offset can carry the date across the year boundary (at most
        // 127 days, so one neighbouring edition on either side suffices).
        neighbouring_years(year)
            .filter_map(|edition| self.edition(edition))
            .filter(|d| d.year() == year)
            .collect()
    }
}

impl NthWeekday {
    /// The holiday date of the edition anchored in `year` — the Nth weekday
    /// of the month plus the day offset — or `None` if the rule does not
    /// apply to that year (bounds, exclusion, no fifth occurrence, or a year
    /// outside the representable date range). With an offset the date can
    /// fall in a neighbouring year.
    fn edition(&self, year: i16) -> Option<Date> {
        if !year_in_bounds(year, self.from_year, self.to_year) || self.exclude_years.contains(&year)
        {
            return None;
        }
        let first = Date::new(year, self.month, 1).ok()?;
        let weeks = i64::from(self.nth.abs()) - 1;
        let nth_date = if self.nth > 0 {
            // First matching weekday of the month, then (nth − 1) weeks on.
            let to_weekday = i64::from(first.weekday().until(self.weekday));
            shifted(first, to_weekday + weeks * 7)?
        } else {
            // Last matching weekday of the month, then (|nth| − 1) weeks back.
            let last = first.last_of_month();
            let back_to_weekday = i64::from(self.weekday.until(last.weekday()));
            shifted(last, -(back_to_weekday + weeks * 7))?
        };
        // A fifth occurrence does not exist in every month/year edition —
        // the walk above then lands outside the target month.
        if nth_date.month() != self.month {
            return None;
        }
        // The day offset applies after the in-month check: it anchors a
        // holiday to the Nth weekday and may deliberately leave the month.
        shifted(nth_date, i64::from(self.offset_days))
    }
}

// ---------------------------------------------------------------------------
// EasterRelative
// ---------------------------------------------------------------------------

/// A holiday defined relative to Easter Sunday.
///
/// Examples:
/// - Good Friday: offset = −2
/// - Easter Monday: offset = +1
/// - Ascension Thursday: offset = +39
/// - Whit Monday: offset = +50
pub struct EasterRelative {
    name: String,
    offset: i16,
}

impl EasterRelative {
    /// Creates a new Easter-relative holiday rule.
    ///
    /// The `offset` is in days relative to Easter Sunday: negative for before,
    /// positive for after. An offset of 0 represents Easter Sunday itself.
    /// An offset that carries the date into the neighbouring year is handled:
    /// the date is reported for the year it falls in.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is beyond ±366 days — no holiday is anchored a
    /// year or more away from its Easter.
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::EasterRelative;
    ///
    /// let good_friday = EasterRelative::new("Good Friday", -2);
    /// let easter_monday = EasterRelative::new("Easter Monday", 1);
    /// ```
    pub fn new(name: impl Into<String>, offset: i16) -> Self {
        assert!(
            offset.abs() <= 366,
            "EasterRelative offset must be within ±366 days, got {offset}"
        );
        Self {
            name: name.into(),
            offset,
        }
    }
}

impl HolidayRule for EasterRelative {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        let anchored = |edition: i16| {
            Date::new(edition, 1, 1)
                .ok()
                .and_then(|_| shifted(easter_sunday(edition), i64::from(self.offset)))
        };
        // Easter falls between 22 March and 25 April, so an offset within
        // −80..=250 days never leaves the year — the common case, one Easter
        // computation. Beyond that (within the ±366 bound) the date can land
        // in a neighbouring year, so those editions are evaluated too.
        if (-80..=250).contains(&self.offset) {
            return anchored(year).into_iter().collect();
        }
        neighbouring_years(year)
            .filter_map(anchored)
            .filter(|d| d.year() == year)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// DateList
// ---------------------------------------------------------------------------

/// A holiday defined by an explicit list of dates.
///
/// Useful for ad-hoc or one-off holidays that don't follow a recurring pattern
/// (e.g. royal weddings, special bank holidays, COVID closures), and for
/// recurring holidays that resist rules (lunar/religious dates, gazette
/// closure spans) — declare the maintained window for those via
/// [`with_valid_range`](Self::with_valid_range).
pub struct DateList {
    name: String,
    dates: BTreeSet<Date>,
    from: Option<Date>,
    to: Option<Date>,
}

impl DateList {
    /// Creates a new date-list holiday rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::DateList;
    /// use jiff::civil::date;
    ///
    /// let queens_funeral = DateList::new(
    ///     "Queen's Funeral",
    ///     [date(2022, 9, 19)],
    /// );
    /// ```
    pub fn new(name: impl Into<String>, dates: impl IntoIterator<Item = Date>) -> Self {
        Self {
            name: name.into(),
            dates: dates.into_iter().collect(),
            from: None,
            to: None,
        }
    }

    /// Declares the inclusive year range this list exhaustively covers.
    ///
    /// Use for recurring holidays maintained as explicit dates: the list is
    /// authoritative within the declared range and silent outside it.
    /// [`RuleBasedCalendarBuilder::build`](crate::rule_calendar::RuleBasedCalendarBuilder::build)
    /// validates the owning calendar's valid range against this coverage, so
    /// extending a calendar without extending its date lists fails loudly at
    /// build time. Leave unset for genuine one-offs (a jubilee, a funeral),
    /// which are exhaustive for all time by construction.
    ///
    /// # Panics
    ///
    /// Panics if `from_year > to_year`, if either year is outside the
    /// representable date range, or if the list holds a date outside the
    /// declared range — a list that is authoritative for 2022–2026 has no
    /// business carrying a 2027 date; extend the range together with the
    /// dates.
    pub fn with_valid_range(mut self, from_year: i16, to_year: i16) -> Self {
        assert!(
            from_year <= to_year,
            "from_year ({from_year}) must be <= to_year ({to_year})"
        );
        let from = Date::new(from_year, 1, 1).expect("valid from_year");
        let to = Date::new(to_year, 12, 31).expect("valid to_year");
        if let Some(stray) = self.dates.iter().find(|d| **d < from || **d > to) {
            panic!(
                "DateList '{}' holds {stray}, outside its declared range {from_year}..={to_year}",
                self.name
            );
        }
        self.from = Some(from);
        self.to = Some(to);
        self
    }
}

impl HolidayRule for DateList {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        self.dates
            .iter()
            .copied()
            .filter(|d| d.year() == year)
            .collect()
    }

    fn valid_range(&self) -> (RangeBound, RangeBound) {
        (
            self.from.map_or(RangeBound::Unbounded, RangeBound::Date),
            self.to.map_or(RangeBound::Unbounded, RangeBound::Date),
        )
    }
}

// ---------------------------------------------------------------------------
// FixedDateRange
// ---------------------------------------------------------------------------

/// A holiday rule that marks every day in a fixed month/day range as a holiday.
///
/// All days from `(month, from_day)` through `(month, to_day)` inclusive are
/// produced each year. The range must fall within a single month.
///
/// Example: Baltic Exchange Christmas closure — Dec 25 through Dec 31.
pub struct FixedDateRange {
    name: String,
    month: i8,
    from_day: i8,
    to_day: i8,
}

impl FixedDateRange {
    /// Creates a new fixed-date-range holiday rule.
    ///
    /// # Panics
    ///
    /// Panics if `from_day > to_day`, or if the range is not a valid span of
    /// days in every year (rejects out-of-range months/days and Feb 29).
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::FixedDateRange;
    /// use fincal::rules::HolidayRule;
    ///
    /// let xmas_week = FixedDateRange::new("Christmas Week", 12, 25, 31);
    /// assert_eq!(xmas_week.holidays_in(2025).len(), 7);
    /// ```
    pub fn new(name: impl Into<String>, month: i8, from_day: i8, to_day: i8) -> Self {
        assert!(
            from_day <= to_day,
            "from_day ({from_day}) must be <= to_day ({to_day})"
        );
        // Validate against a non-leap year so the range exists in any year.
        assert!(
            Date::new(2001, month, from_day).is_ok() && Date::new(2001, month, to_day).is_ok(),
            "invalid FixedDateRange month/days: month {month}, days {from_day}..={to_day}"
        );
        Self {
            name: name.into(),
            month,
            from_day,
            to_day,
        }
    }
}

impl HolidayRule for FixedDateRange {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        // Month/days are validated at construction; only an unrepresentable
        // year can fail here.
        (self.from_day..=self.to_day)
            .filter_map(|day| Date::new(year, self.month, day).ok())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// WeekendsExcept
// ---------------------------------------------------------------------------

/// Every Saturday and Sunday as an explicit holiday — except designated
/// working weekend days.
///
/// For markets whose weekends are *not* uniformly non-working (e.g. the
/// Chinese State Council gazette designates makeup working weekends around
/// major holidays), the weekend convention cannot be a weekday pattern.
/// Instead, the owning calendar disables the implicit weekend
/// (`with_weekends(&[])`) and carries this rule, which emits every weekend
/// day as a holiday **except** the enumerated working days. Downstream
/// consumers (e.g. vendor holiday lists) then see explicit weekend holidays —
/// conventionally named `"Weekend"` — and treat unlisted weekend days as
/// working days.
pub struct WeekendsExcept {
    name: String,
    working_days: BTreeSet<Date>,
}

impl WeekendsExcept {
    /// Creates the rule with the given working-weekend exceptions.
    ///
    /// # Panics
    ///
    /// Panics if any exception date is not a Saturday or Sunday.
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::{HolidayRule, WeekendsExcept};
    /// use jiff::civil::date;
    ///
    /// // 2025 Chinese makeup working weekends (extract)
    /// let rule = WeekendsExcept::new("Weekend", [date(2025, 1, 26), date(2025, 2, 8)]);
    /// let hols = rule.holidays_in(2025);
    /// assert!(!hols.contains(&date(2025, 1, 26))); // working Sunday
    /// assert!(hols.contains(&date(2025, 1, 25)));  // ordinary Saturday
    /// ```
    pub fn new(name: impl Into<String>, working_days: impl IntoIterator<Item = Date>) -> Self {
        let working_days: BTreeSet<Date> = working_days.into_iter().collect();
        for d in &working_days {
            assert!(
                matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday),
                "working-weekend exception {d} is not a weekend day"
            );
        }
        Self {
            name: name.into(),
            working_days,
        }
    }
}

impl HolidayRule for WeekendsExcept {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        let (Ok(start), Ok(end)) = (Date::new(year, 1, 1), Date::new(year, 12, 31)) else {
            return vec![];
        };
        crate::days_inclusive(start, end)
            .filter(|d| {
                matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday)
                    && !self.working_days.contains(d)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// ReducedWeekdaySchedule
// ---------------------------------------------------------------------------

/// A reduced publication schedule: within a date window, every weekday
/// *except* the given publication weekdays is a non-publication day.
///
/// Models benchmark-frequency reductions such as the Baltic Exchange moving
/// BLPG1 from daily to weekly (Wednesdays), then twice-weekly (Tue+Thu),
/// during the 2026 Strait of Hormuz crisis. Saturdays and Sundays are never
/// produced (they are non-business days via the weekend convention, and
/// emitting them would double-count).
///
/// The window end is optional: `None` means "until further notice" — the rule
/// stays active to the end of any queried year, so the owning calendar should
/// bound its `valid_range` to the horizon actually announced.
pub struct ReducedWeekdaySchedule {
    name: String,
    from: Date,
    to: Option<Date>,
    publication_weekdays: Vec<Weekday>,
}

impl ReducedWeekdaySchedule {
    /// Creates a reduced-schedule rule for `[from, to]` (inclusive; `to =
    /// None` is open-ended) with the given publication weekdays.
    ///
    /// # Panics
    ///
    /// Panics if `to` is before `from`, or if `publication_weekdays` is empty
    /// (an empty publication set would be a full closure — use explicit
    /// holiday rules for that).
    ///
    /// # Examples
    ///
    /// ```
    /// use fincal::rules::{HolidayRule, ReducedWeekdaySchedule};
    /// use jiff::civil::{Weekday, date};
    ///
    /// // Weekly (Wednesday-only) publication for March 2026:
    /// let rule = ReducedWeekdaySchedule::new(
    ///     "weekly demo",
    ///     date(2026, 3, 4),
    ///     Some(date(2026, 3, 31)),
    ///     [Weekday::Wednesday],
    /// );
    /// let holidays = rule.holidays_in(2026);
    /// assert!(holidays.contains(&date(2026, 3, 5)));   // Thursday suppressed
    /// assert!(!holidays.contains(&date(2026, 3, 11))); // Wednesday publishes
    /// assert!(!holidays.contains(&date(2026, 3, 7)));  // Saturday not emitted
    /// ```
    pub fn new(
        name: impl Into<String>,
        from: Date,
        to: Option<Date>,
        publication_weekdays: impl IntoIterator<Item = Weekday>,
    ) -> Self {
        let publication_weekdays: Vec<Weekday> = publication_weekdays.into_iter().collect();
        assert!(
            !publication_weekdays.is_empty(),
            "publication_weekdays must not be empty"
        );
        if let Some(to) = to {
            assert!(from <= to, "from ({from}) must be <= to ({to})");
        }
        Self {
            name: name.into(),
            from,
            to,
            publication_weekdays,
        }
    }
}

impl HolidayRule for ReducedWeekdaySchedule {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        let (Ok(year_start), Ok(year_end)) = (Date::new(year, 1, 1), Date::new(year, 12, 31))
        else {
            return vec![];
        };
        let start = self.from.max(year_start);
        let end = self.to.map_or(year_end, |to| to.min(year_end));

        crate::days_inclusive(start, end)
            .filter(|d| {
                let wd = d.weekday();
                !matches!(wd, Weekday::Saturday | Weekday::Sunday)
                    && !self.publication_weekdays.contains(&wd)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Observance policies
// ---------------------------------------------------------------------------

/// Policy for shifting a weekend holiday to a nearby weekday.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservancePolicy {
    /// US federal: Saturday → preceding Friday, Sunday → following Monday.
    SaturdayToFridaySundayToMonday,
    /// Saturday or Sunday → following Monday.
    NextMonday,
    /// Only Sunday → following Monday (Saturday is ignored).
    ///
    /// Used by Platts for Asian market calendars: when a holiday falls on
    /// Saturday (already a non-publication day), no substitute Monday is given.
    /// Only Sunday holidays shift to Monday.
    SundayToMonday,
}

// ---------------------------------------------------------------------------
// ObservedFixedDate
// ---------------------------------------------------------------------------

/// An observance rule for a fixed-date holiday that falls on a weekend.
///
/// When the base date (month/day) falls on a weekend, this rule produces the
/// observed (shifted) date according to the given [`ObservancePolicy`]. When
/// the base date falls on a weekday, this rule produces **no** dates — the
/// base [`FixedDate`] rule handles it, avoiding double-counting.
///
/// # Cross-year boundaries
///
/// A Saturday January 1 shifts to Friday December 31 of the *previous* year
/// under [`ObservancePolicy::SaturdayToFridaySundayToMonday`]. The rule
/// handles this internally by checking editions `year-1`, `year`, and
/// `year+1`, returning only dates that fall within `year`.
pub struct ObservedFixedDate {
    name: String,
    month: i8,
    day: i8,
    policy: ObservancePolicy,
    from_year: Option<i16>,
    to_year: Option<i16>,
}

impl ObservedFixedDate {
    /// Creates a new observed-fixed-date rule.
    ///
    /// # Panics
    ///
    /// Panics if `month`/`day` is not a valid date in every year (rejects
    /// out-of-range values and Feb 29).
    pub fn new(name: impl Into<String>, month: i8, day: i8, policy: ObservancePolicy) -> Self {
        // Validate against a non-leap year so the rule can fire in any year.
        assert!(
            Date::new(2001, month, day).is_ok(),
            "invalid ObservedFixedDate month/day: {month}-{day}"
        );
        Self {
            name: name.into(),
            month,
            day,
            policy,
            from_year: None,
            to_year: None,
        }
    }

    /// Sets the earliest year this rule applies.
    pub fn with_from_year(mut self, year: i16) -> Self {
        self.from_year = Some(year);
        self
    }

    /// Sets the latest year this rule applies.
    pub fn with_to_year(mut self, year: i16) -> Self {
        self.to_year = Some(year);
        self
    }
}

impl HolidayRule for ObservedFixedDate {
    fn name(&self) -> &str {
        &self.name
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        if !year_in_bounds(year, self.from_year, self.to_year) {
            return vec![];
        }
        let mut results = Vec::new();
        // Check editions year-1, year, year+1 to catch cross-year shifts.
        // Only return dates that actually fall within `year`.
        for edition in neighbouring_years(year) {
            if !year_in_bounds(edition, self.from_year, self.to_year) {
                continue;
            }
            // Month/day are validated at construction; an edition outside
            // the representable date range simply does not exist.
            let Ok(raw) = Date::new(edition, self.month, self.day) else {
                continue;
            };
            let observed = match (raw.weekday(), self.policy) {
                (Weekday::Saturday, ObservancePolicy::SaturdayToFridaySundayToMonday) => {
                    shifted(raw, -1)
                }
                (Weekday::Sunday, ObservancePolicy::SaturdayToFridaySundayToMonday) => {
                    shifted(raw, 1)
                }
                (Weekday::Saturday, ObservancePolicy::NextMonday) => shifted(raw, 2),
                (Weekday::Sunday, ObservancePolicy::NextMonday) => shifted(raw, 1),
                (Weekday::Sunday, ObservancePolicy::SundayToMonday) => shifted(raw, 1),
                // Saturday with SundayToMonday: no shift (already non-publication).
                (Weekday::Saturday, ObservancePolicy::SundayToMonday) => None,
                // Weekday: no observance shift needed — base FixedDate handles it.
                _ => None,
            };
            if let Some(d) = observed
                && d.year() == year
            {
                results.push(d);
            }
        }
        results
    }
}

// ---------------------------------------------------------------------------
// UK Christmas/Boxing Day cascade
// ---------------------------------------------------------------------------

/// UK substitute day for Christmas (December 25).
///
/// When Christmas falls on a Saturday, the observed day is Monday December 27.
/// When Christmas falls on a Sunday, the observed day is Tuesday December 27.
/// On weekdays this rule produces no dates.
pub struct UkChristmasObserved;

impl HolidayRule for UkChristmasObserved {
    fn name(&self) -> &str {
        "Christmas Day (UK Observed)"
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        let Ok(christmas) = Date::new(year, 12, 25) else {
            return vec![];
        };
        match christmas.weekday() {
            // Sat 25 → Mon 27, Sun 25 → Tue 27
            Weekday::Saturday | Weekday::Sunday => shifted(christmas, 2).into_iter().collect(),
            _ => vec![],
        }
    }
}

/// UK substitute day for Boxing Day (December 26).
///
/// When Boxing Day falls on a Saturday, the observed day is Monday December 28.
/// When Boxing Day falls on a Sunday, the observed day is Tuesday December 28.
/// On weekdays this rule produces no dates.
pub struct UkBoxingDayObserved;

impl HolidayRule for UkBoxingDayObserved {
    fn name(&self) -> &str {
        "Boxing Day (UK Observed)"
    }

    fn holidays_in(&self, year: i16) -> Vec<Date> {
        let Ok(boxing_day) = Date::new(year, 12, 26) else {
            return vec![];
        };
        match boxing_day.weekday() {
            // Sat 26 → Mon 28, Sun 26 → Tue 28
            Weekday::Saturday | Weekday::Sunday => shifted(boxing_day, 2).into_iter().collect(),
            _ => vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

/// Returns a [`FixedDate`] rule paired with an [`ObservedFixedDate`] rule.
///
/// This is the common pattern for US federal holidays: the base rule fires on
/// weekdays, and the observed rule fires when the base date is a weekend.
///
/// # Examples
///
/// ```
/// use fincal::rules::{fixed_date_with_observed, ObservancePolicy};
///
/// let rules = fixed_date_with_observed(
///     "New Year's Day",
///     1, 1,
///     ObservancePolicy::SaturdayToFridaySundayToMonday,
/// );
/// assert_eq!(rules.len(), 2);
/// ```
pub fn fixed_date_with_observed(
    name: &str,
    month: i8,
    day: i8,
    policy: ObservancePolicy,
) -> Vec<Arc<dyn HolidayRule>> {
    vec![
        Arc::new(FixedDate::new(name, month, day)),
        Arc::new(ObservedFixedDate::new(
            format!("{name} (Observed)"),
            month,
            day,
            policy,
        )),
    ]
}

/// Returns a year-bounded [`FixedDate`] rule paired with an [`ObservedFixedDate`] rule.
///
/// Both rules share the same `from_year` and `to_year` bounds.
pub fn fixed_date_with_observed_bounded(
    name: &str,
    month: i8,
    day: i8,
    policy: ObservancePolicy,
    from_year: Option<i16>,
    to_year: Option<i16>,
) -> Vec<Arc<dyn HolidayRule>> {
    let mut base = FixedDate::new(name, month, day);
    let mut observed = ObservedFixedDate::new(format!("{name} (Observed)"), month, day, policy);
    if let Some(y) = from_year {
        base = base.with_from_year(y);
        observed = observed.with_from_year(y);
    }
    if let Some(y) = to_year {
        base = base.with_to_year(y);
        observed = observed.with_to_year(y);
    }
    vec![Arc::new(base), Arc::new(observed)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    // --- FixedDate ---

    #[test]
    fn fixed_date_resolves() {
        let rule = FixedDate::new("Christmas Day", 12, 25);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 12, 25)]);
        assert_eq!(rule.name(), "Christmas Day");
    }

    #[test]
    fn fixed_date_with_bounds() {
        let rule = FixedDate::new("Juneteenth", 6, 19).with_from_year(2021);
        assert_eq!(rule.holidays_in(2021), vec![date(2021, 6, 19)]);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 6, 19)]);
        assert!(rule.holidays_in(2020).is_empty());
    }

    #[test]
    fn fixed_date_to_year_bound() {
        let rule = FixedDate::new("Abolished Holiday", 3, 15).with_to_year(2020);
        assert_eq!(rule.holidays_in(2020), vec![date(2020, 3, 15)]);
        assert!(rule.holidays_in(2021).is_empty());
    }

    // --- NthWeekday ---

    #[test]
    fn nth_weekday_thanksgiving() {
        // 4th Thursday of November
        let rule = NthWeekday::new("Thanksgiving", 11, Weekday::Thursday, 4);
        assert_eq!(rule.holidays_in(2024), vec![date(2024, 11, 28)]);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 11, 27)]);
        assert_eq!(rule.holidays_in(2023), vec![date(2023, 11, 23)]);
    }

    #[test]
    fn nth_weekday_mlk_day() {
        // 3rd Monday of January
        let rule = NthWeekday::new("MLK Day", 1, Weekday::Monday, 3);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 1, 20)]);
        assert_eq!(rule.holidays_in(2024), vec![date(2024, 1, 15)]);
    }

    #[test]
    fn nth_weekday_memorial_day() {
        // Last Monday of May
        let rule = NthWeekday::new("Memorial Day", 5, Weekday::Monday, -1);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 5, 26)]);
        assert_eq!(rule.holidays_in(2024), vec![date(2024, 5, 27)]);
        assert_eq!(rule.holidays_in(2023), vec![date(2023, 5, 29)]);
    }

    #[test]
    fn nth_weekday_labor_day() {
        // 1st Monday of September
        let rule = NthWeekday::new("Labor Day", 9, Weekday::Monday, 1);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 9, 1)]);
        assert_eq!(rule.holidays_in(2024), vec![date(2024, 9, 2)]);
    }

    #[test]
    fn nth_weekday_with_exclude_year() {
        let rule = NthWeekday::new("Early May", 5, Weekday::Monday, 1).with_exclude_year(2020);
        // Normal years work
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 5, 5)]);
        assert_eq!(rule.holidays_in(2019), vec![date(2019, 5, 6)]);
        // Excluded year returns empty
        assert!(rule.holidays_in(2020).is_empty());
    }

    #[test]
    fn nth_weekday_with_from_year() {
        let rule = NthWeekday::new("Test", 1, Weekday::Monday, 1).with_from_year(2024);
        assert!(rule.holidays_in(2023).is_empty());
        assert!(!rule.holidays_in(2024).is_empty());
    }

    // --- EasterRelative ---

    #[test]
    fn easter_relative_good_friday() {
        let rule = EasterRelative::new("Good Friday", -2);
        // 2025: Easter = Apr 20 -> Good Friday = Apr 18
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 4, 18)]);
        // 2024: Easter = Mar 31 -> Good Friday = Mar 29
        assert_eq!(rule.holidays_in(2024), vec![date(2024, 3, 29)]);
    }

    #[test]
    fn easter_relative_easter_monday() {
        let rule = EasterRelative::new("Easter Monday", 1);
        // 2025: Easter = Apr 20 -> Monday = Apr 21
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 4, 21)]);
    }

    // --- DateList ---

    #[test]
    fn date_list_resolves_matching_year() {
        let rule = DateList::new("Special", [date(2022, 9, 19), date(2023, 5, 8)]);
        assert_eq!(rule.holidays_in(2022), vec![date(2022, 9, 19)]);
        assert_eq!(rule.holidays_in(2023), vec![date(2023, 5, 8)]);
        assert!(rule.holidays_in(2024).is_empty());
    }

    // --- ObservedFixedDate ---

    #[test]
    fn observed_us_new_year_saturday() {
        // 2022: Jan 1 is Saturday -> observed Fri Dec 31, 2021
        let rule = ObservedFixedDate::new(
            "New Year's Day (Observed)",
            1,
            1,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        );
        // Querying 2022 returns nothing — the observed date fell in 2021.
        assert!(rule.holidays_in(2022).is_empty());
        // Querying 2021 returns the Dec 31 date.
        assert_eq!(rule.holidays_in(2021), vec![date(2021, 12, 31)]);
    }

    #[test]
    fn observed_us_new_year_sunday() {
        // 2023: Jan 1 is Sunday -> observed Mon Jan 2
        let rule = ObservedFixedDate::new(
            "New Year's Day (Observed)",
            1,
            1,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        );
        let dates = rule.holidays_in(2023);
        assert_eq!(dates, vec![date(2023, 1, 2)]);
    }

    #[test]
    fn observed_us_weekday_returns_empty() {
        // 2025: Jan 1 is Wednesday -> no observed shift
        let rule = ObservedFixedDate::new(
            "New Year's Day (Observed)",
            1,
            1,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        );
        assert!(rule.holidays_in(2025).is_empty());
    }

    #[test]
    fn observed_next_monday_saturday() {
        // NextMonday: Saturday -> following Monday
        let rule = ObservedFixedDate::new("Test (Observed)", 1, 1, ObservancePolicy::NextMonday);
        // 2022: Jan 1 is Saturday -> Mon Jan 3
        let dates = rule.holidays_in(2022);
        assert_eq!(dates, vec![date(2022, 1, 3)]);
    }

    #[test]
    fn observed_next_monday_sunday() {
        // NextMonday: Sunday -> following Monday
        let rule = ObservedFixedDate::new("Test (Observed)", 1, 1, ObservancePolicy::NextMonday);
        // 2023: Jan 1 is Sunday -> Mon Jan 2
        let dates = rule.holidays_in(2023);
        assert_eq!(dates, vec![date(2023, 1, 2)]);
    }

    #[test]
    fn observed_sunday_to_monday_sunday() {
        // SundayToMonday: Sunday → following Monday
        let rule =
            ObservedFixedDate::new("Test (Observed)", 1, 1, ObservancePolicy::SundayToMonday);
        // 2023: Jan 1 is Sunday → Mon Jan 2
        let dates = rule.holidays_in(2023);
        assert_eq!(dates, vec![date(2023, 1, 2)]);
    }

    #[test]
    fn observed_sunday_to_monday_saturday_ignored() {
        // SundayToMonday: Saturday → no shift (ignored)
        let rule =
            ObservedFixedDate::new("Test (Observed)", 1, 1, ObservancePolicy::SundayToMonday);
        // 2022: Jan 1 is Saturday → no observed shift
        assert!(rule.holidays_in(2022).is_empty());
    }

    #[test]
    fn observed_sunday_to_monday_weekday() {
        // SundayToMonday: weekday → no shift
        let rule =
            ObservedFixedDate::new("Test (Observed)", 1, 1, ObservancePolicy::SundayToMonday);
        // 2025: Jan 1 is Wednesday → no shift
        assert!(rule.holidays_in(2025).is_empty());
    }

    #[test]
    fn observed_with_bounds() {
        let rule = ObservedFixedDate::new(
            "Test (Observed)",
            6,
            19,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )
        .with_from_year(2021);
        // 2021: Jun 19 is Saturday -> observed Fri Jun 18
        assert_eq!(rule.holidays_in(2021), vec![date(2021, 6, 18)]);
        // 2020: before from_year
        assert!(rule.holidays_in(2020).is_empty());
    }

    // --- UK Christmas/Boxing cascade ---

    #[test]
    fn uk_christmas_observed_saturday() {
        // 2021: Dec 25 is Saturday -> observed Mon Dec 27
        assert_eq!(
            UkChristmasObserved.holidays_in(2021),
            vec![date(2021, 12, 27)]
        );
    }

    #[test]
    fn uk_christmas_observed_sunday() {
        // 2022: Dec 25 is Sunday -> observed Tue Dec 27
        assert_eq!(
            UkChristmasObserved.holidays_in(2022),
            vec![date(2022, 12, 27)]
        );
    }

    #[test]
    fn uk_christmas_observed_weekday() {
        // 2024: Dec 25 is Wednesday -> no observed shift
        assert!(UkChristmasObserved.holidays_in(2024).is_empty());
    }

    #[test]
    fn uk_boxing_day_observed_saturday() {
        // 2020: Dec 26 is Saturday -> observed Mon Dec 28
        assert_eq!(
            UkBoxingDayObserved.holidays_in(2020),
            vec![date(2020, 12, 28)]
        );
    }

    #[test]
    fn uk_boxing_day_observed_sunday() {
        // 2021: Dec 26 is Sunday -> observed Tue Dec 28
        assert_eq!(
            UkBoxingDayObserved.holidays_in(2021),
            vec![date(2021, 12, 28)]
        );
    }

    #[test]
    fn uk_boxing_day_observed_weekday() {
        // 2024: Dec 26 is Thursday -> no observed shift
        assert!(UkBoxingDayObserved.holidays_in(2024).is_empty());
    }

    // --- FixedDateRange ---

    #[test]
    fn fixed_date_range_resolves_all_days() {
        let rule = FixedDateRange::new("Christmas Week", 12, 25, 31);
        let dates = rule.holidays_in(2025);
        assert_eq!(dates.len(), 7);
        assert_eq!(dates[0], date(2025, 12, 25));
        assert_eq!(dates[6], date(2025, 12, 31));
        assert_eq!(rule.name(), "Christmas Week");
    }

    #[test]
    fn fixed_date_range_single_day() {
        let rule = FixedDateRange::new("Single", 1, 15, 15);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 1, 15)]);
    }

    // --- Convenience constructors ---

    #[test]
    fn fixed_date_with_observed_creates_pair() {
        let rules = fixed_date_with_observed(
            "New Year's Day",
            1,
            1,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        );
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].name(), "New Year's Day");
        assert_eq!(rules[1].name(), "New Year's Day (Observed)");
    }

    #[test]
    fn fixed_date_with_observed_bounded_creates_bounded_pair() {
        let rules = fixed_date_with_observed_bounded(
            "Juneteenth",
            6,
            19,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
            Some(2021),
            None,
        );
        assert_eq!(rules.len(), 2);
        // Before from_year: both return empty (holiday didn't exist yet)
        assert!(rules[0].holidays_in(2020).is_empty());
        assert!(rules[1].holidays_in(2020).is_empty());
        // From 2021: base fires
        assert_eq!(rules[0].holidays_in(2025), vec![date(2025, 6, 19)]);
    }

    // --- Constructor validation ---

    #[test]
    #[should_panic(expected = "invalid FixedDate")]
    fn fixed_date_rejects_invalid_day() {
        FixedDate::new("Bad", 2, 30);
    }

    #[test]
    #[should_panic(expected = "invalid FixedDate")]
    fn fixed_date_rejects_feb_29() {
        // Feb 29 does not exist in every year — a FixedDate can't carry it.
        FixedDate::new("Bad", 2, 29);
    }

    #[test]
    #[should_panic(expected = "invalid NthWeekday month")]
    fn nth_weekday_rejects_invalid_month() {
        NthWeekday::new("Bad", 13, Weekday::Monday, 1);
    }

    #[test]
    #[should_panic(expected = "within ±5")]
    fn nth_weekday_rejects_nth_beyond_five() {
        NthWeekday::new("Bad", 1, Weekday::Monday, 6);
    }

    #[test]
    #[should_panic(expected = "invalid FixedDateRange")]
    fn fixed_date_range_rejects_invalid_days() {
        FixedDateRange::new("Bad", 2, 25, 30);
    }

    #[test]
    #[should_panic(expected = "invalid ObservedFixedDate")]
    fn observed_fixed_date_rejects_invalid_day() {
        ObservedFixedDate::new("Bad", 4, 31, ObservancePolicy::NextMonday);
    }

    // --- Dates that cross the year boundary ---

    #[test]
    fn nth_weekday_offset_crossing_the_year_is_reported_in_the_year_it_falls_in() {
        // Last Monday of December + 7 days lands in the following January.
        let rule = NthWeekday::new("Week after", 12, Weekday::Monday, -1).with_offset_days(7);
        // Edition 2025: Mon 2025-12-29 + 7 = Mon 2026-01-05.
        assert_eq!(rule.holidays_in(2026), vec![date(2026, 1, 5)]);
        // Edition 2024: Mon 2024-12-30 + 7 = Mon 2025-01-06.
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 1, 6)]);
        // Bounds refer to the year of the Nth weekday, not of the result.
        let bounded = NthWeekday::new("Week after", 12, Weekday::Monday, -1)
            .with_offset_days(7)
            .with_from_year(2025);
        assert!(bounded.holidays_in(2025).is_empty());
        assert_eq!(bounded.holidays_in(2026), vec![date(2026, 1, 5)]);
    }

    #[test]
    fn nth_weekday_matches_a_day_by_day_search() {
        // The closed-form weekday arithmetic against the obvious definition.
        for year in [2023i16, 2024, 2025, 2026] {
            for month in 1..=12i8 {
                let in_month: Vec<Date> = crate::days_inclusive(
                    date(year, month, 1),
                    date(year, month, 1).last_of_month(),
                )
                .filter(|d| d.weekday() == Weekday::Wednesday)
                .collect();
                for nth in 1..=5i8 {
                    let forward = NthWeekday::new("n", month, Weekday::Wednesday, nth);
                    let expected: Vec<Date> = in_month
                        .get(nth as usize - 1)
                        .copied()
                        .into_iter()
                        .collect();
                    assert_eq!(
                        forward.holidays_in(year),
                        expected,
                        "{year}-{month} nth {nth}"
                    );
                    let backward = NthWeekday::new("n", month, Weekday::Wednesday, -nth);
                    let expected: Vec<Date> = in_month
                        .len()
                        .checked_sub(nth as usize)
                        .map(|i| in_month[i])
                        .into_iter()
                        .collect();
                    assert_eq!(
                        backward.holidays_in(year),
                        expected,
                        "{year}-{month} nth -{nth}"
                    );
                }
            }
        }
    }

    #[test]
    fn easter_relative_offset_crossing_the_year() {
        // Easter 2025 = Apr 20; +300 days = 2026-02-14.
        let rule = EasterRelative::new("Easter + 300", 300);
        assert_eq!(rule.holidays_in(2026), vec![date(2026, 2, 14)]);
        // 2025 gets the date anchored on Easter 2024 (Mar 31) + 300 = 2025-01-25.
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 1, 25)]);
    }

    #[test]
    fn easter_relative_fast_path_bounds_stay_in_the_year() {
        // −80 and +250 are the widest offsets served by the single-edition
        // path; over four centuries of Easters they must never leave the year.
        for year in 1900i16..=2300 {
            for offset in [-80i16, 250] {
                let dates = EasterRelative::new("edge", offset).holidays_in(year);
                assert_eq!(dates.len(), 1);
                assert_eq!(dates[0].year(), year, "offset {offset}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "within ±366 days")]
    fn easter_relative_rejects_absurd_offset() {
        EasterRelative::new("Bad", 400);
    }

    // --- Totality: no panic for any year ---

    #[test]
    fn rules_are_total_over_all_years() {
        let rules: Vec<Arc<dyn HolidayRule>> = vec![
            Arc::new(FixedDate::new("f", 12, 31)),
            Arc::new(NthWeekday::new("n", 12, Weekday::Monday, -1).with_offset_days(127)),
            Arc::new(NthWeekday::new("n", 1, Weekday::Monday, 1).with_offset_days(-127)),
            Arc::new(EasterRelative::new("e", 366)),
            Arc::new(EasterRelative::new("e", -366)),
            Arc::new(DateList::new("d", [date(9999, 12, 31)])),
            Arc::new(FixedDateRange::new("r", 12, 25, 31)),
            Arc::new(WeekendsExcept::new("w", [])),
            Arc::new(ReducedWeekdaySchedule::new(
                "s",
                date(9999, 12, 1),
                None,
                [Weekday::Wednesday],
            )),
            Arc::new(ObservedFixedDate::new(
                "o",
                1,
                1,
                ObservancePolicy::SaturdayToFridaySundayToMonday,
            )),
            Arc::new(ObservedFixedDate::new(
                "o",
                12,
                31,
                ObservancePolicy::NextMonday,
            )),
            Arc::new(UkChristmasObserved),
            Arc::new(UkBoxingDayObserved),
        ];
        for year in [
            i16::MIN,
            -10000,
            -9999,
            -9998,
            0,
            9998,
            9999,
            10000,
            i16::MAX,
        ] {
            for rule in &rules {
                for d in rule.holidays_in(year) {
                    assert_eq!(d.year(), year, "rule '{}'", rule.name());
                }
            }
        }
    }

    // --- Fifth-occurrence semantics ---

    #[test]
    fn nth_weekday_fifth_occurrence_only_when_it_exists() {
        // Feb 2025 has four Mondays → no date; June 2025 has five → fires.
        let rule = NthWeekday::new("Fifth Monday", 2, Weekday::Monday, 5);
        assert!(rule.holidays_in(2025).is_empty());
        let rule = NthWeekday::new("Fifth Monday", 6, Weekday::Monday, 5);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 6, 30)]);
    }

    #[test]
    fn nth_weekday_negative_fifth_occurrence_only_when_it_exists() {
        let rule = NthWeekday::new("Fifth-last Monday", 2, Weekday::Monday, -5);
        assert!(rule.holidays_in(2025).is_empty());
        let rule = NthWeekday::new("Fifth-last Monday", 6, Weekday::Monday, -5);
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 6, 2)]);
    }

    // --- DateList coverage ---

    #[test]
    #[should_panic(expected = "outside its declared range 2022..=2026")]
    fn date_list_rejects_date_outside_declared_range() {
        let _ = DateList::new("Stray", [date(2024, 5, 1), date(2027, 5, 1)])
            .with_valid_range(2022, 2026);
    }

    #[test]
    fn date_list_valid_range_declaration() {
        let one_off = DateList::new("One-off", [date(2022, 9, 19)]);
        assert_eq!(
            one_off.valid_range(),
            (RangeBound::Unbounded, RangeBound::Unbounded)
        );
        let recurring =
            DateList::new("Recurring", [date(2022, 9, 19)]).with_valid_range(2022, 2026);
        assert_eq!(
            recurring.valid_range(),
            (
                RangeBound::Date(date(2022, 1, 1)),
                RangeBound::Date(date(2026, 12, 31))
            )
        );
    }
}
