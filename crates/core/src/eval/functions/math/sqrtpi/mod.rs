use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SQRTPI(n)` — returns sqrt(n * PI). n must be >= 0, else #NUM!.
pub fn sqrtpi_fn(args: &[Value]) -> Value {
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
    Value::Number((n * std::f64::consts::PI).sqrt())
}

#[cfg(test)]
mod tests;
