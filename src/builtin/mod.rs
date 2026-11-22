//! Built-in holiday rules and market calendars.
//!
//! This module provides two layers:
//!
//! - [`holidays`] — individual holiday rule constructors (the atoms)
//! - [`calendars`] — pre-assembled market calendars that compose those rules
//!
//! Two catalogue functions collect everything the library knows about:
//!
//! - [`all_rules`] — every distinct holiday rule, for use in
//!   [`RuleRegistry`](crate::registry::RuleRegistry) and rule fitting
//! - [`all_calendars`] — every pre-assembled calendar, for use in
//!   [`explain_dates`](crate::compare::explain_dates); [`calendar`] looks one
//!   up by name

pub mod calendars;
pub mod holidays;

use std::sync::Arc;

use crate::rule_calendar::RuleBasedCalendar;
use crate::rules::HolidayRule;

/// Returns every distinct built-in holiday rule.
///
/// This is the global rule pool — one entry per unique rule definition across
/// all built-in calendars. Rules that appear in multiple calendars (e.g.
/// Christmas Day in TARGET, US, and UK) are included once.
///
/// The returned rules are suitable for populating a
/// [`RuleRegistry`](crate::registry::RuleRegistry) or for
/// rule-fitting analysis.
///
/// # Examples
///
/// ```
/// use fincal::builtin::all_rules;
///
/// let rules = all_rules();
/// assert!(rules.len() >= 20); // one entry per distinct rule across the builtin calendars
/// ```
pub fn all_rules() -> Vec<Arc<dyn HolidayRule>> {
    vec![
        // --- Shared ---
        holidays::new_years_day(),
        holidays::good_friday(),
        holidays::easter_monday(),
        holidays::christmas_day(),
        // --- TARGET-specific ---
        holidays::labour_day(),
        holidays::st_stephens_day(),
        // --- EEX-specific ---
        holidays::christmas_eve(),
        holidays::new_years_eve(),
        // --- US-specific ---
        holidays::us_new_years_day_observed(),
        holidays::mlk_day(),
        holidays::presidents_day(),
        holidays::memorial_day(),
        holidays::juneteenth(),
        holidays::us_juneteenth_observed(),
        holidays::independence_day(),
        holidays::us_independence_day_observed(),
        holidays::labor_day(),
        holidays::columbus_day(),
        holidays::veterans_day(),
        holidays::us_veterans_day_observed(),
        holidays::thanksgiving(),
        holidays::day_after_thanksgiving(),
        holidays::us_christmas_day_observed(),
        // --- UK-specific ---
        holidays::uk_new_years_day_observed(),
        holidays::early_may_bank_holiday(),
        holidays::spring_bank_holiday(),
        holidays::summer_bank_holiday(),
        holidays::boxing_day(),
        holidays::uk_christmas_day_observed(),
        holidays::uk_boxing_day_observed(),
        // --- UK moved bank holidays ---
        holidays::ve_day_2020(),
        holidays::spring_bank_holiday_2002(),
        holidays::spring_bank_holiday_2012(),
        holidays::spring_bank_holiday_2022(),
        // --- UK one-off bank holidays ---
        holidays::queens_golden_jubilee(),
        holidays::royal_wedding_2011(),
        holidays::queens_diamond_jubilee(),
        holidays::queens_platinum_jubilee(),
        holidays::queens_funeral(),
        holidays::coronation_bank_holiday(),
        // --- Baltic Exchange ---
        holidays::baltic_christmas(),
        holidays::baltic_us_mlk_day(),
        holidays::baltic_us_juneteenth(),
        holidays::baltic_us_juneteenth_observed(),
        // --- Platts US-specific ---
        holidays::platts_juneteenth(),
        holidays::platts_juneteenth_observed(),
        holidays::platts_us_july4_bridge(),
        holidays::platts_us_day_after_christmas_2025(),
        // --- Platts US gas-specific ---
        holidays::platts_gas_columbus_day(),
        holidays::platts_gas_veterans_day(),
        holidays::platts_gas_veterans_day_observed(),
        holidays::platts_gas_day_after_new_year_2026(),
        // --- OPIS-specific ---
        holidays::opis_day_after_christmas_2025(),
        // --- ICIS-specific ---
        holidays::icis_europe_chemicals_year_end(),
        holidays::icis_cn_weekend(),
        holidays::china_exchanges_cny_eve(),
        holidays::china_new_year(),
        holidays::china_lunar_new_year(),
        holidays::china_qingming(),
        holidays::china_labour_day(),
        holidays::china_dragon_boat(),
        holidays::china_mid_autumn(),
        holidays::china_golden_week(),
        // --- Argus-specific ---
        holidays::argus_year_end_2018(),
        holidays::blpg1_hormuz_weekly(),
        holidays::blpg1_hormuz_tue_thu(),
        // --- Singapore-specific ---
        holidays::sg_national_day(),
        holidays::sg_national_day_observed(),
        holidays::sg_new_years_day_observed(),
        holidays::sg_labour_day_observed(),
        holidays::sg_christmas_day_observed(),
        holidays::sg_chinese_new_year(),
        holidays::sg_hari_raya_puasa(),
        holidays::sg_hari_raya_haji(),
        holidays::sg_vesak_day(),
        holidays::sg_deepavali(),
        holidays::sg50_public_holiday(),
        holidays::sg_polling_day(),
        // --- Platts-specific SG observed (SundayToMonday) ---
        holidays::sg_sunmon_new_years_day_observed(),
        holidays::sg_sunmon_labour_day_observed(),
        holidays::sg_sunmon_national_day_observed(),
        holidays::sg_sunmon_christmas_day_observed(),
        // --- Platts global one-off closures ---
        holidays::platts_year_end_2018(),
        // --- ICE Futures Europe observed (SundayToMonday) ---
        holidays::ice_christmas_day_observed(),
        holidays::ice_new_years_day_observed(),
        // --- CME Group / NYMEX ---
        holidays::cme_new_years_day_observed(),
        holidays::cme_juneteenth(),
        holidays::cme_juneteenth_observed(),
    ]
}

