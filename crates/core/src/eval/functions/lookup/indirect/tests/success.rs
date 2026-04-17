use super::super::indirect_fn;
use crate::types::Value;

#[test]
fn a1_ref_returns_empty() {
    assert_eq!(indirect_fn(&[Value::Text("A1".to_string())]), Value::Empty);
}

#[test]
fn b10_ref_returns_empty() {
    assert_eq!(indirect_fn(&[Value::Text("B10".to_string())]), Value::Empty);
}

#[test]
fn r1c1_ref_returns_empty() {
    assert_eq!(indirect_fn(&[Value::Text("R1C1".to_string())]), Value::Empty);
}

#[test]
fn explicit_a1_flag_returns_empty() {
    assert_eq!(
        indirect_fn(&[Value::Text("A1".to_string()), Value::Bool(true)]),
        Value::Empty
    );
}
