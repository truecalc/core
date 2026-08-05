use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `QUOTIENT(numerator, denominator)` — integer portion of division.
pub fn quotient_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let num = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let den = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if den == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let result = (num / den).trunc();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
