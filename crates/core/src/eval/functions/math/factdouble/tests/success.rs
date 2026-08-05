use super::super::factdouble_fn;
use crate::types::Value;

#[test]
fn factdouble_odd_5() {
    // 5!! = 5*3*1 = 15
    assert_eq!(factdouble_fn(&[Value::Number(5.0)]), Value::Number(15.0));
}

#[test]
fn factdouble_even_6() {
    // 6!! = 6*4*2 = 48
    assert_eq!(factdouble_fn(&[Value::Number(6.0)]), Value::Number(48.0));
}

#[test]
fn factdouble_one() {
    // 1!! = 1
    assert_eq!(factdouble_fn(&[Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn factdouble_zero() {
    // 0!! = 1
    assert_eq!(factdouble_fn(&[Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn factdouble_two() {
    // 2!! = 2
    assert_eq!(factdouble_fn(&[Value::Number(2.0)]), Value::Number(2.0));
}

#[test]
fn factdouble_four() {
    // 4!! = 4*2 = 8
    assert_eq!(factdouble_fn(&[Value::Number(4.0)]), Value::Number(8.0));
}

#[test]
fn factdouble_odd_7() {
    // 7!! = 7*5*3*1 = 105
    assert_eq!(factdouble_fn(&[Value::Number(7.0)]), Value::Number(105.0));
}

#[test]
fn factdouble_even_8() {
    // 8!! = 8*6*4*2 = 384
    assert_eq!(factdouble_fn(&[Value::Number(8.0)]), Value::Number(384.0));
}

#[test]
fn factdouble_10() {
    // 10!! = 10*8*6*4*2 = 3840
    assert_eq!(factdouble_fn(&[Value::Number(10.0)]), Value::Number(3840.0));
}

#[test]
fn factdouble_9() {
    // 9!! = 9*7*5*3*1 = 945
    assert_eq!(factdouble_fn(&[Value::Number(9.0)]), Value::Number(945.0));
}
