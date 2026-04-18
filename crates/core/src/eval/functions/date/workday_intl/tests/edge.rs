use super::super::workday_intl_fn;
use crate::types::Value;

#[test]
fn negative_days_backward() {
    // WORKDAY.INTL(DATE(2024,1,1), -1, 1) = 45289  (Dec 29 2023, Friday)
    let args = [Value::Number(45292.0), Value::Number(-1.0), Value::Number(1.0)];
    assert_eq!(workday_intl_fn(&args), Value::Number(45289.0));
}

#[test]
fn code_15_thu_only_skipped() {
    // WORKDAY.INTL(DATE(2024,1,1), 1, 15) = 45293
    // Code 15 = Thu only weekend. From Mon: Tue is next working day (Mon+1=Tue).
    // Wait: Mon is working (not Thu), so Mon+1 working day = Tue = 45293
    let args = [Value::Number(45292.0), Value::Number(1.0), Value::Number(15.0)];
    assert_eq!(workday_intl_fn(&args), Value::Number(45293.0));
}

#[test]
fn default_weekend_same_as_code_1() {
    let with_default = workday_intl_fn(&[Value::Number(45292.0), Value::Number(5.0)]);
    let with_code1   = workday_intl_fn(&[Value::Number(45292.0), Value::Number(5.0), Value::Number(1.0)]);
    assert_eq!(with_default, with_code1);
}

#[test]
fn fri_plus_1_with_fri_sat_weekend_skips_to_sun() {
    // Code 7 = Fri+Sat. WORKDAY.INTL(Fri, 1, 7) → next working day after Fri,
    // skipping Fri(weekend) and Sat(weekend) → Sun? No: start is Fri, next day is Sat (weekend),
    // then Sun (not weekend in code 7) → Sun = 45298
    // Actually code 7 is Fri+Sat weekend. Fri is a weekend day.
    // From Fri (45296): step to Sat (weekend), step to Sun (working) → remaining=0 → Sun = 45298
    let args = [Value::Number(45296.0), Value::Number(1.0), Value::Number(7.0)];
    assert_eq!(workday_intl_fn(&args), Value::Number(45298.0));
}
