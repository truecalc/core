//! SEQUENCE array-orientation tests (truecalc/core#707).
//!
//! A result whose column dimension is 1 must be an N-row × 1-column column
//! vector (`[[1],[2],[3]]`), matching Google Sheets — not a 1×N row. The
//! already-correct 1×N and M×N shapes are locked here too so they cannot
//! silently regress.

use crate::evaluate;
use crate::types::Value;
use std::collections::HashMap;

fn ev(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn num(n: f64) -> Value {
    Value::Number(n)
}

fn row(vals: &[f64]) -> Value {
    Value::Array(vals.iter().copied().map(num).collect())
}

/// Nested 2-D array from rows of values.
fn grid(rows: &[&[f64]]) -> Value {
    Value::Array(rows.iter().map(|r| row(r)).collect())
}

// ── Fixed: N×1 column vectors keep their orientation ──────────────────────────

#[test]
fn sequence_n_is_column_vector() {
    // =SEQUENCE(3) → [[1],[2],[3]] (3 rows × 1 col)
    assert_eq!(ev("=SEQUENCE(3)"), grid(&[&[1.0], &[2.0], &[3.0]]));
}

#[test]
fn sequence_n_1_is_column_vector() {
    // =SEQUENCE(3,1) → [[1],[2],[3]] — the smoking gun: rows=3, cols=1.
    assert_eq!(ev("=SEQUENCE(3,1)"), grid(&[&[1.0], &[2.0], &[3.0]]));
}

#[test]
fn sequence_column_vector_with_start_and_step() {
    // =SEQUENCE(3,1,5,2) → [[5],[7],[9]]
    assert_eq!(ev("=SEQUENCE(3,1,5,2)"), grid(&[&[5.0], &[7.0], &[9.0]]));
}

// ── Locked: already-correct 1×N and M×N shapes are unchanged ──────────────────

#[test]
fn sequence_1_n_stays_single_row() {
    // =SEQUENCE(1,3) → [[1,2,3]] (1 row × 3 cols)
    assert_eq!(ev("=SEQUENCE(1,3)"), grid(&[&[1.0, 2.0, 3.0]]));
}

#[test]
fn sequence_m_n_stays_2d() {
    // =SEQUENCE(2,3) → [[1,2,3],[4,5,6]] (2 rows × 3 cols, row-major)
    assert_eq!(
        ev("=SEQUENCE(2,3)"),
        grid(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]])
    );
}

#[test]
fn sequence_m_n_row_major_fill_with_step() {
    // =SEQUENCE(2,2,10,-1) → [[10,9],[8,7]] (row-major fill preserved)
    assert_eq!(
        ev("=SEQUENCE(2,2,10,-1)"),
        grid(&[&[10.0, 9.0], &[8.0, 7.0]])
    );
}

// ── 1×1 ───────────────────────────────────────────────────────────────────────

#[test]
fn sequence_1_is_single_cell() {
    // =SEQUENCE(1) → [[1]] (1 row × 1 col). A single-cell view collapses to 1.
    assert_eq!(ev("=SEQUENCE(1)"), grid(&[&[1.0]]));
}
