//! Resource limits — the single source of truth for the structural caps of
//! the scope ADR (`2026-06-07-workbook-v1-scope.md`, Decision 5).
//!
//! These are **library constants, not schema constants** (schema spec §1):
//! a limit can rise without a schema version bump, and a document is always
//! validated against the limits of the library version that loads it. Raising
//! a limit is non-breaking; lowering one is not.
//!
//! Structural limits are enforced at [`Workbook::from_json`](crate::Workbook::from_json);
//! the serialized byte cap is enforced at serialize/deserialize time only
//! (computing canonical byte length per mutation would be O(document) per
//! edit — ADR Decision 5).

/// Maximum populated cells across the whole workbook. Spilled/materialized
/// cells count toward this cap (ADR Decision 5).
pub const MAX_CELLS_PER_WORKBOOK: usize = 1_000_000;

/// Maximum length of a `text` value, in Unicode scalar values.
pub const MAX_TEXT_LEN: usize = 50_000;

/// Maximum element count (`m × n`) of a single `array` value.
pub const MAX_ARRAY_ELEMENTS: usize = 1_000_000;

/// Maximum worksheets per workbook.
pub const MAX_SHEETS: usize = 256;

/// Maximum length of a sheet name, in Unicode scalar values (schema spec §3).
pub const MAX_SHEET_NAME_LEN: usize = 100;

/// Maximum row index, 1-based (schema spec §3 / ADR Decision 5).
pub const MAX_ROW: u32 = 10_000_000;

/// Maximum column index, 1-based: `ZZZ` = 18,278 (schema spec §3 / ADR Decision 5).
pub const MAX_COLUMN: u32 = 18_278;

/// Maximum verbatim formula length, in bytes.
pub const MAX_FORMULA_LEN: usize = 32 * 1024;

/// Maximum workbook-scoped named ranges.
pub const MAX_NAMED_RANGES: usize = 10_000;

/// Maximum workbook-scoped tables (structured-references spec §4,
/// truecalc/core#861) — the same magnitude as [`MAX_NAMED_RANGES`], since a
/// table costs the same order of workbook-scoped bookkeeping. This also
/// bounds the cost of the O(tables) scans a table incurs on every mutation
/// (`overlapping_table`, `expand_table_on_append` — the latter runs on every
/// `Workbook::set()`).
pub const MAX_TABLES: usize = 10_000;

/// Maximum serialized canonical JSON document size, in bytes (100 MiB).
/// A workbook exceeding this cannot be serialized, and `from_json` rejects
/// oversized inputs (ADR Decision 5).
pub const MAX_SERIALIZED_BYTES: usize = 100 * 1024 * 1024;
