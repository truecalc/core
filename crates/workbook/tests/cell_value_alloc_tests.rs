//! An empty-cell read must not scan the sheet (issue #910).
//!
//! `GridResolver::cell_value` is an early-out ladder: cycle set → `new_values`
//! → the stored grid → the two per-pass spill maps → `prev_values` → the
//! stored grid's spills → empty. A populated cell returns three branches in; an
//! **empty** cell misses every branch and pays the whole ladder — and its last
//! rung, `grid_spilled_value`, scanned **every authored cell on the sheet**,
//! allocating a `CellRef` per cell scanned, just to find the handful whose
//! stored value is an array. One empty-cell read cost `O(cells on sheet)`.
//!
//! That asymmetry is why it hid: a benchmark whose range scans read populated
//! cells never reaches the branch at all.
//!
//! This is pinned in **allocation counts**, never wall-clock time, so it holds
//! on any machine and in either build profile and cannot go flaky — the method
//! `grid_lookup_alloc_tests.rs` established for #887. A wall clock would also
//! measure the inherent quadratic of a range scan (`=SUM(A$1:A{row})` down a
//! column genuinely visits ~N²/2 elements) rather than this constant.
//!
//! The count is here also the exact scan length: the old scan allocated exactly
//! one `CellRef` sheet name per cell it examined, so "allocations per
//! empty-cell read" *is* "cells examined per empty-cell read", and holding it
//! flat as the sheet grows is what proves the scan is gone. The complementary
//! exact count — how many anchors a lookup examines — is asserted directly in
//! the `grid_spills` unit tests.
//!
//! The counter is process-wide, so it cannot be read from two tests running
//! concurrently in the same binary.

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

use truecalc_workbook::{
    Address, Cell, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// Every allocation the process makes. `realloc` is not overridden, so the
/// default `GlobalAlloc::realloc` routes through `alloc` and is counted too.
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Runs `body` and returns how many allocations it made.
fn allocations_during<T>(body: impl FnOnce() -> T) -> usize {
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    black_box(body());
    ALLOCATIONS.load(Ordering::Relaxed) - before
}

/// A workbook whose single formula sums the `cols`-wide block spanning rows
/// `1..=scanned`, over a sheet holding literals in rows `1..=authored` of those
/// same columns.
///
/// Rows `authored+1 ..= scanned` are **empty**, so one pass makes exactly
/// `cols * (scanned - authored)` empty-cell reads. The formula itself sits in
/// column T, clear of the block.
fn scan_workbook(authored: u32, scanned: u32, cols: u32) -> Workbook {
    let mut sheet = Worksheet::new("Sheet1");
    for row in 1..=authored {
        for col in 1..=cols {
            sheet.cells_mut().insert(
                Address::new(row, col).unwrap().to_a1(),
                Cell::literal(Value::Number(f64::from(row))).unwrap(),
            );
        }
    }
    let last = Address::new(1, cols).unwrap().to_a1();
    let last_col = last.trim_end_matches('1');
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(sheet).unwrap();
    wb.set(
        "Sheet1",
        Address::new(1, 20).unwrap(),
        CellInput::Formula(format!("=SUM(A$1:{last_col}{scanned})")),
    )
    .unwrap();
    wb
}

fn allocations_to_recalc(authored: u32, scanned: u32, cols: u32) -> usize {
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    let mut wb = scan_workbook(authored, scanned, cols);
    allocations_during(|| wb.recalc(&ctx))
}

/// The marginal cost of reading one more **empty** cell, in allocations.
///
/// Before #910 this was one allocation per authored cell on the sheet, per
/// pass: it *grew with the sheet*, measuring 4008, 8008 and 16008 at the three
/// sizes below. After, it is flat at 4.00 — and that residual is not the scan
/// at all but `cell_value`'s per-element constant, which every element pays
/// whether it is empty or not (#904). Flatness is the property this budget
/// exists to hold.
const MAX_ALLOCATIONS_PER_EMPTY_READ: f64 = 4.50;

/// Sheet sizes the empty-cell read is measured at, in authored rows. The point
/// is not the absolute figure but that it does not move as the sheet grows 4x.
const SHEET_SIZES: [u32; 3] = [1_000, 2_000, 4_000];

/// Empty rows read past the end of the authored block.
const EMPTY_ROWS: u32 = 100;

/// Columns in the block read by the measurement. Anything but 1: a
/// single-column range is materialized as core's Nx1 column shape, one
/// nested `Array` per row, which would add a per-element allocation of its
/// own to the figure.
const BLOCK_COLS: u32 = 2;

#[test]
fn an_empty_cell_read_costs_the_same_whatever_the_sheet_size() {
    // Warm up the recalc path (lazy statics, the function registry) first.
    black_box(allocations_to_recalc(64, 64, BLOCK_COLS));

    let mut measured: Vec<(u32, f64)> = Vec::new();
    for authored in SHEET_SIZES {
        // Difference two recalcs over the *same* sheet so every fixed cost —
        // the grid build, the graph, the populated part of the scan — cancels
        // and only the extra empty reads remain.
        let one_k = allocations_to_recalc(authored, authored + EMPTY_ROWS, BLOCK_COLS);
        let two_k = allocations_to_recalc(authored, authored + 2 * EMPTY_ROWS, BLOCK_COLS);
        let extra_reads = f64::from(EMPTY_ROWS * BLOCK_COLS);
        let per_empty_read = (two_k - one_k) as f64 / extra_reads;
        eprintln!(
            "allocations per empty-cell read on a {} cell sheet: {per_empty_read:.2}",
            authored * BLOCK_COLS
        );
        measured.push((authored * BLOCK_COLS, per_empty_read));
    }

    for &(cells, per_empty_read) in &measured {
        assert!(
            per_empty_read <= MAX_ALLOCATIONS_PER_EMPTY_READ,
            "reading one more empty cell on a {cells}-cell sheet cost \
             {per_empty_read:.2} allocations (budget \
             {MAX_ALLOCATIONS_PER_EMPTY_READ:.2}); an empty-cell read must look \
             the sheet's spill anchors up, not scan every authored cell: \
             {measured:?}"
        );
    }
}
