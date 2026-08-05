use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `CEILING.MATH(number, [significance], [mode])` — round up toward positive infinity.
///
/// - significance defaults to 1 (sign of significance is ignored; abs is used)
/// - mode=0 (default): negative numbers round toward positive infinity (toward zero)
/// - mode≠0: negative numbers round toward negative infinity (away from zero)
pub fn ceiling_math_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 3) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sig = if args.len() >= 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v.abs(),
        }
    } else {
        1.0
    };
    let mode = if args.len() == 3 {
        match to_number(args[2].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        0.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    let result = if n < 0.0 && mode != 0.0 {
        // mode≠0: round away from zero (toward negative infinity)
        (n / sig).floor() * sig
    } else {
        // default: round toward positive infinity
        (n / sig).ceil() * sig
    };
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `CEILING.PRECISE(number, [significance])` — round up toward positive infinity.
///
/// Sign of significance is ignored. Equivalent to CEILING.MATH with mode=0.
pub fn ceiling_precise_fn(args: &[Value]) -> Value {
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
            Ok(v) => v.abs(),
        }
    } else {
        1.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    let result = (n / sig).ceil() * sig;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `FLOOR.MATH(number, [significance], [mode])` — round down toward negative infinity.
///
/// - significance defaults to 1 (sign of significance is ignored; abs is used)
/// - mode=0 (default): negative numbers round toward negative infinity (away from zero)
/// - mode≠0: negative numbers round toward zero
pub fn floor_math_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 3) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sig = if args.len() >= 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v.abs(),
        }
    } else {
        1.0
    };
    let mode = if args.len() == 3 {
        match to_number(args[2].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        0.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    let result = if n < 0.0 && mode != 0.0 {
        // mode≠0: round toward zero (ceiling for negative numbers)
        (n / sig).ceil() * sig
    } else {
        // default: round toward negative infinity
        (n / sig).floor() * sig
    };
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `FLOOR.PRECISE(number, [significance])` — round down toward negative infinity.
///
/// Sign of significance is ignored. Equivalent to FLOOR.MATH with mode=0.
pub fn floor_precise_fn(args: &[Value]) -> Value {
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
            Ok(v) => v.abs(),
        }
    } else {
        1.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    let result = (n / sig).floor() * sig;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `ISO.CEILING(number, [significance])` — identical to CEILING.PRECISE.
///
/// ISO standard version. Sign of significance is ignored.
pub fn iso_ceiling_fn(args: &[Value]) -> Value {
    ceiling_precise_fn(args)
}

#[cfg(test)]
mod tests;
