use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// `GCD(value1, value2, ...)` — greatest common divisor of all arguments.
/// All args must be non-negative integers (floats are truncated).
pub fn gcd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, usize::MAX) {
        return err;
    }
    let mut result: u64 = 0;
    for arg in args {
        let n = match to_number(arg.clone()) {
            Err(e) => return e,
            Ok(v) => v,
        };
        if n < 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        let n_int = n.trunc() as u64;
        result = gcd(result, n_int);
    }
    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
