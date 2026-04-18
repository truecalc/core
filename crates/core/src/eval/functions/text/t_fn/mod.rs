use crate::eval::functions::check_arity;
use crate::types::Value;

/// `T(value)` — if `value` is text, returns it unchanged; otherwise returns an empty string.
pub fn t_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Text(s) => Value::Text(s.clone()),
        Value::Error(_) => args[0].clone(),
        _ => Value::Text(String::new()),
    }
}
