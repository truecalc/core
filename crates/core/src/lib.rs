//! Spreadsheet formula parser and evaluator for Google Sheets and Excel
//! conformance.
//!
//! # Quick start
//!
//! ```rust
//! use std::collections::HashMap;
//! use truecalc_core::{Engine, Value};
//!
//! let engine = Engine::sheets();
//!
//! let mut vars = HashMap::new();
//! vars.insert("A1".to_string(), Value::Number(10.0));
//! vars.insert("A2".to_string(), Value::Number(20.0));
//! vars.insert("A3".to_string(), Value::Number(30.0));
//!
//! let result = engine.evaluate("=SUM(A1,A2,A3)", &vars);
//! assert_eq!(result, Value::Number(60.0));
//! ```
//!
//! # Engine flavors
//!
//! The engine flavor is **required and immutable**: create one engine per
//! evaluation context and reuse it.
//!
//! | Constructor | Target conformance |
//! |---|---|
//! | [`Engine::sheets()`] | Google Sheets |
//! | [`Engine::excel()`] | Excel (parse/validate only; evaluation pending) |
//!
//! # Evaluation modes
//!
//! - **Variable map** — [`Engine::evaluate`]: references are looked up in a
//!   `HashMap<String, Value>`. Suitable for ad-hoc formula evaluation outside a
//!   workbook.
//! - **Resolver** — [`Engine::evaluate_with_resolver`]: every cell, range, and
//!   name reference is resolved through a [`Resolver`] implementation. This is
//!   the workbook layer's entry point; see `truecalc-workbook`.
//! - **Pinned clock** — [`Engine::evaluate_at`] / [`Engine::evaluate_with_resolver_at`]:
//!   volatile functions (`NOW`, `TODAY`) are pinned to a fixed spreadsheet serial,
//!   producing deterministic results.
//!
//! # Migration from 0.6.x
//!
//! The free `evaluate()` function and `Engine::google_sheets()` constructor were
//! deprecated in 0.7.0. Update call sites as follows:
//!
//! ```rust
//! // Before (0.6.x):
//! // use truecalc_core::evaluate;
//! // let result = evaluate("=SUM(A1,A2)", &vars);
//!
//! // After (0.7+):
//! use std::collections::HashMap;
//! use truecalc_core::{Engine, Value};
//! let vars: HashMap<String, Value> = HashMap::new();
//! let result = Engine::sheets().evaluate("=SUM(A1,A2)", &vars);
//! ```
//!
//! See [`MIGRATION.md`](https://github.com/truecalc/core/blob/main/crates/core/MIGRATION.md)
//! for the full migration guide.
//!
//! # Related crates
//!
//! - [`truecalc-workbook`](https://docs.rs/truecalc-workbook): full workbook
//!   layer — engine-locked workbook, worksheet, cell mutation, and recalc.

pub mod display;
pub mod engine;
pub mod eval;
pub mod parser;
pub mod search;
pub mod types;

pub use display::display_number;
pub use engine::{Engine, EngineFlavor};
#[allow(deprecated)]
pub use parser::{parse, validate};
pub use parser::Expr;
pub use parser::{CellAddr, Ref};
pub use types::{ErrorKind, ParseError, Value};

pub use eval::functions::{FunctionMeta, Registry};
pub use eval::{extract_refs, Resolver};
pub use search::{search_functions, FunctionMatch};

use std::collections::HashMap;

/// Evaluate a formula string with named variables, targeting Google Sheets conformance.
///
/// Returns `Value::Error(ErrorKind::Value)` on parse failure.
#[deprecated(since = "0.7.0", note = "use Engine::sheets()/Engine::excel() and engine.evaluate() — engine flavor is required; see ADR 2026-04-27; removal target: 0.7.0 coordinated release")]
pub fn evaluate(formula: &str, variables: &HashMap<String, Value>) -> Value {
    Engine::sheets().evaluate(formula, variables)
}
