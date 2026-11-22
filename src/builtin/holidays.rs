//! Individual holiday rule constructors.
//!
//! Each public function returns a single [`HolidayRule`] as an `Arc<dyn HolidayRule>`.
//! These are the atomic building blocks that
//! [`calendars`](super::calendars) compose into complete market calendars.
//!
//! Rules are organised into four groups:
//!
//! - **Shared** — holidays observed across multiple markets (New Year's Day,
//!   Good Friday, Easter Monday, Christmas Day)
//! - **US-specific** — US federal holidays and their observance variants
//! - **UK-specific** — UK bank holidays and their observance variants
//! - **Baltic Exchange** — Christmas closure (Dec 25–31)
//! - **Singapore-specific** — SGX holidays including fixed-date with
//!   Sun→Mon observance and lunar/religious date-list rules (2010–2027)
//!
//! All functions in this module are collected by [`super::all_rules`] to form
//! the global rule pool for rule-fitting analysis.

use std::sync::Arc;

use jiff::civil::Weekday;

use crate::rules::{
    DateList, EasterRelative, FixedDate, FixedDateRange, HolidayRule, NthWeekday, ObservancePolicy,
    ObservedFixedDate, ReducedWeekdaySchedule, UkBoxingDayObserved, UkChristmasObserved,
};

// ---------------------------------------------------------------------------
// Shared rules — used by multiple calendars
// ---------------------------------------------------------------------------

/// New Year's Day — January 1 (fixed).
///
/// Used by TARGET, US (base), and UK (base).
pub fn new_years_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("New Year's Day", 1, 1))
}

/// Good Friday — Easter Sunday minus 2 days.
///
/// Used by TARGET and UK.
pub fn good_friday() -> Arc<dyn HolidayRule> {
    Arc::new(EasterRelative::new("Good Friday", -2))
}

/// Easter Monday — Easter Sunday plus 1 day.
///
/// Used by TARGET and UK.
pub fn easter_monday() -> Arc<dyn HolidayRule> {
    Arc::new(EasterRelative::new("Easter Monday", 1))
}

/// Christmas Day — December 25 (fixed).
///
/// Used by TARGET, US (base), and UK (base).
pub fn christmas_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Christmas Day", 12, 25))
}

// ---------------------------------------------------------------------------
// TARGET-specific rules
// ---------------------------------------------------------------------------

/// Labour Day — May 1 (fixed, European).
///
/// Used by TARGET.
pub fn labour_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Labour Day", 5, 1))
}

/// St. Stephen's Day — December 26 (fixed).
///
/// Used by TARGET. Same date as Boxing Day but different name.
pub fn st_stephens_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("St. Stephen's Day", 12, 26))
}

/// Christmas Eve — December 24 (fixed).
///
/// Used by EEX Power Derivatives (full-day exchange closure).
pub fn christmas_eve() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Christmas Eve", 12, 24))
}

/// New Year's Eve — December 31 (fixed).
///
/// Used by EEX Power Derivatives (full-day exchange closure).
pub fn new_years_eve() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("New Year's Eve", 12, 31))
}

// ---------------------------------------------------------------------------
// US-specific rules
// ---------------------------------------------------------------------------

/// New Year's Day (US Observed) — US federal policy.
///
/// Saturday → preceding Friday, Sunday → following Monday.
pub fn us_new_years_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "New Year's Day (US Observed)",
        1,
        1,
        ObservancePolicy::SaturdayToFridaySundayToMonday,
    ))
}

/// Martin Luther King Jr. Day — 3rd Monday of January.
pub fn mlk_day() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new(
        "Martin Luther King Jr. Day",
        1,
        Weekday::Monday,
        3,
    ))
}

/// Presidents' Day — 3rd Monday of February.
pub fn presidents_day() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new("Presidents' Day", 2, Weekday::Monday, 3))
}

/// Memorial Day — last Monday of May.
pub fn memorial_day() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new("Memorial Day", 5, Weekday::Monday, -1))
}

/// Juneteenth — June 19 (fixed, from 2021).
pub fn juneteenth() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Juneteenth", 6, 19).with_from_year(2021))
}

/// Juneteenth (Observed) — US federal policy (from 2021).
///
/// Saturday → preceding Friday, Sunday → following Monday.
pub fn us_juneteenth_observed() -> Arc<dyn HolidayRule> {
    Arc::new(
        ObservedFixedDate::new(
            "Juneteenth (Observed)",
            6,
            19,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )
        .with_from_year(2021),
    )
}

/// Independence Day — July 4 (fixed).
pub fn independence_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Independence Day", 7, 4))
}

/// Independence Day (Observed) — US federal policy.
///
/// Saturday → preceding Friday, Sunday → following Monday.
pub fn us_independence_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Independence Day (Observed)",
        7,
        4,
        ObservancePolicy::SaturdayToFridaySundayToMonday,
    ))
}

/// Labor Day — 1st Monday of September (US spelling).
pub fn labor_day() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new("Labor Day", 9, Weekday::Monday, 1))
}

/// Columbus Day — 2nd Monday of October.
pub fn columbus_day() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new("Columbus Day", 10, Weekday::Monday, 2))
}

/// Veterans Day — November 11 (fixed).
pub fn veterans_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Veterans Day", 11, 11))
}

/// Veterans Day (Observed) — US federal policy.
///
/// Saturday → preceding Friday, Sunday → following Monday.
pub fn us_veterans_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Veterans Day (Observed)",
        11,
        11,
        ObservancePolicy::SaturdayToFridaySundayToMonday,
    ))
}

/// Thanksgiving — 4th Thursday of November.
pub fn thanksgiving() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new("Thanksgiving", 11, Weekday::Thursday, 4))
}

/// Day after Thanksgiving — 4th Friday of November.
///
/// Not a US federal holiday, but observed by the Baltic Exchange as a
/// non-publication day for US-reported tanker routes (BCTI TC14/TC18/TC21/TC24,
/// BDTI TD9/TD21/TD25-TD29). Confirmed in the 2026 non-publication dates PDF.
pub fn day_after_thanksgiving() -> Arc<dyn HolidayRule> {
    // 4th Thursday + 1 day — NOT the 4th Friday, which is a week early
    // whenever November starts on a Friday (2019, 2024).
    Arc::new(
        NthWeekday::new("Day after Thanksgiving", 11, Weekday::Thursday, 4).with_offset_days(1),
    )
}

/// Christmas Day (US Observed) — US federal policy.
///
/// Saturday → preceding Friday, Sunday → following Monday.
pub fn us_christmas_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Christmas Day (US Observed)",
        12,
        25,
        ObservancePolicy::SaturdayToFridaySundayToMonday,
    ))
}

// ---------------------------------------------------------------------------
// UK-specific rules
// ---------------------------------------------------------------------------

/// New Year's Day (UK Observed) — UK policy.
///
/// Saturday or Sunday → following Monday.
pub fn uk_new_years_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "New Year's Day (UK Observed)",
        1,
        1,
        ObservancePolicy::NextMonday,
    ))
}

/// Early May Bank Holiday — 1st Monday of May.
///
/// Excluded in 2020 because the government moved it from 4 May to 8 May
/// for the 75th anniversary of VE Day. See [`ve_day_2020`].
pub fn early_may_bank_holiday() -> Arc<dyn HolidayRule> {
    Arc::new(
        NthWeekday::new("Early May Bank Holiday", 5, Weekday::Monday, 1).with_exclude_year(2020),
    )
}

/// Spring Bank Holiday — last Monday of May.
///
/// Excluded in 2002 (Golden Jubilee), 2012 (Diamond Jubilee) and 2022
/// (Platinum Jubilee), when the government moved it from late May to early
/// June to create a four-day Jubilee weekend. See
/// [`spring_bank_holiday_2002`], [`spring_bank_holiday_2012`] and
/// [`spring_bank_holiday_2022`].
pub fn spring_bank_holiday() -> Arc<dyn HolidayRule> {
    Arc::new(
        NthWeekday::new("Spring Bank Holiday", 5, Weekday::Monday, -1)
            .with_exclude_year(2002)
            .with_exclude_year(2012)
            .with_exclude_year(2022),
    )
}

