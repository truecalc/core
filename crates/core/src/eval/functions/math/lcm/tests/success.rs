use super::super::lcm_fn;
use crate::types::Value;

#[test]
fn lcm_4_6() {
    assert_eq!(lcm_fn(&[Value::Number(4.0), Value::Number(6.0)]), Value::Number(12.0));
}

#[test]
fn lcm_5_3() {
    assert_eq!(lcm_fn(&[Value::Number(5.0), Value::Number(3.0)]), Value::Number(15.0));
}

#[test]
fn lcm_same_values() {
    assert_eq!(lcm_fn(&[Value::Number(7.0), Value::Number(7.0)]), Value::Number(7.0));
}

#[test]
fn lcm_1_5() {
    assert_eq!(lcm_fn(&[Value::Number(1.0), Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn lcm_three_args() {
    assert_eq!(
        lcm_fn(&[Value::Number(12.0), Value::Number(8.0), Value::Number(6.0)]),
        Value::Number(24.0)
    );
}

#[test]
fn lcm_1_1() {
    assert_eq!(lcm_fn(&[Value::Number(1.0), Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn lcm_x_1() {
    assert_eq!(lcm_fn(&[Value::Number(7.0), Value::Number(1.0)]), Value::Number(7.0));
}

#[test]
fn lcm_single_value() {
    assert_eq!(lcm_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn lcm_truncates_floats() {
    // 4.9 -> 4, 6.9 -> 6 => LCM(4, 6) = 12
    assert_eq!(lcm_fn(&[Value::Number(4.9), Value::Number(6.9)]), Value::Number(12.0));
}
