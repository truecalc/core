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
//!
//! # Two of these limits are enforced on `wasm32` only
//!
//! [`MAX_CELLS_PER_WORKBOOK`] and [`MAX_SERIALIZED_BYTES`] exist because of a
//! property of the 32-bit WebAssembly target, not because a workbook that
//! large is meaningless. Both constants keep their values on every target — a
//! 64-bit tool can still ask "would a browser load this?" — but the checks
//! against them, [`exceeds_cell_cap`] and [`exceeds_serialized_cap`], only
//! reject when `target_arch = "wasm32"`.
//!
//! The `wasm32` constraint is the **address space**. A wasm32 linear memory is
//! indexed by a 32-bit pointer and so cannot exceed 4,294,967,296 bytes; a
//! growth request past that byte fails identically in Node and in desktop
//! browsers. Measured against this crate a stored cell costs about 108 bytes
//! as a number and about 149 as a formula, which puts the real ceiling at
//! roughly 29–40 million cells — the same order as the cap.
//!
//! What makes that wall worth a cap, rather than something to let fail on its
//! own, is *how* it fails. It is not a refusal a caller can catch and report:
//! the allocation failure aborts the entire wasm module. Because one wasm
//! instance normally backs every workbook a host has open, a single oversized
//! workbook destroys the unrelated ones alongside it. These caps turn that into
//! an ordinary [`WorkbookError`](crate::WorkbookError) raised before any of the
//! memory is asked for, and are currently the only thing that does.
//!
//! No other target has that wall. A 64-bit build is bounded by machine memory,
//! which is both far larger and operator-controlled, so a wasm-shaped cap there
//! only refuses documents the engine handles comfortably. (Exhausting machine
//! memory is still an abort rather than an error — but it is a host-level
//! failure at a height the operator chooses, not a fixed ceiling an ordinary
//! spreadsheet reaches.)
//!
//! This stays consistent with "library constants, not schema constants": off
//! `wasm32` these limits are effectively *raised*, which is non-breaking, and
//! the `wasm32` behaviour is unchanged, so nothing is lowered anywhere. A
//! document a 64-bit build accepts may of course exceed what a `wasm32` build
//! will load, which is the same version-dependent validation described above.

/// Maximum populated cells across the whole workbook. Spilled/materialized
/// cells count toward this cap (ADR Decision 5).
///
/// **Enforced on `wasm32` only** — see the module docs, and
/// [`exceeds_cell_cap`], which is the predicate every enforcement site uses.
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
///
/// **Enforced on `wasm32` only** — see the module docs, and
/// [`exceeds_serialized_cap`], which is the predicate every enforcement site
/// uses. This cap is gated together with [`MAX_CELLS_PER_WORKBOOK`] because
/// the two were deliberately matched: canonical JSON costs roughly 53 bytes
/// per numeric cell and ~81 per formula cell, so one million formula cells is
/// about 81 MiB — 81% of this. Relaxing the cell cap alone would only move the
/// same address-space failure from `set` to `to_json`.
pub const MAX_SERIALIZED_BYTES: usize = 100 * 1024 * 1024;

/// Whether a workbook holding `cells` populated cells breaches
/// [`MAX_CELLS_PER_WORKBOOK`] on a target that enforces it.
///
/// Always `false` off `wasm32`: the cap tracks the 32-bit address-space wall
/// described in the module docs, and no other target has one.
#[inline]
#[must_use]
pub const fn exceeds_cell_cap(cells: usize) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        cells > MAX_CELLS_PER_WORKBOOK
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = cells;
        false
    }
}

/// Whether a serialized document of `bytes` bytes breaches
/// [`MAX_SERIALIZED_BYTES`] on a target that enforces it.
///
/// Always `false` off `wasm32`, for the reason given on
/// [`MAX_SERIALIZED_BYTES`].
#[inline]
#[must_use]
pub const fn exceeds_serialized_cap(bytes: usize) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        bytes > MAX_SERIALIZED_BYTES
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bytes;
        false
    }
}
