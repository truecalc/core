use crate::eval::functions::lookup::vlookup::vlookup_fn;
use crate::types::{ErrorKind, Value};

/// Build a 2D array: rows x cols, outer=rows, inner=cols.
fn make_2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(|r| Value::Array(r)).collect())
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

// VLOOKUP(2, [[1,"a"],[2,"b"],[3,"c"]], 2, false) → "b"
#[test]
fn exact_match_found() {
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(2.0), t("b")],
        vec![n(3.0), t("c")],
    ]);
    let result = vlookup_fn(&[n(2.0), range, n(2.0), Value::Bool(false)]);
    assert_eq!(result, t("b"));
}

// VLOOKUP(99, [[1,"a"],[2,"b"]], 2, false) → #N/A
#[test]
fn exact_match_not_found() {
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(2.0), t("b")],
    ]);
    let result = vlookup_fn(&[n(99.0), range, n(2.0), Value::Bool(false)]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

// VLOOKUP with wrong arg count → #N/A
#[test]
fn wrong_arg_count() {
    assert_eq!(vlookup_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(vlookup_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

// VLOOKUP first column match returns first column value itself
#[test]
fn exact_match_col_1() {
    let range = make_2d(vec![
        vec![n(5.0), t("x")],
        vec![n(10.0), t("y")],
    ]);
    let result = vlookup_fn(&[n(10.0), range, n(1.0), Value::Bool(false)]);
    assert_eq!(result, n(10.0));
}

// VLOOKUP approximate match (is_sorted=true): largest <= search_key
#[test]
fn approximate_match() {
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(3.0), t("b")],
        vec![n(5.0), t("c")],
    ]);
    // Search 4 → largest <= 4 is 3 → "b"
    let result = vlookup_fn(&[n(4.0), range, n(2.0), Value::Bool(true)]);
    assert_eq!(result, t("b"));
}

// VLOOKUP col_index out of range → #REF!
#[test]
fn col_index_out_of_range() {
    let range = make_2d(vec![vec![n(1.0), t("a")]]);
    let result = vlookup_fn(&[n(1.0), range, n(5.0), Value::Bool(false)]);
    assert_eq!(result, Value::Error(ErrorKind::Ref));
}
