use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use regex_lite::Regex;

/// `REGEXEXTRACT(text, pattern)` — returns the first match of pattern in text.
/// Returns `#N/A` if no match. Returns `#REF!` if the pattern contains capture groups.
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
    // Capture groups (beyond the implicit group 0) → #REF!
    if re.captures_len() > 1 {
        return Value::Error(ErrorKind::Ref);
    }
    match re.find(&text) {
        Some(m) => Value::Text(m.as_str().to_string()),
        None => Value::Error(ErrorKind::NA),
    }
}

#[cfg(test)]
mod tests;
