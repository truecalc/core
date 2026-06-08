mod helpers;

/// Top-left (anchor-cell) element of a possibly-array result.
///
/// P1.4 (#526): the engine returns array results unspilled.  These tests
/// were verified against the Google Sheets anchor cell (the top-left
/// element of the spilled range), which is what the workbook/surface
/// layer's single-cell view shows -- so they assert on the top-left
/// element of the unspilled array.
fn top_left(v: Value) -> Value {
    match v {
        Value::Array(items) if !items.is_empty() => {
            top_left(items.into_iter().next().unwrap())
        }
        other => other,
    }
}
use truecalc_core::{Value, ErrorKind};
use helpers::{eval, eval_with};

// 1. Basic arithmetic
#[test]
fn arithmetic_add() {
    assert_eq!(eval("=1+2"), Value::Number(3.0));
}

// 2. Variable resolution
#[test]
fn variable_resolution() {
    assert_eq!(eval_with("=A1+B1", [("A1", Value::Number(5.0)), ("B1", Value::Number(3.0))]),
               Value::Number(8.0));
}

// 3. SUM function
#[test]
fn sum_function() {
    assert_eq!(eval("=SUM(1,2,3)"), Value::Number(6.0));
}

// 4. IF function — lazy evaluation
#[test]
fn if_function_true_branch() {
    assert_eq!(eval("=IF(TRUE,\"yes\",\"no\")"), Value::Text("yes".to_string()));
}

// 5. IF function — false branch
#[test]
fn if_function_false_branch() {
    assert_eq!(eval("=IF(FALSE,\"yes\",\"no\")"), Value::Text("no".to_string()));
}

// 6. Division by zero
#[test]
fn division_by_zero() {
    assert_eq!(eval("=1/0"), Value::Error(ErrorKind::DivByZero));
}

// 7. Unknown variable returns Empty
#[test]
fn unknown_variable_returns_empty() {
    assert_eq!(eval("=Z99"), Value::Empty);
}

// 8. Unknown function returns #NAME?
#[test]
fn unknown_function_returns_name_error() {
    assert_eq!(eval("=NONEXISTENT(1)"), Value::Error(ErrorKind::Name));
}

// 9. String concatenation
#[test]
fn string_concatenation() {
    assert_eq!(eval("=\"hello\"&\" world\""), Value::Text("hello world".to_string()));
}

// 10. Comparison — equal
#[test]
fn comparison_equal() {
    assert_eq!(eval("=1=1"), Value::Bool(true));
    assert_eq!(eval("=1=2"), Value::Bool(false));
}

// 11. Percent operator
#[test]
fn percent_operator() {
    assert_eq!(eval("=50%"), Value::Number(0.5));
}

// 12. Nested function call: IF(SUM(0,0)=0, "empty", "not")
#[test]
fn nested_if_with_sum() {
    assert_eq!(eval("=IF(SUM(0,0)=0,\"empty\",\"not\")"), Value::Text("empty".to_string()));
}

// 13. Parse error returns #VALUE!
#[test]
fn invalid_formula_returns_value_error() {
    assert_eq!(eval("=("), Value::Error(ErrorKind::Value));
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn display_number_parses_back(n in -1e9f64..=1e9f64) {
        // display_number should produce a string that, when parsed as f64,
        // round-trips with relative tolerance 1e-9 and absolute floor 1e-12
        // (consistent with display_number's 14 significant digits of precision).
        let s = truecalc_core::display_number(n);
        // If it is a valid display string (not "#NUM!"), it should parse back.
        if !s.starts_with('#') {
            let parsed: f64 = s.parse().expect("display_number output should be parseable");
            prop_assert!((parsed - n).abs() <= n.abs() * 1e-9 + 1e-12,
                "round-trip failed for n={n}, displayed as {s:?}, parsed back as {parsed}");
        }
    }
}

