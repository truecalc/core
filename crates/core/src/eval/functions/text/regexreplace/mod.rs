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
    let text = match &args[0] {
        Value::Text(s) => s.clone(),
        Value::Empty => String::new(),
        Value::Error(e) => return Value::Error(e.clone()),
        _ => return Value::Error(ErrorKind::Value),
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
        Err(_) => return Value::Error(ErrorKind::Ref),
    };
    // GS behaviour: after a non-empty match, skip any zero-length match at the
    // same position so that e.g. REGEXREPLACE("hello",".*","world") -> "world"
    // (not "worldworld" which regex_lite::replace_all would produce).
    let text_bytes = text.as_bytes();
    let mut result = String::new();
    let mut last_end = 0usize;
    let mut prev_match_end: Option<usize> = None;
    for m in re.find_iter(&text) {
        // Skip a zero-length match at the same byte offset as the previous match end
        if m.start() == m.end() {
            if let Some(prev) = prev_match_end {
                if m.start() == prev {
                    continue;
                }
            }
        }
        result.push_str(&text[last_end..m.start()]);
        result.push_str(&replacement);
        last_end = m.end();
        prev_match_end = Some(m.end());
        // After a zero-length match, advance by one byte to avoid infinite loop
        if m.start() == m.end() {
            if last_end < text_bytes.len() {
                result.push(text_bytes[last_end] as char);
                last_end += 1;
            }
        }
    }
    result.push_str(&text[last_end..]);
    let _ = text_bytes; // suppress unused warning
    Value::Text(result)
}

#[cfg(test)]
mod tests;
