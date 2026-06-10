// truecalc-core: spreadsheet formula parser and evaluator

pub mod display;
pub mod engine;
pub mod eval;
pub mod parser;
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

use std::collections::HashMap;

/// Evaluate a formula string with named variables, targeting Google Sheets conformance.
///
/// Returns `Value::Error(ErrorKind::Value)` on parse failure.
#[deprecated(since = "0.7.0", note = "use Engine::sheets()/Engine::excel() and engine.evaluate() — engine flavor is required; see ADR 2026-04-27; removal target: 0.7.0 coordinated release")]
pub fn evaluate(formula: &str, variables: &HashMap<String, Value>) -> Value {
    Engine::sheets().evaluate(formula, variables)
}
