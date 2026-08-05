use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Google Sheets limits: 18,278 columns (≈ column ZZZ) and 10,000,000 rows.
pub(crate) const MAX_COL: usize = 18_278;
pub(crate) const MAX_ROW: usize = 10_000_000;

/// `INDIRECT(ref_text, [a1])` -- converts a string reference to a value.
///
/// This evaluator has no cell grid, so:
/// - A syntactically valid, in-range cell/range reference returns `Value::Empty`.
/// - An invalid, empty, or out-of-range reference returns `Error(Ref)`.
/// - Wrong arity returns `Error(NA)`.
pub fn indirect_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let ref_text = match &args[0] {
        Value::Text(s) => s.clone(),
        _ => return Value::Error(ErrorKind::Ref),
    };
    if is_valid_ref(&ref_text) {
        Value::Empty
    } else {
        Value::Error(ErrorKind::Ref)
    }
}

/// Returns true if `s` is a valid, in-range A1-style or R1C1-style reference.
pub fn is_valid_ref(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    is_a1_ref(s) || is_a1_range(s) || is_r1c1_ref(s)
}

/// Parse a column label (e.g. "A" → 1, "ZZZ" → 18278) — 1-based.
fn col_label_to_index(label: &str) -> usize {
    let mut result: usize = 0;
    for c in label.chars() {
        result = result * 26 + (c.to_ascii_uppercase() as usize - b'A' as usize + 1);
    }
    result
}

/// A1-style single cell: one or more ASCII letters + one or more ASCII digits,
/// with column ≤ MAX_COL and row ≤ MAX_ROW.
fn is_a1_ref(s: &str) -> bool {
    let bytes = s.as_bytes();
    let col_end = bytes.iter().take_while(|b| b.is_ascii_alphabetic()).count();
    if col_end == 0 || col_end == bytes.len() {
        return false;
    }
    let rest = &s[col_end..];
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let col = col_label_to_index(&s[..col_end]);
    let row: usize = rest.parse().unwrap_or(0);
    (1..=MAX_COL).contains(&col) && (1..=MAX_ROW).contains(&row)
}

/// A1-style range: "A1:B2" — both halves must be valid single A1 cell refs.
fn is_a1_range(s: &str) -> bool {
    if let Some(colon) = s.find(':') {
        is_a1_ref(&s[..colon]) && is_a1_ref(&s[colon + 1..])
    } else {
        false
    }
}

/// R1C1-style: R followed by digits, C followed by digits, both in range.
fn is_r1c1_ref(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.first().map(|b| b.to_ascii_uppercase()) != Some(b'R') {
        return false;
    }
    let rest = &bytes[1..];
    let row_end = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    if row_end == 0 {
        return false;
    }
    let row: usize = std::str::from_utf8(&rest[..row_end]).unwrap().parse().unwrap_or(0);
    let rest = &rest[row_end..];
    if rest.first().map(|b| b.to_ascii_uppercase()) != Some(b'C') {
        return false;
    }
    let rest = &rest[1..];
    if rest.is_empty() || !rest.iter().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let col: usize = std::str::from_utf8(rest).unwrap().parse().unwrap_or(0);
    (1..=MAX_ROW).contains(&row) && (1..=MAX_COL).contains(&col)
}

#[cfg(test)]
mod tests;
