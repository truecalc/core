use crate::eval::coercion::{to_bool, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

const MAX_LEN: usize = 32767;

fn collect_strings(v: &Value, ignore_empty: bool, out: &mut Vec<String>) -> Option<Value> {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                if let Some(err) = collect_strings(elem, ignore_empty, out) {
                    return Some(err);
                }
            }
        }
        other => match to_string_val(other.clone()) {
            Ok(s) => {
                if !ignore_empty || !s.is_empty() {
                    out.push(s);
                }
            }
            Err(e) => return Some(e),
        },
    }
    None
}

/// `TEXTJOIN(delimiter, ignore_empty, value1, ...)` -- returns `#VALUE!` if result > 32767 chars.
pub fn textjoin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 255) {
        return err;
    }
    let delimiter = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let ignore_empty = match to_bool(args[1].clone()) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let mut parts: Vec<String> = Vec::new();
    for arg in &args[2..] {
        if let Some(err) = collect_strings(arg, ignore_empty, &mut parts) {
            return err;
        }
    }
    let result = parts.join(&delimiter);
    if result.chars().count() > MAX_LEN {
        return Value::Error(ErrorKind::Value);
    }
    Value::Text(result)
}

#[cfg(test)]
mod tests;
