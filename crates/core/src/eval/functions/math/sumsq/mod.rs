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
        match sumsq_value(arg) {
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
fn sumsq_value(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Array(elems) => {
            let mut total = 0.0_f64;
            for elem in elems {
                total += sumsq_value(elem)?;
            }
            Ok(total)
        }
        Value::Number(n) => Ok(n * n),
        Value::Bool(b) => {
            let n = if *b { 1.0_f64 } else { 0.0_f64 };
            Ok(n * n)
        }
        Value::Empty => Ok(0.0),
        Value::Text(s) => {
            if s.is_empty() {
                Ok(0.0)
            } else {
                match s.parse::<f64>() {
                    Ok(n) => Ok(n * n),
                    Err(_) => Ok(0.0), // non-numeric text ignored
                }
            }
        }
        Value::Error(_) => Err(v.clone()),
        Value::Date(n) => Ok(n * n),
    }
}

#[cfg(test)]
mod tests;
