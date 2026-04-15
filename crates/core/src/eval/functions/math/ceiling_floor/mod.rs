use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `CEILING(number, significance)` — round up to nearest multiple of significance.
pub fn ceiling_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sig = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if sig == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let result = (n / sig).ceil() * sig;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `FLOOR(number, significance)` — round down to nearest multiple of significance.
pub fn floor_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sig = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if sig == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let result = (n / sig).floor() * sig;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
