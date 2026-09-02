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
//!
//! # `shift_refs_for_move`: the MOVE sibling
//!
//! [`AxisMove`] and [`shift_refs_for_move`] cover a third structural edit —
//! relocating a contiguous band of rows/columns elsewhere on the *same*
//! sheet, without inserting or deleting anything. This needs a genuinely
//! different algorithm from insert/delete's [`map_coord`], not another
//! variant of it: nothing is ever created or destroyed by a move, so every
//! coordinate on the moved axis maps to exactly one output coordinate (see
//! [`move_coord`]) — there is no `#REF!` case here, and no [`Role`] to pick
//! a clamp direction with.
//!
//! ## The three regions
//!
//! An [`AxisMove`] relocates the band `start..=end` so its first row/column
//! lands at `at`. Every coordinate on the axis falls into exactly one of:
//!
//! - inside the moved band (`start..=end`) — translates by `at - start`
//!   onto the band's new position,
//! - between the band's old and new position (the "gap-fill" zone) — slides
//!   by the band's width in the *opposite* direction of the move, closing
//!   the gap the band left or making room for where it landed,
//! - everywhere else — unchanged.
//!
//! `at` landing inside the band itself (`start..=end`, other than `at ==
//! start`, which is the literal identity) has no well-defined destination —
//! there is no way to "move a band to the middle of itself", and no real
//! spreadsheet UI can even express such a drag target — so the whole closed
//! interval `start..=end` is a no-op, the same way [`GridEdit`] treats its
//! own `count: 0` as a silent no-op rather than inventing a result. `at ==
//! end + 1` is deliberately *not* part of that no-op range: it is the
//! smallest genuine forward move, swapping the band with the immediately
//! following equal-width block.
//!
//! ## Corner swap on inversion
//!
//! [`move_coord`] is not monotonic — the band and its gap-fill zone move in
//! opposite directions relative to each other — so independently mapping a
//! range's two endpoints can flip their relative numeric order even when
//! they were written ascending to begin with. The issue's own canonical
//! example: moving rows 5:7 to before row 2 sends row 3's content to row 6
//! and row 6's content to row 3, so `A4:A6` (ascending: 4 <= 6) maps its
//! corners to A7 and A3 — inverted.
//!
//! [`moved_ref_text`] tells this apart from a range that was *already*
//! written backwards (e.g. `A7:A5`, left untouched by some unrelated move)
//! by recording whether the original range was ascending **before** either
//! corner is mapped, then comparing that to whether the *mapped* corners
//! come out ascending. A mismatch between the two means the move flipped
//! the written order — always an artifact of independent per-corner
//! mapping, never the user's intent — so it is corrected back, in whichever
//! direction the mismatch runs:
//!
//! - originally ascending, mapped corners come out descending: swap the two
//!   already-mapped corners back into ascending order.
//! - originally descending, mapped corners come out ascending (or equal):
//!   the mirror case — independent mapping "uncrossed" a range the user
//!   deliberately wrote backwards — swap back into descending order so it
//!   *stays* backwards, mirroring how [`edited_ref_text`] preserves a
//!   backwards-written range through insert/delete.
//! - either orientation, mapped corners keep the same relative order as the
//!   originals: render them exactly as computed, no swap.
//!
//! The `ascending` flag has to come from the *original* addresses: once both
//! corners are mapped, numeric order alone cannot distinguish an order flip
//! the move caused (needs correcting) from a range whose order was simply
//! never going to change (leave it alone).
//!
//! ## `$` anchors and out-of-bounds
//!
//! As with insert/delete, `$` anchors do not exempt an axis from a move —
//! `$` governs how a reference copies, not what it points at. [`move_addr`]
//! carries them through unconditionally, never inspecting them.
//!
//! Unlike insert/delete, a move never grows the sheet, so an out-of-bounds
//! *result* cannot happen from a well-formed [`AxisMove`] — but an
//! out-of-bounds *request* can (an `at` that leaves no room for the band
//! before the axis maximum). That is a property of the `AxisMove` itself,
//! not of any individual reference, so [`shift_refs_for_move`] rejects it
//! once at the entry point rather than threading an error path through
//! [`move_coord`] per reference.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
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

