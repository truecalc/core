use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `FINDB(find_text, within_text, [start_num])` — returns the 1-based byte position of find_text.
/// Case-sensitive. For ASCII text this is identical to FIND.
/// Returns `#VALUE!` if not found or start_num < 1.
pub fn findb_fn(args: &[Value]) -> Value {
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
    let start_byte = (start_num as usize) - 1;
    let total = within_text.len();
    if start_byte > total {
        return Value::Error(ErrorKind::Value);
    }
    // Snap start to char boundary
    let start_byte = (start_byte..=total)
        .find(|&i| within_text.is_char_boundary(i))
        .unwrap_or(total);
    let search_in = &within_text[start_byte..];
    if find_text.is_empty() {
        return Value::Number((start_byte + 1) as f64);
    }
    match search_in.find(find_text.as_str()) {
        Some(rel_byte_pos) => Value::Number((start_byte + rel_byte_pos + 1) as f64),
        None => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
