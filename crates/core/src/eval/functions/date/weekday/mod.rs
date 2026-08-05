use chrono::Datelike;
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `WEEKDAY(date_serial, [return_type])` — day of week as an integer.
///
/// return_type mapping:
/// - 1 (default) / 17: Sunday=1, Saturday=7
/// - 2 / 11: Monday=1, Sunday=7
/// - 3: Monday=0, Sunday=6
/// - 12: Tuesday=1, Monday=7
/// - 13: Wednesday=1, Tuesday=7
/// - 14: Thursday=1, Wednesday=7
/// - 15: Friday=1, Thursday=7
/// - 16: Saturday=1, Friday=7
pub fn weekday_fn(args: &[Value]) -> Value {
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

    // chrono: 0=Mon, 1=Tue, ..., 6=Sun
    let wd = date.weekday().num_days_from_monday() as i32;

    let result = match return_type as u32 {
        1 | 17 => ((wd + 1) % 7) + 1,  // Sun=1, Mon=2, ..., Sat=7
        2 | 11 => wd + 1,               // Mon=1, ..., Sun=7
        3      => wd,                   // Mon=0, ..., Sun=6
        12     => (wd + 6) % 7 + 1,    // Tue=1, ..., Mon=7
        13     => (wd + 5) % 7 + 1,    // Wed=1, ..., Tue=7
        14     => (wd + 4) % 7 + 1,    // Thu=1, ..., Wed=7
        15     => (wd + 3) % 7 + 1,    // Fri=1, ..., Thu=7
        16     => (wd + 2) % 7 + 1,    // Sat=1, ..., Fri=7
        _      => return Value::Error(ErrorKind::Num),
    };

    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
