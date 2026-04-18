use super::super::*;
use crate::types::Value;

#[test]
fn all_ones_is_minus_one() {
    assert_eq!(bin2dec_fn(&[Value::Text("1111111111".to_string())]), Value::Number(-1.0));
}

#[test]
fn sign_bit_only_is_minus_512() {
    assert_eq!(bin2dec_fn(&[Value::Text("1000000000".to_string())]), Value::Number(-512.0));
}

#[test]
fn one_is_one() {
    assert_eq!(bin2dec_fn(&[Value::Text("1".to_string())]), Value::Number(1.0));
}

#[test]
fn zero_padded_one() {
    assert_eq!(bin2dec_fn(&[Value::Text("0000000001".to_string())]), Value::Number(1.0));
}

#[test]
fn max_positive() {
    assert_eq!(bin2dec_fn(&[Value::Text("111111111".to_string())]), Value::Number(511.0));
}

#[test]
fn zero() {
    assert_eq!(bin2dec_fn(&[Value::Text("0".to_string())]), Value::Number(0.0));
}

#[test]
fn numeric_input_coerced() {
    assert_eq!(bin2dec_fn(&[Value::Number(1.0)]), Value::Number(1.0));
}
