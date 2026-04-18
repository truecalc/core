use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn abs_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.abs();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
