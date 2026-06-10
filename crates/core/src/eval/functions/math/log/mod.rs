use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `LOG(number, [base])` — logarithm, default base 10.
pub fn log_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = if args.len() == 2 {
        let base = match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        };
        if base <= 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        if base == 1.0 {
            // log base 1 = ln(x)/ln(1) = ln(x)/0 → #DIV/0!
            return Value::Error(ErrorKind::DivByZero);
        }
        // Use dedicated methods to avoid FP round-trip errors (LOG(1000,10) must be 3.0).
        if (base - 10.0).abs() < f64::EPSILON {
            libm::log10(n)
        } else if (base - std::f64::consts::E).abs() < f64::EPSILON {
            libm::log(n)
        } else {
            libm::log(n) / libm::log(base)
        }
    } else {
        libm::log10(n)
    };
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `LOG10(number)` — base-10 logarithm.
pub fn log10_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = libm::log10(n);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `LN(number)` — natural logarithm.
pub fn ln_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = libm::log(n);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
