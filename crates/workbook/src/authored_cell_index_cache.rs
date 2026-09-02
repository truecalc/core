//! The authored-cell-index cache (issue #991 fallback design).
//!
//! `seed_spill_sensitive` calls [`AuthoredCellIndex::build`] the first time it
//! examines a range precedent (issue #927), but that build used to be a fresh
//! `O(authored cells)` sweep **every incremental recalc call** that touched
//! any range precedent at all — the same shape of bug `anchor_rectangles()`
//! had before #984, just one level down: the *decision to build* was already
//! lazy, but the *build itself* was never memoized across calls. This module
//! holds that built index on the workbook and hands it back until a mutation
//! could have changed it, mirroring [`crate::spill_anchor_cache`]'s own
//! `SpillAnchorCache` pattern (which itself mirrors [`crate::graph_cache`]).
//!
//! # What the index is a function of
//!
//! [`AuthoredCellIndex::build`] depends on exactly: every sheet's folded name,
//! and which addresses are authored (formula or literal — an entry exists in
//! `Worksheet::iter()`) on each sheet. It is **not** a function of any cell's
//! *value*, so recalc's own value write-back does not invalidate it — see the
//! next section.
//!
//! # The invalidation contract
//!
//! Unlike the spill-anchor cache, this one *can* stay warm across an ordinary
//! recalc: `Workbook::apply_changes` (`crate::recalc`) only ever rewrites a
//! cell that **already has formula text** (it reads the formula off the
//! existing authored cell before writing back through
//! `sheets_mut_untracked()`), so it can change a cell's *value* but never adds
//! or removes an authored-cell *entry*. A spilled cell is reconstructed on
//! read and never separately authored (schema spec §5), so placing, resizing
//! or removing a spill does not touch the authored-cell set either. The
//! narrow, correct condition — checked at every site that can actually add or
//! remove an authored cell — is:
//!
//! - **`Workbook::set`**: invalidate only when the write introduces a
//!   genuinely *new* cell (`introduces_new_cell`, already computed there for
//!   the cell-count cap) — an overwrite of an already-authored cell changes no
//!   entry.
//! - **`Workbook::clear`**: invalidate only when a cell was actually removed
//!   (`prev.is_some()`).
//! - Every sheet-structure mutation that hands out unobserved write access, or
//!   changes the folded-name key space the index is keyed by
//!   (`sheets_mut`, `sheet_mut`, `insert_sheet`, `remove_sheet`,
//!   `rename_sheet`) — the same worst-case reasoning the graph cache and the
//!   spill-anchor cache already apply to those methods. `move_sheet` is
//!   deliberately **not** included: reordering tabs changes neither the
//!   folded-name keys nor any sheet's authored cells (the graph cache and the
//!   spill-anchor cache already exempt it for the same reason). Declaring or
//!   redefining a table is also **not** included: a table is metadata over a
//!   range, and never itself adds or removes an authored cell.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::authored_index::AuthoredCellIndex;

/// The workbook's authored-cell-index cache slot.
///
/// A field of [`Workbook`](crate::Workbook), so it must not disturb the
/// workbook's value-object contract: it is skipped by serde, compares equal to
/// every other cache, and hashes to nothing. Two workbooks with the same
/// content are still equal and still hash the same whether or not either has
/// recalculated.
#[derive(Clone, Default)]
pub(crate) struct AuthoredCellIndexCache {
    entry: Option<Arc<AuthoredCellIndex>>,
    /// How many authored-cell indexes this workbook has built.
    /// Instrumentation for the cache's own tests: "builds per recalc" is the
    /// exact-count metric behind the cache, and wall clock is too
    /// machine-dependent to assert on. Kept per workbook rather than in a
    /// global counter so tests running in parallel cannot perturb each
    /// other's reading.
    builds: u64,
}

impl AuthoredCellIndexCache {
    /// The cached entry, if warm.
    pub(crate) fn get(&self) -> Option<Arc<AuthoredCellIndex>> {
        self.entry.clone()
    }

    /// Stores a freshly built entry and counts the build.
    pub(crate) fn store(&mut self, entry: Arc<AuthoredCellIndex>) {
        self.entry = Some(entry);
        self.builds += 1;
    }

    /// Drops the entry. Idempotent, and always sound: the worst a spurious
    /// invalidation costs is a rebuild.
    pub(crate) fn invalidate(&mut self) {
        self.entry = None;
    }

    /// How many authored-cell indexes this workbook has built.
    pub(crate) fn builds(&self) -> u64 {
        self.builds
    }

    /// Whether an entry is currently held.
    pub(crate) fn is_warm(&self) -> bool {
        self.entry.is_some()
    }
}

/// Hand-written rather than derived: [`AuthoredCellIndex`] does not implement
/// [`std::fmt::Debug`] (it holds no public fields worth printing), and a
/// derived impl would require it to. The cache's own build count and warmth
/// are what a debug print of a [`Workbook`](crate::Workbook) actually needs.
impl std::fmt::Debug for AuthoredCellIndexCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthoredCellIndexCache")
            .field("warm", &self.entry.is_some())
            .field("builds", &self.builds)
            .finish()
    }
}

/// Every cache compares equal to every other: the cache is derived state, so
/// two workbooks that differ only in whether they have recalculated are the
/// same workbook (schema spec §8 — the document is the value).
impl PartialEq for AuthoredCellIndexCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Hashes to nothing, for the same reason [`PartialEq`] ignores it: `a == b`
/// must imply `hash(a) == hash(b)`.
impl Hash for AuthoredCellIndexCache {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
