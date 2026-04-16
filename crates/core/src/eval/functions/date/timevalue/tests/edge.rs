use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < 1e-9 } else { false }
}

#[test]
fn end_of_day() {
    let args = [Value::Text("23:59:59".to_string())];
    assert!(approx(timevalue_fn(&args), (23.0 * 3600.0 + 59.0 * 60.0 + 59.0) / 86400.0));
}

#[test]
fn hm_format_no_seconds() {
    // "14:30" same as "14:30:00"
    let args_hm  = [Value::Text("14:30".to_string())];
    let args_hms = [Value::Text("14:30:00".to_string())];
    assert_eq!(timevalue_fn(&args_hm), timevalue_fn(&args_hms));
}
