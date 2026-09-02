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

// ------------------------------------------------------ backwards-written ranges

#[test]
fn backwards_range_untouched_by_the_edit_is_left_alone() {
    // `A5:A1` is a legal way to write rows 1..5; an edit below it must not
    // mistake the written order for a deleted span.
    assert_eq!(
        t("=SUM(A5:A1)", InsertRows { at: 10, count: 1 }),
        "=SUM(A5:A1)"
    );
    assert_eq!(
        t("=SUM(A5:A1)", DeleteRows { at: 10, count: 1 }),
        "=SUM(A5:A1)"
    );
}

#[test]
fn backwards_range_shrinks_from_the_written_end() {
    // rows 4 and 5 go; the range covered rows 1..5, so rows 1..3 survive.
    assert_eq!(
        t("=SUM(A5:A1)", DeleteRows { at: 4, count: 2 }),
        "=SUM(A3:A1)"
    );
}

#[test]
fn backwards_range_wholly_deleted_is_a_ref_error() {
    assert_eq!(
        t("=SUM(A5:A1)", DeleteRows { at: 1, count: 9 }),
        "=SUM(#REF!)"
    );
}

#[test]
fn backwards_range_shifts_whole_on_insert_above_it() {
    assert_eq!(
        t("=SUM(A5:A1)", InsertRows { at: 1, count: 2 }),
        "=SUM(A7:A3)"
    );
}

#[test]
fn backwards_column_range_shrinks_correctly() {
    assert_eq!(
        t("=SUM(E1:A1)", DeleteColumns { at: 4, count: 2 }),
        "=SUM(C1:A1)"
    );
}

// ------------------------------------------------- column axis, in full

#[test]
fn delete_columns_wholly_containing_a_range_makes_it_a_ref_error() {
    assert_eq!(
        t("=SUM(B1:D1)", DeleteColumns { at: 1, count: 9 }),
        "=SUM(#REF!)"
    );
}

#[test]
fn delete_columns_overlapping_a_ranges_start_clamps_the_start() {
    assert_eq!(
        t("=SUM(B1:E1)", DeleteColumns { at: 1, count: 3 }),
        "=SUM(A1:B1)"
    );
}

#[test]
fn delete_columns_overlapping_a_ranges_end_clamps_the_end() {
    assert_eq!(
        t("=SUM(A1:E1)", DeleteColumns { at: 3, count: 9 }),
        "=SUM(A1:B1)"
    );
}

#[test]
fn absolute_column_anchor_is_preserved_across_a_column_delete() {
    assert_eq!(
        t("=SUM($C$1:$E$1)", DeleteColumns { at: 4, count: 1 }),
        "=SUM($C$1:$D$1)"
    );
}

#[test]
fn insert_that_pushes_a_column_off_the_grid_makes_it_a_ref_error() {
    // ZZZ is the last column of the Sheets grid.
    assert_eq!(t("=ZZZ1", InsertColumns { at: 1, count: 1 }), "=#REF!");
}

// -------------------------------------------------------- more coverage

#[test]
fn a_surviving_range_keeps_its_sheet_qualifier_through_a_shrink() {
    assert_eq!(
        t("=SUM(Sheet1!A1:A5)", DeleteRows { at: 3, count: 1 }),
        "=SUM(Sheet1!A1:A4)"
    );
}

#[test]
fn lambda_params_and_shadowed_body_uses_are_left_untouched() {
    assert_eq!(
        t("=LAMBDA(A5, A5*2)(3)", InsertRows { at: 1, count: 1 }),
        "=LAMBDA(A5, A5*2)(3)"
    );
}

#[test]
fn lambda_call_arguments_are_real_references_and_shift() {
    assert_eq!(
        t("=LAMBDA(X, X*2)(A5)", InsertRows { at: 1, count: 1 }),
        "=LAMBDA(X, X*2)(A6)"
    );
}

#[test]
fn a_two_dimensional_range_wholly_removed_by_a_row_delete_is_a_ref_error() {
    assert_eq!(
        t("=SUM(B2:D8)", DeleteRows { at: 1, count: 100 }),
        "=SUM(#REF!)"
    );
}

#[test]
fn a_two_dimensional_range_shifts_only_on_the_edited_axis() {
    assert_eq!(
        t("=SUM(B2:D8)", DeleteRows { at: 1, count: 1 }),
        "=SUM(B1:D7)"
    );
}

