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
