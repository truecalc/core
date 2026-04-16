use super::super::*;
use crate::types::Value;

#[test]
fn geomean_value_of_one() {
    // GEOMEAN(1, 1, 1) = 1
    let result = geomean_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]);
    if let Value::Number(n) = result {
        assert!((n - 1.0).abs() < 1e-10, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn geomean_array_arg() {
    // GEOMEAN([4, 9]) = 6
    let arr = Value::Array(vec![Value::Number(4.0), Value::Number(9.0)]);
    let result = geomean_fn(&[arr]);
    if let Value::Number(n) = result {
        assert!((n - 6.0).abs() < 1e-10, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn geomean_large_values() {
    // GEOMEAN(1e100, 1e100) = 1e100 (exact)
    let result = geomean_fn(&[Value::Number(1e100), Value::Number(1e100)]);
    if let Value::Number(n) = result {
        assert!((n - 1e100).abs() / 1e100 < 1e-10, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
