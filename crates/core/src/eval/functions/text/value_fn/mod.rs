use crate::eval::coercion::{parse_number_text, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{text_to_date_serial, text_to_time_serial};
use crate::types::{ErrorKind, Value};

/// `VALUE(text)` — parses a text string to a number. Returns `#VALUE!` if unparseable.
/// Empty string returns 0, matching Google Sheets behaviour.
/// Handles comma-formatted numbers (`1,234.56`), percentages (`12%`), and currency (`$42`).
/// Shares its numeric-parsing rules with implicit arithmetic coercion via
/// [`parse_number_text`] so the two agree on every input.
pub fn value_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if let Some(n) = parse_number_text(&text) {
        return Value::Number(n);
    }
    let trimmed = text.trim();
    // Date string: "7/20/1969" -> 25404
    if let Some(serial) = text_to_date_serial(trimmed) {
        return Value::Number(serial);
    }
    // Time string: "12:00:00" -> 0.5
    if let Some(frac) = text_to_time_serial(trimmed) {
        return Value::Number(frac);
    }
    Value::Error(ErrorKind::Value)
}

#[cfg(test)]
mod tests;
