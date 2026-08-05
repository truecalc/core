use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn to_dollars_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Number(n) => Value::Number(*n),
        Value::Bool(b)   => Value::Bool(*b),
        Value::Text(s)   => Value::Text(s.clone()),
        Value::Error(_) | Value::ErrorMsg(_, _)  => args[0].clone(),
        // `TO_*` family behaviour — see `super::to_text::to_text_fn`.
        Value::Sparkline(_) => Value::Text(String::new()),
        _                => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
