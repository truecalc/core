# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0](https://github.com/truecalc/core/compare/truecalc-workbook-v0.6.5...truecalc-workbook-v0.7.0) - 2026-06-08

### Added

- *(workbook)* canonical JCS serializer + strict from_json (P2.3)
- Workbook/Worksheet/Cell value types + serde

### Fixed

- *(workbook)* enable serde_json float_roundtrip for byte-stable round-trip
- *(review)* enforce schema-version reader rule, reject null formula and 1x1 arrays

### Other

- *(workbook)* round-trip + schema-validation property tests + v1 schema (P2.4)
