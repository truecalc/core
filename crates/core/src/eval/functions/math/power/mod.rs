use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

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
    let result = base.powf(exp);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
