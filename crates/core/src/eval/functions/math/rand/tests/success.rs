use crate::Engine;
use crate::types::Value;
use std::collections::HashMap;

#[test]
fn rand_returns_number_in_range() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RAND()", &HashMap::new());
    if let Value::Number(n) = result {
        assert!(n >= 0.0 && n < 1.0, "RAND() must be in [0, 1), got {}", n);
    } else {
        panic!("Expected Number from RAND()");
    }
}

#[test]
fn randbetween_returns_number_in_range() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RANDBETWEEN(1, 10)", &HashMap::new());
    if let Value::Number(n) = result {
        assert!(n >= 1.0 && n <= 10.0, "RANDBETWEEN must be in [1,10], got {}", n);
        assert_eq!(n, n.floor(), "RANDBETWEEN must return an integer");
    } else {
        panic!("Expected Number from RANDBETWEEN()");
    }
}

#[test]
fn randbetween_same_low_high() {
    let eng = Engine::sheets();
    let result = eng.evaluate("=RANDBETWEEN(5, 5)", &HashMap::new());
    assert_eq!(result, Value::Number(5.0));
}