/// A row or column relocation on a single sheet: moves the contiguous band
/// `start..=end` (1-based, inclusive) so its first row/column lands at `at`
/// (1-based) after the move — Sheets' own convention: "move rows 5:7 to row
/// 2" means the band's new start is row 2, in either direction.
///
/// Unlike [`GridEdit`], nothing is created or destroyed: every coordinate on
/// `axis` maps to exactly one output coordinate (see [`move_coord`]).
///
/// A move is backward when `at < start` and forward when `at > end`. `at`
/// landing inside the band itself (`start..=end`) is a no-op — see the
/// module doc. `start == 0` or `at == 0` is rejected (rows/columns are
/// 1-based); `start > end` is rejected as a malformed band.
///
/// A well-formed `AxisMove` also has `end` itself within the sheet's grid
/// bounds — that precondition is on the caller, the same way it is on
/// `start <= end`. [`shift_refs_for_move`] validates the *destination*
/// footprint (`at ..= at + width - 1`) against the axis maximum, since an
/// out-of-bounds destination is the one thing a move can newly request that
/// insert/delete cannot, but it does not separately re-validate `end`
/// against that maximum: an `end` already past the grid could only reach
/// this function from a caller that mis-described the sheet's own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisMove {
    pub axis: Axis,
    pub start: u32,
    pub end: u32,
    pub at: u32,
}

impl AxisMove {
    /// Upper bound of the moved axis in the Sheets grid.
    fn axis_max(self) -> u32 {
        match self.axis {
            Axis::Row => MAX_ROW as u32,
            Axis::Column => MAX_COL as u32,
        }
    }

    /// How many rows/columns the band spans.
    fn width(self) -> u32 {
        self.end - self.start + 1
    }
}

/// The coordinate of `addr` on `axis`.
fn axis_coord(addr: CellAddr, axis: Axis) -> u32 {
    match axis {
        Axis::Row => addr.row,
        Axis::Column => addr.col,
    }
}

/// Total remap of one coordinate on `mv`'s axis: every `v` maps to exactly
/// one output coordinate, since a move never creates or destroys a
/// row/column — contrast [`map_coord`], which can return `None`. Callers
/// must exclude the no-op range (`mv.at` inside `mv.start..=mv.end`) first;
/// see the module doc for why calling this inside that range would produce
/// a nonsensical cyclic permutation rather than a real answer.
fn move_coord(v: u32, mv: AxisMove) -> u32 {
    let width = mv.width();
    if (mv.start..=mv.end).contains(&v) {
        // Inside the moved band: translate the in-band offset onto the
        // band's new start.
        return v - mv.start + mv.at;
    }
    if mv.at < mv.start {
        // Backward move: the band vacates start..=end and slides down to
        // at..=(at+width-1). Rows/columns strictly between the new and old
        // start slide forward by the band's width to close the gap.
        if (mv.at..mv.start).contains(&v) {
            return v + width;
        }
    } else {
        // Forward move (mv.at > mv.end, guaranteed once the no-op range
        // above is excluded): rows/columns between the old end and the
        // band's new, wider footprint slide back by the band's width.
        let gap_end = mv.at + width - 1;
        if (mv.end + 1..=gap_end).contains(&v) {
            return v - width;
        }
    }
    v
}

/// Apply `mv` to `addr`, leaving the untouched axis (and both `$` anchors)
/// exactly as they are. Total: unlike [`map_addr`], this never returns
/// `None` — a move never drops a reference.
fn move_addr(addr: CellAddr, mv: AxisMove) -> CellAddr {
    let (col, row) = match mv.axis {
        Axis::Row => (addr.col, move_coord(addr.row, mv)),
        Axis::Column => (move_coord(addr.col, mv), addr.row),
    };
    CellAddr {
        col,
        row,
        col_abs: addr.col_abs,
        row_abs: addr.row_abs,
    }
}

