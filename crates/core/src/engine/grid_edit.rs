//! `shift_refs_for_grid_edit`: rewrite the cell/range references in a formula
//! for a row/column **insert or delete** on one sheet — the structural-edit
//! reference-rewrite transform.
//!
//! This is not [`super::translate`]'s uniform offset. A structural edit moves
//! references *conditionally*, by their position relative to the edit, and can
//! remove one entirely:
//!
//! - references before the edit index do not move,
//! - references at or after it shift by `count` (an insert pushes them away,
//!   a delete pulls them back),
//! - a range straddling the edit grows (insert) or shrinks (delete),
//! - a reference whose every row/column was deleted becomes `#REF!`.
//!
//! Unlike fill/paste, `$` anchors do **not** exempt an axis: `$` controls how
//! a reference is *copied*, not which cell it points at, so `$A$5` tracks the
//! same cell through an insert exactly as `A5` does. The anchors themselves
//! are preserved.
//!
//! Only references that resolve to the edited sheet are touched — a bare
//! reference resolves to the sheet the formula lives on. String literals,
//! function names, defined names and `LET`/`LAMBDA` bindings are left alone,
//! the same contract [`super::rename`] documents for its own case; that falls
//! out of sharing `collect_shiftable_refs`. Mechanically this mirrors both
//! existing transforms: parse, collect the reference spans, splice
//! replacement text back into the original string right-to-left.

use crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW};
use crate::parser::{CellAddr, Ref};
use crate::types::{ErrorKind, ParseError};

use super::rename::same_sheet;
use super::translate::collect_shiftable_refs;

/// A row or column insert/delete on a single sheet, described by the 1-based
/// index of the first row/column affected and how many are inserted/deleted.
///
/// `InsertRows { at: 3, count: 2 }` inserts two blank rows so that they occupy
/// rows 3 and 4 and the old row 3 becomes row 5. `DeleteRows { at: 3, count: 2 }`
/// removes rows 3 and 4 so that the old row 5 becomes row 3.
///
/// `count: 0` is a no-op; `at: 0` is rejected (rows and columns are 1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GridEdit {
    /// Insert `count` rows above the current row `at`.
    InsertRows { at: u32, count: u32 },
    /// Delete the `count` rows starting at row `at`.
    DeleteRows { at: u32, count: u32 },
    /// Insert `count` columns to the left of the current column `at`.
    InsertColumns { at: u32, count: u32 },
    /// Delete the `count` columns starting at column `at`.
    DeleteColumns { at: u32, count: u32 },
}

/// Which axis of a [`CellAddr`] an edit moves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    Row,
    Column,
}

impl GridEdit {
    fn axis(self) -> Axis {
        match self {
            GridEdit::InsertRows { .. } | GridEdit::DeleteRows { .. } => Axis::Row,
            GridEdit::InsertColumns { .. } | GridEdit::DeleteColumns { .. } => Axis::Column,
        }
    }

    fn is_insert(self) -> bool {
        matches!(self, GridEdit::InsertRows { .. } | GridEdit::InsertColumns { .. })
    }

    fn at(self) -> u32 {
        match self {
            GridEdit::InsertRows { at, .. }
            | GridEdit::DeleteRows { at, .. }
            | GridEdit::InsertColumns { at, .. }
            | GridEdit::DeleteColumns { at, .. } => at,
        }
    }

    fn count(self) -> u32 {
        match self {
            GridEdit::InsertRows { count, .. }
            | GridEdit::DeleteRows { count, .. }
            | GridEdit::InsertColumns { count, .. }
            | GridEdit::DeleteColumns { count, .. } => count,
        }
    }

    /// Upper bound of the edited axis in the Sheets grid.
    fn axis_max(self) -> u32 {
        match self.axis() {
            Axis::Row => MAX_ROW as u32,
            Axis::Column => MAX_COL as u32,
        }
    }
}

/// What a coordinate is part of, which decides what happens when the edit
/// deletes it: a lone cell simply ceases to exist, while a range endpoint
/// collapses onto the cut — the start forward, the end back — so that a
/// partially deleted range shrinks instead of erroring.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    /// A single cell's coordinate.
    Cell,
    /// A range's start.
    RangeStart,
    /// A range's end.
    RangeEnd,
}

