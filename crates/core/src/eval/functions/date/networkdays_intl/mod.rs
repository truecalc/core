use chrono::{Datelike, Duration};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::eval::functions::date::weekend::weekend_mask;
use crate::types::{ErrorKind, Value};

/// `NETWORKDAYS.INTL(start, end, [weekend], [holidays])` — count of working days
/// with a configurable weekend pattern.  If start > end, result is negative.
/// The optional holidays argument is accepted but ignored.
pub fn networkdays_intl_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 4) {
        return err;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let end_serial   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let mask = match weekend_mask(args.get(2)) {
        Ok(m) => m,
        Err(e) => return e,
    };

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
        let wd = current.weekday().num_days_from_monday() as usize;
        if !mask[wd] {
            count += 1;
        }
        current += Duration::days(1);
    }

    Value::Number((count * sign) as f64)
}

#[cfg(test)]
mod tests;
