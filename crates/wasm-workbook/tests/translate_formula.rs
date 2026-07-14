//! Surface-shape tests for `@truecalc/workbook`'s `translateFormula` export
//! (issue #715). These assert the exact rewritten formula text produced by the
//! binding for the reference-adjustment cases a host performs on fill / paste.
//!
//! They run natively under `cargo nextest`/`cargo test`: `TranslateResult` is a
//! plain struct with `pub` fields, so no wasm runtime or `JsValue` is involved.

use truecalc_wasm_workbook::translate_formula;

#[test]
fn relative_reference_shifts_by_offset() {
    let result = translate_formula("=A1", 1, 1);
    assert_eq!(result.formula.as_deref(), Some("=B2"));
    assert_eq!(result.error, None);
}

#[test]
fn dollar_pinned_reference_is_unchanged() {
    let result = translate_formula("=$A$1", 5, 5);
    assert_eq!(result.formula.as_deref(), Some("=$A$1"));
}

#[test]
fn mixed_dollar_axes_shift_only_relative_axis() {
    // `$A1` keeps its column, shifts its row; `A$1` shifts its column, keeps
    // its row.
    let result = translate_formula("=$A1+A$1", 1, 1);
    assert_eq!(result.formula.as_deref(), Some("=$A2+B$1"));
}

#[test]
fn range_endpoints_both_shift() {
    let result = translate_formula("=SUM(A1:B2)", 1, 1);
    assert_eq!(result.formula.as_deref(), Some("=SUM(B2:C3)"));
}

#[test]
fn cross_sheet_reference_shifts_and_preserves_sheet_name() {
    let result = translate_formula("=Sheet1!A1", 1, 1);
    assert_eq!(result.formula.as_deref(), Some("=Sheet1!B2"));
}

#[test]
fn string_literals_and_function_names_are_untouched() {
    // The `A1` inside the quoted string and the `CONCAT` name must not be
    // rewritten — only the bare `B1` reference shifts.
    let result = translate_formula("=CONCAT(\"A1\",B1)", 1, 0);
    assert_eq!(result.formula.as_deref(), Some("=CONCAT(\"A1\",B2)"));
}

#[test]
fn let_bound_name_is_not_rewritten() {
    // The case a text tokenizer cannot get right: `A1` is bound by `LET`, so
    // the binding name and the body use of `A1` stay put; only the free `A1`
    // in the value expression (evaluated before the binding takes effect)
    // shifts. A tokenizer would wrongly rewrite all three.
    let result = translate_formula("=LET(A1, A1+1, A1*2)", 1, 0);
    assert_eq!(result.formula.as_deref(), Some("=LET(A1, A2+1, A1*2)"));
}

#[test]
fn out_of_bounds_becomes_ref_error_text() {
    let result = translate_formula("=A1", -1, 0);
    assert_eq!(result.formula.as_deref(), Some("=#REF!"));
    assert_eq!(result.error, None);
}

#[test]
fn parse_error_surfaces_in_error_field() {
    let result = translate_formula("=SUM(", 0, 0);
    assert_eq!(result.formula, None);
    assert!(result.error.is_some());
}
