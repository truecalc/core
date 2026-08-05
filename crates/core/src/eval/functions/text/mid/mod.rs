use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::len::utf16_units;
use crate::types::{ErrorKind, Value};

/// `MID(text, start_num, num_chars)` — returns a substring starting at `start_num` (1-based).
/// Returns `#VALUE!` if start_num < 1 or num_chars < 0.
///
/// Indexes by UTF-16 code unit, matching Google Sheets' internal text
/// representation — an astral character (surrogate pair) counts as 2
/// positions, and a window that lands mid-pair splits it. See issue #848.
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
    let units = utf16_units(&text);
    let start = (start as usize) - 1;
    let start = start.min(units.len());
    let take = (num_chars as usize).min(units.len() - start);
    let result = String::from_utf16_lossy(&units[start..start + take]);
    Value::Text(result)
}

#[cfg(test)]
mod tests;
