use super::super::sumsq_fn;
use crate::types::Value;

#[test]
fn non_numeric_text_returns_error() {
    // GS: non-numeric text in SUMSQ -> #VALUE!
    use crate::types::ErrorKind;
    let result = sumsq_fn(&[Value::Number(3.0), Value::Text("hello".to_string())]);
    assert_eq!(result, Value::Error(ErrorKind::Value));
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
