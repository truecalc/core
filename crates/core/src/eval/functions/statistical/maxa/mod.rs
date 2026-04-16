use crate::types::{ErrorKind, Value};

/// `MAXA(value1, ...)` — largest value, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - No args → `#N/A`.
pub fn maxa_fn(args: &[Value]) -> Value {
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
            Some(cur) => cur.max(n),
        });
    }
    match result {
        Some(n) => Value::Number(n),
        None    => Value::Error(ErrorKind::NA),
    }
}

#[cfg(test)]
mod tests;
