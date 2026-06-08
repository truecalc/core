//! truecalc-workbook: the workbook layer for the truecalc spreadsheet engine.
//!
//! A [`Workbook`] is a *value object*: an engine-locked collection of ordered
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
//! [`Workbook::to_json`] / [`Workbook::from_json`] are the canonical
//! (RFC 8785 / JCS) serialization boundary — the byte-identical cross-surface
//! contract of schema spec §8.

mod address;
mod canonical;
mod cell;
mod engine;
mod error;
pub mod limits;
mod named_range;
mod named_ref;
mod strict_json;
mod validate;
mod value;
mod workbook;
mod worksheet;

pub use cell::Cell;
pub use engine::EngineFlavor;
pub use error::WorkbookError;
pub use named_range::NamedRange;
pub use value::Value;
pub use workbook::{Workbook, SCHEMA_VERSION};
pub use worksheet::Worksheet;
