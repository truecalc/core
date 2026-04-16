use super::super::*;
use crate::types::Value;

#[test]
fn harmean_of_two_numbers() {
    // HARMEAN(1, 4) = 2 / (1 + 0.25) = 1.6
    let result = harmean_fn(&[Value::Number(1.0), Value::Number(4.0)]);
    if let Value::Number(n) = result {
        assert!((n - 1.6).abs() < 1e-10);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn harmean_single_value() {
    assert_eq!(harmean_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}
