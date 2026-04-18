use super::super::convert_fn;
use crate::types::Value;

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

fn round6(v: f64) -> f64 {
    (v * 1000000.0).round() / 1000000.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[test]
fn kg_to_lbm_rounded_4() {
    let result = convert_fn(&[Value::Number(1.0), Value::Text("kg".to_string()), Value::Text("lbm".to_string())]);
    match result {
        Value::Number(n) => assert_eq!(round4(n), 2.2046),
        other => panic!("expected Number, got {other:?}"),
    }
}

#[test]
fn lbm_to_kg_rounded_6() {
    let result = convert_fn(&[Value::Number(1.0), Value::Text("lbm".to_string()), Value::Text("kg".to_string())]);
    match result {
        Value::Number(n) => assert_eq!(round6(n), 0.453592),
        other => panic!("expected Number, got {other:?}"),
    }
}

#[test]
fn yr_to_day_rounded_2() {
    let result = convert_fn(&[Value::Number(1.0), Value::Text("yr".to_string()), Value::Text("day".to_string())]);
    match result {
        Value::Number(n) => assert_eq!(round2(n), 365.25),
        other => panic!("expected Number, got {other:?}"),
    }
}

#[test]
fn mi_to_km_rounded_4() {
    let result = convert_fn(&[Value::Number(1.0), Value::Text("mi".to_string()), Value::Text("km".to_string())]);
    match result {
        Value::Number(n) => assert_eq!(round4(n), 1.6093),
        other => panic!("expected Number, got {other:?}"),
    }
}
