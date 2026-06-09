use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::{dbcs_len, dbcs_char_width};
use crate::types::{ErrorKind, Value};

/// `RIGHTB(text, [num_bytes=1])` — returns the last N DBCS bytes of text.
/// Partial double-byte characters at the leading boundary are excluded.
pub fn rightb_fn(args: &[Value]) -> Value {
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
    let total = dbcs_len(&text);
    let skip = if budget >= total { 0 } else { total - budget };
    // Walk forward snapping to char boundary if skip lands inside a char.
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0usize;
    let mut char_start = chars.len();
    for (i, &c) in chars.iter().enumerate() {
        let w = dbcs_char_width(c);
        if pos == skip {
            char_start = i;
            break;
        }
        if pos < skip && pos + w > skip {
            // skip falls inside this char: snap forward to next char
            char_start = i + 1;
            break;
        }
        pos += w;
        if pos == skip {
            char_start = i + 1;
            break;
        }
    }
    Value::Text(chars[char_start..].iter().collect())
}

#[cfg(test)]
mod tests;