/// Map one coordinate on the edited axis. `None` means the coordinate no
/// longer exists — either deleted with nothing to clamp onto, or pushed off
/// the grid by an insert.
fn map_coord(v: u32, edit: GridEdit, role: Role) -> Option<u32> {
    let (at, count) = (edit.at(), edit.count());
    if edit.is_insert() {
        if v < at {
            return Some(v);
        }
        let shifted = v as u64 + count as u64;
        (shifted <= edit.axis_max() as u64).then_some(shifted as u32)
    } else {
        // `at + count - 1` is the last deleted index; `at >= 1` is enforced by
        // the caller and `count == 0` degenerates to the identity map.
        let last = at as u64 + count as u64 - 1;
        if (v as u64) < at as u64 {
            Some(v)
        } else if v as u64 > last {
            Some(v - count)
        } else {
            match role {
                // The cell itself was deleted; nothing to clamp onto.
                Role::Cell => None,
                // A deleted start clamps forward to `at`, the position the
                // first surviving row/column below the cut now occupies.
                Role::RangeStart => Some(at),
                // A deleted end clamps back to `at - 1`, the last position
                // above the cut. `at == 1` gives 0, which is below every
                // possible start, so the range reads as wholly removed.
                Role::RangeEnd => at.checked_sub(1),
            }
        }
    }
}

/// Apply `edit` to `addr`, leaving the untouched axis (and both `$` anchors)
/// as they are. `None` if the address no longer exists.
fn map_addr(addr: CellAddr, edit: GridEdit, role: Role) -> Option<CellAddr> {
    let (col, row) = match edit.axis() {
        Axis::Row => (addr.col, map_coord(addr.row, edit, role)?),
        Axis::Column => (map_coord(addr.col, edit, role)?, addr.row),
    };
    Some(CellAddr { col, row, col_abs: addr.col_abs, row_abs: addr.row_abs })
}

/// Render `r` after `edit`. A reference that no longer exists — a cell in a
/// deleted band, a range whose every row/column went, or anything pushed off
/// the grid — becomes a bare `#REF!`, sheet qualifier and all: `Sheet1!#REF!`
/// is not re-parseable, so it would not survive a round trip.
fn edited_ref_text(r: &Ref, edit: GridEdit) -> String {
    let rewritten = match r {
        Ref::Cell { sheet, addr } => map_addr(*addr, edit, Role::Cell)
            .map(|addr| Ref::Cell { sheet: sheet.clone(), addr }),
        Ref::Range { sheet, start, end } => {
            match (map_addr(*start, edit, Role::RangeStart), map_addr(*end, edit, Role::RangeEnd)) {
                (Some(start), Some(end)) => {
                    // The whole span was deleted: the clamped start now sits
                    // past the clamped end.
                    let (lo, hi) = match edit.axis() {
                        Axis::Row => (start.row, end.row),
                        Axis::Column => (start.col, end.col),
                    };
                    (lo <= hi).then_some(Ref::Range { sheet: sheet.clone(), start, end })
                }
                _ => None,
            }
        }
        Ref::Name(_) => unreachable!("collect_shiftable_refs never returns Ref::Name"),
        Ref::Table { .. } => unreachable!("collect_shiftable_refs never returns Ref::Table"),
    };
    match rewritten {
        Some(r) => r.to_string(),
        None => ErrorKind::Ref.to_string(),
    }
}

/// True if `r` resolves to `edited_sheet`. A bare reference resolves to
/// `formula_sheet`, the sheet the formula itself lives on.
fn targets_edited_sheet(r: &Ref, formula_sheet: &str, edited_sheet: &str) -> bool {
    let target = match r {
        Ref::Cell { sheet, .. } | Ref::Range { sheet, .. } => sheet.as_deref(),
        _ => return false,
    };
    same_sheet(target.unwrap_or(formula_sheet), edited_sheet)
}

/// Parse `formula`, rewrite every reference that resolves to `edited_sheet`
/// for `edit`, and splice the result back into the original text.
///
/// `formula_sheet` is the sheet the formula lives on — what a bare `A1`
/// resolves to. Sheet matching is case-insensitive, as in
/// [`super::rename`]. No-op if no reference resolves to `edited_sheet`.
pub(crate) fn shift_refs_text(
    formula: &str,
    formula_sheet: &str,
    edited_sheet: &str,
    edit: GridEdit,
) -> Result<String, ParseError> {
    if edit.at() == 0 {
        return Err(ParseError {
            message: "shift_refs_for_grid_edit: rows and columns are 1-based; `at` must be >= 1"
                .into(),
            position: 0,
        });
    }
    let expr = crate::parser::parse_formula(formula)?;
    let mut spans: Vec<_> = collect_shiftable_refs(&expr)
        .into_iter()
        .filter(|(_, r)| targets_edited_sheet(r, formula_sheet, edited_sheet))
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse(s.0.offset)); // right to left
    let mut out = formula.to_string();
    for (span, r) in spans {
        let replacement = edited_ref_text(&r, edit);
        let start = span.offset;
        let end = span.offset + span.length;
        out.replace_range(start..end, &replacement);
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
