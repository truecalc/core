use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `CHAR(n)` — returns the character for the given Windows-1252 / ASCII code point.
pub fn char_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let n_i = n.trunc() as i64;
    if n_i < 1 {
        return Value::Error(ErrorKind::Num);
    }
    match char::from_u32(n_i as u32) {
        Some(c) => Value::Text(c.to_string()),
        None    => Value::Error(ErrorKind::Num),
    }
}

/// `UNICHAR(n)` — returns the Unicode character for the given code point.
///
/// GS: n <= 0 → #VALUE! (type/domain); invalid code point → #VALUE!.
pub fn unichar_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let n_i = n.trunc() as i64;
    if n_i < 1 {
        return Value::Error(ErrorKind::Value);
    }
    match char::from_u32(n_i as u32) {
        Some(c) => Value::Text(c.to_string()),
        None    => Value::Error(ErrorKind::Value),
    }
}
