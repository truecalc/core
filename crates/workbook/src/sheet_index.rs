//! The workbook's sheet list, indexed by name — built once per recalc or per
//! dependency-graph build instead of re-scanned per cell (issue #952).
//!
//! Resolving a sheet name used to be
//! `sheets().iter().position(|s| simple_fold(s.name()) == target)`: a linear
//! walk that case-folded — and so allocated — **every sheet name it passed**.
//! That scan ran once per formula cell in six places on the recalc path and
//! once per cross-sheet reference during graph build, which made recalculation
//! `O(cells × sheets × sheet-name-length)`. Profiled on a 200-sheet workbook,
//! 90% of `Workbook::recalc` was inside it, and naming a tab
//! `Cash Flow Statement 2027` instead of `S17` cost 3.6× the wall clock.
//!
//! [`SheetIndex`] does the folding once per sheet and answers every subsequent
//! lookup from a hash map, so a lookup is `O(1)` in the sheet count and — on
//! the folded path — performs no fold at all.
//!
//! # Agreement with the scan it replaces
//!
//! The scan it replaces is `position`, so it is **first-wins**: when two sheet
//! names fold to the same key (constructible through
//! [`Workbook::sheets_mut`](crate::Workbook::sheets_mut), which can push a
//! sheet named `DATA` into a workbook that already has `Data`), the lookup
//! must answer with the *earlier* tab. Both maps here are first-wins, and the
//! authored-name map is populated from the folded map rather than from the
//! sheet's own position, so an exact-spelling hit and a folded hit can never
//! disagree.

use std::collections::HashMap;

use icu_casemap::CaseMapperBorrowed;

use crate::casefold::simple_fold;
use crate::Workbook;

/// Every sheet's tab index, keyed by folded name and by authored name.
///
/// Build one per recalc / per graph build and share it across every cell of
/// that pass — the same lifetime rule the shared `Engine` (issue #886) and
/// `GridSpillIndex` (issue #910) follow. It borrows nothing from the workbook,
/// so it goes stale the moment the sheet list changes; never store one.
#[doc(hidden)]
pub struct SheetIndex {
    /// Folded name → tab index, first-wins.
    by_folded: HashMap<String, usize>,
    /// Authored (as-written) name → tab index, first-wins. Present so a
    /// reference that spells the sheet name exactly as the tab does — the
    /// overwhelmingly common case in real formulas — resolves without folding
    /// anything at all. Each entry's value is looked up *through* `by_folded`,
    /// so it answers what the scan would have answered even for names that
    /// fold together.
    by_authored: HashMap<String, usize>,
    /// Each sheet's folded name, by tab index.
    folded: Vec<String>,
}

impl SheetIndex {
    /// Indexes every sheet of `workbook`. Folds each sheet name exactly once,
    /// so building costs `O(total sheet-name length)` — once, rather than once
    /// per cell.
    #[doc(hidden)]
    pub fn build(workbook: &Workbook) -> Self {
        let folder = CaseMapperBorrowed::new();
        let sheets = workbook.sheets();
        let folded: Vec<String> = sheets
            .iter()
            .map(|s| simple_fold(&folder, s.name()))
            .collect();

        let mut by_folded: HashMap<String, usize> = HashMap::with_capacity(sheets.len());
        for (i, name) in folded.iter().enumerate() {
            by_folded.entry(name.clone()).or_insert(i);
        }

        let mut by_authored: HashMap<String, usize> = HashMap::with_capacity(sheets.len());
        for (i, sheet) in sheets.iter().enumerate() {
            // Through `by_folded`, never `i`: `["DATA", "Data"]` must resolve
            // *both* spellings to tab 0, exactly as `position` does.
            let resolved = by_folded[&folded[i]];
            by_authored
                .entry(sheet.name().to_owned())
                .or_insert(resolved);
        }

        Self {
            by_folded,
            by_authored,
            folded,
        }
    }

    /// The tab index of the sheet whose folded name is `folded`, or `None`.
    /// Performs no fold: the caller already holds a folded key (every
    /// [`CellRef`](crate::CellRef) does).
    #[doc(hidden)]
    pub fn index_of_folded(&self, folded: &str) -> Option<usize> {
        self.by_folded.get(folded).copied()
    }

    /// The tab index of the sheet named `name` (case-insensitive), or `None` —
    /// the indexed equivalent of [`Workbook::sheet_index`](crate::Workbook::sheet_index).
    ///
    /// Folds `name` only when it does not match a tab's authored spelling
    /// exactly, so the common case (a formula that spells the sheet the way the
    /// tab does) costs no fold and is insensitive to the name's length.
    #[doc(hidden)]
    pub fn index_of_name(&self, name: &str) -> Option<usize> {
        if let Some(i) = self.by_authored.get(name) {
            return Some(*i);
        }
        let folder = CaseMapperBorrowed::new();
        self.by_folded.get(&simple_fold(&folder, name)).copied()
    }

    /// The folded name of tab `index`.
    #[doc(hidden)]
    pub fn folded_name(&self, index: usize) -> &str {
        &self.folded[index]
    }

    /// The folded name of the sheet named `name` (case-insensitive), or `None`
    /// when the workbook has no such sheet. One lookup where callers used to
    /// do a scan (`workbook.sheet(name)`) *and* a fold.
    #[doc(hidden)]
    pub fn folded_of_name(&self, name: &str) -> Option<&str> {
        self.index_of_name(name).map(|i| self.folded_name(i))
    }

    /// How many sheets are indexed.
    #[doc(hidden)]
    pub fn len(&self) -> usize {
        self.folded.len()
    }

    /// Whether the workbook has no sheets.
    #[doc(hidden)]
    pub fn is_empty(&self) -> bool {
        self.folded.is_empty()
    }
}
