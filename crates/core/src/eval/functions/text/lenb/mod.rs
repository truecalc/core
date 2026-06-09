use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// DBCS byte width: codepoints >= U+0100 count as 2 bytes, others as 1.
/// Matches Google Sheets / Excel DBCS semantics.
pub(crate) fn dbcs_char_width(c: char) -> usize {
    if (c as u32) >= 256 { 2 } else { 1 }
}

pub(crate) fn dbcs_len(s: &str) -> usize {
    s.chars().map(dbcs_char_width).sum()
}

/// `LENB(text)` — returns the DBCS byte count of a string.
/// Non-Latin-1 characters (codepoint >= 256) count as 2 bytes; others count as 1.
pub fn lenb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    Value::Number(dbcs_len(&text) as f64)
}

#[cfg(test)]
mod tests;
