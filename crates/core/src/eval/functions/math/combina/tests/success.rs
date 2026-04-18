use super::super::combina_fn;
use crate::types::Value;

#[test]
fn combina_5_2() {
    // C(5+2-1, 2) = C(6,2) = 15
    assert_eq!(combina_fn(&[Value::Number(5.0), Value::Number(2.0)]), Value::Number(15.0));
}

#[test]
fn combina_3_2() {
    // C(3+2-1, 2) = C(4,2) = 6
    assert_eq!(combina_fn(&[Value::Number(3.0), Value::Number(2.0)]), Value::Number(6.0));
}

#[test]
fn combina_1_1() {
    // C(1+1-1, 1) = C(1,1) = 1
    assert_eq!(combina_fn(&[Value::Number(1.0), Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn combina_10_0() {
    assert_eq!(combina_fn(&[Value::Number(10.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn combina_5_0() {
    assert_eq!(combina_fn(&[Value::Number(5.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn combina_3_3() {
    // C(3+3-1, 3) = C(5,3) = 10
    assert_eq!(combina_fn(&[Value::Number(3.0), Value::Number(3.0)]), Value::Number(10.0));
}

#[test]
fn combina_2_4() {
    // C(2+4-1, 4) = C(5,4) = 5
    assert_eq!(combina_fn(&[Value::Number(2.0), Value::Number(4.0)]), Value::Number(5.0));
}

#[test]
fn combina_10_2() {
    // C(10+2-1, 2) = C(11,2) = 55
    assert_eq!(combina_fn(&[Value::Number(10.0), Value::Number(2.0)]), Value::Number(55.0));
}

#[test]
fn combina_4_3() {
    // C(4+3-1, 3) = C(6,3) = 20
    assert_eq!(combina_fn(&[Value::Number(4.0), Value::Number(3.0)]), Value::Number(20.0));
}
