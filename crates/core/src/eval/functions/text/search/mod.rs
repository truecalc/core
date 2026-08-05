use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Match `pattern` as a prefix of `text` (prefix match, not full match).
/// `?` matches any single char; `*` matches any sequence of chars.
/// `~?` matches literal `?`, `~*` matches literal `*`, `~~` matches literal `~`.
/// When pattern is exhausted, remaining text is ignored (prefix semantics).
fn wildcard_match(pattern: &[char], text: &[char]) -> bool {
    match pattern.first() {
        // Pattern exhausted — prefix matched successfully
        None => true,
        Some('~') if pattern.len() >= 2 => {
            // Tilde escape: next char is treated as a literal
            match text.first() {
                None => false,
                Some(t) => {
                    let escaped = &pattern[1];
                    if escaped.to_lowercase().next() == t.to_lowercase().next() {
                        wildcard_match(&pattern[2..], &text[1..])
                    } else {
                        false
                    }
                }
            }
        }
        Some('*') => {
            // '*' can consume 0, 1, 2, ... chars from text
            for i in 0..=text.len() {
                if wildcard_match(&pattern[1..], &text[i..]) {
                    return true;
                }
            }
            false
        }
        Some(_) => match text.first() {
            // Pattern has chars left but text is empty — no match
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

/// `SEARCH(find_text, within_text, [start_num])` — case-insensitive search with wildcards.
/// `?` matches any single character; `*` matches any sequence.
/// Returns 1-based position or `#VALUE!` if not found.
pub fn search_fn(args: &[Value]) -> Value {
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
    let start_idx = (start_num as usize).saturating_sub(1);
    let within_chars: Vec<char> = within_text.chars().collect();
    if start_idx > within_chars.len() {
        return Value::Error(ErrorKind::Value);
    }
    let pattern_chars: Vec<char> = find_text.chars().collect();
    match wildcard_find(&pattern_chars, &within_chars, start_idx) {
        Some(pos) => Value::Number((pos + 1) as f64),
        None => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