#[test]
fn display_number_nan_returns_num_error() {
    assert_eq!(truecalc_core::display_number(f64::NAN), "#NUM!");
}

#[test]
fn display_number_infinity_returns_num_error() {
    assert_eq!(truecalc_core::display_number(f64::INFINITY), "#NUM!");
}

// Temporary debug test for UNIQUE
#[test]
fn debug_unique_counta() {
    let result = helpers::eval("=UNIQUE({1,2,2,3,3,3},FALSE,FALSE)");
    println!("UNIQUE result: {:?}", result);
    
    let result2 = helpers::eval("=COUNTA(UNIQUE({1,2,2,3,3,3},FALSE,FALSE))");
    println!("COUNTA(UNIQUE) result: {:?}", result2);
}

#[test]
fn debug_counta_array() {
    // Test COUNTA with a direct array
    let result = helpers::eval("=COUNTA({1,2,2,3,3,3})");
    println!("COUNTA direct array result: {:?}", result);
}

// ── CHOOSECOLS ────────────────────────────────────────────────────────────────

#[test]
fn choosecols_select_single_column() {
    // anchor-cell (top-left) view of [[2],[5]] -> 2
    assert_eq!(top_left(helpers::eval("=CHOOSECOLS({1,2,3;4,5,6},2)")), Value::Number(2.0));
}

#[test]
fn choosecols_select_multiple_columns() {
    // anchor-cell (top-left) view of [[1,3],[4,6]] -> 1
    assert_eq!(top_left(helpers::eval("=CHOOSECOLS({1,2,3;4,5,6},1,3)")), Value::Number(1.0));
}

#[test]
fn choosecols_negative_index_from_end() {
    // anchor-cell (top-left) view of [[3],[6]] -> 3
    assert_eq!(top_left(helpers::eval("=CHOOSECOLS({1,2,3;4,5,6},-1)")), Value::Number(3.0));
}

#[test]
fn choosecols_zero_index_returns_value_error() {
    assert_eq!(helpers::eval("=CHOOSECOLS({1,2,3},0)"), Value::Error(ErrorKind::Value));
}

#[test]
fn choosecols_out_of_bounds_returns_value_error() {
    assert_eq!(helpers::eval("=CHOOSECOLS({1,2,3},5)"), Value::Error(ErrorKind::Value));
}

// ── CHOOSEROWS ────────────────────────────────────────────────────────────────

#[test]
fn chooserows_select_single_row() {
    // anchor-cell (top-left) view of [3,4] -> 3
    assert_eq!(top_left(helpers::eval("=CHOOSEROWS({1,2;3,4;5,6},2)")), Value::Number(3.0));
}

#[test]
fn chooserows_select_multiple_rows_reorder() {
    // anchor-cell (top-left) view of [[5,6],[1,2]] -> 5
    assert_eq!(top_left(helpers::eval("=CHOOSEROWS({1,2;3,4;5,6},3,1)")), Value::Number(5.0));
}

#[test]
fn chooserows_negative_index_from_end() {
    // anchor-cell (top-left) view of [5,6] -> 5
    assert_eq!(top_left(helpers::eval("=CHOOSEROWS({1,2;3,4;5,6},-1)")), Value::Number(5.0));
}

#[test]
fn chooserows_zero_index_returns_value_error() {
    assert_eq!(helpers::eval("=CHOOSEROWS({1,2,3},0)"), Value::Error(ErrorKind::Value));
}

#[test]
fn chooserows_out_of_bounds_returns_value_error() {
    assert_eq!(helpers::eval("=CHOOSEROWS({1;2;3},5)"), Value::Error(ErrorKind::Value));
}

// ── HSTACK ────────────────────────────────────────────────────────────────────

#[test]
fn hstack_two_column_vectors() {
    // anchor-cell (top-left) view of [[1,3],[2,4]] -> 1
    assert_eq!(top_left(helpers::eval("=HSTACK({1;2},{3;4})")), Value::Number(1.0));
}

