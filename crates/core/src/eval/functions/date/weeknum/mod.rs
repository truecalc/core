use chrono::Datelike;
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `WEEKNUM(date_serial, [return_type])` — week number of the year.
///
/// return_type:
/// - 1 (default): week starts Sunday
/// - 2: week starts Monday
/// - 11: week starts Monday (same as 2)
/// - 12: week starts Tuesday
/// - 13: week starts Wednesday
/// - 14: week starts Thursday
/// - 15: week starts Friday
/// - 16: week starts Saturday
/// - 17: week starts Sunday (same as 1)
/// - 21: ISO week number (same as ISOWEEKNUM)
pub fn weeknum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let return_type = if args.len() > 1 {
        match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e }
    } else {
        1.0
    };

    let date = match serial_to_date(serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if return_type as u32 == 21 {
        return Value::Number(date.iso_week().week() as f64);
    }

    let jan1 = chrono::NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap();
    let day_of_year = date.ordinal() as i32 - 1; // 0-indexed

    // Offset: how many days before the first week-start day after/on Jan 1
    // num_days_from_X returns 0 for X, cycling through the week
    let offset = match return_type as u32 {
        1 | 17 => jan1.weekday().num_days_from_sunday() as i32,
        2 | 11 => jan1.weekday().num_days_from_monday() as i32,
        12     => (jan1.weekday().num_days_from_monday() as i32 + 6) % 7, // Tue=0
        13     => (jan1.weekday().num_days_from_monday() as i32 + 5) % 7, // Wed=0
        14     => (jan1.weekday().num_days_from_monday() as i32 + 4) % 7, // Thu=0
        15     => (jan1.weekday().num_days_from_monday() as i32 + 3) % 7, // Fri=0
        16     => (jan1.weekday().num_days_from_monday() as i32 + 2) % 7, // Sat=0
        _      => return Value::Error(ErrorKind::Num),
    };

    let week = (day_of_year + offset) / 7 + 1;
    Value::Number(week as f64)
}

#[cfg(test)]
mod tests;
