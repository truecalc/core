use crate::eval::functions::check_arity;
use crate::types::Value;
use std::collections::HashSet;

/// A key for deduplication. Numbers and text are distinct types even when they
/// have the same "display" value (e.g. `1` vs `"1"`). Text comparison is
/// case-insensitive. Empty values (Empty or Text("")) are ignored.
#[derive(PartialEq, Eq, Hash)]
enum UniqueKey {
    Number(u64),   // bit-cast of f64 for hashing
    Text(String),  // lowercased
    Bool(bool),
}

fn to_unique_key(v: &Value) -> Option<UniqueKey> {
    match v {
        Value::Number(n) => Some(UniqueKey::Number(n.to_bits())),
        Value::Bool(b) => Some(UniqueKey::Bool(*b)),
        Value::Text(s) if !s.is_empty() => Some(UniqueKey::Text(s.to_lowercase())),
        Value::Text(_) | Value::Empty => None, // blank — ignore
        Value::Error(_) | Value::Date(_) | Value::Array(_) => None,
    }
}

/// `COUNTUNIQUE(value1, value2, ...)` — count unique distinct values across all
/// arguments. Text comparison is case-insensitive. Empty / blank values are ignored.
/// Numbers and text with the same display are treated as different types.
pub fn countunique_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }

    let mut seen: HashSet<UniqueKey> = HashSet::new();

    for arg in args {
        match arg {
            Value::Array(arr) => {
                for v in arr {
                    if let Some(key) = to_unique_key(v) {
                        seen.insert(key);
                    }
                }
            }
            other => {
                if let Some(key) = to_unique_key(other) {
                    seen.insert(key);
                }
            }
        }
    }

    Value::Number(seen.len() as f64)
}

#[cfg(test)]
mod tests;
