use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use regex_lite::Regex;

/// `REGEXREPLACE(text, pattern, replacement)` — replaces ALL non-overlapping matches
/// of pattern in text with replacement.
pub fn regexreplace_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let pattern = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let replacement = match to_string_val(args[2].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Value::Error(ErrorKind::Value),
    };
    Value::Text(re.replace_all(&text, replacement.as_str()).into_owned())
}

#[cfg(test)]
mod tests;
