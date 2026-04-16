use chrono::Datelike;
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::{ErrorKind, Value};

/// `ISOWEEKNUM(date_serial)` — ISO 8601 week number.
///
/// Week 1 is the week containing the first Thursday of the year.
/// Weeks run Monday–Sunday. Uses chrono's built-in ISO week calculation.
pub fn isoweeknum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let serial = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let date = match serial_to_date(serial) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    Value::Number(date.iso_week().week() as f64)
}

#[cfg(test)]
mod tests;
