//! `translate_formula`: shift a formula's relative cell/range references by
//! (d_row, d_col) — the fill/copy-paste reference-adjustment transform.
//! `$`-absolute axes are left unchanged. See
//! `docs/superpowers/specs/2026-07-14-translate-formula-design.md`.

use crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW};
use crate::parser::refs::write_sheet;
use crate::parser::{CellAddr, Ref};
use crate::types::ErrorKind;

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

fn addr_text(addr: CellAddr, d_row: i64, d_col: i64) -> String {
    match shift_addr(addr, d_row, d_col) {
        Some(shifted) => shifted.to_string(),
        None => ErrorKind::Ref.to_string(),
    }
}

/// Render `r` shifted by `(d_row, d_col)` back to formula text. A corner
/// that shifts out of the Sheets grid becomes literal `#REF!`; the other
/// corner of a range (if in bounds) keeps its shifted address.
// TODO(#709): remove once translate_text (Task 4) calls this.
#[allow(dead_code)]
pub(crate) fn shift_ref_text(r: &Ref, d_row: i64, d_col: i64) -> String {
    match r {
        Ref::Cell { sheet, addr } => {
            let mut out = String::new();
            write_sheet(&mut out, sheet).expect("String::write_str is infallible");
            out.push_str(&addr_text(*addr, d_row, d_col));
            out
        }
        Ref::Range { sheet, start, end } => {
            let mut out = String::new();
            write_sheet(&mut out, sheet).expect("String::write_str is infallible");
            out.push_str(&addr_text(*start, d_row, d_col));
            out.push(':');
            out.push_str(&addr_text(*end, d_row, d_col));
            out
        }
        Ref::Name(name) => name.clone(), // never reached by the Task 3 traversal
    }
}

#[cfg(test)]
mod tests;
