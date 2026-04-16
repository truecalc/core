use super::super::workday_intl_fn;
use crate::types::Value;

// Oracle: WORKDAY.INTL(DATE(2024,1,1),5,1)=45299
//         WORKDAY.INTL(DATE(2024,1,1),1,7)=45293
//         WORKDAY.INTL(DATE(2024,1,1),5,"0000011")=45299
//         WORKDAY.INTL(DATE(2024,1,1),0,1)=45292

#[test]
fn weekend_1_mon_plus_5() {
    // WORKDAY.INTL(DATE(2024,1,1), 5, 1) = 45299
    let args = [Value::Number(45292.0), Value::Number(5.0), Value::Number(1.0)];
    assert_eq!(workday_intl_fn(&args), Value::Number(45299.0));
}

#[test]
fn weekend_7_fri_sat_mon_plus_1() {
    // WORKDAY.INTL(DATE(2024,1,1), 1, 7) = 45293  (Tue; Mon is work, so Mon+1=Tue)
    // Code 7 = Fri+Sat weekend. Mon is a working day, so Mon+1 working day = Tue
    let args = [Value::Number(45292.0), Value::Number(1.0), Value::Number(7.0)];
    assert_eq!(workday_intl_fn(&args), Value::Number(45293.0));
}

#[test]
fn string_mask_sat_sun_mon_plus_5() {
    // WORKDAY.INTL(DATE(2024,1,1), 5, "0000011") = 45299
    let args = [
        Value::Number(45292.0),
        Value::Number(5.0),
        Value::Text("0000011".to_string()),
    ];
    assert_eq!(workday_intl_fn(&args), Value::Number(45299.0));
}

#[test]
fn zero_days_returns_start() {
    // WORKDAY.INTL(DATE(2024,1,1), 0, 1) = 45292
    let args = [Value::Number(45292.0), Value::Number(0.0), Value::Number(1.0)];
    assert_eq!(workday_intl_fn(&args), Value::Number(45292.0));
}
