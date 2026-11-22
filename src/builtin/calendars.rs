//! Built-in market calendar constructors.
//!
//! Each function assembles a [`RuleBasedCalendar`] from individual rules defined
//! in [`holidays`]. The calendar definitions are declarative —
//! just a name, time zone, and a list of rule references.
//!
//! | Calendar | Function | Rules |
//! |----------|----------|-------|
//! | ECB TARGET | [`target`] | 6 rules, no observance |
//! | EEX Power Derivatives | [`eex_power_derivatives`] | 8 rules, no observance |
//! | US Settlement | [`us_settlement`] | 11 base + observed variants |
//! | UK Settlement | [`uk_settlement`] | 8 base + observed + one-offs |
//! | Platts European Market Scan | [`platts_european_market_scan`] | UK base rules + Platts 2018 year-end closure |
//! | ICE UK | [`ice_uk`] | 4 rules (New Year, Good Friday, Christmas + Christmas observed) |
//! | Singapore Exchange (SGX) | [`sgx`] | 15 rules (fixed + observed + date-list), 2010–2027 |
//! | Baltic Exchange LPG | [`baltic_lpg`] | UK bank holidays + Christmas closure + one-offs |
//! | Baltic Exchange US tanker | [`baltic_us`] | Baltic base + US holidays (Memorial Day, July 4 + observed, Labor Day, Thanksgiving; MLK Day + Juneteenth from 2027) |
//! | Baltic BLPG1 (Hormuz crisis) | [`baltic_lpg1_hormuz`] | Baltic base + 2026 reduced publication (Wed weekly, then Tue+Thu), valid to 2026 |
//! | Baltic Exchange Asia | [`baltic_asia`] | Singapore holidays (Sun→Mon observed) + Dec 25–31 closure, 2010–2027 |
//! | Platts Asia Marketscan | [`platts_asia_marketscan`] | 17 rules (shared + SG Sun→Mon observed + date-list + polling day + SG50 + Platts 2018 year-end closure), 2010–2027 |
//! | Platts US Marketscan | [`platts_us_marketscan`] | 18 rules (US pricing holidays + Good Friday + Juneteenth-from-2022 + July-4 bridges + one-offs), 2016–2027 |
//! | Platts US Gas Daily | [`platts_us_gas_daily`] | 22 rules (the US Marketscan set + Veterans Day from 2022 + Columbus Day from 2024 + 2026-01-02 one-off), 2016–2027 |
//! | ICIS Europe Chemicals | [`icis_europe_chemicals`] | UK base rules + ICIS year-end closure DateList, 2022–2026 |
//! | ICIS China Chemicals | [`icis_china_chemicals`] | 7 Chinese-gazette DateLists, 2022–2027 |
//! | China Futures Exchanges | [`china_futures_exchanges`] | 7 shared Chinese-gazette DateLists + 2024 CNY-eve closure, implicit weekends, 2022–2027 |
//! | OPIS PetroChem Wire | [`opis_petrochem_wire`] | 16 rules (US pricing holidays + Good Friday + one-off), 2025–2027 |
//! | Argus International LPG (Asia) | [`argus_asia_lpg`] | 17 rules — Platts Asia dates with Argus-provenance 2018 year-end closure, 2010–2027 |
//! | Argus International LPG (Europe) | [`argus_europe_lpg`] | UK base rules + Argus 2018 year-end closure |
//! | ICE UK averaging days | [`ice_uk_averaging`] | ICE UK rules + New Year Sun→Mon |
//! | NYMEX (CME Group energy) | [`nymex`] | 14 rules (10 US holidays without settlement; New Year Sun→Mon only, Juneteenth from 2022), 2021–2099 |
//! | ICE Endex gas | [`ice_endex`] | 5 rules (New Year, Good Friday, Easter Monday, 25 and 26 December), no observance, 2018–2099 |

use jiff::tz::TimeZone;

use super::holidays;
use crate::rule_calendar::RuleBasedCalendar;

/// Returns the ECB TARGET [`RuleBasedCalendar`].
///
/// 6 holidays, no observance shifts. Time zone: Europe/Berlin.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::target;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = target();
/// assert!(cal.is_holiday(date(2025, 5, 1)).unwrap()); // Labour Day
/// assert!(cal.is_business_day(date(2025, 5, 2)).unwrap());
/// ```
pub fn target() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("TARGET")
        .with_description(
            "TARGET2 euro settlement days (ECB) — governs EUR cash/settlement conventions",
        )
        .with_tz(TimeZone::get("Europe/Berlin").expect("Europe/Berlin timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::easter_monday(),
            holidays::labour_day(),
            holidays::christmas_day(),
            holidays::st_stephens_day(),
        ])
        .build()
        .expect("TARGET calendar: rules cover requested range")
}

/// Returns the EEX Power Derivatives [`RuleBasedCalendar`].
///
/// EEX exchange days are all Monday–Friday business days except the eight
/// full-day closures below. Source: EEX *Holiday Calendar — Derivatives &
/// Emissions Spot* (Power column; the "X = markets closed full day" marker
/// applies to all eight for Power, unlike Gas/Freight which trade reduced
/// hours on Dec 24 & 31). The calendar applies continuously for all years.
/// No weekend-observance shifts. Time zone: Europe/Berlin (CET).
///
/// 8 holidays: New Year (Jan 1), Good Friday, Easter Monday, Labour Day
/// (May 1), Christmas Eve (Dec 24), Christmas Day (Dec 25), Boxing Day
/// (Dec 26), New Year's Eve (Dec 31).
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::eex_power_derivatives;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = eex_power_derivatives();
/// assert!(cal.is_holiday(date(2025, 12, 24)).unwrap()); // Christmas Eve
/// assert!(cal.is_holiday(date(2025, 12, 31)).unwrap()); // New Year's Eve
/// assert!(cal.is_holiday(date(2025, 4, 18)).unwrap());   // Good Friday
/// assert!(cal.is_business_day(date(2025, 5, 2)).unwrap());
/// ```
pub fn eex_power_derivatives() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("EEXPowerDerivatives")
        .with_description(
            "EEX Power Derivatives trading days — closed only New Year, Good Friday, \
             Easter Monday, Labour Day, Dec 24/25/26/31 (no observance shifts)",
        )
        .with_tz(TimeZone::get("Europe/Berlin").expect("Europe/Berlin timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::easter_monday(),
            holidays::labour_day(),
            holidays::christmas_eve(),
            holidays::christmas_day(),
            holidays::boxing_day(),
            holidays::new_years_eve(),
        ])
        .build()
        .expect("EEXPowerDerivatives calendar: rules cover requested range")
}

/// Returns the US Settlement (Federal) [`RuleBasedCalendar`].
///
/// 11 holidays with US federal observance policy. Time zone: America/New_York.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::us_settlement;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = us_settlement();
/// assert!(cal.is_holiday(date(2025, 7, 4)).unwrap()); // Independence Day
/// assert!(cal.is_holiday(date(2025, 11, 27)).unwrap()); // Thanksgiving
/// ```
pub fn us_settlement() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("USSettlement")
        .with_description("US federal-holiday settlement calendar")
        .with_tz(TimeZone::get("America/New_York").expect("America/New_York timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(vec![
            // New Year's Day + observed
            holidays::new_years_day(),
            holidays::us_new_years_day_observed(),
            // MLK Day
            holidays::mlk_day(),
            // Presidents' Day
            holidays::presidents_day(),
            // Memorial Day
            holidays::memorial_day(),
            // Juneteenth + observed (from 2021)
            holidays::juneteenth(),
            holidays::us_juneteenth_observed(),
            // Independence Day + observed
            holidays::independence_day(),
            holidays::us_independence_day_observed(),
            // Labor Day
            holidays::labor_day(),
            // Columbus Day
            holidays::columbus_day(),
            // Veterans Day + observed
            holidays::veterans_day(),
            holidays::us_veterans_day_observed(),
            // Thanksgiving
            holidays::thanksgiving(),
            // Christmas Day + observed
            holidays::christmas_day(),
            holidays::us_christmas_day_observed(),
        ])
        .build()
        .expect("USSettlement calendar: rules cover requested range")
}

/// Returns the common UK bank holiday rules (recurring + one-off).
///
/// Shared between [`uk_settlement`] and [`platts_european_market_scan`].
fn uk_base_rules() -> Vec<std::sync::Arc<dyn crate::rules::HolidayRule>> {
    vec![
        // New Year's Day + observed
        holidays::new_years_day(),
        holidays::uk_new_years_day_observed(),
        // Good Friday
        holidays::good_friday(),
        // Easter Monday
        holidays::easter_monday(),
        // Early May Bank Holiday
        holidays::early_may_bank_holiday(),
        // Spring Bank Holiday
        holidays::spring_bank_holiday(),
        // Summer Bank Holiday
        holidays::summer_bank_holiday(),
        // Christmas Day + UK cascade observed
        holidays::christmas_day(),
        holidays::uk_christmas_day_observed(),
        // Boxing Day + UK cascade observed
        holidays::boxing_day(),
        holidays::uk_boxing_day_observed(),
        // Moved bank holidays (recurring rule excluded for that year)
        holidays::ve_day_2020(),
        holidays::spring_bank_holiday_2002(),
        holidays::spring_bank_holiday_2012(),
        holidays::spring_bank_holiday_2022(),
        // One-off proclaimed bank holidays
        holidays::queens_golden_jubilee(),
        holidays::royal_wedding_2011(),
        holidays::queens_diamond_jubilee(),
        holidays::queens_platinum_jubilee(),
        holidays::queens_funeral(),
        holidays::coronation_bank_holiday(),
    ]
}

/// Returns the UK Settlement (Bank Holiday) [`RuleBasedCalendar`].
///
/// 8 recurring holidays with UK observance (NextMonday for most, cascade for
/// Christmas/Boxing Day), plus proclaimed one-off bank holidays (jubilees,
/// coronation, state funeral). Time zone: Europe/London.
///
/// Uses `uk_base_rules` only; [`platts_european_market_scan`] also uses
/// `uk_base_rules` but adds the 2018 Platts global year-end closure (Mon
/// 24 Dec 2018 and Mon 31 Dec 2018) — neither of which were UK bank
/// holidays, so they're omitted here.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::uk_settlement;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = uk_settlement();
/// assert!(cal.is_holiday(date(2025, 12, 25)).unwrap()); // Christmas
/// assert!(cal.is_holiday(date(2025, 12, 26)).unwrap()); // Boxing Day
/// assert!(cal.is_holiday(date(2023, 5, 8)).unwrap());   // Coronation
/// ```
pub fn uk_settlement() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("UKSettlement")
        .with_description("UK (England & Wales) bank-holiday settlement calendar")
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(uk_base_rules())
        .build()
        .expect("UKSettlement calendar: rules cover requested range")
}

/// Returns the Platts European Market Scan [`RuleBasedCalendar`].
///
/// Uses the UK bank-holiday rule set (shared with [`uk_settlement`]) plus
/// the [`crate::builtin::holidays::platts_year_end_2018`] one-off closure.
///
/// Differs from [`uk_settlement`] only by the 2018 year-end Platts global
/// closure on Mon 2018-12-24 and Mon 2018-12-31 — Platts elected a full
/// no-publication on those Mondays, while neither was a UK bank holiday.
///
/// Also governs **daily EU petrochemical benchmarks** (e.g. Benzene CIF ARA):
/// daily Platts assessments follow the regional MarketScan calendar, not a
/// product-type one — the Platts schedules' year-end "Petrochemicals"
/// suspension notices apply to weekly publications only.
///
/// # Verification
///
/// - Official Platts holiday schedule workbooks 2016–2026: all 93 weekday
///   holidays confirmed; one schedule-side anomaly (2018-05-01) refuted by
///   assessment data.
/// - Observed settlement series: Naphtha CIF NWE (2012–2025) and Benzene CIF
///   ARA (2010–2025) match 100%.
///
/// Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::platts_european_market_scan;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = platts_european_market_scan();
/// assert!(cal.is_holiday(date(2023, 5, 8)).unwrap());   // Coronation
/// assert!(cal.is_holiday(date(2025, 12, 25)).unwrap()); // Christmas
/// assert!(cal.is_holiday(date(2018, 12, 24)).unwrap()); // Platts 2018 year-end (24 Dec)
/// assert!(cal.is_holiday(date(2018, 12, 31)).unwrap()); // Platts 2018 year-end (31 Dec)
/// ```
pub fn platts_european_market_scan() -> RuleBasedCalendar {
    let mut rules = uk_base_rules();
    rules.push(holidays::platts_year_end_2018());
    RuleBasedCalendar::builder("PlattsEuropeanMarketScan")
        .with_description(
            "Daily Platts London-assessed benchmarks — European Marketscan oil products \
             (e.g. Naphtha CIF NWE) and daily EU petrochemicals (e.g. Benzene CIF ARA)",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(rules)
        .build()
        .expect("PlattsEuropeanMarketScan calendar: rules cover requested range")
}

/// Returns the ICE UK [`RuleBasedCalendar`].
///
/// Minimal calendar for ICE Futures Europe cleared products: the days on
/// which ICE publishes a settlement record. ICE trades through most UK bank
/// holidays; only New Year's Day, Good Friday, and Christmas Day are
/// observed. Sunday→Monday observance for Christmas only (Saturday holidays
/// are not shifted to Friday). New Year has no weekend observance — a
/// settlement record is published on Jan 2 even when Jan 1 falls on Sunday.
/// On that day every price is carried over from the previous session and
/// the day does not count in ICE's monthly averages: use
/// [`ice_uk_averaging`] for fixing days. Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::ice_uk;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = ice_uk();
/// assert!(cal.is_holiday(date(2025, 1, 1)).unwrap());  // New Year's Day
/// assert!(cal.is_holiday(date(2025, 4, 18)).unwrap()); // Good Friday
/// assert!(cal.is_holiday(date(2025, 12, 25)).unwrap()); // Christmas Day
/// // ICE trades through UK bank holidays
/// assert!(cal.is_business_day(date(2025, 4, 21)).unwrap()); // Easter Monday
/// assert!(cal.is_business_day(date(2025, 12, 26)).unwrap()); // Boxing Day
/// ```
pub fn ice_uk() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("ICEUK")
        .with_description(
            "ICE Futures Europe cleared oil products — trades through most UK bank \
             holidays; closed only New Year, Good Friday, Christmas (+ observed)",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::christmas_day(),
            holidays::ice_christmas_day_observed(),
        ])
        .build()
        .expect("ICEUK calendar: rules cover requested range")
}

/// Returns the Singapore Exchange (SGX) [`RuleBasedCalendar`].
///
/// 15 rules covering fixed-date holidays with Sun→Mon observance, Good Friday,
/// and date-list rules for lunar/religious holidays. Valid range: 2010–2027.
/// Time zone: Asia/Singapore.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::sgx;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = sgx();
/// assert!(cal.is_holiday(date(2025, 1, 29)).unwrap()); // Chinese New Year
/// assert!(cal.is_holiday(date(2025, 4, 18)).unwrap()); // Good Friday
/// ```
pub fn sgx() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("SGX")
        .with_description("Singapore Exchange trading days (SGX-listed derivatives)")
        .with_tz(TimeZone::get("Asia/Singapore").expect("Asia/Singapore timezone"))
        .with_valid_range(2010, 2027)
        .with_rules(vec![
            // Shared algorithmic rules
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::labour_day(),
            holidays::christmas_day(),
            // SG observance (Sun→Mon)
            holidays::sg_new_years_day_observed(),
            holidays::sg_labour_day_observed(),
            holidays::sg_national_day(),
            holidays::sg_national_day_observed(),
            holidays::sg_christmas_day_observed(),
            // DateList holidays
            holidays::sg_chinese_new_year(),
            holidays::sg_hari_raya_puasa(),
            holidays::sg_hari_raya_haji(),
            holidays::sg_vesak_day(),
            holidays::sg_deepavali(),
            // One-off
            holidays::sg50_public_holiday(),
        ])
        .build()
        .expect("SGX calendar: rules cover requested range")
}

/// Returns the Baltic Exchange LPG [`RuleBasedCalendar`].
///
/// UK bank holidays plus the Baltic Exchange Christmas closure (Dec 25–31).
/// Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::baltic_lpg;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = baltic_lpg();
/// assert!(cal.is_holiday(date(2025, 12, 29)).unwrap()); // Christmas closure
/// assert!(cal.is_holiday(date(2025, 4, 18)).unwrap());  // Good Friday
/// ```
pub fn baltic_lpg() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("BalticLPG")
        .with_description("BDTI, BCTI + BLPG + BLNG")
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(baltic_base_rules())
        .build()
        .expect("BalticLPG calendar: rules cover requested range")
}

/// Shared holiday rules for all Baltic Exchange calendars (Group 1).
///
/// UK bank holidays plus Baltic Exchange Christmas closure (Dec 25-31),
/// including one-off proclaimed bank holidays and year-specific moves.
fn baltic_base_rules() -> Vec<std::sync::Arc<dyn crate::rules::HolidayRule>> {
    vec![
        holidays::new_years_day(),
        // Weekend New Year → UK-observed Monday: BLPG1/TC5 data confirm no
        // publication on 2023-01-02 (Jan 1 fell Sunday).
        holidays::uk_new_years_day_observed(),
        holidays::good_friday(),
        holidays::easter_monday(),
        holidays::early_may_bank_holiday(),
        holidays::spring_bank_holiday(),
        holidays::summer_bank_holiday(),
        holidays::baltic_christmas(),
        // Moved bank holidays (recurring rule excluded for that year)
        holidays::ve_day_2020(),
        holidays::spring_bank_holiday_2002(),
        holidays::spring_bank_holiday_2012(),
        holidays::spring_bank_holiday_2022(),
        // One-off proclaimed bank holidays
        holidays::queens_golden_jubilee(),
        holidays::royal_wedding_2011(),
        holidays::queens_diamond_jubilee(),
        holidays::queens_platinum_jubilee(),
        holidays::queens_funeral(),
        holidays::coronation_bank_holiday(),
    ]
}

