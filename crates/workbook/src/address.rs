//! A1 cell addressing: parsing, bounds checking, and the A1↔`(row, column)`
//! conversion utilities of the runtime grid (plan item 3.1).
//!
//! A serialized cell key MUST match `^[A-Z]{1,3}[1-9][0-9]{0,7}$` **and** lie
//! within the address bounds of the limits ADR (rows `1..=10_000_000`,
//! columns `1..=18_278`, i.e. `A..=ZZZ`). [`Workbook::from_json`] rejects every
//! other key — no `$`, no sheet qualifier, no lowercase, no leading zero.
//!
//! An [`Address`] is the parsed, bounds-validated form used as the in-memory
//! grid key (plan item 3.1). It can only be constructed in bounds, so every
//! address held by a [`Worksheet`] is guaranteed valid; [`Address::to_a1`]
//! re-emits the exact plain-uppercase key the canonical serializer writes.
//!
//! [`Workbook::from_json`]: crate::Workbook::from_json
//! [`Worksheet`]: crate::Worksheet

use crate::limits::{MAX_COLUMN, MAX_ROW};

/// A parsed, in-bounds A1 address: 1-based `(row, column)`.
///
/// The grid key of a [`Worksheet`](crate::Worksheet). Every constructor is
/// bounds-checked (rows `1..=10_000_000`, columns `1..=18_278`), so an
/// `Address` value is always serializable to a valid A1 key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    /// 1-based row.
    pub row: u32,
    /// 1-based column (`A` = 1).
    pub column: u32,
}

impl Address {
    /// Builds an address from 1-based `(row, column)`, enforcing the address
    /// bounds (rows `1..=10_000_000`, columns `1..=18_278`). Returns `None`
    /// when either coordinate is `0` or out of bounds.
    pub fn new(row: u32, column: u32) -> Option<Self> {
        if row == 0 || row > MAX_ROW || column == 0 || column > MAX_COLUMN {
            return None;
        }
        Some(Self { row, column })
    }

    /// Parses a plain uppercase A1 address (e.g. `A1`, `BC42`), enforcing the
    /// normative key syntax (`^[A-Z]{1,3}[1-9][0-9]{0,7}$`) and the address
    /// bounds (schema spec §3). Returns `None` on any malformed or
    /// out-of-bounds key.
    pub fn from_a1(key: &str) -> Option<Self> {
        parse_a1(key)
    }

    /// Re-emits the plain-uppercase A1 key (`A1`, `BC42`) — the inverse of
    /// [`Address::from_a1`] and the exact key the canonical serializer writes.
    pub fn to_a1(&self) -> String {
        let mut s = column_to_letters(self.column);
        s.push_str(&self.row.to_string());
        s
    }
}

/// Parses a plain uppercase A1 address, enforcing the normative key syntax
/// (`^[A-Z]{1,3}[1-9][0-9]{0,7}$`) and the address bounds (schema spec §3).
///
/// Returns `None` on any malformed key or out-of-bounds row/column. Hand-rolled
/// rather than regex-backed to keep the crate dependency-light and to fold the
/// bounds check into the same pass. Kept as a free function because the
/// document validator and named-ref parser call it on untrusted keys.
pub fn parse_a1(key: &str) -> Option<Address> {
    let bytes = key.as_bytes();
    let mut i = 0;

    // 1..=3 uppercase ASCII letters.
    let mut column: u32 = 0;
    while i < bytes.len() && bytes[i].is_ascii_uppercase() {
        if i >= 3 {
            return None; // more than 3 letters
        }
        column = column * 26 + (bytes[i] - b'A' + 1) as u32;
        i += 1;
    }
    if i == 0 {
        return None; // no leading letters
    }

    // First digit 1..=9 (no leading zero), then up to 7 more digits.
    let digits = &bytes[i..];
    if digits.is_empty() || digits.len() > 8 {
        return None;
    }
    if digits[0] == b'0' {
        return None; // leading zero forbidden by `[1-9]`
    }
    let mut row: u32 = 0;
    for &b in digits {
        if !b.is_ascii_digit() {
            return None;
        }
        row = row.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }

    Address::new(row, column)
}

/// Converts a 1-based column index to its bijective base-26 letters
/// (`1 → "A"`, `26 → "Z"`, `27 → "AA"`, `18_278 → "ZZZ"`). The input is
/// assumed in bounds (it always is when called on an [`Address`] column).
pub(crate) fn column_to_letters(mut column: u32) -> String {
    let mut buf = [0u8; 3]; // ZZZ is the widest in-bounds column.
    let mut len = 0;
    while column > 0 {
        let rem = ((column - 1) % 26) as u8;
        buf[len] = b'A' + rem;
        len += 1;
        column = (column - 1) / 26;
    }
    buf[..len].reverse();
    // Safe: every byte written is an ASCII uppercase letter.
    String::from_utf8(buf[..len].to_vec()).expect("ASCII letters are valid UTF-8")
}