/// Summer Bank Holiday — last Monday of August.
pub fn summer_bank_holiday() -> Arc<dyn HolidayRule> {
    Arc::new(NthWeekday::new(
        "Summer Bank Holiday",
        8,
        Weekday::Monday,
        -1,
    ))
}

/// Boxing Day — December 26 (fixed).
///
/// Used by UK (base). Same date as St. Stephen's Day but different name.
pub fn boxing_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Boxing Day", 12, 26))
}

/// Christmas Day (Observed) — UK cascade substitute.
///
/// Saturday Dec 25 → Monday Dec 27. Sunday Dec 25 → Tuesday Dec 27.
pub fn uk_christmas_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(UkChristmasObserved)
}

/// Boxing Day (Observed) — UK cascade substitute.
///
/// Saturday Dec 26 → Monday Dec 28. Sunday Dec 26 → Tuesday Dec 28.
pub fn uk_boxing_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(UkBoxingDayObserved)
}

// ---------------------------------------------------------------------------
// UK one-off bank holidays (proclaimed by royal/government decree)
// ---------------------------------------------------------------------------

/// VE Day 75th Anniversary — Early May Bank Holiday moved to 8 May 2020.
///
/// The government moved the Early May Bank Holiday from its usual first-Monday
/// slot (4 May) to Friday 8 May to mark the 75th anniversary of VE Day.
/// See [`early_may_bank_holiday`] which excludes 2020.
pub fn ve_day_2020() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "VE Day 75th Anniversary",
        [jiff::civil::date(2020, 5, 8)],
    ))
}

/// Spring Bank Holiday 2002 (moved) — moved to 4 June for the Golden Jubilee.
///
/// The government moved the Spring Bank Holiday from its usual last-Monday-of-May
/// slot (27 May) to Tuesday 4 June, next to the Golden Jubilee extra bank
/// holiday on Monday 3 June, creating a four-day weekend.
/// See [`spring_bank_holiday`] which excludes 2002 and [`queens_golden_jubilee`]
/// for the additional 3 June bank holiday.
pub fn spring_bank_holiday_2002() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Spring Bank Holiday 2002 (moved)",
        [jiff::civil::date(2002, 6, 4)],
    ))
}

/// Spring Bank Holiday 2012 (moved) — moved to 4 June for the Diamond Jubilee.
///
/// The government moved the Spring Bank Holiday from its usual last-Monday-of-May
/// slot (28 May) to Monday 4 June, with 5 June as the Diamond Jubilee extra
/// bank holiday, creating a four-day weekend.
/// See [`spring_bank_holiday`] which excludes 2012 and [`queens_diamond_jubilee`]
/// for the additional 5 June bank holiday.
pub fn spring_bank_holiday_2012() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Spring Bank Holiday 2012 (moved)",
        [jiff::civil::date(2012, 6, 4)],
    ))
}

/// Spring Bank Holiday 2022 (moved) — moved to 2 June for the Platinum Jubilee.
///
/// The government moved the Spring Bank Holiday from its usual last-Monday-of-May
/// slot (30 May) to Thursday 2 June, with 3 June as the Platinum Jubilee extra
/// bank holiday, creating a four-day weekend.
/// See [`spring_bank_holiday`] which excludes 2022.
pub fn spring_bank_holiday_2022() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Spring Bank Holiday 2022 (moved)",
        [jiff::civil::date(2022, 6, 2)],
    ))
}

/// Queen's Golden Jubilee — extra bank holiday on 3 June 2002.
///
/// The Spring Bank Holiday moved next to it, see [`spring_bank_holiday_2002`].
pub fn queens_golden_jubilee() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Queen's Golden Jubilee",
        [jiff::civil::date(2002, 6, 3)],
    ))
}

/// Royal Wedding (William & Catherine) — extra bank holiday on 29 April 2011.
pub fn royal_wedding_2011() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Royal Wedding 2011",
        [jiff::civil::date(2011, 4, 29)],
    ))
}

/// Queen's Diamond Jubilee — extra bank holiday on 5 June 2012.
///
/// The Spring Bank Holiday was moved from its usual last-Monday-of-May
/// slot to 4 June 2012, with 5 June as an additional bank holiday,
/// creating a four-day Jubilee weekend. See [`spring_bank_holiday_2012`].
pub fn queens_diamond_jubilee() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Queen's Diamond Jubilee",
        [jiff::civil::date(2012, 6, 5)],
    ))
}

/// Queen's Platinum Jubilee — extra bank holiday on 3 June 2022.
///
/// The Spring Bank Holiday was moved from its usual last-Monday-of-May
/// slot to 2 June 2022, with 3 June as an additional bank holiday.
pub fn queens_platinum_jubilee() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Queen's Platinum Jubilee",
        [jiff::civil::date(2022, 6, 3)],
    ))
}

/// Queen Elizabeth II State Funeral — bank holiday on 19 September 2022.
pub fn queens_funeral() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "Queen Elizabeth II State Funeral",
        [jiff::civil::date(2022, 9, 19)],
    ))
}

/// King Charles III Coronation — extra bank holiday on 8 May 2023.
pub fn coronation_bank_holiday() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "King Charles III Coronation",
        [jiff::civil::date(2023, 5, 8)],
    ))
}

// ---------------------------------------------------------------------------
// Baltic Exchange rules
// ---------------------------------------------------------------------------

/// Baltic Exchange Christmas closure — December 25 through December 31 (inclusive).
///
/// The Baltic Exchange closes for the last week of December every year.
/// This single rule produces 7 holiday dates (Dec 25–31) per year.
pub fn baltic_christmas() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDateRange::new("Baltic Exchange Christmas", 12, 25, 31))
}

/// Martin Luther King Jr. Day for the US-reported Baltic tanker routes —
/// 3rd Monday of January, **from 2027 only**.
///
/// The Baltic Exchange "Non-publication dates 2027" schedule adds
/// "Martin Luther King Day 18 January" to the BCTI TC14/18/21/24, MRA
/// Triangulation & BDTI TD9/21/25–29/33 group; the 2025 and 2026 schedules
/// do not list it (verified exact-match without it). Used by
/// [`crate::builtin::calendars::baltic_us`].
pub fn baltic_us_mlk_day() -> Arc<dyn HolidayRule> {
    Arc::new(
        NthWeekday::new(
            "Martin Luther King Jr. Day (Baltic, from 2027)",
            1,
            Weekday::Monday,
            3,
        )
        .with_from_year(2027),
    )
}

/// Juneteenth for the US-reported Baltic tanker routes — June 19,
/// **from 2027 only** (see [`baltic_us_mlk_day`]; the 2027 schedule lists
/// "Juneteenth 18 June", i.e. the Friday observance of Sat 19 June).
pub fn baltic_us_juneteenth() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Juneteenth (Baltic, from 2027)", 6, 19).with_from_year(2027))
}

/// Juneteenth (Observed) for the US-reported Baltic tanker routes — US
/// federal policy (Sat → Fri, Sun → Mon), from 2027 only.
pub fn baltic_us_juneteenth_observed() -> Arc<dyn HolidayRule> {
    Arc::new(
        ObservedFixedDate::new(
            "Juneteenth (Baltic Observed, from 2027)",
            6,
            19,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )
        .with_from_year(2027),
    )
}

/// Juneteenth for Platts US publications — from 2022 only.
///
/// Juneteenth became a US federal holiday mid-June 2021, too late for the
/// 2021 market calendar: the Platts 2021 schedule has no June 19 entry (US
/// Marketscan published), while 2022+ schedules suspend US publications.
/// The federal [`juneteenth`] rule (from 2021) is therefore wrong for Platts.
pub fn platts_juneteenth() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Juneteenth (Platts, from 2022)", 6, 19).with_from_year(2022))
}

/// Juneteenth (Observed) for Platts US publications — from 2022 only.
///
/// Saturday → preceding Friday, Sunday → following Monday (e.g. Sun
/// 2022-06-19 → Mon 2022-06-20, per the Platts 2022 schedule).
pub fn platts_juneteenth_observed() -> Arc<dyn HolidayRule> {
    Arc::new(
        ObservedFixedDate::new(
            "Juneteenth (Platts Observed, from 2022)",
            6,
            19,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )
        .with_from_year(2022),
    )
}

