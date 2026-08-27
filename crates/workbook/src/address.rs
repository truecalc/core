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
        self.a1_key().as_str().to_owned()
    }

    /// Renders the plain-uppercase A1 key into a stack buffer, allocating
    /// nothing. The borrowed form of [`to_a1`](Self::to_a1): identical bytes,
    /// no heap. Read-side grid operations key the cell map through this.
    pub(crate) fn a1_key(&self) -> A1Key {
        let mut key = A1Key {
            buf: [0; A1_KEY_CAPACITY],
            len: 0,
        };

        // Column: bijective base-26 digits come out least-significant first,
        // so stage them and copy back in reverse.
        let mut letters = [0u8; MAX_COLUMN_LETTERS];
        let mut n = 0;
        let mut column = self.column;
        while column > 0 {
            letters[n] = b'A' + ((column - 1) % 26) as u8;
            n += 1;
            column = (column - 1) / 26;
        }
        while n > 0 {
            n -= 1;
            key.push(letters[n]);
        }

        // Row: same story for the decimal digits. A row is always >= 1, so this
        // never emits the empty string.
        let mut digits = [0u8; MAX_ROW_DIGITS];
        let mut d = 0;
        let mut row = self.row;
        while row > 0 {
            digits[d] = b'0' + (row % 10) as u8;
            d += 1;
            row /= 10;
        }
        while d > 0 {
            d -= 1;
            key.push(digits[d]);
        }

        key
    }
}

/// The widest in-bounds column (`ZZZ`) is three letters.
const MAX_COLUMN_LETTERS: usize = 3;
/// The widest in-bounds row (`10000000`) is eight digits.
const MAX_ROW_DIGITS: usize = 8;
/// Every in-bounds A1 key fits in `ZZZ10000000`.
const A1_KEY_CAPACITY: usize = MAX_COLUMN_LETTERS + MAX_ROW_DIGITS;

/// A plain-uppercase A1 key rendered into a fixed stack buffer.
///
/// A [`Worksheet`](crate::Worksheet) keys its grid by `String`, but
/// `BTreeMap<String, _>` probes through `Borrow<str>` — a lookup only needs a
/// `&str`, never an owned key. Rendering here instead of into a `String` is
/// what keeps a range scan from paying a heap allocation per cell it visits.
pub(crate) struct A1Key {
    buf: [u8; A1_KEY_CAPACITY],
    len: usize,
}

impl A1Key {
    fn push(&mut self, byte: u8) {
        self.buf[self.len] = byte;
        self.len += 1;
    }

    /// The rendered key. Byte-for-byte what [`Address::to_a1`] returns.
    pub(crate) fn as_str(&self) -> &str {
        // Every byte written is an ASCII uppercase letter or digit.
        std::str::from_utf8(&self.buf[..self.len]).expect("A1 keys are ASCII")
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
