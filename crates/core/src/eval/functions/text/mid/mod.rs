use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `MID(text, start_num, num_chars)` — returns a substring starting at `start_num` (1-based).
/// Returns `#VALUE!` if start_num < 1 or num_chars < 0.
pub fn mid_fn(args: &[Value]) -> Value {
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
    let num_chars = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    // GS: start < 1 → domain error #NUM!; num_chars < 0 → type error #VALUE!
    if start < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    if num_chars < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start = (start as usize) - 1;
    let num_chars = num_chars as usize;
    let result: String = text.chars().skip(start).take(num_chars).collect();
    Value::Text(result)
}

#[cfg(test)]
mod tests;
