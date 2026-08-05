use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `LEFT(text, [num_chars])` — returns the first N characters of a string.
/// Default N=1. Returns `#VALUE!` if N < 0.
pub fn left_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let n = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Ok(n) => n,
            Err(e) => return e,
        }
    } else {
        1.0
    };
    if n < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let n = n as usize;
    let result: String = text.chars().take(n).collect();
    Value::Text(result)
}

#[cfg(test)]
mod tests;
