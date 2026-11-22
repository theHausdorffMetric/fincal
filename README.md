# fincal

A rule-based financial holiday calendar framework in Rust. `fincal` answers the
core questions of market-data scheduling — *is this a business day?*, *what is the
next one?*, *which holiday is this?* — and the inverse problem — *given a list of
dates, which market calendar do they follow?*

It is a pure library with no I/O, no async, and no database; with
`default-features = false` it depends on [`jiff`](https://docs.rs/jiff) alone.
The default-on Cargo feature `cli` adds the `fincal` binary (pulls `clap`).
It ships with built-in calendars for major energy and commodity benchmark
publication schedules and a CLI for inspecting and matching them.

## Overview

The library is built in three layers:

1. **Rules** (`HolidayRule`) — self-describing atoms that compute the date(s) of a
   single holiday for any year (e.g. "Thanksgiving = 4th Thursday of November").
2. **Calendars** (`HolidayCalendar`) — compose rules (or explicit date sets) into a
   complete market calendar with a time zone, weekend pattern, and valid range.
3. **Analysis** — compare two calendars, rank candidate calendars against observed
   dates, or fit individual rules to explain gaps in a time series.

A `RuleRegistry` sits alongside as a flat pool of all known rules, so questions like
"*which* rules fire on 2025-12-26?" can be answered without going through a specific
calendar.

## Features

- `HolidayCalendar` trait: `is_holiday`, `is_business_day`, business-day navigation,
  ISDA date adjustment (all six conventions), business-day counts, month-end queries,
  and holiday naming.
- Ten reusable rule types covering fixed dates, nth-weekday patterns, Easter-relative
  holidays, ad-hoc date lists, date ranges, weekend-observance shifts, reduced
  weekday publication schedules, and explicit weekends with gazette working-day
  exceptions.
- Twenty built-in market calendars (TARGET, EEX Power, US, UK, ICE, SGX, Platts, OPIS, ICIS, China futures exchanges, Argus, Baltic Exchange).
- In-house Easter computation (Meeus/Jones/Butcher) — no extra dependency.
- Bounded calendars with explicit `valid_range`; out-of-range queries return a typed
  error rather than a wrong answer.
- Calendar comparison and calendar/rule "explain" tooling for reconciling observed
  market data against expected calendars.
- A `fincal` CLI for listing calendars, dumping holidays, explaining a date,
  matching piped dates, and checking how long each calendar remains valid.

## The `HolidayCalendar` trait

Every calendar implements five required methods — `name`, `tz`, `valid_range`,
`is_holiday`, `is_business_day` — and inherits eleven derived methods that can be
overridden for performance:

| Method | Description |
|--------|-------------|
| `covers(from, to)` | Whether the valid range contains the inclusive range `[from, to]` |
| `next_business_day` / `prev_business_day` | Strictly next/previous business day |
| `add_business_days(date, n)` | Step `n` business days forward (`n > 0`) or backward (`n < 0`); the start date is never counted |
| `adjust(date, convention)` | ISDA business-day adjustment |
| `business_days_between(start, end)` | Count business days in `[start, end)` (signed) |
| `holidays_between(start, end)` | Holiday dates in `[start, end)` |
| `holidays_in_year(year)` | Holiday dates of a whole calendar year (31 December included) |
| `end_of_month` / `is_end_of_month` | Last business day of the month |
| `holiday_name(date)` | Name of the holiday on a date, if any |

Date-dependent methods return `Result<_, OutOfRangeError>`: a query outside the
calendar's `valid_range` is treated as a logical error, not a silent default. Calendars
with no natural bound (e.g. `NoHolidays`) use `RangeBound::Unbounded`; they fail only
where a walk would leave the range of representable dates (years −9999..=9999). No
method panics.

Two conventions hold for every implementation:

- **Holidays vs weekends.** Each day is a business day, a holiday, or a regular
  non-business day (the implicit weekend). `is_holiday` implies not `is_business_day`,
  and a holiday dated on an implicit weekend day is *not* reported as a holiday — the
  weekend already closes the day; the holiday shows up through its observed substitute.
- **Intervals.** The `*_between` methods are half-open `[start, end)`; everything else
  (`valid_range`, `covers`, `holidays_in_year`, the comparison and fitting functions)
  is inclusive.

`adjust` supports the six ISDA `BusinessDayConvention` variants: `NoAdjustment`,
`Following`, `ModifiedFollowing`, `Preceding`, `ModifiedPreceding`, and `Nearest`.

## Calendar types

**Structural calendars** (`calendars` module):

| Type | Behaviour |
|------|-----------|
| `NoHolidays` | Every day is a business day (unbounded) |
| `WeekendsOnly` | Sat/Sun are non-business days, no holidays (unbounded) |
| `BespokeCalendar` | Explicit holiday set + weekend pattern over a bounded range |

`BespokeCalendar` is the bridge for external data: build it from a list of holidays
(`with_holidays`) or from an authoritative list of business days
(`from_business_days`), set a custom weekend pattern (`with_weekends`), and mutate it
at runtime (`add_holiday` / `remove_holiday`).

**Rule-based calendars** (`rule_calendar` module): `RuleBasedCalendar` composes
`HolidayRule`s and can extrapolate holidays to any year. It is built via a builder:

```rust
use std::sync::Arc;
use fincal::rule_calendar::RuleBasedCalendar;
use fincal::rules::{FixedDate, EasterRelative};
use fincal::HolidayCalendar;
use jiff::civil::date;

let cal = RuleBasedCalendar::builder("Simple")
    .with_rules(vec![
        Arc::new(FixedDate::new("Christmas Day", 12, 25)),
        Arc::new(EasterRelative::new("Good Friday", -2)),
    ])
    .with_valid_range(2000, 2099)
    .build()
    .unwrap();

assert!(cal.is_holiday(date(2025, 12, 25)).unwrap());
assert_eq!(cal.holiday_name(date(2025, 4, 18)).unwrap(), Some("Good Friday"));
```

The builder validates that the requested `valid_range` fits within the coverage of
every rule (`CalendarBuildError::RuleCoverage`); if no range is set, it infers the
tightest envelope of the rules. A range without a single day in it — inverted, or
inferred from rule coverages that do not overlap — is rejected as
`CalendarBuildError::EmptyRange`.

## Holiday rules

Rules implement the `HolidayRule` trait (`name`, `holidays_in(year)`, `valid_range`).
The contract of `holidays_in`: every returned date lies in the queried year (a rule
whose date can shift across the year boundary evaluates the neighbouring editions
itself), and the method never panics, whatever the year. Ten built-in types cover the
common cases:

| Rule | Produces | Example |
|------|----------|---------|
| `FixedDate` | Same month/day every year | Christmas Day (Dec 25) |
| `NthWeekday` | Nth (or last) weekday of a month | Thanksgiving (4th Thu of Nov) |
| `EasterRelative` | Easter Sunday ± offset | Good Friday (Easter − 2) |
| `DateList` | Explicit one-off dates | Coronation, lunar holidays |
| `FixedDateRange` | Every day in a month/day range | Baltic Christmas closure (Dec 25–31) |
| `ReducedWeekdaySchedule` | Non-publication weekdays within a date window | BLPG1 weekly/Tue+Thu publication (2026 Hormuz crisis) |
| `WeekendsExcept` | Every weekend day as an explicit holiday, minus working-weekend exceptions | ICIS China "Weekend" holidays vs gazette makeup days |
| `ObservedFixedDate` | Weekend → weekday shift of a fixed date | New Year's Day observed |
| `UkChristmasObserved` | UK Christmas substitute day | Sat/Sun Dec 25 → following workday |
| `UkBoxingDayObserved` | UK Boxing Day substitute day | Sat/Sun Dec 26 → following workday |

`FixedDate`, `NthWeekday`, and `ObservedFixedDate` can be year-bounded
(`with_from_year` / `with_to_year`) to model holidays that were introduced or
abolished (e.g. Juneteenth from 2021), and `NthWeekday` can exclude individual years
(`with_exclude_year`) for one-off moves (e.g. the UK Early May bank holiday moved to
8 May 2020 for VE Day).

### Observance is modelled as rules, not post-processing

Weekend-to-weekday shifting is a separate `ObservedFixedDate` rule that fires *only*
when the base date lands on a weekend, so the base `FixedDate` and its observed
variant never double-count. The shift policy is one of three `ObservancePolicy`
values:

| Policy | Sat → | Sun → | Used by |
|--------|-------|-------|---------|
| `SaturdayToFridaySundayToMonday` | preceding Fri | following Mon | US federal |
| `NextMonday` | following Mon | following Mon | UK, Singapore (SGX) |
| `SundayToMonday` | (no shift) | following Mon | Platts Asia, ICE Futures Europe |

Observance rules resolve a ±1-year window so cross-year shifts work correctly
(e.g. Saturday 1 Jan 2022 → observed Friday 31 Dec 2021).

## Built-in market calendars

`builtin::all_calendars()` returns all twenty-two, `builtin::calendar(name)` looks one up
by name (ignoring case), and `builtin::all_rules()` returns the 88 distinct rules they
compose. Calendar names are a stable interface — configurations refer to them.

| Calendar | Time zone | Valid range | Notes |
|----------|-----------|-------------|-------|
| `TARGET` | Europe/Berlin | 2000–2099 | ECB; 6 holidays, no observance shifts |
| `EEXPowerDerivatives` | Europe/Berlin | 2000–2099 | EEX Power: New Year, Good Friday, Easter Monday, Labour Day, Christmas Eve/Day, Boxing Day, New Year's Eve; no observance shifts |
| `USSettlement` | America/New_York | 2000–2099 | 11 US federal holidays + observance |
| `UKSettlement` | Europe/London | 2000–2099 | Bank holidays, cascade observance, proclaimed one-offs |
| `PlattsEuropeanMarketScan` | Europe/London | 2000–2099 | `UKSettlement` + Platts 2018 year-end closure (Dec 24/31) |
| `ICEUK` | Europe/London | 2000–2099 | ICE Futures Europe, days with a published settlement record: New Year, Good Friday, Christmas (+ Christmas Sun→Mon) |
| `SGX` | Asia/Singapore | 2010–2027 | Singapore Exchange; fixed + observed + lunar/religious date lists |
| `BalticLPG` | Europe/London | 2000–2099 | UK bank holidays + Baltic Christmas closure (Dec 25–31) |
| `BalticUS` | Europe/London | 2000–2099 | Baltic base + US holidays (incl. day after Thanksgiving; MLK Day + Juneteenth from 2027) |
| `BalticAsia` | Europe/London | 2010–2027 | Singapore holidays (Sun→Mon-only) + Baltic Christmas closure; publishes through UK bank holidays |
| `BalticLPG1Hormuz` | Europe/London | 2000–2026 | BLPG1 route: `BalticLPG` + 2026 Hormuz-crisis reduced publication (Wed weekly, then Tue+Thu) |
| `PlattsAsiaMarketScan` | Asia/Singapore | 2010–2027 | Platts Asian commodities; Sun→Mon-only observance + election polling days |
| `PlattsUSMarketScan` | America/New_York | 2016–2027 | Platts US/Americas daily benchmarks; US pricing holidays + Good Friday, Juneteenth from 2022, July-4 bridge days |
| `PlattsUSGasDaily` | America/New_York | 2016–2027 | Platts North American gas (Gas Daily: Henry Hub and the US spot hubs); `PlattsUSMarketScan` + Veterans Day from 2022 + Columbus Day from 2024 |
| `OPISPetroChemWire` | America/New_York | 2025–2027 | OPIS US daily petchem reports; date-equivalent to `PlattsUSMarketScan` over the verified window |
| `ICISEuropeChemicals` | Europe/London | 2022–2026 | ICIS Europe daily chemicals (Aromatics family); UK working days + drifting year-end non-publishing weeks |
| `ICISChinaChemicals` | Asia/Shanghai | 2022–2027 | ICIS China-CN daily chemicals; Chinese gazette holidays + explicit "Weekend" holidays minus makeup working weekends |
| `ChinaFuturesExchanges` | Asia/Shanghai | 2022–2027 | ZCE/DCE/SHFE/CFFEX/INE day-session closures (CNY FX-fixing days match); gazette spans, closed on all weekends incl. makeup working weekends |
| `ArgusAsiaLPG` | Asia/Singapore | 2010–2027 | Argus International LPG (Asia); date-equivalent to `PlattsAsiaMarketScan` incl. 2018 year-end closure |
| `ArgusEuropeLPG` | Europe/London | 2000–2099 | Argus International LPG (Europe); `UKSettlement` + 2018 year-end closure |
| `ICEUKAveraging` | Europe/London | 2000–2099 | ICE Futures Europe fixing days for monthly averages; `ICEUK` + the Monday after a Sunday New Year (prices only carried over) |
| `NYMEX` | America/New_York | 2021–2099 | NYMEX (CME Group) energy settlement days — WTI (CL), Henry Hub (NG/HH); ten US holidays, New Year Sun→Mon only, Juneteenth from 2022 |
| `ICEEndex` | Europe/Amsterdam | 2018–2099 | ICE Endex gas futures settlement days — TTF, THE, PSV, CEGH VTP, PEG; closed New Year, Good Friday, Easter Monday, 25 and 26 December, no weekend observance |

> This table is illustrative; the calendar set evolves. Run `fincal calendars` (or
> call `builtin::all_calendars()`) for the authoritative, up-to-date list.

The non-obvious market behaviours are encoded faithfully: the Baltic Exchange closes
the whole of Dec 25–31; ICE trades through UK bank holidays and does not shift
Saturday holidays to Friday; Platts Asia gives no substitute Monday when a holiday
falls on a Saturday; Platts Gas Daily observes the "US gas and power pricing holidays"
(Veterans Day from 2022, Columbus Day from 2024) that the oil and petrochemical
MarketScans trade through; NYMEX settles on the Friday before a Saturday New Year, on
Columbus Day, Veterans Day and the day after Thanksgiving, and took up Juneteenth only
in 2022; and the UK one-off bank holidays (jubilees, the 2011 royal
wedding, the 2022 state funeral, the 2023 coronation) and the 2002/2012/2020/2022 moved bank
holidays are all included.

## Calendar comparison and explanation

The `compare` module compares two `HolidayCalendar`s day-by-day over a range,
classifying each day into one of four buckets (business in both, non-business in both,
or business in only one). Two wrappers turn a slice of dates into a `BespokeCalendar`
first:

- `compare_dates` — input dates are **business days**.
- `compare_holidays` — input dates are **holidays**.

`CalendarComparison` reports the discrepancies and a `match_rate()`. To identify a
mystery time series, `explain_dates` / `explain_holidays` rank multiple candidate
calendars by how well they fit. Candidates whose valid range does not cover the
requested range are left out of the ranking (`HolidayCalendar::covers` tells which),
so one narrow calendar never voids the ranking of the others:

```rust
use fincal::{builtin, explain_holidays, HolidayCalendar};
use jiff::civil::date;

let cals = builtin::all_calendars();
let candidates: Vec<&dyn HolidayCalendar> =
    cals.iter().map(|c| c as &dyn HolidayCalendar).collect();

// Which calendar best explains these observed holidays?
let holidays = vec![date(2025, 1, 1), date(2025, 4, 18), date(2025, 12, 25)];
let fits = explain_holidays(&holidays, date(2025, 1, 1), date(2025, 12, 31), &candidates).unwrap();
println!("best fit: {} ({:.0}%)", fits[0].calendar.name(), fits[0].comparison.match_rate() * 100.0);
```

## Rule fitting

`fit_rules` works at the rule level. Given observed business days, it derives the
implied holidays (weekday gaps), scores each candidate rule by **hits** (fires on a
gap) versus **false positives** (fires where data exists), and greedily assembles the
"perfect-fit" rules (hits, no false positives) into an explanation. The
`RuleFitResult` reports the per-rule fits, the explained/unexplained holiday sets, and
an overall `match_rate`. This is how you discover *which holiday rules* a provider's
publication schedule follows, and surface holidays no known rule explains.

## CLI

The `fincal` binary wraps the library:

```bash
# List all built-in calendars (time zone, rule count, valid range)
fincal calendars

# Show a calendar's holidays for a year, list, or range
fincal holidays USSettlement --years 2025
fincal holidays UKSettlement --years 2020-2025
fincal holidays SGX --years 2024,2025

# Show which rules and calendars treat a date as a holiday
fincal explain 2022-09-19

# Days until each calendar's valid range ends; exit status 2 if any ends within 90 days
fincal horizon
fincal horizon --within 180

# Match piped dates against the built-in calendars (default: dates are holidays)
cat holidays.txt | fincal match
cat holidays.txt | fincal match --calendar TARGET      # compare against one calendar
cat bdays.txt    | fincal match --business-days --fit  # treat as business days, add rule fitting
```

`horizon` is the maintenance check: calendars built on maintained date lists (lunar
holidays, gazette spans, publisher schedules) end at the last maintained year, and
every query past it fails. Schedule `fincal horizon` (e.g. weekly) — it exits with
status 2 as soon as a calendar ends within `--within` days (default 90), which is the
signal to extend that calendar's date lists and `valid_range`.

Output piped into a reader that stops early (`fincal holidays UKSettlement --years
2000-2099 | head`) ends quietly with status 0.

`match` reads one `YYYY-MM-DD` date per line from stdin. With no `--calendar` it ranks
every calendar whose valid range covers the input (and lists the ones it had to leave
out); `--fit` adds rule-level analysis;
`--from` / `--to` override the inferred range; `--no-weekend` disables Sat/Sun
filtering during fitting.

## Build and test

`fincal` is a standalone crate with its own `Cargo.lock`; build it from its own
directory:

```bash
cargo build --release        # library + `fincal` CLI binary (default features)
cargo test                   # full test suite (incl. doctests)
cargo run -- calendars       # run the CLI
cargo check --no-default-features   # jiff-only library build
```

`just gate` is the done-gate: `cargo fmt --check`, `cargo clippy -D warnings` and
`cargo test` for both feature sets, `cargo doc` with warnings denied, and
`cargo audit`. `tests/invariants.rs` sweeps every built-in rule and calendar
over every day of its valid range (rule contract, predicate consistency, documented
calendar equivalences, closure-length sanity bound) — a new calendar is covered
without touching it.

## Design notes

Each built-in calendar's documentation states its sources and how it was
verified — against the publisher's or exchange's own schedules, government
gazettes, and observed settlement data. The publisher schedules themselves are
not redistributed here; consult the publishers for the originals.

Holiday dates are facts about when markets and publications close; the names
of exchanges, publishers and benchmarks are used only to identify which
schedule a calendar follows. fincal is not affiliated with or endorsed by any
of them.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
```
