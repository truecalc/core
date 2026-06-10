use crate::Engine;
use crate::types::Value;
use std::collections::HashMap;

#[test]
fn rand_result_is_finite() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RAND()", &HashMap::new());
    if let Value::Number(n) = result {
        assert!(n.is_finite());
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn randbetween_negative_range() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RANDBETWEEN(-5, -1)", &HashMap::new());
    if let Value::Number(n) = result {
        assert!(n >= -5.0 && n <= -1.0);
        assert_eq!(n, n.floor());
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn randbetween_zero_range() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RANDBETWEEN(0, 0)", &HashMap::new());
    assert_eq!(result, Value::Number(0.0));
}
