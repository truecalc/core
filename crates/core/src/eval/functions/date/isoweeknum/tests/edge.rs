use super::super::isoweeknum_fn;
use crate::types::Value;

// Dates near year boundaries that cross ISO week years

#[test]
fn dec30_2024_monday_is_iso_week1_of_2025() {
    // =ISOWEEKNUM(DATE(2024,12,30)) → 1 (belongs to 2025's week 1)
    // DATE(2024,12,30) = 45656
    let args = [Value::Number(45656.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn jan1_2025_is_iso_week1() {
    // =ISOWEEKNUM(DATE(2025,1,1)) → 1
    // DATE(2025,1,1) = 45658
    let args = [Value::Number(45658.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn dec31_2015_is_week53() {
    // =ISOWEEKNUM(DATE(2015,12,31)) → 53
    // DATE(2015,12,31) = 42369
    let args = [Value::Number(42369.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(53.0));
}
