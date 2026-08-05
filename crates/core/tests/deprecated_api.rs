//! The deprecated engine-less free functions must keep working (with a
//! deprecation warning at compile time) until they are removed.
//! See ADR 2026-04-27-engine-flavor-explicit-everywhere.
#![allow(deprecated)]

use std::collections::HashMap;
use truecalc_core::{Engine, Value};

#[test]
fn free_evaluate_still_works() {
    let result = truecalc_core::evaluate("=SUM(1,2)", &HashMap::new());
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn free_evaluate_matches_sheets_engine() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), Value::Number(4.0));
    let free = truecalc_core::evaluate("=x*2+1", &vars);
    let engine = Engine::sheets().evaluate("=x*2+1", &vars);
    assert_eq!(free, engine);
}

#[test]
fn free_parse_still_works() {
    assert!(truecalc_core::parse("=1+2").is_ok());
    assert!(truecalc_core::parse("=1+").is_err());
}

#[test]
fn free_validate_still_works() {
    assert!(truecalc_core::validate("=ROUND(1.23, 1)").is_ok());
    assert!(truecalc_core::validate("=ROUND(1.23,").is_err());
}

#[test]
fn engine_google_sheets_alias_still_works() {
    let engine = Engine::google_sheets();
    assert_eq!(engine.evaluate("=SUM(1,2)", &HashMap::new()), Value::Number(3.0));
}
