//! The cell-count and serialized-byte caps are enforced on `wasm32` only.
//!
//! Both caps track the 32-bit WebAssembly address-space wall (see the
//! `truecalc_workbook::limits` module docs), so off `wasm32` the predicates the
//! enforcement sites call must never reject. The two caps are gated together on
//! purpose: relaxing the cell cap alone would only move the same failure from
//! `Workbook::set` to `Workbook::to_json`.

use truecalc_workbook::limits::{
    exceeds_cell_cap, exceeds_serialized_cap, MAX_CELLS_PER_WORKBOOK, MAX_SERIALIZED_BYTES,
};

/// The constants themselves are unchanged on every target. Downstream hosts
/// mirror these values to predict whether a browser build will accept a
/// document, so gating enforcement must not move the numbers.
#[test]
fn cap_constants_are_unchanged_on_every_target() {
    assert_eq!(MAX_CELLS_PER_WORKBOOK, 1_000_000);
    assert_eq!(MAX_SERIALIZED_BYTES, 100 * 1024 * 1024);
}

/// A workbook exactly at the cap is within it on every target: both predicates
/// are strictly-greater-than tests.
#[test]
fn a_document_exactly_at_either_cap_is_never_rejected() {
    assert!(!exceeds_cell_cap(MAX_CELLS_PER_WORKBOOK));
    assert!(!exceeds_serialized_cap(MAX_SERIALIZED_BYTES));
    assert!(!exceeds_cell_cap(0));
    assert!(!exceeds_serialized_cap(0));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn wasm32_still_rejects_past_either_cap() {
    assert!(exceeds_cell_cap(MAX_CELLS_PER_WORKBOOK + 1));
    assert!(exceeds_serialized_cap(MAX_SERIALIZED_BYTES + 1));
}

/// The 30 MB financial model this change exists for: 1,651,024 cells and about
/// 134 MB of canonical JSON. It breaches *both* caps, which is why both had to
/// be gated — off `wasm32` neither predicate rejects it.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn other_targets_accept_a_document_past_both_caps() {
    const MODEL_CELLS: usize = 1_651_024;
    const MODEL_BYTES: usize = 134_000_000;

    // Compile-time checks, not runtime assertions: both sides are `const`, so
    // `assert!` here would only assert on the compiler (clippy::assertions_on_constants).
    const { assert!(MODEL_CELLS > MAX_CELLS_PER_WORKBOOK) };
    const { assert!(MODEL_BYTES > MAX_SERIALIZED_BYTES) };

    assert!(!exceeds_cell_cap(MODEL_CELLS));
    assert!(!exceeds_serialized_cap(MODEL_BYTES));
    assert!(!exceeds_cell_cap(usize::MAX));
    assert!(!exceeds_serialized_cap(usize::MAX));
}
