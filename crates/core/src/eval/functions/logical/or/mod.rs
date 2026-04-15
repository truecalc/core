use crate::eval::coercion::to_bool;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::Value;

/// `OR(val1, ...)` — TRUE if ANY argument is truthy.
///
/// Short-circuits on the first true value. Returns `#VALUE!` with no args
/// or if any arg cannot be coerced to bool.
pub fn or_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, usize::MAX) {
        return err;
    }
    for arg in args {
        let val = evaluate_expr(arg, ctx);
        match to_bool(val) {
            Ok(true) => return Value::Bool(true),
            Ok(false) => {}
            Err(e) => return e,
        }
    }
    Value::Bool(false)
}

#[cfg(test)]
mod tests;
