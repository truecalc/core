use chrono::{Datelike, NaiveDate};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn year_days(year: i32) -> f64 {
    if is_leap(year) { 366.0 } else { 365.0 }
}

/// US 30/360 (NASD) day count.
fn days_30_360_us(start: NaiveDate, end: NaiveDate) -> i32 {
    let (mut d1, m1, y1) = (start.day() as i32, start.month() as i32, start.year());
    let (mut d2, m2, y2) = (end.day() as i32, end.month() as i32, end.year());
    if d1 == 31 { d1 = 30; }
    if d2 == 31 && d1 == 30 { d2 = 30; }
    (y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1)
}

/// European 30/360 day count.
fn days_30_360_eu(start: NaiveDate, end: NaiveDate) -> i32 {
    let (mut d1, m1, y1) = (start.day() as i32, start.month() as i32, start.year());
    let (mut d2, m2, y2) = (end.day() as i32, end.month() as i32, end.year());
    if d1 == 31 { d1 = 30; }
    if d2 == 31 { d2 = 30; }
    (y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1)
}

/// Actual/Actual ISDA: split at year boundaries and divide by each year's length.
fn yearfrac_actual_actual(start: NaiveDate, end: NaiveDate) -> f64 {
    if start == end {
        return 0.0;
    }
    let (start, end) = if start <= end { (start, end) } else { (end, start) };
    let actual_days = end.signed_duration_since(start).num_days();

    if start.year() == end.year() {
        return actual_days as f64 / year_days(start.year());
    }

    // Portion in the first year
    let first_year_end = NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).unwrap();
    let days_first = first_year_end.signed_duration_since(start).num_days() as f64;
    let mut total = days_first / year_days(start.year());

    // Full intermediate years
    for y in (start.year() + 1)..end.year() {
        total += 1.0;
        let _ = y;
    }

    // Portion in the last year
    let last_year_start = NaiveDate::from_ymd_opt(end.year(), 1, 1).unwrap();
    let days_last = end.signed_duration_since(last_year_start).num_days() as f64;
    total += days_last / year_days(end.year());

    total
}

/// `YEARFRAC(start_date, end_date, [basis])` — fraction of year between two dates.
pub fn yearfrac_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let end_serial   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let basis = if args.len() > 2 {
        match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e }
    } else {
        0.0
    };

    let start = match serial_to_date(start_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let end = match serial_to_date(end_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    let result = match basis as u32 {
        0 => days_30_360_us(start, end) as f64 / 360.0,
        1 => yearfrac_actual_actual(start, end),
        2 => end.signed_duration_since(start).num_days() as f64 / 360.0,
        3 => end.signed_duration_since(start).num_days() as f64 / 365.0,
        4 => days_30_360_eu(start, end) as f64 / 360.0,
        _ => return Value::Error(ErrorKind::Num),
    };

    Value::Number(result)
}

#[cfg(test)]
mod tests;
