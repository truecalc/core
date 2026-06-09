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

/// Convert a 1-based DBCS byte offset to a char-index boundary.
/// If the byte offset falls inside a 2-byte char, snap forward to the next char boundary.
pub(crate) fn dbcs_byte_to_char_idx(s: &str, dbcs_byte_1based: usize) -> usize {
    if dbcs_byte_1based == 0 {
        return 0;
    }
    let target = dbcs_byte_1based - 1; // 0-based
    let mut pos = 0usize;
    for (i, c) in s.chars().enumerate() {
        if pos >= target {
            return i;
        }
        pos += dbcs_char_width(c);
    }
    s.chars().count()
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
