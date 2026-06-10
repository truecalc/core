# Changelog

All notable changes to the truecalc workspace are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-06-11

### Breaking

- Engine flavor is now required: use `Engine::sheets()` or `Engine::excel()`. Free
  `evaluate`/`parse`/`validate` functions are deprecated and will be removed in this release.
- `Engine::google_sheets()` is deprecated; use `Engine::sheets()`.

### Added

- **truecalc-workbook**: workbook runtime (recalc, serialization, JSON schema v1).
- **truecalc-mcp**: workbook session tools (`workbook_create` / `workbook_set` /
  `workbook_get` / `workbook_recalc` / `workbook_export` / `workbook_import`).
- **@truecalc/workbook** (`crates/wasm-workbook`): full Workbook API compiled to
  WebAssembly, published to npm as `@truecalc/workbook`.
- PRF-keyed per-cell RNG: `RAND` / `RANDBETWEEN` / `RANDARRAY` are now deterministic
  under `RecalcContext`.
- `ErrorKind::Unsupported` for `Engine::excel()` evaluate (distinct from `#N/A`).

### Fixed

- `Engine::excel().evaluate` now returns `#UNSUPPORTED!` instead of `#N/A` for
  Google-Sheets-only functions.

## Packages

| Package                   | Version | Registry      |
|---------------------------|---------|---------------|
| `truecalc-core`           | 1.0.0   | crates.io     |
| `truecalc-workbook`       | 1.0.0   | crates.io     |
| `truecalc-mcp`            | 1.0.0   | crates.io     |
| `@truecalc/core`          | 1.0.0   | npm           |
| `@truecalc/workbook`      | 1.0.0   | npm           |

> **Note**: Publishing to crates.io and npm is owner-triggered per ADR Decision 6.
> Git tags will be created post-merge using the crate-prefixed convention, e.g.
> `truecalc-core-v1.0.0`, `truecalc-workbook-v1.0.0`, etc.
