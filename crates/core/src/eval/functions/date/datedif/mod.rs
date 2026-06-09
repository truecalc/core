use chrono::{Datelike, NaiveDate};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `DATEDIF(start_date, end_date, unit)` -- difference between two dates.
pub fn datedif_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 3, 3) {
        return e;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let end_serial   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let unit = match &args[2] {
        Value::Text(s) => s.to_uppercase(),
        _ => return Value::Error(ErrorKind::Num),
    };

    let start = match serial_to_date(start_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Num),
    };
    let end = match serial_to_date(end_serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Num),
    };

    if start > end {
        return Value::Error(ErrorKind::Num);
    }

    match unit.as_str() {
        "Y" => {
            let years = end.year() - start.year();
            let had_anniversary = (end.month(), end.day()) >= (start.month(), start.day());
            Value::Number(if had_anniversary { years } else { years - 1 } as f64)
        }
        "M" => {
            let months = (end.year() - start.year()) * 12
                + (end.month() as i32 - start.month() as i32);
            let had_day_pass = end.day() >= start.day();
            Value::Number(if had_day_pass { months } else { months - 1 } as f64)
        }
        "D" => {
            Value::Number((end - start).num_days() as f64)
        }
        "MD" => {
            // Google Sheets implementation: end.day - start.day, and if the result
            // is negative, add the number of days in the month before end.
            let diff = end.day() as i32 - start.day() as i32;
            let result = if diff < 0 {
                let prev = prev_month(end.year(), end.month());
                diff + days_in_month(prev.0, prev.1) as i32
            } else {
                diff
            };
            Value::Number(result as f64)
        }
        "YM" => {
            let total_months = (end.year() - start.year()) * 12
                + (end.month() as i32 - start.month() as i32);
            let had_day_pass = end.day() >= start.day();
            let complete_months = if had_day_pass { total_months } else { total_months - 1 };
            Value::Number((complete_months % 12) as f64)
        }
        "YD" => {
            // Days from start to end as if they were in the same year (start's year).
            let same_year_end = NaiveDate::from_ymd_opt(start.year(), end.month(), end.day())
                .or_else(|| NaiveDate::from_ymd_opt(start.year(), end.month() + 1, 1))
                .unwrap();
            let days = if same_year_end >= start {
                (same_year_end - start).num_days()
            } else {
                let next_year_end = NaiveDate::from_ymd_opt(start.year() + 1, end.month(), end.day())
                    .or_else(|| NaiveDate::from_ymd_opt(start.year() + 1, end.month() + 1, 1))
                    .unwrap();
                (next_year_end - start).num_days()
            };
            Value::Number(days as f64)
        }
        _ => Value::Error(ErrorKind::Num),
    }
}

/// Returns (year, month) for the month preceding the given month.
fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

/// Number of days in the given (year, month).
fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year  = if month == 12 { year + 1 } else { year };
    let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    let first_of_curr = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    (first_of_next - first_of_curr).num_days() as u32
}

#[cfg(test)]
mod tests;
