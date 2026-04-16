use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn kurt_exactly_four_values() {
    // KURT(1, 2, 3, 4) — minimum valid n; should return a finite number
    let result = kurt_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
    ]);
    if let Value::Number(n) = result {
        assert!(n.is_finite(), "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn kurt_five_values_with_known_result() {
    // KURT(2, 4, 4, 4, 5, 5, 7, 9) ≈ 0.2323 (Excel reference)
    // Use a simpler set: KURT(1,2,3,4,5) — symmetric, negative excess kurtosis
    let result = kurt_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    if let Value::Number(n) = result {
        // Symmetric uniform-like: excess kurtosis should be negative
        assert!(n < 0.0, "expected negative excess kurtosis, got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn kurt_array_arg() {
    // kurt via Array argument
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
    ]);
    let result = kurt_fn(&[arr]);
    if let Value::Number(n) = result {
        assert!(n.is_finite(), "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn kurt_three_values_returns_div0() {
    // n < 4 → #DIV/0!
    assert_eq!(
        kurt_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}
