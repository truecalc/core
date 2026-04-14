// ganit-core: spreadsheet formula parser and evaluator

pub mod display;
pub mod eval;
pub mod parser;
pub mod types;

pub use display::display_number;
pub use parser::{parse, validate, Expr};
pub use types::{ErrorKind, ParseError, Value};

use std::collections::HashMap;
use eval::{evaluate_expr, Context, EvalCtx, Registry};

/// Evaluate a formula string with named variables.
///
/// Returns `Value::Error(ErrorKind::Value)` on parse failure.
///
/// Note: constructs a fresh `Registry` on every call. For bulk evaluation
/// across many formulas, consider reusing a `Registry` via the lower-level
/// `eval::evaluate_expr` API.
pub fn evaluate(formula: &str, variables: HashMap<String, Value>) -> Value {
    match parse(formula) {
        Err(_) => Value::Error(ErrorKind::Value),
        Ok(expr) => {
            let ctx = Context::new(variables);
            let registry = Registry::new();
            let mut eval_ctx = EvalCtx::new(ctx, &registry);
            evaluate_expr(&expr, &mut eval_ctx)
        }
    }
}
