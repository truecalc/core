//! A per-sheet index of **authored** cells, built once and then asked
//! "does this rectangle contain a cell nobody authored?" (issue #927).
//!
//! # Why this exists
//!
//! `seed_spill_sensitive` asks that question once per range precedent of every
//! formula cell, and it used to answer it by scanning every authored cell on
//! the sheet and counting the ones inside the rectangle. One seeding pass over
//! a workbook of `F` range-reading formulas on a sheet of `A` authored cells
//! therefore cost `O(F * A)` — and the shapes that cost the most (a `SUM` row
//! total per row, a block subtotal per block) are exactly the ones where every
//! cell of every range *is* authored, so the scan ran in full and found
//! nothing, every time.
//!
//! # Why not the dependency graph's row index (issue #908)
//!
//! That index keys **formula** cells. This question is about **authored**
//! cells, and an authored cell is very often a literal — `=SUM(A2:E2)` over
//! five typed-in numbers reads a fully authored range holding no formula at
//! all. Answering from the formula index would call that range unauthored and
//! seed its reader on every recalc: the exact cost this module removes, kept
//! and re-labelled. The two populations differ, so they need two indexes.
//!
//! # Shape
//!
//! Per sheet: the authored-cell count, and the authored columns of each
//! **occupied** row in ascending order. Empty rows are not keyed, so a
//! reference spanning ten million mostly-empty rows costs nothing for the
//! empty ones.

use std::collections::{BTreeMap, HashMap};

use icu_casemap::CaseMapperBorrowed;

use crate::casefold::simple_fold;
use crate::depgraph::RangeRef;
use crate::workbook::Workbook;

/// One sheet's authored cells, indexed by row.
struct SheetRows {
    /// Authored cells on the sheet, all rows together. Bounds the area of any
    /// fully authored rectangle, which is what makes the oversized-rectangle
    /// answer constant-time.
    total: u64,
    /// Occupied row → that row's authored columns, ascending. Rows holding no
    /// authored cell are absent, never stored as an empty entry.
    rows: BTreeMap<u32, Vec<u32>>,
}

/// Every sheet's authored cells, indexed by folded sheet name and row.
///
/// Built once per incremental seeding pass in `O(authored cells)` — the cost of
/// a single one of the scans it replaces — and then queried once per range
/// precedent.
#[doc(hidden)]
pub struct AuthoredCellIndex {
    sheets: HashMap<String, SheetRows>,
}

impl AuthoredCellIndex {
    /// Indexes every authored cell of every sheet in `workbook`.
    #[doc(hidden)]
    pub fn build(workbook: &Workbook) -> Self {
        let folder = CaseMapperBorrowed::new();
        let mut sheets = HashMap::with_capacity(workbook.sheets().len());
        for sheet in workbook.sheets() {
            let mut rows: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
            for (addr, _) in sheet.iter() {
                rows.entry(addr.row).or_default().push(addr.column);
            }
            // A worksheet iterates in canonical A1-key order (`A10` before
            // `A2`), which is not column order within a row.
            for columns in rows.values_mut() {
                columns.sort_unstable();
            }
            sheets.insert(
                simple_fold(&folder, sheet.name()),
                SheetRows {
                    total: sheet.len() as u64,
                    rows,
                },
            );
        }
        Self { sheets }
    }

    /// Whether the rectangle `r` contains at least one cell that is **not**
    /// authored (so it is empty, or occupied only by a spill).
    ///
    /// Answers exactly what comparing the rectangle's area to the authored
    /// cells inside it answered, including its two conservative edges: a range
    /// on a sheet the workbook does not have is (vacuously) all-unauthored, and
    /// so is a rectangle whose corners arrive the wrong way round.
    #[doc(hidden)]
    pub fn range_has_unauthored_cell(&self, r: &RangeRef) -> bool {
        self.range_has_unauthored_cell_examined(r).0
    }

    /// [`range_has_unauthored_cell`](Self::range_has_unauthored_cell), plus how
    /// many authored cells the lookup examined to reach its answer.
    ///
    /// Instrumentation, not a feature: this change is a change in *how much is
    /// examined* per seeding decision, and wall-clock is too machine-dependent
    /// to pin in a test. Both values come out of the one lookup, so the count
    /// cannot drift from what the lookup actually does.
    ///
    /// The count is authored cells probed. The lookup also walks one B-tree
    /// node per *occupied row the rectangle spans*, which is a property of the
    /// rectangle, not of the sheet.
    #[doc(hidden)]
    pub fn range_has_unauthored_cell_examined(&self, r: &RangeRef) -> (bool, usize) {
        let mut examined = 0usize;

        // `RangeRef`'s fields are public, so the corners can arrive the wrong
        // way round. Such a rectangle holds no cell at all; seed conservatively
        // rather than subtract the corners (which underflows) or hand
        // `BTreeMap::range` an inverted bound (which panics).
        if r.start.row > r.end.row || r.start.column > r.end.column {
            return (true, examined);
        }
        let Some(sheet) = self.sheets.get(&r.sheet) else {
            // The range targets a missing sheet; nothing authored there.
            return (true, examined);
        };

        let height = u64::from(r.end.row - r.start.row) + 1;
        let width = u64::from(r.end.column - r.start.column) + 1;
        let area = height.saturating_mul(width);

        // Constant time: a rectangle with more cells than the sheet has
        // authored anywhere cannot be fully authored.
        if area > sheet.total {
            return (true, examined);
        }

        // Otherwise every row the rectangle spans must be occupied, and
        // occupied across the rectangle's full width. The first row that is
        // not ends the walk.
        let mut expected = u64::from(r.start.row);
        for (&row, columns) in sheet.rows.range(r.start.row..=r.end.row) {
            if u64::from(row) != expected {
                return (true, examined); // a row with nothing authored on it
            }
            let lo = partition_point(columns, &mut examined, |c| c < r.start.column);
            let hi = partition_point(columns, &mut examined, |c| c <= r.end.column);
            if (hi - lo) as u64 != width {
                return (true, examined);
            }
            expected += 1;
        }
        // Trailing rows the walk never reached hold nothing at all.
        (expected <= u64::from(r.end.row), examined)
    }
}

/// [`slice::partition_point`], adding one to `examined` per element actually
/// probed — the only reason this is written out rather than called.
///
/// `columns` is ascending and `keep` is monotone (true for a prefix), so the
/// answer is the length of that prefix.
fn partition_point(columns: &[u32], examined: &mut usize, keep: impl Fn(u32) -> bool) -> usize {
    let (mut lo, mut hi) = (0usize, columns.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        *examined += 1;
        if keep(columns[mid]) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}
