use super::super::*;
use crate::types::Value;

#[test]
fn stdeva_true_counts_as_one() {
    // Bool(true)=1.0; [1.0, 3.0]: sample var=2, stdev=sqrt(2)
    let result = stdeva_fn(&[Value::Bool(true), Value::Number(3.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}

#[test]
fn stdeva_false_counts_as_zero() {
    // Bool(false)=0.0; [0.0, 2.0]: sample var=2, stdev=sqrt(2)
    let result = stdeva_fn(&[Value::Bool(false), Value::Number(2.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}

#[test]
fn stdeva_non_parseable_text_counts_as_zero() {
    // AVERAGEA semantics: non-parseable text direct arg -> 0.0
    // [0.0, 4.0]: sample var=8, stdev=sqrt(8)
    let result = stdeva_fn(&[Value::Text("hello".to_string()), Value::Number(4.0)]);
    if let Value::Number(v) = result {
        assert!((v - 8.0_f64.sqrt()).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}

#[test]
fn stdeva_all_same_values_returns_zero() {
    assert_eq!(
        stdeva_fn(&[Value::Number(5.0), Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(0.0)
    );
}
