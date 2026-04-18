use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `FIND(find_text, within_text, [start_num])` — returns the 1-based position of find_text.
/// Case-sensitive. Returns `#VALUE!` if not found or start_num < 1.
pub fn find_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let find_text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let within_text = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start_num = if args.len() == 3 {
        match to_number(args[2].clone()) {
            Ok(n) => n,
            Err(e) => return e,
        }
    } else {
        1.0
    };
    if start_num < 1.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start_idx = (start_num as usize) - 1;
    let within_chars: Vec<char> = within_text.chars().collect();
    if start_idx > within_chars.len() {
        return Value::Error(ErrorKind::Value);
    }
    let search_in: String = within_chars[start_idx..].iter().collect();
    match search_in.find(&find_text) {
        Some(byte_pos) => {
            // Convert byte offset to char offset
            let char_offset = search_in[..byte_pos].chars().count();
            Value::Number((start_idx + char_offset + 1) as f64)
        }
        None => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