/// Returns the Baltic Exchange US-reported tanker routes [`RuleBasedCalendar`].
///
/// Baltic base rules (Group 1) plus US holidays: Memorial Day, Independence
/// Day, Labor Day, Thanksgiving, and — from 2027, when the Baltic schedule
/// first lists them — Martin Luther King Jr. Day and Juneteenth. Applies to
/// BCTI routes TC14/TC18/TC21/TC24, MRA Triangulation and BDTI routes
/// TD9/TD21/TD25-TD29/TD33.
///
/// Time zone: Europe/London. Verified against the official "Non-publication
/// dates" PDFs 2025–2027.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::baltic_us;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = baltic_us();
/// assert!(cal.is_holiday(date(2025, 7, 4)).unwrap());   // Independence Day
/// assert!(cal.is_holiday(date(2025, 11, 27)).unwrap());  // Thanksgiving
/// assert!(cal.is_holiday(date(2025, 11, 28)).unwrap());  // Day after Thanksgiving
/// assert!(cal.is_holiday(date(2025, 12, 29)).unwrap());  // Christmas closure
/// ```
pub fn baltic_us() -> RuleBasedCalendar {
    let mut rules = baltic_base_rules();
    rules.extend([
        holidays::memorial_day(),
        holidays::independence_day(),
        // Sat July 4 → observed Friday: 2026 Baltic schedule lists 3 July.
        holidays::us_independence_day_observed(),
        holidays::labor_day(),
        holidays::thanksgiving(),
        holidays::day_after_thanksgiving(),
        // 2027 schedule adds MLK Day (18 Jan) and Juneteenth (Fri 18 Jun);
        // absent from the 2025/2026 schedules, hence from-2027 rules.
        holidays::baltic_us_mlk_day(),
        holidays::baltic_us_juneteenth(),
        holidays::baltic_us_juneteenth_observed(),
    ]);
    RuleBasedCalendar::builder("BalticUS")
        .with_description(
            "US-reported Baltic tanker routes (BCTI TC14/TC18/TC21/TC24, MRA Triangulation, \
             BDTI TD9/TD21/TD25-TD29/TD33)",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(rules)
        .build()
        .expect("BalticUS calendar: rules cover requested range")
}

/// Returns the BLPG1-during-Hormuz-crisis [`RuleBasedCalendar`].
///
/// [`baltic_lpg`] plus the 2026 Strait of Hormuz reduced publication
/// schedule for the **BLPG1 route only** (Ras Tanura → Chiba VLGC):
///
/// - until 2026-03-03: daily (identical to `BalticLPG`);
/// - 2026-03-04 → 2026-04-06: weekly, Wednesdays (Circular 13/26);
/// - from 2026-04-07: Tuesdays + Thursdays, until further notice
///   (Circular 16/26).
///
/// The other routes in the BDTI/BCTI/BLPG/BLNG group kept daily publication —
/// use [`baltic_lpg`] for them. **Valid range ends 2026-12-31**: the reduced
/// schedule is open-ended ("until further notice"), so queries beyond the
/// announced horizon fail loudly rather than assuming the crisis schedule
/// continues. Extend when the Baltic issues the next circular.
///
/// # Verification
///
/// Baltic Exchange circulars 13/26 and 16/26. Observed BLPG1 settlement
/// data matches the two regimes exactly
/// (2026-03 → 2026-07, checked 2026-07-08).
///
/// Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::baltic_lpg1_hormuz;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = baltic_lpg1_hormuz();
/// assert!(cal.is_business_day(date(2026, 3, 3)).unwrap());  // last daily day
/// assert!(cal.is_business_day(date(2026, 3, 4)).unwrap());  // Wednesday publishes
/// assert!(cal.is_holiday(date(2026, 3, 5)).unwrap());       // Thursday suppressed
/// assert!(cal.is_business_day(date(2026, 4, 7)).unwrap());  // Tuesday publishes
/// assert!(cal.is_holiday(date(2026, 4, 8)).unwrap());       // Wednesday now suppressed
/// assert!(cal.is_business_day(date(2026, 4, 9)).unwrap());  // Thursday publishes
/// // Beyond the announced horizon: loud failure
/// assert!(cal.is_business_day(date(2027, 1, 5)).is_err());
/// ```
pub fn baltic_lpg1_hormuz() -> RuleBasedCalendar {
    let mut rules = baltic_base_rules();
    rules.extend([
        holidays::blpg1_hormuz_weekly(),
        holidays::blpg1_hormuz_tue_thu(),
    ]);
    RuleBasedCalendar::builder("BalticLPG1Hormuz")
        .with_description(
            "BLPG1 route during the 2026 Hormuz-crisis reduced publication (weekly Wed from \
             2026-03-04, Tue+Thu from 2026-04-07, until further notice) — other BDTI/BCTI/BLPG \
             routes stay on BalticLPG",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2026)
        .with_rules(rules)
        .build()
        .expect("BalticLPG1Hormuz calendar: rules cover requested range")
}

/// Returns the Baltic Exchange Asia indices [`RuleBasedCalendar`].
///
/// Singapore public holidays (Sun→Mon-only observance, like the Platts/Argus
/// Singapore publication calendars) plus the Baltic Exchange Dec 25–31
/// Christmas closure. Applies to BITR-Asia, BES-Asia, BEP-Asia, BII.
///
/// **Not** the UK bank-holiday base: per the official Baltic non-publication
/// schedules, the Asia indices publish through Easter Monday and the UK
/// May/Spring/Summer bank holidays (Good Friday is a Singapore holiday and
/// stays). Saturday-falling SG holidays get no substitute Monday.
///
/// # Verification
///
/// Official Baltic Exchange non-publication schedules 2025–2026
/// exact match after this rule set
/// replaced the earlier UK-base model (which over-declared 9 holidays in
/// those two years). Pre-2025 behaviour has no primary source or settlement
/// data to verify against (the earlier calibration source was a vendor
/// holiday list, which disagrees with the official schedules).
///
/// Time zone: Europe/London (published by Baltic Exchange, Singapore
/// holiday schedule).
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::baltic_asia;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = baltic_asia();
/// assert!(cal.is_holiday(date(2026, 8, 10)).unwrap());       // SG National Day (Sun→Mon)
/// assert!(cal.is_holiday(date(2025, 12, 29)).unwrap());      // Christmas closure
/// // Sat-falling SG National Day 2025: no substitute Monday
/// assert!(cal.is_business_day(date(2025, 8, 11)).unwrap());
/// // Publishes through UK bank holidays
/// assert!(cal.is_business_day(date(2025, 4, 21)).unwrap());  // Easter Monday
/// ```
pub fn baltic_asia() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("BalticAsia")
        .with_description(
            "Baltic Asia indices (BITR-Asia, BES-Asia, BEP-Asia, BII) — SG holidays \
             (Sun→Mon observed) + Dec 25-31 closure; publishes through UK bank holidays",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2010, 2027)
        .with_rules(vec![
            // Shared algorithmic rules
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::labour_day(),
            // SG observance (Sun→Mon only, Saturday ignored)
            holidays::sg_sunmon_new_years_day_observed(),
            holidays::sg_sunmon_labour_day_observed(),
            holidays::sg_national_day(),
            holidays::sg_sunmon_national_day_observed(),
            // DateList holidays (SG-gazetted dates)
            holidays::sg_chinese_new_year(),
            holidays::sg_hari_raya_puasa(),
            holidays::sg_hari_raya_haji(),
            holidays::sg_vesak_day(),
            holidays::sg_deepavali(),
            // Baltic Exchange year-end closure (Dec 25-31, covers Christmas)
            holidays::baltic_christmas(),
        ])
        .build()
        .expect("BalticAsia calendar: rules cover requested range")
}

/// Returns the Platts Asia Marketscan [`RuleBasedCalendar`].
///
/// Covers Platts-assessed Asian commodities out of Singapore. Differs from
/// SGX in two key ways:
///
/// 1. **Observance policy**: Platts only observes Sunday→Monday substitution.
///    When a holiday falls on Saturday (already a non-publication day), no
///    substitute Monday is given. SGX follows the MOM gazette which substitutes
///    both Saturday and Sunday holidays.
/// 2. **Election Polling Days**: Included as ad-hoc holidays (SGX may or may
///    not close for elections; Platts confirmed non-publication).
///
/// Also governs **daily Asian petrochemical benchmarks** (e.g. Benzene FOB
/// Korea): daily Platts assessments follow the regional MarketScan calendar,
/// not a product-type one.
///
/// # Verification
///
/// - Official Platts holiday schedule workbooks 2016–2026: exact match,
///   111/111 weekday holidays.
/// - Observed settlement series: Naphtha C+F Japan and Benzene FOB Korea
///   (2010–2025, a handful of documented pre-2016 anomalies aside).
///
/// Valid range: 2010–2027 (constrained by date-list rules).
/// Time zone: Asia/Singapore.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::platts_asia_marketscan;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = platts_asia_marketscan();
/// assert!(cal.is_holiday(date(2025, 3, 31)).unwrap()); // Hari Raya Puasa
/// assert!(cal.is_holiday(date(2025, 5, 12)).unwrap()); // Vesak Day
/// assert!(cal.is_holiday(date(2021, 8, 9)).unwrap());  // National Day
/// assert!(cal.is_holiday(date(2020, 7, 10)).unwrap()); // GE Polling Day
/// assert!(cal.is_holiday(date(2023, 6, 2)).unwrap());  // Vesak Day 2023
/// ```
pub fn platts_asia_marketscan() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("PlattsAsiaMarketScan")
        .with_description(
            "Daily Platts Singapore-assessed benchmarks — Asia-Pacific/Arab Gulf Marketscan \
             oil products (e.g. Naphtha C+F Japan) and daily Asia petrochemicals (e.g. \
             Benzene FOB Korea)",
        )
        .with_tz(TimeZone::get("Asia/Singapore").expect("Asia/Singapore timezone"))
        .with_valid_range(2010, 2027)
        .with_rules(vec![
            // Shared algorithmic rules
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::labour_day(),
            holidays::christmas_day(),
            // SG observance (Sun→Mon only, Saturday ignored)
            holidays::sg_sunmon_new_years_day_observed(),
            holidays::sg_sunmon_labour_day_observed(),
            holidays::sg_national_day(),
            holidays::sg_sunmon_national_day_observed(),
            holidays::sg_christmas_day_observed(),
            // DateList holidays
            holidays::sg_chinese_new_year(),
            holidays::sg_hari_raya_puasa(),
            holidays::sg_hari_raya_haji(),
            holidays::sg_vesak_day(),
            holidays::sg_deepavali(),
            // One-off election holidays
            holidays::sg_polling_day(),
            // Singapore one-off public holiday — SG50 (7 Aug 2015)
            holidays::sg50_public_holiday(),
            // One-off Platts global closures
            holidays::platts_year_end_2018(),
        ])
        .build()
        .expect("PlattsAsiaMarketScan calendar: rules cover requested range")
}

/// Returns the Platts US Marketscan [`RuleBasedCalendar`].
///
/// Covers Platts-assessed US/Americas daily benchmarks: the US Marketscan
/// oil products (with Oilgram Price Report, North America Crude and Product
/// Scan, Latin American Wire) and — per the regional-MarketScan principle —
/// daily US petrochemicals (Platts US aromatics).
///
/// US pricing holidays with US federal observance (Sat→Fri / Sun→Mon), plus
/// Good Friday, **without** Columbus Day and Veterans Day (those appear in
/// the schedules only as partial "US gas and power pricing holiday" rows —
/// US Marketscan publishes). Platts adopted Juneteenth from **2022** (not
/// the federal 2021). July 4 on Tue/Thu brings a bridge day (Mon before /
/// Fri after) — a maintained DateList. One-offs: the 2018 global year-end
/// closure and Fri 2025-12-26.
///
/// # Verification
///
/// Extracted from the official Platts holiday schedule workbooks
/// (`holsked2016`–`2026`): full-suspension rows ("US Marketscan … not
/// published", "No markets assessed in US, Canada or Latin America",
/// worldwide closures). Exact match 2016–2026 — 122/122 weekday days. One
/// workbook error was adjudicated and excluded (2026-06-04 "Corpus Christi",
/// a Brazil-only holiday whose publications cell over-stated) — and later
/// confirmed: Platts's July 2026 workbook revision removes the claim.
///
/// Valid range: 2016–2027 (the July-4 bridge DateList makes earlier years
/// unreliable — 2013's bridge, for instance, is not modelled).
/// Time zone: America/New_York.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::platts_us_marketscan;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = platts_us_marketscan();
/// assert!(cal.is_holiday(date(2024, 1, 15)).unwrap());       // MLK Day
/// assert!(cal.is_holiday(date(2024, 7, 5)).unwrap());        // July 4 bridge (Thu→Fri)
/// assert!(cal.is_holiday(date(2022, 6, 20)).unwrap());       // Juneteenth observed
/// assert!(cal.is_business_day(date(2021, 6, 18)).unwrap());  // no Juneteenth 2021
/// assert!(cal.is_business_day(date(2024, 10, 14)).unwrap()); // Columbus Day publishes
/// assert!(cal.is_business_day(date(2026, 6, 4)).unwrap());   // excluded workbook error
/// ```
pub fn platts_us_marketscan() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("PlattsUSMarketScan")
        .with_description(
            "Daily Platts US-assessed benchmarks — US Marketscan oil products (with OPR, \
             NACPS, Latin American Wire) and daily US petrochemicals (Platts US \
             aromatics)",
        )
        .with_tz(TimeZone::get("America/New_York").expect("America/New_York timezone"))
        .with_valid_range(2016, 2027)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::us_new_years_day_observed(),
            holidays::mlk_day(),
            holidays::presidents_day(),
            holidays::good_friday(),
            holidays::memorial_day(),
            holidays::platts_juneteenth(),
            holidays::platts_juneteenth_observed(),
            holidays::independence_day(),
            holidays::us_independence_day_observed(),
            holidays::platts_us_july4_bridge(),
            holidays::labor_day(),
            holidays::thanksgiving(),
            holidays::day_after_thanksgiving(),
            holidays::christmas_day(),
            holidays::us_christmas_day_observed(),
            // One-off closures
            holidays::platts_year_end_2018(),
            holidays::platts_us_day_after_christmas_2025(),
        ])
        .build()
        .expect("PlattsUSMarketScan calendar: rules cover requested range")
}

/// Returns the Platts US Gas Daily [`RuleBasedCalendar`].
///
/// Publication days of Platts's **North American natural gas** assessments
/// — Gas Daily (with Gas Daily Market Fundamentals, Gas Daily Mexico and
/// Energy Trader): the daily spot indices such as Henry Hub, Houston Ship
/// Channel, Chicago city-gates, Waha and Algonquin city-gates.
///
/// Gas follows the US Marketscan set ([`platts_us_marketscan`]: US pricing
/// holidays with federal Sat→Fri / Sun→Mon observance, Good Friday,
/// Juneteenth from 2022, the July-4 bridge days, the 2018 year-end closure,
/// Fri 2025-12-26) **plus** the "US gas and power pricing holidays" that oil
/// and petrochemicals trade through: **Veterans Day from 2022** (federal
/// observance — Fri 2023-11-10) and **Columbus Day from 2024**, and the
/// one-off Fri **2026-01-02**. Canadian holidays (CGPR Daily) and the
/// European Gas Daily closures do not affect it.
///
/// # Verification
///
/// Extracted from the Gas rows / "Power & Gas" columns of the official
/// Platts holiday schedule workbooks (`holsked2016`–`2026`): rows stating
/// "No assessments for North American gas" / "Gas Daily … will not be
/// published" or a worldwide closure. Exact match 2016–2026 — 131/131
/// weekday days. Independently confirmed against the Platts Henry Hub
/// trade-date midpoint assessments: the weekdays without a value from
/// 2016-01 to 2026-10 are exactly this calendar's holidays, with no
/// exception.
///
/// Valid range: 2016–2027 (as [`platts_us_marketscan`]: the bridge-day
/// DateList makes earlier years unreliable, and the Columbus/Veterans Day
/// start years are only known from the schedules). Time zone:
/// America/New_York.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::platts_us_gas_daily;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = platts_us_gas_daily();
/// assert!(cal.is_holiday(date(2024, 10, 14)).unwrap());      // Columbus Day (from 2024)
/// assert!(cal.is_business_day(date(2023, 10, 9)).unwrap());  // … published in 2023
/// assert!(cal.is_holiday(date(2023, 11, 10)).unwrap());      // Veterans Day observed (Sat→Fri)
/// assert!(cal.is_business_day(date(2021, 11, 11)).unwrap()); // … published before 2022
/// assert!(cal.is_holiday(date(2026, 1, 2)).unwrap());        // one-off
/// assert!(cal.is_holiday(date(2024, 7, 5)).unwrap());        // July 4 bridge (Thu→Fri)
/// ```
pub fn platts_us_gas_daily() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("PlattsUSGasDaily")
        .with_description(
            "Daily Platts North American natural gas assessments — Gas Daily (Henry Hub and the \
             US spot hubs; with Gas Daily Market Fundamentals, Gas Daily Mexico, Energy Trader): \
             the PlattsUSMarketScan set + Veterans Day (from 2022) + Columbus Day (from 2024)",
        )
        .with_tz(TimeZone::get("America/New_York").expect("America/New_York timezone"))
        .with_valid_range(2016, 2027)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::us_new_years_day_observed(),
            holidays::mlk_day(),
            holidays::presidents_day(),
            holidays::good_friday(),
            holidays::memorial_day(),
            holidays::platts_juneteenth(),
            holidays::platts_juneteenth_observed(),
            holidays::independence_day(),
            holidays::us_independence_day_observed(),
            holidays::platts_us_july4_bridge(),
            holidays::labor_day(),
            // US gas and power pricing holidays — oil/petchem publish
            holidays::platts_gas_columbus_day(),
            holidays::platts_gas_veterans_day(),
            holidays::platts_gas_veterans_day_observed(),
            holidays::thanksgiving(),
            holidays::day_after_thanksgiving(),
            holidays::christmas_day(),
            holidays::us_christmas_day_observed(),
            // One-off closures
            holidays::platts_year_end_2018(),
            holidays::platts_us_day_after_christmas_2025(),
            holidays::platts_gas_day_after_new_year_2026(),
        ])
        .build()
        .expect("PlattsUSGasDaily calendar: rules cover requested range")
}

