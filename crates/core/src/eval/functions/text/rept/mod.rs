use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

const MAX_REPT: usize = 32767;

/// `REPT(text, number_times)` -- repeats text N times.
/// Returns `#VALUE!` if N < 0 or result would exceed 32767 characters.
pub fn rept_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) { return err; }
    let text = match to_string_val(args[0].clone()) { Ok(s) => s, Err(e) => return e };
    let n = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    if n < 0.0 { return Value::Error(ErrorKind::Value); }
    let times = n as usize;
    if times == 0 { return Value::Text(String::new()); }
    if text.chars().count() * times > MAX_REPT { return Value::Error(ErrorKind::Value); }
    Value::Text(text.repeat(times))
}

#[cfg(test)]
mod tests;
