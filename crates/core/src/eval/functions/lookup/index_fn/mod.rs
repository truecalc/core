use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `INDEX(array, row, [col])` — returns the element at the given 1-based position
/// in an array. For a flat (1-D) array, only `row` is needed.
pub fn index_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let idx = match to_number(args[1].clone()) {
        Ok(n) => n as usize,
        Err(e) => return e,
    };
    if idx == 0 {
        return Value::Error(ErrorKind::Value);
    }
    match &args[0] {
        Value::Array(elems) => {
            elems
                .get(idx - 1)
                .cloned()
                .unwrap_or(Value::Error(ErrorKind::Ref))
        }
        other => {
            // Scalar: only valid when idx == 1
            if idx == 1 {
                other.clone()
            } else {
                Value::Error(ErrorKind::Ref)
            }
        }
    }
}

#[cfg(test)]
mod tests;
