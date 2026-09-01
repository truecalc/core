use super::GridEdit::*;
use super::*;

/// Edit and formula both live on `Sheet1` — the common single-sheet case.
fn t(formula: &str, edit: GridEdit) -> String {
    shift_refs_text(formula, "Sheet1", "Sheet1", edit).unwrap()
}

// ---------------------------------------------------------------- row insert

#[test]
fn insert_row_above_a_cell_shifts_it_down() {
    assert_eq!(t("=A5", InsertRows { at: 2, count: 1 }), "=A6");
}

#[test]
fn insert_row_below_a_cell_leaves_it_alone() {
    assert_eq!(t("=A5", InsertRows { at: 6, count: 1 }), "=A5");
}

#[test]
fn insert_row_exactly_at_a_cell_shifts_it_down() {
    assert_eq!(t("=A5", InsertRows { at: 5, count: 1 }), "=A6");
}

#[test]
fn insert_multiple_rows_shifts_by_the_count() {
    assert_eq!(t("=A5", InsertRows { at: 1, count: 3 }), "=A8");
}

#[test]
fn insert_row_inside_a_range_expands_it() {
    assert_eq!(
        t("=SUM(A1:A5)", InsertRows { at: 3, count: 1 }),
        "=SUM(A1:A6)"
    );
}

#[test]
fn insert_row_at_a_ranges_first_row_moves_the_whole_range() {
    // Both endpoints are >= the insertion index, so both shift: the range
    // moves down rather than growing.
    assert_eq!(
        t("=SUM(A2:A5)", InsertRows { at: 2, count: 1 }),
        "=SUM(A3:A6)"
    );
}

#[test]
fn insert_row_at_a_ranges_last_row_expands_it() {
    assert_eq!(
        t("=SUM(A1:A3)", InsertRows { at: 3, count: 1 }),
        "=SUM(A1:A4)"
    );
}

#[test]
fn insert_row_just_past_a_ranges_last_row_leaves_it_alone() {
    assert_eq!(
        t("=SUM(A1:A3)", InsertRows { at: 4, count: 1 }),
        "=SUM(A1:A3)"
    );
}

#[test]
fn insert_row_does_not_touch_the_column_axis() {
    assert_eq!(
        t("=SUM(B2:D4)", InsertRows { at: 1, count: 1 }),
        "=SUM(B3:D5)"
    );
}

// ---------------------------------------------------------------- row delete

#[test]
fn delete_row_above_a_cell_shifts_it_up() {
    assert_eq!(t("=A5", DeleteRows { at: 2, count: 1 }), "=A4");
}

#[test]
fn delete_row_below_a_cell_leaves_it_alone() {
    assert_eq!(t("=A5", DeleteRows { at: 6, count: 1 }), "=A5");
}

#[test]
fn delete_the_row_a_cell_points_at_makes_it_a_ref_error() {
    assert_eq!(t("=A5", DeleteRows { at: 5, count: 1 }), "=#REF!");
}

#[test]
fn delete_rows_wholly_containing_a_range_makes_it_a_ref_error() {
    assert_eq!(
        t("=SUM(A2:A4)", DeleteRows { at: 1, count: 5 }),
        "=SUM(#REF!)"
    );
}

#[test]
fn delete_rows_exactly_covering_a_range_makes_it_a_ref_error() {
    assert_eq!(
        t("=SUM(A2:A4)", DeleteRows { at: 2, count: 3 }),
        "=SUM(#REF!)"
    );
}

#[test]
fn delete_a_row_inside_a_range_shrinks_it() {
    assert_eq!(
        t("=SUM(A1:A5)", DeleteRows { at: 3, count: 1 }),
        "=SUM(A1:A4)"
    );
}

#[test]
fn delete_rows_overlapping_a_ranges_start_clamps_the_start() {
    // rows 1..3 go; A2:A5 loses rows 2 and 3, and old rows 4,5 become 1,2.
    assert_eq!(
        t("=SUM(A2:A5)", DeleteRows { at: 1, count: 3 }),
        "=SUM(A1:A2)"
    );
}

