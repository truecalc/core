use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `CONCATENATE(text1, ...)` — joins all arguments as strings.
pub fn concatenate_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut result = String::new();
    for arg in args {
        match to_string_val(arg.clone()) {
            Ok(s) => result.push_str(&s),
            Err(e) => return e,
        }
    }
    Value::Text(result)
}

#[cfg(test)]
mod tests;
