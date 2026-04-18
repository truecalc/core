use super::super::*;
use crate::types::Value;

#[test]
fn all_f_is_minus_one() {
    assert_eq!(hex2dec_fn(&[Value::Text("FFFFFFFFFF".to_string())]), Value::Number(-1.0));
}

#[test]
fn a5_is_165() {
    assert_eq!(hex2dec_fn(&[Value::Text("A5".to_string())]), Value::Number(165.0));
}

#[test]
fn one_is_one() {
    assert_eq!(hex2dec_fn(&[Value::Text("1".to_string())]), Value::Number(1.0));
}

#[test]
fn zero_is_zero() {
    assert_eq!(hex2dec_fn(&[Value::Text("0".to_string())]), Value::Number(0.0));
}

#[test]
fn lowercase_accepted() {
    assert_eq!(hex2dec_fn(&[Value::Text("a5".to_string())]), Value::Number(165.0));
}

#[test]
fn max_positive() {
    assert_eq!(hex2dec_fn(&[Value::Text("7FFFFFFFFF".to_string())]), Value::Number(549_755_813_887.0));
}