#[test]
fn delete_rows_overlapping_a_ranges_end_clamps_the_end() {
    assert_eq!(
        t("=SUM(A1:A5)", DeleteRows { at: 3, count: 9 }),
        "=SUM(A1:A2)"
    );
}

#[test]
fn delete_rows_entirely_below_a_range_leaves_it_alone() {
    assert_eq!(
        t("=SUM(A1:A3)", DeleteRows { at: 4, count: 2 }),
        "=SUM(A1:A3)"
    );
}

#[test]
fn delete_rows_entirely_above_a_range_shifts_it_up() {
    assert_eq!(
        t("=SUM(A5:A7)", DeleteRows { at: 1, count: 2 }),
        "=SUM(A3:A5)"
    );
}

// ------------------------------------------------------------ column insert/delete

#[test]
fn insert_column_left_of_a_cell_shifts_it_right() {
    assert_eq!(t("=C5", InsertColumns { at: 2, count: 1 }), "=D5");
}

#[test]
fn insert_column_right_of_a_cell_leaves_it_alone() {
    assert_eq!(t("=C5", InsertColumns { at: 4, count: 1 }), "=C5");
}

#[test]
fn delete_the_column_a_cell_points_at_makes_it_a_ref_error() {
    assert_eq!(t("=C5", DeleteColumns { at: 3, count: 1 }), "=#REF!");
}

#[test]
fn delete_a_column_inside_a_range_shrinks_it() {
    assert_eq!(
        t("=SUM(A1:E1)", DeleteColumns { at: 3, count: 1 }),
        "=SUM(A1:D1)"
    );
}

#[test]
fn delete_column_does_not_touch_the_row_axis() {
    assert_eq!(
        t("=SUM(B2:D4)", DeleteColumns { at: 1, count: 1 }),
        "=SUM(A2:C4)"
    );
}

// -------------------------------------------------------------------- $ anchors

#[test]
fn absolute_row_anchor_still_shifts_and_is_preserved() {
    // `$` controls copy/fill translation, not structural insert/delete: an
    // anchored reference still tracks the cell it points at.
    assert_eq!(t("=A$5", InsertRows { at: 1, count: 1 }), "=A$6");
}

#[test]
fn fully_absolute_reference_still_shifts_and_is_preserved() {
    assert_eq!(t("=$A$5", InsertRows { at: 1, count: 1 }), "=$A$6");
}

#[test]
fn absolute_column_anchor_shifts_on_column_insert() {
    assert_eq!(t("=$C$5", InsertColumns { at: 1, count: 1 }), "=$D$5");
}

#[test]
fn absolute_range_endpoints_are_preserved_when_shrinking() {
    assert_eq!(
        t("=SUM($A$1:$A$5)", DeleteRows { at: 3, count: 1 }),
        "=SUM($A$1:$A$4)"
    );
}

// ------------------------------------------------------------------- sheets

#[test]
fn qualified_reference_to_the_edited_sheet_shifts() {
    assert_eq!(
        t("=Sheet1!A5", InsertRows { at: 1, count: 1 }),
        "=Sheet1!A6"
    );
}

#[test]
fn qualified_reference_to_another_sheet_does_not_move() {
    assert_eq!(t("=Other!A5", InsertRows { at: 1, count: 1 }), "=Other!A5");
}

#[test]
fn qualified_reference_to_another_sheet_is_never_deleted() {
    assert_eq!(
        t("=SUM(Other!A2:A4)", DeleteRows { at: 1, count: 9 }),
        "=SUM(Other!A2:A4)"
    );
}

#[test]
fn bare_reference_in_a_formula_on_another_sheet_does_not_move() {
    // The formula lives on Sheet2; the edit is on Sheet1. A bare `A5` means
    // Sheet2!A5, which the Sheet1 edit does not touch.
    let out = shift_refs_text("=A5", "Sheet2", "Sheet1", InsertRows { at: 1, count: 1 }).unwrap();
    assert_eq!(out, "=A5");
}

