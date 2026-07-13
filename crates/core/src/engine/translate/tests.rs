use super::*;

#[test]
fn shifts_relative_both_axes() {
    let a = CellAddr::new(1, 1); // A1
    assert_eq!(shift_addr(a, 1, 1), Some(CellAddr::new(2, 2)));
}

#[test]
fn skips_absolute_column() {
    let a = CellAddr::new(1, 1).with_col_abs(true);
    assert_eq!(shift_addr(a, 5, 5), Some(CellAddr::new(1, 6).with_col_abs(true)));
}

#[test]
fn skips_absolute_row() {
    let a = CellAddr::new(1, 1).with_row_abs(true);
    assert_eq!(shift_addr(a, 5, 5), Some(CellAddr::new(6, 1).with_row_abs(true)));
}

#[test]
fn skips_both_absolute_axes() {
    let a = CellAddr::new(1, 1).with_col_abs(true).with_row_abs(true);
    assert_eq!(shift_addr(a, 5, 5), Some(a));
}

#[test]
fn out_of_bounds_negative_row_is_none() {
    let a = CellAddr::new(1, 1);
    assert_eq!(shift_addr(a, -5, 0), None);
}

#[test]
fn out_of_bounds_negative_col_is_none() {
    let a = CellAddr::new(1, 1);
    assert_eq!(shift_addr(a, 0, -5), None);
}

#[test]
fn in_bounds_at_grid_edge() {
    let a = CellAddr::new(MAX_COL as u32, 1);
    assert_eq!(shift_addr(a, 0, 0), Some(a));
}

#[test]
fn out_of_bounds_past_grid_edge() {
    let a = CellAddr::new(MAX_COL as u32, 1);
    assert_eq!(shift_addr(a, 0, 1), None);
}

#[test]
fn cell_within_bounds_renders_shifted_text() {
    let r = Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, 1, 1), "B2");
}

#[test]
fn cell_out_of_bounds_renders_ref_error() {
    let r = Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, -5, 0), "#REF!");
}

#[test]
fn sheet_qualified_cell_preserves_sheet_prefix() {
    let r = Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, 1, 0), "Data!A2");
}

#[test]
fn quoted_sheet_name_preserved() {
    let r = Ref::Cell { sheet: Some("Q2 Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, 1, 0), "'Q2 Data'!A2");
}

#[test]
fn range_shifts_both_corners_independently() {
    let r = Ref::Range {
        sheet: None,
        start: CellAddr::new(1, 1),
        end: CellAddr::new(4, 4).with_col_abs(true),
    };
    // start A1 -> B2; end $D4 -> $D5 (column absolute, row shifts 4 -> 5)
    assert_eq!(shift_ref_text(&r, 1, 1), "B2:$D5");
}

#[test]
fn range_one_corner_out_of_bounds_only_that_corner_becomes_ref_error() {
    let r = Ref::Range { sheet: None, start: CellAddr::new(1, 1), end: CellAddr::new(2, 10) };
    // start row 1-5=-4 (OOB); end row 10-5=5 (OK)
    assert_eq!(shift_ref_text(&r, -5, 0), "#REF!:B5");
}

fn spans_text<'a>(formula: &'a str) -> Vec<&'a str> {
    let expr = crate::parser::parse_formula(formula).unwrap();
    collect_shiftable_refs(&expr)
        .into_iter()
        .map(|(span, _)| &formula[span.offset..span.offset + span.length])
        .collect()
}

#[test]
fn bare_cell_reference_is_collected() {
    assert_eq!(spans_text("=A1"), vec!["A1"]);
}

#[test]
fn sheet_qualified_reference_is_collected() {
    assert_eq!(spans_text("=Sheet1!A1"), vec!["Sheet1!A1"]);
}

#[test]
fn defined_name_is_not_collected() {
    assert_eq!(spans_text("=TAX_RATE"), Vec::<&str>::new());
}

#[test]
fn function_name_is_not_collected() {
    assert_eq!(spans_text("=SUM(A1,B1)"), vec!["A1", "B1"]);
}

#[test]
fn string_literal_is_not_collected() {
    assert_eq!(spans_text("=CONCAT(\"A1\", B1)"), vec!["B1"]);
}

#[test]
fn range_is_collected_as_single_span() {
    assert_eq!(spans_text("=SUM(A1:B2)"), vec!["A1:B2"]);
}

#[test]
fn let_binding_name_and_shadowed_body_use_are_skipped() {
    assert_eq!(spans_text("=LET(A1, 5, A1*2)"), Vec::<&str>::new());
}

#[test]
fn let_value_expr_self_reference_is_a_real_cell_ref() {
    // A1 inside the value expr is not yet bound (LET binds only after its
    // own value expr evaluates), so it's the real cell, not the local name.
    assert_eq!(spans_text("=LET(A1, A1+1, A1*2)"), vec!["A1"]);
}

#[test]
fn let_second_pair_value_expr_sees_first_binding_as_local() {
    assert_eq!(spans_text("=LET(A1, 5, B1, A1+1, B1)"), Vec::<&str>::new());
}

#[test]
fn lambda_param_and_body_use_are_skipped() {
    assert_eq!(spans_text("=LAMBDA(A1, A1+1)(5)"), Vec::<&str>::new());
}

#[test]
fn lambda_call_args_are_evaluated_in_outer_scope() {
    // the invocation argument is a real cell ref, not shadowed by the param
    assert_eq!(spans_text("=LAMBDA(A1, A1+1)(B1)"), vec!["B1"]);
}
