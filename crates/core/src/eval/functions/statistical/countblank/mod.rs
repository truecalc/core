use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::Value;

pub(crate) fn count_blanks_in(values: &[Value]) -> usize {
    let mut count = 0;
    for v in values {
        match v {
            Value::Empty => count += 1,
            Value::Text(s) if s.is_empty() => count += 1,
            Value::Array(arr) => count += count_blanks_in(arr),
            _ => {}
        }
    }
    count
}

/// `COUNTBLANK(range)` — counts empty/blank values.
///
/// Registered as a lazy function so that error values in arguments (e.g.
/// COUNTBLANK(1/0)) are received as Value::Error and counted as non-blank
/// (returning 0), rather than being short-circuited by the eager evaluator.
pub fn countblank_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 255) {
        return err;
    }
    let mut evaluated: Vec<Value> = Vec::with_capacity(args.len());
    for arg in args {
        evaluated.push(evaluate_expr(arg, ctx));
    }
    Value::Number(count_blanks_in(&evaluated) as f64)
}

#[cfg(test)]
mod tests;
