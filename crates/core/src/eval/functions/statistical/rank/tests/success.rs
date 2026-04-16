use super::super::*;
use crate::types::Value;

#[test]
fn rank_descending_default() {
    // RANK(3, [1,2,3,4,5]) descending: values > 3 = {4,5} → rank 3
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(3.0), arr]), Value::Number(3.0));
}

#[test]
fn rank_descending_explicit_order0() {
    // RANK(5, [1,2,3,4,5], 0) = 1 (highest gets rank 1)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(5.0), arr, Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn rank_ascending_order1() {
    // RANK(3, [1,2,3,4,5], 1) ascending: values < 3 = {1,2} → rank 3
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(3.0), arr, Value::Number(1.0)]), Value::Number(3.0));
}

#[test]
fn rank_ties_get_same_rank() {
    // RANK(3, [1,2,3,3,5]) descending: values > 3 = {5} → rank 2 (both 3s share rank 2)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(3.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(3.0), arr]), Value::Number(2.0));
}
