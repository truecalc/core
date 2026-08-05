use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `CEILING(number[, significance])` — round up to nearest multiple of significance.
/// When called with 1 arg, significance defaults to 1.
pub fn ceiling_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sig = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        1.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    if n > 0.0 && sig < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = (n / sig).ceil() * sig;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `FLOOR(number[, significance])` — round down to nearest multiple of significance.
/// When called with 1 arg, significance defaults to 1.
pub fn floor_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sig = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        1.0
    };
    if sig == 0.0 {
        // GS: FLOOR(0, 0) = 0; FLOOR(x≠0, 0) = #DIV/0!
        if n == 0.0 { return Value::Number(0.0); }
        return Value::Error(ErrorKind::DivByZero);
    }
    if n > 0.0 && sig < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = (n / sig).floor() * sig;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
