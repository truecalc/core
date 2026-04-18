use super::super::*;
use crate::types::Value;

#[test]
fn varpa_true_counts_as_one() {
    // Bool(true)=1.0; single value → pop var=0
    assert_eq!(varpa_fn(&[Value::Bool(true)]), Value::Number(0.0));
}

#[test]
fn varpa_false_counts_as_zero() {
    // Bool(false)=0.0; single value → pop var=0
    assert_eq!(varpa_fn(&[Value::Bool(false)]), Value::Number(0.0));
}

#[test]
fn varpa_text_returns_value_error() {
    // Literal text as direct arg → #VALUE! (Google Sheets oracle)
    let result = varpa_fn(&[Value::Text("hello".to_string()), Value::Number(4.0)]);
    assert_eq!(result, Value::Error(crate::types::ErrorKind::Value));
}

#[test]
fn varpa_bool_and_number() {
    // [true=1, false=0, 5.0]: mean=2, pop var=((1-2)²+(0-2)²+(5-2)²)/3=(1+4+9)/3=14/3
    let result = varpa_fn(&[Value::Bool(true), Value::Bool(false), Value::Number(5.0)]);
    if let Value::Number(v) = result {
        assert!((v - 14.0 / 3.0).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn varpa_all_same_values_returns_zero() {
    assert_eq!(
        varpa_fn(&[Value::Number(3.0), Value::Number(3.0), Value::Number(3.0)]),
        Value::Number(0.0)
    );
}
