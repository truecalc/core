use super::super::multinomial_fn;
use crate::types::Value;

#[test]
fn multinomial_with_zero() {
    // MULTINOMIAL(0, 3) = (0+3)! / (0! * 3!) = 6 / (1 * 6) = 1
    assert_eq!(
        multinomial_fn(&[Value::Number(0.0), Value::Number(3.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn multinomial_all_zeros() {
    assert_eq!(
        multinomial_fn(&[Value::Number(0.0), Value::Number(0.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn multinomial_truncates_floats() {
    // 2.9 -> 2, 3.9 -> 3 => same as MULTINOMIAL(2, 3) = 10
    assert_eq!(
        multinomial_fn(&[Value::Number(2.9), Value::Number(3.9)]),
        Value::Number(10.0)
    );
}
