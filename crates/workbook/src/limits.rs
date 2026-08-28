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
//! 64-bit tool can still ask "would a browser load this?" **by comparing
//! against the constants directly.** The crate-internal checks that actually
//! reject, `exceeds_cell_cap` and `exceeds_serialized_cap`, only do so when
//! `target_arch = "wasm32"` — off `wasm32` they always return `false`, so they
//! answer a different, narrower question ("does *this build* reject it?") and
//! are not exported for that reason (see their doc comments).
//!
//! The `wasm32` constraint is the **address space**. A wasm32 linear memory is
//! indexed by a 32-bit pointer and so cannot exceed 4,294,967,296 bytes; a
//! growth request past that byte fails identically in Node and in desktop
//! browsers.
//!
//! # Measured cost per formula cell (the dependency-graph cache)
//!
//! Since the dependency-graph cache (`graph_cache` module), a formula cell's
//! **retained** cost on `wasm32` is no longer just the document — it is the
//! document *plus* the cached graph, kept alive between recalculations
//! instead of being rebuilt and dropped every time:
//!
//! | | wasm32 | native |
//! |---|---:|---:|
//! | document, retained | 106.9 B/cell | 183.4 |
//! | cache, retained | 545.2 | 855.7 |
//! | **idle total, retained** | **≈652 B/cell** | ≈1,039 |
//! | peak, mid-recalc | 856.7 | — |
//!
//! (Measured with a counting global allocator compiled for
//! `wasm32-unknown-unknown` and run in Node, at 160,000 formula cells.)
//!
//! The important nuance: **the peak was already ≈857 B/cell before this
//! cache existed**, because `DependencyGraph::build` allocated the same graph
//! transiently on every `recalc`/`recalc_incremental` call and dropped it
//! again at the end. So a *single* workbook's mid-recalc high-water mark is
//! unchanged — still bounded by the peak, not the idle floor — and the real
//! ceiling for one workbook actively recalculating at the cap is
//! 4,294,967,296 / 857 ≈ 5.0 million formula cells, the same before and after
//! this cache. (Before this cache, the previously-documented estimate here
//! was "a stored cell costs about 108 bytes as a number and about 149 as a
//! formula, roughly 29–40 million cells" — that was the *retained*, not the
//! *peak*, cost, and so was always an undercount of the true wall a
//! recalculating workbook hits; this cache didn't create that gap, it just
//! made the retained number closer to the peak one, which is why the section
//! below now matters.)
//!
//! What changed is the **idle** floor: 107 → 652 B/cell. A workbook sitting
//! in memory between edits, not currently recalculating, now costs roughly
//! 6× what it did, because the graph it built for its last recalculation is
//! still attached to it. [`Workbook::drop_derived_state`](crate::Workbook::drop_derived_state)
//! releases that cache (at the cost of a rebuild on the next recalc/query)
//! for a host that wants the old idle floor back for a workbook it isn't
//! actively using.
//!
//! What makes that wall worth a cap, rather than something to let fail on its
//! own, is *how* it fails. It is not a refusal a caller can catch and report:
//! the allocation failure aborts the entire wasm module. Because one wasm
//! instance normally backs every workbook a host has open, a single oversized
//! workbook destroys the unrelated ones alongside it. These caps turn that into
//! an ordinary [`WorkbookError`](crate::WorkbookError) raised before any of the
//! memory is asked for, and are currently the only thing that does.
//!
//! # Multi-workbook consequence of the idle floor
//!
//! The scenario the cap exists for (previous paragraph) is one wasm instance
//! holding several open workbooks, at most a few of them recalculating at
//! once. At the enforced 1,000,000-cell cap, with one workbook actively
//! recalculating (paying the ≈857 B/cell peak) and the rest idle:
//!
//! - **Before this cache**: an idle workbook retained ≈107 B/cell (document
//!   only), so `(4,294,967,296 − 857,000,000) / 107,000,000 ≈ 32` more idle
//!   workbooks fit alongside the recalculating one — **W ≈ 32** open
//!   workbooks (rounding down) before the 4 GiB wall.
//! - **After this cache**: an idle workbook retains ≈652 B/cell (document
//!   *and* cache), so `(4,294,967,296 − 857,000,000) / 652,000,000 ≈ 5` more
//!   idle workbooks fit — **W ≈ 6** open workbooks.
//!
//! Whether that consequence justifies lowering [`MAX_CELLS_PER_WORKBOOK`], or
//! documenting the multi-workbook budget more explicitly to hosts, is a
//! decision for later — this paragraph is the number it should be made from,
//! not a call this module makes on its own.
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
/// **Enforced on `wasm32` only** — see the module docs. This constant itself
/// keeps its value on every target; the crate-internal `exceeds_cell_cap`
/// predicate every enforcement site uses is what varies by build.
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
/// **Enforced on `wasm32` only** — see the module docs. This constant itself
/// keeps its value on every target; the crate-internal
/// `exceeds_serialized_cap` predicate every enforcement site uses is what
/// varies by build. This cap is gated together with
/// [`MAX_CELLS_PER_WORKBOOK`] because the two were deliberately matched:
/// canonical JSON costs roughly 53 bytes per numeric cell and ~81 per formula
/// cell, so one million formula cells is about 81 MiB — 81% of this.
/// Relaxing the cell cap alone would only move the same address-space
/// failure from `set` to `to_json`.
pub const MAX_SERIALIZED_BYTES: usize = 100 * 1024 * 1024;

/// Whether a workbook holding `cells` populated cells breaches
/// [`MAX_CELLS_PER_WORKBOOK`] on a target that enforces it.
///
/// Always `false` off `wasm32`: the cap tracks the 32-bit address-space wall
/// described in the module docs, and no other target has one.
///
/// **Internal enforcement policy, not a public API.** This answers "does
/// *this build* reject it?", not "would a browser load this?" — the two
/// questions differ off `wasm32`, where this always returns `false`. A host
/// asking the browser-load question should compare against
/// [`MAX_CELLS_PER_WORKBOOK`] directly. `#[doc(hidden)]` rather than
/// `pub(crate)` because the gating integration test in `tests/` is a separate
/// crate and needs to reach this (CLAUDE.md §9 forbids inline `#[cfg(test)]`
/// in production sources); the precedent for that shape is
/// `DepGraph::formula_precedent_cells_examined`. Being `const fn` does not
/// mean "same result on every target" here — it means "evaluable at compile
/// time on whichever target is compiling", so a downstream `const` built
/// against this can bake in a native `false` without warning, and rustdoc
/// renders the host (x86_64) body, so docs.rs would otherwise show this
/// always returning `false` right next to a constants page that still says
/// 1,000,000.
#[doc(hidden)]
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
///
/// **Internal enforcement policy, not a public API** — see
/// [`exceeds_cell_cap`]'s doc comment, which this mirrors: it answers "does
/// *this build* reject it?", a host asking "would a browser load this?"
/// should compare against [`MAX_SERIALIZED_BYTES`] directly, and `const fn`
/// here means "evaluable at compile time on whichever target is compiling",
/// not "same result everywhere".
#[doc(hidden)]
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
