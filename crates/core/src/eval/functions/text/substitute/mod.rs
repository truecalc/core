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
            let mut result = String::new();
            let mut remaining = text.as_str();
            let mut count = 0usize;
            while let Some(pos) = remaining.find(&old_text) {
                count += 1;
                result.push_str(&remaining[..pos]);
                if count == target {
                    result.push_str(&new_text);
                } else {
                    result.push_str(&old_text);
                }
                remaining = &remaining[pos + old_text.len()..];
            }
            result.push_str(remaining);
            Value::Text(result)
        }
    }
}

#[cfg(test)]
mod tests;
