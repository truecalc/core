use chrono::{Datelike, Duration, NaiveDate};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::date_to_serial;
use crate::types::{ErrorKind, Value};

/// `DATE(year, month, day)` — returns the spreadsheet serial number for the given date.
///
/// Handles month and day overflow/underflow (Google Sheets compatible):
/// - Month 13 of 2024 → January 2025
/// - Month 0 of 2024 → December 2023
/// - Day 0 of March → last day of February
/// All inputs are truncated to integer before use.
pub fn date_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let year  = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let month = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let day   = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };

    // Truncate to integers (Google Sheets truncates, not rounds)
    let mut year_i  = year.trunc() as i64;
    let month_i = month.trunc() as i64;
    let day_i   = day.trunc() as i64;

    // Normalize month into 1..=12, adjusting year
    let month_adj = month_i - 1; // 0-indexed
    let year_delta = month_adj.div_euclid(12);
    let month_norm = (month_adj.rem_euclid(12) + 1) as u32; // back to 1-indexed
    year_i += year_delta;

    if year_i < 0 || year_i > 9999 {
        return Value::Error(ErrorKind::Num);
    }

    // Build the 1st of the normalized month, then add (day - 1) days
    let base = match NaiveDate::from_ymd_opt(year_i as i32, month_norm, 1) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Num),
    };
    let date = match base.checked_add_signed(Duration::days(day_i - 1)) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Num),
    };

    // Ensure within valid spreadsheet range
    let serial = date_to_serial(date);
    if serial < 0.0 || date.year() > 9999 {
        return Value::Error(ErrorKind::Num);
    }

    Value::Number(serial)
}

#[cfg(test)]
mod tests;