#[test]
fn a_range_with_only_one_endpoint_pushed_off_the_grid_is_a_ref_error() {
    // Deliberately whole-reference, not per-corner: `#REF!:A9999999` does not
    // re-parse, so a half-`#REF!` range would not survive a round trip.
    assert_eq!(
        t("=SUM(A9999999:A10000000)", InsertRows { at: 1, count: 1 }),
        "=SUM(#REF!)"
    );
}

#[test]
fn an_index_beyond_the_axis_maximum_is_a_no_op() {
    assert_eq!(
        t(
            "=SUM(A1:A5)",
            InsertRows {
                at: 20_000_000,
                count: 1
            }
        ),
        "=SUM(A1:A5)"
    );
    assert_eq!(
        t(
            "=SUM(A1:A5)",
            DeleteRows {
                at: 20_000_000,
                count: 1
            }
        ),
        "=SUM(A1:A5)"
    );
}

#[test]
fn huge_counts_do_not_overflow() {
    assert_eq!(
        t(
            "=A5",
            InsertRows {
                at: 1,
                count: u32::MAX
            }
        ),
        "=#REF!"
    );
    assert_eq!(
        t(
            "=A5",
            DeleteRows {
                at: 1,
                count: u32::MAX
            }
        ),
        "=#REF!"
    );
    assert_eq!(
        t(
            "=A5",
            DeleteRows {
                at: 6,
                count: u32::MAX
            }
        ),
        "=A5"
    );
}

// ------------------------------------------------------------ engine surface

#[test]
fn excel_flavor_is_rejected() {
    let err = crate::Engine::excel()
        .shift_refs_for_grid_edit("=A1", "Sheet1", "Sheet1", InsertRows { at: 1, count: 1 })
        .unwrap_err();
    assert!(err.message.contains("Excel flavor not yet supported"));
}

#[test]
fn engine_surface_matches_the_module_level_transform() {
    let edit = DeleteRows { at: 2, count: 2 };
    let out = crate::Engine::sheets()
        .shift_refs_for_grid_edit("=SUM(A1:A5)+A3", "Sheet1", "Sheet1", edit)
        .unwrap();
    assert_eq!(out, t("=SUM(A1:A5)+A3", edit));
}

// ============================================================= AxisMove

/// Edit and formula both live on `Sheet1` — the common single-sheet case.
fn mv(formula: &str, mv: AxisMove) -> String {
    shift_refs_for_move(formula, "Sheet1", "Sheet1", mv).unwrap()
}

// ---------------------------------------------------------------- unaffected

