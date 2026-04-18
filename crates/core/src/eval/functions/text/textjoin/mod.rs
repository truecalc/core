use crate::eval::coercion::{to_bool, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::Value;

/// Flatten a value into string parts, recursively expanding arrays.
fn collect_strings(v: &Value, out: &mut Vec<String>) -> Option<Value> {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                if let Some(err) = collect_strings(elem, out) {
                    return Some(err);
                }
            }
        }
        other => match to_string_val(other.clone()) {
            Ok(s) => out.push(s),
            Err(e) => return Some(e),
        },
    }
    None
}

/// `TEXTJOIN(delimiter, ignore_empty, value1, value2, ...)` — joins values with
/// a delimiter, optionally ignoring empty strings.
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
        if let Some(err) = collect_strings(arg, &mut parts) {
            return err;
        }
    }
    if ignore_empty {
        parts.retain(|s| !s.is_empty());
    }
    Value::Text(parts.join(&delimiter))
}

#[cfg(test)]
mod tests;
