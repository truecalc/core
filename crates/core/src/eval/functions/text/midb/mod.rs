use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `MIDB(text, start_byte, num_bytes)` — returns a substring by byte position.
/// start_byte is 1-based. For ASCII text this is identical to MID.
/// Returns `#VALUE!` if start_byte < 1 or num_bytes < 0.
pub fn midb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let num_bytes = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if start < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    if num_bytes < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start_byte = (start as usize) - 1;
    let num_bytes = num_bytes as usize;
    let total = text.len();
    if start_byte >= total {
        return Value::Text(String::new());
    }
    // Snap to char boundary
    let start_byte = (start_byte..=total)
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(total);
    let end_byte = (start_byte + num_bytes).min(total);
    let end_byte = (end_byte..=total)
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(total);
    Value::Text(text[start_byte..end_byte].to_string())
}

#[cfg(test)]
mod tests;
