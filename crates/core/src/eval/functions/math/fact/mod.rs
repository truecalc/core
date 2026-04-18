use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `FACT(n)` — returns n! (factorial). n must be a non-negative integer ≤ 170.
pub fn fact_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    // Check raw float first — negative fractional values like -0.1 truncate to 0
    // but GS still returns #NUM! for any negative input.
    if n < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let n_int = n.trunc() as i64;
    if n_int > 170 {
        return Value::Error(ErrorKind::Num);
    }
    let mut result = 1.0f64;
    for i in 2..=(n_int as u64) {
        result *= i as f64;
    }
    Value::Number(result)
}
