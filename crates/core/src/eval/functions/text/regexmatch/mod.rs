use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use regex_lite::Regex;

/// `REGEXMATCH(text, pattern)` — returns TRUE if text contains a match for pattern.
/// Partial match (pattern can match anywhere in text).
pub fn regexmatch_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
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
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Value::Error(ErrorKind::Value),
    };
    Value::Bool(re.is_match(&text))
}

#[cfg(test)]
mod tests;
