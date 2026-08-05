use super::super::*;
use crate::types::Value;

#[test]
fn negative_number_integer_format() {
    assert_eq!(
        text_fn(&[Value::Number(-5.7), Value::Text("0".to_string())]),
        Value::Text("-6".to_string())
    );
}

#[test]
fn zero_integer_format() {
    assert_eq!(
        text_fn(&[Value::Number(0.0), Value::Text("0".to_string())]),
        Value::Text("0".to_string())
    );
}

#[test]
fn bool_not_coerced_returns_text() {
    // GS: TEXT(TRUE, ...) returns "TRUE" — boolean is NOT coerced to a number
    assert_eq!(
        text_fn(&[Value::Bool(true), Value::Text("0".to_string())]),
        Value::Text("TRUE".to_string())
    );
}

#[test]
fn unsupported_format_falls_back_to_display_number() {
    // "General" is not a recognised pattern; should fall back to display_number
    assert_eq!(
        text_fn(&[Value::Number(3.5), Value::Text("General".to_string())]),
        Value::Text("3.5".to_string())
    );
}

#[test]
fn empty_format_renders_nothing() {
    // GS: an empty number format is the "hidden" format — it renders nothing,
    // whatever the value is. The rule is about the format being empty, not
    // about it lacking digit tokens (see
    // `unsupported_format_falls_back_to_display_number` above).
    for value in [
        Value::Number(1234.0),
        Value::Number(0.0),
        Value::Number(-12.5),
        Value::Text("1234".to_string()),
        Value::Date(25404.0),
    ] {
        assert_eq!(
            text_fn(&[value.clone(), Value::Text(String::new())]),
            Value::Text(String::new()),
            "empty format should render nothing for {value:?}"
        );
    }
}

#[test]
fn percent_only_format_renders_just_the_percent_sign() {
    // Recorded: `=TEXT(0.285,"%")` and `=TEXT(1234,"%")` are both "%". The
    // percent branch strips the sign and recurses with an empty remainder, and
    // an empty format renders nothing — so no digits survive. This falls out of
    // the rule living in `apply_format`; it is not special-cased.
    assert_eq!(
        text_fn(&[Value::Number(0.285), Value::Text("%".to_string())]),
        Value::Text("%".to_string())
    );
    assert_eq!(
        text_fn(&[Value::Number(1234.0), Value::Text("%".to_string())]),
        Value::Text("%".to_string())
    );
}

#[test]
fn boolean_ignores_the_format_entirely() {
    // Recorded: `=TEXT(TRUE,"")` is "TRUE", not "". Booleans are answered
    // before the format is read, and the empty-format rule must not reach them
    // — #789's first acceptance criterion asserted the opposite, and the
    // pipeline says otherwise.
    assert_eq!(
        text_fn(&[Value::Bool(true), Value::Text(String::new())]),
        Value::Text("TRUE".to_string())
    );
    assert_eq!(
        text_fn(&[Value::Bool(false), Value::Text(String::new())]),
        Value::Text("FALSE".to_string())
    );
}

#[test]
fn empty_format_does_not_swallow_argument_errors() {
    // Arg-0 resolution happens before the format is read, so an error value
    // still propagates rather than being rendered as the empty string.
    assert_eq!(
        text_fn(&[
            Value::Error(crate::types::ErrorKind::NA),
            Value::Text(String::new())
        ]),
        Value::Error(crate::types::ErrorKind::NA)
    );
}

#[test]
fn hash_format_suppresses_trailing_zeros() {
    // "0.##" — # suppresses trailing zeros; 1.5 -> "1.5" (not "1.50")
    assert_eq!(
        text_fn(&[Value::Number(1.5), Value::Text("0.##".to_string())]),
        Value::Text("1.5".to_string())
    );
}