/// Returns the ICIS Europe Chemicals [`RuleBasedCalendar`].
///
/// Publication days of ICIS's **Europe daily chemicals** assessments — the
/// Aromatics daily publication family (region "Europe/Africa/Turkey";
/// e.g. benzene). Base: **"every UK working day"** (UK bank holidays incl.
/// proclaimed one-offs; no continental holidays) plus the ICIS-specific
/// **year-end non-publishing week**, whose span drifts year to year
/// (maintained as a DateList — see
/// [`holidays::icis_europe_chemicals_year_end`]).
///
/// The sibling "Gasoline, Naphtha, Crude" daily family has a *shorter*
/// year-end closure until the families merge in 2026 — a separate calendar
/// if/when needed.
///
/// # Verification
///
/// Built from the official ICIS annual publishing schedules 2022–2026.
/// Caveat: mixed schedule vintages — the 2023 edition is v1a (Dec 2022,
/// pre-year), so in-year 2023 changes may be missing; not yet verified
/// against settlement data.
///
/// Valid range: 2022–2026 (schedule-covered era only; extend per year).
/// Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::icis_europe_chemicals;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = icis_europe_chemicals();
/// assert!(cal.is_holiday(date(2024, 12, 30)).unwrap());       // year-end week
/// assert!(cal.is_holiday(date(2026, 1, 2)).unwrap());         // 2025 week runs to 2 Jan
/// assert!(cal.is_business_day(date(2024, 12, 23)).unwrap());  // week starts 24 Dec 2024
/// assert!(cal.is_holiday(date(2023, 5, 1)).unwrap());        // UK Early May BH
/// assert!(cal.is_business_day(date(2023, 11, 1)).unwrap());   // no continental holidays
/// assert!(cal.is_holiday(date(2021, 12, 25)).is_err());       // before schedule era
/// ```
pub fn icis_europe_chemicals() -> RuleBasedCalendar {
    let mut rules = uk_base_rules();
    rules.push(holidays::icis_europe_chemicals_year_end());
    RuleBasedCalendar::builder("ICISEuropeChemicals")
        .with_description(
            "ICIS Europe daily chemicals assessments (Aromatics publication family, e.g. \
             benzene) — UK working days + ICIS year-end non-publishing weeks; the \
             Gasoline/Naphtha/Crude family closes less at year-end until 2026",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2022, 2026)
        .with_rules(rules)
        .build()
        .expect("ICISEuropeChemicals calendar: rules cover requested range")
}

/// Returns the ICIS China Chemicals [`RuleBasedCalendar`].
///
/// Publication days of ICIS's **China-CN daily chemicals reports** ("All
/// dailies", region China-CN in the ICIS annual publishing schedules).
/// These follow the **Chinese public-holiday gazette** — New Year, the
/// multi-day Lunar New Year and Golden Week spans (incl. makeup-extended
/// weeks), Qing Ming, Labour Day, Dragon Boat, Mid-Autumn — all as
/// per-festival DateLists since the spans drift with the lunar calendar and
/// State Council decisions. Singapore and UK holidays are **publication
/// days** (the schedules mark them "Published — US dollar prices not
/// assessed").
///
/// **Weekends are explicit** (the usual model for such vendor lists): the implicit
/// weekend convention is disabled and every Saturday/Sunday is a holiday
/// named `"Weekend"` — except the gazette's makeup working weekends, on
/// which most China-CN dailies publish (only MEG/PTA explicitly don't; a
/// weekday-only variant can be added for those). Makeup days and holiday
/// spans are gazette-sourced for all years incl. 2026 — see
/// [`holidays::icis_cn_weekend`].
///
/// # Verification
///
/// Built from the official ICIS annual publishing schedules 2022–2026; same
/// vintage caveat as the Europe calendar. Not yet verified against
/// settlement data. 2027 is taken from the State Council
/// gazette alone (国办发明电〔2026〕10号, 2026-11-05) — the ICIS 2027
/// publishing schedule was not yet released; re-check when it is.
///
/// Valid range: 2022–2027. Time zone: Asia/Shanghai.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::icis_china_chemicals;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = icis_china_chemicals();
/// assert!(cal.is_holiday(date(2026, 2, 23)).unwrap());       // LNY makeup Monday
/// assert!(cal.is_holiday(date(2025, 10, 8)).unwrap());       // Golden Week 2025
/// assert!(cal.is_business_day(date(2026, 4, 3)).unwrap());   // Good Friday publishes
/// assert!(cal.is_business_day(date(2025, 12, 25)).unwrap()); // Christmas publishes
/// // Weekends are explicit holidays ("Weekend") except gazette makeup days:
/// assert!(cal.is_holiday(date(2025, 1, 25)).unwrap());        // ordinary Saturday
/// assert!(cal.is_business_day(date(2025, 2, 8)).unwrap());    // makeup working Saturday
/// ```
pub fn icis_china_chemicals() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("ICISChinaChemicals")
        .with_description(
            "ICIS China-CN daily chemicals reports — Chinese public-holiday gazette (multi-day \
             CNY/Golden Week spans) with explicit \"Weekend\" holidays except gazette makeup \
             working weekends; SG/UK holidays publish (partial content)",
        )
        .with_tz(TimeZone::get("Asia/Shanghai").expect("Asia/Shanghai timezone"))
        .with_valid_range(2022, 2027)
        .with_weekends(&[])
        .with_rules(vec![
            holidays::icis_cn_weekend(),
            holidays::china_new_year(),
            holidays::china_lunar_new_year(),
            holidays::china_qingming(),
            holidays::china_labour_day(),
            holidays::china_dragon_boat(),
            holidays::china_mid_autumn(),
            holidays::china_golden_week(),
        ])
        .build()
        .expect("ICISChinaChemicals calendar: rules cover requested range")
}

/// Returns the China Futures Exchanges [`RuleBasedCalendar`].
///
/// Day-session closure calendar of the mainland-China futures exchanges —
/// ZCE, DCE, SHFE, CFFEX, INE (the stock exchanges close identically).
/// The exchanges follow the State Council public-holiday gazette exactly:
/// closed for the full gazette holiday spans, and closed on **all**
/// weekends — including the gazette makeup working weekends (调休), on
/// which offices work but exchanges never trade. That is the one
/// difference from [`icis_china_chemicals`], which shares the same seven
/// gazette DateLists but publishes on makeup weekends.
///
/// The **interbank FX market / CNY central parity (CFETS fixing) days
/// follow the same convention**: no fixing is published on gazette makeup
/// weekends (verified in the PBOC parity archive — e.g. none on Sat
/// 2026-05-09) and the 2024-02-09 CNY-eve closure was market-wide, so this
/// calendar also serves onshore CNY FX fixing days. Note the interbank
/// money/bond markets differ: CFETS〔2025〕461号 lists the makeup weekends
/// as normal trading days there — do not use this calendar (nor
/// [`icis_china_chemicals`], which differs on 2024-02-09) for interbank
/// bond/repo series without checking that day.
///
/// # Verification
///
/// Holiday spans: State Council gazettes 2022–2027 (2026:
/// 国办发明电〔2025〕7号; 2027: 国办发明电〔2026〕10号, 2026-11-05),
/// data-verified via the toluene East China series. The exchange
/// convention (gazette spans + all weekends closed) confirmed against the
/// DCE 2026 Spring Festival trading-arrangement notice: closed
/// 15–23 Feb 2026, reopening 24 Feb, with the makeup Saturdays 14 and
/// 28 Feb explicitly listed as weekend closures. Data-verified
/// (2026-07-15) against the ZCE methanol settlement series
/// (2022-01 → 2026-07): no values on any calendar holiday — including
/// all 31 gazette makeup weekends and the 2024-02-09 CNY-eve closure
/// (exchanges closed beyond the gazette that year, per the Dec-2023
/// notices; [`holidays::china_exchanges_cny_eve`]).
pub fn china_futures_exchanges() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("ChinaFuturesExchanges")
        .with_description(
            "Mainland China futures exchanges day-session closures (ZCE/DCE/SHFE/CFFEX/INE; \
             stock exchanges and CNY FX-fixing days match) — State Council gazette holiday \
             spans; closed on all weekends incl. gazette makeup working weekends (exchanges \
             never trade weekends; interbank money/bond markets DO trade those days)",
        )
        .with_tz(TimeZone::get("Asia/Shanghai").expect("Asia/Shanghai timezone"))
        .with_valid_range(2022, 2027)
        .with_rules(vec![
            holidays::china_new_year(),
            holidays::china_lunar_new_year(),
            holidays::china_qingming(),
            holidays::china_labour_day(),
            holidays::china_dragon_boat(),
            holidays::china_mid_autumn(),
            holidays::china_golden_week(),
            holidays::china_exchanges_cny_eve(),
        ])
        .build()
        .expect("ChinaFuturesExchanges calendar: rules cover requested range")
}

/// Returns the OPIS PetroChem Wire [`RuleBasedCalendar`].
///
/// Publication days of OPIS's **US daily petrochemical** reports — the
/// PetroChem Wire Daily/Midday Reports, North America Aromatics Daily
/// Report, PetroChemical Forwards Curves, and Refinery/Propylene Focus
/// Report all share one schedule per the official OPIS holiday publish
/// workbooks. US pricing holidays with federal observance + Good Friday +
/// Juneteenth, plus the one-off Fri 2025-12-26 (shared with Platts US —
/// industry-wide that year). Columbus Day and Veterans Day publish.
///
/// Date-equivalent to [`platts_us_marketscan`] over the verified window —
/// the two publishers run the same US petchem schedule.
///
/// **Valid range 2025–2027**: only the 2025+2026 OPIS schedules are
/// available; earlier behaviour (Platts-style July-4 bridge days? the 2018
/// year-end closure? Juneteenth adoption year?) is unverified, and the
/// observed data are too patchy to settle it. Extend when more schedule years
/// land — and check the bridge-day question when the 2028 schedule appears
/// (July 4 2028 falls on a Tuesday).
///
/// # Verification
///
/// Official OPIS holiday publish schedules 2025–2026: exact match, 23/23
/// weekday days (plus 2027-01-01), across all five US daily petchem
/// publications.
///
/// Time zone: America/New_York.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::opis_petrochem_wire;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = opis_petrochem_wire();
/// assert!(cal.is_holiday(date(2026, 7, 3)).unwrap());        // July 4 observed
/// assert!(cal.is_holiday(date(2025, 12, 26)).unwrap());      // one-off
/// assert!(cal.is_business_day(date(2025, 10, 13)).unwrap()); // Columbus Day publishes
/// assert!(cal.is_holiday(date(2024, 12, 25)).is_err());      // before verified range
/// ```
pub fn opis_petrochem_wire() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("OPISPetroChemWire")
        .with_description(
            "OPIS US daily petrochemical reports (PetroChem Wire Daily/Midday, North America \
             Aromatics Daily, Forwards Curves, Refinery/Propylene Focus) — date-equivalent to \
             PlattsUSMarketScan over the verified window",
        )
        .with_tz(TimeZone::get("America/New_York").expect("America/New_York timezone"))
        .with_valid_range(2025, 2027)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::us_new_years_day_observed(),
            holidays::mlk_day(),
            holidays::presidents_day(),
            holidays::good_friday(),
            holidays::memorial_day(),
            holidays::juneteenth(),
            holidays::us_juneteenth_observed(),
            holidays::independence_day(),
            holidays::us_independence_day_observed(),
            holidays::labor_day(),
            holidays::thanksgiving(),
            holidays::day_after_thanksgiving(),
            holidays::christmas_day(),
            holidays::us_christmas_day_observed(),
            holidays::opis_day_after_christmas_2025(),
        ])
        .build()
        .expect("OPISPetroChemWire calendar: rules cover requested range")
}

/// Returns the Argus International LPG (Asia delivery) [`RuleBasedCalendar`].
///
/// Models the publication schedule of Argus's Asia/Singapore-delivered LPG
/// products — including the Argus Far East Index (AFEI) Propane and Butane,
/// Argus cfr Ningbo, and related East-Asia LPG assessments published in the
/// **Argus International LPG** report.
///
/// The Asia-delivery LPG calendar follows **Singapore gazetted public
/// holidays** with Sun→Mon observance (Saturday holidays are not
/// substituted). **Scope caveat**: Argus International LPG bundles many
/// regional series (US Mont Belvieu, Russia daf Brest, China, …), each on its
/// own regional calendar — this calendar applies to the Asia/Singapore subset
/// only.
///
/// Date-equivalent to [`platts_asia_marketscan`]: Argus also closed on the
/// 2018 year-end Mondays (24/31 Dec 2018 — data-evidenced, see
/// [`holidays::argus_year_end_2018`]), carried here as the Argus-provenance
/// rule rather than the Platts one. Kept as a separate constructor so the
/// two publishers can diverge without coupling.
///
/// # Verification
///
/// - Official Argus Media Publishing Schedule workbooks 2020–2026 (Argus
///   International LPG rows): exact match, 69/69 weekday holidays.
/// - Observed AFEI Propane settlement data (2020-10 to 2025-12): 100% fit
///   (two capture gaps in the data aside).
///
/// Valid range: 2010–2027 (constrained by date-list rules).
/// Time zone: Asia/Singapore.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::argus_asia_lpg;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = argus_asia_lpg();
/// // Singapore gazetted holidays — confirmed in Argus 2026 publishing schedule
/// assert!(cal.is_holiday(date(2026, 1, 1)).unwrap());   // New Year (Thu)
/// assert!(cal.is_holiday(date(2026, 2, 17)).unwrap());  // Chinese New Year day 1
/// assert!(cal.is_holiday(date(2026, 5, 27)).unwrap());  // Hari Raya Haji
/// assert!(cal.is_holiday(date(2026, 8, 10)).unwrap());  // National Day observed (9 Aug Sun)
/// // Per Argus 2026 schedule, Asia LPG publishes on NYE (only EE excluded)
/// assert!(cal.is_business_day(date(2026, 12, 31)).unwrap());
/// // 2023 Presidential Election polling day — verified against historical AFEI gap
/// assert!(cal.is_holiday(date(2023, 9, 1)).unwrap());
/// ```
pub fn argus_asia_lpg() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("ArgusAsiaLPG")
        .with_description(
            "Argus Asia/Far-East LPG assessments (AFEI Propane/Butane, cfr Ningbo) — Asia \
             subset of Argus International LPG only, not its US/Europe/Russia series",
        )
        .with_tz(TimeZone::get("Asia/Singapore").expect("Asia/Singapore timezone"))
        .with_valid_range(2010, 2027)
        .with_rules(vec![
            // Shared algorithmic rules
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::labour_day(),
            holidays::christmas_day(),
            // SG observance (Sun→Mon only, Saturday ignored) — shared with Platts
            holidays::sg_sunmon_new_years_day_observed(),
            holidays::sg_sunmon_labour_day_observed(),
            holidays::sg_national_day(),
            holidays::sg_sunmon_national_day_observed(),
            holidays::sg_christmas_day_observed(),
            // DateList holidays
            holidays::sg_chinese_new_year(),
            holidays::sg_hari_raya_puasa(),
            holidays::sg_hari_raya_haji(),
            holidays::sg_vesak_day(),
            holidays::sg_deepavali(),
            // One-off election holidays
            holidays::sg_polling_day(),
            // Singapore one-off public holiday — SG50 (7 Aug 2015)
            holidays::sg50_public_holiday(),
            // Argus 2018 year-end closure (Dec 24/31) — see rule docs
            holidays::argus_year_end_2018(),
        ])
        .build()
        .expect("ArgusAsiaLPG calendar: rules cover requested range")
}

/// Returns the Argus International LPG (Europe delivery) [`RuleBasedCalendar`].
///
/// Models the publication schedule of Argus's Europe-assessed LPG series —
/// propane/butane NWE cif large cargoes and related northwest-Europe LPG
/// assessments published in the **Argus International LPG** report out of
/// London. **Scope caveat**: the report bundles many regional series (Asia,
/// US Mont Belvieu, Russia daf Brest, China, …), each on its own regional
/// calendar — this calendar applies to the Europe/UK-assessed subset only
/// (see [`argus_asia_lpg`] for the Asia subset).
///
/// [`uk_settlement`]'s rules (UK bank holidays incl. proclaimed one-offs)
/// plus the [`Argus 2018 year-end closure`](holidays::argus_year_end_2018):
/// Argus assesses Europe LPG on every London business day, publishing
/// through continental holidays (e.g. May Day) and — 2018 aside — on the
/// Dec 24 / Dec 31 half-days.
///
/// # Verification
///
/// - Official Argus Media Publishing Schedule workbooks 2020–2026 (Argus
///   International LPG rows; Europe off = full suspension or any "No UK
///   data" variant): exact match, 59/59 weekday holidays.
/// - Observed settlement data: Argus Propane CIF ARA large cargo
///   (2007-02 → 2026) matches 100.0%, modulo a known 2022-03-17/18 capture
///   gap in one of the two data sources.
///
/// Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::argus_europe_lpg;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = argus_europe_lpg();
/// assert!(cal.is_holiday(date(2023, 5, 8)).unwrap());   // Coronation
/// assert!(cal.is_holiday(date(2022, 9, 19)).unwrap());  // State Funeral
/// assert!(cal.is_holiday(date(2026, 12, 28)).unwrap()); // Boxing Day observed
/// // Argus publishes the Dec 24 / Dec 31 half-days and continental May Day
/// assert!(cal.is_business_day(date(2024, 12, 24)).unwrap());
/// assert!(cal.is_business_day(date(2024, 12, 31)).unwrap());
/// assert!(cal.is_business_day(date(2024, 5, 1)).unwrap());
/// // …except the 2018 year-end closure (data-evidenced)
/// assert!(cal.is_holiday(date(2018, 12, 24)).unwrap());
/// assert!(cal.is_holiday(date(2018, 12, 31)).unwrap());
/// ```
pub fn argus_europe_lpg() -> RuleBasedCalendar {
    let mut rules = uk_base_rules();
    rules.push(holidays::argus_year_end_2018());
    RuleBasedCalendar::builder("ArgusEuropeLPG")
        .with_description(
            "Argus Europe LPG assessments (propane/butane NWE cif etc.) — Europe/UK-assessed \
             subset of Argus International LPG only; UKSettlement + 2018 year-end closure",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(rules)
        .build()
        .expect("ArgusEuropeLPG calendar: rules cover requested range")
}

/// Returns the ICE UK averaging-days [`RuleBasedCalendar`].
///
/// The days that count as a fixing in an ICE Futures Europe monthly average
/// (e.g. Brent 1st Line, average price options): [`ice_uk`] plus the Monday
/// after a Sunday New Year. On that Monday ICE publishes a settlement record
/// — so [`ice_uk`], the publication calendar, has it as a business day — but
/// every price is carried over and the day is left out of the average.
/// The two calendars differ on 14 days in 2000–2099 (2 January 2006, 2012,
/// 2017, 2023, 2034, …).
///
/// # Verification
///
/// One instance in ICE's end-of-day settlement files: on Mon 2023-01-02 all
/// Brent, WTI, Henry and Brent 1st Line strips have a zero change, and ICE's
/// January 2023 Brent 1st Line final settlement, 83.953, is the average of
/// the front-month Brent settlements over the 21 days without 2 January
/// (83.9529; with it, over 22 days: 84.0418).
///
/// Time zone: Europe/London.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::{ice_uk, ice_uk_averaging};
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = ice_uk_averaging();
/// assert!(cal.is_holiday(date(2023, 1, 2)).unwrap());        // carried prices, no fixing
/// assert!(ice_uk().is_business_day(date(2023, 1, 2)).unwrap()); // … but a record is published
/// assert!(cal.is_business_day(date(2022, 1, 3)).unwrap());   // New Year on Saturday: no shift
/// ```
pub fn ice_uk_averaging() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("ICEUKAveraging")
        .with_description(
            "ICE Futures Europe fixing days for monthly averages (Brent 1st Line, APOs) — \
             ICEUK plus the Monday after a Sunday New Year, when prices are only carried over",
        )
        .with_tz(TimeZone::get("Europe/London").expect("Europe/London timezone"))
        .with_valid_range(2000, 2099)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::ice_new_years_day_observed(),
            holidays::good_friday(),
            holidays::christmas_day(),
            holidays::ice_christmas_day_observed(),
        ])
        .build()
        .expect("ICEUKAveraging calendar: rules cover requested range")
}

