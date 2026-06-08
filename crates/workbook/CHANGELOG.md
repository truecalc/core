# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0](https://github.com/truecalc/core/compare/truecalc-workbook-v0.7.0...truecalc-workbook-v0.8.0) - 2026-06-08

### Added

- *(workbook)* array spill — Sheets semantics (P3.5)
- *(workbook)* dependency graph from extract_refs (P3.2, #534)
- *(workbook)* sparse grid, A1 addressing, and sheet management (P3.1)

### Fixed

- *(workbook)* incremental ≡ full across spill shrink and unblock ([#591](https://github.com/truecalc/core/pull/591))
- *(workbook)* unify EngineFlavor with truecalc-core's flavor enum ([#567](https://github.com/truecalc/core/pull/567))

### Other

- recalc engine — full, incremental, cycles, RecalcContext
- Merge branch 'main' into feat/536-mutation-api
- *(workbook)* order mod depgraph alphabetically in lib.rs

## [0.7.0](https://github.com/truecalc/core/compare/truecalc-workbook-v0.6.5...truecalc-workbook-v0.7.0) - 2026-06-08

### Added

- *(workbook)* canonical JCS serializer + strict from_json (P2.3)
- Workbook/Worksheet/Cell value types + serde

### Fixed

- *(workbook)* enable serde_json float_roundtrip for byte-stable round-trip
- *(review)* enforce schema-version reader rule, reject null formula and 1x1 arrays

### Other

- *(workbook)* round-trip + schema-validation property tests + v1 schema (P2.4)
