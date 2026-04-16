use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `LENB(text)` — returns the number of bytes in a string.
/// For ASCII text this is identical to LEN.
pub fn lenb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    Value::Number(text.len() as f64)
}

#[cfg(test)]
mod tests;
