use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::eval::functions::text::lenb::dbcs_char_width;
use crate::types::{ErrorKind, Value};

// Pattern token: (char, is_wildcard). Tilde-escaped chars have is_wildcard=false.
type Pat = (char, bool);

/// Match pattern against the PREFIX of text, returning the number of text chars consumed
/// if successful, or None if no match. '*' consumes 0..N chars greedily.
fn wc_prefix_match(pat: &[Pat], text: &[char]) -> Option<usize> {
    if pat.is_empty() { return Some(0); }
    let (pc, is_wc) = pat[0];
    if is_wc && pc == '*' {
        // '*' can consume 0..text.len() chars; try greedy first (longest match)
        for skip in (0..=text.len()).rev() {
            if let Some(rest) = wc_prefix_match(&pat[1..], &text[skip..]) {
                return Some(skip + rest);
            }
        }
        return None;
    }
    if is_wc { // '?' consumes exactly one char
        if text.is_empty() { return None; }
        return wc_prefix_match(&pat[1..], &text[1..]).map(|n| n + 1);
    }
    // Literal char: case-insensitive compare
    match text.first() {
        None => None,
        Some(t) => {
            if pc.to_lowercase().next() == t.to_lowercase().next() {
                wc_prefix_match(&pat[1..], &text[1..]).map(|n| n + 1)
            } else {
                None
            }
        },
    }
}

/// Find the first char-index >= start where pattern matches as a prefix of text[i..].
fn wc_find(pat: &[Pat], text: &[char], start: usize) -> Option<usize> {
    (start..=text.len()).find(|&i| wc_prefix_match(pat, &text[i..]).is_some())
}

fn dbcs_to_char_idx(s: &str, byte_1based: usize) -> usize {
    if byte_1based <= 1 { return 0; }
    let target = byte_1based - 1;
    let mut pos = 0usize;
    for (i, c) in s.chars().enumerate() {
        let w = dbcs_char_width(c);
        if pos >= target { return i; }
        if pos < target && pos + w > target { return i + 1; }
        pos += w;
    }
    s.chars().count()
}

fn char_to_dbcs_byte(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(dbcs_char_width).sum::<usize>() + 1
}

fn unescape_pattern(raw: &str) -> Vec<Pat> {
    let mut out = Vec::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '~' {
            if let Some(&next) = chars.peek() {
                if next == '?' || next == '*' || next == '~' {
                    chars.next();
                    out.push((next, false));
                    continue;
                }
            }
            out.push((c, false));
        } else if c == '?' || c == '*' {
            out.push((c, true));
        } else {
            out.push((c, false));
        }
    }
    out
}

pub fn searchb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) { return err; }
    let find_text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let within_text = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let start_byte = if args.len() == 3 {
        match to_number(args[2].clone()) {
            Ok(n) => {
                if n < 1.0 { return Value::Error(ErrorKind::Value); }
                n as usize
            },
            Err(e) => return e,
        }
    } else {
        1
    };
    let start_char = dbcs_to_char_idx(&within_text, start_byte);
    let within_chars: Vec<char> = within_text.chars().collect();
    if start_char > within_chars.len() { return Value::Error(ErrorKind::Value); }
    let pat = unescape_pattern(&find_text);
    match wc_find(&pat, &within_chars, start_char) {
        None => Value::Error(ErrorKind::Value),
        Some(char_idx) => {
            let byte_pos = char_to_dbcs_byte(&within_text, char_idx);
            Value::Number(byte_pos as f64)
        },
    }
}
