use super::super::*;
use crate::types::Value;

#[test]
fn result_is_reasonable_upper_bound() {
    // Sanity: the result should not be unreasonably large (e.g., > serial for 2100-01-01 ≈ 73051).
    if let Value::Number(n) = today_fn(&[]) {
        assert!(n < 73051.0, "today serial {n} seems unreasonably large");
    } else {
        panic!("expected Number");
    }
}
