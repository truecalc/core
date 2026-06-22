use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn product_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut product = 1.0_f64;
    for arg in args {
        match product_top_level(arg) {
            Err(e) => return e,
            Ok(n) => product *= n,
        }
    }
    if !product.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(product)
}

/// Top-level: arrays use array-context (booleans/text skipped).
/// Direct scalars use full coercion.
fn product_top_level(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Array(_) => product_array_value(v),
        other => to_number(other.clone()),
    }
}

/// Array-context: booleans, text, empty silently skipped (contribute 1).
fn product_array_value(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Array(elems) => {
            let mut p = 1.0_f64;
            for elem in elems {
                p *= product_array_value(elem)?;
            }
            Ok(p)
        }
        Value::Bool(_) | Value::Text(_) | Value::Empty => Ok(1.0),
        Value::Zoned(_) => Ok(1.0),
        Value::Error(_) => Err(v.clone()),
        Value::Number(n) | Value::Date(n) => Ok(*n),
    }
}

#[cfg(test)]
mod tests;
