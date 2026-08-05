use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::dbcs_char_width;
use crate::types::{ErrorKind, Value};

/// `MIDB(text, starting_at, num_bytes)` — returns a substring that starts at a
/// 1-based *character* index and is limited to `num_bytes` DBCS bytes.
///
/// The two arguments are measured in different units, which is what the recorded
/// Google Sheets behaviour requires: `starting_at` counts characters, while the
/// length budget counts DBCS bytes (1 for single-byte, 2 for double-byte). A
/// start past the last character yields an empty string, and a trailing
/// character that does not fit the remaining byte budget is dropped.
pub fn midb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let num_bytes = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if start < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    if num_bytes < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    if num_bytes == 0.0 {
        return Value::Text(String::new());
    }
    let skip_chars = (start as usize) - 1; // 0-based character index
    let budget = num_bytes as usize;
    let mut result = String::new();
    let mut bytes_taken = 0usize;
    // `chars()` (Unicode scalar values) is the same iteration `dbcs_char_width`
    // is defined over, so the index and the byte widths stay in step.
    for c in text.chars().skip(skip_chars) {
        let w = dbcs_char_width(c);
        if bytes_taken + w > budget {
            // Partial char at end — skip
            break;
        }
        result.push(c);
        bytes_taken += w;
        if bytes_taken == budget {
            break;
        }
    }
    Value::Text(result)
}

#[cfg(test)]
mod tests;
