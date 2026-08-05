use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{serial_to_time, text_to_time_serial};
use crate::types::{ErrorKind, Value};

pub fn second_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) { return e; }
    let serial = match coerce_to_time_serial(&args[0]) { Ok(n) => n, Err(e) => return e };
    Value::Number(serial_to_time(serial).2 as f64)
}

fn coerce_to_time_serial(v: &Value) -> Result<f64, Value> {
    match v {
        Value::Text(s) => text_to_time_serial(s)
            .ok_or(Value::Error(ErrorKind::Value)),
        other => to_number(other.clone()),
    }
}

#[cfg(test)]
mod tests;
