use super::super::*;
use crate::types::Value;

#[test]
fn var_matches_var_s_result() {
    // VAR delegates to VAR.S (sample variance)
    // [2, 4, 6]: sample var=4
    let result = var_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn var_two_values() {
    // [1, 3]: sample var=2
    let result = var_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(2.0));
}
