use crate::eval::coercion::to_bool;
use crate::eval::functions::EvalCtx;
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `IFS(cond1, val1, cond2, val2, ...)` — returns the value paired with the
/// first true condition.
///
/// Requires an even number of args (≥ 2). Returns `#N/A` if no condition matches.
/// Returns `#VALUE!` if the arg count is wrong or a condition cannot be coerced.
pub fn ifs_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    let count = args.len();
    if count < 2 || !count.is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }
    let mut i = 0;
    while i < count {
        let cond_val = evaluate_expr(&args[i], ctx);
        match to_bool(cond_val) {
            Ok(true) => return evaluate_expr(&args[i + 1], ctx),
            Ok(false) => {}
            Err(e) => return e,
        }
        i += 2;
    }
    Value::Error(ErrorKind::NA)
}

#[cfg(test)]
mod tests;
