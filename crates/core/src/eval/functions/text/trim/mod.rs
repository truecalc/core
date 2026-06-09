use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `TRIM(text)` — removes leading/trailing whitespace and collapses internal spaces.
pub fn trim_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    // GS TRIM: strips/collapses ASCII whitespace (space, tab, newline, etc.)
    // but does NOT treat U+00A0 (non-breaking space, CHAR(160)) as whitespace.
    // Strategy: split on runs of ASCII whitespace (chars where c.is_ascii_whitespace()),
    // filter empty segments, rejoin with single space.
    let result: String = text
        .split(|c: char| c.is_ascii_whitespace())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    Value::Text(result)
}

#[cfg(test)]
mod tests;
