use super::super::*;
use crate::types::Value;

#[test]
fn all_sevens_is_minus_one() {
    assert_eq!(oct2dec_fn(&[Value::Text("7777777777".to_string())]), Value::Number(-1.0));
}

#[test]
fn seven_is_seven() {
    assert_eq!(oct2dec_fn(&[Value::Text("7".to_string())]), Value::Number(7.0));
}

#[test]
fn zero_is_zero() {
    assert_eq!(oct2dec_fn(&[Value::Text("0".to_string())]), Value::Number(0.0));
}

#[test]
fn ten_octal_is_eight() {
    assert_eq!(oct2dec_fn(&[Value::Text("10".to_string())]), Value::Number(8.0));
}

#[test]
fn max_positive() {
    assert_eq!(oct2dec_fn(&[Value::Text("3777777777".to_string())]), Value::Number(536_870_911.0));
}

#[test]
fn min_negative() {
    assert_eq!(oct2dec_fn(&[Value::Text("4000000000".to_string())]), Value::Number(-536_870_912.0));
}
