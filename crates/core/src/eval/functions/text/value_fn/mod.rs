use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `VALUE(text)` — parses a text string to a number. Returns `#VALUE!` if unparseable.
/// Empty string returns 0, matching Google Sheets behaviour.
pub fn value_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Value::Number(0.0);
    }
    match trimmed.parse::<f64>() {
        Ok(n) => Value::Number(n),
        Err(_) => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
