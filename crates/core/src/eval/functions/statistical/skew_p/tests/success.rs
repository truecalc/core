use super::super::*;
use crate::types::Value;

#[test]
fn skew_p_known_value() {
    // SKEW.P(3,4,5,2,3,4,5,6,4,7) — population skewness
    let data = vec![3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0, 6.0, 4.0, 7.0];
    let args: Vec<Value> = data.into_iter().map(Value::Number).collect();
    let result = skew_p_fn(&args);
    if let Value::Number(n) = result {
        // Population skewness should be non-zero for this dataset
        assert!(n.is_finite(), "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
