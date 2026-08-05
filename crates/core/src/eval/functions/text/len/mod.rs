use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// UTF-16 code-unit count of `s`, matching Google Sheets' internal text
/// representation: a character outside the Basic Multilingual Plane (an
/// astral character — most emoji, mathematical alphanumeric symbols, etc.)
/// is encoded as a surrogate pair and counts as 2. See issue #848.
pub(crate) fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// UTF-16 code units of `s`, for slicing text by code-unit position the way
/// MID/LEFT/RIGHT do.
pub(crate) fn utf16_units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `LEN(text)` — returns the number of characters in a string.
///
/// Counts UTF-16 code units (Google Sheets' internal text representation),
/// not Unicode codepoints — see [`utf16_len`].
pub fn len_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    Value::Number(utf16_len(&text) as f64)
}

#[cfg(test)]
mod tests;
