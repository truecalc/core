use chrono::{Datelike, Duration};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{date_to_serial, serial_to_date};
use crate::eval::functions::date::weekend::weekend_mask;
use crate::types::{ErrorKind, Value};

/// `WORKDAY.INTL(start, days, [weekend], [holidays])` — date serial that is `days`
/// working days from `start`, with a configurable weekend pattern.
/// Negative `days` moves backward.  The start date itself is not counted.
/// The optional holidays argument is accepted but ignored.
pub fn workday_intl_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 4) {
        return err;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let days_raw     = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let mask = match weekend_mask(args.get(2)) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let start = match serial_to_date(start_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    // All-weekend mask makes it impossible to advance — return #NUM!
    if mask.iter().all(|&w| w) {
        return Value::Error(ErrorKind::Num);
    }

    let days = days_raw as i64;
    if days == 0 {
        return Value::Number(date_to_serial(start));
    }

    let step = if days > 0 { 1i64 } else { -1i64 };
    let mut remaining = days.abs();
    let mut current = start;

    while remaining > 0 {
        current += Duration::days(step);
        let wd = current.weekday().num_days_from_monday() as usize;
        if !mask[wd] {
            remaining -= 1;
        }
    }

    Value::Number(date_to_serial(current))
}

#[cfg(test)]
mod tests;
