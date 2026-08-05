use super::super::{
    daverage_fn, dcount_fn, dcounta_fn, dget_fn, dmax_fn, dmin_fn, dproduct_fn, dstdev_fn,
    dstdevp_fn, dsum_fn, dvar_fn, dvarp_fn,
};
use crate::types::{ErrorKind, Value};

/// Build the shared test database as a Value::Array of rows.
///
///   Name   | Age | Sales
///   Alice  |  30 |   100
///   Bob    |  25 |   200
///   Alice  |  35 |   150
fn test_db() -> Value {
    let row = |name: &str, age: f64, sales: f64| {
        Value::Array(vec![
            Value::Text(name.to_string()),
            Value::Number(age),
            Value::Number(sales),
        ])
    };
    Value::Array(vec![
        Value::Array(vec![
            Value::Text("Name".to_string()),
            Value::Text("Age".to_string()),
            Value::Text("Sales".to_string()),
        ]),
        row("Alice", 30.0, 100.0),
        row("Bob", 25.0, 200.0),
        row("Alice", 35.0, 150.0),
    ])
}

/// Criteria matching Name = "Alice"
fn criteria_alice() -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Alice".to_string())]),
    ])
}

/// Criteria matching Age > 25
fn criteria_age_gt_25() -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("Age".to_string())]),
        Value::Array(vec![Value::Text(">25".to_string())]),
    ])
}

/// Criteria matching Name = "Bob"
fn criteria_bob() -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Bob".to_string())]),
    ])
}

/// Criteria matching all rows (Age >= 1)
fn criteria_all() -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("Age".to_string())]),
        Value::Array(vec![Value::Text(">=1".to_string())]),
    ])
}

// ── DSUM ─────────────────────────────────────────────────────────────────────

#[test]
fn dsum_alice_sales() {
    // Alice rows: 100 + 150 = 250
    let result = dsum_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(250.0));
}

#[test]
fn dsum_field_by_number_index() {
    // field=3 is the Sales column (1-based)
    let result = dsum_fn(&[test_db(), Value::Number(3.0), criteria_alice()]);
    assert_eq!(result, Value::Number(250.0));
}

#[test]
fn dsum_all_rows() {
    // 100 + 200 + 150 = 450
    let result = dsum_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(450.0));
}

// ── DCOUNT ───────────────────────────────────────────────────────────────────

#[test]
fn dcount_age_gt_25() {
    // Alice(30), Alice(35) → 2 numeric age values
    let result = dcount_fn(&[test_db(), Value::Text("Age".to_string()), criteria_age_gt_25()]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn dcount_bob() {
    // Bob → 1 numeric age value
    let result = dcount_fn(&[test_db(), Value::Text("Age".to_string()), criteria_bob()]);
    assert_eq!(result, Value::Number(1.0));
}

// ── DAVERAGE ─────────────────────────────────────────────────────────────────

#[test]
fn daverage_alice_sales() {
    // (100 + 150) / 2 = 125
    let result = daverage_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(125.0));
}

#[test]
fn daverage_all_sales() {
    // (100 + 200 + 150) / 3 = 150
    let result = daverage_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(150.0));
}

// ── DMAX ─────────────────────────────────────────────────────────────────────

#[test]
fn dmax_all_sales() {
    // max(100, 200, 150) = 200
    let result = dmax_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(200.0));
}

#[test]
fn dmax_alice_sales() {
    // max(100, 150) = 150
    let result = dmax_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(150.0));
}

// ── DMIN ─────────────────────────────────────────────────────────────────────

#[test]
fn dmin_all_age() {
    // min(30, 25, 35) = 25
    let result = dmin_fn(&[test_db(), Value::Text("Age".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(25.0));
}

#[test]
fn dmin_bob_age() {
    // Bob age = 25
    let result = dmin_fn(&[test_db(), Value::Text("Age".to_string()), criteria_bob()]);
    assert_eq!(result, Value::Number(25.0));
}

// ── DGET ─────────────────────────────────────────────────────────────────────

#[test]
fn dget_unique_match_returns_value() {
    // Bob matches exactly once; Sales = 200
    let result = dget_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_bob()]);
    assert_eq!(result, Value::Number(200.0));
}

#[test]
fn dget_multiple_matches_returns_num_error() {
    // Alice matches twice → #NUM!
    let result = dget_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Error(ErrorKind::Num));
}

#[test]
fn dget_no_match_returns_value_error() {
    // No one named "Charlie" → #VALUE!
    let criteria_charlie = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Charlie".to_string())]),
    ]);
    let result = dget_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_charlie]);
    assert_eq!(result, Value::Error(ErrorKind::Value));
}

// ── DPRODUCT ─────────────────────────────────────────────────────────────────

#[test]
fn dproduct_alice_sales() {
    // 100 * 150 = 15000
    let result = dproduct_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(15000.0));
}

// ── DCOUNTA ──────────────────────────────────────────────────────────────────

#[test]
fn dcounta_all_names() {
    // 3 non-empty name values
    let result = dcounta_fn(&[test_db(), Value::Text("Name".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(3.0));
}

// ── DSTDEV / DSTDEVP ─────────────────────────────────────────────────────────

#[test]
fn dstdev_alice_sales() {
    // Alice Sales: 100, 150. sample stdev = sqrt(((100-125)^2 + (150-125)^2) / 1) = sqrt(1250) ≈ 35.355
    let result = dstdev_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    if let Value::Number(n) = result {
        let expected = 1250.0_f64.sqrt();
        assert!((n - expected).abs() < 1e-9, "expected {expected}, got {n}");
    } else {
        panic!("expected Number, got {result:?}");
    }
}

#[test]
fn dstdevp_alice_sales() {
    // Alice Sales: 100, 150. population stdev = sqrt(((100-125)^2 + (150-125)^2) / 2) = sqrt(625) = 25
    let result = dstdevp_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(25.0));
}

// ── DVAR / DVARP ─────────────────────────────────────────────────────────────

#[test]
fn dvar_alice_sales() {
    // sample variance = 1250
    let result = dvar_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(1250.0));
}

#[test]
fn dvarp_alice_sales() {
    // population variance = 625
    let result = dvarp_fn(&[test_db(), Value::Text("Sales".to_string()), criteria_alice()]);
    assert_eq!(result, Value::Number(625.0));
}
