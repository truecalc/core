use super::*;
use std::collections::HashMap;

#[test]
fn google_sheets_evaluates_sum() {
    let engine = Engine::google_sheets();
    let result = engine.evaluate("=SUM(1,2)", &HashMap::new());
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn google_sheets_evaluates_with_variables() {
    let engine = Engine::google_sheets();
    let mut vars = HashMap::new();
    vars.insert("A".to_string(), Value::Number(10.0));
    let result = engine.evaluate("=A+5", &vars);
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn parse_error_returns_value_error() {
    let engine = Engine::google_sheets();
    let result = engine.evaluate("not a formula", &HashMap::new());
    assert_eq!(result, Value::Error(ErrorKind::Value));
}
