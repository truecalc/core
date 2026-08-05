use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::eval::functions::engineering::{format_hex, get_places, parse_oct};
use crate::types::{ErrorKind, Value};

pub fn oct2hex_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let places = match get_places(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let s = match to_string_val(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let n = match parse_oct(&s) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Num),
    };
    match format_hex(n, places) {
        Ok(result) => Value::Text(result),
        Err(()) => Value::Error(ErrorKind::Num),
    }
}

#[cfg(test)]
mod tests;
