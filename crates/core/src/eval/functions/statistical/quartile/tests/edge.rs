use super::super::*;
use crate::types::Value;

#[test]
fn quartile_single_value_all_quarts() {
    // QUARTILE([7], 0) = QUARTILE([7], 1) = ... = 7.0
    for q in [0.0_f64, 1.0, 2.0, 3.0, 4.0] {
        assert_eq!(
            quartile_fn(&[Value::Number(7.0), Value::Number(q)]),
            Value::Number(7.0)
        );
    }
}

#[test]
fn quartile_q3() {
    // QUARTILE([1,2,3,4,5], 3) = 4.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_fn(&[arr, Value::Number(3.0)]), Value::Number(4.0));
}
