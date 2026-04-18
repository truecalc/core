use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

fn proper_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for c in s.chars() {
        if c.is_alphabetic() {
            if capitalize_next {
                result.extend(c.to_uppercase());
            } else {
                result.extend(c.to_lowercase());
            }
            capitalize_next = false;
        } else {
            result.push(c);
            capitalize_next = true;
        }
    }
    result
}

/// `PROPER(text)` — capitalizes the first letter of each word.
pub fn proper_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    Value::Text(proper_case(&text))
}

#[cfg(test)]
mod tests;
