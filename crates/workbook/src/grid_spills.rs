//! An index of the spill anchors sitting on the **stored** grid (issue #910).
//!
//! A cell inside a placed spill is a derived view: it is never authored and
//! never serialized, so resolving a read of one means finding the anchor whose
//! rectangle covers it and indexing into that anchor's stored array (schema
//! spec §5). During an *incremental* recalc the covering anchor may not be
//! dirty this pass, so it never enters the per-pass spill maps and the stored
//! grid is the only place its rectangle can be recovered from.
//!
//! [`GridResolver::grid_spilled_value`] used to recover it by scanning **every
//! authored cell on the sheet**, on every read of an empty cell, allocating a
//! [`CellRef`] per cell scanned — `O(cells on sheet)` for one read, when only
//! the handful of cells whose stored value is an array can possibly match.
//! This module builds that handful once and hands out the per-sheet slice, so
//! a read examines only real anchors.
//!
//! # Why once per recalc is enough
//!
//! Both inputs are fixed for the whole of a recompute:
//!
//! - the **stored grid** does not change until `apply_changes` runs, after the
//!   last pass; and
//! - the **recomputed** set (`to_eval`) is fixed when the recompute starts.
//!
//! So the `#591` exclusion below can be applied at build time instead of once
//! per scanned cell, which is also what removes the per-cell allocation.
//!
//! [`GridResolver::grid_spilled_value`]: crate::recalc

use std::collections::{BTreeMap, BTreeSet};

use icu_casemap::CaseMapperBorrowed;

use crate::address::Address;
use crate::casefold::simple_fold;
use crate::depgraph::CellRef;
use crate::spill::{spill_rect, SpillRect};
use crate::value::Value;
use crate::workbook::Workbook;

/// The stored grid's spill anchors, grouped by folded sheet name.
///
/// Built by [`GridSpillIndex::build`] once per recalc and shared by every
/// resolver of that recalc.
#[derive(Debug)]
pub(crate) struct GridSpillIndex {
    /// Folded sheet name → that sheet's anchors, in `Worksheet::iter` order
    /// (canonical A1 key order) — the order the replaced scan visited them in,
    /// so first-match resolution is unchanged. Sheets with no anchors are
    /// absent, which is the overwhelmingly common case.
    by_sheet: BTreeMap<String, Vec<(Address, SpillRect)>>,
}

impl GridSpillIndex {
    /// Indexes every in-bounds spill anchor on `workbook`'s stored grid,
    /// **excluding** anchors in `recomputed`.
    ///
    /// The exclusion is the `#591` rule: an anchor being recomputed this recalc
    /// still holds its *pre-recalc* array on the grid until `apply_changes`
    /// runs, so its stored rectangle is stale and must never be resolved
    /// through. Its authoritative spill state for this recalc is the per-pass
    /// `spills`/`prev_spills` maps. Without this, a reader of a cell an anchor
    /// *stops* spilling onto — because the anchor blocked or shrank — would
    /// resolve the vacated cell from the obsolete stored array.
    pub(crate) fn build(workbook: &Workbook, recomputed: &BTreeSet<CellRef>) -> Self {
        let folder = CaseMapperBorrowed::new();
        let mut by_sheet: BTreeMap<String, Vec<(Address, SpillRect)>> = BTreeMap::new();
        for sheet in workbook.sheets() {
            let folded = simple_fold(&folder, sheet.name());
            let mut anchors: Vec<(Address, SpillRect)> = Vec::new();
            for (addr, cell) in sheet.iter() {
                let Value::Array(rows) = cell.value() else {
                    continue;
                };
                let nrows = rows.len();
                let ncols = rows.first().map_or(0, Vec::len);
                let Some(rect) = spill_rect(addr, nrows, ncols) else {
                    continue; // an out-of-bounds rectangle is a blocked spill
                };
                let anchor = CellRef {
                    sheet: folded.clone(),
                    addr,
                };
                if recomputed.contains(&anchor) {
                    continue;
                }
                anchors.push((addr, rect));
            }
            if !anchors.is_empty() {
                by_sheet.insert(folded, anchors);
            }
        }
        Self { by_sheet }
    }

    /// The anchors on `sheet_folded`, in stored-grid order.
    ///
    /// This slice is exactly what a lookup walks, so its length is the number
    /// of cells one empty-cell read examines — a function of how many spills
    /// the sheet holds, never of how many cells it holds.
    pub(crate) fn anchors(&self, sheet_folded: &str) -> &[(Address, SpillRect)] {
        self.by_sheet.get(sheet_folded).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests;
