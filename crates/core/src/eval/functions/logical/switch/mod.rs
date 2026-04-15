use crate::eval::functions::EvalCtx;
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `SWITCH(expr, case1, val1, ..., [default])` — matches expr against cases.
///
/// - First arg is the expression to match.
/// - Remaining args are case/value pairs with an optional default as the last arg
///   when the remaining count is odd.
/// - Returns `#N/A` if no match and no default.
/// - Returns `#VALUE!` if fewer than 3 args are provided.
pub fn switch_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    // Need at least: expr + 1 case + 1 value = 3 args
    if args.len() < 3 {
        return Value::Error(ErrorKind::Value);
    }
    let expr_val = evaluate_expr(&args[0], ctx);
    let rest = &args[1..];
    // odd rest count means last element is the default
    let has_default = !rest.len().is_multiple_of(2);
    let pairs_end = if has_default { rest.len() - 1 } else { rest.len() };

    let mut i = 0;
    while i < pairs_end {
        let case_val = evaluate_expr(&rest[i], ctx);
        if case_val == expr_val {
            return evaluate_expr(&rest[i + 1], ctx);
        }
        i += 2;
    }

    if has_default {
        evaluate_expr(&rest[rest.len() - 1], ctx)
    } else {
        Value::Error(ErrorKind::NA)
    }
}

#[cfg(test)]
mod tests;
