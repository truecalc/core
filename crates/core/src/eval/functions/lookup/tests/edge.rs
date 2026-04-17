use crate::eval::functions::lookup::index_match::match_fn;
use crate::types::Value;

fn make_1d(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

// MATCH boundary cases
#[test]
fn match_approx_between_values() {
    // 2.5 → largest <= 2.5 is 2, at position 2
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(2.5), arr, n(1.0)]), n(2.0));
}

#[test]
fn match_first_element() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(1.0), arr, n(0.0)]), n(1.0));
}

#[test]
fn match_last_element() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(3.0), arr, n(0.0)]), n(3.0));
}
