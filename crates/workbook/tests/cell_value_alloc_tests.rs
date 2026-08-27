//! Allocation-free cell resolution (issues #904 and #910).
//!
//! `GridResolver::cell_value` is an early-out ladder: cycle set → `new_values`
//! → the stored grid → the two per-pass spill maps → `prev_values` → the
//! stored grid's spills → empty. A populated cell returns three branches in; an
//! **empty** cell misses every branch and pays the whole ladder. The two
//! defects pinned here sit on opposite sides of that asymmetry:
//!
//! - **#904** — every element scanned, populated or not, allocated an owned
//!   folded sheet name for its `CellRef` map key and re-resolved the target
//!   sheet by case-folding every sheet name in the workbook. Neither can change
//!   between the elements of one range.
//! - **#910** — an empty cell fell through to `grid_spilled_value`, which
//!   scanned **every authored cell on the sheet** and allocated a `CellRef` per
//!   cell scanned, just to find the handful whose stored value is an array. One
//!   empty-cell read cost `O(cells on sheet)`.
//!
//! Both are pinned in **allocation counts**, never wall-clock time, so they
//! hold on any machine and in either build profile and cannot go flaky — the
//! method `grid_lookup_alloc_tests.rs` established for #887. A wall clock would
//! also measure the inherent quadratic of a range scan (`=SUM(A$1:A{row})` down
//! a column genuinely visits ~N²/2 elements) rather than these constants.
//!
//! For #910 the count is additionally the exact scan length: the old scan
//! allocated exactly one `CellRef` sheet name per cell it examined, so
//! "allocations per empty-cell read" *is* "cells examined per empty-cell read",
//! and holding it flat as the sheet grows is what proves the scan is gone. The
//! complementary exact count — how many anchors a lookup examines — is
//! asserted directly in the `grid_spills` unit tests.
//!
//! Everything lives in one `#[test]` because a process-wide allocation counter
//! cannot be read from two tests running concurrently in the same binary.

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

/// The marginal cost of scanning one more populated element of a **block**
/// range, in allocations: 4.00 before #904 — an owned folded sheet name for the
/// `CellRef` map key, plus a fresh fold of every sheet name to re-resolve the
/// target sheet — and 0.00 after. This is the budget that pins `cell_value`
/// itself.
const MAX_ALLOCATIONS_PER_BLOCK_ELEMENT: f64 = 0.50;

/// The same figure for a **single-column** range: 6.00 before #904, 2.00 after.
///
/// The residual 2.00 is not `cell_value`. A single-column range is materialized
/// as core's Nx1 column shape — one nested one-element `Array` per row, so that
/// elementwise operations over it keep their column orientation and spill down
/// — and that wrapper is a `Vec` per element by construction. It is a
/// deliberate semantic, not a defect; #904's "6 per element" figure was this
/// shape, of which 4 were `cell_value`'s.
const MAX_ALLOCATIONS_PER_COLUMN_ELEMENT: f64 = 2.50;

/// The marginal cost of reading one more **empty** cell, in allocations. Before
/// #910 this was one allocation per authored cell on the sheet, per pass — it
/// grew with the sheet, measuring 2010, 4010 and 8010 at the three sizes below.
/// After, it is 0.00 at every size.
const MAX_ALLOCATIONS_PER_EMPTY_READ: f64 = 0.50;

/// Sheet sizes the empty-cell read is measured at, in authored rows. The point
/// is not the absolute figure but that it does not move as the sheet grows 4x.
const SHEET_SIZES: [u32; 3] = [1_000, 2_000, 4_000];

/// Empty rows read past the end of the authored block.
const EMPTY_ROWS: u32 = 100;

/// Columns in the block used for the #910 measurement — anything but 1, so the
/// Nx1 column wrapper above does not muddy the figure.
const BLOCK_COLS: u32 = 2;

#[test]
fn cell_reads_do_not_allocate_per_element_or_per_empty_cell() {
    // Warm up the recalc path (lazy statics, the function registry) first.
    black_box(allocations_to_recalc(64, 64, BLOCK_COLS));

    // ── 1. #904: a populated element stays within its budget ─────────────
    // Both recalcs read only populated cells, so neither reaches the
    // `grid_spilled_value` branch: this isolates the per-element constant.
    let small = allocations_to_recalc(1_000, 1_000, BLOCK_COLS);
    let large = allocations_to_recalc(2_000, 2_000, BLOCK_COLS);
    let per_block_element = (large - small) as f64 / f64::from(1_000 * BLOCK_COLS);
    eprintln!("allocations per populated block element: {per_block_element:.2}");
    assert!(
        per_block_element <= MAX_ALLOCATIONS_PER_BLOCK_ELEMENT,
        "scanning one more populated range element cost {per_block_element:.2} \
         allocations (budget {MAX_ALLOCATIONS_PER_BLOCK_ELEMENT:.2}); a range \
         element must not allocate an owned map key or re-fold the sheet names"
    );

    let small = allocations_to_recalc(1_000, 1_000, 1);
    let large = allocations_to_recalc(2_000, 2_000, 1);
    let per_column_element = (large - small) as f64 / 1_000.0;
    eprintln!("allocations per populated column element: {per_column_element:.2}");
    assert!(
        per_column_element <= MAX_ALLOCATIONS_PER_COLUMN_ELEMENT,
        "scanning one more element of a single-column range cost \
         {per_column_element:.2} allocations (budget \
         {MAX_ALLOCATIONS_PER_COLUMN_ELEMENT:.2}); only the Nx1 column wrapper \
         should remain"
    );

    // ── 2. #910: an empty read costs the same at every sheet size ────────
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
