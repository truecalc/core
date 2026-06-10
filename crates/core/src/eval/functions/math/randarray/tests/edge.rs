use crate::Engine;
use crate::types::Value;
use std::collections::HashMap;

#[test]
fn one_by_one_array_returns_single_nested_value() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RANDARRAY(1, 1)", &HashMap::new());
    if let Value::Array(outer) = result {
        assert_eq!(outer.len(), 1);
        if let Value::Array(inner) = &outer[0] {
            assert_eq!(inner.len(), 1);
            assert!(matches!(inner[0], Value::Number(_)));
        } else {
            panic!("inner should be array");
        }
    } else {
        panic!("expected array");
    }
}

#[test]
fn fractional_rows_truncates_to_integer() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RANDARRAY(2.9)", &HashMap::new());
    if let Value::Array(outer) = result {
        assert_eq!(outer.len(), 2);
    } else {
        panic!("expected array");
    }
}
