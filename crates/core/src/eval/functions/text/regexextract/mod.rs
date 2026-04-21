use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use regex_lite::Regex;

/// `REGEXEXTRACT(text, pattern)` — returns the first match of pattern in text.
/// If the pattern contains a capture group, returns the content of the first capture group.
/// Returns `#N/A` if no match.
pub fn regexextract_fn(args: &[Value]) -> Value {
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
    match re.captures(&text) {
        Some(caps) => {
            // If there is a capture group, return the first group; otherwise the full match.
            let matched = caps.get(1).unwrap_or_else(|| caps.get(0).unwrap());
            Value::Text(matched.as_str().to_string())
        }
        None => Value::Error(ErrorKind::NA),
    }
}

#[cfg(test)]
mod tests;
