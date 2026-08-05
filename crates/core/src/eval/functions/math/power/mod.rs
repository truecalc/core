use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Real-valued `base^exp`, the shared semantics behind both `POWER()` and the
/// `^` operator (core#846: the two must agree). GS: POWER(-8, 1/3) = -2.0 —
/// a negative base raised to a fractional exponent p/q (q odd) has a real
/// odd root, so that case is computed via the magnitude and re-signed rather
/// than deferring to `libm::pow`, which yields NaN for a negative base with
/// a non-integer exponent. An even root (or any other non-finite result)
/// stays `#NUM!`.
pub fn real_pow(base: f64, exp: f64) -> Value {
    let result = if base < 0.0 && exp.fract() != 0.0 {
        // Check if 1/exp is close to an odd integer -> odd root
        let inv = 1.0 / exp;
        let inv_round = inv.round();
        if (inv - inv_round).abs() < 1e-9 && (inv_round.abs() as i64) % 2 == 1 {
            let mag = libm::pow(base.abs(), exp);
            -mag
        } else {
            // non-real result -> #NUM!
            return Value::Error(ErrorKind::Num);
        }
    } else {
        libm::pow(base, exp)
    };
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn power_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let base = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let exp = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    real_pow(base, exp)
}

#[cfg(test)]
mod tests;
