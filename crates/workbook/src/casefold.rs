//! Unicode **simple** case folding (schema spec §2), the single source of
//! truth for every case-insensitive comparison in this crate: sheet-name
//! uniqueness, named-range uniqueness, and case-insensitive sheet lookup
//! (plan item 3.1 sheet management).
//!
//! Pinned to Unicode *simple* `Case_Folding` (not `to_lowercase()` / full
//! folding, which disagree on some case pairs and would break cross-surface
//! determinism for names differing only in such characters).

use std::sync::atomic::{AtomicU64, Ordering};

use icu_casemap::CaseMapperBorrowed;

/// How many times [`simple_fold`] has been called, process-wide.
static FOLD_CALLS: AtomicU64 = AtomicU64::new(0);

/// How many bytes of input [`simple_fold`] has folded, process-wide.
static FOLD_BYTES: AtomicU64 = AtomicU64::new(0);

/// Applies simple case folding to `s`, character by character.
pub fn simple_fold(folder: &CaseMapperBorrowed<'static>, s: &str) -> String {
    FOLD_CALLS.fetch_add(1, Ordering::Relaxed);
    FOLD_BYTES.fetch_add(s.len() as u64, Ordering::Relaxed);
    s.chars().map(|c| folder.simple_fold(c)).collect()
}

/// How many folds this process has performed, and over how many input bytes.
///
/// Instrumentation, not a feature (same rationale as
/// [`AuthoredCellIndex::range_has_unauthored_cell_examined`](crate::AuthoredCellIndex::range_has_unauthored_cell_examined)):
/// a case fold allocates and walks its input, and "how many folds does one
/// recalc perform, and does that number depend on the cell count or the length
/// of the sheet names?" is the exact, machine-independent metric behind the
/// sheet-lookup work of issue #952. Wall clock is too machine-dependent to pin
/// in a test; the counters come out of the one function every fold in this
/// crate goes through, so they cannot drift from what actually ran.
///
/// The counters are process-wide and monotonic — read them before and after
/// the operation under test and difference. Two tests in the same binary that
/// both read them cannot run concurrently; keep such assertions in a single
/// `#[test]`.
///
/// The byte count is a proxy for "does sheet-name length still multiply into
/// the per-cell cost": a fold's cost is linear in its input, so a per-cell
/// marginal byte count of zero is what makes name length stop mattering.
#[doc(hidden)]
pub fn folds_performed() -> (u64, u64) {
    (
        FOLD_CALLS.load(Ordering::Relaxed),
        FOLD_BYTES.load(Ordering::Relaxed),
    )
}
