use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `CODE(text)` — returns the numeric code for the first character of `text`.
pub fn code_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let s = match to_string_val(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    match s.chars().next() {
        Some(c) => Value::Number(c as u32 as f64),
        None => Value::Error(ErrorKind::Value),
    }
}

/// `UNICODE(text)` — returns the Unicode code point of the first character of `text`.
pub fn unicode_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let s = match to_string_val(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    match s.chars().next() {
        Some(c) => Value::Number(c as u32 as f64),
        None => Value::Error(ErrorKind::Value),
    }
}
