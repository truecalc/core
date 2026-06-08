//! truecalc-workbook: the workbook layer for the truecalc spreadsheet engine.
//!
//! A [workbook] is a *value object*: an engine-locked collection of ordered
//! worksheets, sparse cell grids, and workbook-scoped named ranges. Its JSON
//! schema is the cross-surface contract — the same canonical bytes are
//! produced on every distribution surface (Rust, WASM, MCP, REST).
//!
//! The engine flavor (`sheets` | `excel`) is explicit and required at
//! workbook creation and immutable for the workbook's lifetime
//! (ADR 2026-04-27-engine-flavor-explicit-everywhere).
//!
//! This crate is MIT-licensed and ships separately from `truecalc-core`
//! (ADR 2026-04-27-workbook-crate-separate-mit).
//!
//! [workbook]: https://github.com/truecalc/core
