use crate::eval::functions::lookup::vlookup::hlookup_fn;
use crate::types::{ErrorKind, Value};

fn make_2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(|r| Value::Array(r)).collect())
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

// HLOOKUP(2, [[1,2,3],["a","b","c"]], 2, false) → "b"
#[test]
fn exact_match_found() {
    let range = make_2d(vec![
        vec![n(1.0), n(2.0), n(3.0)],
        vec![t("a"), t("b"), t("c")],
    ]);
    let result = hlookup_fn(&[n(2.0), range, n(2.0), Value::Bool(false)]);
    assert_eq!(result, t("b"));
}

// HLOOKUP(99, [[1,2,3],["a","b","c"]], 2, false) → #N/A
#[test]
fn exact_match_not_found() {
    let range = make_2d(vec![
        vec![n(1.0), n(2.0), n(3.0)],
        vec![t("a"), t("b"), t("c")],
    ]);
    let result = hlookup_fn(&[n(99.0), range, n(2.0), Value::Bool(false)]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

// HLOOKUP wrong arg count → #N/A
#[test]
fn wrong_arg_count() {
    assert_eq!(hlookup_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(hlookup_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

// HLOOKUP row_index out of range → #REF!
#[test]
fn row_index_out_of_range() {
    let range = make_2d(vec![vec![n(1.0), n(2.0)]]);
    let result = hlookup_fn(&[n(1.0), range, n(5.0), Value::Bool(false)]);
    assert_eq!(result, Value::Error(ErrorKind::Ref));
}