#[test]
fn hstack_two_row_arrays() {
    // anchor-cell (top-left) view of [1,2,3,4] -> 1
    assert_eq!(top_left(helpers::eval("=HSTACK({1,2},{3,4})")), Value::Number(1.0));
}

#[test]
fn hstack_three_scalars() {
    // anchor-cell (top-left) view of [1,2,3] -> 1
    assert_eq!(top_left(helpers::eval("=HSTACK({1},{2},{3})")), Value::Number(1.0));
}

#[test]
fn hstack_2d_arrays() {
    // anchor-cell (top-left) view of [[1,2,5],[3,4,6]] -> 1
    assert_eq!(top_left(helpers::eval("=HSTACK({1,2;3,4},{5;6})")), Value::Number(1.0));
}

// ── VSTACK ────────────────────────────────────────────────────────────────────

#[test]
fn vstack_two_row_arrays() {
    // anchor-cell (top-left) view of [[1,2],[3,4]] -> 1
    assert_eq!(top_left(helpers::eval("=VSTACK({1,2},{3,4})")), Value::Number(1.0));
}

#[test]
fn vstack_three_rows() {
    // anchor-cell (top-left) view of [[1,2],[3,4],[5,6]] -> 1
    assert_eq!(top_left(helpers::eval("=VSTACK({1,2},{3,4},{5,6})")), Value::Number(1.0));
}

#[test]
fn vstack_2d_on_top_of_row() {
    // anchor-cell (top-left) view of [[1,2],[3,4],[5,6]] -> 1
    assert_eq!(top_left(helpers::eval("=VSTACK({1,2;3,4},{5,6})")), Value::Number(1.0));
}

// ── TOCOL ─────────────────────────────────────────────────────────────────────

#[test]
fn tocol_flattens_row_to_column() {
    // anchor-cell (top-left) view of [[1],[2],[3]] -> 1
    assert_eq!(top_left(helpers::eval("=TOCOL({1,2,3})")), Value::Number(1.0));
}

#[test]
fn tocol_flattens_2d_array() {
    // anchor-cell (top-left) view of [[1],[2],[3],[4]] -> 1
    assert_eq!(top_left(helpers::eval("=TOCOL({1,2;3,4})")), Value::Number(1.0));
}

#[test]
fn tocol_single_element() {
    // anchor-cell (top-left) view: TOCOL({5}) -> single-element array [5] -> first -> 5
    assert_eq!(top_left(helpers::eval("=TOCOL({5})")), Value::Number(5.0));
}

// ── TOROW ─────────────────────────────────────────────────────────────────────

#[test]
fn torow_flattens_column_to_row() {
    // anchor-cell (top-left) view of [1,2,3] -> 1
    assert_eq!(top_left(helpers::eval("=TOROW({1;2;3})")), Value::Number(1.0));
}

#[test]
fn torow_flattens_2d_array() {
    // anchor-cell (top-left) view of [1,2,3,4] -> 1
    assert_eq!(top_left(helpers::eval("=TOROW({1,2;3,4})")), Value::Number(1.0));
}

#[test]
fn torow_single_element() {
    // anchor-cell (top-left) view: TOROW({7}) -> [7] -> first -> 7
    assert_eq!(top_left(helpers::eval("=TOROW({7})")), Value::Number(7.0));
}

// ── WRAPCOLS ──────────────────────────────────────────────────────────────────

#[test]
fn wrapcols_evenly_divisible() {
    // anchor-cell (top-left) view of [[1,3,5],[2,4,6]] -> 1
    assert_eq!(top_left(helpers::eval("=WRAPCOLS({1,2,3,4,5,6},2)")), Value::Number(1.0));
}

#[test]
fn wrapcols_with_padding() {
    // anchor-cell (top-left) view of [[1,3,5],[2,4,Empty]] -> 1
    assert_eq!(top_left(helpers::eval("=WRAPCOLS({1,2,3,4,5},2)")), Value::Number(1.0));
}

