use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `FACTDOUBLE(n)` — double factorial: n!! = n * (n-2) * (n-4) * ... down to 1 or 2.
/// FACTDOUBLE(0) = 1, FACTDOUBLE(-1) = #NUM!, n > 300 = #NUM! (overflow).
pub fn factdouble_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let n_int = n.trunc() as i64;
    if n_int > 300 {
        return Value::Error(ErrorKind::Num);
    }
    if n_int == 0 {
        return Value::Number(1.0);
    }
    let mut result = 1.0_f64;
    let mut i = n_int;
    while i > 0 {
        result *= i as f64;
        i -= 2;
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