/// Platts US July 4 bridge days.
///
/// When Independence Day falls on Tuesday, Platts also suspends US
/// publications on the Monday before; on Thursday, the Friday after
/// (holsked 2016–2026: 2017, 2019, 2023, 2024 — no counter-example).
/// Shared by [`super::calendars::platts_us_marketscan`] and
/// [`super::calendars::platts_us_gas_daily`]: the Gas rows of the same
/// workbooks suspend North American gas on exactly these four days, and the
/// Platts Henry Hub trade-date series has no value on any of them.
/// Maintained as a DateList: extend when a new schedule year has July 4 on
/// Tue/Thu (next: 2028-07-03 for Tue 2028-07-04, if the pattern holds).
pub fn platts_us_july4_bridge() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "Platts US July 4 bridge day",
            [
                date(2017, 7, 3), // Mon before Tue July 4
                date(2019, 7, 5), // Fri after Thu July 4
                date(2023, 7, 3), // Mon before Tue July 4
                date(2024, 7, 5), // Fri after Thu July 4
            ],
        )
        .with_valid_range(2016, 2027),
    )
}

/// Platts US one-off closure — Fri 26 Dec 2025.
///
/// The Platts 2025 schedule suspends US assessments on the Friday after
/// Christmas ("No markets assessed in US, Canada or Latin America"; the Gas
/// row: "No assessments for North American gas") — a one-off; no other year
/// closes the day after Christmas when it falls on a weekday that isn't
/// already an observance day. Shared by
/// [`super::calendars::platts_us_marketscan`] and
/// [`super::calendars::platts_us_gas_daily`].
pub fn platts_us_day_after_christmas_2025() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(DateList::new(
        "Platts US day after Christmas (2025)",
        [date(2025, 12, 26)],
    ))
}

/// Columbus Day for Platts North American gas publications — from **2024**.
///
/// The 2nd Monday of October is a "US gas and power pricing holiday" in the
/// Platts schedules ("No assessments for North American gas … Gas Daily …
/// will not be published"), but only from 2024: the 2016–2023 workbooks
/// carry no North American gas suspension that day (their rows are Canadian
/// Thanksgiving / CGPR Daily only), and the Platts Henry Hub trade-date
/// series has a value on every Columbus Day 2016–2023 and none on
/// 2024-10-14 or 2025-10-13. US Marketscan (oil) and US petrochemicals
/// publish on Columbus Day in every year.
pub fn platts_gas_columbus_day() -> Arc<dyn HolidayRule> {
    Arc::new(
        NthWeekday::new(
            "Columbus Day (Platts gas, from 2024)",
            10,
            Weekday::Monday,
            2,
        )
        .with_from_year(2024),
    )
}

/// Veterans Day for Platts North American gas publications — from **2022**.
///
/// November 11 is a "US gas pricing holiday" in the Platts schedules from
/// 2022 ("No assessments for North American gas"; Gas Daily appears with
/// the previous day's assessments or not at all). The 2016–2021 schedules
/// have no gas entry on Veterans Day and the Henry Hub trade-date series
/// has a value on each weekday instance. Oil and petrochemical publications
/// are unaffected in every year.
pub fn platts_gas_veterans_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Veterans Day (Platts gas, from 2022)", 11, 11).with_from_year(2022))
}

/// Veterans Day (Observed) for Platts North American gas publications —
/// from 2022, US federal policy.
///
/// Saturday → preceding Friday (Sat 2023-11-11 → Fri 2023-11-10, per the
/// Platts 2023 schedule and the Henry Hub data), Sunday → following Monday.
pub fn platts_gas_veterans_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(
        ObservedFixedDate::new(
            "Veterans Day (Platts gas Observed, from 2022)",
            11,
            11,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )
        .with_from_year(2022),
    )
}

/// Platts North American gas one-off closure — Fri 2 Jan 2026.
///
/// The Platts 2026 schedule lists Friday 2026-01-02 as a "US gas and power
/// pricing holiday" ("No assessments for North American gas … Gas Daily …
/// will not be published"); US Marketscan (oil) and petrochemicals publish
/// that day. With [`platts_us_day_after_christmas_2025`] this is the
/// Thursday-holiday → Friday-bridge pattern also seen on July 4
/// ([`platts_us_july4_bridge`]), but no earlier Thursday Christmas or New
/// Year falls inside the verified window, so it stays a one-off.
pub fn platts_gas_day_after_new_year_2026() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(DateList::new(
        "Platts US gas day after New Year's Day (2026)",
        [date(2026, 1, 2)],
    ))
}

/// ICIS Europe chemicals year-end closures (2022–2026).
///
/// ICIS suspends its Europe daily chemicals assessments (the Aromatics
/// publication family) for a year-end "non-publishing week" whose span
/// drifts year to year (per the official ICIS annual publishing
/// schedules): 2022: 23 Dec + 26 Dec–2 Jan; 2023:
/// 22 Dec + 25 Dec–1 Jan; 2024: 24 Dec–1 Jan; 2025: 24 Dec–2 Jan; 2026:
/// 25 Dec–1 Jan. This DateList carries only the weekday dates **not**
/// already covered by the UK bank-holiday base rules. Extend with each new
/// schedule year.
pub fn icis_europe_chemicals_year_end() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "ICIS Europe chemicals year-end closure",
            [
                // 2022: 23 Dec + 26 Dec – 2 Jan 2023 (26/27 Dec + 2 Jan are UK holidays)
                date(2022, 12, 23),
                date(2022, 12, 28),
                date(2022, 12, 29),
                date(2022, 12, 30),
                // 2023: 22 Dec + 25 Dec – 1 Jan 2024
                date(2023, 12, 22),
                date(2023, 12, 27),
                date(2023, 12, 28),
                date(2023, 12, 29),
                // 2024: 24 Dec – 1 Jan 2025
                date(2024, 12, 24),
                date(2024, 12, 27),
                date(2024, 12, 30),
                date(2024, 12, 31),
                // 2025: 24 Dec – 2 Jan 2026
                date(2025, 12, 24),
                date(2025, 12, 29),
                date(2025, 12, 30),
                date(2025, 12, 31),
                date(2026, 1, 2),
                // 2026: 25 Dec – 1 Jan 2027 (25/28 Dec + 1 Jan are UK holidays)
                date(2026, 12, 29),
                date(2026, 12, 30),
                date(2026, 12, 31),
            ],
        )
        .with_valid_range(2022, 2026),
    )
}

// ---------------------------------------------------------------------------
// ICIS China closures (China-CN daily reports, per the ICIS annual
// publishing schedules 2022–2026 — weekday dates only; the multi-day spans
// follow the Chinese State Council gazette incl. makeup-extended weeks).
// 2027 is gazette-only so far (国办发明电〔2026〕10号, 2026-11-05); the
// ICIS 2027 publishing schedule was not yet available when added.
// ---------------------------------------------------------------------------

