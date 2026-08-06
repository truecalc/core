use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::len::utf16_units;
use crate::types::{ErrorKind, Value};

/// `LEFT(text, [num_chars])` — returns the first N characters of a string.
/// Default N=1. Returns `#VALUE!` if N < 0.
///
/// Indexes by UTF-16 code unit, matching Google Sheets' internal text
/// representation — an astral character (surrogate pair) counts as 2
/// positions, and a window that lands mid-pair splits it. See issue #848.
pub fn left_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let n = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Ok(n) => n,
            Err(e) => return e,
        }
    } else {
        1.0
    };
    if n < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let units = utf16_units(&text);
    let end = (n as usize).min(units.len());
    let result = String::from_utf16_lossy(&units[..end]);
    Value::Text(result)
}

#[cfg(test)]
mod tests;
