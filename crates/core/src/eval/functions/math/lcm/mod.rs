use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        a / gcd(a, b) * b
    }
}

/// `LCM(value1, value2, ...)` — least common multiple of all arguments.
/// All args must be non-negative integers (floats are truncated).
pub fn lcm_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, usize::MAX) {
        return err;
    }
    let mut result: u64 = 1;
    for arg in args {
        let n = match to_number(arg.clone()) {
            Err(e) => return e,
            Ok(v) => v,
        };
        if n < 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        let n_int = n.trunc() as u64;
        result = lcm(result, n_int);
    }
    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
