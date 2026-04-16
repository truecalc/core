use super::super::networkdays_intl_fn;
use crate::types::Value;

// Serials: DATE(2024,1,1)=45292 Mon, DATE(2024,1,2)=45293 Tue,
//          DATE(2024,1,4)=45295 Thu, DATE(2024,1,5)=45296 Fri, DATE(2024,1,6)=45297 Sat

#[test]
fn weekend_1_sat_sun_mon_to_fri() {
    // NETWORKDAYS.INTL(DATE(2024,1,1), DATE(2024,1,5), 1) = 5
    let args = [Value::Number(45292.0), Value::Number(45296.0), Value::Number(1.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(5.0));
}

#[test]
fn weekend_2_sun_mon_tue_to_sat() {
    // NETWORKDAYS.INTL(DATE(2024,1,2), DATE(2024,1,6), 2) = 5
    let args = [Value::Number(45293.0), Value::Number(45297.0), Value::Number(2.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(5.0));
}

#[test]
fn weekend_7_fri_sat_mon_to_thu() {
    // NETWORKDAYS.INTL(DATE(2024,1,1), DATE(2024,1,4), 7) = 4
    let args = [Value::Number(45292.0), Value::Number(45295.0), Value::Number(7.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(4.0));
}

#[test]
fn string_mask_sat_sun() {
    // NETWORKDAYS.INTL(DATE(2024,1,1), DATE(2024,1,5), "0000011") = 5
    let args = [
        Value::Number(45292.0),
        Value::Number(45296.0),
        Value::Text("0000011".to_string()),
    ];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(5.0));
}