#[test]
fn wrapcols_wrap_count_equals_length() {
    // anchor-cell (top-left) view of [[1],[2],[3]] -> 1
    assert_eq!(top_left(helpers::eval("=WRAPCOLS({1,2,3},3)")), Value::Number(1.0));
}

#[test]
fn wrapcols_invalid_wrap_count_returns_value_error() {
    assert_eq!(helpers::eval("=WRAPCOLS({1,2,3},0)"), Value::Error(ErrorKind::Value));
}

// ── WRAPROWS ──────────────────────────────────────────────────────────────────

#[test]
fn wraprows_evenly_divisible() {
    // anchor-cell (top-left) view of [[1,2,3],[4,5,6]] -> 1
    assert_eq!(top_left(helpers::eval("=WRAPROWS({1,2,3,4,5,6},3)")), Value::Number(1.0));
}

#[test]
fn wraprows_with_padding() {
    // anchor-cell (top-left) view of [[1,2,3],[4,5,Empty]] -> 1
    assert_eq!(top_left(helpers::eval("=WRAPROWS({1,2,3,4,5},3)")), Value::Number(1.0));
}

#[test]
fn wraprows_wrap_count_equals_length() {
    // anchor-cell (top-left) view of [1,2,3] -> 1
    assert_eq!(top_left(helpers::eval("=WRAPROWS({1,2,3},3)")), Value::Number(1.0));
}

#[test]
fn wraprows_invalid_wrap_count_returns_value_error() {
    assert_eq!(helpers::eval("=WRAPROWS({1,2,3},0)"), Value::Error(ErrorKind::Value));
}

// ── SORTBY ────────────────────────────────────────────────────────────────────

#[test]
fn sortby_ascending_by_key() {
    // Sort {10;20;30} by key {3;1;2} ascending -> {20;30;10}; anchor-cell (top-left) view: 20
    assert_eq!(top_left(helpers::eval("=SORTBY({10;20;30},{3;1;2},1)")), Value::Number(20.0));
}

#[test]
fn sortby_descending_by_key() {
    // Sort {10;20;30} by key {3;1;2} descending -> {10;30;20}; anchor-cell (top-left) view: 10
    assert_eq!(top_left(helpers::eval("=SORTBY({10;20;30},{3;1;2},-1)")), Value::Number(10.0));
}

#[test]
fn sortby_default_ascending() {
    // anchor-cell (top-left) view of [[10],[20],[30]] -> 10
    assert_eq!(top_left(helpers::eval("=SORTBY({30;10;20},{3;1;2})")), Value::Number(10.0));
}

#[test]
fn sortby_mismatched_key_length_returns_value_error() {
    assert_eq!(
        helpers::eval("=SORTBY({10;20;30},{1;2})"),
        Value::Error(ErrorKind::Value)
    );
}

// ── MMULT ─────────────────────────────────────────────────────────────────────

#[test]
fn mmult_2x2_identity() {
    // anchor-cell (top-left) view of [[5,6],[7,8]] -> 5
    assert_eq!(top_left(helpers::eval("=MMULT({1,0;0,1},{5,6;7,8})")), Value::Number(5.0));
}

#[test]
fn mmult_2x2_matrices() {
    // {1,2;3,4} * {5,6;7,8} = {19,22;43,50}; anchor-cell (top-left) view: 19
    assert_eq!(top_left(helpers::eval("=MMULT({1,2;3,4},{5,6;7,8})")), Value::Number(19.0));
}

#[test]
fn mmult_1x2_by_2x1() {
    // {1,2} * {3;4} = 11; anchor-cell (top-left) view: [11] -> 11
    assert_eq!(top_left(helpers::eval("=MMULT({1,2},{3;4})")), Value::Number(11.0));
}

#[test]
fn mmult_incompatible_dimensions_returns_value_error() {
    assert_eq!(
        helpers::eval("=MMULT({1,2},{1,2,3})"),
        Value::Error(ErrorKind::Value)
    );
}

