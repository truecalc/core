use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

const MAX_BIT: u64 = 281_474_976_710_655; // 2^48 - 1

pub fn bitlshift_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let number = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let shift = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if number < 0.0 || number > MAX_BIT as f64 {
        return Value::Error(ErrorKind::Num);
    }
    let num = number as u64;
    let shift_amount = shift as i64;
    let result = if shift_amount >= 0 {
        let s = shift_amount as u32;
        if s >= 48 {
            return Value::Error(ErrorKind::Num);
        }
        num.checked_shl(s).unwrap_or(u64::MAX)
    } else {
        let s = (-shift_amount) as u32;
        if s >= 64 {
            0
        } else {
            num >> s
        }
    };
    if result > MAX_BIT {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
