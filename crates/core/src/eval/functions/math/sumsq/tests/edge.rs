use super::super::sumsq_fn;
use crate::types::Value;

#[test]
fn non_numeric_text_ignored() {
    // Non-numeric text should contribute 0
    let result = sumsq_fn(&[Value::Number(3.0), Value::Text("hello".to_string())]);
    assert_eq!(result, Value::Number(9.0));
}

#[test]
fn many_ones() {
    // SUMSQ(1,1,1,1,1) = 5
    assert_eq!(
        sumsq_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ]),
        Value::Number(5.0)
    );
}

#[test]
fn fractions() {
    // SUMSQ(0.5, 0.5) = 0.25 + 0.25 = 0.5
    assert_eq!(
        sumsq_fn(&[Value::Number(0.5), Value::Number(0.5)]),
        Value::Number(0.5)
    );
}
