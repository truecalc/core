use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// Flatten a value into a list of strings, recursively expanding arrays.
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

/// `JOIN(delimiter, value1, value2, ...)` — joins values with a delimiter.
pub fn join_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 255) {
        return err;
    }
    let delimiter = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let mut parts: Vec<String> = Vec::new();
    for arg in &args[1..] {
        if let Some(err) = collect_strings(arg, &mut parts) {
            return err;
        }
    }
    Value::Text(parts.join(&delimiter))
}

#[cfg(test)]
mod tests;
