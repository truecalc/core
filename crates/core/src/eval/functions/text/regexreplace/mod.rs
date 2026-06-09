use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use regex_lite::Regex;

/// `REGEXREPLACE(text, pattern, replacement)` -- replaces ALL non-overlapping matches
/// of pattern in text with replacement. Matches GS semantics: zero-length matches
/// after non-empty matches are included (unlike regex_lite default behaviour).
pub fn regexreplace_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) { return err; }
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
    // GS includes zero-length matches after non-empty matches.
    // regex_lite skips them; we scan manually to match GS semantics.
    let text_bytes = text.as_bytes();
    let text_len = text_bytes.len();
    let mut result = String::new();
    let mut pos = 0usize;
    while pos <= text_len {
        let slice = &text[pos..];
        if let Some(m) = re.find(slice) {
            let abs_start = pos + m.start();
            let abs_end = pos + m.end();
            result.push_str(&text[pos..abs_start]);
            result.push_str(&replacement);
            if m.start() == m.end() {
                // Zero-length match: copy current byte to advance
                if abs_end < text_len {
                    // Safety: abs_end is a valid UTF-8 char boundary because
                    // regex_lite only matches at char boundaries.
                    let next_char_end = text[abs_end..].chars().next()
                        .map(|c| abs_end + c.len_utf8())
                        .unwrap_or(text_len);
                    result.push_str(&text[abs_end..next_char_end]);
                    pos = next_char_end;
                } else {
                    pos = abs_end + 1;
                }
            } else {
                pos = abs_end;
            }
        } else {
            result.push_str(&text[pos..]);
            break;
        }
    }
    Value::Text(result)
}

#[cfg(test)]
mod tests;
