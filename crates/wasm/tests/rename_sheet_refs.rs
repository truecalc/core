//! Surface-shape tests for `rename_sheet_refs`/`RenameSheetRefsResult` (issue #720).
//!
//! These run natively under `cargo nextest`/`cargo test`; no wasm runtime is
//! needed since neither the input nor the output touches `JsValue`.

use truecalc_wasm::rename_sheet_refs;

#[test]
fn rewrites_matching_sheet_qualified_ref() {
    let result = rename_sheet_refs("=Data!A1", "Data", "Results");
    assert_eq!(result.formula.as_deref(), Some("=Results!A1"));
    assert_eq!(result.error, None);
}

#[test]
fn leaves_other_sheet_refs_untouched() {
    let result = rename_sheet_refs("=Data!A1+Other!B1", "Data", "Results");
    assert_eq!(result.formula.as_deref(), Some("=Results!A1+Other!B1"));
}

#[test]
fn requotes_when_new_name_needs_quoting() {
    let result = rename_sheet_refs("=Data!A1", "Data", "Q2 Data");
    assert_eq!(result.formula.as_deref(), Some("='Q2 Data'!A1"));
}

#[test]
fn parse_error_surfaces_in_error_field() {
    let result = rename_sheet_refs("=SUM(", "Data", "Results");
    assert_eq!(result.formula, None);
    assert!(result.error.is_some());
}
