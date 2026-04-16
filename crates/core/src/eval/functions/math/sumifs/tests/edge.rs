use super::super::sumifs_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn no_matches_returns_zero() {
    // SUMIFS({1,2,3}, {1,2,3}, ">10") → 0
    let range = nums(&[1.0, 2.0, 3.0]);
    let result = sumifs_fn(&[
        range.clone(),
        range.clone(),
        Value::Text(">10".to_string()),
    ]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn ne_criterion() {
    // SUMIFS({10,20,30}, {1,2,3}, "<>2") → 40 (10+30)
    let result = sumifs_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        nums(&[1.0, 2.0, 3.0]),
        Value::Text("<>2".to_string()),
    ]);
    assert_eq!(result, Value::Number(40.0));
}

#[test]
fn empty_sum_range_returns_zero() {
    let result = sumifs_fn(&[
        Value::Array(vec![]),
        Value::Array(vec![]),
        Value::Number(1.0),
    ]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn two_criteria_text_and_number() {
    // SUMIFS({100,200,300,400}, {"a","b","a","b"}, "a", {1,2,1,2}, 1) → 400 (100+300)
    let result = sumifs_fn(&[
        nums(&[100.0, 200.0, 300.0, 400.0]),
        Value::Array(vec![
            Value::Text("a".into()),
            Value::Text("b".into()),
            Value::Text("a".into()),
            Value::Text("b".into()),
        ]),
        Value::Text("a".to_string()),
        nums(&[1.0, 2.0, 1.0, 2.0]),
        Value::Number(1.0),
    ]);
    assert_eq!(result, Value::Number(400.0));
}
