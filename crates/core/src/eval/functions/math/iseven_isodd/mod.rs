use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `ISEVEN(number)` — returns TRUE if number (truncated to integer) is even.
pub fn iseven_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Bool((n.trunc() as i64) % 2 == 0)
}

/// `ISODD(number)` — returns TRUE if number (truncated to integer) is odd.
pub fn isodd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Bool((n.trunc() as i64) % 2 != 0)
}

#[cfg(test)]
mod tests;
