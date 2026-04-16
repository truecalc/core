# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.0...ganit-core-v0.3.1) - 2026-04-16

### Added

- implement CONVERT unit conversion function ([#176](https://github.com/tryganit/ganit-core/pull/176))
- implement TO_DATE, TO_DOLLARS, TO_PERCENT, TO_PURE_NUMBER, TO_TEXT parser functions

### Fixed

- truncate mi3 volume literal to suppress clippy::excessive_precision

### Other

- Merge pull request #292 from tryganit/feat/176-convert
- activate m2_parser_conformance (all 6 parser functions implemented)

## [0.3.0](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.2.1...ganit-core-v0.3.0) - 2026-04-16

### Added

- add Value::Date and implement ISDATE (closes #208)
- implement CELL function (closes #215)
- implement ISREF and ISFORMULA (closes #211, #213)
- *(math)* implement COUNTIF, SUMIF, AVERAGEIF (#273, #274, #275)
- *(text)* implement SEARCH with wildcard support ([#271](https://github.com/tryganit/ganit-core/pull/271))
- *(statistical)* implement COUNTBLANK ([#272](https://github.com/tryganit/ganit-core/pull/272))
- *(text)* implement PROPER function ([#270](https://github.com/tryganit/ganit-core/pull/270))
- *(parser)* add {} array literal syntax ([#269](https://github.com/tryganit/ganit-core/pull/269))
- *(date)* implement all 26 M2 date/time functions ([#75](https://github.com/tryganit/ganit-core/pull/75))
- *(date)* scaffold 26 date/time function stubs
- *(tests)* add Google Sheets oracle fixtures for M2, M3, M4 conformance

### Fixed

- align CELL info_type list with Google Sheets docs
- use range contains() for clippy manual_range_contains lint
- use is_empty() for clippy len_zero lint
- *(clippy)* remove unused ErrorKind import in countblank
- *(clippy)* resolve 4 clippy warnings in date functions
- *(tests)* mark M2/M3/M4 conformance tests as pending until implemented

### Other

- Expand M2 conformance coverage for issue #276 functions

## [0.2.1](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.2.0...ganit-core-v0.2.1) - 2026-04-15

### Other

- Merge pull request #266 from tryganit/fix/registry-driven-list-functions
- replace static function tables with live registry reference

## [0.2.0](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.6...ganit-core-v0.2.0) - 2026-04-15

### Fixed

- *(mcp)* make list_functions registry-driven, delete static catalogue

## [0.1.6](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.4...ganit-core-v0.1.6) - 2026-04-15

### Other

- release v0.1.5
- Merge pull request #73 from tryganit/docs/readme-badges-and-usage
- add badges and per-crate READMEs for crates.io and npm

## [0.1.5](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.4...ganit-core-v0.1.5) - 2026-04-15

### Other

- Merge pull request #73 from tryganit/docs/readme-badges-and-usage
- add badges and per-crate READMEs for crates.io and npm

## [0.1.4](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.3...ganit-core-v0.1.4) - 2026-04-15

### Added

- *(eval)* implement wave-1 M1 functions (#49–#53, #56–#58)

### Fixed

- *(clippy)* use is_some() instead of if let Some(_) pattern
- *(conformance)* pass all 6 M1 oracle conformance test suites

## [0.1.3](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.1...ganit-core-v0.1.3) - 2026-04-15

### Other

- release v0.1.2
- add M1 oracle conformance harness driven by Google Sheets

## [0.1.2](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.1...ganit-core-v0.1.2) - 2026-04-15

### Other

- add M1 oracle conformance harness driven by Google Sheets

## [0.1.1](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.1.0...ganit-core-v0.1.1) - 2026-04-15

### Fixed

- *(core)* evaluate() takes variables by reference ([#34](https://github.com/tryganit/ganit-core/pull/34))
