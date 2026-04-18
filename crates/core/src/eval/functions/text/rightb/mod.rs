use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `RIGHTB(text, num_bytes)` — returns the last N bytes of a string.
/// For ASCII text this is identical to RIGHT. Returns `#VALUE!` if num_bytes < 0.
pub fn rightb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let n = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if n < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let n = n as usize;
    let total_bytes = text.len();
    let skip_bytes = total_bytes.saturating_sub(n);
    // Find the char boundary at or after skip_bytes
    let start = (skip_bytes..=total_bytes)
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(total_bytes);
    Value::Text(text[start..].to_string())
}

#[cfg(test)]
mod tests;