#[test]
fn cross_sheet_formula_still_shifts_its_qualified_refs_to_the_edited_sheet() {
    let out = shift_refs_text(
        "=Sheet1!A5+A5",
        "Sheet2",
        "Sheet1",
        InsertRows { at: 1, count: 1 },
    )
    .unwrap();
    assert_eq!(out, "=Sheet1!A6+A5");
}

#[test]
fn sheet_matching_is_case_insensitive() {
    let out = shift_refs_text(
        "=SHEET1!A5",
        "Sheet2",
        "sheet1",
        InsertRows { at: 1, count: 1 },
    )
    .unwrap();
    assert_eq!(out, "=SHEET1!A6");
}

#[test]
fn quoted_sheet_name_is_requoted_on_output() {
    let out = shift_refs_text(
        "='Q2 Data'!A5",
        "Other",
        "Q2 Data",
        InsertRows { at: 1, count: 1 },
    )
    .unwrap();
    assert_eq!(out, "='Q2 Data'!A6");
}

#[test]
fn deleting_a_qualified_reference_drops_the_sheet_qualifier_with_it() {
    // `Sheet1!#REF!` does not re-parse, so the whole reference span — sheet
    // qualifier included — becomes a bare `#REF!`.
    assert_eq!(t("=Sheet1!A5", DeleteRows { at: 5, count: 1 }), "=#REF!");
}

// ------------------------------------------------------------ non-references

#[test]
fn string_literals_are_left_untouched() {
    assert_eq!(
        t("=CONCAT(\"A5\",A5)", InsertRows { at: 1, count: 1 }),
        "=CONCAT(\"A5\",A6)"
    );
}

#[test]
fn function_names_and_defined_names_are_left_untouched() {
    assert_eq!(
        t("=SUM(A5,TAX_RATE)", InsertRows { at: 1, count: 1 }),
        "=SUM(A6,TAX_RATE)"
    );
}

#[test]
fn let_bound_names_that_look_like_addresses_are_left_untouched() {
    assert_eq!(
        t("=LET(A5, 5, A5*2)", InsertRows { at: 1, count: 1 }),
        "=LET(A5, 5, A5*2)"
    );
}

// ----------------------------------------------------------------- mechanics

#[test]
fn multiple_references_splice_correctly_when_lengths_change() {
    // A9 -> A10 grows a byte; the earlier A1 -> A2 splice must survive it.
    assert_eq!(t("=A1+A9", InsertRows { at: 1, count: 1 }), "=A2+A10");
}

#[test]
fn zero_count_is_a_no_op() {
    assert_eq!(
        t("=SUM(A1:A5)", InsertRows { at: 3, count: 0 }),
        "=SUM(A1:A5)"
    );
    assert_eq!(
        t("=SUM(A1:A5)", DeleteRows { at: 3, count: 0 }),
        "=SUM(A1:A5)"
    );
}

#[test]
fn index_zero_is_rejected() {
    assert!(shift_refs_text("=A1", "Sheet1", "Sheet1", InsertRows { at: 0, count: 1 }).is_err());
    assert!(shift_refs_text("=A1", "Sheet1", "Sheet1", DeleteRows { at: 0, count: 1 }).is_err());
}

#[test]
fn parse_errors_propagate() {
    assert!(shift_refs_text("=SUM(", "Sheet1", "Sheet1", InsertRows { at: 1, count: 1 }).is_err());
}

#[test]
fn insert_that_pushes_a_reference_off_the_grid_makes_it_a_ref_error() {
    let out = t("=A10000000", InsertRows { at: 1, count: 1 });
    assert_eq!(out, "=#REF!");
}

#[test]
fn output_of_a_deleting_edit_re_parses() {
    let out = t("=SUM(Sheet1!A2:A4)+1", DeleteRows { at: 1, count: 9 });
    assert_eq!(out, "=SUM(#REF!)+1");
    assert!(crate::parser::parse_formula(&out).is_ok());
}
