use crate::eval::coercion::to_bool;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::Value;

/// `AND(val1, ...)` — TRUE only if ALL arguments are truthy.
///
/// Short-circuits on the first false value. Returns `#VALUE!` with no args
/// or if any arg cannot be coerced to bool. Array arguments are flattened:
/// each element is checked individually (GS/Excel compatible).
pub fn and_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, usize::MAX) {
        return err;
    }
    for arg in args {
        let val = evaluate_expr(arg, ctx);
        match check_all_true(val) {
            Ok(true) => {}
            Ok(false) => return Value::Bool(false),
            Err(e) => return e,
        }
    }
    Value::Bool(true)
}

/// Recursively check that all elements of a value (array or scalar) are truthy.
/// Returns Ok(true) if all truthy, Ok(false) if any falsy, Err if coercion fails.
fn check_all_true(val: Value) -> Result<bool, Value> {
    match val {
        Value::Array(elems) => {
            for elem in elems {
                match check_all_true(elem) {
                    Ok(true) => {}
                    Ok(false) => return Ok(false),
                    Err(e) => return Err(e),
                }
            }
            Ok(true)
        }
        other => to_bool(other),
    }
}

#[cfg(test)]
mod tests;
