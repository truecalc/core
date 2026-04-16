use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::engineering::{format_oct, get_places};
use crate::types::{ErrorKind, Value};

const MIN_DEC: i64 = -536_870_912; // -2^29
const MAX_DEC: i64 =  536_870_911; //  2^29 - 1

pub fn dec2oct_fn(args: &[Value]) -> Value {
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
    if !(MIN_DEC..=MAX_DEC).contains(&n) {
        return Value::Error(ErrorKind::Num);
    }
    match format_oct(n, places) {
        Ok(result) => Value::Text(result),
        Err(()) => Value::Error(ErrorKind::Num),
    }
}

#[cfg(test)]
mod tests;