/// Returns the ICE Endex [`RuleBasedCalendar`].
///
/// The **settlement days** of ICE Endex natural gas futures — Dutch TTF,
/// German THE, Italian PSV, Austrian CEGH VTP and French PEG: the days a
/// settlement series has a new value and a monthly average (ICE's TTF 1st
/// Line, an APO on it) takes a fixing. Five days are closed: New Year's
/// Day, Good Friday, Easter Monday, Christmas Day and 26 December. None is
/// moved when it falls on a weekend — Mon 2 Jan 2023 and Tue 27 Dec 2022
/// settled normally — and the UK bank holidays and 1 May settle.
///
/// This is not ICE Futures Europe's calendar: [`ice_uk`] and
/// [`ice_uk_averaging`] (Brent) are open on Easter Monday and 26 December,
/// and ICE's UK NBP gas future follows the England and Wales bank holidays
/// ([`uk_settlement`]). Nor is it the calendar the TTF **expiry rules**
/// count in: a TTF future stops trading two *UK* business days before its
/// delivery month ([`uk_settlement`]).
///
/// # Verification
///
/// ICE's end-of-day settlement files. On Easter Monday and 26 December ICE
/// publishes a record with every price carried over; on the other three
/// days it publishes none. TTF, 2018-04-26 to 2026-11-18: the 34 weekdays
/// without a new settlement (21 without a record, 13 carried) are exactly
/// this calendar's holidays. THE, PSV and CEGH VTP (from February 2021) and
/// PEG (from June 2022) are closed on exactly the same days. ICE's final
/// settlement of its TTF 1st Line contract for May 2026 is the average of
/// the front-month settlements without the carried Easter Monday, 6 April.
/// Belgian ZTP and ICE Endex's non-gas contracts are not verified.
///
/// Time zone: Europe/Amsterdam. Valid from 2018, the first year in the data.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::{ice_endex, ice_uk_averaging};
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = ice_endex();
/// assert!(cal.is_holiday(date(2026, 4, 6)).unwrap());            // Easter Monday: prices carried
/// assert!(cal.is_holiday(date(2025, 12, 26)).unwrap());          // 26 December
/// assert!(ice_uk_averaging().is_business_day(date(2026, 4, 6)).unwrap()); // … Brent fixes
/// assert!(cal.is_business_day(date(2023, 1, 2)).unwrap());       // Sunday New Year: no shift
/// assert!(cal.is_business_day(date(2026, 5, 1)).unwrap());       // 1 May settles
/// assert!(cal.is_business_day(date(2026, 8, 31)).unwrap());      // UK bank holiday settles
/// ```
pub fn ice_endex() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("ICEEndex")
        .with_description(
            "ICE Endex gas futures settlement days (TTF, THE, PSV, CEGH VTP, PEG) — closed \
             New Year, Good Friday, Easter Monday, 25 and 26 December, no weekend observance",
        )
        .with_tz(TimeZone::get("Europe/Amsterdam").expect("Europe/Amsterdam timezone"))
        .with_valid_range(2018, 2099)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::good_friday(),
            holidays::easter_monday(),
            holidays::christmas_day(),
            holidays::st_stephens_day(),
        ])
        .build()
        .expect("ICEEndex calendar: rules cover requested range")
}

