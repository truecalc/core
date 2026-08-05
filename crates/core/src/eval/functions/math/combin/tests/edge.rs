use super::super::combin_fn;
use crate::types::Value;

#[test]
fn combin_0_0() {
    // C(0,0) = 1 by convention
    assert_eq!(combin_fn(&[Value::Number(0.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn combin_n_0() {
    // C(n,0) = 1 for any n
    assert_eq!(combin_fn(&[Value::Number(100.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn combin_n_n() {
    // C(n,n) = 1 for any n
    assert_eq!(combin_fn(&[Value::Number(50.0), Value::Number(50.0)]), Value::Number(1.0));
}
