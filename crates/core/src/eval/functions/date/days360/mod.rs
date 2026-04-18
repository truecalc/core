use chrono::Datelike;
use crate::eval::coercion::{to_bool, to_number};
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `DAYS360(start_date, end_date, [method])` — days between dates on a 360-day calendar.
///
/// Every month is treated as 30 days.
/// method=FALSE (default): US (NASD) convention.
/// method=TRUE: European convention.
pub fn days360_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 3) {
        return e;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let end_serial   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let european = if args.len() == 3 {
        match to_bool(args[2].clone()) { Ok(b) => b, Err(e) => return e }
    } else {
        false
    };

    let start_date = match serial_to_date(start_serial) {
        Some(d) => d,
        None    => return Value::Error(ErrorKind::Num),
    };
    let end_date = match serial_to_date(end_serial) {
        Some(d) => d,
        None    => return Value::Error(ErrorKind::Num),
    };

    let y1 = start_date.year();
    let m1 = start_date.month() as i32;
    let mut d1 = start_date.day() as i32;

    let y2 = end_date.year();
    let m2 = end_date.month() as i32;
    let mut d2 = end_date.day() as i32;

    if european {
        if d1 == 31 { d1 = 30; }
        if d2 == 31 { d2 = 30; }
    } else {
        // US (NASD) method
        let start_is_feb_end = m1 == 2 && is_last_day_of_month(start_date);
        let end_is_feb_end   = m2 == 2 && is_last_day_of_month(end_date);

        if start_is_feb_end || d1 == 31 {
            d1 = 30;
        }

        if (start_is_feb_end && end_is_feb_end) || (d2 == 31 && d1 == 30) {
            d2 = 30;
        }
    }

    let result = (y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1);
    Value::Number(result as f64)
}

fn is_last_day_of_month(date: chrono::NaiveDate) -> bool {
    // The last day of a month is when adding 1 day changes the month
    match date.succ_opt() {
        Some(next) => next.month() != date.month(),
        None => true, // at max date, treat as last day
    }
}

#[cfg(test)]
mod tests;
