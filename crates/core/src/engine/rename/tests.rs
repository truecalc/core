use super::*;

use crate::parser::CellAddr;

#[test]
fn same_sheet_is_case_insensitive() {
    assert!(same_sheet("Data", "DATA"));
    assert!(same_sheet("data", "Data"));
    assert!(!same_sheet("Data", "Results"));
}

#[test]
fn ref_sheet_returns_none_for_unqualified_ref() {
    let r = Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) };
    assert_eq!(ref_sheet(&r), None);
}

#[test]
fn ref_sheet_returns_qualifier() {
    let r = Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(ref_sheet(&r), Some("Data"));
}

#[test]
fn renamed_ref_text_swaps_unquoted_sheet() {
    let r = Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(renamed_ref_text(&r, "Results"), "Results!A1");
}

#[test]
fn renamed_ref_text_requotes_when_new_name_needs_it() {
    let r = Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(renamed_ref_text(&r, "Q2 Data"), "'Q2 Data'!A1");
}

#[test]
fn renamed_ref_text_unquotes_when_new_name_no_longer_needs_it() {
    let r = Ref::Cell { sheet: Some("Q2 Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(renamed_ref_text(&r, "Results"), "Results!A1");
}

#[test]
fn renamed_ref_text_swaps_range_sheet() {
    let r = Ref::Range {
        sheet: Some("Data".to_string()),
        start: CellAddr::new(1, 1),
        end: CellAddr::new(2, 2),
    };
    assert_eq!(renamed_ref_text(&r, "Results"), "Results!A1:B2");
}

#[test]
fn rewrites_unquoted_sheet_qualified_cell() {
    assert_eq!(
        rename_sheet_refs_text("=Data!A1", "Data", "Results").unwrap(),
        "=Results!A1"
    );
}

#[test]
fn rewrites_quoted_sheet_qualified_cell() {
    assert_eq!(
        rename_sheet_refs_text("='Old Name'!A1", "Old Name", "Results").unwrap(),
        "=Results!A1"
    );
}

#[test]
fn requotes_when_new_name_needs_quoting() {
    assert_eq!(
        rename_sheet_refs_text("=Data!A1", "Data", "Q2 Data").unwrap(),
        "='Q2 Data'!A1"
    );
}

#[test]
fn rewrites_range_reference() {
    assert_eq!(
        rename_sheet_refs_text("=SUM(Data!A1:B2)", "Data", "Results").unwrap(),
        "=SUM(Results!A1:B2)"
    );
}

#[test]
fn matches_case_insensitively() {
    assert_eq!(
        rename_sheet_refs_text("=data!A1", "Data", "Results").unwrap(),
        "=Results!A1"
    );
}

#[test]
fn leaves_unqualified_refs_untouched() {
    assert_eq!(rename_sheet_refs_text("=A1+B1", "Data", "Results").unwrap(), "=A1+B1");
}

#[test]
fn leaves_other_sheet_refs_untouched() {
    assert_eq!(
        rename_sheet_refs_text("=Data!A1+Other!B1", "Data", "Results").unwrap(),
        "=Results!A1+Other!B1"
    );
}

#[test]
fn leaves_string_literals_untouched() {
    assert_eq!(
        rename_sheet_refs_text("=CONCAT(\"Data!A1\",Data!B1)", "Data", "Results").unwrap(),
        "=CONCAT(\"Data!A1\",Results!B1)"
    );
}

#[test]
fn leaves_function_names_untouched() {
    assert_eq!(
        rename_sheet_refs_text("=SUM(Data!A1)", "Data", "Results").unwrap(),
        "=SUM(Results!A1)"
    );
}

#[test]
fn leaves_defined_names_untouched() {
    assert_eq!(
        rename_sheet_refs_text("=SUM(Data!A1,TAX_RATE)", "Data", "Results").unwrap(),
        "=SUM(Results!A1,TAX_RATE)"
    );
}

#[test]
fn no_op_when_no_matching_refs() {
    assert_eq!(
        rename_sheet_refs_text("=Other!A1+B1", "Data", "Results").unwrap(),
        "=Other!A1+B1"
    );
}

#[test]
fn propagates_parse_error() {
    assert!(rename_sheet_refs_text("=SUM(", "Data", "Results").is_err());
}

#[test]
fn multiple_refs_on_same_sheet_all_rewritten() {
    assert_eq!(
        rename_sheet_refs_text("=Data!A1+Data!B2:C3", "Data", "Results").unwrap(),
        "=Results!A1+Results!B2:C3"
    );
}
