use chrono::{Local, Timelike};
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{date_to_serial, time_to_serial};
use crate::types::Value;

/// `NOW()` — returns the current local datetime as a spreadsheet serial number.
/// Integer part = date serial; fractional part = time-of-day.
pub fn now_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 0, 0) {
        return e;
    }
    let now = Local::now().naive_local();
    let date_serial = date_to_serial(now.date());
    let time_frac = time_to_serial(now.hour(), now.minute(), now.second());
    Value::Number(date_serial + time_frac)
}

#[cfg(test)]
mod tests;
