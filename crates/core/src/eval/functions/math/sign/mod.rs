use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

pub fn sign_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = if n > 0.0 {
        1.0
    } else if n < 0.0 {
        -1.0
    } else {
        0.0
    };
    Value::Number(result)
}

#[cfg(test)]
mod tests;
