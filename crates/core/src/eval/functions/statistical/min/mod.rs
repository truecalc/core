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
    fn min_val(v: &Value, result: &mut Option<f64>) -> Option<Value> {
        match v {
            Value::Array(elems) => {
                for elem in elems {
                    if let Some(e) = min_val(elem, result) {
                        return Some(e);
                    }
                }
                None
            }
            Value::Number(n) => {
                *result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
                None
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                *result = Some(result.map_or(n, |cur: f64| cur.min(n)));
                None
            }
            Value::Text(_) => Some(Value::Error(ErrorKind::Value)),
            Value::Empty => None,
            _ => None,
        }
    }
    for arg in args {
        if let Some(e) = min_val(arg, &mut result) {
            return e;
        }
    }
    Value::Number(result.unwrap_or(0.0))
}

#[cfg(test)]
mod tests;