/// ICIS China — explicit weekend holidays, minus Chinese makeup working weekends.
///
/// The Chinese State Council gazette designates some weekend days as makeup
/// working days (调休) around major holidays; most ICIS China-CN dailies
/// publish on those. Weekends are therefore modelled explicitly (rule name
/// `"Weekend"`, the name vendor holiday lists commonly use), with the
/// makeup days excepted. Sources: the State Council gazette announcements
/// 2022–2027 (2026: 国办发明电〔2025〕7号, 2025-11-04,
/// <https://www.gov.cn/zhengce/zhengceku/202511/content_7047091.htm>;
/// 2027: 国办发明电〔2026〕10号, 2026-11-05, Xinhua authorised release
/// <https://www.news.cn/politics/20261105/f90cd6ad4df94ab4833b141e52ccfc13/c.html>);
/// the past 2026 dates are additionally data-verified against the toluene
/// East China series (2026-07-14: values on 01-04/02-14/02-28/05-09).
/// The 2027 makeup days are gazette-inferred only — re-check against the
/// ICIS 2027 publishing schedule once it is released.
pub fn icis_cn_weekend() -> Arc<dyn HolidayRule> {
    use crate::rules::WeekendsExcept;
    use jiff::civil::date;
    Arc::new(WeekendsExcept::new(
        "Weekend",
        [
            // 2022: CNY, Qingming, Labour, Golden Week makeups
            date(2022, 1, 29),
            date(2022, 1, 30),
            date(2022, 4, 2),
            date(2022, 4, 24),
            date(2022, 5, 7),
            date(2022, 10, 8),
            date(2022, 10, 9),
            // 2023
            date(2023, 1, 28),
            date(2023, 1, 29),
            date(2023, 4, 23),
            date(2023, 5, 6),
            date(2023, 6, 25),
            date(2023, 10, 7),
            date(2023, 10, 8),
            // 2024
            date(2024, 2, 4),
            date(2024, 2, 18),
            date(2024, 4, 7),
            date(2024, 4, 28),
            date(2024, 5, 11),
            date(2024, 9, 14),
            date(2024, 9, 29),
            date(2024, 10, 12),
            // 2025
            date(2025, 1, 26),
            date(2025, 2, 8),
            date(2025, 4, 27),
            date(2025, 9, 28),
            date(2025, 10, 11),
            // 2026 — gazette 国办发明电〔2025〕7号 (2025-11-04): New Year
            // 01-04; Spring Festival 02-14 + 02-28; Labour Day 05-09;
            // National Day 09-20 + 10-10. Past dates also data-verified vs
            // the toluene East China series (2026-07-14). The previously
            // inferred 04-26 and 09-27 are NOT makeup days (09-27 is inside
            // the Mid-Autumn closure 09-25..09-27).
            date(2026, 1, 4),
            date(2026, 2, 14),
            date(2026, 2, 28),
            date(2026, 5, 9),
            date(2026, 9, 20),
            date(2026, 10, 10),
            // 2027 — gazette 国办发明电〔2026〕10号 (2026-11-05): Spring
            // Festival 01-31 (Sun) + 02-14 (Sun); Labour Day 05-08 (Sat);
            // National Day 09-26 (Sun) + 10-09 (Sat). No makeup day for
            // New Year (01-01..03 Fri–Sun) or Qingming (04-03..05 Sat–Mon).
            date(2027, 1, 31),
            date(2027, 2, 14),
            date(2027, 5, 8),
            date(2027, 9, 26),
            date(2027, 10, 9),
        ],
    ))
}

/// China futures/stock exchanges — CNY-eve closures beyond the gazette.
///
/// In 2024 the gazette holiday ran 02-10 → 02-17 (eve excluded), but the
/// futures exchanges (SHFE/ZCE/DCE/GFE) and stock exchanges announced
/// closure for the eve as well: closed Fri 2024-02-09, reopening Mon
/// 2024-02-19. In every other covered year (2022, 2023, 2025, 2026, 2027)
/// the eve falls inside the gazette span, so no extra date is needed. Used by
/// `ChinaFuturesExchanges` only — ICIS China-CN dailies published on
/// 2024-02-09.
pub fn china_exchanges_cny_eve() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new("China Exchanges CNY Eve Closure", [date(2024, 2, 9)])
            .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — New Year closures (incl. observed Mondays and the 2026 two-day span; 2027: 1–3 Jan Fri–Sun).
pub fn china_new_year() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China New Year",
            [
                date(2022, 1, 3),
                date(2023, 1, 2),
                date(2024, 1, 1),
                date(2025, 1, 1),
                date(2026, 1, 1),
                date(2026, 1, 2),
                date(2027, 1, 1),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — Lunar New Year closure weeks.
pub fn china_lunar_new_year() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China Lunar New Year",
            [
                // 2022: 31 Jan – 4 Feb
                date(2022, 1, 31),
                date(2022, 2, 1),
                date(2022, 2, 2),
                date(2022, 2, 3),
                date(2022, 2, 4),
                // 2023: 23–27 Jan
                date(2023, 1, 23),
                date(2023, 1, 24),
                date(2023, 1, 25),
                date(2023, 1, 26),
                date(2023, 1, 27),
                // 2024: 12–16 Feb
                date(2024, 2, 12),
                date(2024, 2, 13),
                date(2024, 2, 14),
                date(2024, 2, 15),
                date(2024, 2, 16),
                // 2025: 28–31 Jan + 3–4 Feb
                date(2025, 1, 28),
                date(2025, 1, 29),
                date(2025, 1, 30),
                date(2025, 1, 31),
                date(2025, 2, 3),
                date(2025, 2, 4),
                // 2026: 16–23 Feb (weekdays)
                date(2026, 2, 16),
                date(2026, 2, 17),
                date(2026, 2, 18),
                date(2026, 2, 19),
                date(2026, 2, 20),
                date(2026, 2, 23),
                // 2027: 5–13 Feb (weekdays; 5 Feb is the eve)
                date(2027, 2, 5),
                date(2027, 2, 8),
                date(2027, 2, 9),
                date(2027, 2, 10),
                date(2027, 2, 11),
                date(2027, 2, 12),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — Qing Ming Festival closures.
pub fn china_qingming() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China Qing Ming Festival",
            [
                date(2022, 4, 4),
                date(2022, 4, 5),
                date(2023, 4, 5),
                date(2024, 4, 4),
                date(2024, 4, 5),
                date(2025, 4, 4),
                date(2026, 4, 6),
                date(2027, 4, 5),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — Labour Day closure spans.
pub fn china_labour_day() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China Labour Day",
            [
                date(2022, 5, 2),
                date(2022, 5, 3),
                date(2022, 5, 4),
                date(2023, 5, 1),
                date(2023, 5, 2),
                date(2023, 5, 3),
                date(2024, 5, 1),
                date(2024, 5, 2),
                date(2024, 5, 3),
                date(2025, 5, 1),
                date(2025, 5, 2),
                date(2025, 5, 5),
                date(2026, 5, 1),
                date(2026, 5, 4),
                date(2026, 5, 5),
                // 2027: 1–5 May (weekdays)
                date(2027, 5, 3),
                date(2027, 5, 4),
                date(2027, 5, 5),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — Dragon Boat Festival closures.
pub fn china_dragon_boat() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China Dragon Boat Festival",
            [
                date(2022, 6, 3),
                date(2023, 6, 22),
                date(2023, 6, 23),
                date(2024, 6, 10),
                date(2025, 6, 2),
                date(2026, 6, 19),
                date(2027, 6, 9),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — Mid-Autumn Festival closures (2025 merged into Golden Week).
pub fn china_mid_autumn() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China Mid-Autumn Festival",
            [
                date(2022, 9, 12),
                date(2023, 9, 29),
                date(2024, 9, 16),
                date(2024, 9, 17),
                date(2026, 9, 25),
                date(2027, 9, 15),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// Chinese State Council gazette — National Day (Golden Week) closure spans.
pub fn china_golden_week() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "China National Day (Golden Week)",
            [
                // 2022: 3–7 Oct
                date(2022, 10, 3),
                date(2022, 10, 4),
                date(2022, 10, 5),
                date(2022, 10, 6),
                date(2022, 10, 7),
                // 2023: 2–6 Oct
                date(2023, 10, 2),
                date(2023, 10, 3),
                date(2023, 10, 4),
                date(2023, 10, 5),
                date(2023, 10, 6),
                // 2024: 1–7 Oct (weekdays)
                date(2024, 10, 1),
                date(2024, 10, 2),
                date(2024, 10, 3),
                date(2024, 10, 4),
                date(2024, 10, 7),
                // 2025: 1–8 Oct (weekdays; incl. Mid-Autumn)
                date(2025, 10, 1),
                date(2025, 10, 2),
                date(2025, 10, 3),
                date(2025, 10, 6),
                date(2025, 10, 7),
                date(2025, 10, 8),
                // 2026: 1–7 Oct (weekdays)
                date(2026, 10, 1),
                date(2026, 10, 2),
                date(2026, 10, 5),
                date(2026, 10, 6),
                date(2026, 10, 7),
                // 2027: 1–7 Oct (weekdays)
                date(2027, 10, 1),
                date(2027, 10, 4),
                date(2027, 10, 5),
                date(2027, 10, 6),
                date(2027, 10, 7),
            ],
        )
        .with_valid_range(2022, 2027),
    )
}

/// OPIS US petrochemical one-off closure — Fri 26 Dec 2025.
///
/// The OPIS 2025 holiday publish schedule suspends all US daily
/// petrochemical reports on the Friday after Christmas ("No Report or
/// Data") — matching the Platts one-off
/// [`platts_us_day_after_christmas_2025`]; evidently an industry-wide US
/// petchem closure that year. Separate rule for OPIS provenance.
pub fn opis_day_after_christmas_2025() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(DateList::new(
        "OPIS day after Christmas (2025)",
        [date(2025, 12, 26)],
    ))
}

/// Argus year-end closure 2018 — Mon 24 Dec and Mon 31 Dec 2018.
///
/// Like Platts (see [`platts_year_end_2018`]), Argus took the two Mondays
/// before the Tuesday holidays off in 2018. Evidence: the Argus Propane
/// CIF ARA large cargo settlement series (history 2007–2026) has values on
/// every other `ArgusEuropeLPG` business day — including all other Dec
/// 24/31 half-days — but none on these two dates; an Argus Far East
/// (Singapore) holiday list independently carries both.
/// Kept separate from the Platts rule for provenance, not behaviour.
pub fn argus_year_end_2018() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(DateList::new(
        "Argus year-end closure (2018)",
        [date(2018, 12, 24), date(2018, 12, 31)],
    ))
}

/// BLPG1 reduced publication, phase 1 — weekly (Wednesdays only),
/// 2026-03-04 to 2026-04-06.
///
/// Baltic Exchange Circular 13/26 (2026 Strait of Hormuz crisis): BLPG1
/// moved from daily to weekly publication, Wednesdays 16:00 UK.
pub fn blpg1_hormuz_weekly() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(ReducedWeekdaySchedule::new(
        "BLPG1 Hormuz weekly publication (Wed only)",
        date(2026, 3, 4),
        Some(date(2026, 4, 6)),
        [Weekday::Wednesday],
    ))
}

