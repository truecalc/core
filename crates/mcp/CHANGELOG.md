# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.19...truecalc-mcp-v0.5.0) - 2026-04-22

### Added

- *(mcp)* add --conformance flag and per-call conformance override

### Other

- separate test files from production code

## [0.4.19](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.18...truecalc-mcp-v0.4.19) - 2026-04-21

### Other

- update Cargo.lock dependencies

## [0.4.16](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.15...truecalc-mcp-v0.4.16) - 2026-04-21

### Other

- update Cargo.lock dependencies

## [0.4.14](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.13...truecalc-mcp-v0.4.14) - 2026-04-20

### Other

- update Cargo.lock dependencies

## [0.4.11](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.10...truecalc-mcp-v0.4.11) - 2026-04-19

### Other

- update Cargo.lock dependencies

## [0.4.9](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.8...truecalc-mcp-v0.4.9) - 2026-04-19

### Added

- *(mcp)* add get_stats tool returning version and per-category function counts

## [0.4.8](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.7...truecalc-mcp-v0.4.8) - 2026-04-19

### Other

- update Cargo.lock dependencies

## [0.4.5](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.4...truecalc-mcp-v0.4.5) - 2026-04-18

### Other

- release v0.4.5

## [0.4.4](https://github.com/truecalc/core/releases/tag/truecalc-mcp-v0.4.4) - 2026-04-18

### Added

- *(rebrand)* update all URLs, READMEs, npm package, and docs to truecalc
- *(rebrand)* update Rust source files to use truecalc_core
- *(rebrand)* rename Rust crates to truecalc-* in Cargo.toml
- add Value::Date and implement ISDATE (closes #208)
- *(mcp)* implement 5-tool MCP server over stdio — issue #9

### Fixed

- *(mcp)* make list_functions registry-driven, delete static catalogue
- *(core)* evaluate() takes variables by reference ([#34](https://github.com/truecalc/core/pull/34))
- *(publish)* add version to ganit-core path dep in ganit-mcp
- *(publish)* add description and repository to crate manifests
- *(mcp)* JSON-RPC parse errors, isError flag, graceful EOF, remove hot-path unwrap

### Other

- release v0.4.3
- Merge pull request #363 from tryganit/docs/mcp-readme-badges-install
- *(mcp)* add ganit-core badge and use --force for install
- release v0.4.0
- release v0.3.11
- release v0.3.8
- release v0.3.6
- release v0.3.4
- release v0.3.2
- release v0.3.0
- release v0.2.1
- Merge pull request #266 from tryganit/fix/registry-driven-list-functions
- replace static function tables with live registry reference
- *(mcp)* add README and wire readme field in Cargo.toml
- release v0.1.6
- release v0.1.3
- release v0.1.1
- Set up Cargo workspace with core, wasm, and mcp crates

## [0.4.3](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.2...truecalc-mcp-v0.4.3) - 2026-04-17

### Other

- Merge pull request #363 from truecalc/docs/mcp-readme-badges-install
- *(mcp)* add truecalc-core badge and use --force for install

## [0.4.2](https://github.com/truecalc/core/compare/truecalc-mcp-v0.4.1...truecalc-mcp-v0.4.2) - 2026-04-17

### Other

- update Cargo.lock dependencies

## [0.3.12](https://github.com/truecalc/core/compare/truecalc-mcp-v0.3.11...truecalc-mcp-v0.3.12) - 2026-04-16

### Other

- update Cargo.lock dependencies

## [0.3.8](https://github.com/truecalc/core/compare/truecalc-mcp-v0.3.7...truecalc-mcp-v0.3.8) - 2026-04-16

### Other

- update Cargo.lock dependencies

## [0.3.6](https://github.com/truecalc/core/compare/truecalc-mcp-v0.3.5...truecalc-mcp-v0.3.6) - 2026-04-16

### Other

- update Cargo.lock dependencies

## [0.3.4](https://github.com/truecalc/core/compare/truecalc-mcp-v0.3.3...truecalc-mcp-v0.3.4) - 2026-04-16

### Other

- update Cargo.lock dependencies

## [0.3.2](https://github.com/truecalc/core/compare/truecalc-mcp-v0.3.1...truecalc-mcp-v0.3.2) - 2026-04-16

### Other

- update Cargo.lock dependencies

## [0.3.0](https://github.com/truecalc/core/compare/truecalc-mcp-v0.2.1...truecalc-mcp-v0.3.0) - 2026-04-16

### Added

- add Value::Date and implement ISDATE (closes #208)

## [0.2.1](https://github.com/truecalc/core/compare/truecalc-mcp-v0.2.0...truecalc-mcp-v0.2.1) - 2026-04-15

### Other

- Merge pull request #266 from truecalc/fix/registry-driven-list-functions
- replace static function tables with live registry reference
- *(mcp)* add README and wire readme field in Cargo.toml

## [0.2.0](https://github.com/truecalc/core/compare/truecalc-mcp-v0.1.6...truecalc-mcp-v0.2.0) - 2026-04-15

### Fixed

- *(mcp)* make list_functions registry-driven, delete static catalogue

## [0.1.6](https://github.com/truecalc/core/compare/truecalc-mcp-v0.1.5...truecalc-mcp-v0.1.6) - 2026-04-15

### Other

- update Cargo.lock dependencies

## [0.1.3](https://github.com/truecalc/core/compare/truecalc-mcp-v0.1.2...truecalc-mcp-v0.1.3) - 2026-04-15

### Other

- update Cargo.lock dependencies

## [0.1.1](https://github.com/truecalc/core/compare/truecalc-mcp-v0.1.0...truecalc-mcp-v0.1.1) - 2026-04-15

### Fixed

- *(core)* evaluate() takes variables by reference ([#34](https://github.com/truecalc/core/pull/34))
