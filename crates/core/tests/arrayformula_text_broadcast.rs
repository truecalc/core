// crates/core/tests/arrayformula_text_broadcast.rs
//
// Regression coverage for ARRAYFORMULA broadcasting of SEARCH, FIND and
// EXACT (issue #875). These three two/three-argument text functions were
// missing from the explicit allow-list in
// `crates/core/src/eval/functions/array/mod.rs::broadcast_expr`, so under
// ARRAYFORMULA they fell through to the plain scalar evaluation path and
// hit `#VALUE!` as soon as an array argument reached `to_string_val`.
//
// IMPORTANT: none of the array-shaped expected values asserted here are
// verified against live Google Sheets (no fixture coverage exists for
// these formulas in `tests/fixtures/google_sheets/`). They are derived
// from — and only assert internal consistency with — the engine's own
// established broadcasting convention already exercised by LEN/UPPER/IF
// in that same fixture set:
//   - a scalar argument is reused unchanged at every broadcast position
//     (see `broadcast_eager`'s `None => v.clone()` arm, and `broadcast_if`
//     doing the same for a non-array branch value);
//   - a per-position failure (e.g. `SEARCH` not finding its text) lands
//     as an error *inside* the result array at that position, rather than
//     collapsing the whole ARRAYFORMULA result to a single error (see
//     `broadcast_eager`'s `row.push(f(&per_pos))`).
// This must be re-verified against Google Sheets before being treated as
// ground truth.

mod helpers;
use helpers::eval;
use truecalc_core::{ErrorKind, Value};

fn arr(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}

// ── SEARCH ────────────────────────────────────────────────────────────────

// Array in the `within_text` (2nd) position, scalar `find_text`.
// "apple" contains "a" at position 1; "berry" does not contain "a" at all,
// so per the per-position-error convention that position holds #VALUE!
// rather than collapsing the whole result to an error.
#[test]
fn search_broadcasts_over_within_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(SEARCH("a", {"apple","berry"}))"#),
        arr(vec![Value::Number(1.0), Value::Error(ErrorKind::Value)])
    );
}

// Array in the `find_text` (1st) position, scalar `within_text`.
#[test]
fn search_broadcasts_over_find_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(SEARCH({"a","b"}, "abc"))"#),
        arr(vec![Value::Number(1.0), Value::Number(2.0)])
    );
}

// Arrays in both positions (same shape).
#[test]
fn search_broadcasts_over_both_arrays() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(SEARCH({"a","b"}, {"cat","bat"}))"#),
        arr(vec![Value::Number(2.0), Value::Number(1.0)])
    );
}

// A plain, non-ARRAYFORMULA call is untouched by this change — it keeps
// implicit-intersection/scalar-collapse behavior. (No fixture exists for
// this out-of-ARRAYFORMULA shape either; asserting the pre-existing
// scalar-argument #VALUE! behavior that already held before this fix.)
#[test]
fn search_without_arrayformula_still_errors_on_array_arg() {
    assert!(matches!(
        eval(r#"=SEARCH("a", {"apple","berry"})"#),
        Value::Error(ErrorKind::Value)
    ));
}

// ── FIND ──────────────────────────────────────────────────────────────────

// Array in the `within_text` (2nd) position. FIND is case-sensitive:
// "cat" contains "a" at position 2; "dog" does not contain "a".
#[test]
fn find_broadcasts_over_within_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(FIND("a", {"cat","dog"}))"#),
        arr(vec![Value::Number(2.0), Value::Error(ErrorKind::Value)])
    );
}

// Array in the `find_text` (1st) position.
#[test]
fn find_broadcasts_over_find_text_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(FIND({"c","d"}, "cat dog"))"#),
        arr(vec![Value::Number(1.0), Value::Number(5.0)])
    );
}

// ── EXACT ─────────────────────────────────────────────────────────────────

// Array in the 1st position, scalar 2nd argument.
#[test]
fn exact_broadcasts_over_first_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(EXACT({"a","b"}, "a"))"#),
        arr(vec![Value::Bool(true), Value::Bool(false)])
    );
}

// Array in the 2nd position, scalar 1st argument.
#[test]
fn exact_broadcasts_over_second_array() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(EXACT("a", {"a","b"}))"#),
        arr(vec![Value::Bool(true), Value::Bool(false)])
    );
}

// Arrays in both positions.
#[test]
fn exact_broadcasts_over_both_arrays() {
    assert_eq!(
        eval(r#"=ARRAYFORMULA(EXACT({"a","b"}, {"a","c"}))"#),
        arr(vec![Value::Bool(true), Value::Bool(false)])
    );
}