/// BLPG1 reduced publication, phase 2 — Tuesdays + Thursdays, from
/// 2026-04-07 until further notice.
///
/// Baltic Exchange Circular 16/26. Open-ended: the owning calendar bounds
/// the horizon via its `valid_range`.
pub fn blpg1_hormuz_tue_thu() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(ReducedWeekdaySchedule::new(
        "BLPG1 Hormuz Tue/Thu publication",
        date(2026, 4, 7),
        None,
        [Weekday::Tuesday, Weekday::Thursday],
    ))
}

// ---------------------------------------------------------------------------
// Singapore-specific rules
// ---------------------------------------------------------------------------

/// Singapore National Day — August 9 (fixed).
pub fn sg_national_day() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Singapore National Day", 8, 9))
}

/// Singapore National Day (Observed) — Sunday → following Monday.
pub fn sg_national_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Singapore National Day (Observed)",
        8,
        9,
        ObservancePolicy::NextMonday,
    ))
}

/// New Year's Day (SG Observed) — Sunday → following Monday.
pub fn sg_new_years_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "New Year's Day (SG Observed)",
        1,
        1,
        ObservancePolicy::NextMonday,
    ))
}

/// Labour Day (SG Observed) — Sunday → following Monday.
pub fn sg_labour_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Labour Day (SG Observed)",
        5,
        1,
        ObservancePolicy::NextMonday,
    ))
}

/// Christmas Day (SG Observed) — Sunday → following Monday.
pub fn sg_christmas_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Christmas Day (SG Observed)",
        12,
        25,
        ObservancePolicy::NextMonday,
    ))
}

/// Chinese New Year — two-day holiday, with substitute Monday when a day
/// falls on Sunday (2010–2027).
pub fn sg_chinese_new_year() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "Chinese New Year",
            [
                // 2010: Feb 14 (Sun), 15 (Mon), 16 (Tue sub for Sun)
                date(2010, 2, 14),
                date(2010, 2, 15),
                date(2010, 2, 16),
                // 2011
                date(2011, 2, 3),
                date(2011, 2, 4),
                // 2012
                date(2012, 1, 23),
                date(2012, 1, 24),
                // 2013: Feb 10 (Sun), 11 (Mon), 12 (Tue sub)
                date(2013, 2, 10),
                date(2013, 2, 11),
                date(2013, 2, 12),
                // 2014
                date(2014, 1, 31),
                date(2014, 2, 1),
                // 2015
                date(2015, 2, 19),
                date(2015, 2, 20),
                // 2016
                date(2016, 2, 8),
                date(2016, 2, 9),
                // 2017: Jan 28 (Sat), 29 (Sun), 30 (Mon sub)
                date(2017, 1, 28),
                date(2017, 1, 29),
                date(2017, 1, 30),
                // 2018
                date(2018, 2, 16),
                date(2018, 2, 17),
                // 2019
                date(2019, 2, 5),
                date(2019, 2, 6),
                // 2020: Jan 25 (Sat), 26 (Sun), 27 (Mon sub)
                date(2020, 1, 25),
                date(2020, 1, 26),
                date(2020, 1, 27),
                // 2021
                date(2021, 2, 12),
                date(2021, 2, 13),
                // 2022
                date(2022, 2, 1),
                date(2022, 2, 2),
                // 2023: Jan 22 (Sun), 23 (Mon), 24 (Tue sub)
                date(2023, 1, 22),
                date(2023, 1, 23),
                date(2023, 1, 24),
                // 2024: Feb 10 (Sat), 11 (Sun), 12 (Mon sub)
                date(2024, 2, 10),
                date(2024, 2, 11),
                date(2024, 2, 12),
                // 2025
                date(2025, 1, 29),
                date(2025, 1, 30),
                // 2026
                date(2026, 2, 17),
                date(2026, 2, 18),
                // 2027: Feb 6 (Sat), 7 (Sun), 8 (Mon sub) — MOM press release
                // "Public Holidays for 2027" (18 Jun 2026)
                date(2027, 2, 6),
                date(2027, 2, 7),
                date(2027, 2, 8),
            ],
        )
        .with_valid_range(2010, 2027),
    )
}

/// Hari Raya Puasa (Eid al-Fitr) — with substitute Monday when the date
/// falls on Sunday (2010–2027). 2027 confirmed by the MOM press release
/// "Public Holidays for 2027" (18 Jun 2026).
pub fn sg_hari_raya_puasa() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "Hari Raya Puasa",
            [
                date(2010, 9, 10),
                date(2011, 8, 30),
                // 2012: Aug 19 (Sun), 20 (Mon sub)
                date(2012, 8, 19),
                date(2012, 8, 20),
                date(2013, 8, 8),
                date(2014, 7, 28),
                date(2015, 7, 17),
                date(2016, 7, 6),
                // 2017: Jun 25 (Sun), 26 (Mon sub)
                date(2017, 6, 25),
                date(2017, 6, 26),
                date(2018, 6, 15),
                date(2019, 6, 5),
                // 2020: May 24 (Sun), 25 (Mon sub)
                date(2020, 5, 24),
                date(2020, 5, 25),
                date(2021, 5, 13),
                date(2022, 5, 3),
                date(2023, 4, 22),
                date(2024, 4, 10),
                date(2025, 3, 31),
                date(2026, 3, 21), // tentative
                date(2027, 3, 10), // MOM 2027 gazette
            ],
        )
        .with_valid_range(2010, 2027),
    )
}

/// Hari Raya Haji (Eid al-Adha) — with substitute Monday when the date
/// falls on Sunday (2010–2027). 2027 confirmed by the MOM press release
/// "Public Holidays for 2027" (18 Jun 2026).
pub fn sg_hari_raya_haji() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "Hari Raya Haji",
            [
                date(2010, 11, 17),
                // 2011: Nov 6 (Sun), 7 (Mon sub)
                date(2011, 11, 6),
                date(2011, 11, 7),
                date(2012, 10, 26),
                date(2013, 10, 15),
                // 2014: Oct 5 (Sun), 6 (Mon sub)
                date(2014, 10, 5),
                date(2014, 10, 6),
                date(2015, 9, 24),
                date(2016, 9, 12),
                date(2017, 9, 1),
                date(2018, 8, 22),
                // 2019: Aug 11 (Sun), 12 (Mon sub)
                date(2019, 8, 11),
                date(2019, 8, 12),
                date(2020, 7, 31),
                date(2021, 7, 20),
                // 2022: Jul 10 (Sun), 11 (Mon sub)
                date(2022, 7, 10),
                date(2022, 7, 11),
                date(2023, 6, 29),
                date(2024, 6, 17),
                date(2025, 6, 7),
                date(2026, 5, 27), // tentative
                date(2027, 5, 17), // MOM 2027 gazette
            ],
        )
        .with_valid_range(2010, 2027),
    )
}

