use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::dbcs_char_width;
use crate::types::{ErrorKind, Value};

fn wildcard_match(pattern: &[char], text: &[char]) -> bool {
    match pattern.first() {
        None => true,
        Some('*') => {
            for i in 0..=text.len() {
                if wildcard_match(&pattern[1..], &text[i..]) {
                    return true;
                }
            }
            false
        }
        Some(_) => match text.first() {
            None => false,
            Some(t) => {
                let p = &pattern[0];
                if *p == '?' || p.to_lowercase().next() == t.to_lowercase().next() {
                    wildcard_match(&pattern[1..], &text[1..])
                } else {
                    false
                }
            }
        },
    }
}

fn wildcard_find(pattern: &[char], text: &[char], start_idx: usize) -> Option<usize> {
    if pattern.is_empty() {
        return if start_idx <= text.len() { Some(start_idx) } else { None };
    }
    for i in start_idx..=text.len() {
        if wildcard_match(pattern, &text[i..]) {
            return Some(i);
        }
    }
    None
}

/// Convert a 1-based DBCS start byte to a char index, snapping forward if inside a char.
fn dbcs_start_to_char_idx(s: &str, start_1based: usize) -> usize {
    if start_1based <= 1 {
        return 0;
    }
    let target = start_1based - 1;
    let mut pos = 0usize;
    for (i, c) in s.chars().enumerate() {
        let w = dbcs_char_width(c);
        if pos >= target {
            return i;
        }
        if pos < target && pos + w > target {
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

/// `SEARCHB(find_text, within_text, [start_num])` — case-insensitive DBCS byte-position search
/// with wildcard support (`?` = any char, `*` = any sequence, `~` escapes).
pub fn searchb_fn(args: &[Value]) -> Value {
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
    let within_chars: Vec<char> = within_text.chars().collect();
    let char_start = dbcs_start_to_char_idx(&within_text, start_dbcs);
    if char_start > within_chars.len() {
        return Value::Error(ErrorKind::Value);
    }
    // Unescape tilde sequences in pattern
    let pattern_raw: Vec<char> = find_text.chars().collect();
    let mut pattern: Vec<char> = Vec::with_capacity(pattern_raw.len());
    let mut i = 0;
    while i < pattern_raw.len() {
        if pattern_raw[i] == '~' && i + 1 < pattern_raw.len() {
            match pattern_raw[i + 1] {
                '?' | '*' | '~' => {
                    pattern.push(pattern_raw[i + 1]);
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        pattern.push(pattern_raw[i]);
        i += 1;
    }
    match wildcard_find(&pattern, &within_chars, char_start) {
        Some(char_idx) => {
            let dbcs_pos = char_idx_to_dbcs_byte(&within_text, char_idx);
            Value::Number(dbcs_pos as f64)
        }
        None => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
