use super::super::*;
use crate::types::Value;

#[test]
fn zero_to_text() {
    let args = [Value::Number(0.0)];
    assert_eq!(to_text_fn(&args), Value::Text("0".to_string()));
}

#[test]
fn negative_to_text() {
    let args = [Value::Number(-5.0)];
    assert_eq!(to_text_fn(&args), Value::Text("-5".to_string()));
}

#[test]
fn date_serial_to_text() {
    let args = [Value::Date(45292.0)];
    assert_eq!(to_text_fn(&args), Value::Text("45292".to_string()));
}

#[test]
fn very_large_number_to_text_scientific() {
    // Google Sheets switches to scientific notation for abs(n) >= 1e15.
    // TO_TEXT(1E15) = "1E+15" (upper-case E, explicit + sign on exponent).
    let args = [Value::Number(1e15_f64)];
    assert_eq!(to_text_fn(&args), Value::Text("1E+15".to_string()));
}
