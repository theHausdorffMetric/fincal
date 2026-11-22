//! Rule registry — a shared pool of holiday rules queryable by date.
//!
//! The [`RuleRegistry`] collects [`HolidayRule`]s from
//! multiple calendars into a single lookup structure. This enables questions
//! like "which rules fire on this date?" without going through a specific calendar.

use std::sync::Arc;

use jiff::civil::Date;

use crate::rules::HolidayRule;

/// A shared pool of [`HolidayRule`]s that can be queried independently of any
/// specific calendar.
///
/// Rules are stored as [`Arc`] references, allowing them to be shared with
/// [`RuleBasedCalendar`](crate::rule_calendar::RuleBasedCalendar)s.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use fincal::registry::RuleRegistry;
/// use fincal::rules::FixedDate;
/// use jiff::civil::date;
///
/// let mut reg = RuleRegistry::new();
/// reg.add(Arc::new(FixedDate::new("Christmas Day", 12, 25)));
///
/// let matching = reg.explain_date(date(2025, 12, 25));
/// assert_eq!(matching.len(), 1);
/// assert_eq!(matching[0].name(), "Christmas Day");
/// ```
pub struct RuleRegistry {
    rules: Vec<Arc<dyn HolidayRule>>,
}

impl RuleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a single rule to the registry.
    pub fn add(&mut self, rule: Arc<dyn HolidayRule>) {
        self.rules.push(rule);
    }

    /// Adds multiple rules to the registry.
    pub fn add_many(&mut self, rules: impl IntoIterator<Item = Arc<dyn HolidayRule>>) {
        self.rules.extend(rules);
    }

    /// Returns a slice of all registered rules.
    pub fn rules(&self) -> &[Arc<dyn HolidayRule>] {
        &self.rules
    }

    /// Returns all rules that produce the given date as a holiday.
    ///
    /// A rule whose data coverage ([`HolidayRule::valid_range`]) does not
    /// include the date simply produces nothing, so its absence from the
    /// result can mean either "not a holiday under this rule" or "outside
    /// the rule's coverage" — consult `valid_range()` to distinguish.
    /// (Calendar-level [`explain_date`](crate::rule_calendar::RuleBasedCalendar::explain_date)
    /// does not share this ambiguity: a calendar's build step guarantees its
    /// valid range lies within every constituent rule's coverage.)
    pub fn explain_date(&self, date: Date) -> Vec<Arc<dyn HolidayRule>> {
        self.rules
            .iter()
            .filter(|r| r.holidays_in(date.year()).contains(&date))
            .cloned()
            .collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{FixedDate, ObservancePolicy, ObservedFixedDate};
    use jiff::civil::date;

    #[test]
    fn empty_registry() {
        let reg = RuleRegistry::new();
        assert!(reg.rules().is_empty());
        assert!(reg.explain_date(date(2025, 12, 25)).is_empty());
    }

    #[test]
    fn add_and_query() {
        let mut reg = RuleRegistry::new();
        reg.add(Arc::new(FixedDate::new("Christmas Day", 12, 25)));
        reg.add(Arc::new(FixedDate::new("New Year's Day", 1, 1)));

        let matching = reg.explain_date(date(2025, 12, 25));
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].name(), "Christmas Day");

        let matching = reg.explain_date(date(2025, 6, 15));
        assert!(matching.is_empty());
    }

    #[test]
    fn add_many() {
        let mut reg = RuleRegistry::new();
        reg.add_many(vec![
            Arc::new(FixedDate::new("A", 1, 1)) as Arc<dyn HolidayRule>,
            Arc::new(FixedDate::new("B", 12, 25)),
        ]);
        assert_eq!(reg.rules().len(), 2);
    }

    #[test]
    fn cross_year_query() {
        let mut reg = RuleRegistry::new();
        // 2022: Jan 1 is Saturday -> observed Fri Dec 31, 2021
        reg.add(Arc::new(ObservedFixedDate::new(
            "New Year's Day (Observed)",
            1,
            1,
            ObservancePolicy::SaturdayToFridaySundayToMonday,
        )));

        let matching = reg.explain_date(date(2021, 12, 31));
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].name(), "New Year's Day (Observed)");
    }

    #[test]
    fn default_is_empty() {
        let reg = RuleRegistry::default();
        assert!(reg.rules().is_empty());
    }
}
