//! The spill-anchor-rectangle cache (issue #984).
//!
//! `Workbook::anchor_rectangles` (private, `crate::recalc`) is a full scan of
//! every authored cell on every sheet, looking for an array-valued (spilled)
//! cell — and it used to run unconditionally, once per incremental recalc
//! call and twice more per spill-widen pass, regardless of whether the
//! workbook had any spills, any formulas, or any range precedents at all.
//! This module holds that result on the workbook and hands it back until a
//! mutation could have changed it, mirroring [`crate::graph_cache`]'s
//! `CachedGraph`/`GraphCache` pattern.
//!
//! # What the map is a function of
//!
//! Reading `anchor_rectangles`, the map depends on exactly: every authored
//! cell's `(sheet, address, value)` where the value is `Value::Array`. It is
//! **not** a function of formulas, names, or tables as such — only of which
//! cells currently hold an array value.
//!
//! # The invalidation contract — a genuinely separate schedule from the graph
//!
//! This cache **cannot** ride [`crate::graph_cache::GraphCache`]'s
//! invalidation schedule. Recalc's own value write-back
//! (`Workbook::apply_changes`, through `sheets_mut_untracked`) is exactly the
//! write that places, resizes, or removes a spill — and the graph-cache module
//! docs deliberately document that write-back as one of the two cases that
//! must **not** invalidate the graph cache, since it preserves formula text
//! and adds/removes no node or edge. So an ordinary recalc that changes a
//! spill's footprint changes this cache's answer while leaving the graph cache
//! warm on purpose: sharing the graph's schedule would silently serve stale
//! rectangles across the spill-widen loop's own before/after comparison — a
//! correctness bug, not just a missed optimization.
//!
//! A literal write can also create or destroy an array-valued cell directly
//! (`Workbook::set` accepts `CellInput::Literal(Value::Array(..))`, and
//! `check_value_limits` has a dedicated `Value::Array` branch enforcing the
//! element-count cap — array literals are a supported input shape, not just a
//! formula-evaluation output), so this cache cannot be keyed off "did a
//! formula change" either.
//!
//! The narrow, correct condition — checked at every grid-cell write site
//! (`Workbook::set`, `Workbook::clear`, and `apply_changes`'s per-cell
//! write-back) — is: **the old value was `Value::Array`, or the new value is
//! `Value::Array`.** That covers a new spill, a cleared spill, and a resized
//! spill (a resize already satisfies "old was Array", so no separate size
//! comparison is needed). Every other worksheet-mutating entry point on
//! [`Workbook`](crate::Workbook) that hands out unobserved write access
//! (`sheets_mut`, `sheet_mut`, `insert_sheet`, `remove_sheet`, `rename_sheet`)
//! invalidates unconditionally on the same worst-case reasoning the graph
//! cache already applies to those methods — the borrow, or the incoming
//! sheet's cell content, is unobservable from here.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::depgraph::CellRef;
use crate::spill::SpillRect;

/// The workbook's spill-anchor-rectangle cache slot.
///
/// A field of [`Workbook`](crate::Workbook), so it must not disturb the
/// workbook's value-object contract: it is skipped by serde, compares equal to
/// every other cache, and hashes to nothing. Two workbooks with the same
/// content are still equal and still hash the same whether or not either has
/// recalculated.
#[derive(Debug, Clone, Default)]
pub(crate) struct SpillAnchorCache {
    entry: Option<Arc<BTreeMap<CellRef, SpillRect>>>,
    /// How many anchor-rectangle maps this workbook has built. Instrumentation
    /// for the cache's own tests: "builds per recalc" is the exact-count
    /// metric behind the cache, and wall clock is too machine-dependent to
    /// assert on. Kept per workbook rather than in a global counter so tests
    /// running in parallel cannot perturb each other's reading.
    builds: u64,
}

impl SpillAnchorCache {
    /// The cached entry, if warm.
    pub(crate) fn get(&self) -> Option<Arc<BTreeMap<CellRef, SpillRect>>> {
        self.entry.clone()
    }

    /// Stores a freshly built entry and counts the build.
    pub(crate) fn store(&mut self, entry: Arc<BTreeMap<CellRef, SpillRect>>) {
        self.entry = Some(entry);
        self.builds += 1;
    }

    /// Drops the entry. Idempotent, and always sound: the worst a spurious
    /// invalidation costs is a rebuild.
    pub(crate) fn invalidate(&mut self) {
        self.entry = None;
    }

    /// How many anchor-rectangle maps this workbook has built.
    pub(crate) fn builds(&self) -> u64 {
        self.builds
    }

    /// Whether an entry is currently held.
    pub(crate) fn is_warm(&self) -> bool {
        self.entry.is_some()
    }
}

/// Every cache compares equal to every other: the cache is derived state, so
/// two workbooks that differ only in whether they have recalculated are the
/// same workbook (schema spec §8 — the document is the value).
impl PartialEq for SpillAnchorCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Hashes to nothing, for the same reason [`PartialEq`] ignores it: `a == b`
/// must imply `hash(a) == hash(b)`.
impl Hash for SpillAnchorCache {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
