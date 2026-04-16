use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

const MAX_BIT: u64 = 281_474_976_710_655; // 2^48 - 1

pub fn bitand_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let a = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let b = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if a < 0.0 || b < 0.0 || a > MAX_BIT as f64 || b > MAX_BIT as f64 {
        return Value::Error(ErrorKind::Num);
    }
    let result = (a as u64) & (b as u64);
    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
