//! Per-recalc pre-image-count instrumentation (issue #991, Design A).
//!
//! `Workbook::recalc_incremental_measured` used to snapshot every formula
//! cell's value up front (the deleted `snapshot_formula_values`) before an
//! incremental recalc could touch any of them. It now accumulates the
//! pre-image of only the cells it actually writes, lazily, from
//! `recompute`'s own returned change list. Wall clock cannot prove the win an
//! O(1) edit gets from this — it is a difference in *how many cells were
//! recorded*, not in how long anything took. This module holds an exact count
//! of the last incremental call's pre-image-map size so a test can assert it
//! directly, the same rationale [`crate::graph_cache`] and
//! [`crate::spill_anchor_cache`] give for their own build counters.
//!
//! Deliberately holds only the **last** call's count, not a running total:
//! "how many pre-images did the call I just made record" is the question a
//! caller asks after a single edit, and a cumulative counter would force
//! every test to account for every earlier recalc in the same workbook's
//! history instead of just the one under test.

/// The workbook's pre-image-count instrumentation slot.
///
/// A field of [`Workbook`](crate::Workbook), so it must not disturb the
/// workbook's value-object contract: skipped by serde, ignored by
/// `PartialEq`, contributes nothing to `Hash` (see the `graph_cache` and
/// `spill_anchor_cache` module docs for why derived/instrumentation state
/// must not affect either).
#[derive(Debug, Clone, Default)]
pub(crate) struct PreImageStats {
    count: u64,
}

impl PreImageStats {
    /// Records `count` as the last incremental call's pre-image-map size.
    pub(crate) fn record(&mut self, count: usize) {
        self.count = count as u64;
    }

    /// How many cells the last incremental recalc recorded a pre-image for.
    pub(crate) fn count(&self) -> u64 {
        self.count
    }
}

/// Every instance compares equal to every other: this is derived
/// instrumentation, not document content (schema spec §8 — the document is
/// the value).
impl PartialEq for PreImageStats {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Hashes to nothing, for the same reason [`PartialEq`] ignores it: `a == b`
/// must imply `hash(a) == hash(b)`.
impl std::hash::Hash for PreImageStats {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}
