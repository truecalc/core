use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `EXACT(text1, text2)` — returns TRUE if the two strings are identical (case-sensitive).
pub fn exact_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let a = match to_string_val(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let b = match to_string_val(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Bool(a == b)
}
