use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SUM(n1, n2, ...)` — returns the arithmetic sum of 1–255 arguments.
///
/// Each direct argument is coerced to a number via [`to_number`].
/// Array arguments are flattened; within arrays, Bool and Text elements are
/// silently skipped (GS/Excel: only numbers and dates inside arrays contribute).
/// The first error value encountered is returned immediately.
/// Overflow to infinity returns `Value::Error(ErrorKind::Num)`.
pub fn sum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut sum = 0.0_f64;
    for arg in args {
        match sum_top_level(arg) {
            Err(e) => return e,
            Ok(n) => sum += n,
        }
    }
    if !sum.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(sum)
}

/// Handle a top-level (direct) argument to SUM.
/// Arrays are expanded with array-context rules (booleans/text skipped).
/// Non-array values are coerced via to_number (booleans count as 0/1).
fn sum_top_level(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Array(_) => sum_array_value(v),
        // Direct non-array arg: full to_number coercion (Bool → 0/1, Text → parsed)
        other => to_number(other.clone()),
    }
}

/// Recursively sum a value inside an array context.
/// Booleans and Text are silently skipped (GS/Excel array-context rule).
fn sum_array_value(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Array(elems) => {
            let mut total = 0.0_f64;
            for elem in elems {
                total += sum_array_value(elem)?;
            }
            Ok(total)
        }
        // In array context: booleans and text are silently skipped
        Value::Bool(_) | Value::Text(_) | Value::Empty => Ok(0.0),
        // Errors propagate
        Value::Error(_) => Err(v.clone()),
        // Numbers and Dates contribute their value
        Value::Number(n) | Value::Date(n) => Ok(*n),
    }
}

#[cfg(test)]
mod tests;
