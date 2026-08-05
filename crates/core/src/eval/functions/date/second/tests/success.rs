use super::super::*;
use crate::types::Value;

#[test]
fn second_45() {
    // TIME(14,30,45)
    let serial = (14.0 * 3600.0 + 30.0 * 60.0 + 45.0) / 86400.0;
    assert_eq!(second_fn(&[Value::Number(serial)]), Value::Number(45.0));
}

#[test]
fn second_0() {
    // TIME(14,30,0)
    let serial = (14.0 * 3600.0 + 30.0 * 60.0) / 86400.0;
    assert_eq!(second_fn(&[Value::Number(serial)]), Value::Number(0.0));
}

#[test]
fn second_59() {
    // TIME(14,30,59)
    let serial = (14.0 * 3600.0 + 30.0 * 60.0 + 59.0) / 86400.0;
    assert_eq!(second_fn(&[Value::Number(serial)]), Value::Number(59.0));
}

#[test]
fn second_30() {
    // TIME(0,0,30)
    let serial = 30.0 / 86400.0;
    assert_eq!(second_fn(&[Value::Number(serial)]), Value::Number(30.0));
}
