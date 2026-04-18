use super::super::combina_fn;
use crate::types::Value;

#[test]
fn combina_0_0() {
    assert_eq!(combina_fn(&[Value::Number(0.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn combina_0_k_returns_zero() {
    // n=0, k>0 -> 0
    assert_eq!(combina_fn(&[Value::Number(0.0), Value::Number(3.0)]), Value::Number(0.0));
}

#[test]
fn combina_truncates_floats() {
    // 5.9 -> 5, 2.9 -> 2 => C(6,2) = 15
    assert_eq!(combina_fn(&[Value::Number(5.9), Value::Number(2.9)]), Value::Number(15.0));
}
