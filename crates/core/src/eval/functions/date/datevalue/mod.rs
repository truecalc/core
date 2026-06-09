use chrono::{Datelike, NaiveDate};
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::date_to_serial;
use crate::types::{ErrorKind, Value};

/// Returns true if `s` looks like a slash-separated date where the year field
/// (the last `/`-delimited token) is exactly 1 or 2 digits.  Used to prevent
/// chrono's `%m/%d/%Y` from consuming "01/15/99" as year 99 CE instead of
/// routing it through the two-digit-year century-remap path.
fn has_two_digit_slash_year(s: &str) -> bool {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        let y = parts[2].trim();
        y.len() <= 2 && y.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// `DATEVALUE(date_text)` — parses a date string and returns its serial number.
///
/// Supports common formats: ISO (YYYY-MM-DD), US slash (M/D/YYYY), long/short month names,
/// slash Y/m/d, and datetime strings (the time component is stripped).
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

    // Strip trailing time component only when the suffix after the space looks
    // like a time string (starts with a digit followed by ':').  This preserves
    // month-name formats such as "January 1, 2023".
    let date_part: &str = if let Some(idx) = text.find(' ') {
        let after = &text[idx + 1..];
        let looks_like_time = after.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            && after.contains(':');
        if looks_like_time { &text[..idx] } else { text.as_str() }
    } else {
        text.as_str()
    };

    // Formats without two-digit year (%y) — parsed directly.
    // Skip %m/%d/%Y when the year field is only 1-2 digits; those inputs must
    // go through the century-remap path below (%m/%d/%y).
    let formats_4y = [
        "%m/%d/%Y",
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%b-%Y",
        "%B %d, %Y",
        "%b %d, %Y",
        "%B %d %Y",
        "%b %d %Y",
    ];

    for fmt in &formats_4y {
        // Guard: don't let %m/%d/%Y swallow two-digit-year slash inputs.
        if *fmt == "%m/%d/%Y" && has_two_digit_slash_year(date_part) {
            continue;
        }
        if let Ok(date) = NaiveDate::parse_from_str(date_part, fmt) {
            return Value::Number(date_to_serial(date));
        }
    }

    // Two-digit-year formats: chrono maps %y to the literal year (e.g. 99 → year 99),
    // but GS/Excel use 00-29 → 2000-2029 and 30-99 → 1930-1999.
    let formats_2y: &[&str] = &["%m/%d/%y", "%d-%b-%y"];
    for fmt in formats_2y {
        if let Ok(date) = NaiveDate::parse_from_str(date_part, fmt) {
            let y = date.year();
            // Remap raw two-digit year to spreadsheet century.
            let century_year = if y <= 29 { y + 2000 } else if y <= 99 { y + 1900 } else { y };
            if let Some(remapped) = NaiveDate::from_ymd_opt(century_year, date.month(), date.day()) {
                return Value::Number(date_to_serial(remapped));
            }
        }
    }

    Value::Error(ErrorKind::Value)
}

#[cfg(test)]
mod tests;
