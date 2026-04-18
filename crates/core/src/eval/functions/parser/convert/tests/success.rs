use super::super::convert_fn;
use crate::types::Value;

#[test]
fn km_to_m() {
    assert_eq!(convert_fn(&[Value::Number(1.0), Value::Text("km".to_string()), Value::Text("m".to_string())]), Value::Number(1000.0));
}

#[test]
fn m_to_km() {
    assert_eq!(convert_fn(&[Value::Number(1.0), Value::Text("m".to_string()), Value::Text("km".to_string())]), Value::Number(0.001));
}

#[test]
fn celsius_to_fahrenheit_100() {
    assert_eq!(convert_fn(&[Value::Number(100.0), Value::Text("C".to_string()), Value::Text("F".to_string())]), Value::Number(212.0));
}

#[test]
fn celsius_to_fahrenheit_0() {
    assert_eq!(convert_fn(&[Value::Number(0.0), Value::Text("C".to_string()), Value::Text("F".to_string())]), Value::Number(32.0));
}

#[test]
fn fahrenheit_to_celsius_32() {
    assert_eq!(convert_fn(&[Value::Number(32.0), Value::Text("F".to_string()), Value::Text("C".to_string())]), Value::Number(0.0));
}

#[test]
fn kelvin_to_celsius() {
    // 273.15 K = 0.0 C (exact within floating point)
    let result = convert_fn(&[Value::Number(273.15), Value::Text("K".to_string()), Value::Text("C".to_string())]);
    match result {
        Value::Number(n) => assert!((n - 0.0).abs() < 1e-9, "expected ~0.0, got {n}"),
        other => panic!("expected Number, got {other:?}"),
    }
}

#[test]
fn hr_to_mn() {
    assert_eq!(convert_fn(&[Value::Number(1.0), Value::Text("hr".to_string()), Value::Text("mn".to_string())]), Value::Number(60.0));
}

#[test]
fn day_to_hr() {
    assert_eq!(convert_fn(&[Value::Number(1.0), Value::Text("day".to_string()), Value::Text("hr".to_string())]), Value::Number(24.0));
}
