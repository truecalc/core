//! `translate_formula`: shift a formula's relative cell/range references by
//! (d_row, d_col) — the fill/copy-paste reference-adjustment transform.
//! `$`-absolute axes are left unchanged. See
//! `docs/superpowers/specs/2026-07-14-translate-formula-design.md`.

use crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW};
use crate::parser::CellAddr;

/// Shift `addr` by `(d_row, d_col)`, skipping any axis marked `$`-absolute.
/// Returns `None` if a *relative* axis lands outside the Sheets grid
/// (`1..=MAX_COL` / `1..=MAX_ROW`) — the caller renders that as `#REF!`.
pub(crate) fn shift_addr(addr: CellAddr, d_row: i64, d_col: i64) -> Option<CellAddr> {
    let col = if addr.col_abs { addr.col as i64 } else { addr.col as i64 + d_col };
    let row = if addr.row_abs { addr.row as i64 } else { addr.row as i64 + d_row };
    if (1..=MAX_COL as i64).contains(&col) && (1..=MAX_ROW as i64).contains(&row) {
        Some(CellAddr::new(col as u32, row as u32).with_col_abs(addr.col_abs).with_row_abs(addr.row_abs))
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
