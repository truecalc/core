use super::super::*;
use crate::types::Value;

#[test]
fn stdevpa_true_counts_as_one() {
    // Bool(true)=1.0; single value → pop stdev=0
    assert_eq!(stdevpa_fn(&[Value::Bool(true)]), Value::Number(0.0));
}

#[test]
fn stdevpa_false_counts_as_zero() {
    // Bool(false)=0.0; single value → pop stdev=0
    assert_eq!(stdevpa_fn(&[Value::Bool(false)]), Value::Number(0.0));
}

#[test]
fn stdevpa_text_returns_value_error() {
    // Literal text as direct arg → #VALUE! (Google Sheets oracle)
    let result = stdevpa_fn(&[Value::Text("hello".to_string()), Value::Number(4.0)]);
    assert_eq!(result, Value::Error(crate::types::ErrorKind::Value));
}

#[test]
fn stdevpa_bool_and_number() {
    // [true=1, false=0, 5.0]: pop var=14/3, stdev=sqrt(14/3)
    let result = stdevpa_fn(&[Value::Bool(true), Value::Bool(false), Value::Number(5.0)]);
    if let Value::Number(v) = result {
        assert!((v - (14.0_f64 / 3.0).sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn stdevpa_all_same_values_returns_zero() {
    assert_eq!(
        stdevpa_fn(&[Value::Number(3.0), Value::Number(3.0), Value::Number(3.0)]),
        Value::Number(0.0)
    );
}
