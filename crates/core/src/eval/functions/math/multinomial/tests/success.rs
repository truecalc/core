use super::super::multinomial_fn;
use crate::types::Value;

#[test]
fn multinomial_2_3() {
    // (2+3)! / (2! * 3!) = 120 / 12 = 10
    assert_eq!(
        multinomial_fn(&[Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(10.0)
    );
}

#[test]
fn multinomial_1_1_1() {
    // (1+1+1)! / (1! * 1! * 1!) = 6
    assert_eq!(
        multinomial_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]),
        Value::Number(6.0)
    );
}

#[test]
fn multinomial_single_value() {
    // (4)! / 4! = 1
    assert_eq!(multinomial_fn(&[Value::Number(4.0)]), Value::Number(1.0));
}

#[test]
fn multinomial_3_2_1() {
    // (3+2+1)! / (3! * 2! * 1!) = 720 / 12 = 60
    assert_eq!(
        multinomial_fn(&[Value::Number(3.0), Value::Number(2.0), Value::Number(1.0)]),
        Value::Number(60.0)
    );
}

#[test]
fn multinomial_1_2() {
    // (1+2)! / (1! * 2!) = 6 / 2 = 3
    assert_eq!(
        multinomial_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn multinomial_2_3_4() {
    // (2+3+4)! / (2! * 3! * 4!) = 362880 / (2 * 6 * 24) = 1260
    assert_eq!(
        multinomial_fn(&[Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)]),
        Value::Number(1260.0)
    );
}

#[test]
fn multinomial_2_2_2() {
    // (2+2+2)! / (2! * 2! * 2!) = 720 / 8 = 90
    assert_eq!(
        multinomial_fn(&[Value::Number(2.0), Value::Number(2.0), Value::Number(2.0)]),
        Value::Number(90.0)
    );
}
