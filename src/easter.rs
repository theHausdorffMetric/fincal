//! Easter Sunday computation using the Meeus/Jones/Butcher algorithm.
//!
//! This module provides a single function [`easter_sunday`] that computes the
//! date of Easter Sunday for any given year. The algorithm is the anonymous
//! Gregorian algorithm (often called Meeus/Jones/Butcher), which is valid for
//! all years in the Gregorian calendar.

use jiff::civil::Date;

/// Computes Easter Sunday for a given year using the Meeus/Jones/Butcher algorithm.
///
/// This is the anonymous Gregorian algorithm, valid for all years in the
/// Gregorian calendar (1583 onwards, though it will produce dates for earlier
/// years too).
///
/// # Panics
///
/// Panics if the computed date cannot be represented by [`jiff::civil::Date`].
///
/// # Examples
///
/// ```
/// use fincal::easter::easter_sunday;
/// use jiff::civil::date;
///
/// assert_eq!(easter_sunday(2025), date(2025, 4, 20));
/// ```
pub fn easter_sunday(year: i16) -> Date {
    let y = year as i32;
    let a = y % 19;
    let b = y / 100;
    let c = y % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    Date::new(year, month as i8, day as i8).expect("invalid easter date")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn known_easter_dates_2020_to_2030() {
        let expected = [
            (2020, date(2020, 4, 12)),
            (2021, date(2021, 4, 4)),
            (2022, date(2022, 4, 17)),
            (2023, date(2023, 4, 9)),
            (2024, date(2024, 3, 31)),
            (2025, date(2025, 4, 20)),
            (2026, date(2026, 4, 5)),
            (2027, date(2027, 3, 28)),
            (2028, date(2028, 4, 16)),
            (2029, date(2029, 4, 1)),
            (2030, date(2030, 4, 21)),
        ];
        for (year, expected_date) in expected {
            assert_eq!(easter_sunday(year), expected_date, "Easter {year} mismatch");
        }
    }

    #[test]
    fn early_and_late_easters() {
        // Earliest possible: March 22
        assert_eq!(easter_sunday(1818), date(1818, 3, 22));
        // Latest possible: April 25
        assert_eq!(easter_sunday(1943), date(1943, 4, 25));
    }
}
