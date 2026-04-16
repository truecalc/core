# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.12](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.10...ganit-core-v0.3.12) - 2026-04-16

### Other

- Merge pull request #320 from tryganit/release-plz-2026-04-16T22-48-53Z
- activate M2 info and logical conformance tests
- Merge pull request #321 from tryganit/feat/120-activate-engineering-conformance
- activate M2 engineering conformance and fix 10 failures

## [0.3.11](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.10...ganit-core-v0.3.11) - 2026-04-16

### Other

- Merge pull request #321 from tryganit/feat/120-activate-engineering-conformance
- activate M2 engineering conformance and fix 10 failures

## [0.3.10](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.9...ganit-core-v0.3.10) - 2026-04-16

### Added

- implement BIN2DEC/HEX/OCT, DEC2BIN/HEX/OCT, HEX2BIN/DEC/OCT, OCT2BIN/DEC/HEX

## [0.3.9](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.8...ganit-core-v0.3.9) - 2026-04-16

### Added

- implement BITAND, BITOR, BITXOR, BITLSHIFT, BITRSHIFT, DELTA, GESTEP

## [0.3.8](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.6...ganit-core-v0.3.8) - 2026-04-16

### Added

- implement COUNTBLANK, COUNTUNIQUE, COUNTIFS, SUMIFS
- implement COMBIN, COMBINA, MULTINOMIAL, GCD, LCM
- implement SQRTPI, SUMSQ, FACTDOUBLE, SERIESSUM
- implement CEILING.MATH, CEILING.PRECISE, FLOOR.MATH, FLOOR.PRECISE, ISO.CEILING

### Fixed

- remove stray countifs/countunique/sumifs module stubs from mod.rs
- remove spurious pub mod entries from math mod.rs

### Other

- release v0.3.7
- Merge pull request #311 from tryganit/feat/105-math-base-conversion
- Merge pull request #310 from tryganit/feat/105-math-simple
- Merge pull request #309 from tryganit/feat/105-math-advanced-rounding

## [0.3.7](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.6...ganit-core-v0.3.7) - 2026-04-16

### Added

- implement COUNTBLANK, COUNTUNIQUE, COUNTIFS, SUMIFS
- implement COMBIN, COMBINA, MULTINOMIAL, GCD, LCM
- implement SQRTPI, SUMSQ, FACTDOUBLE, SERIESSUM
- implement CEILING.MATH, CEILING.PRECISE, FLOOR.MATH, FLOOR.PRECISE, ISO.CEILING

### Fixed

- remove stray countifs/countunique/sumifs module stubs from mod.rs
- remove spurious pub mod entries from math mod.rs

### Other

- Merge pull request #311 from tryganit/feat/105-math-base-conversion
- Merge pull request #310 from tryganit/feat/105-math-simple
- Merge pull request #309 from tryganit/feat/105-math-advanced-rounding

## [0.3.6](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.4...ganit-core-v0.3.6) - 2026-04-16

### Added

- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement REGEXMATCH, REGEXEXTRACT, REGEXREPLACE (#89 group C)
- implement ASC, JOIN, SPLIT, TEXTJOIN text functions
- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement ARABIC, ROMAN, CLEAN, FIXED, DOLLAR text functions

### Fixed

- replace regex with regex-lite to reduce WASM binary size
- remove stray module declarations from other PRs in text/mod.rs

### Other

- release v0.3.5
- replace flat tests.rs with tests/ subdirectory structure for arabic, clean, dollar, fixed, roman

## [0.3.5](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.4...ganit-core-v0.3.5) - 2026-04-16

### Added

- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement REGEXMATCH, REGEXEXTRACT, REGEXREPLACE (#89 group C)
- implement ASC, JOIN, SPLIT, TEXTJOIN text functions
- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement ARABIC, ROMAN, CLEAN, FIXED, DOLLAR text functions

### Fixed

- replace regex with regex-lite to reduce WASM binary size
- remove stray module declarations from other PRs in text/mod.rs

### Other

- replace flat tests.rs with tests/ subdirectory structure for arabic, clean, dollar, fixed, roman

## [0.3.4](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.2...ganit-core-v0.3.4) - 2026-04-16

### Added

- implement 14 order statistics functions (M2 #82)
- implement shape/distribution statistical functions (M2 #82)
- implement variance, stddev, covariance, and deviation statistical functions

### Fixed

- restore shape-stats module files deleted during rebase conflict resolution

### Other

- release v0.3.3
- enable m2 statistical conformance test (all 46 functions implemented)
- Merge pull request #298 from tryganit/feat/82-order-stats
- add unit tests for order statistics functions; remove shape-stats duplicates
- add edge tests and failure tests for shape-stats functions
- add unit tests for variance/stddev statistical functions

## [0.3.3](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.2...ganit-core-v0.3.3) - 2026-04-16

### Added

- implement 14 order statistics functions (M2 #82)
- implement shape/distribution statistical functions (M2 #82)
- implement variance, stddev, covariance, and deviation statistical functions

### Fixed

- restore shape-stats module files deleted during rebase conflict resolution

### Other

- enable m2 statistical conformance test (all 46 functions implemented)
- Merge pull request #298 from tryganit/feat/82-order-stats
- add unit tests for order statistics functions; remove shape-stats duplicates
- add edge tests and failure tests for shape-stats functions
- add unit tests for variance/stddev statistical functions

## [0.3.2](https://github.com/tryganit/ganit-core/compare/ganit-core-v0.3.0...ganit-core-v0.3.2) - 2026-04-16

### Added

- implement CONVERT unit conversion function ([#176](https://github.com/tryganit/ganit-core/pull/176))
- implement TO_DATE, TO_DOLLARS, TO_PERCENT, TO_PURE_NUMBER, TO_TEXT parser functions

### Fixed

- truncate mi3 volume literal to suppress clippy::excessive_precision

### Other

- release v0.3.1
- Merge pull request #292 from tryganit/feat/176-convert
- activate m2_parser_conformance (all 6 parser functions implemented)

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
