use super::super::*;
use crate::types::Value;

#[test]
fn start_beyond_end() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(6.0), Value::Number(3.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn zero_num_chars() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(2.0), Value::Number(0.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn clamp_beyond_length() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(3.0), Value::Number(100.0)]),
        Value::Text("llo".to_string())
    );
}

// Google Sheets indexes MID by UTF-16 code unit, not Unicode codepoint. An
// astral character (e.g. 😀 = U+1F600) is a surrogate pair, so a `num_chars`
// window that lands entirely inside it still returns 2 UTF-16 units, and a
// window that splits the pair yields one code unit each side. See #848.
#[test]
fn whole_astral_character_takes_two_utf16_units() {
    assert_eq!(
        mid_fn(&[Value::Text("😀X".to_string()), Value::Number(1.0), Value::Number(2.0)]),
        Value::Text("😀".to_string())
    );
}

#[test]
fn splits_surrogate_pair_by_utf16_unit_position() {
    // units: [high surrogate, low surrogate, 'X']
    assert_eq!(
        mid_fn(&[Value::Text("😀X".to_string()), Value::Number(1.0), Value::Number(1.0)]),
        Value::Text("\u{FFFD}".to_string())
    );
    assert_eq!(
        mid_fn(&[Value::Text("😀X".to_string()), Value::Number(2.0), Value::Number(1.0)]),
        Value::Text("\u{FFFD}".to_string())
    );
    assert_eq!(
        mid_fn(&[Value::Text("😀X".to_string()), Value::Number(3.0), Value::Number(1.0)]),
        Value::Text("X".to_string())
    );
}

// Exact example from #848: "👍🏽" is two astral codepoints (base emoji U+1F44D
// + skin-tone modifier U+1F3FD), each a surrogate pair, so it's 4 UTF-16
// units. MID(...,1,2) takes only the first 2 units — the base emoji without
// its modifier — matching Google Sheets' UTF-16-unit slicing.
#[test]
fn splits_emoji_from_its_modifier_across_multi_codepoint_grapheme() {
    assert_eq!(
        mid_fn(&[Value::Text("👍🏽abc".to_string()), Value::Number(1.0), Value::Number(2.0)]),
        Value::Text("👍".to_string())
    );
}
