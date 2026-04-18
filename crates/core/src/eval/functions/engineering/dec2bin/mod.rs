use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::engineering::{format_bin, get_places};
use crate::types::{ErrorKind, Value};

pub fn dec2bin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let places = match get_places(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let n = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let n = n.trunc() as i64;
    if !(-512..=511).contains(&n) {
        return Value::Error(ErrorKind::Num);
    }
    match format_bin(n, places) {
        Ok(result) => Value::Text(result),
        Err(()) => Value::Error(ErrorKind::Num),
    }
}

#[cfg(test)]
mod tests;
