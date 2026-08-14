//! Table-specific validation not already covered by reusing
//! `named_ref.rs`'s name/ref rules (structured-references spec §4): range
//! overlap between two tables, and header-row column-name validity.

use crate::named_ref::{is_valid_name, ParsedRef};

/// A canonicalized range's sheet + inclusive row/column bounds, for overlap
/// checking. Built from an already-canonical `ref` string (validate via
/// `named_ref::parse_canonical_ref` first).
pub struct ParsedRangeBounds {
    pub sheet: String,
    pub row_start: u32,
    pub row_end: u32,
    pub col_start: u32,
    pub col_end: u32,
}

/// Extracts row/column bounds from an already-canonical `ref` string and its
/// pre-parsed sheet (from `named_ref::parse_canonical_ref`). Returns `None`
/// only if `r` is malformed in a way `parse_canonical_ref` should already
/// have rejected — callers validate canonicality first.
pub fn parsed_range_bounds(r: &str, parsed: &ParsedRef) -> Option<ParsedRangeBounds> {
    let a1_part = r.rsplit_once('!')?.1;
    let (start_raw, end_raw) = a1_part.split_once(':')?;
    let start = crate::address::parse_a1(start_raw)?;
    let end = crate::address::parse_a1(end_raw)?;
    Some(ParsedRangeBounds {
        sheet: parsed.sheet.clone(),
        row_start: start.row,
        row_end: end.row,
        col_start: start.column,
        col_end: end.column,
    })
}

/// Do two table ranges overlap? Different sheets never overlap; same sheet,
/// standard rectangle-intersection test.
pub fn ranges_overlap(a: &ParsedRangeBounds, b: &ParsedRangeBounds) -> bool {
    a.sheet == b.sheet
        && a.row_start <= b.row_end
        && b.row_start <= a.row_end
        && a.col_start <= b.col_end
        && b.col_start <= a.col_end
}

/// Validates a table's header-row cell texts as column names: each must be
/// a valid bare identifier (same rule as a table/named-range `name`, spec
/// §4), and all must be unique (case-insensitive, simple case-folding).
/// Returns the column names in order on success.
pub fn header_row_columns<'a>(
    header_row_cells: impl Iterator<Item = &'a str>,
) -> Result<Vec<String>, String> {
    use icu_casemap::CaseMapperBorrowed;
    let folder = CaseMapperBorrowed::new();
    let mut seen = std::collections::HashSet::new();
    let mut columns = Vec::new();
    for cell in header_row_cells {
        if !is_valid_name(cell) {
            return Err(format!(
                "table header cell {cell:?} is not a valid column name (must match \
                 ^[A-Za-z_][A-Za-z0-9_]*$, not an A1 address, not a boolean literal)"
            ));
        }
        let folded = crate::casefold::simple_fold(&folder, cell);
        if !seen.insert(folded) {
            return Err(format!("duplicate table column name {cell:?}"));
        }
        columns.push(cell.to_string());
    }
    Ok(columns)
}

#[cfg(test)]
mod tests;
