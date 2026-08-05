use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::eval::functions::engineering::parse_hex;
use crate::types::{ErrorKind, Value};

pub fn hex2dec_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let s = match to_string_val(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match parse_hex(&s) {
        Some(n) => Value::Number(n as f64),
        None => Value::Error(ErrorKind::Num),
    }
}

#[cfg(test)]
mod tests;
