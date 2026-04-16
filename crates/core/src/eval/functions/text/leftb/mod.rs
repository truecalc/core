use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `LEFTB(text, num_bytes)` — returns the first N bytes of a string.
/// For ASCII text this is identical to LEFT. Returns `#VALUE!` if num_bytes < 0.
pub fn leftb_fn(args: &[Value]) -> Value {
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
    // For ASCII, byte count == char count. Take chars until byte budget exhausted.
    let mut byte_count = 0usize;
    let result: String = text
        .chars()
        .take_while(|c| {
            let cb = c.len_utf8();
            if byte_count + cb <= n {
                byte_count += cb;
                true
            } else {
                false
            }
        })
        .collect();
    Value::Text(result)
}

#[cfg(test)]
mod tests;
