use crate::eval::coercion::{to_bool, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `SPLIT(text, delimiter, [split_by_each=TRUE], [remove_empty_text=TRUE])` —
/// splits text by delimiter, returns an array.
///
/// - `split_by_each` (arg 3): when TRUE (default), splits on each character of the
///   delimiter string individually. When FALSE, treats the whole delimiter as a unit.
/// - `remove_empty_text` (arg 4): when TRUE (default), removes empty-string elements
///   from the result. When FALSE, keeps them.
pub fn split_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 4) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let delimiter = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let split_by_each = if args.len() >= 3 {
        match to_bool(args[2].clone()) {
            Ok(b) => b,
            Err(e) => return e,
        }
    } else {
        true
    };
    let remove_empty = if args.len() >= 4 {
        match to_bool(args[3].clone()) {
            Ok(b) => b,
            Err(e) => return e,
        }
    } else {
        true
    };

    let raw_parts: Vec<&str> = if split_by_each {
        // Split on any character in the delimiter string
        text.split(|c: char| delimiter.contains(c)).collect()
    } else {
        text.split(delimiter.as_str()).collect()
    };

    let parts: Vec<Value> = raw_parts
        .into_iter()
        .filter(|s| !remove_empty || !s.is_empty())
        .map(|s| {
            if s.is_empty() {
                Value::Empty
            } else {
                Value::Text(s.to_string())
            }
        })
        .collect();

    Value::Array(parts)
}

#[cfg(test)]
mod tests;
