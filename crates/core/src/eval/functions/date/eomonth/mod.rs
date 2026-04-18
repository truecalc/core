use chrono::{Datelike, Duration, Months, NaiveDate};
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{date_to_serial, serial_to_date};
use crate::types::{ErrorKind, Value};

/// `EOMONTH(start_date, months)` — serial of the last day of the month N months away.
///
/// Same month arithmetic as EDATE, but always returns the last day of the target month.
pub fn eomonth_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let start_serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let months_f     = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let date = match serial_to_date(start_serial) {
        Some(d) => d,
        None    => return Value::Error(ErrorKind::Num),
    };

    let months = months_f.trunc() as i64;
    let months_u32 = months.unsigned_abs() as u32;

    let target = if months >= 0 {
        date.checked_add_months(Months::new(months_u32))
    } else {
        date.checked_sub_months(Months::new(months_u32))
    };

    let target_date = match target {
        Some(d) => d,
        None    => return Value::Error(ErrorKind::Num),
    };

    let last_day = last_day_of_month(target_date.year(), target_date.month());
    Value::Number(date_to_serial(last_day))
}

fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    // First day of next month minus one day
    let first_of_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    first_of_next.unwrap() - Duration::days(1)
}

#[cfg(test)]
mod tests;
