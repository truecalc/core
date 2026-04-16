use crate::eval::evaluate_expr;
use crate::eval::functions::EvalCtx;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

// ── Eager versions (kept for unit tests) ─────────────────────────────────────

/// `COUNT(value1, ...)` — count of numeric-coercible values (Numbers, Bools, numeric Text).
/// Used only in unit tests; the evaluator uses the lazy version.
pub fn count_fn(args: &[Value]) -> Value {
    let n = args.iter().filter(|v| matches!(v, Value::Number(_))).count();
    Value::Number(n as f64)
}

/// `COUNTA(value1, ...)` — count of non-empty values.
pub fn counta_fn(args: &[Value]) -> Value {
    let n = args.iter().filter(|v| !matches!(v, Value::Empty)).count();
    Value::Number(n as f64)
}

// ── Lazy versions (registered) ────────────────────────────────────────────────

fn count_in_value(v: &Value) -> usize {
    match v {
        Value::Number(_) | Value::Bool(_) => 1,
        Value::Text(s) if s.parse::<f64>().is_ok() => 1,
        Value::Array(items) => items.iter().map(count_in_value).sum(),
        _ => 0,
    }
}

fn counta_in_value(v: &Value) -> usize {
    match v {
        Value::Empty => 0,
        Value::Array(items) => items.iter().map(counta_in_value).sum(),
        _ => 1,
    }
}

/// Lazy COUNT: counts Numbers, Booleans, and numeric Text; ignores errors/empty.
/// Returns #N/A when called with no arguments.
pub fn count_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut n = 0usize;
    for arg in args {
        n += count_in_value(&evaluate_expr(arg, ctx));
    }
    Value::Number(n as f64)
}

/// Lazy COUNTA: counts everything that is not Empty (including errors).
/// Returns #N/A when called with no arguments.
pub fn counta_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut n = 0usize;
    for arg in args {
        n += counta_in_value(&evaluate_expr(arg, ctx));
    }
    Value::Number(n as f64)
}

#[cfg(test)]
mod tests;
