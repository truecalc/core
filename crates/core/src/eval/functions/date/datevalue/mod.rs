use chrono::NaiveDate;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::date_to_serial;
use crate::types::{ErrorKind, Value};

/// `DATEVALUE(date_text)` — parses a date string and returns its serial number.
///
/// Supports common formats: ISO (YYYY-MM-DD), US slash (M/D/YYYY), and long/short month names.
/// Returns `Value::Error(ErrorKind::Value)` for unparseable or invalid strings.
pub fn datevalue_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match &args[0] {
        Value::Text(s) => s.clone(),
        Value::Error(_) => return args[0].clone(),
        _ => return Value::Error(ErrorKind::Value),
    };

    if text.is_empty() {
        return Value::Error(ErrorKind::Value);
    }

    let formats = [
        "%m/%d/%Y",
        "%m/%d/%y",
        "%Y-%m-%d",
        "%d-%b-%Y",
        "%d-%b-%y",
        "%B %d, %Y",
        "%b %d, %Y",
        "%B %d %Y",
        "%b %d %Y",
    ];

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(&text, fmt) {
            return Value::Number(date_to_serial(date));
        }
    }

    Value::Error(ErrorKind::Value)
}

#[cfg(test)]
mod tests;
