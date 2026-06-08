//! Reference model for the workbook grammar (P1.2).
//!
//! A [`Ref`] is the parser-level description of *what a formula points at*:
//! a single cell, a rectangular range, or a named reference. Resolution is
//! delegated to the caller (core = language); the engine itself never knows
//! what a sheet or a named range contains.

use std::fmt;

/// A1-style cell address with 1-based column and row.
///
/// `A1` → `CellAddr { col: 1, row: 1 }`, `BC42` → `CellAddr { col: 55, row: 42 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAddr {
    /// 1-based column index (`A` = 1, `Z` = 26, `AA` = 27).
    pub col: u32,
    /// 1-based row index.
    pub row: u32,
}

impl CellAddr {
    /// Parse an A1-style address (case-insensitive): letters followed by digits.
    ///
    /// Returns `None` for anything else (empty column/row part, row `0`,
    /// trailing characters, or out-of-range values).
    pub fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        let col_end = bytes.iter().take_while(|b| b.is_ascii_alphabetic()).count();
        if col_end == 0 || col_end == bytes.len() {
            return None;
        }
        let row_str = &text[col_end..];
        if !row_str.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let row: u32 = row_str.parse().ok()?;
        if row == 0 {
            return None;
        }
        let mut col: u32 = 0;
        for b in &bytes[..col_end] {
            let v = (b.to_ascii_uppercase() - b'A' + 1) as u32;
            col = col.checked_mul(26)?.checked_add(v)?;
        }
        Some(Self { col, row })
    }
}

impl fmt::Display for CellAddr {
    /// Canonical A1 form: uppercase column letters followed by the row number.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut col = self.col;
        let mut letters = [0u8; 8];
        let mut n = 0;
        while col > 0 {
            let rem = ((col - 1) % 26) as u8;
            letters[n] = b'A' + rem;
            n += 1;
            col = (col - 1) / 26;
        }
        for b in letters[..n].iter().rev() {
            write!(f, "{}", *b as char)?;
        }
        write!(f, "{}", self.row)
    }
}

/// A parsed reference. Resolution (what the reference evaluates to) is
/// delegated to the embedding application; see the workbook v1 scope ADR.
#[derive(Debug, Clone, PartialEq)]
pub enum Ref {
    /// A single cell, optionally sheet-qualified: `A1`, `Sheet1!A1`, `'Q2 Data'!A1`.
    Cell {
        /// Sheet name without quoting/escaping; `None` for a bare `A1` reference.
        sheet: Option<String>,
        addr: CellAddr,
    },
    /// A rectangular range, optionally sheet-qualified: `A1:D4`, `Sheet1!A1:B2`.
    Range {
        /// Sheet name without quoting/escaping; `None` for a bare range.
        sheet: Option<String>,
        start: CellAddr,
        end: CellAddr,
    },
    /// A named reference (named range, defined name): `TAX_RATE`.
    Name(String),
}

impl Ref {
    /// Classify a bare identifier or range string into a [`Ref`].
    ///
    /// `"A1"` → [`Ref::Cell`] (no sheet), `"A1:D4"` → [`Ref::Range`] (no sheet),
    /// anything else → [`Ref::Name`]. This is the mapping used for bare
    /// identifiers in formulas, which stay [`crate::Expr::Variable`] nodes in
    /// the AST for compatibility.
    pub fn classify(text: &str) -> Ref {
        if let Some(addr) = CellAddr::parse(text) {
            return Ref::Cell { sheet: None, addr };
        }
        if let Some((s, e)) = text.split_once(':') {
            if let (Some(start), Some(end)) = (CellAddr::parse(s), CellAddr::parse(e)) {
                return Ref::Range { sheet: None, start, end };
            }
        }
        Ref::Name(text.to_string())
    }
}

/// True if `name` looks like an in-grid A1 cell address (1-3 column
/// letters followed by digits), so it must be quoted before `!` to
/// disambiguate: a sheet named `A1` is written `'A1'!B2`, while `Sheet1`
/// (five column letters - beyond any real grid) stays unquoted.
fn looks_like_cell_addr(name: &str) -> bool {
    let letters = name.bytes().take_while(|b| b.is_ascii_alphabetic()).count();
    (1..=3).contains(&letters) && CellAddr::parse(name).is_some()
}

/// True if `name` cannot appear unquoted before `!` in canonical form.
/// Unquoted sheet names must match `^[A-Za-z_][A-Za-z0-9_]*$`, must not
/// look like an A1 cell address (see [`looks_like_cell_addr`]), and must
/// not be the boolean literals `TRUE`/`FALSE` (which the parser recognizes
/// before identifiers, so an unquoted `TRUE!A1` would not re-parse).
fn sheet_needs_quoting(name: &str) -> bool {
    let mut chars = name.chars();
    let bare = match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    };
    !bare
        || looks_like_cell_addr(name)
        || name.eq_ignore_ascii_case("TRUE")
        || name.eq_ignore_ascii_case("FALSE")
}

fn write_sheet(f: &mut fmt::Formatter<'_>, sheet: &Option<String>) -> fmt::Result {
    match sheet {
        None => Ok(()),
        Some(name) => {
            if sheet_needs_quoting(name) {
                write!(f, "'{}'!", name.replace('\'', "''"))
            } else {
                write!(f, "{}!", name)
            }
        }
    }
}

impl fmt::Display for Ref {
    /// Canonical text form per the workbook JSON schema v1 spec: sheet name
    /// single-quoted iff it is not a bare identifier or would parse as an A1
    /// address; embedded single quotes doubled (`''`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ref::Cell { sheet, addr } => {
                write_sheet(f, sheet)?;
                write!(f, "{}", addr)
            }
            Ref::Range { sheet, start, end } => {
                write_sheet(f, sheet)?;
                write!(f, "{}:{}", start, end)
            }
            Ref::Name(name) => write!(f, "{}", name),
        }
    }
}
