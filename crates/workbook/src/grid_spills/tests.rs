use super::*;

use crate::cell::Cell;
use crate::engine::EngineFlavor;
use crate::worksheet::Worksheet;

/// A 2x1 array — the smallest storable spill (§6 collapses a 1x1 to its scalar).
fn two_row_array(first: f64) -> Value {
    Value::Array(vec![
        vec![Value::Number(first)],
        vec![Value::Number(first + 1.0)],
    ])
}

/// A workbook whose only sheet holds `literals` plain literal cells down
/// column A, plus `anchors` spill anchors spread across column C.
///
/// The anchors are two rows tall and are placed 10 rows apart so their
/// rectangles never overlap, whatever `anchors` is.
fn sheet_with(literals: u32, anchors: u32) -> Workbook {
    let mut sheet = Worksheet::new("Sheet1");
    for row in 1..=literals {
        sheet.cells_mut().insert(
            Address::new(row, 1).unwrap().to_a1(),
            Cell::literal(Value::Number(f64::from(row))).unwrap(),
        );
    }
    for i in 0..anchors {
        sheet.cells_mut().insert(
            Address::new(1 + i * 10, 3).unwrap().to_a1(),
            Cell::with_formula("=SEQUENCE(2)", two_row_array(f64::from(i))),
        );
    }
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(sheet).unwrap();
    wb
}

/// The exact defect #910 metric: how many cells one empty-cell read examines.
///
/// A read that resolves to no spill walks the whole of `anchors()`, so the
/// slice length **is** the examined count. It must be a function of how many
/// spills the sheet holds, never of how many cells it holds — the sheet grows
/// 64x here and the count does not move.
#[test]
fn cells_examined_per_empty_read_does_not_grow_with_the_sheet() {
    let recomputed = BTreeSet::new();
    let mut examined = Vec::new();
    for literals in [1_000u32, 8_000, 64_000] {
        let wb = sheet_with(literals, 3);
        let index = GridSpillIndex::build(&wb, &recomputed);
        examined.push((literals, index.anchors("sheet1").len()));
    }
    assert_eq!(
        examined,
        vec![(1_000, 3), (8_000, 3), (64_000, 3)],
        "an empty-cell read must examine only the sheet's spill anchors, not \
         its cells"
    );
}

/// A sheet with no spills costs nothing at all to consult.
#[test]
fn a_sheet_without_spills_has_no_anchors() {
    let wb = sheet_with(1_000, 0);
    let index = GridSpillIndex::build(&wb, &BTreeSet::new());
    assert!(index.anchors("sheet1").is_empty());
    assert!(index.anchors("nosuchsheet").is_empty());
}

/// The anchors keep stored-grid order, so first-match resolution is unchanged.
#[test]
fn anchors_are_indexed_in_stored_grid_order_with_their_rectangles() {
    let wb = sheet_with(0, 2);
    let index = GridSpillIndex::build(&wb, &BTreeSet::new());
    let anchors = index.anchors("sheet1");
    assert_eq!(anchors.len(), 2);
    assert_eq!(anchors[0].0, Address::new(1, 3).unwrap());
    assert_eq!(anchors[1].0, Address::new(11, 3).unwrap());
    // C1's 2x1 array covers C1:C2, so C2 resolves through it and C3 does not.
    assert_eq!(
        anchors[0].1.offset_of(Address::new(2, 3).unwrap()),
        Some((1, 0))
    );
    assert_eq!(anchors[0].1.offset_of(Address::new(3, 3).unwrap()), None);
}

/// The `#591` rule: an anchor being recomputed this recalc has a stale stored
/// array, so it is excluded from the index entirely rather than skipped per
/// scanned cell.
#[test]
fn an_anchor_being_recomputed_is_excluded() {
    let wb = sheet_with(0, 2);
    let recomputed: BTreeSet<CellRef> = [CellRef {
        sheet: "sheet1".to_owned(),
        addr: Address::new(1, 3).unwrap(),
    }]
    .into_iter()
    .collect();
    let index = GridSpillIndex::build(&wb, &recomputed);
    let anchors = index.anchors("sheet1");
    assert_eq!(
        anchors.len(),
        1,
        "the recomputed anchor must not be indexed"
    );
    assert_eq!(anchors[0].0, Address::new(11, 3).unwrap());
}

/// Sheet names are matched case-insensitively, on the folded name the resolver
/// already carries.
#[test]
fn anchors_are_keyed_by_the_folded_sheet_name() {
    let mut sheet = Worksheet::new("MiXeD");
    sheet.cells_mut().insert(
        "A1".to_owned(),
        Cell::with_formula("=SEQUENCE(2)", two_row_array(1.0)),
    );
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(sheet).unwrap();
    let index = GridSpillIndex::build(&wb, &BTreeSet::new());
    assert_eq!(index.anchors("mixed").len(), 1);
    assert!(index.anchors("MiXeD").is_empty());
}
