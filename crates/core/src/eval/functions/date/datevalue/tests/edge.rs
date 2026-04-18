use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn leap_day_2024() {
    let args = [Value::Text("2024-02-29".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Number(45351.0));
}

#[test]
fn invalid_feb29_non_leap() {
    let args = [Value::Text("2023-02-29".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Error(ErrorKind::Value));
}
