use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn to_pure_number_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Number(n) => Value::Number(*n),
        Value::Date(n)   => Value::Number(*n),
        Value::Bool(b)   => Value::Number(if *b { 1.0 } else { 0.0 }),
        Value::Text(s)   => Value::Text(s.clone()),
        Value::Error(_)  => args[0].clone(),
        _                => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