/// Returns every built-in market calendar.
///
/// Currently: TARGET, EEXPowerDerivatives, USSettlement, UKSettlement,
/// PlattsEuropeanMarketScan, ICEUK, SGX, BalticLPG, BalticUS, BalticAsia,
/// PlattsAsiaMarketScan, PlattsUSMarketScan, PlattsUSGasDaily, OPISPetroChemWire,
/// ICISEuropeChemicals, ICISChinaChemicals, ChinaFuturesExchanges,
/// ArgusAsiaLPG, ArgusEuropeLPG, BalticLPG1Hormuz, ICEUKAveraging, NYMEX,
/// ICEEndex.
///
/// # Examples
///
/// ```
/// use fincal::builtin::all_calendars;
///
/// let cals = all_calendars();
/// assert_eq!(cals.len(), 23);
/// ```
pub fn all_calendars() -> Vec<RuleBasedCalendar> {
    vec![
        calendars::target(),
        calendars::eex_power_derivatives(),
        calendars::us_settlement(),
        calendars::uk_settlement(),
        calendars::platts_european_market_scan(),
        calendars::ice_uk(),
        calendars::sgx(),
        calendars::baltic_lpg(),
        calendars::baltic_us(),
        calendars::baltic_asia(),
        calendars::platts_asia_marketscan(),
        calendars::platts_us_marketscan(),
        calendars::platts_us_gas_daily(),
        calendars::opis_petrochem_wire(),
        calendars::icis_europe_chemicals(),
        calendars::icis_china_chemicals(),
        calendars::china_futures_exchanges(),
        calendars::argus_asia_lpg(),
        calendars::argus_europe_lpg(),
        calendars::baltic_lpg1_hormuz(),
        calendars::ice_uk_averaging(),
        calendars::nymex(),
        calendars::ice_endex(),
    ]
}

/// Returns the built-in calendar with the given name, or `None`.
///
/// The match ignores ASCII case (`"iceuk"` finds `ICEUK`). Calendar names
/// are a stable interface — configurations refer to calendars by name — so
/// use this instead of searching [`all_calendars`] by hand.
///
/// # Examples
///
/// ```
/// use fincal::HolidayCalendar;
/// use fincal::builtin::calendar;
///
/// assert_eq!(calendar("target").unwrap().name(), "TARGET");
/// assert!(calendar("NoSuchCalendar").is_none());
/// ```
pub fn calendar(name: &str) -> Option<RuleBasedCalendar> {
    use crate::HolidayCalendar;
    all_calendars()
        .into_iter()
        .find(|c| c.name().eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HolidayCalendar;
    use std::collections::HashSet;

    #[test]
    fn all_rules_has_no_duplicate_names() {
        let rules = all_rules();
        let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
        let unique: HashSet<&str> = names.iter().copied().collect();
        assert_eq!(
            names.len(),
            unique.len(),
            "duplicate rule names: {:?}",
            names
        );
    }

    #[test]
    fn all_rules_count() {
        assert_eq!(all_rules().len(), 88);
    }

    #[test]
    fn all_rules_registers_every_holiday_constructor() {
        // A rule missing from all_rules() is invisible to the registry, the
        // fit/explain layer, and `fincal explain` — and no other test catches
        // the omission. Every `pub fn` in builtin/holidays.rs is a rule
        // constructor, so the counts must match exactly.
        let constructors = include_str!("holidays.rs")
            .lines()
            .filter(|line| line.starts_with("pub fn "))
            .count();
        assert_eq!(
            all_rules().len(),
            constructors,
            "all_rules() must register every `pub fn` in builtin/holidays.rs"
        );
    }

    #[test]
    fn all_calendars_registers_every_calendar_constructor() {
        // A calendar missing from all_calendars() cannot be found by name —
        // by the CLI, by a consumer's configuration, or by any ranking. Every top-level
        // `pub fn` in builtin/calendars.rs is a calendar constructor (the
        // shared rule-set helpers are private), so the counts must match.
        let constructors = include_str!("calendars.rs")
            .lines()
            .filter(|line| line.starts_with("pub fn "))
            .count();
        assert_eq!(
            all_calendars().len(),
            constructors,
            "all_calendars() must register every `pub fn` in builtin/calendars.rs"
        );
    }

    #[test]
    fn all_calendars_count() {
        assert_eq!(all_calendars().len(), 23);
    }

    #[test]
    fn all_calendars_names() {
        let cals = all_calendars();
        let names: Vec<&str> = cals.iter().map(|c| c.name()).collect();
        assert_eq!(
            names,
            vec![
                "TARGET",
                "EEXPowerDerivatives",
                "USSettlement",
                "UKSettlement",
                "PlattsEuropeanMarketScan",
                "ICEUK",
                "SGX",
                "BalticLPG",
                "BalticUS",
                "BalticAsia",
                "PlattsAsiaMarketScan",
                "PlattsUSMarketScan",
                "PlattsUSGasDaily",
                "OPISPetroChemWire",
                "ICISEuropeChemicals",
                "ICISChinaChemicals",
                "ChinaFuturesExchanges",
                "ArgusAsiaLPG",
                "ArgusEuropeLPG",
                "BalticLPG1Hormuz",
                "ICEUKAveraging",
                "NYMEX",
                "ICEEndex"
            ],
        );
    }
}
