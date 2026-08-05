use super::super::*;
use crate::types::Value;

#[test]
fn skew_known_value() {
    // SKEW(3,4,5,2,3,4,5,6,4,7) ≈ 0.3595
    let data = vec![3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0, 6.0, 4.0, 7.0];
    let args: Vec<Value> = data.into_iter().map(Value::Number).collect();
    let result = skew_fn(&args);
    if let Value::Number(n) = result {
        assert!((n - 0.3595).abs() < 1e-3, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
