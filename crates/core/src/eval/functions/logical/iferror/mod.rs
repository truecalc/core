use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `IFERROR(value, [error_val])` — returns value unless it is any error, then error_val.
/// When called with 1 arg, error_val defaults to `""` (empty string).
pub fn iferror_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 2) {
        return err;
    }
    let val = evaluate_expr(&args[0], ctx);
    if matches!(val, Value::Error(_)) {
        if args.len() == 2 {
            evaluate_expr(&args[1], ctx)
        } else {
            Value::Text(String::new())
        }
    } else {
        val
    }
}

/// `IFNA(value, [na_val])` — returns value unless it is `#N/A`, then na_val.
/// When called with 1 arg, na_val defaults to `""` (empty string).
pub fn ifna_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 2) {
        return err;
    }
    let val = evaluate_expr(&args[0], ctx);
    if matches!(val, Value::Error(ErrorKind::NA)) {
        if args.len() == 2 {
            evaluate_expr(&args[1], ctx)
        } else {
            Value::Text(String::new())
        }
    } else {
        val
    }
}

#[cfg(test)]
mod tests;
