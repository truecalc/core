//! Plain uppercase A1 cell-address parsing and bounds checking (schema spec §3).
//!
//! A serialized cell key MUST match `^[A-Z]{1,3}[1-9][0-9]{0,7}$` **and** lie
//! within the address bounds of the limits ADR (rows `1..=10_000_000`,
//! columns `1..=18_278`, i.e. `A..=ZZZ`). `from_json` rejects every other key
//! — no `$`, no sheet qualifier, no lowercase, no leading zero.

use crate::limits::{MAX_COLUMN, MAX_ROW};

/// A parsed A1 address: 1-based `(row, column)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    /// 1-based row.
    pub row: u32,
    /// 1-based column (`A` = 1).
    pub column: u32,
}

/// Parses a plain uppercase A1 address, enforcing the normative key syntax
/// (`^[A-Z]{1,3}[1-9][0-9]{0,7}$`) and the address bounds (schema spec §3).
///
/// Returns `None` on any malformed key or out-of-bounds row/column. Hand-rolled
/// rather than regex-backed to keep the crate dependency-light and to fold the
/// bounds check into the same pass.
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

    if row == 0 || row > MAX_ROW || column == 0 || column > MAX_COLUMN {
        return None;
    }
    Some(Address { row, column })
}
