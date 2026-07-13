use super::*;
use std::collections::HashMap;

#[test]
fn sheets_evaluates_sum() {
    let engine = Engine::sheets();
    let result = engine.evaluate("=SUM(1,2)", &HashMap::new());
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn sheets_evaluates_with_variables() {
    let engine = Engine::sheets();
    let mut vars = HashMap::new();
    vars.insert("A".to_string(), Value::Number(10.0));
    let result = engine.evaluate("=A+5", &vars);
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn parse_error_returns_value_error() {
    let engine = Engine::sheets();
    let result = engine.evaluate("not a formula", &HashMap::new());
    assert_eq!(result, Value::Error(ErrorKind::Value));
}

#[test]
fn sheets_parses_valid_formula() {
    let engine = Engine::sheets();
    assert!(engine.parse("=SUM(1,2)").is_ok());
}

#[test]
fn sheets_parse_rejects_invalid_formula() {
    let engine = Engine::sheets();
    assert!(engine.parse("=SUM(1,").is_err());
}

#[test]
fn sheets_validates_formula() {
    let engine = Engine::sheets();
    assert!(engine.validate("=1+2").is_ok());
    assert!(engine.validate("=1+").is_err());
}

#[test]
fn excel_parses_and_validates() {
    let engine = Engine::excel();
    assert!(engine.parse("=SUM(1,2)").is_ok());
    assert!(engine.validate("=SUM(1,2)").is_ok());
    assert!(engine.validate("=SUM(1,").is_err());
}

#[test]
fn excel_evaluate_returns_unsupported_error() {
    // Excel evaluation semantics are not implemented yet — evaluate must
    // return a clear Unsupported error (not #N/A) so callers can distinguish
    // "Excel eval not implemented" from a legitimate N/A result.
    let engine = Engine::excel();
    let result = engine.evaluate("=SUM(1,2)", &HashMap::new());
    assert_eq!(result, Value::Error(ErrorKind::Unsupported));
}

#[test]
fn excel_evaluate_is_not_na() {
    // ErrorKind::Unsupported must be distinct from ErrorKind::NA.
    let engine = Engine::excel();
    let result = engine.evaluate("=1+1", &HashMap::new());
    assert_ne!(result, Value::Error(ErrorKind::NA));
    assert_eq!(result, Value::Error(ErrorKind::Unsupported));
}

#[test]
fn sheets_na_formula_is_still_na() {
    // Engine::sheets() evaluating =NA() must still return ErrorKind::NA.
    let engine = Engine::sheets();
    let result = engine.evaluate("=NA()", &HashMap::new());
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

#[test]
#[allow(deprecated)]
fn google_sheets_is_deprecated_alias_for_sheets() {
    let engine = Engine::google_sheets();
    let result = engine.evaluate("=SUM(1,2)", &HashMap::new());
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn translate_formula_shifts_relative_reference() {
    let engine = Engine::sheets();
    assert_eq!(engine.translate_formula("=A1", 1, 1), Ok("=B2".to_string()));
}

#[test]
fn translate_formula_preserves_absolute_reference() {
    let engine = Engine::sheets();
    assert_eq!(engine.translate_formula("=$A$1", 5, 5), Ok("=$A$1".to_string()));
}

#[test]
fn translate_formula_excel_flavor_is_unsupported() {
    let engine = Engine::excel();
    assert!(engine.translate_formula("=A1", 1, 1).is_err());
}

#[test]
fn translate_formula_propagates_parse_error() {
    let engine = Engine::sheets();
    assert!(engine.translate_formula("=SUM(", 0, 0).is_err());
}
