//! Workbook layer for the [truecalc](https://docs.rs/truecalc-core) spreadsheet
//! engine: engine-locked workbook, worksheet, and cell value types with a
//! canonical JSON serialization contract.
//!
//! # Quick start
//!
//! ```rust
//! use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};
//!
//! // 1. Create a workbook locked to Google Sheets semantics.
//! let mut wb = Workbook::new(EngineFlavor::Sheets);
//!
//! // 2. Add a sheet and write some cells.
//! wb.add_sheet(Worksheet::new("Budget")).unwrap();
//! let a1 = Address::from_a1("A1").unwrap();
//! let a2 = Address::from_a1("A2").unwrap();
//! let a3 = Address::from_a1("A3").unwrap();
//! wb.set("Budget", a1, CellInput::Literal(Value::Number(1000.0))).unwrap();
//! wb.set("Budget", a2, CellInput::Literal(Value::Number(500.0))).unwrap();
//! wb.set("Budget", a3, CellInput::Formula("=SUM(A1:A2)".to_string())).unwrap();
//!
//! // 3. Recalculate to evaluate all formulas.
//! // timestamp_ms = Unix epoch in ms, tz = IANA timezone id, rng_seed = 0
//! let ctx = RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).unwrap();
//! let _changes = wb.recalc(&ctx);
//! assert_eq!(wb.get("Budget", a3).unwrap().value(), &Value::Number(1500.0));
//! ```
//!
//! # Core concepts
//!
//! - **[`Workbook`]** is a *value object*: `Clone + PartialEq + Hash`, no
//!   hidden state or callbacks. Mutate with [`Workbook::set`] / [`Workbook::clear`]
//!   and then drive recalc with [`Workbook::recalc`] or
//!   [`Workbook::recalc_incremental`].
//!
//! - **Engine flavor** ([`EngineFlavor`]) is required at creation
//!   ([`Workbook::new`]) and immutable for the workbook's lifetime. It controls
//!   formula semantics and the date serial system.
//!
//! - **[`RecalcContext`]** pins volatile functions (`NOW`, `TODAY`) to a fixed
//!   UTC instant + IANA timezone. Same workbook + same context ⇒ byte-identical
//!   recomputed grid.
//!
//! - **[`CellInput`]** is the discriminated input type for [`Workbook::set`]:
//!   `CellInput::Literal(value)` or `CellInput::Formula("=...".to_string())`.
//!   Formula syntax is validated against the locked engine on `set`.
//!
//! # Serialization
//!
//! [`Workbook::to_json`] / [`Workbook::from_json`] implement the canonical
//! (RFC 8785 / JCS) serialization boundary — the byte-identical cross-surface
//! contract used across Rust, WASM, MCP, and REST surfaces.
//!
//! # Related crates
//!
//! - [`truecalc-core`](https://docs.rs/truecalc-core): the formula parser and
//!   evaluator; the [`Engine`](truecalc_core::Engine) type and all formula
//!   evaluation lives there.

mod address;
mod canonical;
mod casefold;
mod cell;
mod depgraph;
mod engine;
mod error;
pub mod limits;
mod mutate;
mod named_range;
mod named_ref;
mod recalc;
mod spill;
mod strict_json;
mod validate;
mod value;
mod workbook;
mod worksheet;

pub use address::Address;
pub use cell::Cell;
pub use depgraph::{CellRef, DependencyGraph, Precedent, RangeRef};
pub use engine::EngineFlavor;
pub use error::WorkbookError;
pub use mutate::{CellInput, Resolved};
pub use named_range::NamedRange;
pub use recalc::{Change, RecalcContext, CIRCULAR_ERROR};
pub use spill::{SpillRect, BLOCKED_SPILL_ERROR};
pub use value::Value;
pub use workbook::{Workbook, SCHEMA_VERSION};
pub use worksheet::Worksheet;

/// Re-exported so callers of [`Workbook::trace_cell`] can implement or wire a
/// hook without a direct `truecalc-core` dependency (issue #743) — same
/// rationale as [`EngineFlavor`]'s re-export above: one trait, no drift
/// between the two crates.
pub use truecalc_core::eval::EvalHook;
