use super::super::*;
use crate::types::Value;

#[test]
fn quartile_inc_single_element() {
    // QUARTILE.INC([7], q) = 7.0 for all q in {0,1,2,3,4}
    for q in [0.0_f64, 1.0, 2.0, 3.0, 4.0] {
        assert_eq!(
            quartile_inc_fn(&[Value::Number(7.0), Value::Number(q)]),
            Value::Number(7.0)
        );
    }
}

#[test]
fn quartile_inc_q3() {
    // QUARTILE.INC([1,2,3,4,5], 3) = 4.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(3.0)]), Value::Number(4.0));
}

#[test]
fn quartile_inc_same_as_quartile() {
    // QUARTILE.INC and QUARTILE should agree
    let arr1 = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    let arr2 = arr1.clone();
    assert_eq!(
        quartile_inc_fn(&[arr1, Value::Number(1.0)]),
        crate::eval::functions::statistical::quartile::quartile_fn(&[arr2, Value::Number(1.0)])
    );
}
