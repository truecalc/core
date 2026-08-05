use super::super::gcd_fn;
use crate::types::Value;

#[test]
fn gcd_12_8() {
    assert_eq!(gcd_fn(&[Value::Number(12.0), Value::Number(8.0)]), Value::Number(4.0));
}

#[test]
fn gcd_15_25() {
    assert_eq!(gcd_fn(&[Value::Number(15.0), Value::Number(25.0)]), Value::Number(5.0));
}

#[test]
fn gcd_coprime() {
    assert_eq!(gcd_fn(&[Value::Number(7.0), Value::Number(13.0)]), Value::Number(1.0));
}

#[test]
fn gcd_three_args() {
    assert_eq!(
        gcd_fn(&[Value::Number(100.0), Value::Number(75.0), Value::Number(50.0)]),
        Value::Number(25.0)
    );
}

#[test]
fn gcd_same_values() {
    assert_eq!(gcd_fn(&[Value::Number(6.0), Value::Number(6.0)]), Value::Number(6.0));
}

#[test]
fn gcd_one_value() {
    assert_eq!(gcd_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn gcd_1_1() {
    assert_eq!(gcd_fn(&[Value::Number(1.0), Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn gcd_truncates_floats() {
    // 12.9 -> 12, 8.9 -> 8 => GCD(12, 8) = 4
    assert_eq!(gcd_fn(&[Value::Number(12.9), Value::Number(8.9)]), Value::Number(4.0));
}
