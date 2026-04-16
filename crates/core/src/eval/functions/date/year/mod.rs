use chrono::Datelike;
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{serial_to_date, text_to_date_serial};
use crate::types::{ErrorKind, Value};

pub fn year_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) { return e; }
    let serial = match coerce_to_date_serial(&args[0]) { Ok(n) => n, Err(e) => return e };
    match serial_to_date(serial) {
        Some(d) => Value::Number(d.year() as f64),
        None => Value::Error(ErrorKind::Num),
    }
}

fn coerce_to_date_serial(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Text(s) => text_to_date_serial(s)
            .ok_or(Value::Error(ErrorKind::Value)),
        other => to_number(other.clone()),
    }
}

#[cfg(test)]
mod tests;
