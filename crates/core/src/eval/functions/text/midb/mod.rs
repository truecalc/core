use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::dbcs_char_width;
use crate::types::{ErrorKind, Value};

/// `MIDB(text, start_byte, num_bytes)` — returns a substring by DBCS byte position.
/// start_byte is 1-based. Partial double-byte characters are excluded.
/// If start_byte falls inside a double-byte char, returns empty string.
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
    let start_dbcs = (start as usize) - 1; // 0-based DBCS byte
    let budget = num_bytes as usize;
    let mut pos = 0usize;
    let mut result = String::new();
    let mut bytes_taken = 0usize;
    let mut collecting = false;
    for c in text.chars() {
        let w = dbcs_char_width(c);
        let char_end = pos + w;
        if !collecting {
            if pos == start_dbcs {
                collecting = true;
            } else if pos < start_dbcs && char_end > start_dbcs {
                // start_dbcs is inside this char — return empty (partial char boundary)
                return Value::Text(String::new());
            }
        }
        if collecting {
            if bytes_taken + w <= budget {
                result.push(c);
                bytes_taken += w;
                if bytes_taken == budget {
                    break;
                }
            } else {
                // Partial char at end — skip
                break;
            }
        }
        pos += w;
    }
    Value::Text(result)
}

#[cfg(test)]
mod tests;
