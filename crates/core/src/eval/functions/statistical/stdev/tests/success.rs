use super::super::*;
use crate::types::Value;

#[test]
fn stdev_matches_stdev_s_result() {
    // STDEV delegates to STDEV.S (sample stdev)
    // [2, 4, 6]: sample var=4, stdev=2.0
    let result = stdev_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn stdev_two_values() {
    // [1, 3]: sample var=2, stdev=sqrt(2)
    let result = stdev_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}
