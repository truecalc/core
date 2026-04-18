use super::super::randarray_fn;
use crate::types::Value;

fn n(v: f64) -> Value {
    Value::Number(v)
}

#[test]
fn one_by_one_array_returns_single_nested_value() {
    let result = randarray_fn(&[n(1.0), n(1.0)]);
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
    let result = randarray_fn(&[n(2.9)]);
    if let Value::Array(outer) = result {
        assert_eq!(outer.len(), 2);
    } else {
        panic!("expected array");
    }
}
