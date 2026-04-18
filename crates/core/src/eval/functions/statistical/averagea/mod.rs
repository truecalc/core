use crate::types::{ErrorKind, Value};

/// `AVERAGEA(value1, ...)` — average of all values, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - No args → `#N/A`.
pub fn averagea_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut total = 0.0_f64;
    let mut count = 0usize;
    for arg in args {
        let n = match arg {
            Value::Number(n) => *n,
            Value::Bool(b)   => if *b { 1.0 } else { 0.0 },
            Value::Text(_)   => return Value::Error(ErrorKind::Value),
            Value::Empty     => continue,
            _                => continue,
        };
        total += n;
        count += 1;
    }
    if count == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(total / count as f64)
}

#[cfg(test)]
mod tests;
