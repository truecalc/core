use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `INDIRECT(ref_text, [a1])` -- converts a string reference to a value.
///
/// This evaluator has no cell grid, so:
/// - A syntactically valid cell reference returns `Value::Empty` (blank cell).
/// - An invalid or empty reference string returns `Error(Ref)`.
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

/// Returns true if `s` looks like a valid A1-style or R1C1-style cell reference.
fn is_valid_ref(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    is_a1_ref(s) || is_r1c1_ref(s)
}

/// A1-style: one or more ASCII letters followed by one or more ASCII digits.
/// Examples: A1, B2, AA100, ZZ9999
fn is_a1_ref(s: &str) -> bool {
    let bytes = s.as_bytes();
    let col_end = bytes.iter().take_while(|b| b.is_ascii_alphabetic()).count();
    if col_end == 0 {
        return false;
    }
    let rest = &bytes[col_end..];
    !rest.is_empty() && rest.iter().all(|b| b.is_ascii_digit())
}

/// R1C1-style: R followed by digits, C followed by digits.
/// Examples: R1C1, R2C3, R10C100
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
    let rest = &rest[row_end..];
    if rest.first().map(|b| b.to_ascii_uppercase()) != Some(b'C') {
        return false;
    }
    let rest = &rest[1..];
    !rest.is_empty() && rest.iter().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests;
