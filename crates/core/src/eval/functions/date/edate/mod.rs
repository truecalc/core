use chrono::Months;
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{date_to_serial, serial_to_date};
use crate::types::{ErrorKind, Value};

/// `EDATE(start_date, months)` — serial date N whole months from start_date.
///
/// If the resulting month has fewer days, clamps to the last day of that month.
/// Negative months move backward.
pub fn edate_fn(args: &[Value]) -> Value {
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

    let result = if months >= 0 {
        date.checked_add_months(Months::new(months_u32))
    } else {
        date.checked_sub_months(Months::new(months_u32))
    };

    match result {
        Some(d) => Value::Number(date_to_serial(d)),
        None    => Value::Error(ErrorKind::Num),
    }
}

#[cfg(test)]
mod tests;
