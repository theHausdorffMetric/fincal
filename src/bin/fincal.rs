//! Financial calendar CLI — inspect built-in holiday calendars and match
//! observed dates against them.

use std::fmt;
use std::io::{self, BufRead, Write};

use clap::{Parser, Subcommand};
use jiff::civil::{Date, Weekday};

use fincal::builtin::{all_calendars, all_rules};
use fincal::rule_calendar::RuleBasedCalendar;
use fincal::{
    CalendarComparison, CalendarFit, HolidayCalendar, RangeBound, RuleFitResult, compare_dates,
    compare_holidays, explain_dates, explain_holidays, fit_rules,
};

/// Financial calendar inspector.
#[derive(Parser)]
#[command(name = "fincal", version, about, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all built-in calendars.
    Calendars,
    /// Show holidays for a named calendar.
    Holidays {
        /// Calendar name (case-insensitive).
        calendar: String,
        /// Years: single (2025), list (2020,2021), or range (2020-2025).
        /// Default: current year.
        #[arg(long)]
        years: Option<String>,
    },
    /// Show how long each built-in calendar remains valid.
    ///
    /// Calendars built on maintained date lists (lunar holidays, gazette
    /// spans, publisher schedules) end at the last maintained year; past it
    /// every query fails. Exits with status 2 if any calendar ends within
    /// `--within` days (or has ended), so the command can gate a scheduled
    /// job.
    Horizon {
        /// Warn about calendars ending within this many days.
        #[arg(long, default_value_t = 90)]
        within: i32,
        /// Reference date (default: today).
        #[arg(long)]
        today: Option<Date>,
    },
    /// Show which rules and calendars match a given date.
    Explain {
        /// Date in YYYY-MM-DD format.
        date: Date,
    },
    /// Match piped dates against built-in calendars and rules.
    ///
    /// By default, input dates are treated as **holidays**. Use `--business-days`
    /// to treat them as business days instead (the previous default).
    Match {
        /// Compare against a specific calendar (case-insensitive).
        #[arg(long)]
        calendar: Option<String>,
        /// Add rule-level fitting analysis.
        #[arg(long)]
        fit: bool,
        /// Disable Sat/Sun weekend filtering for rule fitting.
        #[arg(long)]
        no_weekend: bool,
        /// Treat input as business days instead of holidays.
        #[arg(long)]
        business_days: bool,
        /// Override range start (default: min of input dates).
        #[arg(long)]
        from: Option<Date>,
        /// Override range end (default: max of input dates).
        #[arg(long)]
        to: Option<Date>,
    },
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// What can go wrong in a subcommand.
enum CliError {
    /// Writing the output failed. A broken pipe (the reader went away, as
    /// in `fincal holidays … | head`) is not reported as an error.
    Io(io::Error),
    /// Anything else, already worded for the user.
    Message(String),
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        CliError::Message(message)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "cannot write output: {e}"),
            CliError::Message(message) => f.write_str(message),
        }
    }
}

/// Result of a subcommand; `?` converts I/O errors and plain messages.
type CliResult<T> = Result<T, CliError>;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the current calendar year.
fn current_year() -> i16 {
    jiff::ZonedDateTime::now().date().year()
}

/// Three-letter weekday abbreviation.
fn weekday_abbrev(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

/// Case-insensitive calendar lookup. Error lists available names.
fn resolve_calendar<'a>(
    name: &str,
    calendars: &'a [RuleBasedCalendar],
) -> Result<&'a RuleBasedCalendar, String> {
    calendars
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            let mut msg = format!("unknown calendar '{name}'\n\nAvailable calendars:");
            for c in calendars {
                msg.push_str(&format!("\n  {}", c.name()));
            }
            msg
        })
}

/// Parses a years spec: single (`2025`), comma-separated (`2020,2021,2025`),
/// or dash range (`2000-2020`).
fn parse_years(s: &str) -> Result<Vec<i16>, String> {
    let years = parse_year_spec(s)?;
    match years.iter().find(|y| Date::new(**y, 1, 1).is_err()) {
        Some(y) => Err(format!("year {y} is out of range (dates run to year 9999)")),
        None => Ok(years),
    }
}

