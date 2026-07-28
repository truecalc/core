use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SUMSQ(value1, value2, ...)` — returns sum of squares of all arguments.
/// Arrays are flattened. Non-numeric text is ignored. Errors are propagated.
pub fn sumsq_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut sum = 0.0_f64;
    for arg in args {
        match sumsq_value(arg, false) {
            Err(e) => return e,
            Ok(n) => sum += n,
        }
    }
    if !sum.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(sum)
}

/// Recursively square-sum a value, flattening arrays.
/// Non-numeric text and empty values contribute 0.
fn sumsq_value(v: &Value, in_array: bool) -> Result<f64, Value> {
    match v {
        Value::Array(elems) => {
            let mut total = 0.0_f64;
            for elem in elems {
                total += sumsq_value(elem, true)?;
            }
            Ok(total)
        }
        Value::Number(n) => Ok(n * n),
        Value::Bool(_) if in_array => Ok(0.0), // skipped in array context
        Value::Bool(b) => {
            let n = if *b { 1.0_f64 } else { 0.0_f64 };
            Ok(n * n)
        }
        Value::Empty => Ok(0.0),
        Value::Text(_) if in_array => Ok(0.0), // skipped in array context
        Value::Text(s) => {
            // Direct arg: non-numeric text -> #VALUE!
            if let Ok(n) = s.trim().parse::<f64>() { Ok(n * n) }
            else { Err(Value::Error(crate::types::ErrorKind::Value)) }
        }
        // Aggregates skip a sparkline in any position (google.tsv: SUM and MAX
        // both skip a direct sparkline argument).
        Value::Sparkline(_) => Ok(0.0),
        Value::Zoned(_) if in_array => Ok(0.0), // skipped in array context
        Value::Zoned(_) => Err(Value::Error(crate::types::ErrorKind::Value)),
        Value::Error(_) | Value::ErrorMsg(_, _) => Err(v.clone()),
        Value::Date(n) => Ok(n * n),
    }
}

#[cfg(test)]
mod tests;
