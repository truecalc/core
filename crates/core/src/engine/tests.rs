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

// ── evaluate_with_resolver_at_keyed_hooked (issue #743) ─────────────────────

/// A resolver reading a single flat map keyed by canonical reference text —
/// same shape as the [`crate::Resolver`] doctest's `MapResolver`.
struct MapResolver(HashMap<String, Value>);

impl crate::Resolver for MapResolver {
    fn resolve(&mut self, r: &crate::Ref) -> Value {
        self.0.get(&r.to_string()).cloned().unwrap_or(Value::Empty)
    }
}

fn a1_resolver() -> MapResolver {
    let mut cells = HashMap::new();
    cells.insert("A1".to_string(), Value::Number(5.0));
    MapResolver(cells)
}

#[test]
fn hooked_keyed_resolver_eval_with_none_hook_matches_unhooked_method() {
    // `hook: None` on the `_hooked` method must be indistinguishable from
    // calling the pre-existing, non-hooked method: same formula, same
    // resolver contents, same pinned clock/RNG key ⇒ same value.
    let engine = Engine::sheets();
    let unhooked = engine.evaluate_with_resolver_at_keyed(
        "=A1*2",
        &mut a1_resolver(),
        Some(45_000.0),
        Some(0),
        Some((0, 0, 0, 0)),
    );
    let hooked_none = engine.evaluate_with_resolver_at_keyed_hooked(
        "=A1*2",
        &mut a1_resolver(),
        Some(45_000.0),
        Some(0),
        Some((0, 0, 0, 0)),
        None,
    );
    assert_eq!(unhooked, Value::Number(10.0));
    assert_eq!(hooked_none, unhooked);
}

#[test]
fn hooked_keyed_resolver_eval_with_hook_attached_is_same_value_and_fires() {
    // Attaching a hook must not change the computed value versus `None`
    // (issue #743 additive invariant), and the hook must actually observe
    // the reference-precedent read.
    let engine = Engine::sheets();
    let mut seen: Vec<Value> = Vec::new();
    let mut hook = |_op: crate::eval::EvalOp<'_>, _span: crate::Span, value: &Value| {
        seen.push(value.clone());
    };
    let hooked = engine.evaluate_with_resolver_at_keyed_hooked(
        "=A1*2",
        &mut a1_resolver(),
        Some(45_000.0),
        Some(0),
        Some((0, 0, 0, 0)),
        Some(&mut hook),
    );
    assert_eq!(hooked, Value::Number(10.0));
    // Post-order: the A1 reference (value 5) fires before the top-level
    // multiplication (value 10, matching the returned result).
    assert!(seen.contains(&Value::Number(5.0)));
    assert_eq!(seen.last(), Some(&Value::Number(10.0)));
}
