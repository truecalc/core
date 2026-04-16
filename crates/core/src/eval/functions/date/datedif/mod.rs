use chrono::{Datelike, NaiveDate};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `DATEDIF(start_date, end_date, unit)` — difference between two dates.
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
            // Complete months elapsed from start to end, then measure remaining days.
            let total_months = (end.year() - start.year()) * 12
                + (end.month() as i32 - start.month() as i32);
            let had_day_pass = end.day() >= start.day();
            let complete_months = if had_day_pass { total_months } else { total_months - 1 };
            let adjusted_start = add_months(start, complete_months);
            Value::Number((end - adjusted_start).num_days() as f64)
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

/// Add `months` months to a date, clamping to end-of-month on overflow.
fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total_months = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total_months / 12;
    let month = (total_months % 12 + 1) as u32;
    NaiveDate::from_ymd_opt(year, month, date.day())
        .or_else(|| NaiveDate::from_ymd_opt(year, month + 1, 1))
        .unwrap()
}

#[cfg(test)]
mod tests;
