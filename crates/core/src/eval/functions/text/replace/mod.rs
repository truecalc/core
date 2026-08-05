use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `REPLACE(old_text, start_num, num_chars, new_text)` — replaces N chars starting at position.
/// start_num is 1-based. Returns `#VALUE!` if start_num < 1 or num_chars < 0.
pub fn replace_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 4) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start_num = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let num_chars = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let new_text = match to_string_val(args[3].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if start_num < 1.0 || num_chars < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start = (start_num as usize) - 1;
    let num_chars = num_chars as usize;
    let chars: Vec<char> = text.chars().collect();
    let before: String = chars[..start.min(chars.len())].iter().collect();
    let after_start = (start + num_chars).min(chars.len());
    let after: String = chars[after_start..].iter().collect();
    Value::Text(format!("{}{}{}", before, new_text, after))
}

#[cfg(test)]
mod tests;