// ── MDETERM ───────────────────────────────────────────────────────────────────

#[test]
fn mdeterm_1x1() {
    assert_eq!(helpers::eval("=MDETERM({7})"), Value::Number(7.0));
}

#[test]
fn mdeterm_2x2() {
    // det({1,2;3,4}) = 4 - 6 = -2
    assert_eq!(helpers::eval("=MDETERM({1,2;3,4})"), Value::Number(-2.0));
}

#[test]
fn mdeterm_3x3_singular() {
    // det({1,2,3;4,5,6;7,8,9}) = 0 (linearly dependent rows)
    assert_eq!(helpers::eval("=MDETERM({1,2,3;4,5,6;7,8,9})"), Value::Number(0.0));
}

#[test]
fn mdeterm_non_square_returns_value_error() {
    assert_eq!(helpers::eval("=MDETERM({1,2,3;4,5,6})"), Value::Error(ErrorKind::Value));
}

// ── FREQUENCY ─────────────────────────────────────────────────────────────────

#[test]
fn frequency_basic_bins() {
    // FREQUENCY returns an array; in scalar context Google Sheets returns #REF!
    assert_eq!(helpers::eval("=FREQUENCY({1,2,3,4,5},{2,4})"), Value::Error(ErrorKind::Ref));
}

#[test]
fn frequency_single_bin() {
    // FREQUENCY returns an array; in scalar context Google Sheets returns #REF!
    assert_eq!(helpers::eval("=FREQUENCY({1,2,3},{2})"), Value::Error(ErrorKind::Ref));
}

#[test]
fn frequency_all_below_bin() {
    // FREQUENCY returns an array; in scalar context Google Sheets returns #REF!
    assert_eq!(helpers::eval("=FREQUENCY({1,2},{5})"), Value::Error(ErrorKind::Ref));
}

#[test]
fn frequency_all_above_bin() {
    // FREQUENCY returns an array; in scalar context Google Sheets returns #REF!
    assert_eq!(helpers::eval("=FREQUENCY({6,7,8},{5})"), Value::Error(ErrorKind::Ref));
}

// ── INDEX edge cases ──────────────────────────────────────────────────────────

#[test]
fn index_returns_whole_row() {
    // INDEX({1,2,3;4,5,6}, 1) -> [1,2,3]; anchor-cell (top-left) view: 1
    assert_eq!(top_left(helpers::eval("=INDEX({1,2,3;4,5,6},1)")), Value::Number(1.0));
}

#[test]
fn index_returns_second_row() {
    // anchor-cell (top-left) view of [4,5,6] -> 4
    assert_eq!(top_left(helpers::eval("=INDEX({1,2,3;4,5,6},2)")), Value::Number(4.0));
}

#[test]
fn index_returns_whole_column() {
    // INDEX({1,2;3,4}, 0, 1) -> [[1],[3]]; anchor-cell (top-left) view: 1
    assert_eq!(top_left(helpers::eval("=INDEX({1,2;3,4},0,1)")), Value::Number(1.0));
}

#[test]
fn index_out_of_bounds_row_returns_ref_error() {
    assert_eq!(helpers::eval("=INDEX({1,2,3;4,5,6},5,1)"), Value::Error(ErrorKind::Ref));
}

#[test]
fn index_out_of_bounds_col_returns_ref_error() {
    assert_eq!(helpers::eval("=INDEX({1,2,3;4,5,6},1,10)"), Value::Error(ErrorKind::Ref));
}

// ── TRANSPOSE of empty array ──────────────────────────────────────────────────

#[test]
fn transpose_single_element_passthrough() {
    // TRANSPOSE({1}) -> [1]; anchor-cell (top-left) view: first -> 1
    assert_eq!(top_left(helpers::eval("=TRANSPOSE({1})")), Value::Number(1.0));
}