/// Render `r` after the move `mv`. Unlike [`edited_ref_text`], nothing is
/// ever dropped, so there is no `#REF!` case and no [`Role`] to pick a clamp
/// direction with — a single-cell reference and each range endpoint go
/// through the same [`move_coord`] call. See the module doc for the
/// corner-swap reasoning below.
fn moved_ref_text(r: &Ref, mv: AxisMove) -> String {
    let rewritten = match r {
        Ref::Cell { sheet, addr } => Ref::Cell {
            sheet: sheet.clone(),
            addr: move_addr(*addr, mv),
        },
        Ref::Range { sheet, start, end } => {
            // Record orientation from the ORIGINAL addresses, before either
            // corner is mapped: move_coord is not monotonic, so mapping the
            // two corners independently can flip their numeric order even
            // when they were written ascending — or "unflip" a range that
            // was deliberately written descending. Once both are mapped
            // there is no way to tell an order flip the move caused (needs
            // correcting) apart from one that was simply never going to
            // change (leave it), so the comparison has to be made against
            // this original flag, not against the mapped coordinates alone.
            let ascending = axis_coord(*start, mv.axis) <= axis_coord(*end, mv.axis);
            let new_start = move_addr(*start, mv);
            let new_end = move_addr(*end, mv);
            let mapped_ascending = axis_coord(new_start, mv.axis) <= axis_coord(new_end, mv.axis);
            let (start, end) = if ascending == mapped_ascending {
                // The mapped corners kept the same relative order the
                // originals had (both ascending or both descending): render
                // exactly as computed, no swap.
                (new_start, new_end)
            } else {
                // The move flipped the order: an originally-ascending range
                // came out descending (swap back to ascending), or an
                // originally-descending range came out ascending — the
                // mapping "uncrossed" a range the user deliberately wrote
                // backwards, so swap back to keep it descending. Either way
                // this is the mapping's artifact, never the user's intent.
                (new_end, new_start)
            };
            Ref::Range {
                sheet: sheet.clone(),
                start,
                end,
            }
        }
        Ref::Name(_) => unreachable!("collect_shiftable_refs never returns Ref::Name"),
        Ref::Table { .. } => unreachable!("collect_shiftable_refs never returns Ref::Table"),
    };
    rewritten.to_string()
}

/// Parse `formula`, rewrite every reference that resolves to `edited_sheet`
/// for the move `mv`, and splice the result back into the original text.
///
/// `formula_sheet` is the sheet the formula lives on — what a bare `A1`
/// resolves to. Sheet matching is case-insensitive, as in
/// [`shift_refs_text`]. No-op if no reference resolves to `edited_sheet`,
/// and also if `mv.at` lands inside `mv.start..=mv.end` (see the module
/// doc).
pub(crate) fn shift_refs_for_move(
    formula: &str,
    formula_sheet: &str,
    edited_sheet: &str,
    mv: AxisMove,
) -> Result<String, ParseError> {
    if mv.start == 0 || mv.at == 0 {
        return Err(ParseError {
            message:
                "shift_refs_for_move: rows and columns are 1-based; `start` and `at` must be >= 1"
                    .into(),
            position: 0,
        });
    }
    if mv.start > mv.end {
        return Err(ParseError {
            message: "shift_refs_for_move: `start` must be <= `end`".into(),
            position: 0,
        });
    }
    let expr = crate::parser::parse_formula(formula)?;
    if (mv.start..=mv.end).contains(&mv.at) {
        // `at` inside the band has no well-defined destination: a no-op,
        // not an error — see the module doc.
        return Ok(formula.to_string());
    }
    let width = mv.end as u64 - mv.start as u64 + 1;
    if mv.at as u64 + width - 1 > mv.axis_max() as u64 {
        return Err(ParseError {
            message: "shift_refs_for_move: destination pushes the band off the grid".into(),
            position: 0,
        });
    }
    let mut spans: Vec<_> = collect_shiftable_refs(&expr)
        .into_iter()
        .filter(|(_, r)| targets_edited_sheet(r, formula_sheet, edited_sheet))
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse(s.0.offset)); // right to left
    let mut out = formula.to_string();
    for (span, r) in spans {
        let replacement = moved_ref_text(&r, mv);
        let start = span.offset;
        let end = span.offset + span.length;
        out.replace_range(start..end, &replacement);
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
