use crate::eval::coercion::to_bool;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `NOT(value)` — inverts a boolean value.
///
/// Accepts exactly 1 argument. GS special case: empty text `""` is treated
/// as 0 (false), so `NOT("")` = TRUE. Other non-bool text remains `#VALUE!`.
pub fn not_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    // GS: NOT("") = TRUE (empty text treated as false/0)
    let coerced = match &args[0] {
        Value::Text(s) if s.is_empty() => Ok(false),
        _ => to_bool(args[0].clone()),
    };
    match coerced {
        Ok(b) => Value::Bool(!b),
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests;
