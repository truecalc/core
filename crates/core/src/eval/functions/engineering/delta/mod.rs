use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

pub fn delta_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let a = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let b = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Ok(n) => n,
            Err(e) => return e,
        }
    } else {
        0.0
    };
    if a == b {
        Value::Number(1.0)
    } else {
        Value::Number(0.0)
    }
}

#[cfg(test)]
mod tests;
