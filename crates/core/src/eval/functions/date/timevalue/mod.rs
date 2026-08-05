use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::text_to_time_serial;
use crate::types::{ErrorKind, Value};

/// `TIMEVALUE(time_text)` -- parses a time string and returns a fractional day serial.
///
/// Supports HH:MM:SS, HH:MM, 12-hour AM/PM, fractional seconds ("11:59:59.50 PM"),
/// and datetime strings (date part is discarded, e.g. "2023-06-15 14:30:00").
/// Returns `Value::Error(ErrorKind::Value)` for unparseable strings or non-text inputs.
pub fn timevalue_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match &args[0] {
        Value::Text(s) => s.clone(),
        Value::Error(_) | Value::ErrorMsg(_, _) => return args[0].clone(),
        _ => return Value::Error(ErrorKind::Value),
    };

    if text.is_empty() {
        return Value::Error(ErrorKind::Value);
    }

    match text_to_time_serial(&text) {
        Some(serial) => Value::Number(serial),
        None => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
