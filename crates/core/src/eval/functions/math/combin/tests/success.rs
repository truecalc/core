use super::super::combin_fn;
use crate::types::Value;

#[test]
fn combin_5_2() {
    assert_eq!(combin_fn(&[Value::Number(5.0), Value::Number(2.0)]), Value::Number(10.0));
}

#[test]
fn combin_10_0() {
    assert_eq!(combin_fn(&[Value::Number(10.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn combin_10_10() {
    assert_eq!(combin_fn(&[Value::Number(10.0), Value::Number(10.0)]), Value::Number(1.0));
}

#[test]
fn combin_10_5() {
    assert_eq!(combin_fn(&[Value::Number(10.0), Value::Number(5.0)]), Value::Number(252.0));
}

#[test]
fn combin_4_2() {
    assert_eq!(combin_fn(&[Value::Number(4.0), Value::Number(2.0)]), Value::Number(6.0));
}

#[test]
fn combin_1_1() {
    assert_eq!(combin_fn(&[Value::Number(1.0), Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn combin_20_10() {
    assert_eq!(combin_fn(&[Value::Number(20.0), Value::Number(10.0)]), Value::Number(184756.0));
}

#[test]
fn combin_8_2() {
    assert_eq!(combin_fn(&[Value::Number(8.0), Value::Number(2.0)]), Value::Number(28.0));
}

#[test]
fn combin_truncates_floats() {
    // 5.9 truncates to 5, 2.9 truncates to 2 => C(5,2) = 10
    assert_eq!(combin_fn(&[Value::Number(5.9), Value::Number(2.9)]), Value::Number(10.0));
}
