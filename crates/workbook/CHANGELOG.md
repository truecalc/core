# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [6.1.0](https://github.com/truecalc/core/compare/truecalc-workbook-v6.0.1...truecalc-workbook-v6.1.0) - 2026-07-25

### Fixed

- fully flatten nested arrays in remaining statistical functions
- preserve column orientation for vertical ranges in elementwise ops

## [6.0.1](https://github.com/truecalc/core/compare/truecalc-workbook-v6.0.0...truecalc-workbook-v6.0.1) - 2026-07-22

### Other

- remove hardcoded function counts, add live badges + download badges

## [6.0.0](https://github.com/truecalc/core/compare/truecalc-workbook-v5.0.3...truecalc-workbook-v6.0.0) - 2026-07-22

### Added

- accept #REF! and the error-literal family as parser tokens

## [5.0.0](https://github.com/truecalc/core/compare/truecalc-workbook-v4.0.0...truecalc-workbook-v5.0.0) - 2026-07-20

### Added

- *(core,workbook)* reach the per-node eval hook through workbook recalc

### Fixed

- *(workbook)* match trace_cell's spill and cycle behavior to recalc

## [4.0.0](https://github.com/truecalc/core/compare/truecalc-workbook-v3.3.0...truecalc-workbook-v4.0.0) - 2026-07-15

### Added

- *(core)* carry optional diagnostic messages on eval errors

### Fixed

- *(core)* propagate ErrorMsg everywhere + exact Sheets wording

## [3.2.0](https://github.com/truecalc/core/compare/truecalc-workbook-v3.1.0...truecalc-workbook-v3.2.0) - 2026-07-15

### Added

- *(core)* preserve Date type through arithmetic and add Date-typed set input

## [3.0.0](https://github.com/truecalc/core/compare/truecalc-workbook-v2.0.1...truecalc-workbook-v3.0.0) - 2026-07-13

### Fixed

- dedupe Unresolved precedents regardless of $ anchors

## [2.0.0](https://github.com/truecalc/core/compare/truecalc-workbook-v1.0.2...truecalc-workbook-v2.0.0) - 2026-06-23

### Added

- *(core)* TZNOW (deterministic clock), TZINWINDOW, TZCANONICAL
- *(wasm,mcp,workbook)* serialize Value::Zoned across all surfaces

## [1.0.2](https://github.com/truecalc/core/compare/truecalc-workbook-v1.0.0...truecalc-workbook-v1.0.2) - 2026-06-10

### Other

- release v1.0.1
- rustdoc, READMEs, cookbook, migration guide ([#548](https://github.com/truecalc/core/pull/548))

## [1.0.1](https://github.com/truecalc/core/compare/truecalc-workbook-v1.0.0...truecalc-workbook-v1.0.1) - 2026-06-10

### Other

- rustdoc, READMEs, cookbook, migration guide ([#548](https://github.com/truecalc/core/pull/548))

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
