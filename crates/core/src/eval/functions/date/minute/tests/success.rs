use super::super::*;
use crate::types::Value;

#[test]
fn minute_30() {
    // TIME(14,30,0)
    let serial = (14.0 * 3600.0 + 30.0 * 60.0) / 86400.0;
    assert_eq!(minute_fn(&[Value::Number(serial)]), Value::Number(30.0));
}

#[test]
fn minute_0() {
    // TIME(14,0,0)
    let serial = (14.0 * 3600.0) / 86400.0;
    assert_eq!(minute_fn(&[Value::Number(serial)]), Value::Number(0.0));
}

#[test]
fn minute_59() {
    // TIME(14,59,0)
    let serial = (14.0 * 3600.0 + 59.0 * 60.0) / 86400.0;
    assert_eq!(minute_fn(&[Value::Number(serial)]), Value::Number(59.0));
}

#[test]
fn minute_45() {
    // TIME(0,45,0)
    let serial = (45.0 * 60.0) / 86400.0;
    assert_eq!(minute_fn(&[Value::Number(serial)]), Value::Number(45.0));
}