#[test]
fn move_reference_unaffected_before_both_band_and_destination_is_unchanged() {
    // Backward move of rows 5:7 to row 2; row 1 is before both the
    // destination and the band, so it never enters the gap-fill zone.
    assert_eq!(
        mv(
            "=A1",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=A1"
    );
}

#[test]
fn move_does_not_touch_a_reference_on_another_sheet() {
    assert_eq!(
        mv(
            "=Other!A5",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=Other!A5"
    );
}

#[test]
fn bare_reference_in_a_formula_on_another_sheet_is_untouched_by_a_move() {
    // The formula lives on Sheet2; the move is on Sheet1. A bare `A5` means
    // Sheet2!A5, which the Sheet1 move does not touch.
    let out = shift_refs_for_move(
        "=A5",
        "Sheet2",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 5,
            end: 7,
            at: 2,
        },
    )
    .unwrap();
    assert_eq!(out, "=A5");
}

// ----------------------------------------------------------- band and gap

#[test]
fn move_reference_inside_the_band_translates_backward() {
    // rows 5:7 -> row 2: row 5 (band start) lands at row 2.
    assert_eq!(
        mv(
            "=A5",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=A2"
    );
}

#[test]
fn move_reference_inside_the_band_translates_forward() {
    // rows 2:4 -> row 8: row 2 (band start) lands at row 8.
    assert_eq!(
        mv(
            "=A2",
            AxisMove {
                axis: Axis::Row,
                start: 2,
                end: 4,
                at: 8
            }
        ),
        "=A8"
    );
}

#[test]
fn move_backward_shifts_the_gap_region_forward_by_the_bands_width() {
    // rows 5:7 (width 3) -> row 2: row 3, between the new and old start,
    // slides forward by 3 to close the gap the band left.
    assert_eq!(
        mv(
            "=A3",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=A6"
    );
}

#[test]
fn move_forward_shifts_the_gap_region_backward_by_the_bands_width() {
    // rows 2:4 (width 3) -> row 8: row 5, just after the old band end,
    // slides back by 3 to close the gap the band left behind.
    assert_eq!(
        mv(
            "=A5",
            AxisMove {
                axis: Axis::Row,
                start: 2,
                end: 4,
                at: 8
            }
        ),
        "=A2"
    );
}

// ------------------------------------------------------- canonical example

#[test]
fn canonical_example_swaps_row_three_and_row_six() {
    // The issue's own worked example: moving rows 5:7 to before row 2 sends
    // row 3's content to row 6 and row 6's content to row 3.
    assert_eq!(
        mv(
            "=A3+A6",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=A6+A3"
    );
}

#[test]
fn move_induced_corner_swap_normalizes_an_ascending_range() {
    assert_eq!(
        mv(
            "=SUM(A4:A6)",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=SUM(A3:A7)"
    );
}

#[test]
fn a_range_already_written_backwards_is_not_confused_with_a_move_induced_swap() {
    // A7:A5, unaffected by a move entirely elsewhere: it must stay written
    // backwards, not be "corrected" into ascending order.
    assert_eq!(
        mv(
            "=SUM(A7:A5)",
            AxisMove {
                axis: Axis::Row,
                start: 20,
                end: 22,
                at: 30
            }
        ),
        "=SUM(A7:A5)"
    );
}

#[test]
fn a_range_already_written_backwards_stays_backwards_when_the_move_would_uncross_it() {
    // The mirror of `move_induced_corner_swap_normalizes_an_ascending_range`:
    // A6:A4 covers the same physical rows as A4:A6 (whose corners the same
    // move inverts to A3:A7 from an ascending start), just written
    // descending. Independently mapping A6:A4's corners the same way move
    // maps A4:A6's produces new_start=3 (from row 6, inside the band) and
    // new_end=7 (from row 4, in the backward gap-fill zone) — ascending, not
    // descending — so without the mirror correction this would silently
    // collapse to the same "A3:A7" text as the ascending case, losing the
    // fact that the range was deliberately written backwards. It must
    // instead render descending: A7:A3.
    assert_eq!(
        mv(
            "=SUM(A6:A4)",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=SUM(A7:A3)"
    );
}

// -------------------------------------------------------------- $ anchors

#[test]
fn absolute_anchors_are_preserved_through_a_move() {
    assert_eq!(
        mv(
            "=$A$6",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=$A$3"
    );
}

// -------------------------------------------------------------- no-ops

#[test]
fn moving_a_band_to_its_own_start_is_a_no_op() {
    assert_eq!(
        mv(
            "=SUM(A1:A10)",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 5
            }
        ),
        "=SUM(A1:A10)"
    );
}

#[test]
fn moving_a_band_to_a_destination_inside_itself_is_a_no_op() {
    assert_eq!(
        mv(
            "=SUM(A1:A10)",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 6
            }
        ),
        "=SUM(A1:A10)"
    );
}

#[test]
fn moving_a_band_to_a_destination_inside_itself_is_a_no_op_even_for_a_reference_in_the_band() {
    // `SUM(A1:A10)` in the sibling test above is unaffected by the move
    // either way, so it cannot tell a correct no-op apart from a bug in the
    // no-op guard. This test uses references that DO fall inside the band
    // (`start..=end`) and its own would-be gap-fill zone, to confirm the
    // short-circuit really does leave every one of them untouched.
    //
    // `at` landing inside `start..=end` (here `at: 6`, strictly between
    // `start` and `end`) has no well-defined destination: extending
    // move_coord's band/gap-fill formulas past their documented domain (both
    // assume `at` is already outside `start..=end`) does not recover some
    // other "real" move being discarded — it computes a self-referential
    // cyclic permutation of the band with the coordinates just past it
    // (e.g. here 5->6, 6->7, 7->8, 8->5), which is not a rectangle any
    // spreadsheet UI can express as a drag target ("move this band to a row
    // that is itself part of the band"). Treating the whole closed interval
    // as a no-op is therefore not a narrower, buggy special case of
    // `at == start` — it is the correct, documented behavior for every `at`
    // in that range.
    for formula in ["=A5", "=A6", "=A7", "=A8", "=A5+A6+A7+A8"] {
        assert_eq!(
            mv(
                formula,
                AxisMove {
                    axis: Axis::Row,
                    start: 5,
                    end: 7,
                    at: 6
                }
            ),
            formula
        );
    }
}

#[test]
fn moving_a_band_to_immediately_after_itself_is_a_real_move_not_a_no_op() {
    // at == end + 1 is the smallest genuine forward move: the band swaps
    // with the immediately following equal-width block.
    assert_eq!(
        mv(
            "=A5+A8",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 8
            }
        ),
        "=A8+A5"
    );
}

// --------------------------------------------------------------- columns

#[test]
fn column_axis_move_translates_the_band() {
    assert_eq!(
        mv(
            "=E1",
            AxisMove {
                axis: Axis::Column,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=B1"
    );
}

#[test]
fn column_axis_move_shifts_the_gap_region() {
    assert_eq!(
        mv(
            "=C1",
            AxisMove {
                axis: Axis::Column,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=F1"
    );
}

#[test]
fn a_two_dimensional_range_moves_only_on_the_moved_axis() {
    assert_eq!(
        mv(
            "=SUM(B2:D8)",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2
            }
        ),
        "=SUM(B5:D8)"
    );
}

// ------------------------------------------------------------ validation

#[test]
fn zero_based_start_is_rejected() {
    assert!(shift_refs_for_move(
        "=A1",
        "Sheet1",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 0,
            end: 2,
            at: 5
        }
    )
    .is_err());
}

#[test]
fn zero_based_at_is_rejected() {
    assert!(shift_refs_for_move(
        "=A1",
        "Sheet1",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 1,
            end: 2,
            at: 0
        }
    )
    .is_err());
}

#[test]
fn start_greater_than_end_is_rejected() {
    assert!(shift_refs_for_move(
        "=A1",
        "Sheet1",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 5,
            end: 2,
            at: 1
        }
    )
    .is_err());
}

#[test]
fn destination_pushing_the_band_off_the_grid_is_rejected() {
    assert!(shift_refs_for_move(
        "=A1",
        "Sheet1",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 1,
            end: 2,
            at: 10_000_000,
        }
    )
    .is_err());
}

#[test]
fn at_near_u32_max_is_rejected_without_overflow() {
    assert!(shift_refs_for_move(
        "=A1",
        "Sheet1",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 1,
            end: 1,
            at: u32::MAX
        }
    )
    .is_err());
}

#[test]
fn parse_errors_propagate_for_a_move() {
    assert!(shift_refs_for_move(
        "=SUM(",
        "Sheet1",
        "Sheet1",
        AxisMove {
            axis: Axis::Row,
            start: 5,
            end: 7,
            at: 2
        }
    )
    .is_err());
}

// -------------------------------------------------------------- mechanics

#[test]
fn output_of_a_move_re_parses() {
    let out = mv(
        "=SUM(A4:A6)",
        AxisMove {
            axis: Axis::Row,
            start: 5,
            end: 7,
            at: 2,
        },
    );
    assert_eq!(out, "=SUM(A3:A7)");
    assert!(crate::parser::parse_formula(&out).is_ok());
}

// ------------------------------------------------------------ engine surface

#[test]
fn engine_excel_flavor_is_rejected_for_move() {
    let err = crate::Engine::excel()
        .shift_refs_for_move(
            "=A1",
            "Sheet1",
            "Sheet1",
            AxisMove {
                axis: Axis::Row,
                start: 5,
                end: 7,
                at: 2,
            },
        )
        .unwrap_err();
    assert!(err.message.contains("Excel flavor not yet supported"));
}

#[test]
fn engine_surface_matches_the_module_level_move_transform() {
    let axis_move = AxisMove {
        axis: Axis::Row,
        start: 5,
        end: 7,
        at: 2,
    };
    let out = crate::Engine::sheets()
        .shift_refs_for_move("=SUM(A4:A6)", "Sheet1", "Sheet1", axis_move)
        .unwrap();
    assert_eq!(out, mv("=SUM(A4:A6)", axis_move));
}
