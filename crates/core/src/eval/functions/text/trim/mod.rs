use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `TRIM(text)` — removes leading/trailing whitespace and collapses internal spaces.
pub fn trim_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let result = text.split_whitespace().collect::<Vec<_>>().join(" ");
    Value::Text(result)
}

#[cfg(test)]
mod tests;