/// The syntax half of [`parse_years`].
fn parse_year_spec(s: &str) -> Result<Vec<i16>, String> {
    if let Some((a, b)) = s.split_once('-') {
        let from: i16 = a
            .trim()
            .parse()
            .map_err(|_| format!("invalid year '{a}'"))?;
        let to: i16 = b
            .trim()
            .parse()
            .map_err(|_| format!("invalid year '{b}'"))?;
        if from > to {
            return Err(format!("invalid range: {from} > {to}"));
        }
        Ok((from..=to).collect())
    } else if s.contains(',') {
        s.split(',')
            .map(|p| {
                p.trim()
                    .parse::<i16>()
                    .map_err(|_| format!("invalid year '{}'", p.trim()))
            })
            .collect()
    } else {
        let y: i16 = s
            .trim()
            .parse()
            .map_err(|_| format!("invalid year '{s}'"))?;
        Ok(vec![y])
    }
}

/// Reads dates from stdin, one per line (YYYY-MM-DD). Skips blank lines.
fn read_dates_from_stdin() -> Result<Vec<Date>, String> {
    let stdin = io::stdin();
    let mut dates = Vec::new();
    for (i, line) in stdin.lock().lines().enumerate() {
        let line = line.map_err(|e| format!("line {}: read error: {e}", i + 1))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let date: Date = trimmed
            .parse()
            .map_err(|e| format!("line {}: invalid date '{trimmed}': {e}", i + 1))?;
        dates.push(date);
    }
    if dates.is_empty() {
        return Err("no dates provided on stdin".to_string());
    }
    Ok(dates)
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------

/// `fincal calendars` — list all built-in calendars.
fn cmd_calendars(out: &mut impl Write) -> CliResult<()> {
    let mut calendars = all_calendars();
    calendars.sort_by(|a, b| a.name().cmp(b.name()));
    writeln!(
        out,
        "{:<30} {:<20} {:>6}  {:<27} Description",
        "Calendar", "Time Zone", "Rules", "Valid Range"
    )?;
    writeln!(out, "{}", "-".repeat(110))?;
    for cal in &calendars {
        let (from, to) = cal.valid_range();
        let range = match (from, to) {
            (RangeBound::Date(f), RangeBound::Date(t)) => format!("{f} .. {t}"),
            (RangeBound::Date(f), RangeBound::Unbounded) => format!("{f} .."),
            (RangeBound::Unbounded, RangeBound::Date(t)) => format!(".. {t}"),
            (RangeBound::Unbounded, RangeBound::Unbounded) => "unbounded".to_string(),
        };
        writeln!(
            out,
            "{:<30} {:<20} {:>6}  {:<27} {}",
            cal.name(),
            cal.tz().iana_name().unwrap_or("UTC"),
            cal.rules().len(),
            range,
            cal.description().unwrap_or(""),
        )?;
    }
    Ok(())
}

/// `fincal horizon` — days until each built-in calendar's valid range ends.
///
/// Returns `true` if any calendar ends within `within` days of `today` (or
/// has already ended).
fn cmd_horizon(out: &mut impl Write, within: i32, today: Date) -> CliResult<bool> {
    let calendars = all_calendars();
    // (calendar name, last valid date, days left) — bounded calendars only.
    let mut rows: Vec<(&str, Date, i32)> = calendars
        .iter()
        .filter_map(|cal| {
            let end = cal.valid_range().1.date()?;
            let days = end.since(today).expect("civil date span").get_days();
            Some((cal.name(), end, days))
        })
        .collect();
    rows.sort_by_key(|&(name, end, _)| (end, name));

    writeln!(
        out,
        "Reference date: {today}, warning threshold: {within} days"
    )?;
    writeln!(out)?;
    writeln!(
        out,
        "{:<30} {:<12} {:>9}",
        "Calendar", "Valid to", "Days left"
    )?;
    writeln!(out, "{}", "-".repeat(60))?;
    let mut due = false;
    for &(name, end, days) in &rows {
        let flag = if days < 0 {
            "  EXPIRED"
        } else if days <= within {
            "  ENDS SOON"
        } else {
            ""
        };
        due |= days <= within;
        writeln!(out, "{name:<30} {end:<12} {days:>9}{flag}")?;
    }
    let unbounded = calendars.len() - rows.len();
    if unbounded > 0 {
        writeln!(out)?;
        writeln!(
            out,
            "{unbounded} calendar(s) without an upper bound not listed."
        )?;
    }
    Ok(due)
}

/// `fincal holidays <CALENDAR> [--years SPEC]` — show holidays for a calendar.
fn cmd_holidays(out: &mut impl Write, name: &str, years: &[i16]) -> CliResult<()> {
    let calendars = all_calendars();
    let cal = resolve_calendar(name, &calendars)?;
    let multi = years.len() > 1;

    for (i, &year) in years.iter().enumerate() {
        if multi {
            if i > 0 {
                writeln!(out)?;
            }
            writeln!(out, "{year}:")?;
        }
        let holidays = cal.holidays_in_year(year).map_err(|e| e.to_string())?;
        for d in &holidays {
            let name = cal.holiday_name(*d).ok().flatten().unwrap_or("?");
            writeln!(out, "{d}  {name}")?;
        }
    }
    Ok(())
}

/// `fincal explain <DATE>` — show which rules and calendars match a date.
fn cmd_explain(out: &mut impl Write, date: Date) -> CliResult<()> {
    let rules = all_rules();
    let matching: Vec<&str> = rules
        .iter()
        .filter(|r| r.holidays_in(date.year()).contains(&date))
        .map(|r| r.name())
        .collect();

    if matching.is_empty() {
        writeln!(out, "Rules:     (none)")?;
    } else {
        writeln!(out, "Rules:")?;
        for name in &matching {
            writeln!(out, "  {name}")?;
        }
    }

    let calendars = all_calendars();
    let cal_matches: Vec<&str> = calendars
        .iter()
        .filter(|c| c.is_holiday(date).unwrap_or(false))
        .map(|c| c.name())
        .collect();

    writeln!(out)?;
    if cal_matches.is_empty() {
        writeln!(out, "Calendars: (none)")?;
    } else {
        writeln!(out, "Calendars:")?;
        for name in &cal_matches {
            writeln!(out, "  {name}")?;
        }
    }
    Ok(())
}

/// `fincal match` — match piped dates against built-in calendars and rules.
fn cmd_match(
    out: &mut impl Write,
    calendar_name: Option<&str>,
    fit: bool,
    no_weekend: bool,
    business_days: bool,
    from_override: Option<Date>,
    to_override: Option<Date>,
) -> CliResult<()> {
    let dates = read_dates_from_stdin()?;
    let from = from_override.unwrap_or_else(|| *dates.iter().min().unwrap());
    let to = to_override.unwrap_or_else(|| *dates.iter().max().unwrap());

    let mode = if business_days {
        "business days"
    } else {
        "holidays"
    };
    writeln!(
        out,
        "Input: {} {mode}, range {} .. {}",
        dates.len(),
        from,
        to
    )?;

    let calendars = all_calendars();

    if let Some(name) = calendar_name {
        let cal = resolve_calendar(name, &calendars)?;
        let cmp = if business_days {
            compare_dates(&dates, cal, from, to)
        } else {
            compare_holidays(&dates, cal, from, to)
        }
        .map_err(|e| e.to_string())?;
        print_comparison(out, cal, &cmp, business_days)?;
    } else {
        // The ranking leaves out calendars that do not cover the range.
        let candidates: Vec<&dyn HolidayCalendar> = calendars
            .iter()
            .map(|c| c as &dyn HolidayCalendar)
            .collect();
        let fits = if business_days {
            explain_dates(&dates, from, to, &candidates)
        } else {
            explain_holidays(&dates, from, to, &candidates)
        }
        .map_err(|e| e.to_string())?;
        if fits.is_empty() {
            writeln!(out, "\nNo calendars cover the full date range.")?;
        } else {
            print_ranking(out, &fits, business_days)?;
        }
        let skipped: Vec<&str> = calendars
            .iter()
            .filter(|c| !c.covers(from, to))
            .map(|c| c.name())
            .collect();
        if !skipped.is_empty() {
            writeln!(out)?;
            writeln!(
                out,
                "Not ranked (valid range does not cover {from} .. {to}): {}",
                skipped.join(", ")
            )?;
        }
    }

    if fit {
        writeln!(out)?;
        let weekends = if no_weekend {
            vec![]
        } else {
            vec![Weekday::Saturday, Weekday::Sunday]
        };
        let rules = all_rules();
        let result = fit_rules(&dates, from, to, &rules, &weekends);
        print_fit_result(out, &result, &weekends)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Output formatting
// ---------------------------------------------------------------------------

/// Prints the calendar ranking table (best match first).
///
/// In business-day mode: Extra = `a_only`, Missing = `b_only`.
/// In holiday mode: Extra = `b_only`, Missing = `a_only` (semantics are inverted).
fn print_ranking(
    out: &mut impl Write,
    fits: &[CalendarFit],
    business_days: bool,
) -> io::Result<()> {
    writeln!(out)?;
    writeln!(
        out,
        "{:<30} {:>8}  {:>6}  {:>7}",
        "Calendar", "Match", "Extra", "Missing"
    )?;
    writeln!(out, "{}", "-".repeat(55))?;
    for fit in fits {
        let (extra, missing) = if business_days {
            (
                fit.comparison.a_only_bdays.len(),
                fit.comparison.b_only_bdays.len(),
            )
        } else {
            (
                fit.comparison.b_only_bdays.len(),
                fit.comparison.a_only_bdays.len(),
            )
        };
        writeln!(
            out,
            "{:<30} {:>7.1}%  {:>6}  {:>7}",
            fit.calendar.name(),
            fit.comparison.match_rate() * 100.0,
            extra,
            missing,
        )?;
    }
    Ok(())
}

/// Prints detailed comparison of input dates vs a specific calendar.
///
/// In business-day mode: Extra = `a_only` (input bdays not in calendar),
/// Missing = `b_only` (calendar bdays not in input).
/// In holiday mode: Extra = `b_only` (input holidays not in calendar),
/// Missing = `a_only` (calendar holidays not in input).
fn print_comparison(
    out: &mut impl Write,
    cal: &dyn HolidayCalendar,
    cmp: &CalendarComparison,
    business_days: bool,
) -> io::Result<()> {
    writeln!(out, "Calendar: {}", cal.name())?;
    writeln!(out)?;
    writeln!(out, "Match rate: {:.1}%", cmp.match_rate() * 100.0)?;

    let (extra, missing) = if business_days {
        (&cmp.a_only_bdays, &cmp.b_only_bdays)
    } else {
        (&cmp.b_only_bdays, &cmp.a_only_bdays)
    };

    let (extra_label, missing_label) = if business_days {
        (
            format!(
                "Extra dates ({}) — in input but not business days per {}:",
                extra.len(),
                cal.name(),
            ),
            format!(
                "Missing dates ({}) — business days per {} absent from input:",
                missing.len(),
                cal.name(),
            ),
        )
    } else {
        (
            format!(
                "Extra holidays ({}) — in input but not holidays per {}:",
                extra.len(),
                cal.name(),
            ),
            format!(
                "Missing holidays ({}) — holidays per {} absent from input:",
                missing.len(),
                cal.name(),
            ),
        )
    };

    if !extra.is_empty() {
        writeln!(out)?;
        writeln!(out, "{extra_label}")?;
        for &d in extra {
            let wd = weekday_abbrev(d.weekday());
            let name = cal.holiday_name(d).ok().flatten().unwrap_or("");
            if name.is_empty() {
                writeln!(out, "  {d}  {wd}")?;
            } else {
                writeln!(out, "  {d}  {wd}  {name}")?;
            }
        }
    }

    if !missing.is_empty() {
        writeln!(out)?;
        writeln!(out, "{missing_label}")?;
        for &d in missing {
            let wd = weekday_abbrev(d.weekday());
            let name = cal.holiday_name(d).ok().flatten().unwrap_or("");
            if name.is_empty() {
                writeln!(out, "  {d}  {wd}")?;
            } else {
                writeln!(out, "  {d}  {wd}  {name}")?;
            }
        }
    }

    if cmp.is_identical() {
        writeln!(out)?;
        writeln!(out, "Perfect match.")?;
    }
    Ok(())
}

/// Prints rule fitting results.
fn print_fit_result(
    out: &mut impl Write,
    result: &RuleFitResult,
    weekends: &[Weekday],
) -> io::Result<()> {
    let weekend_str = if weekends.is_empty() {
        "none".to_string()
    } else {
        weekends
            .iter()
            .map(|w| weekday_abbrev(*w))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let total = result.explained.len() + result.unexplained.len();
    writeln!(out, "Rule fitting (weekends: {weekend_str}):")?;
    writeln!(
        out,
        "  Explained: {}/{total} holidays ({:.1}%)",
        result.explained.len(),
        result.match_rate * 100.0,
    )?;

    let perfect: Vec<_> = result.fits.iter().filter(|f| f.is_perfect()).collect();
    if !perfect.is_empty() {
        writeln!(out)?;
        writeln!(out, "  Perfect-fit rules:")?;
        for fit in &perfect {
            writeln!(
                out,
                "    {:<40} {:>3} hits",
                fit.rule.name(),
                fit.hits.len(),
            )?;
        }
    }

    let imperfect: Vec<_> = result.fits.iter().filter(|f| !f.is_perfect()).collect();
    if !imperfect.is_empty() {
        writeln!(out)?;
        writeln!(out, "  Imperfect rules:")?;
        for fit in &imperfect {
            writeln!(
                out,
                "    {:<40} {:>3} hits, {} false positives",
                fit.rule.name(),
                fit.hits.len(),
                fit.false_positives.len(),
            )?;
        }
    }

    if result.unexplained.is_empty() {
        writeln!(out)?;
        writeln!(out, "  Unexplained holidays: (none)")?;
    } else {
        writeln!(out)?;
        writeln!(
            out,
            "  Unexplained holidays ({}):",
            result.unexplained.len()
        )?;
        for &d in &result.unexplained {
            let wd = weekday_abbrev(d.weekday());
            writeln!(out, "    {d}  {wd}")?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Dispatches to the appropriate subcommand, writing its output to `out`,
/// and returns the exit status (0, or 2 when `horizon` finds a calendar
/// about to end).
fn run(cli: Cli, out: &mut impl Write) -> CliResult<i32> {
    match cli.command {
        Command::Horizon { within, today } => {
            let today = today.unwrap_or_else(|| jiff::ZonedDateTime::now().date());
            let due = cmd_horizon(out, within, today)?;
            return Ok(if due { 2 } else { 0 });
        }
        Command::Calendars => cmd_calendars(out)?,
        Command::Holidays { calendar, years } => {
            let yrs = match years {
                Some(s) => parse_years(&s)?,
                None => vec![current_year()],
            };
            cmd_holidays(out, &calendar, &yrs)?;
        }
        Command::Explain { date } => cmd_explain(out, date)?,
        Command::Match {
            calendar,
            fit,
            no_weekend,
            business_days,
            from,
            to,
        } => cmd_match(
            out,
            calendar.as_deref(),
            fit,
            no_weekend,
            business_days,
            from,
            to,
        )?,
    }
    Ok(0)
}

fn main() {
    let cli = Cli::parse();
    let mut out = io::BufWriter::new(io::stdout().lock());
    let result = run(cli, &mut out).and_then(|status| {
        out.flush()?;
        Ok(status)
    });
    match result {
        Ok(0) => {}
        Ok(status) => std::process::exit(status),
        // The reader went away (`fincal holidays … | head`): stop quietly.
        Err(CliError::Io(e)) if e.kind() == io::ErrorKind::BrokenPipe => {}
        Err(e) => {
            // Keep whatever was written before the failure.
            let _ = out.flush();
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