/// Returns the NYMEX (CME Group energy) [`RuleBasedCalendar`].
///
/// The **settlement days** of NYMEX energy futures — WTI (CL) and Henry Hub
/// natural gas (NG, and the financial HH): the "business days" the contract
/// rules count in and the days a settlement series has a value. On ten US
/// holidays CME does not derive settlement prices: New Year's Day, MLK Day,
/// Presidents' Day, Good Friday, Memorial Day, Juneteenth (**from 2022**),
/// Independence Day, Labor Day, Thanksgiving and Christmas Day. On most of
/// them CME Globex still trades a shortened session, dated the next business
/// day — this calendar models settlement, not trading hours.
///
/// Observance is the US federal one (Sat → Fri, Sun → Mon) except for New
/// Year's Day: Sunday → Monday only, Friday Dec 31 stays a settlement day.
/// Columbus Day, Veterans Day and the day after Thanksgiving are settlement
/// days.
///
/// The usual expiry rules are a step on this calendar
/// ([`add_business_days`](crate::HolidayCalendar::add_business_days)):
/// Henry Hub — the third business day before the first day of the contract
/// month (NYMEX Rule 220102.F); WTI — three business days before the 25th
/// of the month before the contract month, or before the last business day
/// preceding the 25th if the 25th is not a business day (Rule 200102.F).
/// Listed expiry
/// dates stay in effect when the holiday schedule changes later, so prefer
/// the exchange's listed date where one is at hand.
///
/// # Verification
///
/// Against ICE's end-of-day settlement files, 2021-02-01 to 2026-11-10: the
/// weekdays on which ICE's Henry LD1 future (`H`, settled on NYMEX NG)
/// either has no record (13) or carries every strip unchanged (41) are
/// exactly this calendar's holidays, and the expiry rules above reproduce
/// ICE's expiry dates for all 215 Henry strips and — one day earlier — all
/// 173 WTI (`T`) strips.
///
/// Valid range: 2021–2099 — earlier years are not verified (one-off
/// closures such as days of mourning or Hurricane Sandy are not modelled).
/// Time zone: America/New_York.
///
/// # Examples
///
/// ```
/// use fincal::builtin::calendars::nymex;
/// use fincal::HolidayCalendar;
/// use jiff::civil::date;
///
/// let cal = nymex();
/// assert!(cal.is_holiday(date(2025, 1, 20)).unwrap());        // MLK Day: no settlement
/// assert!(cal.is_holiday(date(2025, 4, 18)).unwrap());        // Good Friday
/// assert!(cal.is_holiday(date(2021, 12, 24)).unwrap());       // Christmas on Saturday → Friday
/// assert!(cal.is_business_day(date(2021, 12, 31)).unwrap());  // New Year on Saturday: no shift
/// assert!(cal.is_business_day(date(2021, 6, 18)).unwrap());   // no Juneteenth before 2022
/// assert!(cal.is_business_day(date(2025, 11, 28)).unwrap());  // day after Thanksgiving
/// assert!(cal.is_business_day(date(2025, 10, 13)).unwrap());  // Columbus Day
///
/// // WTI (CL) July 2022: the 25th is a Saturday → three business days
/// // before Fri 24 June
/// let anchor = cal.prev_business_day(date(2022, 6, 26)).unwrap();
/// assert_eq!(cal.add_business_days(anchor, -3).unwrap(), date(2022, 6, 21));
/// ```
pub fn nymex() -> RuleBasedCalendar {
    RuleBasedCalendar::builder("NYMEX")
        .with_description(
            "NYMEX (CME Group) energy settlement days — WTI (CL), Henry Hub (NG/HH): ten US \
             holidays without settlement prices; New Year Sun→Mon only, Juneteenth from 2022",
        )
        .with_tz(TimeZone::get("America/New_York").expect("America/New_York timezone"))
        .with_valid_range(2021, 2099)
        .with_rules(vec![
            holidays::new_years_day(),
            holidays::cme_new_years_day_observed(),
            holidays::mlk_day(),
            holidays::presidents_day(),
            holidays::good_friday(),
            holidays::memorial_day(),
            holidays::cme_juneteenth(),
            holidays::cme_juneteenth_observed(),
            holidays::independence_day(),
            holidays::us_independence_day_observed(),
            holidays::labor_day(),
            holidays::thanksgiving(),
            holidays::christmas_day(),
            holidays::us_christmas_day_observed(),
        ])
        .build()
        .expect("NYMEX calendar: rules cover requested range")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HolidayCalendar;
    use jiff::civil::date;

    // --- TARGET tests ---

    #[test]
    fn target_2025_holidays() {
        let cal = target();
        let expected = [
            (date(2025, 1, 1), "New Year's Day"),
            (date(2025, 4, 18), "Good Friday"),
            (date(2025, 4, 21), "Easter Monday"),
            (date(2025, 5, 1), "Labour Day"),
            (date(2025, 12, 25), "Christmas Day"),
            (date(2025, 12, 26), "St. Stephen's Day"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be a holiday"
            );
            assert!(
                !cal.is_business_day(d).unwrap(),
                "{name} ({d}) should not be a business day"
            );
        }
    }

    // --- EEX Power Derivatives tests ---

    #[test]
    fn eex_power_2025_holidays() {
        let cal = eex_power_derivatives();
        let expected = [
            (date(2025, 1, 1), "New Year"),
            (date(2025, 4, 18), "Good Friday"),
            (date(2025, 4, 21), "Easter Monday"),
            (date(2025, 5, 1), "Labour Day"),
            (date(2025, 12, 24), "Christmas Eve"),
            (date(2025, 12, 25), "Christmas Day"),
            (date(2025, 12, 26), "Boxing Day"),
            (date(2025, 12, 31), "New Year's Eve"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be an EEX Power holiday"
            );
            assert!(
                !cal.is_business_day(d).unwrap(),
                "{name} ({d}) should not be an EEX Power business day"
            );
        }
    }

    #[test]
    fn eex_power_business_days() {
        let cal = eex_power_derivatives();
        // Ordinary weekdays trade.
        assert!(cal.is_business_day(date(2025, 5, 2)).unwrap());
        assert!(cal.is_business_day(date(2025, 12, 23)).unwrap());
        assert!(cal.is_business_day(date(2025, 12, 30)).unwrap());
        // EEX has no May/Aug bank holidays and no observance shifts: when a
        // fixed holiday lands on a weekend there is no Monday-in-lieu.
        // 2027-12-26 (Boxing Day) is a Sunday → the following Monday trades.
        assert!(cal.is_business_day(date(2027, 12, 27)).unwrap());
    }

    #[test]
    fn target_2024_holidays() {
        let cal = target();
        let expected = [
            date(2024, 1, 1),   // New Year
            date(2024, 3, 29),  // Good Friday
            date(2024, 4, 1),   // Easter Monday
            date(2024, 5, 1),   // Labour Day
            date(2024, 12, 25), // Christmas
            date(2024, 12, 26), // St. Stephen's
        ];
        for d in expected {
            assert!(cal.is_holiday(d).unwrap(), "{d} should be a TARGET holiday");
        }
    }

    #[test]
    fn target_2023_holidays() {
        let cal = target();
        let expected = [
            date(2023, 4, 7),   // Good Friday
            date(2023, 4, 10),  // Easter Monday
            date(2023, 5, 1),   // Labour Day
            date(2023, 12, 25), // Christmas
            date(2023, 12, 26), // St. Stephen's
        ];
        for d in expected {
            assert!(cal.is_holiday(d).unwrap(), "{d} should be a TARGET holiday");
        }
        // Jan 1, 2023 is Sunday — not counted as holiday (weekend)
        assert!(!cal.is_holiday(date(2023, 1, 1)).unwrap());
    }

    #[test]
    fn target_regular_business_day() {
        let cal = target();
        assert!(cal.is_business_day(date(2025, 6, 11)).unwrap());
    }

    #[test]
    fn target_weekend_not_business_day() {
        let cal = target();
        assert!(!cal.is_business_day(date(2025, 6, 14)).unwrap()); // Saturday
        assert!(!cal.is_business_day(date(2025, 6, 15)).unwrap()); // Sunday
    }

    #[test]
    fn target_name_and_tz() {
        let cal = target();
        assert_eq!(cal.name(), "TARGET");
        assert_eq!(*cal.tz(), TimeZone::get("Europe/Berlin").unwrap());
    }

    #[test]
    fn target_rule_count() {
        let cal = target();
        assert_eq!(cal.rules().len(), 6);
    }

    #[test]
    fn target_2020_to_2026_holiday_counts() {
        let cal = target();
        for year in 2020..=2026 {
            let from = jiff::civil::Date::new(year, 1, 1).unwrap();
            let to = jiff::civil::Date::new(year, 12, 31).unwrap();
            let holidays = cal.holidays_between(from, to).unwrap();
            assert!(
                !holidays.is_empty() && holidays.len() <= 6,
                "year {year}: expected 1-6 holidays, got {}",
                holidays.len()
            );
        }
    }

    // --- US Settlement tests ---

    #[test]
    fn us_2025_holidays() {
        let cal = us_settlement();
        let expected = [
            (date(2025, 1, 1), "New Year's Day"),
            (date(2025, 1, 20), "MLK Day"),
            (date(2025, 2, 17), "Presidents' Day"),
            (date(2025, 5, 26), "Memorial Day"),
            (date(2025, 6, 19), "Juneteenth"),
            (date(2025, 7, 4), "Independence Day"),
            (date(2025, 9, 1), "Labor Day"),
            (date(2025, 10, 13), "Columbus Day"),
            (date(2025, 11, 11), "Veterans Day"),
            (date(2025, 11, 27), "Thanksgiving"),
            (date(2025, 12, 25), "Christmas Day"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be a US holiday"
            );
        }
    }

    #[test]
    fn us_2024_holidays() {
        let cal = us_settlement();
        let expected = [
            date(2024, 1, 1),   // New Year's Day
            date(2024, 1, 15),  // MLK Day
            date(2024, 2, 19),  // Presidents' Day
            date(2024, 5, 27),  // Memorial Day
            date(2024, 6, 19),  // Juneteenth (Wednesday)
            date(2024, 7, 4),   // Independence Day (Thursday)
            date(2024, 9, 2),   // Labor Day
            date(2024, 10, 14), // Columbus Day
            date(2024, 11, 11), // Veterans Day
            date(2024, 11, 28), // Thanksgiving
            date(2024, 12, 25), // Christmas (Wednesday)
        ];
        for d in expected {
            assert!(cal.is_holiday(d).unwrap(), "{d} should be a US holiday");
        }
    }

    #[test]
    fn us_observed_saturday_independence_day() {
        let cal = us_settlement();
        // 2020: July 4 is Saturday -> observed Friday July 3
        assert!(cal.is_holiday(date(2020, 7, 3)).unwrap());
        assert!(!cal.is_holiday(date(2020, 7, 4)).unwrap());
    }

    #[test]
    fn us_observed_sunday_independence_day() {
        let cal = us_settlement();
        // 2021: July 4 is Sunday -> observed Monday July 5
        assert!(cal.is_holiday(date(2021, 7, 5)).unwrap());
    }

    #[test]
    fn us_juneteenth_not_before_2021() {
        let cal = us_settlement();
        assert!(!cal.is_holiday(date(2020, 6, 19)).unwrap());
        assert!(cal.is_holiday(date(2021, 6, 18)).unwrap());
    }

    #[test]
    fn us_observed_new_year_cross_year() {
        let cal = us_settlement();
        // 2022: Jan 1 is Saturday -> observed Fri Dec 31, 2021
        assert!(cal.is_holiday(date(2021, 12, 31)).unwrap());
    }

    #[test]
    fn us_2023_observed_new_year() {
        let cal = us_settlement();
        // 2023: Jan 1 is Sunday -> observed Mon Jan 2
        assert!(cal.is_holiday(date(2023, 1, 2)).unwrap());
    }

    #[test]
    fn us_regular_business_day() {
        let cal = us_settlement();
        assert!(cal.is_business_day(date(2025, 6, 11)).unwrap());
    }

    #[test]
    fn us_name_and_tz() {
        let cal = us_settlement();
        assert_eq!(cal.name(), "USSettlement");
        assert_eq!(*cal.tz(), TimeZone::get("America/New_York").unwrap());
    }

    #[test]
    fn us_2020_to_2026_holiday_counts() {
        let cal = us_settlement();
        for year in 2020..=2026 {
            let from = jiff::civil::Date::new(year, 1, 1).unwrap();
            let to = jiff::civil::Date::new(year, 12, 31).unwrap();
            let holidays = cal.holidays_between(from, to).unwrap();
            assert!(
                holidays.len() >= 10 && holidays.len() <= 12,
                "year {year}: expected 10-12 holidays, got {} {:?}",
                holidays.len(),
                holidays
            );
        }
    }

    #[test]
    fn us_2026_holidays() {
        let cal = us_settlement();
        let expected = [
            date(2026, 1, 1),   // New Year's Day (Thursday)
            date(2026, 1, 19),  // MLK Day
            date(2026, 2, 16),  // Presidents' Day
            date(2026, 5, 25),  // Memorial Day
            date(2026, 6, 19),  // Juneteenth (Friday)
            date(2026, 7, 3),   // Independence Day observed (Jul 4 = Sat -> Fri Jul 3)
            date(2026, 9, 7),   // Labor Day
            date(2026, 10, 12), // Columbus Day
            date(2026, 11, 11), // Veterans Day (Wednesday)
            date(2026, 11, 26), // Thanksgiving
            date(2026, 12, 25), // Christmas (Friday)
        ];
        for d in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{d} should be a US holiday in 2026"
            );
        }
    }

    // --- UK Settlement tests ---

    #[test]
    fn uk_2025_holidays() {
        let cal = uk_settlement();
        let expected = [
            (date(2025, 1, 1), "New Year's Day"),
            (date(2025, 4, 18), "Good Friday"),
            (date(2025, 4, 21), "Easter Monday"),
            (date(2025, 5, 5), "Early May Bank Holiday"),
            (date(2025, 5, 26), "Spring Bank Holiday"),
            (date(2025, 8, 25), "Summer Bank Holiday"),
            (date(2025, 12, 25), "Christmas Day"),
            (date(2025, 12, 26), "Boxing Day"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be a UK holiday"
            );
        }
    }

    #[test]
    fn uk_2024_holidays() {
        let cal = uk_settlement();
        let expected = [
            date(2024, 1, 1),   // New Year's Day (Monday)
            date(2024, 3, 29),  // Good Friday
            date(2024, 4, 1),   // Easter Monday
            date(2024, 5, 6),   // Early May Bank Holiday
            date(2024, 5, 27),  // Spring Bank Holiday
            date(2024, 8, 26),  // Summer Bank Holiday
            date(2024, 12, 25), // Christmas (Wednesday)
            date(2024, 12, 26), // Boxing Day (Thursday)
        ];
        for d in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{d} should be a UK holiday in 2024"
            );
        }
    }

    #[test]
    fn uk_christmas_cascade_2021() {
        let cal = uk_settlement();
        // 2021: Dec 25 = Saturday, Dec 26 = Sunday
        assert!(cal.is_holiday(date(2021, 12, 27)).unwrap());
        assert!(cal.is_holiday(date(2021, 12, 28)).unwrap());
        assert!(!cal.is_holiday(date(2021, 12, 25)).unwrap());
        assert!(!cal.is_holiday(date(2021, 12, 26)).unwrap());
    }

    #[test]
    fn uk_christmas_cascade_2022() {
        let cal = uk_settlement();
        // 2022: Dec 25 = Sunday, Dec 26 = Monday
        assert!(cal.is_holiday(date(2022, 12, 27)).unwrap()); // Christmas observed
        assert!(cal.is_holiday(date(2022, 12, 26)).unwrap()); // Boxing Day (Monday)
    }

    #[test]
    fn uk_new_year_observed_saturday() {
        let cal = uk_settlement();
        assert!(cal.is_holiday(date(2022, 1, 3)).unwrap());
    }

    #[test]
    fn uk_new_year_observed_sunday() {
        let cal = uk_settlement();
        assert!(cal.is_holiday(date(2023, 1, 2)).unwrap());
    }

    #[test]
    fn uk_regular_business_day() {
        let cal = uk_settlement();
        assert!(cal.is_business_day(date(2025, 6, 11)).unwrap());
    }

    #[test]
    fn uk_name_and_tz() {
        let cal = uk_settlement();
        assert_eq!(cal.name(), "UKSettlement");
        assert_eq!(*cal.tz(), TimeZone::get("Europe/London").unwrap());
    }

    #[test]
    fn uk_coronation_bank_holiday() {
        let cal = uk_settlement();
        assert!(cal.is_holiday(jiff::civil::date(2023, 5, 8)).unwrap());
        // Not a holiday in other years
        assert!(cal.is_business_day(jiff::civil::date(2025, 5, 8)).unwrap());
    }

    #[test]
    fn uk_queens_funeral() {
        let cal = uk_settlement();
        assert!(cal.is_holiday(jiff::civil::date(2022, 9, 19)).unwrap());
    }

    #[test]
    fn uk_spring_bank_holiday_moved_2002() {
        // Golden Jubilee: the late-May holiday moved to Tue 4 June, next to
        // the jubilee day on Mon 3 June — in every calendar on UK bank holidays.
        for cal in [
            uk_settlement(),
            platts_european_market_scan(),
            argus_europe_lpg(),
            baltic_lpg(),
            baltic_lpg1_hormuz(),
        ] {
            assert!(cal.is_business_day(date(2002, 5, 27)).unwrap());
            assert!(cal.is_holiday(date(2002, 6, 3)).unwrap());
            assert!(cal.is_holiday(date(2002, 6, 4)).unwrap());
        }
        // BalticUS: 27 May 2002 stays closed, as US Memorial Day.
        let cal = baltic_us();
        assert_eq!(
            cal.holiday_name(date(2002, 5, 27)).unwrap(),
            Some("Memorial Day")
        );
        assert!(cal.is_holiday(date(2002, 6, 4)).unwrap());
    }

    #[test]
    fn uk_queens_platinum_jubilee() {
        let cal = uk_settlement();
        assert!(cal.is_holiday(jiff::civil::date(2022, 6, 3)).unwrap());
    }

    #[test]
    fn uk_2020_to_2026_holiday_counts() {
        let cal = uk_settlement();
        for year in 2020..=2026 {
            let from = jiff::civil::Date::new(year, 1, 1).unwrap();
            let to = jiff::civil::Date::new(year, 12, 31).unwrap();
            let holidays = cal.holidays_between(from, to).unwrap();
            // 2022 has extra one-offs (Platinum Jubilee + Funeral = +2),
            // 2023 has Coronation (+1)
            let (min, max) = match year {
                2022 => (10, 12),
                2023 => (9, 11),
                _ => (8, 10),
            };
            assert!(
                holidays.len() >= min && holidays.len() <= max,
                "year {year}: expected {min}-{max} holidays, got {} {:?}",
                holidays.len(),
                holidays,
            );
        }
    }

    #[test]
    fn uk_2026_holidays() {
        let cal = uk_settlement();
        let expected = [
            date(2026, 1, 1),   // New Year's Day (Thursday)
            date(2026, 4, 3),   // Good Friday
            date(2026, 4, 6),   // Easter Monday
            date(2026, 5, 4),   // Early May Bank Holiday
            date(2026, 5, 25),  // Spring Bank Holiday
            date(2026, 8, 31),  // Summer Bank Holiday
            date(2026, 12, 25), // Christmas (Friday)
            date(2026, 12, 28), // Boxing Day observed (Dec 26 = Sat -> Mon 28)
        ];
        for d in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{d} should be a UK holiday in 2026"
            );
        }
    }

    // --- Platts European Market Scan tests ---

    #[test]
    fn platts_matches_uk_settlement() {
        let uk = uk_settlement();
        let platts = platts_european_market_scan();
        // Same holidays for a standard year
        let from = date(2025, 1, 1);
        let to = date(2025, 12, 31);
        let uk_hols = uk.holidays_between(from, to).unwrap();
        let platts_hols = platts.holidays_between(from, to).unwrap();
        assert_eq!(uk_hols, platts_hols);
    }

    #[test]
    fn platts_coronation() {
        let cal = platts_european_market_scan();
        assert!(cal.is_holiday(date(2023, 5, 8)).unwrap());
    }

    #[test]
    fn platts_name_and_tz() {
        let cal = platts_european_market_scan();
        assert_eq!(cal.name(), "PlattsEuropeanMarketScan");
        assert_eq!(*cal.tz(), TimeZone::get("Europe/London").unwrap());
    }

    // --- ICE UK tests ---

    #[test]
    fn ice_uk_2025_holidays() {
        let cal = ice_uk();
        let expected = [
            (date(2025, 1, 1), "New Year's Day"),
            (date(2025, 4, 18), "Good Friday"),
            (date(2025, 12, 25), "Christmas Day"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be an ICEUK holiday"
            );
        }
    }

    #[test]
    fn ice_uk_trades_through_uk_bank_holidays() {
        let cal = ice_uk();
        let uk_only = [
            (date(2025, 4, 21), "Easter Monday"),
            (date(2025, 5, 5), "Early May Bank Holiday"),
            (date(2025, 5, 26), "Spring Bank Holiday"),
            (date(2025, 8, 25), "Summer Bank Holiday"),
            (date(2025, 12, 26), "Boxing Day"),
        ];
        for (d, name) in uk_only {
            assert!(
                cal.is_business_day(d).unwrap(),
                "{name} ({d}) should be an ICEUK business day"
            );
        }
    }

    #[test]
    fn ice_uk_name_and_tz() {
        let cal = ice_uk();
        assert_eq!(cal.name(), "ICEUK");
        assert_eq!(*cal.tz(), TimeZone::get("Europe/London").unwrap());
    }

    #[test]
    fn ice_uk_rule_count() {
        let cal = ice_uk();
        assert_eq!(cal.rules().len(), 4);
    }

    #[test]
    fn ice_uk_christmas_observed_sunday_2022() {
        let cal = ice_uk();
        // 2022: Dec 25 = Sunday → observed Mon Dec 26
        assert!(cal.is_holiday(date(2022, 12, 26)).unwrap());
        // Dec 25 itself is weekend, not reported as holiday
        assert!(!cal.is_holiday(date(2022, 12, 25)).unwrap());
    }

    #[test]
    fn ice_uk_new_year_no_observance_sunday_2023() {
        let cal = ice_uk();
        // 2023: Jan 1 = Sunday → ICE publishes a record on Mon Jan 2 (no observance)
        assert!(cal.is_business_day(date(2023, 1, 2)).unwrap());
    }

    #[test]
    fn ice_uk_christmas_no_observance_saturday_2021() {
        let cal = ice_uk();
        // 2021: Dec 25 = Saturday → no shift, Fri Dec 24 is a business day
        assert!(cal.is_business_day(date(2021, 12, 24)).unwrap());
    }

    #[test]
    fn ice_uk_2020_to_2026_holiday_counts() {
        let cal = ice_uk();
        for year in 2020..=2026 {
            let from = jiff::civil::Date::new(year, 1, 1).unwrap();
            let to = jiff::civil::Date::new(year, 12, 31).unwrap();
            let holidays = cal.holidays_between(from, to).unwrap();
            // 3 holidays, but some may fall on weekends
            assert!(
                !holidays.is_empty() && holidays.len() <= 3,
                "year {year}: expected 1-3 holidays, got {} {:?}",
                holidays.len(),
                holidays
            );
        }
    }

    #[test]
    fn ice_uk_regular_business_day() {
        let cal = ice_uk();
        assert!(cal.is_business_day(date(2025, 6, 11)).unwrap());
    }

    // --- SGX tests ---

    #[test]
    fn sgx_2025_holidays() {
        let cal = sgx();
        let expected = [
            (date(2025, 1, 1), "New Year's Day"),
            (date(2025, 1, 29), "Chinese New Year Day 1"),
            (date(2025, 1, 30), "Chinese New Year Day 2"),
            (date(2025, 3, 31), "Hari Raya Puasa"),
            (date(2025, 4, 18), "Good Friday"),
            (date(2025, 5, 1), "Labour Day"),
            (date(2025, 5, 12), "Vesak Day"),
            // Hari Raya Haji Jun 7 falls on Saturday — no substitute
            (date(2025, 8, 11), "National Day (Observed, Aug 9 = Sat)"),
            (date(2025, 10, 20), "Deepavali"),
            (date(2025, 12, 25), "Christmas Day"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be an SGX holiday"
            );
        }
    }

    #[test]
    fn sgx_2027_holidays_match_mom_gazette() {
        // MOM press release "Public Holidays for 2027" (18 Jun 2026): every
        // gazetted weekday holiday plus the one substitute day (Mon 8 Feb for
        // CNY Day 2 on Sunday). Labour Day and Christmas fall on Saturdays;
        // the SGX fixed-date rules use `NextMonday`, so 3 May and 27 Dec are
        // additionally observed here — that convention is not asserted.
        let cal = sgx();
        for (d, name) in [
            (date(2027, 1, 1), "New Year's Day"),
            (date(2027, 2, 8), "CNY Day 2 (Sun) substitute"),
            (date(2027, 3, 10), "Hari Raya Puasa"),
            (date(2027, 3, 26), "Good Friday"),
            (date(2027, 5, 17), "Hari Raya Haji"),
            (date(2027, 5, 20), "Vesak Day"),
            (date(2027, 8, 9), "National Day"),
            (date(2027, 10, 28), "Deepavali"),
        ] {
            assert!(cal.is_holiday(d).unwrap(), "{name} ({d})");
        }
        assert!(
            cal.is_business_day(date(2027, 10, 29)).unwrap(),
            "superseded Deepavali projection"
        );
    }

    #[test]
    fn sgx_observed_national_day_2015() {
        let cal = sgx();
        // 2015: Aug 9 is Sunday → observed Mon Aug 10
        assert!(cal.is_holiday(date(2015, 8, 10)).unwrap());
        // Aug 9 itself is a weekend, not reported as holiday
        assert!(!cal.is_holiday(date(2015, 8, 9)).unwrap());
    }

    #[test]
    fn sgx_sg50_holiday() {
        let cal = sgx();
        assert!(cal.is_holiday(date(2015, 8, 7)).unwrap());
        // Not a holiday in other years
        assert!(!cal.is_holiday(date(2016, 8, 7)).unwrap());
    }

    #[test]
    fn sgx_chinese_new_year_substitute() {
        let cal = sgx();
        // 2023: CNY Jan 22 (Sun), 23 (Mon), 24 (Tue sub) — Mon and Tue are holidays
        assert!(cal.is_holiday(date(2023, 1, 23)).unwrap());
        assert!(cal.is_holiday(date(2023, 1, 24)).unwrap());
    }

    #[test]
    fn sgx_regular_business_day() {
        let cal = sgx();
        assert!(cal.is_business_day(date(2025, 6, 11)).unwrap());
    }

    #[test]
    fn sgx_name_and_tz() {
        let cal = sgx();
        assert_eq!(cal.name(), "SGX");
        assert_eq!(*cal.tz(), TimeZone::get("Asia/Singapore").unwrap());
    }

    #[test]
    fn sgx_2010_to_2026_holiday_counts() {
        let cal = sgx();
        for year in 2010..=2026 {
            let from = jiff::civil::Date::new(year, 1, 1).unwrap();
            let to = jiff::civil::Date::new(year, 12, 31).unwrap();
            let holidays = cal.holidays_between(from, to).unwrap();
            // Singapore has 11 gazetted holidays; with substitutes some years
            // have more weekday holidays, others fewer when holidays fall on
            // weekends. 2015 has SG50 extra.
            assert!(
                holidays.len() >= 9 && holidays.len() <= 14,
                "year {year}: expected 9-14 holidays, got {} {:?}",
                holidays.len(),
                holidays,
            );
        }
    }

    #[test]
    fn sgx_valid_range() {
        let cal = sgx();
        assert!(
            cal.is_business_day(date(2009, 6, 15)).is_err(),
            "SGX should reject dates before 2010"
        );
        assert!(
            cal.is_business_day(date(2028, 6, 15)).is_err(),
            "SGX should reject dates after 2027"
        );
    }

    // --- Baltic LPG tests ---

    #[test]
    fn baltic_lpg_2025_holidays() {
        let cal = baltic_lpg();
        let expected = [
            (date(2025, 1, 1), "New Year's Day"),
            (date(2025, 4, 18), "Good Friday"),
            (date(2025, 4, 21), "Easter Monday"),
            (date(2025, 5, 5), "Early May Bank Holiday"),
            (date(2025, 5, 26), "Spring Bank Holiday"),
            (date(2025, 8, 25), "Summer Bank Holiday"),
            (date(2025, 12, 25), "Christmas closure"),
            (date(2025, 12, 26), "Christmas closure"),
            (date(2025, 12, 29), "Christmas closure"),
            (date(2025, 12, 30), "Christmas closure"),
            (date(2025, 12, 31), "Christmas closure"),
        ];
        for (d, name) in expected {
            assert!(
                cal.is_holiday(d).unwrap(),
                "{name} ({d}) should be a Baltic LPG holiday"
            );
        }
    }

    #[test]
    fn baltic_lpg_christmas_week_full() {
        let cal = baltic_lpg();
        // Dec 25-31 all tagged as holidays (weekends filtered by calendar)
        for day in 25..=31 {
            let d = date(2024, 12, day);
            if d.weekday() != jiff::civil::Weekday::Saturday
                && d.weekday() != jiff::civil::Weekday::Sunday
            {
                assert!(
                    cal.is_holiday(d).unwrap(),
                    "Dec {day} 2024 should be a Baltic LPG holiday"
                );
            }
        }
    }

    #[test]
    fn baltic_lpg_name_and_tz() {
        let cal = baltic_lpg();
        assert_eq!(cal.name(), "BalticLPG");
        assert_eq!(*cal.tz(), TimeZone::get("Europe/London").unwrap());
    }

    #[test]
    fn baltic_lpg_regular_business_day() {
        let cal = baltic_lpg();
        assert!(cal.is_business_day(date(2025, 6, 11)).unwrap());
    }

    #[test]
    fn baltic_lpg_one_off_holidays() {
        let cal = baltic_lpg();
        // One-off proclaimed bank holidays
        assert!(cal.is_holiday(date(2023, 5, 8)).unwrap(), "Coronation");
        assert!(
            cal.is_holiday(date(2022, 9, 19)).unwrap(),
            "Queen's Funeral"
        );
        assert!(
            cal.is_holiday(date(2022, 6, 3)).unwrap(),
            "Platinum Jubilee"
        );
        assert!(cal.is_holiday(date(2012, 6, 5)).unwrap(), "Diamond Jubilee");
        assert!(cal.is_holiday(date(2011, 4, 29)).unwrap(), "Royal Wedding");
        assert!(cal.is_holiday(date(2002, 6, 3)).unwrap(), "Golden Jubilee");
        // VE Day 2020 move
        assert!(cal.is_holiday(date(2020, 5, 8)).unwrap(), "VE Day 2020");
        assert!(
            !cal.is_holiday(date(2020, 5, 4)).unwrap(),
            "Early May BH moved in 2020"
        );
        // Spring BH 2022 move
        assert!(
            cal.is_holiday(date(2022, 6, 2)).unwrap(),
            "Spring BH 2022 moved to Jun 2"
        );
    }

    // --- Baltic US tests ---

    #[test]
    fn baltic_us_inherits_base() {
        let cal = baltic_us();
        // UK base holidays
        assert!(cal.is_holiday(date(2025, 4, 18)).unwrap(), "Good Friday");
        assert!(
            cal.is_holiday(date(2025, 12, 29)).unwrap(),
            "Christmas closure"
        );
        // One-off
        assert!(cal.is_holiday(date(2023, 5, 8)).unwrap(), "Coronation");
    }

    #[test]
    fn baltic_us_extra_holidays() {
        let cal = baltic_us();
        assert!(cal.is_holiday(date(2025, 5, 26)).unwrap(), "Memorial Day");
        assert!(
            cal.is_holiday(date(2025, 7, 4)).unwrap(),
            "Independence Day"
        );
        assert!(cal.is_holiday(date(2025, 9, 1)).unwrap(), "Labor Day");
        assert!(cal.is_holiday(date(2025, 11, 27)).unwrap(), "Thanksgiving");
        assert!(
            cal.is_holiday(date(2025, 11, 28)).unwrap(),
            "Day after Thanksgiving"
        );
        assert!(
            cal.is_holiday(date(2026, 11, 27)).unwrap(),
            "Day after Thanksgiving 2026"
        );
        // Sat July 4 2026 → observed Fri July 3 (2026 Baltic schedule)
        assert!(
            cal.is_holiday(date(2026, 7, 3)).unwrap(),
            "Independence Day observed"
        );
    }

    #[test]
    fn baltic_lpg1_hormuz_regimes() {
        let cal = baltic_lpg1_hormuz();
        // Daily until 2026-03-03
        assert!(cal.is_business_day(date(2026, 3, 2)).unwrap());
        assert!(cal.is_business_day(date(2026, 3, 3)).unwrap());
        // Weekly (Wed) 2026-03-04..2026-04-06
        assert!(
            cal.is_business_day(date(2026, 3, 4)).unwrap(),
            "first weekly Wed"
        );
        assert!(cal.is_holiday(date(2026, 3, 5)).unwrap(), "Thu suppressed");
        assert!(cal.is_holiday(date(2026, 3, 9)).unwrap(), "Mon suppressed");
        assert!(
            cal.is_business_day(date(2026, 3, 11)).unwrap(),
            "Wed publishes"
        );
        assert!(
            cal.is_holiday(date(2026, 4, 2)).unwrap(),
            "Thu still suppressed in weekly regime"
        );
        assert!(
            cal.is_holiday(date(2026, 4, 6)).unwrap(),
            "Easter Monday + last weekly-regime day"
        );
        // Tue+Thu from 2026-04-07
        assert!(cal.is_business_day(date(2026, 4, 7)).unwrap(), "first Tue");
        assert!(
            cal.is_holiday(date(2026, 4, 8)).unwrap(),
            "Wed now suppressed"
        );
        assert!(
            cal.is_business_day(date(2026, 4, 9)).unwrap(),
            "Thu publishes"
        );
        assert!(cal.is_holiday(date(2026, 4, 10)).unwrap(), "Fri suppressed");
        // Existing holiday rules still apply on publication weekdays
        assert!(
            cal.is_holiday(date(2026, 12, 29)).unwrap(),
            "Christmas closure Tue"
        );
        // Loud failure beyond the announced horizon
        assert!(cal.is_business_day(date(2027, 1, 5)).is_err());
    }

    #[test]
    fn baltic_lpg1_hormuz_equals_baltic_lpg_before_crisis() {
        let cal = baltic_lpg1_hormuz();
        let lpg = baltic_lpg();
        let from = date(2020, 1, 1);
        let to = date(2026, 3, 4); // half-open: up to the first weekly-regime day
        assert_eq!(
            cal.holidays_between(from, to).unwrap(),
            lpg.holidays_between(from, to).unwrap()
        );
    }

    #[test]
    fn baltic_new_year_uk_observed() {
        // Jan 1 2023 fell Sunday; the Baltic followed the UK-observed Monday
        // (TC5 + BLPG1 settlement data have no 2023-01-02 values).
        for cal in [baltic_lpg(), baltic_us()] {
            assert!(cal.is_holiday(date(2023, 1, 2)).unwrap(), "{}", cal.name());
        }
    }

    #[test]
    fn baltic_us_name() {
        assert_eq!(baltic_us().name(), "BalticUS");
    }

    // --- Baltic Asia tests ---

    #[test]
    fn baltic_asia_shares_good_friday_and_christmas_only() {
        let cal = baltic_asia();
        assert!(cal.is_holiday(date(2025, 4, 18)).unwrap(), "Good Friday");
        assert!(
            cal.is_holiday(date(2025, 12, 29)).unwrap(),
            "Christmas closure"
        );
        // Per the official Baltic schedules, Asia indices publish through UK
        // bank holidays and one-offs.
        assert!(
            cal.is_business_day(date(2025, 4, 21)).unwrap(),
            "Easter Monday"
        );
        assert!(
            cal.is_business_day(date(2025, 5, 5)).unwrap(),
            "Early May BH"
        );
        assert!(cal.is_business_day(date(2025, 8, 25)).unwrap(), "Summer BH");
        assert!(cal.is_business_day(date(2023, 5, 8)).unwrap(), "Coronation");
    }

    #[test]
    fn baltic_asia_sg_holidays() {
        let cal = baltic_asia();
        // 2026 SG National Day is Sun Aug 9 → observed Mon Aug 10 (2026 schedule)
        assert!(
            cal.is_holiday(date(2026, 8, 10)).unwrap(),
            "SG National Day Sun→Mon"
        );
        // 2025 SG National Day is Sat Aug 9 → NO substitute Monday (2025
        // schedule omits it)
        assert!(
            cal.is_business_day(date(2025, 8, 11)).unwrap(),
            "no Sat→Mon substitution"
        );
        assert!(
            cal.is_holiday(date(2025, 1, 29)).unwrap(),
            "Chinese New Year"
        );
        assert!(
            cal.is_holiday(date(2026, 5, 27)).unwrap(),
            "Hari Raya Haji 2026"
        );
        assert!(cal.is_holiday(date(2025, 5, 1)).unwrap(), "Labour Day");
    }

    #[test]
    fn baltic_asia_matches_2025_2027_schedules() {
        // Weekday non-publication dates from the official Baltic Exchange
        // "Non-publication dates" PDFs.
        let cal = baltic_asia();
        let expected_2025 = [
            date(2025, 1, 1),
            date(2025, 1, 29),
            date(2025, 1, 30),
            date(2025, 3, 31),
            date(2025, 4, 18),
            date(2025, 5, 1),
            date(2025, 5, 12),
            date(2025, 10, 20),
            date(2025, 12, 25),
            date(2025, 12, 26),
            date(2025, 12, 29),
            date(2025, 12, 30),
            date(2025, 12, 31),
        ];
        let expected_2026 = [
            date(2026, 1, 1),
            date(2026, 2, 17),
            date(2026, 2, 18),
            date(2026, 4, 3),
            date(2026, 5, 1),
            date(2026, 5, 27),
            date(2026, 6, 1),
            date(2026, 8, 10),
            date(2026, 11, 9),
            date(2026, 12, 25),
            date(2026, 12, 28),
            date(2026, 12, 29),
            date(2026, 12, 30),
            date(2026, 12, 31),
        ];
        // 2027 PDF: CNY listed as "8 February" only (6/7 Feb are the
        // weekend); Sat 1 May Labour Day gets no substitute.
        let expected_2027 = [
            date(2027, 1, 1),
            date(2027, 2, 8),
            date(2027, 3, 10),
            date(2027, 3, 26),
            date(2027, 5, 17),
            date(2027, 5, 20),
            date(2027, 8, 9),
            date(2027, 10, 28),
            date(2027, 12, 27),
            date(2027, 12, 28),
            date(2027, 12, 29),
            date(2027, 12, 30),
            date(2027, 12, 31),
        ];
        for (year, expected) in [
            (2025, &expected_2025[..]),
            (2026, &expected_2026[..]),
            (2027, &expected_2027[..]),
        ] {
            let got: Vec<_> = cal
                .holidays_between(date(year, 1, 1), date(year + 1, 1, 1))
                .unwrap()
                .into_iter()
                .filter(|d| {
                    !matches!(
                        d.weekday(),
                        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                    )
                })
                .collect();
            assert_eq!(got, expected, "BalticAsia weekday holidays {year}");
        }
    }

    #[test]
    fn baltic_lpg_and_us_match_2027_schedule() {
        // Weekday non-publication dates from "Non-publication dates 2027"
        // "BDTI, BCTI + BLPG + BLNG" group
        // for BalticLPG; "BCTI TC14/18/21/24, MRA Triangulation & BDTI
        // TD9/21/25–29/33" group for BalticUS.
        let weekdays = |cal: &RuleBasedCalendar| {
            cal.holidays_between(date(2027, 1, 1), date(2028, 1, 1))
                .unwrap()
                .into_iter()
                .filter(|d| {
                    !matches!(
                        d.weekday(),
                        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                    )
                })
                .collect::<Vec<_>>()
        };
        let christmas = [
            date(2027, 12, 27),
            date(2027, 12, 28),
            date(2027, 12, 29),
            date(2027, 12, 30),
            date(2027, 12, 31),
        ];
        let mut lpg = vec![
            date(2027, 1, 1),
            date(2027, 3, 26),
            date(2027, 3, 29),
            date(2027, 5, 3),
            date(2027, 5, 31),
            date(2027, 8, 30),
        ];
        lpg.extend(christmas);
        assert_eq!(weekdays(&baltic_lpg()), lpg, "BalticLPG 2027");

        let mut us = vec![
            date(2027, 1, 1),
            date(2027, 1, 18), // Martin Luther King Day — new in 2027
            date(2027, 3, 26),
            date(2027, 3, 29),
            date(2027, 5, 3),
            date(2027, 5, 31), // Memorial Day = Spring Bank Holiday
            date(2027, 6, 18), // Juneteenth (Sat 19 Jun observed Fri) — new in 2027
            date(2027, 7, 5),  // Independence Day (Sun 4 Jul observed Mon)
            date(2027, 8, 30),
            date(2027, 9, 6),
            date(2027, 11, 25),
            date(2027, 11, 26),
        ];
        us.extend(christmas);
        assert_eq!(weekdays(&baltic_us()), us, "BalticUS 2027");

        // The from-2027 rules must not retro-fit earlier schedule years.
        let cal = baltic_us();
        assert!(cal.is_business_day(date(2026, 1, 19)).unwrap(), "MLK 2026");
        assert!(
            cal.is_business_day(date(2026, 6, 19)).unwrap(),
            "Juneteenth 2026"
        );
        assert!(cal.is_business_day(date(2025, 1, 20)).unwrap(), "MLK 2025");
    }

    #[test]
    fn baltic_asia_name() {
        assert_eq!(baltic_asia().name(), "BalticAsia");
    }

    // --- Platts Asia Marketscan tests ---

    #[test]
    fn platts_asia_marketscan_polling_days() {
        let cal = platts_asia_marketscan();
        // General Election 2020
        assert!(cal.is_holiday(date(2020, 7, 10)).unwrap());
        // Presidential Election 2023
        assert!(cal.is_holiday(date(2023, 9, 1)).unwrap());
    }

    #[test]
    fn platts_asia_marketscan_vesak_2023_corrected() {
        let cal = platts_asia_marketscan();
        // Vesak Day 2023: revised to Jun 2 (not Jun 3)
        assert!(cal.is_holiday(date(2023, 6, 2)).unwrap());
        assert!(!cal.is_holiday(date(2023, 6, 3)).unwrap());
    }

    #[test]
    fn platts_asia_marketscan_sunday_observed_only() {
        let cal = platts_asia_marketscan();
        // 2020: National Day Aug 9 = Sunday → Mon Aug 10 observed
        assert!(cal.is_holiday(date(2020, 8, 10)).unwrap());
        // 2025: National Day Aug 9 = Saturday → NO observed Monday
        assert!(cal.is_business_day(date(2025, 8, 11)).unwrap());

        // 2023: New Year Jan 1 = Sunday → Mon Jan 2 observed
        assert!(cal.is_holiday(date(2023, 1, 2)).unwrap());
        // 2022: New Year Jan 1 = Saturday → NO observed Monday
        assert!(cal.is_business_day(date(2022, 1, 3)).unwrap());

        // 2022: Labour Day May 1 = Sunday → Mon May 2 observed
        assert!(cal.is_holiday(date(2022, 5, 2)).unwrap());
        // 2021: Labour Day May 1 = Saturday → NO observed Monday
        assert!(cal.is_business_day(date(2021, 5, 3)).unwrap());

        // Christmas uses NextMonday (global market closure) — BOTH Sat and Sun shift
        // 2021: Dec 25 = Saturday → Mon Dec 27 observed
        assert!(cal.is_holiday(date(2021, 12, 27)).unwrap());
        // 2022: Dec 25 = Sunday → Mon Dec 26 observed
        assert!(cal.is_holiday(date(2022, 12, 26)).unwrap());
    }

    #[test]
    fn platts_asia_marketscan_name_and_tz() {
        let cal = platts_asia_marketscan();
        assert_eq!(cal.name(), "PlattsAsiaMarketScan");
        assert_eq!(*cal.tz(), TimeZone::get("Asia/Singapore").unwrap());
    }

    // --- Argus International LPG (Asia) tests ---

    #[test]
    fn argus_asia_lpg_name_and_tz() {
        let cal = argus_asia_lpg();
        assert_eq!(cal.name(), "ArgusAsiaLPG");
        assert_eq!(*cal.tz(), TimeZone::get("Asia/Singapore").unwrap());
    }

    // --- PlattsUSMarketScan tests ---

    #[test]
    fn platts_us_marketscan_name_tz_range() {
        let cal = platts_us_marketscan();
        assert_eq!(cal.name(), "PlattsUSMarketScan");
        assert_eq!(*cal.tz(), TimeZone::get("America/New_York").unwrap());
        assert!(cal.is_holiday(date(2016, 1, 1)).unwrap());
        assert!(cal.is_business_day(date(2027, 7, 6)).unwrap());
        assert!(
            cal.is_holiday(date(2015, 12, 25)).is_err(),
            "before valid range"
        );
    }

    #[test]
    fn platts_us_marketscan_2024_and_2026_match_schedule() {
        // Weekday non-publication days from the Platts holiday schedules 2024
        // and 2026; 2026-06-04 "Corpus Christi"
        // excluded as an adjudicated workbook error.
        let cal = platts_us_marketscan();
        let expected_2024 = [
            date(2024, 1, 1),
            date(2024, 1, 15),
            date(2024, 2, 19),
            date(2024, 3, 29),
            date(2024, 5, 27),
            date(2024, 6, 19),
            date(2024, 7, 4),
            date(2024, 7, 5),
            date(2024, 9, 2),
            date(2024, 11, 28),
            date(2024, 11, 29),
            date(2024, 12, 25),
        ];
        let expected_2026 = [
            date(2026, 1, 1),
            date(2026, 1, 19),
            date(2026, 2, 16),
            date(2026, 4, 3),
            date(2026, 5, 25),
            date(2026, 6, 19),
            date(2026, 7, 3),
            date(2026, 9, 7),
            date(2026, 11, 26),
            date(2026, 11, 27),
            date(2026, 12, 25),
        ];
        for (year, expected) in [(2024, &expected_2024[..]), (2026, &expected_2026[..])] {
            let got: Vec<_> = cal
                .holidays_between(date(year, 1, 1), date(year + 1, 1, 1))
                .unwrap()
                .into_iter()
                .filter(|d| {
                    !matches!(
                        d.weekday(),
                        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                    )
                })
                .collect();
            assert_eq!(got, expected, "PlattsUSMarketScan weekday holidays {year}");
        }
    }

    #[test]
    fn platts_us_marketscan_specials() {
        let cal = platts_us_marketscan();
        // Juneteenth: Platts from 2022, not the federal 2021.
        assert!(cal.is_business_day(date(2021, 6, 18)).unwrap());
        assert!(
            cal.is_holiday(date(2022, 6, 20)).unwrap(),
            "Sun→Mon observed"
        );
        // July 4 bridges (Tue→Mon, Thu→Fri).
        assert!(cal.is_holiday(date(2017, 7, 3)).unwrap());
        assert!(cal.is_holiday(date(2019, 7, 5)).unwrap());
        assert!(cal.is_holiday(date(2023, 7, 3)).unwrap());
        // 2018 Wed July 4: no bridge.
        assert!(cal.is_business_day(date(2018, 7, 3)).unwrap());
        assert!(cal.is_business_day(date(2018, 7, 5)).unwrap());
        // Columbus/Veterans Day: US Marketscan publishes.
        assert!(cal.is_business_day(date(2024, 10, 14)).unwrap());
        assert!(cal.is_business_day(date(2024, 11, 11)).unwrap());
        // One-offs: 2018 year-end closure + Fri after Christmas 2025.
        assert!(cal.is_holiday(date(2018, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2018, 12, 31)).unwrap());
        assert!(cal.is_holiday(date(2025, 12, 26)).unwrap());
        // Cross-year New Year observance: Sat 2022-01-01 → Fri 2021-12-31.
        assert!(cal.is_holiday(date(2021, 12, 31)).unwrap());
    }

    // --- ICISEuropeChemicals tests ---

    #[test]
    fn icis_europe_chemicals_year_end_weeks() {
        // Weekday closures per the ICIS annual publishing schedules,
        // Aromatics daily family.
        let cal = icis_europe_chemicals();
        // 2022: 23 Dec + 26 Dec – 2 Jan 2023
        assert!(cal.is_holiday(date(2022, 12, 23)).unwrap());
        assert!(cal.is_holiday(date(2022, 12, 28)).unwrap());
        assert!(cal.is_holiday(date(2022, 12, 30)).unwrap());
        assert!(cal.is_business_day(date(2022, 12, 22)).unwrap());
        // 2023: 22 Dec + 25 Dec – 1 Jan 2024
        assert!(cal.is_holiday(date(2023, 12, 22)).unwrap());
        assert!(cal.is_holiday(date(2023, 12, 29)).unwrap());
        assert!(cal.is_business_day(date(2023, 12, 21)).unwrap());
        // 2024: 24 Dec – 1 Jan 2025 (23 Dec publishes)
        assert!(cal.is_business_day(date(2024, 12, 23)).unwrap());
        assert!(cal.is_holiday(date(2024, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2024, 12, 31)).unwrap());
        // 2025: 24 Dec – 2 Jan 2026
        assert!(cal.is_holiday(date(2025, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2026, 1, 2)).unwrap());
        // 2026: 25 Dec – 1 Jan 2027 (24 Dec publishes; families merged)
        assert!(cal.is_business_day(date(2026, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2026, 12, 31)).unwrap());
    }

    #[test]
    fn icis_europe_chemicals_uk_base_no_continental() {
        let cal = icis_europe_chemicals();
        // UK bank holidays observed (incl. one-offs)
        assert!(cal.is_holiday(date(2022, 9, 19)).unwrap(), "State Funeral");
        assert!(cal.is_holiday(date(2023, 5, 8)).unwrap(), "Coronation");
        assert!(cal.is_holiday(date(2025, 8, 25)).unwrap(), "Summer BH");
        // No continental holidays
        assert!(cal.is_business_day(date(2024, 5, 1)).unwrap(), "May Day");
        assert!(
            cal.is_business_day(date(2023, 11, 1)).unwrap(),
            "All Saints"
        );
        // Schedule-era bounds are loud
        assert!(cal.is_holiday(date(2021, 12, 27)).is_err());
        assert!(cal.is_holiday(date(2027, 1, 1)).is_err());
    }

    // --- ICISChinaChemicals tests ---

    #[test]
    fn icis_china_chemicals_matches_schedules() {
        // Weekday closure sets from the ICIS annual publishing schedules
        // (China-CN "All dailies"), 2025 and 2026; 2027 from the State
        // Council gazette 国办发明电〔2026〕10号 (ICIS schedule pending).
        let cal = icis_china_chemicals();
        let expected_2025 = [
            date(2025, 1, 1),
            date(2025, 1, 28),
            date(2025, 1, 29),
            date(2025, 1, 30),
            date(2025, 1, 31),
            date(2025, 2, 3),
            date(2025, 2, 4),
            date(2025, 4, 4),
            date(2025, 5, 1),
            date(2025, 5, 2),
            date(2025, 5, 5),
            date(2025, 6, 2),
            date(2025, 10, 1),
            date(2025, 10, 2),
            date(2025, 10, 3),
            date(2025, 10, 6),
            date(2025, 10, 7),
            date(2025, 10, 8),
        ];
        let expected_2026 = [
            date(2026, 1, 1),
            date(2026, 1, 2),
            date(2026, 2, 16),
            date(2026, 2, 17),
            date(2026, 2, 18),
            date(2026, 2, 19),
            date(2026, 2, 20),
            date(2026, 2, 23),
            date(2026, 4, 6),
            date(2026, 5, 1),
            date(2026, 5, 4),
            date(2026, 5, 5),
            date(2026, 6, 19),
            date(2026, 9, 25),
            date(2026, 10, 1),
            date(2026, 10, 2),
            date(2026, 10, 5),
            date(2026, 10, 6),
            date(2026, 10, 7),
        ];
        let expected_2027 = [
            date(2027, 1, 1),
            date(2027, 2, 5),
            date(2027, 2, 8),
            date(2027, 2, 9),
            date(2027, 2, 10),
            date(2027, 2, 11),
            date(2027, 2, 12),
            date(2027, 4, 5),
            date(2027, 5, 3),
            date(2027, 5, 4),
            date(2027, 5, 5),
            date(2027, 6, 9),
            date(2027, 9, 15),
            date(2027, 10, 1),
            date(2027, 10, 4),
            date(2027, 10, 5),
            date(2027, 10, 6),
            date(2027, 10, 7),
        ];
        for (year, expected) in [
            (2025, &expected_2025[..]),
            (2026, &expected_2026[..]),
            (2027, &expected_2027[..]),
        ] {
            let got: Vec<_> = cal
                .holidays_between(date(year, 1, 1), date(year + 1, 1, 1))
                .unwrap()
                .into_iter()
                // ignore the explicit "Weekend" holidays; gazette closures only
                .filter(|d| {
                    !matches!(
                        d.weekday(),
                        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                    )
                })
                .collect();
            assert_eq!(got, expected, "ICISChinaChemicals holidays {year}");
        }
    }

    #[test]
    fn icis_china_chemicals_explicit_weekends() {
        let cal = icis_china_chemicals();
        // Ordinary weekends are explicit holidays named "Weekend"...
        assert!(cal.is_holiday(date(2025, 1, 25)).unwrap());
        assert_eq!(
            cal.holiday_name(date(2025, 1, 25)).unwrap(),
            Some("Weekend")
        );
        // ...but gazette makeup working weekends are business days.
        assert!(
            cal.is_business_day(date(2025, 2, 8)).unwrap(),
            "CNY makeup Sat 2025"
        );
        assert!(
            cal.is_business_day(date(2025, 9, 28)).unwrap(),
            "GW makeup Sun 2025"
        );
        assert!(
            cal.is_business_day(date(2024, 2, 4)).unwrap(),
            "CNY makeup Sun 2024"
        );
        assert!(
            cal.is_business_day(date(2022, 10, 8)).unwrap(),
            "GW makeup Sat 2022"
        );
        // 2027 gazette makeup days (国办发明电〔2026〕10号).
        for d in [
            date(2027, 1, 31),
            date(2027, 2, 14),
            date(2027, 5, 8),
            date(2027, 9, 26),
            date(2027, 10, 9),
        ] {
            assert!(cal.is_business_day(d).unwrap(), "makeup weekend 2027 {d}");
        }
        assert!(cal.is_holiday(date(2027, 2, 6)).unwrap(), "CNY Sat 2027");
        assert!(cal.is_holiday(date(2027, 10, 2)).unwrap(), "GW Sat 2027");
    }

    #[test]
    fn icis_china_chemicals_publishes_western_holidays() {
        let cal = icis_china_chemicals();
        // SG/UK/Western holidays are publication days for China-CN dailies.
        assert!(
            cal.is_business_day(date(2026, 4, 3)).unwrap(),
            "Good Friday"
        );
        assert!(
            cal.is_business_day(date(2025, 12, 25)).unwrap(),
            "Christmas"
        );
        assert!(
            cal.is_business_day(date(2025, 8, 11)).unwrap(),
            "no SG National Day"
        );
        // Era bounds are loud.
        assert!(cal.is_holiday(date(2021, 10, 1)).is_err());
        assert!(cal.is_holiday(date(2028, 2, 1)).is_err());
    }

    // --- ChinaFuturesExchanges tests ---

    #[test]
    fn china_futures_exchanges_same_gazette_weekday_closures_as_icis() {
        // The two calendars share the festival DateLists; their weekday
        // holiday sets must be identical for every covered year.
        let fx = china_futures_exchanges();
        let icis = icis_china_chemicals();
        for year in 2022..=2027 {
            let weekdays = |cal: &RuleBasedCalendar| {
                cal.holidays_between(date(year, 1, 1), date(year + 1, 1, 1))
                    .unwrap()
                    .into_iter()
                    .filter(|d| {
                        !matches!(
                            d.weekday(),
                            jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                        )
                    })
                    .collect::<Vec<_>>()
            };
            let mut expected = weekdays(&icis);
            if year == 2024 {
                // Exchange-specific extra closure beyond the gazette.
                expected.insert(0, date(2024, 2, 9));
                expected.sort();
            }
            assert_eq!(weekdays(&fx), expected, "weekday closures {year}");
        }
    }

    #[test]
    fn china_futures_exchanges_cny_eve_2024() {
        // SHFE/ZCE/DCE/GFE closed CNY eve 2024 although the gazette
        // holiday only started 02-10; ICIS published that day.
        let fx = china_futures_exchanges();
        assert!(fx.is_holiday(date(2024, 2, 9)).unwrap());
        assert!(fx.is_business_day(date(2024, 2, 8)).unwrap());
        assert!(
            icis_china_chemicals()
                .is_business_day(date(2024, 2, 9))
                .unwrap()
        );
    }

    #[test]
    fn china_futures_exchanges_closed_on_makeup_weekends() {
        // DCE 2026 Spring Festival notice: the gazette makeup Saturdays
        // 14 and 28 Feb are ordinary weekend closures for the exchanges.
        let cal = china_futures_exchanges();
        for d in [
            date(2027, 2, 14),
            date(2027, 10, 9),
            date(2026, 2, 14),
            date(2026, 2, 28),
            date(2026, 1, 4),
            date(2025, 2, 8),
            date(2024, 2, 4),
            date(2022, 10, 8),
        ] {
            assert!(!cal.is_business_day(d).unwrap(), "makeup weekend {d}");
            // implicit weekend, not a named holiday
            assert!(!cal.is_holiday(d).unwrap(), "makeup weekend {d}");
        }
        // ...while the ICIS China-CN dailies publish on those days.
        assert!(
            icis_china_chemicals()
                .is_business_day(date(2026, 2, 14))
                .unwrap(),
            "ICIS publishes on makeup Saturdays"
        );
    }

    #[test]
    fn china_futures_exchanges_spring_festival_2026() {
        // DCE notice: closed 15–23 Feb 2026, reopening Tue 24 Feb.
        let cal = china_futures_exchanges();
        for day in 16..=20 {
            assert!(
                cal.is_holiday(date(2026, 2, day)).unwrap(),
                "CNY weekday {day}"
            );
        }
        assert!(cal.is_holiday(date(2026, 2, 23)).unwrap());
        assert!(
            cal.is_business_day(date(2026, 2, 24)).unwrap(),
            "reopen 24 Feb"
        );
    }

    #[test]
    fn china_futures_exchanges_trades_western_holidays() {
        let cal = china_futures_exchanges();
        assert!(
            cal.is_business_day(date(2026, 4, 3)).unwrap(),
            "Good Friday"
        );
        assert!(
            cal.is_business_day(date(2025, 12, 25)).unwrap(),
            "Christmas"
        );
        // Era bounds are loud.
        assert!(cal.is_holiday(date(2021, 10, 1)).is_err());
        assert!(cal.is_holiday(date(2028, 2, 1)).is_err());
    }

    #[test]
    fn china_futures_exchanges_spring_festival_2027() {
        // Gazette 国办发明电〔2026〕10号: closed Fri 5 Feb (eve) – Sat 13 Feb
        // 2027, makeup Sundays 31 Jan and 14 Feb; exchanges reopen Mon 15 Feb.
        let cal = china_futures_exchanges();
        assert!(cal.is_business_day(date(2027, 2, 4)).unwrap(), "pre-eve");
        assert!(cal.is_holiday(date(2027, 2, 5)).unwrap(), "eve");
        for day in 8..=12 {
            assert!(
                cal.is_holiday(date(2027, 2, day)).unwrap(),
                "CNY weekday {day}"
            );
        }
        assert!(
            cal.is_business_day(date(2027, 2, 15)).unwrap(),
            "reopen 15 Feb"
        );
    }

    // --- PlattsUSGasDaily tests ---

    #[test]
    fn platts_us_gas_daily_name_tz_range() {
        let cal = platts_us_gas_daily();
        assert_eq!(cal.name(), "PlattsUSGasDaily");
        assert_eq!(*cal.tz(), TimeZone::get("America/New_York").unwrap());
        assert!(cal.is_holiday(date(2016, 1, 1)).unwrap());
        assert!(cal.is_business_day(date(2027, 7, 6)).unwrap());
        assert!(
            cal.is_holiday(date(2015, 12, 25)).is_err(),
            "before valid range"
        );
        assert!(
            cal.is_holiday(date(2028, 1, 3)).is_err(),
            "after valid range"
        );
    }

    #[test]
    fn platts_us_gas_daily_matches_schedules_and_henry_hub_data() {
        // Weekday non-publication days per the Gas rows of the Platts holiday
        // schedules 2016–2026. For 2016-01 … 2026-10 these are also exactly
        // the weekdays without a Platts Henry Hub trade-date assessment.
        let cal = platts_us_gas_daily();
        #[rustfmt::skip]
        let expected: [(i16, &[(i8, i8)]); 11] = [
            (2016, &[(1, 1), (1, 18), (2, 15), (3, 25), (5, 30), (7, 4), (9, 5), (11, 24), (11, 25), (12, 26)]),
            (2017, &[(1, 2), (1, 16), (2, 20), (4, 14), (5, 29), (7, 3), (7, 4), (9, 4), (11, 23), (11, 24), (12, 25)]),
            (2018, &[(1, 1), (1, 15), (2, 19), (3, 30), (5, 28), (7, 4), (9, 3), (11, 22), (11, 23), (12, 24), (12, 25), (12, 31)]),
            (2019, &[(1, 1), (1, 21), (2, 18), (4, 19), (5, 27), (7, 4), (7, 5), (9, 2), (11, 28), (11, 29), (12, 25)]),
            (2020, &[(1, 1), (1, 20), (2, 17), (4, 10), (5, 25), (7, 3), (9, 7), (11, 26), (11, 27), (12, 25)]),
            (2021, &[(1, 1), (1, 18), (2, 15), (4, 2), (5, 31), (7, 5), (9, 6), (11, 25), (11, 26), (12, 24), (12, 31)]),
            (2022, &[(1, 17), (2, 21), (4, 15), (5, 30), (6, 20), (7, 4), (9, 5), (11, 11), (11, 24), (11, 25), (12, 26)]),
            (2023, &[(1, 2), (1, 16), (2, 20), (4, 7), (5, 29), (6, 19), (7, 3), (7, 4), (9, 4), (11, 10), (11, 23), (11, 24), (12, 25)]),
            (2024, &[(1, 1), (1, 15), (2, 19), (3, 29), (5, 27), (6, 19), (7, 4), (7, 5), (9, 2), (10, 14), (11, 11), (11, 28), (11, 29), (12, 25)]),
            (2025, &[(1, 1), (1, 20), (2, 17), (4, 18), (5, 26), (6, 19), (7, 4), (9, 1), (10, 13), (11, 11), (11, 27), (11, 28), (12, 25), (12, 26)]),
            (2026, &[(1, 1), (1, 2), (1, 19), (2, 16), (4, 3), (5, 25), (6, 19), (7, 3), (9, 7), (10, 12), (11, 11), (11, 26), (11, 27), (12, 25)]),
        ];
        let mut total = 0;
        for (year, days) in expected {
            let want: Vec<_> = days.iter().map(|&(m, d)| date(year, m, d)).collect();
            let got: Vec<_> = cal
                .holidays_between(date(year, 1, 1), date(year + 1, 1, 1))
                .unwrap()
                .into_iter()
                .filter(|d| {
                    !matches!(
                        d.weekday(),
                        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                    )
                })
                .collect();
            assert_eq!(got, want, "PlattsUSGasDaily weekday holidays {year}");
            total += want.len();
        }
        assert_eq!(total, 131);
    }

    #[test]
    fn platts_us_gas_daily_is_us_marketscan_plus_gas_pricing_holidays() {
        // Every US Marketscan non-publication weekday is a Gas Daily one; the
        // gas-only weekdays are exactly Veterans Day from 2022 (observed),
        // Columbus Day from 2024 and the 2026-01-02 one-off.
        let gas = platts_us_gas_daily();
        let oil = platts_us_marketscan();
        let mut gas_only = Vec::new();
        let mut d = date(2016, 1, 1);
        while d < date(2027, 1, 1) {
            if !matches!(
                d.weekday(),
                jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
            ) {
                let g = gas.is_holiday(d).unwrap();
                let o = oil.is_holiday(d).unwrap();
                assert!(
                    g || !o,
                    "{d}: US Marketscan holiday but Gas Daily publishes"
                );
                if g && !o {
                    gas_only.push(d);
                }
            }
            d = d.tomorrow().unwrap();
        }
        assert_eq!(
            gas_only,
            vec![
                date(2022, 11, 11),
                date(2023, 11, 10),
                date(2024, 10, 14),
                date(2024, 11, 11),
                date(2025, 10, 13),
                date(2025, 11, 11),
                date(2026, 1, 2),
                date(2026, 10, 12),
                date(2026, 11, 11),
            ]
        );
    }

    #[test]
    fn platts_us_gas_daily_specials() {
        let cal = platts_us_gas_daily();
        // Columbus Day: gas pricing holiday from 2024 only.
        assert!(cal.is_business_day(date(2022, 10, 10)).unwrap());
        assert!(cal.is_business_day(date(2023, 10, 9)).unwrap());
        assert_eq!(
            cal.holiday_name(date(2024, 10, 14)).unwrap(),
            Some("Columbus Day (Platts gas, from 2024)")
        );
        // Veterans Day: from 2022, federal observance (Sat 2023-11-11 → Fri 2023-11-10).
        assert!(cal.is_business_day(date(2021, 11, 11)).unwrap());
        assert!(cal.is_holiday(date(2022, 11, 11)).unwrap());
        assert_eq!(
            cal.holiday_name(date(2023, 11, 10)).unwrap(),
            Some("Veterans Day (Platts gas Observed, from 2022)")
        );
        assert!(cal.is_business_day(date(2023, 11, 13)).unwrap());
        // Juneteenth: Platts from 2022 (2021 published); no Friday bridge after Thu 2025-06-19.
        assert!(cal.is_business_day(date(2021, 6, 18)).unwrap());
        assert!(cal.is_holiday(date(2022, 6, 20)).unwrap());
        assert!(cal.is_business_day(date(2025, 6, 20)).unwrap());
        // Year-end: 2021 federal observances, 2025/2026 Friday bridges, 2018 closure.
        assert!(cal.is_holiday(date(2021, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2021, 12, 31)).unwrap());
        assert!(cal.is_business_day(date(2022, 1, 3)).unwrap());
        assert!(cal.is_holiday(date(2025, 12, 26)).unwrap());
        assert_eq!(
            cal.holiday_name(date(2026, 1, 2)).unwrap(),
            Some("Platts US gas day after New Year's Day (2026)")
        );
        assert!(cal.is_holiday(date(2018, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2018, 12, 31)).unwrap());
        // Canadian / European gas closures do not apply.
        assert!(cal.is_business_day(date(2024, 5, 20)).unwrap()); // Victoria Day (CGPR Daily)
        assert!(cal.is_business_day(date(2024, 12, 26)).unwrap()); // Boxing Day (European Gas Daily)
    }

    // --- OPISPetroChemWire tests ---

    #[test]
    fn opis_petrochem_wire_matches_schedules() {
        // Weekday non-publication days from the official OPIS holiday publish
        // schedules 2025+2026, identical across
        // all five US daily petchem publications.
        let cal = opis_petrochem_wire();
        let expected_2025 = [
            date(2025, 1, 1),
            date(2025, 1, 20),
            date(2025, 2, 17),
            date(2025, 4, 18),
            date(2025, 5, 26),
            date(2025, 6, 19),
            date(2025, 7, 4),
            date(2025, 9, 1),
            date(2025, 11, 27),
            date(2025, 11, 28),
            date(2025, 12, 25),
            date(2025, 12, 26),
        ];
        let expected_2026 = [
            date(2026, 1, 1),
            date(2026, 1, 19),
            date(2026, 2, 16),
            date(2026, 4, 3),
            date(2026, 5, 25),
            date(2026, 6, 19),
            date(2026, 7, 3),
            date(2026, 9, 7),
            date(2026, 11, 26),
            date(2026, 11, 27),
            date(2026, 12, 25),
        ];
        for (year, expected) in [(2025, &expected_2025[..]), (2026, &expected_2026[..])] {
            let got: Vec<_> = cal
                .holidays_between(date(year, 1, 1), date(year + 1, 1, 1))
                .unwrap()
                .into_iter()
                .filter(|d| {
                    !matches!(
                        d.weekday(),
                        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
                    )
                })
                .collect();
            assert_eq!(got, expected, "OPISPetroChemWire weekday holidays {year}");
        }
    }

    #[test]
    fn opis_petrochem_wire_equals_platts_us_over_verified_window() {
        let opis = opis_petrochem_wire();
        let platts = platts_us_marketscan();
        let (from, to) = (date(2025, 1, 1), date(2027, 1, 1));
        assert_eq!(
            opis.holidays_between(from, to).unwrap(),
            platts.holidays_between(from, to).unwrap()
        );
        // Loud failure outside the OPIS-verified era.
        assert!(opis.is_holiday(date(2024, 12, 25)).is_err());
    }

    // --- ArgusEuropeLPG tests ---

    #[test]
    fn argus_europe_lpg_name_and_tz() {
        let cal = argus_europe_lpg();
        assert_eq!(cal.name(), "ArgusEuropeLPG");
        assert_eq!(*cal.tz(), TimeZone::get("Europe/London").unwrap());
    }

    #[test]
    fn argus_europe_lpg_matches_uk_settlement() {
        // Rule-identical to UKSettlement over the schedule-verified window
        // (official Argus publishing schedules 2020–2026: 59/59 weekday match).
        let argus = argus_europe_lpg();
        let uk = uk_settlement();
        let from = date(2020, 1, 1);
        let to = date(2027, 1, 1);
        assert_eq!(
            argus.holidays_between(from, to).unwrap(),
            uk.holidays_between(from, to).unwrap()
        );
    }

    #[test]
    fn argus_europe_lpg_publishes_half_days_and_may_day() {
        let cal = argus_europe_lpg();
        // Argus publishes the Dec 24 / Dec 31 half-days…
        assert!(cal.is_business_day(date(2024, 12, 24)).unwrap());
        assert!(cal.is_business_day(date(2024, 12, 31)).unwrap());
        assert!(cal.is_business_day(date(2021, 12, 24)).unwrap());
        // …except the data-evidenced 2018 year-end closure…
        assert!(cal.is_holiday(date(2018, 12, 24)).unwrap());
        assert!(cal.is_holiday(date(2018, 12, 31)).unwrap());
        // …and through continental May Day (UK holiday is the first Monday).
        assert!(cal.is_business_day(date(2024, 5, 1)).unwrap());
        assert!(cal.is_holiday(date(2024, 5, 6)).unwrap());
    }

    #[test]
    fn argus_calendars_2018_year_end_closure() {
        // Both Argus calendars observe the 2018 Dec 24/31 closure; other
        // years' half-days stay business days.
        for cal in [argus_asia_lpg(), argus_europe_lpg()] {
            assert!(
                cal.is_holiday(date(2018, 12, 24)).unwrap(),
                "{}",
                cal.name()
            );
            assert!(
                cal.is_holiday(date(2018, 12, 31)).unwrap(),
                "{}",
                cal.name()
            );
            assert!(
                cal.is_business_day(date(2019, 12, 24)).unwrap(),
                "{}",
                cal.name()
            );
            assert!(
                cal.is_business_day(date(2019, 12, 31)).unwrap(),
                "{}",
                cal.name()
            );
        }
        // ArgusAsiaLPG is now date-equivalent to PlattsAsiaMarketScan.
        let argus = argus_asia_lpg();
        let platts = platts_asia_marketscan();
        let (from, to) = (date(2010, 1, 1), date(2027, 1, 1));
        assert_eq!(
            argus.holidays_between(from, to).unwrap(),
            platts.holidays_between(from, to).unwrap()
        );
    }

    #[test]
    fn argus_europe_lpg_2022_one_offs() {
        let cal = argus_europe_lpg();
        // Schedule-confirmed: Platinum Jubilee (2–3 Jun) and State Funeral.
        assert!(cal.is_holiday(date(2022, 6, 2)).unwrap());
        assert!(cal.is_holiday(date(2022, 6, 3)).unwrap());
        assert!(cal.is_holiday(date(2022, 9, 19)).unwrap());
    }

    // --- Valid range tests ---

    #[test]
    fn builtin_calendars_have_bounded_ranges() {
        use crate::RangeBound;
        for cal in [
            target(),
            us_settlement(),
            uk_settlement(),
            platts_european_market_scan(),
            ice_uk(),
            sgx(),
            baltic_lpg(),
            platts_asia_marketscan(),
        ] {
            let (from, to) = cal.valid_range();
            assert!(
                matches!(from, RangeBound::Date(_)),
                "{} should have a bounded from range",
                cal.name()
            );
            assert!(
                matches!(to, RangeBound::Date(_)),
                "{} should have a bounded to range",
                cal.name()
            );
        }
    }

    #[test]
    fn builtin_calendars_reject_out_of_range() {
        for cal in [
            target(),
            us_settlement(),
            uk_settlement(),
            platts_european_market_scan(),
            ice_uk(),
        ] {
            assert!(
                cal.is_business_day(date(1999, 6, 15)).is_err(),
                "{} should reject dates before 2000",
                cal.name()
            );
            assert!(
                cal.is_business_day(date(2100, 6, 15)).is_err(),
                "{} should reject dates after 2099",
                cal.name()
            );
        }
    }

    // --- ICE UK averaging tests ---

    #[test]
    fn ice_uk_averaging_differs_from_ice_uk_only_after_a_sunday_new_year() {
        let cmp = crate::compare_calendars(
            &ice_uk(),
            &ice_uk_averaging(),
            date(2000, 1, 1),
            date(2099, 12, 31),
        )
        .unwrap();
        assert!(cmp.b_only_bdays.is_empty());
        assert_eq!(cmp.a_only_bdays.len(), 14);
        for d in &cmp.a_only_bdays {
            assert_eq!((d.month(), d.day()), (1, 2), "{d}");
        }
        assert!(cmp.a_only_bdays.contains(&date(2023, 1, 2)));
    }

    // --- ICE Endex tests ---

    /// Weekdays 2018-04-26 … 2026-11-18 on which ICE's Dutch TTF gas future
    /// has no new settlement (ICE settlement files): no record at all on New Year's Day,
    /// Good Friday and Christmas Day (and on Mon 26 Dec 2022), a record with
    /// every strip carried over on Easter Monday and 26 December.
    const ICE_ENDEX_TTF_CLOSED: [(i16, i8, i8); 34] = [
        (2018, 12, 25),
        (2018, 12, 26),
        (2019, 1, 1),
        (2019, 4, 19),
        (2019, 4, 22),
        (2019, 12, 25),
        (2019, 12, 26),
        (2020, 1, 1),
        (2020, 4, 10),
        (2020, 4, 13),
        (2020, 12, 25),
        (2021, 1, 1),
        (2021, 4, 2),
        (2021, 4, 5),
        (2022, 4, 15),
        (2022, 4, 18),
        (2022, 12, 26),
        (2023, 4, 7),
        (2023, 4, 10),
        (2023, 12, 25),
        (2023, 12, 26),
        (2024, 1, 1),
        (2024, 3, 29),
        (2024, 4, 1),
        (2024, 12, 25),
        (2024, 12, 26),
        (2025, 1, 1),
        (2025, 4, 18),
        (2025, 4, 21),
        (2025, 12, 25),
        (2025, 12, 26),
        (2026, 1, 1),
        (2026, 4, 3),
        (2026, 4, 6),
    ];

    #[test]
    fn ice_endex_matches_ttf_settlement_days() {
        let cal = ice_endex();
        let closed: Vec<jiff::civil::Date> = ICE_ENDEX_TTF_CLOSED
            .iter()
            .map(|&(y, m, d)| date(y, m, d))
            .collect();
        let mut found = Vec::new();
        let mut day = date(2018, 4, 26);
        while day <= date(2026, 11, 18) {
            if !cal.is_weekend_day(day.weekday()) && cal.is_holiday(day).unwrap() {
                found.push(day);
            }
            day = day.tomorrow().unwrap();
        }
        assert_eq!(found, closed);
    }

    #[test]
    fn ice_endex_has_no_weekend_observance() {
        let cal = ice_endex();
        // New Year 2023 and Christmas 2022 on a Sunday, Christmas 2021 and
        // New Year 2022 on a Saturday: the following weekdays settle.
        for (y, m, d) in [
            (2023, 1, 2),
            (2022, 12, 27),
            (2021, 12, 27),
            (2021, 12, 28),
            (2022, 1, 3),
        ] {
            assert!(cal.is_business_day(date(y, m, d)).unwrap(), "{y}-{m}-{d}");
        }
        // … and so do 1 May and the UK bank holidays.
        for (y, m, d) in [(2026, 5, 1), (2026, 5, 4), (2026, 5, 25), (2026, 8, 31)] {
            assert!(cal.is_business_day(date(y, m, d)).unwrap(), "{y}-{m}-{d}");
        }
        assert_eq!(cal.name(), "ICEEndex");
        assert!(
            cal.is_business_day(date(2017, 6, 15)).is_err(),
            "before the data"
        );
    }

    #[test]
    fn ice_endex_against_its_neighbours() {
        let span = |a: &RuleBasedCalendar, b: &RuleBasedCalendar| {
            crate::compare_calendars(a, b, date(2018, 1, 1), date(2099, 12, 31)).unwrap()
        };
        // TARGET is ICE Endex plus 1 May.
        let cmp = span(&ice_endex(), &target());
        assert!(cmp.b_only_bdays.is_empty());
        assert!(!cmp.a_only_bdays.is_empty());
        for d in &cmp.a_only_bdays {
            assert_eq!((d.month(), d.day()), (5, 1), "{d}");
        }
        // Brent's fixing days are open on Easter Monday and 26 December,
        // where ICE Endex is closed …
        let cmp = span(&ice_endex(), &ice_uk_averaging());
        assert!(cmp.b_only_bdays.iter().all(|d| {
            (d.month(), d.day()) == (12, 26) || d.weekday() == jiff::civil::Weekday::Monday
        }));
        assert!(cmp.b_only_bdays.contains(&date(2026, 4, 6)));
        assert!(cmp.b_only_bdays.contains(&date(2025, 12, 26)));
        // … and closed on the Monday after a Sunday New Year, where ICE
        // Endex settles. (The Monday after a Sunday Christmas is 26
        // December: closed for both.)
        assert!(cmp.a_only_bdays.contains(&date(2023, 1, 2)));
        assert!(
            cmp.a_only_bdays
                .iter()
                .all(|d| (d.month(), d.day()) == (1, 2))
        );
    }

    // --- NYMEX tests ---

    /// Weekdays 2021-02-01 … 2026-11-10 on which ICE's Henry LD1 future has
    /// no settlement record (ICE closed).
    const NYMEX_ICE_CLOSED: [(i16, i8, i8); 13] = [
        (2021, 4, 2),
        (2022, 4, 15),
        (2022, 12, 26),
        (2023, 4, 7),
        (2023, 12, 25),
        (2024, 1, 1),
        (2024, 3, 29),
        (2024, 12, 25),
        (2025, 1, 1),
        (2025, 4, 18),
        (2025, 12, 25),
        (2026, 1, 1),
        (2026, 4, 3),
    ];

    /// Days in the same window on which every Henry LD1 strip is carried
    /// unchanged (no NYMEX settlement).
    const NYMEX_NO_SETTLEMENT: [(i16, i8, i8); 41] = [
        (2021, 2, 15),
        (2021, 5, 31),
        (2021, 7, 5),
        (2021, 9, 6),
        (2021, 11, 25),
        (2021, 12, 24),
        (2022, 1, 17),
        (2022, 2, 21),
        (2022, 5, 30),
        (2022, 6, 20),
        (2022, 7, 4),
        (2022, 9, 5),
        (2022, 11, 24),
        (2023, 1, 2),
        (2023, 1, 16),
        (2023, 2, 20),
        (2023, 5, 29),
        (2023, 6, 19),
        (2023, 7, 4),
        (2023, 9, 4),
        (2023, 11, 23),
        (2024, 1, 15),
        (2024, 2, 19),
        (2024, 5, 27),
        (2024, 6, 19),
        (2024, 7, 4),
        (2024, 9, 2),
        (2024, 11, 28),
        (2025, 1, 20),
        (2025, 2, 17),
        (2025, 5, 26),
        (2025, 6, 19),
        (2025, 7, 4),
        (2025, 9, 1),
        (2025, 11, 27),
        (2026, 1, 19),
        (2026, 2, 16),
        (2026, 5, 25),
        (2026, 6, 19),
        (2026, 7, 3),
        (2026, 9, 7),
    ];

    #[test]
    fn nymex_matches_ice_henry_evidence_2021_to_2026() {
        let mut expected: Vec<_> = NYMEX_ICE_CLOSED
            .iter()
            .chain(NYMEX_NO_SETTLEMENT.iter())
            .map(|&(y, m, d)| date(y, m, d))
            .collect();
        expected.sort();
        // holidays_between is half-open: pass the day after the window's end
        let actual = nymex()
            .holidays_between(date(2021, 2, 1), date(2026, 11, 11))
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn nymex_settles_where_other_us_calendars_close() {
        let cal = nymex();
        assert!(cal.is_business_day(date(2021, 6, 18)).unwrap()); // no Juneteenth in 2021
        assert!(cal.is_business_day(date(2021, 12, 31)).unwrap()); // New Year on Saturday
        // CME settlement notice for New Year 2028 (a Saturday): Fri 31 Dec
        // settles at normal times, Mon 3 Jan is open with settlement
        assert!(cal.is_business_day(date(2027, 12, 31)).unwrap());
        assert!(cal.is_business_day(date(2028, 1, 3)).unwrap());
        // … while Christmas 2027 (also a Saturday) is observed on the Friday,
        // per the Christmas 2027 notice; Thu 23 Dec settles early
        assert!(cal.is_holiday(date(2027, 12, 24)).unwrap());
        assert!(cal.is_business_day(date(2027, 12, 23)).unwrap());
        // Independence Day 2027 on a Sunday: Mon 5 July, per the notice
        assert!(cal.is_holiday(date(2027, 7, 5)).unwrap());
        assert!(cal.is_business_day(date(2027, 7, 2)).unwrap());
        // Juneteenth 2027 on a Saturday: Fri 18 June, per the notice
        assert!(cal.is_holiday(date(2027, 6, 18)).unwrap());
        assert!(cal.is_business_day(date(2027, 6, 17)).unwrap());
        assert!(cal.is_business_day(date(2025, 10, 13)).unwrap()); // Columbus Day
        assert!(cal.is_business_day(date(2025, 11, 11)).unwrap()); // Veterans Day
        assert!(cal.is_business_day(date(2025, 11, 28)).unwrap()); // day after Thanksgiving
        assert!(cal.is_business_day(date(2024, 7, 5)).unwrap()); // no July-4 bridge
        assert!(cal.is_business_day(date(2025, 12, 26)).unwrap());
    }

    /// Henry Hub: third-last business day of the month before the contract month.
    fn ng_last_trading_day(year: i16, month: i8) -> jiff::civil::Date {
        nymex().add_business_days(date(year, month, 1), -3).unwrap()
    }

    /// WTI: three business days before the 25th of the month before the
    /// contract month (or before the last business day preceding it).
    fn cl_last_trading_day(year: i16, month: i8) -> jiff::civil::Date {
        let cal = nymex();
        let (py, pm) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        let anchor = cal
            .adjust(date(py, pm, 25), crate::BusinessDayConvention::Preceding)
            .unwrap();
        cal.add_business_days(anchor, -3).unwrap()
    }

    #[test]
    fn nymex_henry_hub_expiries() {
        // ICE Henry LD1 expiration dates (ICE settlement files)
        assert_eq!(ng_last_trading_day(2021, 3), date(2021, 2, 24));
        assert_eq!(ng_last_trading_day(2021, 4), date(2021, 3, 29));
        assert_eq!(ng_last_trading_day(2021, 5), date(2021, 4, 28));
        assert_eq!(ng_last_trading_day(2026, 1), date(2025, 12, 29));
        // New Year 2028 on a Saturday: Fri 2027-12-31 counts
        assert_eq!(ng_last_trading_day(2028, 1), date(2027, 12, 29));
        // Memorial Day Mon 2032-05-31 does not count
        assert_eq!(ng_last_trading_day(2032, 6), date(2032, 5, 26));
    }

    #[test]
    fn nymex_wti_expiries() {
        // ICE WTI (T) expires one business day earlier (ICE settlement files): 2020-…
        // strips are outside the valid range, so from 2021 on.
        assert_eq!(cl_last_trading_day(2021, 7), date(2021, 6, 22)); // ICE: 06-21
        assert_eq!(cl_last_trading_day(2022, 2), date(2022, 1, 20)); // ICE: 01-19
        // Juneteenth observed Mon 2022-06-20; the 25th is a Saturday
        assert_eq!(cl_last_trading_day(2022, 7), date(2022, 6, 21)); // ICE: 06-17
        assert_eq!(cl_last_trading_day(2024, 7), date(2024, 6, 20)); // ICE: 06-18
        assert_eq!(cl_last_trading_day(2026, 7), date(2026, 6, 22)); // ICE: 06-18
    }
}
