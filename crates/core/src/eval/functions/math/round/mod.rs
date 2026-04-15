use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Round `n` to `digits` decimal places, Excel-style (0.5 rounds away from zero).
pub fn round_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let digits = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let d = digits.trunc() as i32;
    let result = round_half_away(n, d);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// Round `n` up (away from zero) to `digits` decimal places.
pub fn roundup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let digits = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let d = digits.trunc() as i32;
    let result = round_away_from_zero(n, d);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// Round `n` down (toward zero) to `digits` decimal places.
pub fn rounddown_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let digits = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let d = digits.trunc() as i32;
    let result = round_toward_zero(n, d);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

fn scale(d: i32) -> f64 {
    10f64.powi(d)
}

fn round_half_away(n: f64, d: i32) -> f64 {
    let s = scale(d);
    (n * s).signum() * ((n * s).abs() + 0.5).floor() / s
}

fn round_away_from_zero(n: f64, d: i32) -> f64 {
    let s = scale(d);
    (n * s).signum() * ((n * s).abs()).ceil() / s
}

fn round_toward_zero(n: f64, d: i32) -> f64 {
    let s = scale(d);
    (n * s).signum() * ((n * s).abs()).floor() / s
}

#[cfg(test)]
mod tests;
