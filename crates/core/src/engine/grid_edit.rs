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
//!
//! # Semantics not yet verified against the conformance fixtures
//!
//! No conformance fixture in this repo covers a structural edit — the fixture
//! pipeline evaluates formulas, it does not perform grid mutations — so the
//! rules below are the ones this module *asserts* rather than ones the repo
//! establishes. They follow the precedent the fill/paste transform set for its
//! own `#REF!` rule (treated as well-established, product-agnostic spreadsheet
//! convention rather than something needing live-Sheets verification), but
//! they should be pinned by the pipeline before anything depends on the exact
//! boundary behaviour:
//!
//! 1. `$` anchors do not exempt an axis from a structural shift (they do
//!    exempt one from a fill/paste translation).
//! 2. An insert at exactly a range's first row moves the whole range rather
//!    than expanding it; an insert at exactly its last row expands it; an
//!    insert one past its last row leaves it alone.
//! 3. A cell inside the deleted band becomes `#REF!`.
//! 4. A partially deleted range shrinks rather than erroring — its start
//!    clamps forward onto the cut, its end clamps back off it.
//! 5. A range whose whole span was deleted becomes `#REF!`.
//! 6. A backwards-written range (`A5:A1`) clamps by coordinate order, not by
//!    written order, and keeps its written orientation.
//! 7. A reference pushed past the grid bound by an insert becomes `#REF!`, and
//!    a range with even one endpoint pushed off becomes `#REF!` entirely.
//!    Sheets itself refuses such an insert rather than damaging formulas; this
//!    is the engine's convention (matching the fill/paste transform's grid
//!    rule), not observed product behaviour.
//! 8. `count: 0` and an `at` beyond the axis maximum are silent no-ops;
//!    `at: 0` is an error.

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
#[derive(Clone, Copy)]
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
        matches!(
            self,
            GridEdit::InsertRows { .. } | GridEdit::InsertColumns { .. }
        )
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
#[derive(Clone, Copy)]
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
        // The last deleted index. Saturating so `map_coord` is safe on its
        // own terms rather than relying on the `at >= 1` check in
        // `shift_refs_text`; `count == 0` degenerates to the identity map.
        let last = (at as u64 + count as u64).saturating_sub(1);
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

/// The coordinate of `addr` on the axis `edit` moves.
fn edited_coord(addr: CellAddr, edit: GridEdit) -> u32 {
    match edit.axis() {
        Axis::Row => addr.row,
        Axis::Column => addr.col,
    }
}

/// Apply `edit` to `addr`, leaving the untouched axis (and both `$` anchors)
/// as they are. `None` if the address no longer exists.
///
/// For [`Role::RangeEnd`] this can return a *sentinel* address with a `0`
/// coordinate — the "clamped back past the top of the grid" case, which
/// [`CellAddr::parse`] would reject. It is only ever compared against the
/// other endpoint, never rendered: the comparison in [`edited_ref_text`]
/// always finds the range wholly removed when it appears.
fn map_addr(addr: CellAddr, edit: GridEdit, role: Role) -> Option<CellAddr> {
    let (col, row) = match edit.axis() {
        Axis::Row => (addr.col, map_coord(addr.row, edit, role)?),
        Axis::Column => (map_coord(addr.col, edit, role)?, addr.row),
    };
    Some(CellAddr {
        col,
        row,
        col_abs: addr.col_abs,
        row_abs: addr.row_abs,
    })
}

/// Render `r` after `edit`. A reference that no longer exists — a cell in a
/// deleted band, a range whose every row/column went, or anything pushed off
/// the grid — becomes a bare `#REF!`, sheet qualifier and all: `Sheet1!#REF!`
/// is not re-parseable, so it would not survive a round trip.
fn edited_ref_text(r: &Ref, edit: GridEdit) -> String {
    let rewritten = match r {
        Ref::Cell { sheet, addr } => map_addr(*addr, edit, Role::Cell).map(|addr| Ref::Cell {
            sheet: sheet.clone(),
            addr,
        }),
        Ref::Range { sheet, start, end } => {
            // A range may be written backwards (`A5:A1`), so the clamping
            // roles follow which endpoint is actually lower on the edited
            // axis, not which one is written first.
            let inverted = edited_coord(*start, edit) > edited_coord(*end, edit);
            let (start_role, end_role) = if inverted {
                (Role::RangeEnd, Role::RangeStart)
            } else {
                (Role::RangeStart, Role::RangeEnd)
            };
            match (
                map_addr(*start, edit, start_role),
                map_addr(*end, edit, end_role),
            ) {
                (Some(new_start), Some(new_end)) => {
                    // Every row/column the range covered was deleted: the two
                    // clamped endpoints have crossed over each other.
                    let (a, b) = (edited_coord(new_start, edit), edited_coord(new_end, edit));
                    let survives = if inverted { a >= b } else { a <= b };
                    survives.then_some(Ref::Range {
                        sheet: sheet.clone(),
                        start: new_start,
                        end: new_end,
                    })
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
