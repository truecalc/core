use super::*;
use crate::named_ref::parse_canonical_ref;

fn bounds(r: &str) -> ParsedRangeBounds {
    let parsed = parse_canonical_ref(r).unwrap();
    parsed_range_bounds(r, &parsed).unwrap()
}

#[test]
fn overlapping_ranges_on_same_sheet_detected() {
    assert!(ranges_overlap(&bounds("Sheet1!A1:D12"), &bounds("Sheet1!C5:E20")));
}

#[test]
fn adjacent_non_overlapping_ranges_are_fine() {
    assert!(!ranges_overlap(&bounds("Sheet1!A1:D12"), &bounds("Sheet1!E1:F12")));
}

#[test]
fn ranges_on_different_sheets_never_overlap() {
    assert!(!ranges_overlap(&bounds("Sheet1!A1:D12"), &bounds("Sheet2!A1:D12")));
}

#[test]
fn header_row_columns_rejects_duplicate_names() {
    let err = header_row_columns(["quantity_g", "quantity_g"].into_iter()).unwrap_err();
    assert!(err.contains("quantity_g"), "error should name the duplicate: {err}");
}

#[test]
fn header_row_columns_rejects_invalid_identifier() {
    let err = header_row_columns(["quantity g"].into_iter()).unwrap_err();
    assert!(err.contains("quantity g"));
}

#[test]
fn header_row_columns_accepts_valid_unique_names() {
    let cols = header_row_columns(["quantity_g", "reference_per_100g"].into_iter()).unwrap();
    assert_eq!(cols, vec!["quantity_g".to_string(), "reference_per_100g".to_string()]);
}
