use chrono::Local;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::date_to_serial;
use crate::types::Value;

/// `TODAY()` — returns the current local date as a spreadsheet serial number.
pub fn today_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 0, 0) {
        return e;
    }
    let today = Local::now().date_naive();
    Value::Number(date_to_serial(today))
}

#[cfg(test)]
mod tests;
