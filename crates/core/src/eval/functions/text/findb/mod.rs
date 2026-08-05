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
            // target is inside this char: snap to next char
            return i + 1;
        }
        pos += w;
    }
    s.chars().count()
}

/// Convert a char index to a 1-based DBCS byte position.
fn char_idx_to_dbcs_byte(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(dbcs_char_width).sum::<usize>() + 1
}

/// `FINDB(find_text, within_text, [start_num])` — case-sensitive DBCS byte-position search.
pub fn findb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let find_text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let within_text = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start_num = if args.len() == 3 {
        match to_number(args[2].clone()) {
            Ok(n) => n,
            Err(e) => return e,
        }
    } else {
        1.0
    };
    if start_num < 1.0 {
        return Value::Error(ErrorKind::Value);
    }
    let start_dbcs = start_num as usize; // 1-based DBCS byte
    let chars: Vec<char> = within_text.chars().collect();
    let char_start = dbcs_start_to_char_idx(&within_text, start_dbcs);
    if char_start > chars.len() {
        return Value::Error(ErrorKind::Value);
    }
    if find_text.is_empty() {
        // Empty search: return the snapped DBCS position
        let snapped_byte = char_idx_to_dbcs_byte(&within_text, char_start);
        return Value::Number(snapped_byte as f64);
    }
    // Search the substring from char_start
    let substr: String = chars[char_start..].iter().collect();
    match substr.find(find_text.as_str()) {
        Some(utf8_pos) => {
            // Count chars in the matched prefix
            let prefix_chars = substr[..utf8_pos].chars().count();
            let match_char_idx = char_start + prefix_chars;
            let match_dbcs = char_idx_to_dbcs_byte(&within_text, match_char_idx);
            Value::Number(match_dbcs as f64)
        }
        None => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
