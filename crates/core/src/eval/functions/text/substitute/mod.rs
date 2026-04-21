use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SUBSTITUTE(text, old_text, new_text, [instance_num])` — replaces occurrences of old_text with new_text.
/// Optional instance_num targets a specific occurrence (1-based).
pub fn substitute_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let old_text = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let new_text = match to_string_val(args[2].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let instance_num = if args.len() == 4 {
        match to_number(args[3].clone()) {
            Ok(n) => Some(n as usize),
            Err(e) => return e,
        }
    } else {
        None
    };

    if old_text.is_empty() {
        return Value::Text(text);
    }

    match instance_num {
        None => Value::Text(text.replace(&old_text, &new_text)),
        Some(target) => {
            if target == 0 {
                return Value::Error(ErrorKind::Value);
            }
            // Count overlapping occurrences: advance one char at a time.
            let chars: Vec<char> = text.chars().collect();
            let old_chars: Vec<char> = old_text.chars().collect();
            let new_chars: Vec<char> = new_text.chars().collect();
            let old_len = old_chars.len();
            let mut result: Vec<char> = Vec::new();
            let mut i = 0usize;
            let mut count = 0usize;
            while i <= chars.len().saturating_sub(old_len) {
                if chars[i..i + old_len] == old_chars[..] {
                    count += 1;
                    if count == target {
                        result.extend_from_slice(&new_chars);
                        i += old_len;
                        // Append the rest unchanged
                        result.extend_from_slice(&chars[i..]);
                        return Value::Text(result.into_iter().collect());
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            // Target not reached — append remaining chars and return unchanged
            result.extend_from_slice(&chars[i..]);
            Value::Text(result.into_iter().collect())
        }
    }
}

#[cfg(test)]
mod tests;
