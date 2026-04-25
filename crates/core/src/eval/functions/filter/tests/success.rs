use super::super::filter_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

fn bools(bs: &[bool]) -> Value {
    Value::Array(bs.iter().map(|&b| Value::Bool(b)).collect())
}

// ---------------------------------------------------------------------------
// FILTER
// ---------------------------------------------------------------------------

#[test]
fn filter_1d_boolean_mask() {
    // FILTER({1,2,3,4,5}, {true,false,true,false,true}) → {1,3,5}
    let result = filter_fn(&[
        nums(&[1.0, 2.0, 3.0, 4.0, 5.0]),
        bools(&[true, false, true, false, true]),
    ]);
    assert_eq!(result, nums(&[1.0, 3.0, 5.0]));
}

#[test]
fn filter_all_true_returns_original() {
    // FILTER({10,20,30}, {true,true,true}) → {10,20,30}
    let result = filter_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        bools(&[true, true, true]),
    ]);
    assert_eq!(result, nums(&[10.0, 20.0, 30.0]));
}

#[test]
fn filter_with_if_empty_on_no_match() {
    // FILTER({1,2,3}, {false,false,false}, -1) → -1
    let result = filter_fn(&[
        nums(&[1.0, 2.0, 3.0]),
        bools(&[false, false, false]),
        Value::Number(-1.0),
    ]);
    assert_eq!(result, Value::Number(-1.0));
}
