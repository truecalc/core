use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `NPV(rate, value1, [value2, ...])` — net present value of cash flows.
pub fn npv_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 256) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let mut npv = 0.0_f64;
    for (i, arg) in args[1..].iter().enumerate() {
        let cf = match to_number(arg.clone()) { Ok(n) => n, Err(e) => return e };
        npv += cf / (1.0 + rate).powi(i as i32 + 1);
    }
    if !npv.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(npv)
}

#[cfg(test)]
mod tests;
