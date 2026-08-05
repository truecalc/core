use super::super::weeknum_fn;
use crate::types::Value;

// DATE(2024,12,31) = 45657 (Tuesday) — last day of year

#[test]
fn dec31_type1_is_week53() {
    // =WEEKNUM(DATE(2024,12,31),1) → 53
    let args = [Value::Number(45657.0), Value::Number(1.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(53.0));
}

#[test]
fn type21_gives_iso_week() {
    // DATE(2024,1,1) = 45292, ISO week 1
    let args = [Value::Number(45292.0), Value::Number(21.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn default_type_omitted_equals_type1() {
    let with_type = [Value::Number(45292.0), Value::Number(1.0)];
    let without_type = [Value::Number(45292.0)];
    assert_eq!(weeknum_fn(&with_type), weeknum_fn(&without_type));
}
