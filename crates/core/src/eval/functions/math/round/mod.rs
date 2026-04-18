use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Round `n` to `digits` decimal places, Excel-style (0.5 rounds away from zero).
/// GS: `num_digits` argument is optional, defaulting to 0.
pub fn round_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let digits = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        0.0
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
    // Add a tiny epsilon (1e-12) before floor to compensate for float representation
    // errors. For example, 1.005 * 100 = 100.4999... in float64 (1e-14 below 100.5),
    // so adding 1e-12 pushes it above the threshold. Values genuinely below 0.5
    // by more than 1e-12 (>1e-12 gap) are not affected.
    (n * s).signum() * ((n * s).abs() + 0.5 + 1e-12).floor() / s
}

fn round_away_from_zero(n: f64, d: i32) -> f64 {
    let s = scale(d);
    (n * s).signum() * ((n * s).abs()).ceil() / s
}

fn round_toward_zero(n: f64, d: i32) -> f64 {
    let s = scale(d);
    (n * s).signum() * ((n * s).abs()).floor() / s
}

/// Round up to nearest even integer, away from zero.
pub fn even_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n == 0.0 {
        return Value::Number(0.0);
    }
    let sign = if n > 0.0 { 1.0 } else { -1.0 };
    let ceil = n.abs().ceil();
    let result = if ceil as i64 % 2 == 0 { ceil } else { ceil + 1.0 };
    Value::Number(sign * result)
}

/// Round up to nearest odd integer, away from zero.
pub fn odd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n == 0.0 {
        return Value::Number(1.0);
    }
    let sign = if n > 0.0 { 1.0 } else { -1.0 };
    let ceil = n.abs().ceil();
    let result = if ceil as i64 % 2 != 0 { ceil } else { ceil + 1.0 };
    Value::Number(sign * result)
}

/// Round `n` to nearest multiple of `multiple`.
pub fn mround_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let multiple = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if multiple == 0.0 {
        return Value::Number(0.0);
    }
    if n * multiple < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number((n / multiple).round() * multiple)
}

/// Truncate `n` toward zero to `digits` decimal places (default 0).
pub fn trunc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let digits = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v.trunc() as i32,
        }
    } else {
        0i32
    };
    let result = round_toward_zero(n, digits);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
