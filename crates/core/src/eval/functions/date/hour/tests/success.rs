use super::super::*;
use crate::types::Value;

#[test]
fn hour_14_30() {
    // TIME(14,30,0) = 0.6041666...
    let serial = (14.0 * 3600.0 + 30.0 * 60.0) / 86400.0;
    assert_eq!(hour_fn(&[Value::Number(serial)]), Value::Number(14.0));
}

#[test]
fn hour_midnight() {
    // TIME(0,0,0) = 0.0
    assert_eq!(hour_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn hour_23() {
    // TIME(23,59,0)
    let serial = (23.0 * 3600.0 + 59.0 * 60.0) / 86400.0;
    assert_eq!(hour_fn(&[Value::Number(serial)]), Value::Number(23.0));
}

#[test]
fn hour_noon() {
    // TIME(12,0,0) = 0.5
    assert_eq!(hour_fn(&[Value::Number(0.5)]), Value::Number(12.0));
}
