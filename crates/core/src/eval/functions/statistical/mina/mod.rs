use crate::types::{ErrorKind, Value};

/// `MINA(value1, ...)` — smallest value, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - No args → `#N/A`.
pub fn mina_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut result: Option<f64> = None;
    for arg in args {
        match arg {
            Value::Number(n) => {
                result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                result = Some(result.map_or(n, |cur: f64| cur.min(n)));
            }
            Value::Text(_) => return Value::Error(ErrorKind::Value),
            Value::Empty => {}
            Value::Array(inner) => {
                // In array context: Numbers included, Bool→1/0, Text→0, Empty→skip
                for v in inner {
                    let n = match v {
                        Value::Number(n) => *n,
                        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
                        Value::Text(_) => 0.0,
                        Value::Empty => continue,
                        Value::Error(e) => return Value::Error(e.clone()),
                        Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
                        _ => continue,
                    };
                    result = Some(result.map_or(n, |cur: f64| cur.min(n)));
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            _ => {}
        }
    }
    match result {
        Some(n) => Value::Number(n),
        None    => Value::Error(ErrorKind::NA),
    }
}

#[cfg(test)]
mod tests;
