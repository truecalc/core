use chrono::{Datelike, Duration};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `NETWORKDAYS(start_date, end_date, [holidays])` — count of working days (Mon–Fri),
/// inclusive of both endpoints.  If start > end, the result is negative.
/// The optional holidays argument is accepted but ignored (array args unsupported).
pub fn networkdays_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let end_serial   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let start = match serial_to_date(start_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let end = match serial_to_date(end_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    let (from, to, sign) = if start <= end {
        (start, end, 1i64)
    } else {
        (end, start, -1i64)
    };

    let mut count = 0i64;
    let mut current = from;
    while current <= to {
        let wd = current.weekday().num_days_from_monday();
        if wd < 5 {
            count += 1;
        }
        current += Duration::days(1);
    }

    Value::Number((count * sign) as f64)
}

#[cfg(test)]
mod tests;
