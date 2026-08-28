//! The cross-recalculation dependency-graph cache.
//!
//! [`DependencyGraph::build`](crate::DependencyGraph::build) is a pure function
//! of the workbook's *structure*, and it used to run from scratch on every
//! [`recalc`](crate::Workbook::recalc) and every
//! [`recalc_incremental`](crate::Workbook::recalc_incremental) — together with
//! the [`evaluation_order`](crate::DependencyGraph::evaluation_order) derived
//! from it, the single largest fixed cost of a recalculation on a large
//! workbook, paid whether one cell changed or none did.
//!
//! This module holds that result on the workbook and hands it back until a
//! mutation could have changed it.
//!
//! # What the graph is a function of
//!
//! Reading [`DependencyGraph::build`](crate::DependencyGraph::build) and every
//! resolver it calls, the graph depends on exactly:
//!
//! 1. the sheet **name set** (a reference to a missing sheet resolves to
//!    `Unresolved`, and every node is keyed by folded sheet name);
//! 2. every formula cell's `(sheet, address, formula text)`;
//! 3. the workbook's **named ranges** — their names and their `ref`s;
//! 4. the workbook's **table declarations** — their names and their `ref`s;
//! 5. **the text values stored in a declared table's header row**, because a
//!    structured reference resolves its column by matching the header cell's
//!    stored `Value::Text`.
//!
//! Point 5 is the one that is easy to get wrong, and it is why "writing a
//! literal cannot change the graph" is **false** in general: writing `"qty"`
//! into a table's header cell moves what `T[qty]` reads. It is true only when
//! the workbook declares no tables at all, since with no table declaration no
//! cell value can reach the graph.
//!
//! It is *not* a function of any other stored value, of spill footprints, or of
//! tab order: spill occupancy is judged against the grid at recalc time, and
//! the graph keys sheets by name, never by tab index.
//!
//! # The invalidation contract
//!
//! The cache is invalidated by the mutation, not by a later check, so the
//! invariant is: **if the entry is `Some`, it equals a build against the
//! workbook as it is now.** Every `&mut self` method of
//! [`Workbook`](crate::Workbook) either invalidates or is documented here as
//! provably structure-preserving. The two exceptions are:
//!
//! * `Workbook::set` / `Workbook::clear` of a **literal over a non-formula
//!   cell in a workbook that declares no tables** — by points 2 and 5 above,
//!   that adds and removes no node, no edge, and no header text the graph can
//!   see;
//! * recalc's own value write-back, which rewrites an existing formula cell's
//!   stored value while preserving its formula text — same reasoning, and
//!   likewise only while no table is declared.
//!
//! Everything else — any formula write, any clear of a formula, any name or
//! table definition, any sheet operation, and **every** `&mut` accessor that
//! hands out interior state ([`Workbook::sheets_mut`](crate::Workbook::sheets_mut),
//! [`Workbook::sheet_mut`](crate::Workbook::sheet_mut),
//! [`Workbook::names_mut`](crate::Workbook::names_mut),
//! [`Workbook::tables_mut`](crate::Workbook::tables_mut)) — invalidates. The
//! `&mut` accessors invalidate on the *borrow*: what a caller does with the
//! borrow is unobservable from here, so the only sound assumption is the worst
//! one.

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::depgraph::{CellRef, DependencyGraph};

/// A built dependency graph together with the evaluation order derived from
/// it. Immutable once constructed: invalidation replaces the whole entry, it
/// never edits one in place, which is what makes sharing it across a
/// [`Workbook`](crate::Workbook) clone sound.
///
/// `pub`, not `pub(crate)`: [`Workbook::cached_graph_entry`](crate::Workbook::cached_graph_entry)
/// hands this out to any crate that only holds `&Workbook` and wants to reuse
/// a warm graph without rebuilding one (the wasm `precedentsOf`/`dependentsOf`
/// binding). Its fields stay `pub(crate)` — external callers reach the graph
/// through [`graph`](Self::graph), not by construction or field access.
#[derive(Debug)]
pub struct CachedGraph {
    pub(crate) graph: DependencyGraph,
    /// [`DependencyGraph::evaluation_order`](crate::DependencyGraph::evaluation_order)'s
    /// order, for this exact graph.
    pub(crate) order: Vec<CellRef>,
    /// The cycle set from that same pass.
    pub(crate) cycle: BTreeSet<CellRef>,
}

impl CachedGraph {
    /// The dependency graph this entry caches.
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }
}

/// The workbook's dependency-graph cache slot.
///
/// A field of [`Workbook`](crate::Workbook), so it must not disturb the
/// workbook's value-object contract: it is skipped by serde, compares equal to
/// every other cache, and hashes to nothing. Two workbooks with the same
/// content are still equal and still hash the same whether or not either has
/// recalculated.
#[derive(Debug, Clone, Default)]
pub(crate) struct GraphCache {
    entry: Option<Arc<CachedGraph>>,
    /// How many graphs this workbook has built. Instrumentation for the
    /// cache's own tests: "builds per recalc" is the exact-count metric behind
    /// the cache, and wall clock is too machine-dependent to assert on. Kept
    /// per workbook rather than in a global counter so tests running in
    /// parallel cannot perturb each other's reading.
    builds: u64,
}

impl GraphCache {
    /// The cached entry, if warm.
    pub(crate) fn get(&self) -> Option<Arc<CachedGraph>> {
        self.entry.clone()
    }

    /// Stores a freshly built entry and counts the build.
    pub(crate) fn store(&mut self, entry: Arc<CachedGraph>) {
        self.entry = Some(entry);
        self.builds += 1;
    }

    /// Drops the entry. Idempotent, and always sound: the worst a spurious
    /// invalidation costs is a rebuild.
    pub(crate) fn invalidate(&mut self) {
        self.entry = None;
    }

    /// How many graphs this workbook has built.
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
impl PartialEq for GraphCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Hashes to nothing, for the same reason [`PartialEq`] ignores it: `a == b`
/// must imply `hash(a) == hash(b)`.
impl Hash for GraphCache {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