/// Vesak Day — with substitute Monday when the date falls on Sunday
/// (2010–2027). 2027 per the MOM press release "Public Holidays for 2027"
/// (18 Jun 2026): Thu 20 May 2027.
pub fn sg_vesak_day() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "Vesak Day",
            [
                date(2010, 5, 28),
                date(2011, 5, 17),
                date(2012, 5, 5),
                date(2013, 5, 24),
                date(2014, 5, 13),
                date(2015, 6, 1),
                date(2016, 5, 21),
                date(2017, 5, 10),
                date(2018, 5, 29),
                // 2019: May 19 (Sun), 20 (Mon sub)
                date(2019, 5, 19),
                date(2019, 5, 20),
                date(2020, 5, 7),
                date(2021, 5, 26),
                // 2022: May 15 (Sun), 16 (Mon sub)
                date(2022, 5, 15),
                date(2022, 5, 16),
                // 2023: revised from Jun 3 to Jun 2 per MOM gazette (Sep 2022)
                date(2023, 6, 2),
                date(2024, 5, 22),
                date(2025, 5, 12),
                // 2026: May 31 (Sun), Jun 1 (Mon sub)
                date(2026, 5, 31),
                date(2026, 6, 1),
                date(2027, 5, 20),
            ],
        )
        .with_valid_range(2010, 2027),
    )
}

/// Deepavali — with substitute Monday when the date falls on Sunday
/// (2010–2027). 2027 per the MOM press release "Public Holidays for 2027"
/// (18 Jun 2026): Thu 28 Oct 2027 (an earlier projection had 29 Oct).
pub fn sg_deepavali() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(
        DateList::new(
            "Deepavali",
            [
                date(2010, 11, 5),
                date(2011, 10, 26),
                date(2012, 11, 13),
                date(2013, 11, 2),
                date(2014, 10, 22),
                date(2015, 11, 10),
                date(2016, 10, 29),
                date(2017, 10, 18),
                date(2018, 11, 6),
                // 2019: Oct 27 (Sun), 28 (Mon sub)
                date(2019, 10, 27),
                date(2019, 10, 28),
                date(2020, 11, 14),
                date(2021, 11, 4),
                date(2022, 10, 24),
                // 2023: Nov 12 (Sun), 13 (Mon sub)
                date(2023, 11, 12),
                date(2023, 11, 13),
                date(2024, 10, 31),
                date(2025, 10, 20),
                // 2026: Nov 8 (Sun), 9 (Mon sub)
                date(2026, 11, 8),
                date(2026, 11, 9),
                date(2027, 10, 28),
            ],
        )
        .with_valid_range(2010, 2027),
    )
}

/// SG50 Public Holiday — one-off holiday on 7 August 2015 marking
/// Singapore's 50th anniversary of independence.
pub fn sg50_public_holiday() -> Arc<dyn HolidayRule> {
    Arc::new(DateList::new(
        "SG50 Public Holiday",
        [jiff::civil::date(2015, 8, 7)],
    ))
}

/// Singapore Polling Day — gazetted public holidays for general and
/// presidential elections. These are irregular one-off holidays declared
/// under the Parliamentary Elections Act / Presidential Elections Act.
///
/// Deliberately without declared coverage: future polling days are
/// unknowable until an election is called, so a missing future date is not
/// a list-maintenance failure the coverage guard could catch.
pub fn sg_polling_day() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(DateList::new(
        "Singapore Polling Day",
        [
            date(2015, 9, 11), // General Election (gazetted, Fri)
            date(2020, 7, 10), // General Election
            date(2023, 9, 1),  // Presidential Election
        ],
    ))
}

// ---------------------------------------------------------------------------
// Platts global one-off closures
// ---------------------------------------------------------------------------

/// Platts global closure (2018 year-end) — one-off full closure of all
/// Platts pricing services and publications on Mon 24 Dec 2018 ("Day
/// before Christmas") and Mon 31 Dec 2018 ("Day before New Year's Day").
///
/// Distinct from the usual "Early Close" Platts observes when Dec 24 / Dec 31
/// fall on weekdays — in 2018 Platts elected a full no-publication closure,
/// covering "All Platts Offices" globally per the 2018 publishing schedule.
/// Other weekday-25-Dec years in the schedule range (2016–2026) show only
/// Early Close entries, so this is treated as a one-off rather than a
/// recurring rule.
///
/// Used by [`crate::builtin::calendars::platts_european_market_scan`] and
/// [`crate::builtin::calendars::platts_asia_marketscan`]. Not used by
/// UK Settlement or Argus Asia LPG (neither publisher observed it).
///
/// Source: the Platts holiday schedule 2018 —
/// "No Platts' pricing services or pricing publications worldwide."
pub fn platts_year_end_2018() -> Arc<dyn HolidayRule> {
    use jiff::civil::date;
    Arc::new(DateList::new(
        "Platts global closure (2018 year-end)",
        [
            date(2018, 12, 24), // Day before Christmas (Mon, Christmas Tue)
            date(2018, 12, 31), // Day before New Year's Day (Mon, NY Day Tue)
        ],
    ))
}

// ---------------------------------------------------------------------------
// SG Sun→Mon-only observed rules (publication calendars)
// ---------------------------------------------------------------------------
//
// Price-reporting publications assessed out of Singapore (Platts Asian
// MarketScan, Argus Asia LPG) follow Singapore gazetted holidays but only
// observe the Sunday→Monday substitution. When a holiday falls on Saturday
// (already a non-publication day), no substitute Monday is given. This
// differs from the SGX/MOM rules which substitute both Saturday and Sunday
// holidays to Monday.

/// New Year's Day (SG Sun→Mon Observed) — only Sunday → Monday.
///
/// When Jan 1 falls on Saturday, no substitute Monday is given.
pub fn sg_sunmon_new_years_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "New Year's Day (SG Sun→Mon Observed)",
        1,
        1,
        ObservancePolicy::SundayToMonday,
    ))
}

/// Labour Day (SG Sun→Mon Observed) — only Sunday → Monday.
///
/// When May 1 falls on Saturday, no substitute Monday is given.
pub fn sg_sunmon_labour_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Labour Day (SG Sun→Mon Observed)",
        5,
        1,
        ObservancePolicy::SundayToMonday,
    ))
}

/// Singapore National Day (SG Sun→Mon Observed) — only Sunday → Monday.
///
/// When Aug 9 falls on Saturday, no substitute Monday is given.
pub fn sg_sunmon_national_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Singapore National Day (SG Sun→Mon Observed)",
        8,
        9,
        ObservancePolicy::SundayToMonday,
    ))
}

/// Christmas Day (SG Sun→Mon Observed) — only Sunday → Monday.
///
/// When Dec 25 falls on Saturday, no substitute Monday is given.
pub fn sg_sunmon_christmas_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Christmas Day (SG Sun→Mon Observed)",
        12,
        25,
        ObservancePolicy::SundayToMonday,
    ))
}

// ---------------------------------------------------------------------------
// ICE Futures Europe observed rules (SundayToMonday only)
// ---------------------------------------------------------------------------
//
// ICE Futures Europe does NOT observe Saturday holidays on the preceding
// Friday (confirmed: Christmas 2021 fell on Saturday, ICE traded Fri Dec 24).
// When a holiday falls on Sunday, the following Monday is closed (confirmed:
// Christmas 2022 fell on Sunday, ICE closed Mon Dec 26).

/// Christmas Day (ICE Observed) — only Sunday → Monday.
///
/// When Dec 25 falls on Saturday, ICE trades normally on Friday.
/// When Dec 25 falls on Sunday, ICE closes the following Monday.
pub fn ice_christmas_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "Christmas Day (ICE Observed)",
        12,
        25,
        ObservancePolicy::SundayToMonday,
    ))
}

