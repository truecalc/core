use chrono::{Datelike, Duration};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{date_to_serial, serial_to_date};
use crate::types::{ErrorKind, Value};

/// `WORKDAY(start_date, days, [holidays])` — date serial that is `days` working days
/// (Mon–Fri) from `start_date`.  Negative `days` moves backward.  The start date
/// itself is not counted.  The optional holidays argument is accepted but ignored.
pub fn workday_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let days_raw     = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let start = match serial_to_date(start_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    let days = days_raw as i64;
    if days == 0 {
        return Value::Number(date_to_serial(start));
    }

    let step = if days > 0 { 1i64 } else { -1i64 };
    let mut remaining = days.abs();
    let mut current = start;

    while remaining > 0 {
        current += Duration::days(step);
        let wd = current.weekday().num_days_from_monday();
        if wd < 5 {
            remaining -= 1;
        }
    }

    Value::Number(date_to_serial(current))
}

#[cfg(test)]
mod tests;
