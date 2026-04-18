use super::super::*;
use crate::types::Value;

#[test]
fn iso_format() {
    let args = [Value::Text("2024-01-15".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Number(45306.0));
}

#[test]
fn us_slash_format() {
    let args = [Value::Text("1/15/2024".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Number(45306.0));
}

#[test]
fn long_month_name() {
    let args = [Value::Text("January 15, 2024".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Number(45306.0));
}

#[test]
fn short_month_name() {
    let args = [Value::Text("Jan 15, 2024".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Number(45306.0));
}

#[test]
fn zero_padded_slash() {
    let args = [Value::Text("06/15/2024".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Number(45458.0));
}