/// New Year's Day (ICE Observed) — only Sunday → Monday.
///
/// Used by [`super::calendars::ice_uk_averaging`] only: on the Monday after
/// a Sunday New Year ICE publishes a settlement record with every price
/// carried over from the previous session (2023-01-02), and the day does not
/// count in ICE's own monthly averages. [`super::calendars::ice_uk`] (days
/// with a published record) deliberately has no New Year observance.
pub fn ice_new_years_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "New Year's Day (ICE Observed)",
        1,
        1,
        ObservancePolicy::SundayToMonday,
    ))
}

// ---------------------------------------------------------------------------
// CME Group / NYMEX rules
// ---------------------------------------------------------------------------
//
// CME Group does not derive settlement prices on US holidays. Independence
// Day and Christmas follow the US federal observance (Sat → Fri, Sun → Mon;
// the shared `us_*_observed` rules); New Year's Day and Juneteenth differ.

/// New Year's Day (CME Observed) — only Sunday → Monday.
///
/// When Jan 1 falls on Saturday the preceding Friday, Dec 31, is a regular
/// settlement day (2021-12-31: NYMEX prices moved; ICE dates the Henry
/// January 2028 expiry on the assumption that Fri 2027-12-31 is a business
/// day). When Jan 1 falls on Sunday the Monday is closed (2023-01-02).
/// CME's settlement notice for New Year 2028 states it: Fri 2027-12-31
/// settles at normal times and Mon 2028-01-03 is open with settlement.
pub fn cme_new_years_day_observed() -> Arc<dyn HolidayRule> {
    Arc::new(ObservedFixedDate::new(
        "New Year's Day (CME Observed)",
        1,
        1,
        ObservancePolicy::SundayToMonday,
    ))
}

/// Juneteenth for CME Group exchanges — from 2022 only.
///
/// CME announced the holiday in December 2021 (SER-8888) for 2022 onwards;
/// Fri 2021-06-18 was a regular settlement day. The federal [`juneteenth`]
/// rule (from 2021) is therefore wrong for CME.
pub fn cme_juneteenth() -> Arc<dyn HolidayRule> {
    Arc::new(FixedDate::new("Juneteenth (CME, from 2022)", 6, 19).with_from_year(2022))
}

