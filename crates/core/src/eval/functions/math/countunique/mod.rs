use crate::eval::functions::check_arity;
use crate::types::Value;
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
    }
}

/// `COUNTUNIQUE(value1, value2, ...)` — count unique distinct values across all
/// arguments. Text comparison is case-sensitive (GS behavior). Empty / blank
/// values are ignored. Errors each count as a unique value (not propagated).
pub fn countunique_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
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

#[cfg(test)]
mod tests;
