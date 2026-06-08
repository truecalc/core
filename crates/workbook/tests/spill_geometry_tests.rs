//! Unit coverage for the spill geometry helpers (plan item 3.5, issue #537):
//! [`SpillRect`] membership, offset indexing, and the spilled-cell iterator,
//! plus the blocked-spill error constant. These exercise the pure geometry the
//! recalc engine and the `resolved` accessor share.

use truecalc_workbook::{Address, SpillRect, BLOCKED_SPILL_ERROR};

fn addr(r: u32, c: u32) -> Address {
    Address::new(r, c).expect("in-bounds address")
}

#[test]
fn blocked_spill_error_is_the_sheets_code() {
    // Schema spec §12 edge-case 1: Sheets reports a blocked array expansion as
    // #REF!. Pinned to the spec's worked example (not a fixture, see the
    // constant's docs).
    assert_eq!(BLOCKED_SPILL_ERROR, "#REF!");
}

#[test]
fn rect_contains_anchor_and_interior_but_not_outside() {
    // 2x3 rectangle anchored at B2 (row 2, col 2): covers rows 2..=3, cols 2..=4.
    let rect = SpillRect {
        anchor: addr(2, 2),
        rows: 2,
        cols: 3,
    };
    assert!(rect.contains(addr(2, 2))); // anchor
    assert!(rect.contains(addr(3, 4))); // far corner
    assert!(rect.contains(addr(2, 4)));
    assert!(!rect.contains(addr(1, 2))); // above
    assert!(!rect.contains(addr(2, 5))); // right of
    assert!(!rect.contains(addr(4, 2))); // below
}

#[test]
fn offset_of_indexes_row_major_from_the_anchor() {
    let rect = SpillRect {
        anchor: addr(5, 10),
        rows: 3,
        cols: 2,
    };
    assert_eq!(rect.offset_of(addr(5, 10)), Some((0, 0))); // anchor
    assert_eq!(rect.offset_of(addr(5, 11)), Some((0, 1)));
    assert_eq!(rect.offset_of(addr(7, 11)), Some((2, 1))); // far corner
    assert_eq!(rect.offset_of(addr(8, 10)), None); // out of range
}

#[test]
fn spilled_cells_excludes_the_anchor_and_lists_reading_order() {
    let rect = SpillRect {
        anchor: addr(1, 1),
        rows: 2,
        cols: 2,
    };
    let cells: Vec<Address> = rect.spilled_cells().collect();
    // Reading order over A1:B2 minus the anchor A1: B1, A2, B2.
    assert_eq!(cells, vec![addr(1, 2), addr(2, 1), addr(2, 2)]);
}
