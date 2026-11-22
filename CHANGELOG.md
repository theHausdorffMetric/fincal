# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/) (0.y: a minor bump may break).

## [Unreleased]

## [0.21.0]

First public release. Versions up to 0.21.0 were developed privately; their
history is not part of this repository.

### Added

- Rule layer: `HolidayRule` atoms (fixed and observed dates, nth/last weekday,
  Easter offsets, date lists, …) that compute a holiday's date(s) for any year.
- Calendar layer: `HolidayCalendar` with time zone, weekend pattern and valid
  range; `RuleBasedCalendar` builder; business-day queries and ISDA date
  adjustment.
- Built-in calendars for settlement systems, exchanges and benchmark
  publishers (among them UK and US settlement, TARGET, ICE UK, ICE Endex,
  NYMEX, Platts, Argus, OPIS, ICIS, Baltic Exchange), each documented with its
  sources and verification.
- Analysis layer: compare calendars, rank calendars against observed dates,
  fit rules to explain gaps in a series; `RuleRegistry` of all known rules.
- `fincal` CLI (feature `cli`, default on): `calendars`, `holidays`, `explain`,
  `match`, `horizon`.

[Unreleased]: https://github.com/theHausdorffMetric/fincal/compare/v0.21.0...HEAD
[0.21.0]: https://github.com/theHausdorffMetric/fincal/releases/tag/v0.21.0
