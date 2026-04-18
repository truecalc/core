use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `REPLACEB(old_text, start_byte, num_bytes, new_text)` — replaces N bytes starting at byte position.
/// start_byte is 1-based. For ASCII text this is identical to REPLACE.
/// Returns `#VALUE!` if start_byte < 1 or num_bytes < 0.
pub fn replaceb_fn(args: &[Value]) -> Value {
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
    let num_bytes = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let new_text = match to_string_val(args[3].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if start_num < 1.0 || num_bytes < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start_byte = (start_num as usize) - 1;
    let num_bytes = num_bytes as usize;
    let total = text.len();
    let start_byte = start_byte.min(total);
    // Snap to char boundary
    let start_byte = (start_byte..=total)
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(total);
    let end_byte = (start_byte + num_bytes).min(total);
    let end_byte = (end_byte..=total)
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(total);
    Value::Text(format!("{}{}{}", &text[..start_byte], new_text, &text[end_byte..]))
}

#[cfg(test)]
mod tests;
