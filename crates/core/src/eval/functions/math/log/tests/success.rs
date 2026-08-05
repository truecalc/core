use super::super::*;
use crate::types::Value;

#[test]
fn log10_of_100() {
    assert_eq!(log_fn(&[Value::Number(100.0)]), Value::Number(2.0));
}

#[test]
fn log_with_base_2() {
    assert_eq!(
        log_fn(&[Value::Number(8.0), Value::Number(2.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn log10_function() {
    assert_eq!(log10_fn(&[Value::Number(1000.0)]), Value::Number(3.0));
}

#[test]
fn ln_of_e() {
    let result = ln_fn(&[Value::Number(std::f64::consts::E)]);
    if let Value::Number(n) = result {
        assert!((n - 1.0).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}
