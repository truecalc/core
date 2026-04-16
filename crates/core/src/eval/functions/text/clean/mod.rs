use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `CLEAN(text)` — removes all non-printable characters (code points < 32) from text.
pub fn clean_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let cleaned: String = text.chars().filter(|&c| (c as u32) >= 32).collect();
    Value::Text(cleaned)
}

#[cfg(test)]
mod tests;
