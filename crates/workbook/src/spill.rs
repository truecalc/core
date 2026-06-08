//! Array-spill geometry and the blocked-spill policy (plan item 3.5, issue
//! #537), shared between the recalc engine ([`crate::recalc`]) and the
//! spill-resolving cell accessor ([`Workbook::get`](crate::Workbook::get)).
//!
//! # The model (schema spec §5)
//!
//! A formula whose result is an `m × n` array is a **spill anchor**: it stores
//! the full array (schema spec §6 `array`) and *occupies* the rectangle
//! `(r..r+m-1, c..c+n-1)` anchored at the formula cell `(r, c)`, in reading
//! order. The non-anchor cells of that rectangle are **spilled** cells: they
//! are a derived view, never authored and **never serialized** — only the
//! anchor's array is on the wire (schema spec §5, superseding the plan's
//! "spilled marker" sketch). A consumer reconstructs the grid by the five-line
//! rule in §5; this module is that rule.
//!
//! # Blocked spill (Sheets semantics)
//!
//! A spill is **blocked** — the anchor takes the Sheets blocked-spill error
//! ([`BLOCKED_SPILL_ERROR`]) and **no array is stored** — when, per §5:
//!
//! - any non-anchor cell of the rectangle is *occupied* (an authored literal or
//!   formula, or a cell already claimed by another spill resolved earlier this
//!   recalc), or
//! - the rectangle would extend **beyond the sheet's address bounds** (limits
//!   ADR) — treated as blocked pending fixture verification (§5).
//!
//! Spill resolution runs in deterministic recalc order (topological, tie-broken
//! by sheet position → row → column — scope ADR Decision 3), so two anchors
//! competing for the same cell resolve reproducibly: the earlier anchor wins
//! and the later one blocks.
//!
//! A 1×1 array never reaches here: it is collapsed to its scalar before storage
//! (schema spec §6), so [`Value::Array`](crate::Value::Array) always describes a
//! spill of at least two cells.

use crate::address::Address;
use crate::limits::{MAX_COLUMN, MAX_ROW};

/// The Sheets blocked-spill error code (schema spec §12 edge-case 1: Sheets
/// reports a blocked array expansion as `#REF!`).
///
/// A dedicated workbook **blocked-spill** fixture is not yet exported into the
/// core crate's fixture tree (the P3.6 set committed there covers cross-sheet,
/// named-range, and date-type rows — see `crates/core/tests/fixtures/`), so this
/// exact code is **not fixture-pinned in this crate**; the schema spec's §12
/// worked example is the authority used here, and the in-repo spill tests assert
/// the *behavior* (a blocked anchor takes this error, stores no array, and
/// leaves the target cells unspilled). The code is re-verified against a
/// `spill` fixture when one lands in core (issue note, mirrors
/// [`crate::recalc::CIRCULAR_ERROR`]'s handling).
pub const BLOCKED_SPILL_ERROR: &str = "#REF!";

/// The rectangle an `rows × cols` array spills into, anchored at `anchor`
/// (reading order): inclusive corners `(anchor.row, anchor.column)` ..
/// `(anchor.row + rows - 1, anchor.column + cols - 1)`.
///
/// Returns `None` if the rectangle would leave the sheet's address bounds (a
/// blocked, out-of-bounds spill — §5). `rows`/`cols` are the dimensions of a
/// stored `array` value, hence always ≥ 1 (and, jointly, ≥ 2 — §6).
pub fn spill_rect(anchor: Address, rows: usize, cols: usize) -> Option<SpillRect> {
    debug_assert!(rows >= 1 && cols >= 1, "an array value is non-empty (§6)");
    let last_row = anchor.row as u64 + rows as u64 - 1;
    let last_col = anchor.column as u64 + cols as u64 - 1;
    if last_row > MAX_ROW as u64 || last_col > MAX_COLUMN as u64 {
        return None;
    }
    Some(SpillRect {
        anchor,
        rows: rows as u32,
        cols: cols as u32,
    })
}

/// An in-bounds spill rectangle: the geometry of one anchor's reconstructed
/// array. Cheap to copy; membership and indexing are `O(1)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpillRect {
    /// The anchor (top-left) cell — the authored formula cell.
    pub anchor: Address,
    /// Row count of the array (≥ 1).
    pub rows: u32,
    /// Column count of the array (≥ 1).
    pub cols: u32,
}

impl SpillRect {
    /// Whether `addr` lies inside this rectangle (anchor included).
    pub fn contains(&self, addr: Address) -> bool {
        addr.row >= self.anchor.row
            && addr.row < self.anchor.row + self.rows
            && addr.column >= self.anchor.column
            && addr.column < self.anchor.column + self.cols
    }

    /// The `(i, j)` offset of `addr` within this rectangle (anchor = `(0, 0)`),
    /// or `None` if `addr` is outside it. The element of the anchor's array at
    /// `[i][j]` is the value spilled to `addr`.
    pub fn offset_of(&self, addr: Address) -> Option<(usize, usize)> {
        if !self.contains(addr) {
            return None;
        }
        Some((
            (addr.row - self.anchor.row) as usize,
            (addr.column - self.anchor.column) as usize,
        ))
    }

    /// Iterates the **non-anchor** cells of the rectangle (the spilled cells),
    /// in reading order. The addresses are all in-bounds by construction.
    pub fn spilled_cells(&self) -> impl Iterator<Item = Address> + '_ {
        let anchor = self.anchor;
        (0..self.rows).flat_map(move |i| {
            (0..self.cols).filter_map(move |j| {
                if i == 0 && j == 0 {
                    return None; // the anchor is authored, not spilled
                }
                Address::new(anchor.row + i, anchor.column + j)
            })
        })
    }
}
