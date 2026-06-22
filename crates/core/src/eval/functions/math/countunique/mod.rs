use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};
use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash)]
enum UniqueKey {
    Number(u64),
    Text(String),     // case-sensitive per GS
    Bool(bool),
    ErrorVal(String), // GS: errors are counted as unique, not propagated
}

fn to_unique_key(v: &Value) -> Option<UniqueKey> {
    match v {
        Value::Number(n) => Some(UniqueKey::Number(n.to_bits())),
        Value::Bool(b) => Some(UniqueKey::Bool(*b)),
        Value::Text(s) if !s.is_empty() => Some(UniqueKey::Text(s.clone())),
        Value::Text(_) | Value::Empty => None,
        Value::Error(e) => Some(UniqueKey::ErrorVal(format!("{e:?}"))),
        Value::Date(_) | Value::Array(_) => None,
        Value::Zoned(_) => None,
    }
}

/// Eager version kept only for unit tests; the registered version is lazy.
pub fn countunique_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) { return err; }
    let mut seen: HashSet<UniqueKey> = HashSet::new();
    for arg in args {
        match arg {
            Value::Array(arr) => {
                for v in arr {
                    if let Some(key) = to_unique_key(v) { seen.insert(key); }
                }
            }
            other => {
                if let Some(key) = to_unique_key(other) { seen.insert(key); }
            }
        }
    }
    Value::Number(seen.len() as f64)
}

/// `COUNTUNIQUE(value1, value2, ...)` — lazy registered version.
/// Evaluates each argument itself so errors are counted as unique values
/// rather than being propagated (GS behavior: COUNTUNIQUE(1/0) → 1).
pub fn countunique_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut seen: HashSet<UniqueKey> = HashSet::new();
    for arg in args {
        let v = evaluate_expr(arg, ctx);
        match &v {
            Value::Array(arr) => {
                for elem in arr {
                    if let Some(key) = to_unique_key(elem) { seen.insert(key); }
                }
            }
            other => {
                if let Some(key) = to_unique_key(other) { seen.insert(key); }
            }
        }
    }
    Value::Number(seen.len() as f64)
}

#[cfg(test)]
mod tests;
