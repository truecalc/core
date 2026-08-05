use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `MOD(number, divisor)` — result sign follows divisor (Excel semantics).
/// Formula: n - divisor * floor(n / divisor)
pub fn mod_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let d = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if d == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    // GS/Excel: MOD on very large numbers loses precision → #NUM!
    if n.abs() >= 1e15 {
        return Value::Error(ErrorKind::Num);
    }
    let result = n - d * (n / d).floor();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
