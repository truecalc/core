use super::super::randarray_fn;
use crate::types::Value;

fn n(v: f64) -> Value {
    Value::Number(v)
}

#[test]
fn no_args_returns_single_number_in_0_1() {
    let result = randarray_fn(&[]);
    assert!(
        matches!(result, Value::Number(_)),
        "expected Number, got {:?}",
        result
    );
    if let Value::Number(v) = result {
        assert!(v >= 0.0 && v < 1.0, "random value {} not in [0, 1)", v);
    }
}

#[test]
fn one_arg_rows_returns_nested_2d_array() {
    let result = randarray_fn(&[n(3.0)]);
    assert!(matches!(result, Value::Array(_)));
    if let Value::Array(outer) = result {
        assert_eq!(outer.len(), 3);
        for row in &outer {
            assert!(matches!(row, Value::Array(_)));
            if let Value::Array(inner) = row {
                assert_eq!(inner.len(), 1);
                assert!(matches!(inner[0], Value::Number(_)));
            }
        }
    }
}

#[test]
fn two_args_rows_cols_returns_correct_shape() {
    let result = randarray_fn(&[n(2.0), n(4.0)]);
    if let Value::Array(outer) = result {
        assert_eq!(outer.len(), 2, "expected 2 rows");
        for row in &outer {
            if let Value::Array(inner) = row {
                assert_eq!(inner.len(), 4, "expected 4 cols");
            } else {
                panic!("expected inner array");
            }
        }
    } else {
        panic!("expected outer array");
    }
}

#[test]
fn values_are_numbers() {
    let result = randarray_fn(&[n(2.0), n(2.0)]);
    if let Value::Array(outer) = result {
        for row in &outer {
            if let Value::Array(inner) = row {
                for v in inner {
                    assert!(
                        matches!(v, Value::Number(_)),
                        "expected Number, got {:?}",
                        v
                    );
                }
            }
        }
    }
}
