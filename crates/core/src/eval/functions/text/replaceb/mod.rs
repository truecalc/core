use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::dbcs_char_width;
use crate::types::{ErrorKind, Value};

/// Convert a 1-based DBCS start byte to a char index, snapping forward if inside a char.
fn dbcs_start_to_char_idx(s: &str, start_1based: usize) -> usize {
    if start_1based <= 1 {
        return 0;
    }
    let target = start_1based - 1; // 0-based
    let mut pos = 0usize;
    for (i, c) in s.chars().enumerate() {
        let w = dbcs_char_width(c);
        if pos >= target {
            return i;
        }
        if pos < target && pos + w > target {
            return i + 1; // snap forward past partial char
        }
        pos += w;
    }
    s.chars().count()
}

/// Convert a char index to a 1-based DBCS byte position.
fn char_idx_to_dbcs_byte_end(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(dbcs_char_width).sum::<usize>()
}

/// `REPLACEB(old_text, start_byte, num_bytes, new_text)` — DBCS byte-position replacement.
pub fn replaceb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 4) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start_num = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let num_bytes = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let new_text = match to_string_val(args[3].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if start_num < 1.0 || num_bytes < 0.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start_dbcs = start_num as usize; // 1-based
    let num = num_bytes as usize;
    let chars: Vec<char> = text.chars().collect();
    // Snap start forward to char boundary
    let start_char = dbcs_start_to_char_idx(&text, start_dbcs);
    // end DBCS byte = (snapped start DBCS byte) + num_bytes
    let start_dbcs_snapped = char_idx_to_dbcs_byte_end(&text, start_char) + 1; // 1-based
    let end_dbcs_target = start_dbcs_snapped.saturating_sub(1) + num; // 0-based end exclusive
    // Find end char: walk from start_char taking num_bytes
    let mut taken = 0usize;
    let mut end_char = start_char;
    for c in chars[start_char..].iter() {
        let w = dbcs_char_width(*c);
        if taken + w <= num {
            taken += w;
            end_char += 1;
            if taken == num { break; }
        } else {
            // Partial char: skip (exclude from replacement range)
            break;
        }
    }
    // Build result: prefix + new_text + suffix
    let prefix: String = chars[..start_char].iter().collect();
    let suffix: String = chars[end_char..].iter().collect();
    let _ = end_dbcs_target; // used for clarity only
    Value::Text(format!("{}{}{}", prefix, new_text, suffix))
}

#[cfg(test)]
mod tests;
