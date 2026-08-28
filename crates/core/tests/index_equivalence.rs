//! `INDEX` behaves exactly as it did before it stopped copying its array
//! (issue #894).
//!
//! `index_fn` was rewritten to read one element in place instead of
//! materialising the whole array first. The rewrite has to be *invisible*:
//! `INDEX` has a lot of edge behaviour that is easy to lose — a zero row or
//! column meaning "the whole column/row", out-of-range giving `#REF!`,
//! negative giving `#VALUE!`, non-integer arguments truncating, ragged and
//! mixed arrays, a single-element array, and a whole-row/whole-column result
//! being handed on to another function rather than read as a scalar.
//!
//! So this file keeps the *previous* implementation, verbatim, as an oracle
//! and asserts the two agree on every combination of a matrix of arrays and
//! index arguments. The oracle is the old code, not a restatement of what the
//! old code was believed to do, so it cannot drift into agreeing by
//! construction.

use std::collections::HashMap;

use truecalc_core::eval::functions::check_arity;
use truecalc_core::eval::functions::lookup::array_utils::{flatten_to_flat, flatten_to_rows};
use truecalc_core::eval::functions::lookup::index_match::index_fn;
use truecalc_core::{Engine, ErrorKind, Value};

// ───────────────────────────── the oracle ─────────────────────────────────
// Verbatim copy of `index_fn` (and its `coerce_index` helper) as they stood
// before the rewrite. Do not "tidy" this; its value is that it is the old
// code.

