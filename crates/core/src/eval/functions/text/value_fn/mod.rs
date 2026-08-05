use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{text_to_date_serial, text_to_time_serial};
use crate::types::{ErrorKind, Value};

/// `VALUE(text)` — parses a text string to a number. Returns `#VALUE!` if unparseable.
/// Empty string returns 0, matching Google Sheets behaviour.
/// Handles comma-formatted numbers (`1,234.56`), percentages (`12%`), and currency (`$42`).
pub fn value_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Value::Number(0.0);
    }
    // Try direct parse first
    if let Ok(n) = trimmed.parse::<f64>() {
        return Value::Number(n);
    }
    // Handle percentage: "12%" → 0.12
    if let Some(pct) = trimmed.strip_suffix('%') {
        if let Ok(n) = pct.trim().replace(',', "").parse::<f64>() {
            return Value::Number(n / 100.0);
        }
    }
    // Handle currency prefix: "$42" → 42
    if let Some(rest) = trimmed.strip_prefix('$') {
        if let Ok(n) = rest.trim().replace(',', "").parse::<f64>() {
            return Value::Number(n);
        }
    }
    // Handle comma-formatted numbers: "1,234.56" → 1234.56
    let no_commas = trimmed.replace(',', "");
    if let Ok(n) = no_commas.parse::<f64>() {
        return Value::Number(n);
    }
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
