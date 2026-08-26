// crates/core/tests/arrayformula_searchb_findb_broadcast.rs
//
// Regression coverage for ARRAYFORMULA broadcasting of SEARCHB and FINDB
// (issue #879, the same defect as #875/#877 in the same allow-list). These
// byte-position variants of SEARCH/FIND were missing from the explicit
// allow-list in `crates/core/src/eval/functions/array/mod.rs::broadcast_expr`,
// so under ARRAYFORMULA they fell through to the plain scalar evaluation
// path and hit `#VALUE!` as soon as an array argument reached
// `to_string_val`.
//
// IMPORTANT: none of the array-shaped expected values asserted here are
// verified against live Google Sheets (no fixture coverage exists for
// these formulas in `tests/fixtures/google_sheets/`, and the scalar cases
// used as ARRAYFORMULA inputs below are themselves not oracle-confirmed
// beyond what's already exercised by SEARCHB/FINDB's own unit tests). They
// are derived from — and only assert internal consistency with — the
// engine's own established broadcasting convention already exercised by
// LEN/UPPER/SEARCH/FIND/EXACT in that same fixture set and in
// `arrayformula_text_broadcast.rs`:
//   - a scalar argument is reused unchanged at every broadcast position
//     (see `broadcast_eager`'s `None => v.clone()` arm);
//   - a per-position failure (e.g. SEARCHB/FINDB not finding its text)
//     lands as an error *inside* the result array at that position, rather
//     than collapsing the whole ARRAYFORMULA result to a single error (see
//     `broadcast_eager`'s `row.push(f(&per_pos))`).
// This must be re-verified against Google Sheets before being treated as
// ground truth.

mod helpers;
use helpers::eval;
use truecalc_core::{ErrorKind, Value};

fn arr(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}

// ── SEARCHB ──────────────────────────────────────────────────────────────

// Array in the `within_text` (2nd) position, scalar `find_text`.
// "apple" contains "a" at byte position 1; "berry" does not contain "a" at
// all, so per the per-position-error convention that position holds
// #VALUE! rather than collapsing the whole result to an error.
#[test]
fn searchb_broadcasts_over_within_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(SEARCHB("a", {"apple","berry"}))"#),
        arr(vec![Value::Number(1.0), Value::Error(ErrorKind::Value)])
    );
}

// Array in the `find_text` (1st) position, scalar `within_text`.
#[test]
fn searchb_broadcasts_over_find_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(SEARCHB({"a","b"}, "abc"))"#),
        arr(vec![Value::Number(1.0), Value::Number(2.0)])
    );
}

// Arrays in both positions (same shape).
#[test]
fn searchb_broadcasts_over_both_arrays() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(SEARCHB({"a","b"}, {"cat","bat"}))"#),
        arr(vec![Value::Number(2.0), Value::Number(1.0)])
    );
}

// A plain, non-ARRAYFORMULA call is untouched by this change — it keeps
// implicit-intersection/scalar-collapse behavior. (No fixture exists for
// this out-of-ARRAYFORMULA shape either; asserting the pre-existing
// scalar-argument #VALUE! behavior that already held before this fix.)
#[test]
fn searchb_without_arrayformula_still_errors_on_array_arg() {
    assert!(matches!(
        eval(r#"=SEARCHB("a", {"apple","berry"})"#),
        Value::Error(ErrorKind::Value)
    ));
}

// ── FINDB ────────────────────────────────────────────────────────────────

// Array in the `within_text` (2nd) position. FINDB is case-sensitive:
// "cat" contains "a" at byte position 2; "dog" does not contain "a".
#[test]
fn findb_broadcasts_over_within_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(FINDB("a", {"cat","dog"}))"#),
        arr(vec![Value::Number(2.0), Value::Error(ErrorKind::Value)])
    );
}

// Array in the `find_text` (1st) position.
#[test]
fn findb_broadcasts_over_find_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(FINDB({"c","d"}, "cat dog"))"#),
        arr(vec![Value::Number(1.0), Value::Number(5.0)])
    );
}