/// Juneteenth (Observed) for CME Group exchanges — from 2022 only.
///
/// Saturday → preceding Friday (Sat 2027-06-19 → Fri 2027-06-18, per CME's
/// settlement notice), Sunday → following Monday (Sun 2022-06-19 → Mon
/// 2022-06-20, a day without NYMEX settlement).
pub fn cme_juneteenth_observed() -> Arc<dyn HolidayRule> {
    Arc::new(
        ObservedFixedDate::new(
            "Juneteenth (CME Observed, from 2022)",
            6,
            19,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )
        .with_from_year(2022),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn shared_rules_resolve_correctly() {
        assert_eq!(new_years_day().holidays_in(2025), vec![date(2025, 1, 1)]);
        assert_eq!(good_friday().holidays_in(2025), vec![date(2025, 4, 18)]);
        assert_eq!(easter_monday().holidays_in(2025), vec![date(2025, 4, 21)]);
        assert_eq!(christmas_day().holidays_in(2025), vec![date(2025, 12, 25)]);
    }

    #[test]
    fn target_specific_rules() {
        assert_eq!(labour_day().holidays_in(2025), vec![date(2025, 5, 1)]);
        assert_eq!(
            st_stephens_day().holidays_in(2025),
            vec![date(2025, 12, 26)]
        );
    }

    #[test]
    fn us_nth_weekday_rules() {
        assert_eq!(mlk_day().holidays_in(2025), vec![date(2025, 1, 20)]);
        assert_eq!(presidents_day().holidays_in(2025), vec![date(2025, 2, 17)]);
        assert_eq!(memorial_day().holidays_in(2025), vec![date(2025, 5, 26)]);
        assert_eq!(labor_day().holidays_in(2025), vec![date(2025, 9, 1)]);
        assert_eq!(columbus_day().holidays_in(2025), vec![date(2025, 10, 13)]);
        assert_eq!(thanksgiving().holidays_in(2025), vec![date(2025, 11, 27)]);
    }

    #[test]
    fn juneteenth_bounded() {
        assert!(juneteenth().holidays_in(2020).is_empty());
        assert_eq!(juneteenth().holidays_in(2025), vec![date(2025, 6, 19)]);
        assert!(us_juneteenth_observed().holidays_in(2020).is_empty());
    }

    #[test]
    fn us_observed_rules() {
        // 2022: Jan 1 is Saturday -> observed Fri Dec 31, 2021
        // Querying 2022 returns nothing; querying 2021 returns the date.
        assert!(us_new_years_day_observed().holidays_in(2022).is_empty());
        assert_eq!(
            us_new_years_day_observed().holidays_in(2021),
            vec![date(2021, 12, 31)]
        );
        // 2020: Jul 4 is Saturday -> observed Fri Jul 3
        assert_eq!(
            us_independence_day_observed().holidays_in(2020),
            vec![date(2020, 7, 3)]
        );
        // 2025: weekday -> no observed shift
        assert!(us_christmas_day_observed().holidays_in(2025).is_empty());
    }

    #[test]
    fn uk_bank_holidays() {
        assert_eq!(
            early_may_bank_holiday().holidays_in(2025),
            vec![date(2025, 5, 5)]
        );
        assert_eq!(
            spring_bank_holiday().holidays_in(2025),
            vec![date(2025, 5, 26)]
        );
        assert_eq!(
            summer_bank_holiday().holidays_in(2025),
            vec![date(2025, 8, 25)]
        );
        assert_eq!(boxing_day().holidays_in(2025), vec![date(2025, 12, 26)]);
    }

    #[test]
    fn early_may_excluded_2020() {
        // 2020: Early May moved to May 8 for VE Day → rule returns empty
        assert!(early_may_bank_holiday().holidays_in(2020).is_empty());
        // VE Day rule covers May 8
        assert_eq!(ve_day_2020().holidays_in(2020), vec![date(2020, 5, 8)]);
        assert!(ve_day_2020().holidays_in(2025).is_empty());
    }

    #[test]
    fn spring_bank_excluded_2022() {
        // 2022: Spring Bank Holiday moved to June 2 for Jubilee → rule returns empty
        assert!(spring_bank_holiday().holidays_in(2022).is_empty());
        // Moved rule covers June 2
        assert_eq!(
            spring_bank_holiday_2022().holidays_in(2022),
            vec![date(2022, 6, 2)]
        );
        assert!(spring_bank_holiday_2022().holidays_in(2025).is_empty());
    }

    #[test]
    fn spring_bank_excluded_2002() {
        // 2002: Spring Bank Holiday moved to June 4 for the Golden Jubilee → rule returns empty
        assert!(spring_bank_holiday().holidays_in(2002).is_empty());
        assert_eq!(
            spring_bank_holiday_2002().holidays_in(2002),
            vec![date(2002, 6, 4)]
        );
        assert!(spring_bank_holiday_2002().holidays_in(2025).is_empty());
        // Golden Jubilee one-off fires June 3
        assert_eq!(
            queens_golden_jubilee().holidays_in(2002),
            vec![date(2002, 6, 3)]
        );
    }

    #[test]
    fn spring_bank_excluded_2012() {
        // 2012: Spring Bank Holiday moved to June 4 for Diamond Jubilee → rule returns empty
        assert!(spring_bank_holiday().holidays_in(2012).is_empty());
        // Moved rule covers June 4
        assert_eq!(
            spring_bank_holiday_2012().holidays_in(2012),
            vec![date(2012, 6, 4)]
        );
        assert!(spring_bank_holiday_2012().holidays_in(2025).is_empty());
        // Diamond Jubilee one-off fires June 5
        assert_eq!(
            queens_diamond_jubilee().holidays_in(2012),
            vec![date(2012, 6, 5)]
        );
        // Regular years still fire on last Mon of May
        assert_eq!(
            spring_bank_holiday().holidays_in(2013),
            vec![date(2013, 5, 27)]
        );
    }

    #[test]
    fn uk_observed_rules() {
        // 2022: Jan 1 is Saturday -> observed Mon Jan 3
        assert_eq!(
            uk_new_years_day_observed().holidays_in(2022),
            vec![date(2022, 1, 3)]
        );
        // 2021: Dec 25 is Saturday -> Christmas observed Mon Dec 27
        assert_eq!(
            uk_christmas_day_observed().holidays_in(2021),
            vec![date(2021, 12, 27)]
        );
        // 2020: Dec 26 is Saturday -> Boxing Day observed Mon Dec 28
        assert_eq!(
            uk_boxing_day_observed().holidays_in(2020),
            vec![date(2020, 12, 28)]
        );
    }

    #[test]
    fn distinct_names_for_dec_26() {
        assert_eq!(st_stephens_day().name(), "St. Stephen's Day");
        assert_eq!(boxing_day().name(), "Boxing Day");
    }

    #[test]
    fn uk_one_off_bank_holidays() {
        assert_eq!(
            queens_golden_jubilee().holidays_in(2002),
            vec![date(2002, 6, 3)]
        );
        assert!(queens_golden_jubilee().holidays_in(2025).is_empty());

        assert_eq!(
            royal_wedding_2011().holidays_in(2011),
            vec![date(2011, 4, 29)]
        );
        assert!(royal_wedding_2011().holidays_in(2025).is_empty());

        assert_eq!(
            queens_diamond_jubilee().holidays_in(2012),
            vec![date(2012, 6, 5)]
        );
        assert!(queens_diamond_jubilee().holidays_in(2025).is_empty());

        assert_eq!(
            queens_platinum_jubilee().holidays_in(2022),
            vec![date(2022, 6, 3)]
        );
        assert!(queens_platinum_jubilee().holidays_in(2025).is_empty());

        assert_eq!(queens_funeral().holidays_in(2022), vec![date(2022, 9, 19)]);
        assert!(queens_funeral().holidays_in(2025).is_empty());

        assert_eq!(
            coronation_bank_holiday().holidays_in(2023),
            vec![date(2023, 5, 8)]
        );
        assert!(coronation_bank_holiday().holidays_in(2025).is_empty());
    }

    // --- Baltic Exchange tests ---

    #[test]
    fn baltic_christmas_produces_seven_dates() {
        let rule = baltic_christmas();
        let dates = rule.holidays_in(2025);
        assert_eq!(dates.len(), 7);
        assert_eq!(dates[0], date(2025, 12, 25));
        assert_eq!(dates[6], date(2025, 12, 31));
    }

    #[test]
    fn baltic_christmas_unbounded() {
        let rule = baltic_christmas();
        assert_eq!(rule.holidays_in(2000).len(), 7);
        assert_eq!(rule.holidays_in(2050).len(), 7);
    }

    // --- Singapore-specific tests ---

    #[test]
    fn sg_national_day_fixed() {
        assert_eq!(sg_national_day().holidays_in(2025), vec![date(2025, 8, 9)]);
    }

    #[test]
    fn sg_national_day_observed_sunday() {
        // 2015: Aug 9 is Sunday → observed Mon Aug 10
        assert_eq!(
            sg_national_day_observed().holidays_in(2015),
            vec![date(2015, 8, 10)]
        );
        // 2025: Aug 9 is Saturday → NextMonday shifts Sat too
        assert_eq!(
            sg_national_day_observed().holidays_in(2025),
            vec![date(2025, 8, 11)]
        );
        // 2024: Aug 9 is Friday → no observed shift
        assert!(sg_national_day_observed().holidays_in(2024).is_empty());
    }

    #[test]
    fn sg_chinese_new_year_dates() {
        let rule = sg_chinese_new_year();
        // 2025: Jan 29, 30 (Wed, Thu)
        assert_eq!(
            rule.holidays_in(2025),
            vec![date(2025, 1, 29), date(2025, 1, 30)]
        );
        // 2023: Jan 22 (Sun), 23 (Mon), 24 (Tue sub)
        assert_eq!(
            rule.holidays_in(2023),
            vec![date(2023, 1, 22), date(2023, 1, 23), date(2023, 1, 24)]
        );
        // 2024: Feb 10 (Sat), 11 (Sun), 12 (Mon sub)
        assert_eq!(
            rule.holidays_in(2024),
            vec![date(2024, 2, 10), date(2024, 2, 11), date(2024, 2, 12)]
        );
    }

    #[test]
    fn sg_hari_raya_puasa_dates() {
        let rule = sg_hari_raya_puasa();
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 3, 31)]);
        // 2020: May 24 (Sun), 25 (Mon sub)
        assert_eq!(
            rule.holidays_in(2020),
            vec![date(2020, 5, 24), date(2020, 5, 25)]
        );
        assert!(rule.holidays_in(2009).is_empty());
    }

    #[test]
    fn sg_hari_raya_haji_dates() {
        let rule = sg_hari_raya_haji();
        // 2019: Aug 11 (Sun), 12 (Mon sub)
        assert_eq!(
            rule.holidays_in(2019),
            vec![date(2019, 8, 11), date(2019, 8, 12)]
        );
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 6, 7)]);
    }

    #[test]
    fn sg_vesak_day_dates() {
        let rule = sg_vesak_day();
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 5, 12)]);
        // 2022: May 15 (Sun), 16 (Mon sub)
        assert_eq!(
            rule.holidays_in(2022),
            vec![date(2022, 5, 15), date(2022, 5, 16)]
        );
        // 2027: Thu 20 May (MOM 2027 gazette)
        assert_eq!(rule.holidays_in(2027), vec![date(2027, 5, 20)]);
    }

    #[test]
    fn sg_deepavali_dates() {
        let rule = sg_deepavali();
        assert_eq!(rule.holidays_in(2025), vec![date(2025, 10, 20)]);
        // 2019: Oct 27 (Sun), 28 (Mon sub)
        assert_eq!(
            rule.holidays_in(2019),
            vec![date(2019, 10, 27), date(2019, 10, 28)]
        );
    }

    #[test]
    fn sg50_one_off() {
        let rule = sg50_public_holiday();
        assert_eq!(rule.holidays_in(2015), vec![date(2015, 8, 7)]);
        assert!(rule.holidays_in(2016).is_empty());
    }

    #[test]
    fn sg_polling_day_dates() {
        let rule = sg_polling_day();
        assert_eq!(rule.holidays_in(2015), vec![date(2015, 9, 11)]);
        assert_eq!(rule.holidays_in(2020), vec![date(2020, 7, 10)]);
        assert_eq!(rule.holidays_in(2023), vec![date(2023, 9, 1)]);
        assert!(rule.holidays_in(2025).is_empty());
    }

    #[test]
    fn sg_sunmon_observed_sunday_only() {
        // National Day: 2020 Aug 9 = Sunday → Mon Aug 10 (fires)
        assert_eq!(
            sg_sunmon_national_day_observed().holidays_in(2020),
            vec![date(2020, 8, 10)]
        );
        // National Day: 2025 Aug 9 = Saturday → no shift (Platts ignores Saturday)
        assert!(
            sg_sunmon_national_day_observed()
                .holidays_in(2025)
                .is_empty()
        );

        // New Year: 2023 Jan 1 = Sunday → Mon Jan 2 (fires)
        assert_eq!(
            sg_sunmon_new_years_day_observed().holidays_in(2023),
            vec![date(2023, 1, 2)]
        );
        // New Year: 2022 Jan 1 = Saturday → no shift
        assert!(
            sg_sunmon_new_years_day_observed()
                .holidays_in(2022)
                .is_empty()
        );

        // Labour Day: 2022 May 1 = Sunday → Mon May 2 (fires)
        assert_eq!(
            sg_sunmon_labour_day_observed().holidays_in(2022),
            vec![date(2022, 5, 2)]
        );
        // Labour Day: 2021 May 1 = Saturday → no shift
        assert!(sg_sunmon_labour_day_observed().holidays_in(2021).is_empty());

        // Christmas: 2022 Dec 25 = Sunday → Mon Dec 26 (fires)
        assert_eq!(
            sg_sunmon_christmas_day_observed().holidays_in(2022),
            vec![date(2022, 12, 26)]
        );
        // Christmas: 2021 Dec 25 = Saturday → no shift
        assert!(
            sg_sunmon_christmas_day_observed()
                .holidays_in(2021)
                .is_empty()
        );
    }

    #[test]
    fn vesak_day_2023_corrected() {
        let rule = sg_vesak_day();
        // 2023: revised to Jun 2 (not Jun 3)
        assert_eq!(rule.holidays_in(2023), vec![date(2023, 6, 2)]);
    }
}
