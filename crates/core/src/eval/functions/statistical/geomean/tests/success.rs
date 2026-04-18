use super::super::*;
use crate::types::Value;

#[test]
fn geomean_of_two_numbers() {
    // GEOMEAN(4, 9) = sqrt(36) = 6
    let result = geomean_fn(&[Value::Number(4.0), Value::Number(9.0)]);
    if let Value::Number(n) = result {
        assert!((n - 6.0).abs() < 1e-10);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn geomean_single_value() {
    let result = geomean_fn(&[Value::Number(5.0)]);
    if let Value::Number(n) = result {
        assert!((n - 5.0).abs() < 1e-10, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
