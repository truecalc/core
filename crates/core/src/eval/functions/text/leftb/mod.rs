use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::dbcs_char_width;
use crate::types::{ErrorKind, Value};

/// `LEFTB(text, [num_bytes=1])` — returns the first N DBCS bytes of text.
/// Partial double-byte characters at the boundary are excluded.
pub fn leftb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let n = if args.len() >= 2 {
        match to_number(args[1].clone()) {
            Ok(v) => v,
            Err(e) => return e,
        }
    } else {
        1.0
    };
    if n < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let budget = n as usize;
    let mut used = 0usize;
    let result: String = text.chars().take_while(|&c| {
        let w = dbcs_char_width(c);
        if used + w <= budget {
            used += w;
            true
        } else {
            false
        }
    }).collect();
    Value::Text(result)
}

#[cfg(test)]
mod tests;
