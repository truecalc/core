use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::time_to_serial;
use crate::types::{ErrorKind, Value};

/// `TIME(hour, minute, second)` — returns a fractional day serial for the given time.
///
/// Components are truncated to integers. Overflow wraps (e.g. 25 hours → 1 hour).
/// Returns `Value::Error(ErrorKind::Num)` if any component is negative after truncation.
pub fn time_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let hour   = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let minute = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let second = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };

    // Truncate to integers
    let h = hour.trunc() as i64;
    let m = minute.trunc() as i64;
    let s = second.trunc() as i64;

    // Negative components are an error
    if h < 0 || m < 0 || s < 0 {
        return Value::Error(ErrorKind::Num);
    }

    // Convert to total seconds, then take mod 86400 to wrap
    let total_seconds = h * 3600 + m * 60 + s;
    let wrapped = total_seconds % 86400;
    let serial = time_to_serial(
        (wrapped / 3600) as u32,
        ((wrapped % 3600) / 60) as u32,
        (wrapped % 60) as u32,
    );

    Value::Number(serial)
}

#[cfg(test)]
mod tests;
