use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SUM(n1, n2, ...)` — returns the arithmetic sum of 1–255 arguments.
///
/// Each argument is coerced to a number via [`to_number`]. The first error
/// value encountered is returned immediately. Overflow to infinity returns
/// `Value::Error(ErrorKind::Num)`.
pub fn sum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut sum = 0.0_f64;
    for arg in args {
        // .clone() required because to_number takes ownership; coercion API is fixed.
        match to_number(arg.clone()) {
            Err(e) => return e,
            Ok(n) => sum += n,
        }
    }
    if !sum.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(sum)
}

#[cfg(test)]
mod tests;
