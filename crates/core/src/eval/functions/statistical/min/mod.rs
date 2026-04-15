use crate::types::{ErrorKind, Value};

/// `MIN(value1, ...)` — smallest numeric value in the arguments.
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty and errors are ignored (errors already propagated by eager dispatcher).
/// - No args → `#N/A`.
pub fn min_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut result: Option<f64> = None;
    for arg in args {
        let n = match arg {
            Value::Number(n) => *n,
            Value::Bool(b)   => if *b { 1.0 } else { 0.0 },
            Value::Text(_)   => return Value::Error(ErrorKind::Value),
            Value::Empty     => continue,
            _                => continue,
        };
        result = Some(match result {
            None      => n,
            Some(cur) => cur.min(n),
        });
    }
    Value::Number(result.unwrap_or(0.0))
}

#[cfg(test)]
mod tests;
