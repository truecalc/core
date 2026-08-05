use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `BASE(value, base, [min_length])` — converts a non-negative integer to a string in the given
/// base (2–36), optionally padded with leading zeros to `min_length`.
pub fn base_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let n_int = n.trunc() as u64;

    let base = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let base_int = base.trunc() as i64;
    if !(2..=36).contains(&base_int) {
        return Value::Error(ErrorKind::Num);
    }
    let base_u64 = base_int as u64;

    let min_len = if args.len() == 3 {
        match to_number(args[2].clone()) {
            Err(e) => return e,
            Ok(v) => v.trunc() as usize,
        }
    } else {
        0
    };

    let digits: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result = if n_int == 0 {
        "0".to_string()
    } else {
        let mut val = n_int;
        let mut chars = Vec::new();
        while val > 0 {
            chars.push(digits[(val % base_u64) as usize] as char);
            val /= base_u64;
        }
        chars.reverse();
        chars.into_iter().collect()
    };

    if result.len() < min_len {
        let padding = min_len - result.len();
        result = "0".repeat(padding) + &result;
    }

    Value::Text(result)
}

#[cfg(test)]
mod tests;
