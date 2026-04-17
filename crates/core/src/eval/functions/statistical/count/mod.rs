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

/// Lazy COUNT: counts Numbers, Booleans, and numeric Text; ignores errors/empty.
/// Returns #N/A when called with no arguments.
pub fn count_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut n = 0usize;
    for arg in args {
        count_numeric(&evaluate_expr(arg, ctx), &mut n);
    }
    Value::Number(n as f64)
}

fn count_numeric(v: &Value, n: &mut usize) {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                count_numeric(elem, n);
            }
        }
        Value::Number(_) => *n += 1,
        Value::Bool(_) => *n += 1,
        Value::Text(s) if s.parse::<f64>().is_ok() => *n += 1,
        _ => {}
    }
}

/// Lazy COUNTA: counts everything that is not Empty (including errors).
/// Arrays are flattened: each non-empty element is counted individually.
/// Returns #N/A when called with no arguments.
pub fn counta_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut n = 0usize;
    for arg in args {
        count_non_empty(&evaluate_expr(arg, ctx), &mut n);
    }
    Value::Number(n as f64)
}

fn count_non_empty(v: &Value, n: &mut usize) {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                count_non_empty(elem, n);
            }
        }
        Value::Empty => {}
        _ => *n += 1,
    }
}

#[cfg(test)]
mod tests;