fn coerce_index(v: &Value) -> Result<i64, Value> {
    match v {
        Value::Number(n) => Ok(n.trunc() as i64),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        Value::Error(e) => Err(Value::Error(e.clone())),
        Value::ErrorMsg(e, m) => Err(Value::ErrorMsg(e.clone(), m.clone())),
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}

fn index_fn_before(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let array_val = &args[0];

    let row_idx_raw = match coerce_index(&args[1]) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if row_idx_raw < 0 {
        return Value::Error(ErrorKind::Value);
    }
    let row_idx = row_idx_raw as usize;

    let col_idx = if args.len() == 3 {
        let col_raw = match coerce_index(&args[2]) {
            Ok(n) => n,
            Err(e) => return e,
        };
        if col_raw < 0 {
            return Value::Error(ErrorKind::Value);
        }
        col_raw as usize
    } else {
        0
    };

    let rows = flatten_to_rows(array_val);
    let is_2d =
        matches!(array_val, Value::Array(v) if v.iter().any(|e| matches!(e, Value::Array(_))));

    if is_2d {
        if row_idx == 0 && col_idx == 0 {
            return array_val.clone();
        }
        if row_idx > rows.len() {
            return Value::Error(ErrorKind::Ref);
        }
        if row_idx == 0 {
            let col = col_idx;
            if col < 1 || rows.iter().any(|r| col > r.len()) {
                return Value::Error(ErrorKind::Ref);
            }
            let col_vals: Vec<Value> = rows.iter().map(|r| r[col - 1].clone()).collect();
            return Value::Array(col_vals);
        }
        let row = &rows[row_idx - 1];
        if col_idx == 0 {
            return Value::Array(row.clone());
        }
        if col_idx > row.len() {
            return Value::Error(ErrorKind::Ref);
        }
        row[col_idx - 1].clone()
    } else {
        let flat = flatten_to_flat(array_val);
        if col_idx == 0 {
            if row_idx == 0 {
                return flat
                    .first()
                    .cloned()
                    .unwrap_or(Value::Error(ErrorKind::Ref));
            }
            if row_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[row_idx - 1].clone()
        } else if row_idx == 0 || row_idx == 1 {
            if col_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[col_idx - 1].clone()
        } else if col_idx == 1 {
            if row_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[row_idx - 1].clone()
        } else {
            Value::Error(ErrorKind::Ref)
        }
    }
}

// ──────────────────────────── the case matrix ─────────────────────────────

fn num(n: f64) -> Value {
    Value::Number(n)
}

fn arrays() -> Vec<(&'static str, Value)> {
    vec![
        ("empty array", Value::Array(vec![])),
        ("single element", Value::Array(vec![num(7.0)])),
        ("scalar, not an array", num(7.0)),
        ("scalar text, not an array", Value::Text("solo".into())),
        (
            "row vector",
            Value::Array(vec![num(1.0), num(2.0), num(3.0)]),
        ),
        (
            "row vector, mixed types",
            Value::Array(vec![
                num(1.0),
                Value::Text("two".into()),
                Value::Bool(true),
                Value::Empty,
                Value::Error(ErrorKind::DivByZero),
            ]),
        ),
        (
            "column vector",
            Value::Array(vec![
                Value::Array(vec![num(1.0)]),
                Value::Array(vec![num(2.0)]),
                Value::Array(vec![num(3.0)]),
            ]),
        ),
        (
            "single-row grid",
            Value::Array(vec![Value::Array(vec![num(1.0), num(2.0)])]),
        ),
        (
            "2x3 grid",
            Value::Array(vec![
                Value::Array(vec![num(11.0), num(12.0), num(13.0)]),
                Value::Array(vec![num(21.0), num(22.0), num(23.0)]),
            ]),
        ),
        (
            "ragged grid, short row first",
            Value::Array(vec![
                Value::Array(vec![num(11.0)]),
                Value::Array(vec![num(21.0), num(22.0), num(23.0)]),
            ]),
        ),
        (
            "ragged grid, long row first",
            Value::Array(vec![
                Value::Array(vec![num(11.0), num(12.0), num(13.0)]),
                Value::Array(vec![num(21.0)]),
            ]),
        ),
        (
            "ragged grid with an empty row",
            Value::Array(vec![
                Value::Array(vec![]),
                Value::Array(vec![num(21.0), num(22.0)]),
            ]),
        ),
        (
            "mixed: a scalar and a nested array side by side",
            Value::Array(vec![
                num(1.0),
                num(2.0),
                Value::Array(vec![num(31.0), num(32.0)]),
            ]),
        ),
        (
            "mixed: a nested array first",
            Value::Array(vec![Value::Array(vec![num(11.0), num(12.0)]), num(2.0)]),
        ),
        (
            "doubly nested, as SEQUENCE() inside a literal produces",
            Value::Array(vec![
                num(1.0),
                Value::Array(vec![
                    Value::Array(vec![num(21.0)]),
                    Value::Array(vec![num(22.0)]),
                ]),
            ]),
        ),
    ]
}

/// Every index argument worth trying: in range, out of range at both ends,
/// zero (meaning "all"), negative, non-integer, and non-numeric.
fn index_args() -> Vec<(&'static str, Value)> {
    vec![
        ("0", num(0.0)),
        ("1", num(1.0)),
        ("2", num(2.0)),
        ("3", num(3.0)),
        ("4 (past the end of most cases)", num(4.0)),
        ("99 (far past the end)", num(99.0)),
        ("-1", num(-1.0)),
        ("-0.5 (truncates toward zero)", num(-0.5)),
        ("1.9 (truncates to 1)", num(1.9)),
        ("2.5 (truncates to 2)", num(2.5)),
        ("TRUE", Value::Bool(true)),
        ("FALSE", Value::Bool(false)),
        ("text", Value::Text("2".into())),
        ("empty", Value::Empty),
        ("an error", Value::Error(ErrorKind::NA)),
        (
            "an error with a message",
            Value::ErrorMsg(ErrorKind::Value, "boom".into()),
        ),
    ]
}

#[test]
fn rewritten_index_matches_the_previous_implementation() {
    let mut compared = 0usize;

    for (array_label, array) in arrays() {
        for (row_label, row) in index_args() {
            // Two-argument form: INDEX(array, row)
            let two = vec![array.clone(), row.clone()];
            assert_eq!(
                index_fn(&two),
                index_fn_before(&two),
                "INDEX({array_label}, {row_label}) diverged from the previous implementation",
            );
            compared += 1;

            // Three-argument form: INDEX(array, row, col)
            for (col_label, col) in index_args() {
                let three = vec![array.clone(), row.clone(), col.clone()];
                assert_eq!(
                    index_fn(&three),
                    index_fn_before(&three),
                    "INDEX({array_label}, {row_label}, {col_label}) diverged from the \
                     previous implementation",
                );
                compared += 1;
            }
        }
    }

    println!("compared {compared} INDEX argument combinations against the previous implementation");
    assert!(compared > 3_000, "the matrix collapsed to {compared} cases");
}

#[test]
fn arity_errors_match_the_previous_implementation() {
    let array = Value::Array(vec![num(1.0), num(2.0)]);
    let cases: Vec<Vec<Value>> = vec![
        vec![],
        vec![array.clone()],
        vec![array.clone(), num(1.0), num(1.0), num(1.0)],
        vec![array, num(1.0), num(1.0), num(1.0), num(1.0)],
    ];
    for args in cases {
        assert_eq!(
            index_fn(&args),
            index_fn_before(&args),
            "arity case with {} argument(s) diverged",
            args.len(),
        );
    }
}

/// A large array must give the same answers as a small one — the rewrite
/// reads through a borrowed slice, and a bound that is right at size 3 and
/// wrong at size 10,000 would otherwise slip through the matrix above.
#[test]
fn a_large_array_matches_the_previous_implementation() {
    let big_row = Value::Array((1..=10_000).map(|i| num(i as f64)).collect());
    let big_grid = Value::Array(
        (1..=3_000)
            .map(|r| Value::Array((1..=3).map(|c| num((r * 10 + c) as f64)).collect()))
            .collect(),
    );

    for probe in [0i64, 1, 2, 4_999, 9_999, 10_000, 10_001, 20_000, -1] {
        let args = vec![big_row.clone(), num(probe as f64)];
        assert_eq!(
            index_fn(&args),
            index_fn_before(&args),
            "big row at {probe}"
        );
    }

    for row in [0i64, 1, 1_500, 3_000, 3_001, -1] {
        for col in [0i64, 1, 3, 4, -1] {
            let args = vec![big_grid.clone(), num(row as f64), num(col as f64)];
            assert_eq!(
                index_fn(&args),
                index_fn_before(&args),
                "big grid at ({row}, {col})",
            );
        }
    }
}

/// `INDEX` used as a *reference* rather than a value: a zero row or column
/// yields the whole column or row, which is then consumed by another
/// function. Checked through the engine so the surrounding evaluation path —
/// argument coercion, array broadcasting, error propagation — is exercised
/// too, and compared against the same formula written without `INDEX`.
#[test]
fn index_as_a_reference_still_feeds_other_functions() {
    let engine = Engine::sheets();
    let vars: HashMap<String, Value> = HashMap::new();

    // (formula using INDEX as a reference, an equivalent formula that does not)
    let pairs = [
        ("=SUM(INDEX({1,2,3;4,5,6},0,2))", "=SUM({2;5})"),
        ("=SUM(INDEX({1,2,3;4,5,6},2,0))", "=SUM({4,5,6})"),
        ("=COUNT(INDEX({1,2,3;4,5,6},0,0))", "=COUNT({1,2,3;4,5,6})"),
        ("=SUM(INDEX({1,2,3;4,5,6},0,1))", "=SUM({1;4})"),
        ("=MAX(INDEX({1,2,3;4,5,6},1,0))", "=MAX({1,2,3})"),
        ("=SUM(INDEX({1,2,3;4,5,6},0,2)*10)", "=SUM({2;5}*10)"),
        (
            "=INDEX(INDEX({1,2,3;4,5,6},2,0),1,3)",
            "=INDEX({4,5,6},1,3)",
        ),
        ("=SUM(INDEX({1;2;3},0))", "=SUM({1;2;3})"),
        (
            "=SUMPRODUCT(INDEX({1,2;3,4},0,1),{10;20})",
            "=SUMPRODUCT({1;3},{10;20})",
        ),
    ];

    for (with_index, without_index) in pairs {
        let got = engine.evaluate(with_index, &vars);
        let expected = engine.evaluate(without_index, &vars);
        assert_eq!(
            got, expected,
            "{with_index} should agree with {without_index}"
        );
        assert!(
            !matches!(got, Value::Error(_) | Value::ErrorMsg(_, _)),
            "{with_index} evaluated to {got:?}; the pair would agree vacuously on two errors",
        );
    }
}
