use crate::eval::coercion::to_bool;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `NOT(value)` — inverts a boolean value.
///
/// Accepts exactly 1 argument. Returns `#VALUE!` if the argument cannot be
/// coerced to bool (e.g. Text, Empty, Array).
pub fn not_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match to_bool(args[0].clone()) {
        Ok(b) => Value::Bool(!b),
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests;
