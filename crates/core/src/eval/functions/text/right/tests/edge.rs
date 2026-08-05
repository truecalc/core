use super::super::*;
use crate::types::Value;

#[test]
fn zero_chars() {
    assert_eq!(
        right_fn(&[Value::Text("Hello".to_string()), Value::Number(0.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn clamp_beyond_length() {
    assert_eq!(
        right_fn(&[Value::Text("Hello".to_string()), Value::Number(100.0)]),
        Value::Text("Hello".to_string())
    );
}

#[test]
fn empty_string() {
    assert_eq!(
        right_fn(&[Value::Text("".to_string()), Value::Number(3.0)]),
        Value::Text("".to_string())
    );
}

// Google Sheets indexes RIGHT by UTF-16 code unit, not Unicode codepoint. See
// #848: RIGHT of 2 units on an astral character (e.g. 😀 = surrogate pair)
// returns the whole character; RIGHT of 1 unit splits the surrogate pair.
#[test]
fn whole_astral_character_takes_two_utf16_units() {
    assert_eq!(
        right_fn(&[Value::Text("X😀".to_string()), Value::Number(2.0)]),
        Value::Text("😀".to_string())
    );
}

#[test]
fn splits_surrogate_pair_by_utf16_unit_position() {
    assert_eq!(
        right_fn(&[Value::Text("X😀".to_string()), Value::Number(1.0)]),
        Value::Text("\u{FFFD}".to_string())
    );
}
