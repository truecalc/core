use super::super::*;
use crate::types::Value;

// Oracle: EOMONTH(DATE(2024,12,1), 1) = 45688 = Jan 31, 2025 (year rollover)
#[test]
fn plus_one_dec_to_jan_next_year() {
    let args = [Value::Number(45627.0), Value::Number(1.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45688.0));
}

// Oracle: EOMONTH(DATE(2024,6,15), -12) = 45107 = Jun 30, 2023
#[test]
fn minus_twelve_months_back_one_year() {
    // DATE(2024,6,15) = 45458, end of Jun 2023 = 45107
    let args = [Value::Number(45458.0), Value::Number(-12.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45107.0));
}

// Always returns last day: mid-month input still gives last day
#[test]
fn mid_month_input_gives_last_day() {
    // DATE(2024,2,15) -> end of Feb 2024 = 45351 (Feb 29, leap)
    // DATE(2024,2,15) = 45337
    let args = [Value::Number(45337.0), Value::Number(0.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45351.0));
}

// Negative months: Feb 2024 - 1 month = Jan 2024 end = 45322
#[test]
fn minus_one_from_feb_to_jan() {
    let args = [Value::Number(45323.0), Value::Number(-1.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45322.0));
}
