use chrono::{NaiveTime, Timelike};
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::time_to_serial;
use crate::types::{ErrorKind, Value};

/// `TIMEVALUE(time_text)` — parses a time string and returns a fractional day serial.
///
/// Supports HH:MM:SS, HH:MM, and 12-hour formats with AM/PM.
/// Returns `Value::Error(ErrorKind::Value)` for unparseable strings.
pub fn timevalue_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match &args[0] {
        Value::Text(s) => s.clone(),
        Value::Error(_) => return args[0].clone(),
        _ => return Value::Error(ErrorKind::Value),
    };

    let formats = [
        "%H:%M:%S",
        "%H:%M",
        "%I:%M %p",
        "%I:%M:%S %p",
        "%I:%M%p",
        "%I:%M:%S%p",
    ];

    for fmt in &formats {
        if let Ok(t) = NaiveTime::parse_from_str(&text, fmt) {
            let serial = time_to_serial(t.hour(), t.minute(), t.second());
            return Value::Number(serial);
        }
    }

    Value::Error(ErrorKind::Value)
}

#[cfg(test)]
mod tests;
