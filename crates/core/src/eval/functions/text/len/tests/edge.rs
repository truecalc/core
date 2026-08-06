use super::super::*;
use crate::types::Value;

#[test]
fn bool_coerced() {
    assert_eq!(
        len_fn(&[Value::Bool(true)]),
        Value::Number(4.0) // "TRUE"
    );
}

#[test]
fn empty_value() {
    assert_eq!(
        len_fn(&[Value::Empty]),
        Value::Number(0.0)
    );
}

#[test]
fn spaces_count() {
    assert_eq!(
        len_fn(&[Value::Text("  a  ".to_string())]),
        Value::Number(5.0)
    );
}

// Google Sheets stores text as UTF-16, so LEN counts UTF-16 code units, not
// Unicode codepoints: an astral character (outside the Basic Multilingual
// Plane — most emoji, mathematical alphanumeric symbols, etc.) is encoded as
// a surrogate pair and counts as 2. See issue #848.
#[test]
fn emoji_counts_as_two_utf16_units() {
    assert_eq!(
        len_fn(&[Value::Text("😀".to_string())]),
        Value::Number(2.0)
    );
}

#[test]
fn astral_math_letter_counts_as_two_utf16_units() {
    // U+1D54F MATHEMATICAL DOUBLE-STRUCK CAPITAL X
    assert_eq!(
        len_fn(&[Value::Text("𝕏".to_string())]),
        Value::Number(2.0)
    );
}

#[test]
fn bmp_character_still_counts_as_one() {
    // Sanity check: a codepoint inside the BMP (single UTF-16 unit) is
    // unaffected by the UTF-16 change.
    assert_eq!(
        len_fn(&[Value::Text("熊".to_string())]),
        Value::Number(1.0)
    );
}
