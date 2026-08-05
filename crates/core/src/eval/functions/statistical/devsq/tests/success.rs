use super::super::*;
use crate::types::Value;

#[test]
fn devsq_basic() {
    // mean=3, deviations squared: (1-3)^2=4, (2-3)^2=1, (3-3)^2=0, (4-3)^2=1, (5-3)^2=4 -> devsq=10
    let result = devsq_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn devsq_two_values() {
    // mean=3, deviations squared: (1-3)^2=4, (5-3)^2=4 -> devsq=8
    let result = devsq_fn(&[Value::Number(1.0), Value::Number(5.0)]);
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn devsq_bool_coerces_to_number() {
    // Direct bool coerces: TRUE->1, FALSE->0
    // values: 2.0, 4.0, 1.0, 0.0 -> mean=1.75
    // devsq=(2-1.75)^2+(4-1.75)^2+(1-1.75)^2+(0-1.75)^2
    //      =0.0625+5.0625+0.5625+3.0625=8.75
    let result = devsq_fn(&[
        Value::Number(2.0),
        Value::Number(4.0),
        Value::Bool(true),
        Value::Bool(false),
    ]);
    assert_eq!(result, Value::Number(8.75));
}
