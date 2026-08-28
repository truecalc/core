//! The recalc engine (plan item 3.3, issue #535): the layer that makes a
//! [`Workbook`] actually recompute.
//!
//! A workbook stores formulas verbatim with their last evaluated result
//! ([`Value::Empty`] until first recalc, P3.4). Recalc walks the dependency
//! graph (P3.2), evaluates every formula cell in dependency order through a
//! grid-backed [`Resolver`] (the core P1.3 seam), and writes each new result
//! back into the grid — returning the ordered list of [`Change`]s it made.
//!
//! # Two modes, one result
//!
//! - [`Workbook::recalc`] is a **full** recalc: it evaluates every formula
//!   cell in topological order.
//! - [`Workbook::recalc_incremental`] is an **incremental** recalc: given the
//!   cells an edit touched, it recomputes only their transitive dependents
//!   (plus all volatile cells, which are always dirty — scope ADR Decision 3),
//!   reusing the stored results of everything outside that closure.
//!
//! Both produce the same grid for the same workbook + context: incremental
//! recalc is full recalc restricted to the dirty closure, and the property
//! `recalc_incremental(edits) ≡ recalc()` is asserted by the test suite (the
//! issue's acceptance criterion).
//!
//! # Determinism and `RecalcContext`
//!
//! Recalc takes an explicit [`RecalcContext`] (scope ADR Decision 3): the same
//! workbook + same context produces a byte-identical grid. The context pins the
//! volatile date functions (`NOW`/`TODAY`) to a fixed instant via core's
//! `evaluate_with_resolver_at` `now_serial` hook, with the UTC→local serial
//! conversion done against a **vendored** IANA timezone database (`chrono-tz`),
//! never the host clock or OS tz tables. See [`RecalcContext`] for the RNG
//! caveat.
//!
//! # Cycles
//!
//! A formula cell on a dependency cycle (and any cell the cycle taints) cannot
//! be evaluated in order; recalc assigns it the Sheets circular-dependency
//! error without looping forever. Cycle membership comes from the graph's
//! Tarjan SCC pass ([`DependencyGraph::cycle_cells`]); see [`CIRCULAR_ERROR`].

use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use chrono::{NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use icu_casemap::CaseMapperBorrowed;
use truecalc_core::eval::EvalHook;
use truecalc_core::{Engine, EngineFlavor, ErrorKind, Ref, Resolver, Value as CoreValue};

use crate::address::Address;
use crate::casefold::simple_fold;
use crate::cell::Cell;
use crate::depgraph::{CellRef, DependencyGraph, Precedent, RangeRef};
use crate::grid_spills::GridSpillIndex;
use crate::named_ref;
use crate::spill::{spill_rect, SpillRect, BLOCKED_SPILL_ERROR};
use crate::table_ref;
use crate::value::Value;
use crate::workbook::Workbook;
use crate::worksheet::Worksheet;

/// The error a cell on (or downstream of) a circular dependency takes.
///
/// Google Sheets reports a circular dependency as `#REF!` (surfaced in the UI
/// as "Circular dependency detected"). A dedicated workbook-level cycle
/// fixture is not yet in the repo (the P3.6 set covers cross-sheet, named
/// ranges, and date-type), so this exact code is **not** fixture-pinned here;
/// the in-repo cycle tests assert the engine's behavior (a cycle is detected,
/// every cell on it takes this error, and recalc terminates), and the code is
/// re-verified once a `cycles` fixture lands (issue note).
pub const CIRCULAR_ERROR: &str = "#REF!";

/// The deterministic context a recalc evaluates against (scope ADR Decision 3).
///
/// Same workbook + same `RecalcContext` ⇒ byte-identical recomputed grid. The
/// context is an **input to recalc**, never part of the workbook value or its
/// JSON (value-object ADR): two recalcs with different contexts legitimately
/// differ, and the property tests compare like-context runs only.
///
/// # Volatile pinning
///
/// - **`NOW()` / `TODAY()`** are pinned: [`timestamp_ms`](Self::timestamp_ms)
///   (a UTC instant) is converted to a local spreadsheet serial against the
///   **vendored** [`timezone`](Self::timezone) (`chrono-tz`, not the host tz
///   database), and that serial is passed to core's
///   `evaluate_with_resolver_at`. The conversion is the determinism envelope:
///   same instant + same timezone + same truecalc version ⇒ same serial.
/// - **`RAND()` / `RANDBETWEEN()` / `RANDARRAY()`** carry a
///   [`rng_seed`](Self::rng_seed) and a per-cell key helper ([`Self::rng_key`])
///   implementing the ADR's `prf(seed, sheet_index, row, col, draw_index)`
///   scheme. **Caveat:** core's RNG functions presently read the system clock
///   directly and take no per-cell key (`crates/core/.../math/rand`), so the
///   workbook layer cannot yet inject this seed into them — full PRF-keyed RNG
///   determinism requires a core change and is tracked for P4. `rng_seed` is
///   carried now so the API is stable; recalc therefore guarantees determinism
///   for non-RNG workbooks (which is every P3.6 fixture).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecalcContext {
    /// The evaluation instant, in milliseconds since the Unix epoch (UTC).
    /// `NOW()`/`TODAY()` derive from this.
    timestamp_ms: i64,
    /// The IANA timezone the instant is rendered into a local serial against,
    /// from the vendored `chrono-tz` snapshot.
    timezone: Tz,
    /// Keys the deterministic per-cell RNG draws (ADR `prf(...)`); see the
    /// type-level caveat about core support.
    rng_seed: u64,
}

impl RecalcContext {
    /// Builds a context from a UTC instant (Unix milliseconds), an IANA
    /// timezone id (e.g. `"Etc/GMT"`, `"America/New_York"`), and an RNG seed.
    ///
    /// Returns `None` if `tz` is not a known IANA id in the vendored database.
    pub fn new(timestamp_ms: i64, tz: &str, rng_seed: u64) -> Option<Self> {
        let timezone: Tz = tz.parse().ok()?;
        Some(Self {
            timestamp_ms,
            timezone,
            rng_seed,
        })
    }

    /// The UTC instant this context pins volatile time to (Unix milliseconds).
    pub fn timestamp_ms(&self) -> i64 {
        self.timestamp_ms
    }

    /// The vendored IANA timezone the instant is localized against.
    pub fn timezone(&self) -> Tz {
        self.timezone
    }

    /// The RNG seed keying deterministic per-cell draws.
    pub fn rng_seed(&self) -> u64 {
        self.rng_seed
    }

    /// The local spreadsheet serial datetime this context pins `NOW()`/`TODAY()`
    /// to: the UTC `timestamp_ms` rendered into `timezone`, expressed as days
    /// since the 1899-12-30 epoch (integer part) plus time-of-day (fraction) —
    /// the `now_serial` core's `evaluate_at` family consumes.
    ///
    /// Returns `None` only if the instant is unrepresentable (e.g. out of
    /// `chrono`'s range), which cannot happen for any realistic timestamp.
    pub fn now_serial(&self) -> Option<f64> {
        let utc = Utc.timestamp_millis_opt(self.timestamp_ms).single()?;
        let local = utc.with_timezone(&self.timezone).naive_local();
        let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
        let days = local.date().signed_duration_since(epoch).num_days() as f64;
        let secs = local.time().num_seconds_from_midnight() as f64;
        Some(days + secs / 86_400.0)
    }

    /// The pinned "now" as an absolute UTC instant in nanoseconds, for the
    /// zone-aware `TZNOW`. Derived from the same `timestamp_ms` as
    /// [`now_serial`](Self::now_serial), so `NOW()` and `TZNOW()` share one
    /// deterministic clock.
    pub fn now_utc_nanos(&self) -> Option<i64> {
        self.timestamp_ms.checked_mul(1_000_000)
    }

    /// The ADR's per-draw RNG key `prf(rng_seed, sheet_index, row, col,
    /// draw_index)`, a deterministic, order-independent mixing of the cell
    /// identity into the seed.
    ///
    /// Exposed (and unit-tested) so the keying scheme is fixed and ready for
    /// the core integration that will consume it; see the type-level caveat.
    pub fn rng_key(&self, sheet_index: u32, row: u32, col: u32, draw_index: u32) -> u64 {
        // SplitMix64-style finalizer chained over the identity tuple — pure,
        // order-independent, and identical across surfaces.
        let mut h = self.rng_seed;
        for part in [
            sheet_index as u64,
            row as u64,
            col as u64,
            draw_index as u64,
        ] {
            h = mix64(h ^ mix64(part));
        }
        h
    }
}

/// One cell whose evaluated value a recalc changed.
///
/// Returned (in deterministic order) by [`Workbook::recalc`] and
/// [`Workbook::recalc_incremental`]: the "change events" of v1, delivered as a
/// value rather than via a callback (value-object ADR). Ordering is pinned —
/// by sheet **tab index**, then row, then column (scope ADR Decision 3) — so
/// the change list is reproducible.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// The sheet's name (its authored casing).
    pub sheet: String,
    /// The recomputed cell's address.
    pub addr: Address,
    /// The cell's value before this recalc (the stored result).
    pub old: Value,
    /// The cell's value after this recalc.
    pub new: Value,
}

impl Workbook {
    /// Recomputes **every** formula cell in dependency order against `ctx`,
    /// writing each new result back into the grid and returning the ordered
    /// list of cells whose value changed.
    ///
    /// Formula cells are evaluated in topological order (precedents first), so
    /// each reads its inputs already current. Cells on a dependency cycle —
    /// and any cell that cannot be ordered because it (transitively) reads one
    /// — take the circular-dependency error ([`CIRCULAR_ERROR`]); recalc always
    /// terminates. Volatile functions are pinned by `ctx` (scope ADR
    /// Decision 3).
    ///
    /// Changes are returned sorted by (sheet tab index, row, column).
    pub fn recalc(&mut self, ctx: &RecalcContext) -> Vec<Change> {
        let graph = DependencyGraph::build(self);
        // Evaluate every formula cell; ordering and cycle handling are shared
        // with the incremental path.
        let to_eval: BTreeSet<CellRef> = graph.formula_cells().cloned().collect();
        self.recompute(&graph, ctx, to_eval)
    }

    /// Recomputes only the formula cells affected by an edit and returns the
    /// ordered changes.
    ///
    /// `edited` lists the cells a mutation touched (the cell written, or — for
    /// a named-range retarget — the name's old and new target cells; callers
    /// pass whatever changed). The recalc closure is the transitive
    /// [`direct_dependents`](DependencyGraph::direct_dependents_of) of those
    /// cells, **plus** every volatile formula cell (always dirty, scope ADR
    /// Decision 3). Everything outside the closure keeps its stored result.
    ///
    /// The result is identical to the subset of [`recalc`](Self::recalc)'s
    /// output for the same edits — the `incremental ≡ full` guarantee.
    pub fn recalc_incremental(
        &mut self,
        ctx: &RecalcContext,
        edited: &[(String, Address)],
    ) -> Vec<Change> {
        let graph = DependencyGraph::build(self);
        let folder = CaseMapperBorrowed::new();

        // Seed the dirty frontier with the dependents of each edited cell.
        let mut dirty: BTreeSet<CellRef> = BTreeSet::new();
        let mut frontier: VecDeque<CellRef> = VecDeque::new();
        for (sheet, addr) in edited {
            let folded = simple_fold(&folder, sheet);
            let seed = CellRef {
                sheet: folded,
                addr: *addr,
            };
            // The edited cell itself recomputes only if it is a formula; its
            // dependents always do.
            if graph.is_formula(&seed) && dirty.insert(seed.clone()) {
                frontier.push_back(seed.clone());
            }
            for dep in graph.direct_dependents_of(&seed) {
                if dirty.insert(dep.clone()) {
                    frontier.push_back(dep);
                }
            }
        }
        // Transitive closure over the formula-cell dependents.
        while let Some(cell) = frontier.pop_front() {
            for dep in graph.direct_dependents_of(&cell) {
                if dirty.insert(dep.clone()) {
                    frontier.push_back(dep);
                }
            }
        }
        // Volatile cells are always dirty (scope ADR Decision 3).
        for cell in graph.formula_cells() {
            if self.is_volatile(cell) {
                dirty.insert(cell.clone());
            }
        }

        // Spill-occupancy seeding (issue #591). A cell's spill footprint or
        // blocked status can change without the dependency graph carrying an
        // edge that would dirty the cells depending on that change, because a
        // spilled cell is not a formula node (P3.2) and a *blocked* anchor
        // stores an error rather than an array that reads its blocker. Two
        // concrete violations of `incremental ≡ full` (P3.3) follow:
        //
        //  - **Shrink / replace-with-scalar.** Setting a former array anchor to
        //    a scalar vacates its old footprint, but `set` has already discarded
        //    the prior array, so the widen loop's `before = anchor_rectangles()`
        //    no longer sees the old rectangle and never dirties the readers of
        //    the vacated cells (e.g. `D1 = =B1+1` after `A1` stops spilling onto
        //    `B1`).
        //  - **Unblock.** Clearing or overwriting the cell that blocks a spill
        //    must let the anchor re-expand, but a blocked anchor has no edge to
        //    its blocker, so clearing the blocker never re-dirties the anchor.
        //
        // Seeding the dirty set with every spill-occupancy-sensitive cell makes
        // the closure independent of which edit triggered the recalc, so the
        // result matches a full recalc despite the lost pre-edit footprint.
        // Over-seeding is safe: a re-evaluated cell whose value is unchanged
        // emits no change event (`diff_against_snapshot`), so `incremental ≡
        // full` is preserved while the minimal-closure guarantee still holds for
        // ordinary (non-spill) edits, which seed nothing here.
        self.seed_spill_sensitive(&graph, &mut dirty);

        // A cell that reads a *spilled* cell has no dependency-graph edge to its
        // spilling anchor (a spilled cell is not a formula node, P3.2), so the
        // closure above can miss a spilled-cell reader when an anchor's spill
        // footprint changes. We widen the dirty set to those readers and re-run
        // until it stabilizes, so an incremental recalc reproduces the full one
        // (`incremental ≡ full`, P3.3) even across spills (§5).
        //
        // To return change events with correct *pre-operation* `old` values
        // despite the multiple internal recomputes, snapshot every formula
        // cell's value first, then recompute over the (growing) dirty set until
        // no anchor's spill footprint changes, and finally diff the resulting
        // grid against the snapshot. The loop is bounded by the formula-cell
        // count (each pass strictly grows the dirty set or stops).
        let pre = self.snapshot_formula_values(&graph);
        let max_widen = graph.formula_cells().count().saturating_add(2).max(1);
        for _ in 0..max_widen {
            let before = self.anchor_rectangles();
            self.recompute(&graph, ctx, dirty.clone());
            let after = self.anchor_rectangles();

            let mut added = false;
            for (sheet, addr) in changed_rectangle_cells(&before, &after) {
                let spilled_ref = CellRef { sheet, addr };
                for dep in graph.direct_dependents_of(&spilled_ref) {
                    if dirty.insert(dep) {
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        self.diff_against_snapshot(pre)
    }

    /// Explains one cell's value against the **currently stored grid** (issue
    /// #743): evaluates `addr`'s formula once through `hook`, resolving every
    /// precedent read to its **stored** value (the same grid-backed
    /// [`Resolver`] semantics `recalc` uses), and returns the value — provably
    /// the same value `recalc`/`recalc_incremental` would write for this cell,
    /// provided the grid is already current for its precedents.
    ///
    /// This is a point-in-time explain, not a recalc: unlike
    /// [`Workbook::recalc`], `trace_cell` does **not** recompute anything
    /// transitively — a precedent's value is whatever is already on the grid
    /// (or, for a cell inside another anchor's placed spill, the
    /// reconstructed spilled element — schema spec §5). If the grid is stale
    /// relative to unapplied edits, `trace_cell` faithfully explains the
    /// *stale* value; call `recalc` or `recalc_incremental` first if the
    /// caller needs a fresh grid.
    ///
    /// Two pieces of `recalc`'s behavior can't be reproduced from the target
    /// cell in isolation, so `trace_cell` matches them explicitly rather than
    /// diverging (an on-demand, single-cell call — a user clicking a cell —
    /// can afford this; see the two call sites below):
    ///
    /// - **Spill occupancy** (schema spec §5): an array result is only stored
    ///   if its target rectangle is free on the current grid; otherwise
    ///   `recalc` stores [`BLOCKED_SPILL_ERROR`] instead, exactly like
    ///   [`Workbook::place_spill`] applies for a real recompute.
    /// - **Dependency cycles**: `recalc` never evaluates a cycle member's
    ///   formula at all — it short-circuits straight to
    ///   [`CIRCULAR_ERROR`] (see [`DependencyGraph::cycle_cells`] and
    ///   `recompute`). Evaluating the formula anyway would diverge whenever it
    ///   *catches* the error (e.g. `IFERROR`), since its precedents' stored
    ///   values already carry the propagated error but `recalc` never gave the
    ///   formula the chance to run.
    ///
    /// `addr` need not be a formula cell: a literal (or empty, or spilled
    /// non-anchor) cell has no expression to trace, so this returns its
    /// resolved value directly without invoking `hook` — `hook` observes no
    /// events in that case, by design (there is nothing to walk). Passing a
    /// hook is optional in the sense that evaluating with `hook = None`'s
    /// counterpart, [`Engine::evaluate_with_resolver_at_keyed`], produces this
    /// same value: `trace_cell` adds observation, it does not change what gets
    /// computed.
    pub fn trace_cell(
        &self,
        sheet: &str,
        addr: Address,
        ctx: &RecalcContext,
        hook: &mut dyn EvalHook,
    ) -> Value {
        let folder = CaseMapperBorrowed::new();
        let own_sheet = simple_fold(&folder, sheet);
        let cell_ref = CellRef {
            sheet: own_sheet.clone(),
            addr,
        };

        // A cycle member never gets its formula evaluated by `recalc` — it is
        // skipped in every pass of `recompute` and then unconditionally
        // assigned `CIRCULAR_ERROR`, regardless of what the formula itself
        // might do with its (already error-tainted) precedents. Match that
        // before evaluating anything. Building the graph is an on-demand,
        // single-cell, interactive call (a user clicking a cell), so
        // correctness beats avoiding the graph walk here.
        let graph = DependencyGraph::build(self);
        if graph.cycle_cells().contains(&cell_ref) {
            return Value::Error(CIRCULAR_ERROR.to_owned());
        }

        // No per-pass recompute state: every precedent read falls straight
        // through to the stored grid (see `GridResolver::cell_value`'s
        // fallback chain), which is exactly "explain given the current grid".
        let empty_values: BTreeMap<CellRef, Value> = BTreeMap::new();
        let empty_spills: BTreeMap<CellRef, SpillRect> = BTreeMap::new();
        let empty_cells: BTreeSet<CellRef> = BTreeSet::new();
        // Nothing is being recomputed, so every anchor on the stored grid is
        // authoritative and the index excludes none of them.
        let sheet_indices = self.sheet_indices_by_folded_name();
        let grid_spills = GridSpillIndex::build(self, &empty_cells);
        let mut resolver = GridResolver {
            workbook: self,
            own_sheet: &own_sheet,
            sheet_indices: &sheet_indices,
            new_values: &empty_values,
            spills: &empty_spills,
            prev_values: &empty_values,
            prev_spills: &empty_spills,
            cycle: &empty_cells,
            grid_spills: &grid_spills,
            current_cell: Some((&own_sheet, addr)),
            scratch_key: fresh_scratch_key(),
        };

        let Some(formula) = self.cell_at(&cell_ref).and_then(Cell::formula) else {
            // Not a formula: nothing to trace. Resolve the cell's own value
            // through the same fallback chain a precedent read would use, so
            // e.g. a spilled (non-anchor) cell still resolves correctly.
            return core_to_workbook(resolver.cell_value(&own_sheet, addr));
        };
        let formula = formula.to_owned();

        let engine = match self.engine() {
            EngineFlavor::Sheets => Engine::sheets(),
            EngineFlavor::Excel => Engine::excel(),
        };
        let sheet_index = sheet_indices.get(&own_sheet).copied().unwrap_or(0);
        let rng_cell = Some((ctx.rng_seed(), sheet_index, addr.row, addr.column));

        let core = engine.evaluate_with_resolver_at_keyed_hooked(
            &formula,
            &mut resolver,
            ctx.now_serial(),
            ctx.now_utc_nanos(),
            rng_cell,
            Some(hook),
        );
        let raw = core_to_workbook(core);

        // Match `eval_formula_cell`'s spill placement: an array result is
        // only stored if its target rectangle is free on the *current*
        // stored grid (`place_spill`/`spill_blocked` read `self.cell_at`
        // directly, so passing fresh, empty per-pass maps here reads exactly
        // that — no real spill state is mutated).
        self.place_spill(&cell_ref, raw, &empty_values, &mut BTreeMap::new())
    }

    /// Shared evaluation core: evaluates `to_eval` (a set of formula cells) in
    /// dependency order through a grid-backed resolver, applies cycle errors,
    /// writes results back, and returns the changes in pinned order.
    fn recompute(
        &mut self,
        graph: &DependencyGraph,
        ctx: &RecalcContext,
        to_eval: BTreeSet<CellRef>,
    ) -> Vec<Change> {
        let now_serial = ctx.now_serial();
        let now_utc_nanos = ctx.now_utc_nanos();
        let rng_seed = ctx.rng_seed();

        // Cells on a cycle short-circuit to the circular error; the rest are
        // evaluated in topological order. Both come from one pass over the
        // graph's formula-cell edges: when the graph is cyclic the order is a
        // best-effort one over the acyclic remainder, so cells that do not
        // touch the cycle still evaluate and cycle-tainted cells fall out as
        // the error below.
        let (order, cycle) = graph.evaluation_order();

        // Evaluate in order, resolving array spills as we go (plan item 3.5,
        // schema spec §5). `new_values` holds each formula's result — a spilling
        // anchor stores its full `array` (its serialized form, §6); a blocked
        // anchor stores the Sheets blocked-spill error and no array. `spills`
        // records the rectangle each *successfully placed* anchor occupies, so
        // (a) a later anchor competing for one of its cells blocks, and (b) the
        // resolver returns spilled values to cells that read them (spilled cells
        // participate in recalc as precedents, §5).
        //
        // A cell that *reads* a spilled cell has no dependency-graph edge to the
        // spilling anchor (a spilled cell is not a formula node, P3.2), so the
        // topological order does not guarantee the anchor is evaluated first. We
        // therefore iterate the pass to a fixpoint: each pass re-evaluates every
        // `to_eval` cell against the prior pass's spills, so a reader that ran
        // before its anchor in one pass sees the spilled value in the next. The
        // grid is finite and spill geometry is monotone (an anchor's array
        // depends only on its own non-spilled precedents), so this converges; we
        // cap the iteration count at the node count as a hard safety bound.
        //
        // Seed the "previous pass" state from the stored grid so an *incremental*
        // recalc — whose `to_eval` is only the dirty closure — still resolves a
        // read of a cell spilled by an anchor that is **not** dirty this pass:
        // that anchor's array is already on the grid, so its spill rectangle is
        // available as a fallback even though it is never re-placed this recalc.
        // A full recalc re-places every anchor, overriding the seed.
        let (mut new_values, mut spills) = self.seed_spills_from_grid();

        // Build the engine — and therefore the function registry — **once** for
        // the whole recalc, not once per formula cell per pass (issue #886).
        // `Engine` holds only a `Copy` flavor and a `Registry` of `fn`-pointer
        // entries; every evaluation entry point takes `&self` and builds its
        // mutable per-evaluation state (`Context`/`EvalCtx`) inside the call, so
        // one instance is safely shared by every cell. Registry construction is
        // ~99 µs against ~0.6 µs to parse and evaluate a cell, so building it
        // per cell was ~97% of recalc time. The folded sheet-name → index map
        // used for the per-cell RNG key is hoisted for the same reason: it was a
        // `CaseMapperBorrowed::new()` plus a linear, allocating scan per cell.
        let engine = match self.engine() {
            EngineFlavor::Sheets => Engine::sheets(),
            EngineFlavor::Excel => Engine::excel(),
        };
        let sheet_indices = self.sheet_indices_by_folded_name();
        // The stored grid's spill anchors, indexed once for the whole recompute
        // (issue #910). Both of its inputs are fixed here: the stored grid does
        // not change until `apply_changes` runs after the last pass, and
        // `to_eval` is fixed on entry. Without it, every read of an *empty*
        // cell fell through to a scan of every authored cell on the sheet.
        let grid_spills = GridSpillIndex::build(self, &to_eval);

        let max_passes = order.len().saturating_add(2).max(1);
        for _ in 0..max_passes {
            let mut next_values: BTreeMap<CellRef, Value> = BTreeMap::new();
            let mut next_spills: BTreeMap<CellRef, SpillRect> = BTreeMap::new();
            for cell in &order {
                if cycle.contains(cell) {
                    continue; // handled in the cycle pass below
                }
                if !to_eval.contains(cell) {
                    continue;
                }
                // Evaluate against this pass's values/spills placed so far, with
                // the *previous* pass's values/spills as a fallback. The
                // fallback is what lets a reader that comes *before* its spill
                // anchor in the order still see the spilled value: the anchor
                // placed its spill in the previous pass, so the reader resolves
                // it from `prev_*` even though `next_*` has not reached the
                // anchor yet this pass.
                let raw = self.eval_formula_cell(
                    cell,
                    &engine,
                    &sheet_indices,
                    now_serial,
                    now_utc_nanos,
                    rng_seed,
                    &next_values,
                    &next_spills,
                    &new_values,
                    &spills,
                    &cycle,
                    &grid_spills,
                );
                // Resolve array results into a placed spill or a blocked-spill
                // error; a placed spill records its rectangle so later anchors
                // and readers see it. Occupancy is judged against authored cells
                // and the spills placed so far this pass.
                let stored = self.place_spill(cell, raw, &next_values, &mut next_spills);
                next_values.insert(cell.clone(), stored);
            }
            let converged = next_values == new_values && next_spills == spills;
            new_values = next_values;
            spills = next_spills;
            if converged {
                break;
            }
        }
        // Cycle cells (and downstream cells the order could not place) take the
        // circular error.
        for cell in &to_eval {
            if !new_values.contains_key(cell) {
                new_values.insert(cell.clone(), Value::Error(CIRCULAR_ERROR.to_owned()));
            }
        }

        self.apply_changes(new_values)
    }

    /// The sheet index every sheet occupies, keyed by its case-folded name —
    /// the `sheet_index` half of the per-cell RNG key. Built once per recalc
    /// (issue #886) so a formula cell costs a map lookup rather than a
    /// `CaseMapperBorrowed::new()` and a linear, allocating scan of the sheet
    /// list.
    fn sheet_indices_by_folded_name(&self) -> BTreeMap<String, u32> {
        let folder = CaseMapperBorrowed::new();
        self.sheets()
            .iter()
            .enumerate()
            .map(|(i, ws)| (simple_fold(&folder, ws.name()), i as u32))
            .collect()
    }

    /// Evaluates a single formula cell through a resolver that reads the *new*
    /// values computed so far this recalc, falling back to the stored grid for
    /// everything else.
    ///
    /// `engine`, `sheet_indices` and `grid_spills` are built once per recalc by
    /// the caller and shared across every cell of the pass (issues #886, #904
    /// and #910).
    #[allow(clippy::too_many_arguments)]
    fn eval_formula_cell(
        &self,
        cell: &CellRef,
        engine: &Engine,
        sheet_indices: &BTreeMap<String, u32>,
        now_serial: Option<f64>,
        now_utc_nanos: Option<i64>,
        rng_seed: u64,
        new_values: &BTreeMap<CellRef, Value>,
        spills: &BTreeMap<CellRef, SpillRect>,
        prev_values: &BTreeMap<CellRef, Value>,
        prev_spills: &BTreeMap<CellRef, SpillRect>,
        cycle: &BTreeSet<CellRef>,
        grid_spills: &GridSpillIndex,
    ) -> Value {
        let formula = match self.cell_at(cell).and_then(Cell::formula) {
            Some(f) => f.to_owned(),
            None => return Value::Empty,
        };
        let sheet_index = sheet_indices.get(&cell.sheet).copied().unwrap_or(0);
        let rng_cell = Some((rng_seed, sheet_index, cell.addr.row, cell.addr.column));
        let mut resolver = GridResolver {
            workbook: self,
            own_sheet: &cell.sheet,
            sheet_indices,
            new_values,
            spills,
            prev_values,
            prev_spills,
            cycle,
            grid_spills,
            current_cell: Some((&cell.sheet, cell.addr)),
            scratch_key: fresh_scratch_key(),
        };
        let core = engine.evaluate_with_resolver_at_keyed(
            &formula,
            &mut resolver,
            now_serial,
            now_utc_nanos,
            rng_cell,
        );
        core_to_workbook(core)
    }

    /// Turns a freshly evaluated formula result into its **stored** value,
    /// applying Sheets spill semantics (plan item 3.5, schema spec §5).
    ///
    /// A non-array result is stored verbatim. An array result is a spill anchor:
    /// it occupies the `m × n` rectangle anchored at `cell`. If every non-anchor
    /// cell of that rectangle is free — not authored, and not already claimed by
    /// an earlier anchor's placed spill (`placed`) — and the rectangle stays in
    /// the sheet's address bounds, the spill is *placed*: its rectangle is
    /// recorded in `placed` and the anchor stores the full array (its serialized
    /// form, §6; the spilled cells are reconstructed, never serialized). If any
    /// target is occupied or the rectangle is out of bounds, the spill is
    /// **blocked**: the anchor takes the Sheets blocked-spill error
    /// ([`BLOCKED_SPILL_ERROR`]) and stores no array (§5, §12).
    fn place_spill(
        &self,
        cell: &CellRef,
        value: Value,
        new_values: &BTreeMap<CellRef, Value>,
        placed: &mut BTreeMap<CellRef, SpillRect>,
    ) -> Value {
        let Value::Array(ref rows) = value else {
            return value; // scalar result: stored as-is
        };
        let nrows = rows.len();
        let ncols = rows.first().map_or(0, Vec::len);
        // `core_array_to_workbook` guarantees a rectangular, ≥ 2-cell array.
        let Some(rect) = spill_rect(cell.addr, nrows, ncols) else {
            // Out-of-bounds rectangle is blocked (§5).
            return Value::Error(BLOCKED_SPILL_ERROR.to_owned());
        };
        if self.spill_blocked(cell, &rect, new_values, placed) {
            return Value::Error(BLOCKED_SPILL_ERROR.to_owned());
        }
        placed.insert(cell.clone(), rect);
        value
    }

    /// Whether the spill `rect` anchored at `cell` is blocked: any non-anchor
    /// cell of the rectangle is authored on that sheet, is itself an evaluated
    /// formula in this recalc (`new_values`), or already lies in an earlier
    /// anchor's placed spill (`placed`). Schema spec §5.
    fn spill_blocked(
        &self,
        cell: &CellRef,
        rect: &SpillRect,
        new_values: &BTreeMap<CellRef, Value>,
        placed: &BTreeMap<CellRef, SpillRect>,
    ) -> bool {
        for addr in rect.spilled_cells() {
            let target = CellRef {
                sheet: cell.sheet.clone(),
                addr,
            };
            // An authored cell in the way (literal or formula).
            if self.cell_at(&target).is_some() {
                return true;
            }
            // A formula cell evaluated this recalc that is not itself authored
            // in the grid cannot exist, but a formula reader could be in
            // `new_values`; treat any computed cell here as occupied for safety.
            if new_values.contains_key(&target) {
                return true;
            }
            // A cell already claimed by an earlier anchor's spill.
            if placed
                .values()
                .any(|r| r.anchor != cell.addr && r.contains(addr))
            {
                return true;
            }
        }
        false
    }

    /// Builds the spill state implied by the **stored** grid: every authored
    /// cell whose stored value is an `array` is a spill anchor occupying its
    /// reconstructed rectangle (schema spec §5). Returns the anchor → array map
    /// and the anchor → rectangle map, used to seed an incremental recalc so a
    /// read of a spilled cell whose anchor is not dirty this pass still resolves
    /// (the anchor placed the spill in a prior recalc). An out-of-bounds stored
    /// array — which a valid document never contains (`from_json` rejects it,
    /// validate.rs §5) — is skipped.
    fn seed_spills_from_grid(&self) -> (BTreeMap<CellRef, Value>, BTreeMap<CellRef, SpillRect>) {
        let folder = CaseMapperBorrowed::new();
        let mut values: BTreeMap<CellRef, Value> = BTreeMap::new();
        let mut spills: BTreeMap<CellRef, SpillRect> = BTreeMap::new();
        for sheet in self.sheets() {
            let folded = simple_fold(&folder, sheet.name());
            for (addr, cell) in sheet.iter() {
                let Value::Array(rows) = cell.value() else {
                    continue;
                };
                let nrows = rows.len();
                let ncols = rows.first().map_or(0, Vec::len);
                if let Some(rect) = spill_rect(addr, nrows, ncols) {
                    let key = CellRef {
                        sheet: folded.clone(),
                        addr,
                    };
                    values.insert(key.clone(), cell.value().clone());
                    spills.insert(key, rect);
                }
            }
        }
        (values, spills)
    }

    /// Writes the recomputed values back, emitting a [`Change`] for each cell
    /// whose value actually changed, in pinned (sheet index, row, column) order.
    fn apply_changes(&mut self, new_values: BTreeMap<CellRef, Value>) -> Vec<Change> {
        let folder = CaseMapperBorrowed::new();
        // Resolve folded sheet names to tab index + authored name once.
        let mut changes: Vec<(usize, Change)> = Vec::new();
        for (cell, new) in new_values {
            let Some(idx) = self.sheet_index_folded(&folder, &cell.sheet) else {
                continue; // sheet vanished (cannot happen mid-recalc)
            };
            let sheet_name = self.sheets()[idx].name().to_owned();
            let old = self.sheets()[idx]
                .get(cell.addr)
                .map(|c| c.value().clone())
                .unwrap_or(Value::Empty);
            if old == new {
                continue;
            }
            // Preserve the formula text; only the stored value updates.
            let formula = self.sheets()[idx]
                .get(cell.addr)
                .and_then(|c| c.formula())
                .map(str::to_owned);
            if let Some(formula) = formula {
                self.sheets_mut()[idx].set(cell.addr, Cell::with_formula(formula, new.clone()));
            }
            changes.push((
                idx,
                Change {
                    sheet: sheet_name,
                    addr: cell.addr,
                    old,
                    new,
                },
            ));
        }
        // Pin order: sheet tab index, then row, then column.
        changes.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.addr.row.cmp(&b.1.addr.row))
                .then(a.1.addr.column.cmp(&b.1.addr.column))
        });
        changes.into_iter().map(|(_, c)| c).collect()
    }

    /// Whether `cell`'s formula calls any volatile function (`NOW`, `TODAY`,
    /// `RAND`, `RANDBETWEEN`, `RANDARRAY` — core's `VOLATILE_FUNCTIONS`).
    /// A volatile cell is always dirty in incremental recalc.
    fn is_volatile(&self, cell: &CellRef) -> bool {
        let Some(formula) = self.cell_at(cell).and_then(Cell::formula) else {
            return false;
        };
        let upper = formula.to_ascii_uppercase();
        truecalc_core::Registry::VOLATILE_FUNCTIONS
            .iter()
            .any(|name| contains_call(&upper, name))
    }

    /// Every formula cell's current stored value, keyed by [`CellRef`]. The
    /// pre-operation snapshot an incremental recalc diffs its final grid against
    /// to emit change events with correct `old` values despite internal
    /// re-recomputes (spill widening).
    fn snapshot_formula_values(&self, graph: &DependencyGraph) -> BTreeMap<CellRef, Value> {
        let mut snap = BTreeMap::new();
        for cell in graph.formula_cells() {
            let value = self
                .cell_at(cell)
                .map(|c| c.value().clone())
                .unwrap_or(Value::Empty);
            snap.insert(cell.clone(), value);
        }
        snap
    }

    /// Adds every spill-occupancy-sensitive formula cell to `dirty` (issue
    /// #591), so an incremental recalc reproduces a full recalc across any spill
    /// footprint or blocked-status change even though the dependency graph
    /// carries no edge for those transitions and `set` discarded the pre-edit
    /// footprint.
    ///
    /// A cell is seeded when it is, or reads something that can become or cease
    /// being, a spill:
    ///
    ///  1. **Every array anchor** (a formula cell whose stored value is an
    ///     array) — re-placed so a footprint that should shrink or grow does so,
    ///     and so a write into its region re-blocks it.
    ///  2. **Every blocked-spill anchor** (a formula cell whose stored value is
    ///     the blocked-spill error) — re-attempted so clearing/overwriting its
    ///     blocker lets it re-expand (the unblock case).
    ///  3. **Every reader of a non-authored single cell** — that precedent is
    ///     empty or spilled today and may flip either way, e.g. `D1 = =B1+1`
    ///     whose `B1` was spilled by a now-shrunk anchor (the vacated-reader
    ///     case), or a reader of a cell a spill is about to grow onto.
    ///  4. **Every reader of a range that overlaps a current spill rectangle** —
    ///     a range aggregation whose window includes spilled cells, so a change
    ///     to that spill (grow/shrink/block) re-aggregates.
    ///
    /// The blocked-spill error string equals [`BLOCKED_SPILL_ERROR`]; a cell
    /// merely *holding* that error that is not actually a former/blocked spill
    /// anchor is harmless to re-evaluate (it recomputes to the same value).
    fn seed_spill_sensitive(&self, graph: &DependencyGraph, dirty: &mut BTreeSet<CellRef>) {
        let rects = self.anchor_rectangles();
        for cell in graph.formula_cells() {
            // (1)/(2): the cell itself is (or held) a spill.
            let is_spill_cell = match self.cell_at(cell).map(Cell::value) {
                Some(Value::Array(_)) => true,
                Some(Value::Error(code)) | Some(Value::ErrorMsg(code, _)) => {
                    code == BLOCKED_SPILL_ERROR
                }
                _ => false,
            };
            let mut seed = is_spill_cell;
            // (3)/(4): the cell reads a spill-sensitive precedent.
            if !seed {
                if let Some(precedents) = graph.precedents_of(cell) {
                    seed = precedents
                        .iter()
                        .any(|p| self.precedent_is_spill_sensitive(p, &rects));
                }
            }
            if seed {
                dirty.insert(cell.clone());
            }
        }
    }

    /// Whether a single precedent reads a cell that is, or could become, a
    /// spilled cell (issue #591): a non-authored single-cell target (empty or
    /// spilled today), or a range overlapping a current spill rectangle.
    fn precedent_is_spill_sensitive(
        &self,
        precedent: &Precedent,
        rects: &BTreeMap<CellRef, SpillRect>,
    ) -> bool {
        match precedent {
            // A single-cell precedent that is not authored is empty or spilled
            // today, and may flip either way (grow/shrink/block/unblock).
            Precedent::Cell(c) => self.cell_at(c).is_none(),
            // A range precedent is spill-sensitive if it overlaps a current
            // spill rectangle (a spill could grow/shrink/block within it) *or*
            // if it contains any non-authored cell — which catches a cell a
            // spill *used to* cover but no longer does (the lost pre-edit
            // footprint of a shrink/collapse), since that cell is now empty.
            Precedent::Range(r) => {
                rects
                    .iter()
                    .any(|(anchor, rect)| anchor.sheet == r.sheet && rect_overlaps_range(rect, r))
                    || self.range_has_unauthored_cell(r)
            }
            // A name resolves to a cell or range; treat it conservatively as
            // spill-sensitive so a name pointing at a spilled cell still seeds
            // its reader. Names are rare and this only widens the dirty set.
            Precedent::Name(_) => true,
            Precedent::Unresolved(_) => false,
        }
    }

    /// Whether the range `r` contains at least one cell that is **not** an
    /// authored cell (empty or spilled). Computed by comparing the range's area
    /// to the number of authored cells inside it — so the cost is bounded by the
    /// sheet's populated-cell count, never the range area (issue #591).
    fn range_has_unauthored_cell(&self, r: &RangeRef) -> bool {
        let folder = CaseMapperBorrowed::new();
        let Some(sheet) = self
            .sheets()
            .iter()
            .find(|s| simple_fold(&folder, s.name()) == r.sheet)
        else {
            // The range targets a missing sheet; nothing authored, so it is
            // (vacuously) all-unauthored — seed conservatively.
            return true;
        };
        let rows = (r.end.row - r.start.row + 1) as u64;
        let cols = (r.end.column - r.start.column + 1) as u64;
        let area = rows.saturating_mul(cols);
        let authored_inside = sheet
            .iter()
            .filter(|(addr, _)| {
                addr.row >= r.start.row
                    && addr.row <= r.end.row
                    && addr.column >= r.start.column
                    && addr.column <= r.end.column
            })
            .count() as u64;
        authored_inside < area
    }

    /// Every spill rectangle currently on the stored grid (anchor → rectangle),
    /// derived from authored cells whose stored value is an array (schema spec
    /// §5). Used to detect when an incremental recompute changed a spill
    /// footprint so the affected readers can be dirtied.
    fn anchor_rectangles(&self) -> BTreeMap<CellRef, SpillRect> {
        let folder = CaseMapperBorrowed::new();
        let mut rects = BTreeMap::new();
        for sheet in self.sheets() {
            let folded = simple_fold(&folder, sheet.name());
            for (addr, cell) in sheet.iter() {
                let Value::Array(rows) = cell.value() else {
                    continue;
                };
                let nrows = rows.len();
                let ncols = rows.first().map_or(0, Vec::len);
                if let Some(rect) = spill_rect(addr, nrows, ncols) {
                    rects.insert(
                        CellRef {
                            sheet: folded.clone(),
                            addr,
                        },
                        rect,
                    );
                }
            }
        }
        rects
    }

    /// Emits the change list for an incremental recalc by diffing the final grid
    /// against the pre-operation `snapshot`: one [`Change`] per formula cell
    /// whose value differs, in the pinned (sheet tab index, row, column) order.
    fn diff_against_snapshot(&self, snapshot: BTreeMap<CellRef, Value>) -> Vec<Change> {
        let folder = CaseMapperBorrowed::new();
        let mut changes: Vec<(usize, Change)> = Vec::new();
        for (cell, old) in snapshot {
            let Some(idx) = self.sheet_index_folded(&folder, &cell.sheet) else {
                continue;
            };
            let new = self.sheets()[idx]
                .get(cell.addr)
                .map(|c| c.value().clone())
                .unwrap_or(Value::Empty);
            if old == new {
                continue;
            }
            changes.push((
                idx,
                Change {
                    sheet: self.sheets()[idx].name().to_owned(),
                    addr: cell.addr,
                    old,
                    new,
                },
            ));
        }
        changes.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.addr.row.cmp(&b.1.addr.row))
                .then(a.1.addr.column.cmp(&b.1.addr.column))
        });
        changes.into_iter().map(|(_, c)| c).collect()
    }

    /// The cell at a [`CellRef`] (folded sheet + address), or `None`.
    fn cell_at(&self, cell: &CellRef) -> Option<&Cell> {
        let folder = CaseMapperBorrowed::new();
        let idx = self.sheet_index_folded(&folder, &cell.sheet)?;
        self.sheets()[idx].get(cell.addr)
    }

    /// Tab index of the sheet whose folded name equals `folded`.
    fn sheet_index_folded(
        &self,
        folder: &CaseMapperBorrowed<'static>,
        folded: &str,
    ) -> Option<usize> {
        self.sheets()
            .iter()
            .position(|s| simple_fold(folder, s.name()) == folded)
    }
}

/// A [`Resolver`] backed by the workbook grid, reading the values computed so
/// far this recalc before falling back to the stored grid.
struct GridResolver<'a> {
    workbook: &'a Workbook,
    own_sheet: &'a str,
    /// Every sheet's tab index, keyed by its case-folded name, built once per
    /// recalc by the caller. Resolving a read's target sheet is a map probe
    /// against this (issue #904); it used to be a linear scan of the sheet list
    /// that case-folded — and so allocated — every sheet name, **per element
    /// scanned**, to find a sheet that cannot change between the elements of
    /// one range.
    sheet_indices: &'a BTreeMap<String, u32>,
    new_values: &'a BTreeMap<CellRef, Value>,
    /// Spills placed so far **this pass** (anchor → rectangle): a read of a cell
    /// inside one of these rectangles resolves to the spilled array element
    /// (schema spec §5 — spilled cells participate as precedents).
    spills: &'a BTreeMap<CellRef, SpillRect>,
    /// The **previous** pass's values, used as a fallback so a reader ordered
    /// before its spill anchor still sees the spilled value (the anchor placed
    /// it last pass). Empty on the first pass.
    prev_values: &'a BTreeMap<CellRef, Value>,
    /// The previous pass's spills (same fallback role as `prev_values`).
    prev_spills: &'a BTreeMap<CellRef, SpillRect>,
    cycle: &'a BTreeSet<CellRef>,
    /// The stored grid's spill anchors, indexed once per recalc, already
    /// excluding the anchors being recomputed (issue #591 — see
    /// [`GridSpillIndex::build`]). Backs the `grid_spilled_value` fallback,
    /// which used to re-derive this by scanning every authored cell on the
    /// sheet on every read of an empty cell (issue #910).
    grid_spills: &'a GridSpillIndex,
    /// The evaluating cell's own `(folded sheet name, address)` — set at the
    /// same call site that computes `rng_cell`'s `(sheet_index, row, col)`, so
    /// both stay in sync by construction. Used by `resolve_table_ref` to look
    /// up the single current-row cell and to infer a table from an
    /// unqualified `[@col]` reference's containment. Always `Some` at every
    /// current construction site (kept `Option` defensively, since a
    /// resolver constructed without a specific evaluating cell would have
    /// nothing to thread here).
    current_cell: Option<(&'a str, Address)>,
    /// A reusable `CellRef` for this resolver's map probes.
    ///
    /// `cell_value` probes three `CellRef`-keyed maps, and the owned sheet name
    /// those keys need used to be allocated fresh **per element scanned**
    /// (issue #904). Every element of one range shares a sheet name, so
    /// [`GridResolver::probe_key`] refills this key in place instead:
    /// `String::clear` keeps the buffer, so the `push_str` that follows reuses
    /// it. `RefCell` rather than `&mut self` because `resolve_range` and
    /// `resolve_table_ref` hold shared borrows of `self` across their
    /// `cell_value` calls.
    scratch_key: RefCell<CellRef>,
}

/// An empty scratch key for a freshly built [`GridResolver`]. The address is a
/// placeholder: `probe_key` overwrites both fields before any probe reads them.
fn fresh_scratch_key() -> RefCell<CellRef> {
    RefCell::new(CellRef {
        sheet: String::new(),
        addr: Address::new(1, 1).expect("A1 is in bounds"),
    })
}

impl GridResolver<'_> {
    /// The current value of a resolved cell: this recalc's fresh value if it
    /// was already computed, else the stored grid value, else empty. A cell on
    /// a cycle resolves to the circular error (so a cell that *reads* a cycle
    /// inherits the taint).
    fn cell_value(&self, sheet_folded: &str, addr: Address) -> CoreValue {
        {
            let key = self.probe_key(sheet_folded, addr);
            if self.cycle.contains(&key) {
                return CoreValue::Error(ErrorKind::Ref);
            }
            if let Some(v) = self.new_values.get(&key) {
                return workbook_to_core(v);
            }
        }
        if let Some(c) = self.sheet(sheet_folded).and_then(|s| s.get(addr)) {
            return workbook_to_core(c.value());
        }
        // Not authored and not freshly computed: it may be a spilled cell of an
        // anchor placed this pass — or, if the anchor is ordered *after* this
        // reader, of the previous pass (schema spec §5). Resolve through the
        // spill, preferring this pass's placement.
        if let Some(v) = self.spilled_value(sheet_folded, addr, self.spills, self.new_values) {
            return workbook_to_core(&v);
        }
        if let Some(v) = self.spilled_value(sheet_folded, addr, self.prev_spills, self.prev_values)
        {
            return workbook_to_core(&v);
        }
        // A cell whose value the previous pass computed but this pass has not
        // reached yet (a reader's plain-cell precedent ordered after it).
        if let Some(v) = self.prev_values.get(&self.probe_key(sheet_folded, addr)) {
            return workbook_to_core(v);
        }
        // Final fallback (matters for *incremental* recalc): the cell may be
        // spilled by an anchor that is not dirty this recalc, so it never enters
        // the per-pass maps. Its array is on the stored grid; reconstruct the
        // element directly (schema spec §5).
        if let Some(v) = self.grid_spilled_value(sheet_folded, addr) {
            return workbook_to_core(&v);
        }
        CoreValue::Empty
    }

    /// The map-probe key for `(sheet_folded, addr)`, refilling the scratch
    /// [`CellRef`] in place rather than allocating a fresh owned sheet name per
    /// probe (issue #904). Consecutive probes of one range share a sheet name,
    /// so the `push_str` reuses the buffer `clear` left behind.
    ///
    /// The borrow must not be held across anything that could re-enter
    /// `cell_value`; every use below is scoped to a single probe.
    fn probe_key(&self, sheet_folded: &str, addr: Address) -> RefMut<'_, CellRef> {
        let mut key = self.scratch_key.borrow_mut();
        if key.sheet != sheet_folded {
            key.sheet.clear();
            key.sheet.push_str(sheet_folded);
        }
        key.addr = addr;
        key
    }

    /// The sheet whose folded name is `sheet_folded`, via the recalc-wide index
    /// (issue #904): a map probe, with no case-folding and no allocation.
    fn sheet(&self, sheet_folded: &str) -> Option<&Worksheet> {
        let index = *self.sheet_indices.get(sheet_folded)? as usize;
        self.workbook.sheets().get(index)
    }

    /// The value spilled to `addr` on `sheet_folded` per the **stored grid**:
    /// finds the anchor whose stored rectangle covers `addr` and reconstructs
    /// the element (schema spec §5). Used as the incremental-recalc fallback for
    /// spills whose anchor is not re-evaluated this pass.
    ///
    /// Only the sheet's *spill anchors* are examined, from the recalc-wide
    /// [`GridSpillIndex`] — which also applies the `#591` exclusion of anchors
    /// being recomputed. This used to scan every authored cell on the sheet, on
    /// every read of an empty cell, at one allocation per cell scanned (issue
    /// #910).
    fn grid_spilled_value(&self, sheet_folded: &str, addr: Address) -> Option<Value> {
        let anchors = self.grid_spills.anchors(sheet_folded);
        if anchors.is_empty() {
            return None;
        }
        let sheet = self.sheet(sheet_folded)?;
        for &(anchor_addr, rect) in anchors {
            if anchor_addr == addr {
                continue;
            }
            let Some((i, j)) = rect.offset_of(addr) else {
                continue;
            };
            let Some(Value::Array(rows)) = sheet.get(anchor_addr).map(Cell::value) else {
                continue; // unreachable: the index only holds array anchors
            };
            return rows.get(i).and_then(|r| r.get(j)).cloned();
        }
        None
    }

    /// The value spilled to `addr` on `sheet_folded` per a given `spills` map
    /// and its backing `values`: the `[i][j]` element of the anchor's stored
    /// array (schema spec §5). `None` if `addr` is not a non-anchor cell of any
    /// spill in `spills`.
    fn spilled_value(
        &self,
        sheet_folded: &str,
        addr: Address,
        spills: &BTreeMap<CellRef, SpillRect>,
        values: &BTreeMap<CellRef, Value>,
    ) -> Option<Value> {
        for (anchor, rect) in spills {
            if anchor.sheet != sheet_folded {
                continue;
            }
            if anchor.addr == addr {
                continue; // the anchor itself is in `values`
            }
            let Some((i, j)) = rect.offset_of(addr) else {
                continue;
            };
            if let Some(Value::Array(rows)) = values.get(anchor) {
                return rows.get(i).and_then(|r| r.get(j)).cloned();
            }
        }
        None
    }

    /// Resolves the folded target sheet name for a `Ref`'s optional sheet
    /// qualifier, or `None` if the named sheet does not exist.
    fn target_sheet(&self, sheet: &Option<String>) -> Option<String> {
        let folder = CaseMapperBorrowed::new();
        match sheet {
            None => Some(self.own_sheet.to_owned()),
            Some(name) => self
                .workbook
                .sheet(name)
                .map(|s| simple_fold(&folder, s.name())),
        }
    }
}

impl Resolver for GridResolver<'_> {
    fn resolve(&mut self, r: &Ref) -> CoreValue {
        match r {
            Ref::Cell { sheet, addr } => {
                let Some(target) = self.target_sheet(sheet) else {
                    return CoreValue::Error(ErrorKind::Ref);
                };
                match Address::new(addr.row, addr.col) {
                    Some(a) => self.cell_value(&target, a),
                    None => CoreValue::Error(ErrorKind::Ref),
                }
            }
            Ref::Range { sheet, start, end } => {
                let Some(target) = self.target_sheet(sheet) else {
                    return CoreValue::Error(ErrorKind::Ref);
                };
                self.resolve_range(&target, start, end)
            }
            Ref::Name(name) => {
                // Resolve the name to its canonical ref, then resolve that.
                let folder = CaseMapperBorrowed::new();
                let folded = simple_fold(&folder, name);
                let target = self
                    .workbook
                    .names()
                    .iter()
                    .find(|nr| simple_fold(&folder, &nr.name) == folded);
                match target {
                    None => CoreValue::Error(ErrorKind::Name),
                    // Re-parse the name's canonical `Sheet!A1` ref so a name
                    // pointing at a cell or a range resolves identically to a
                    // literal ref of the same shape.
                    Some(nr) => self.resolve_name_ref(&nr.r#ref),
                }
            }
            Ref::Table {
                table,
                column,
                this_row,
            } => self.resolve_table_ref(table.as_deref(), column, *this_row),
        }
    }
}

impl GridResolver<'_> {
    /// Materializes a range as a core `Value::Array` of its cells in row-major
    /// reading order — the shape the P1.3 [`Resolver`] contract specifies
    /// ("a range -> a Value::Array of the cells in reading order") and the shape
    /// core's aggregations (SUM/AVERAGE/COUNT/SUMIF) and shape functions
    /// consume.
    ///
    /// A single-column, multi-row range (a *vertical* range) is materialized
    /// as a nested `Array` of one-element row `Array`s — core's Nx1 column
    /// shape (see `to_2d`/`from_2d` in the array functions) — so elementwise
    /// operations over it (e.g. `=A1:A3*2`) spill down like Google Sheets,
    /// instead of losing their column orientation to a flat row. Every other
    /// shape (a single row, a single cell, or a genuine 2-D block) keeps the
    /// existing flat row-major array, unchanged. The own/target sheet has
    /// already been resolved.
    fn resolve_range(
        &self,
        sheet_folded: &str,
        start: &truecalc_core::CellAddr,
        end: &truecalc_core::CellAddr,
    ) -> CoreValue {
        let (r0, r1) = (start.row.min(end.row), start.row.max(end.row));
        let (c0, c1) = (start.col.min(end.col), start.col.max(end.col));
        let is_vertical = r1 > r0 && c0 == c1;
        let mut cells: Vec<CoreValue> = Vec::new();
        for r in r0..=r1 {
            for c in c0..=c1 {
                match Address::new(r, c) {
                    Some(a) => {
                        let v = self.cell_value(sheet_folded, a);
                        // A spill anchor stores the full array; its individual
                        // elements are visited when the range iteration reaches
                        // the spilled positions (which resolve via spilled_value).
                        // Use only the [0][0] element here to avoid double-counting.
                        let scalar = match v {
                            CoreValue::Array(ref rows) => match rows.first() {
                                Some(CoreValue::Array(ref cols)) => {
                                    cols.first().cloned().unwrap_or(CoreValue::Empty)
                                }
                                Some(other) => other.clone(),
                                None => CoreValue::Empty,
                            },
                            other => other,
                        };
                        cells.push(if is_vertical {
                            CoreValue::Array(vec![scalar])
                        } else {
                            scalar
                        });
                    }
                    None => cells.push(if is_vertical {
                        CoreValue::Array(vec![CoreValue::Error(ErrorKind::Ref)])
                    } else {
                        CoreValue::Error(ErrorKind::Ref)
                    }),
                }
            }
        }
        CoreValue::Array(cells)
    }

    /// Resolves a `Ref::Table`: whole-column (`this_row: false`) materializes
    /// the column's data-row values as an array, using the **same** vertical
    /// wrapping [`resolve_range`](Self::resolve_range) uses for a
    /// single-column range (its `is_vertical` branch: one array element per
    /// row, each itself a one-element array — core's Nx1 column shape) — so
    /// `T[col]` broadcasts and spills identically to an equivalent explicit
    /// `A2:A12`-style reference. Current-row (`this_row: true`) looks up the
    /// single cell at `(current row, column)`.
    ///
    /// An unqualified reference (`table: None`) infers the table from
    /// `self.current_cell`'s containment within the table's *data* rows
    /// (excluding the header row); a qualified reference looks the table up
    /// by name directly. `#REF!` if the table doesn't exist, the column
    /// doesn't exist (looked up by reading the header row), or — for
    /// current-row only — the evaluating cell isn't inside the resolved
    /// table's data rows.
    fn resolve_table_ref(&self, table: Option<&str>, column: &str, this_row: bool) -> CoreValue {
        let folder = CaseMapperBorrowed::new();
        let target_table = match table {
            Some(name) => {
                let folded = simple_fold(&folder, name);
                self.workbook
                    .tables()
                    .iter()
                    .find(|t| simple_fold(&folder, &t.name) == folded)
            }
            None => {
                let Some((sheet, addr)) = self.current_cell else {
                    return CoreValue::Error(ErrorKind::Ref);
                };
                self.workbook.tables().iter().find(|t| {
                    named_ref::parse_canonical_ref(&t.r#ref)
                        .ok()
                        .and_then(|parsed| table_ref::parsed_range_bounds(&t.r#ref, &parsed))
                        .is_some_and(|b| {
                            simple_fold(&folder, &b.sheet) == sheet
                                && b.row_start < addr.row
                                && addr.row <= b.row_end
                                && b.col_start <= addr.column
                                && addr.column <= b.col_end
                        })
                })
            }
        };
        let Some(t) = target_table else {
            return CoreValue::Error(ErrorKind::Ref);
        };
        let Ok(parsed) = named_ref::parse_canonical_ref(&t.r#ref) else {
            return CoreValue::Error(ErrorKind::Ref);
        };
        let Some(bounds) = table_ref::parsed_range_bounds(&t.r#ref, &parsed) else {
            return CoreValue::Error(ErrorKind::Ref);
        };
        let sheet_folded = simple_fold(&folder, &bounds.sheet);

        // Find the column's index by reading the header row (`bounds.row_start`).
        // Case-insensitive, same as the table-name and sheet-name lookups
        // above: column names are case-folded at table-definition time
        // (`table_ref::header_row_columns`), so lookup must match.
        let column_folded = simple_fold(&folder, column);
        let mut col = None;
        for c in bounds.col_start..=bounds.col_end {
            if let Some(a) = Address::new(bounds.row_start, c) {
                if let CoreValue::Text(header) = self.cell_value(&sheet_folded, a) {
                    if simple_fold(&folder, &header) == column_folded {
                        col = Some(c);
                        break;
                    }
                }
            }
        }
        let Some(col) = col else {
            return CoreValue::Error(ErrorKind::Ref);
        };

        if this_row {
            let Some((cell_sheet, cell_addr)) = self.current_cell else {
                return CoreValue::Error(ErrorKind::Ref);
            };
            if cell_sheet != sheet_folded
                || cell_addr.row <= bounds.row_start
                || cell_addr.row > bounds.row_end
            {
                return CoreValue::Error(ErrorKind::Ref);
            }
            match Address::new(cell_addr.row, col) {
                Some(a) => self.cell_value(&sheet_folded, a),
                None => CoreValue::Error(ErrorKind::Ref),
            }
        } else {
            let data_start = bounds.row_start + 1;
            let mut cells = Vec::new();
            for r in data_start..=bounds.row_end {
                let scalar = match Address::new(r, col) {
                    Some(a) => {
                        let v = self.cell_value(&sheet_folded, a);
                        // Same spill-anchor unwrap as `resolve_range`: a spill
                        // anchor stores its full array, so use only the
                        // [0][0] element here — otherwise a table-column cell
                        // that happens to be a spill anchor would nest its
                        // whole array as this row's "scalar" instead of
                        // resolving to the same value an equivalent
                        // `A2:A12`-style range would produce.
                        match v {
                            CoreValue::Array(ref rows) => match rows.first() {
                                Some(CoreValue::Array(ref cols)) => {
                                    cols.first().cloned().unwrap_or(CoreValue::Empty)
                                }
                                Some(other) => other.clone(),
                                None => CoreValue::Empty,
                            },
                            other => other,
                        }
                    }
                    None => CoreValue::Error(ErrorKind::Ref),
                };
                // Same wrapping as `resolve_range`'s `is_vertical` branch: one
                // array element per data row, each a one-element array.
                cells.push(CoreValue::Array(vec![scalar]));
            }
            CoreValue::Array(cells)
        }
    }

    /// Resolves a named range's canonical `ref` string (`Sheet!A1` /
    /// `Sheet!A1:B2`) the same way a literal reference resolves.
    fn resolve_name_ref(&mut self, r: &str) -> CoreValue {
        // The ref string parses as a one-reference formula; extract and resolve.
        // Parsed without an `Engine`: parsing is flavor-independent and never
        // reads the function registry, so constructing one per resolved
        // named-range reference was pure waste (issue #900).
        let formula = format!("={r}");
        match truecalc_core::parse_formula(&formula) {
            Ok(expr) => {
                let refs = truecalc_core::extract_refs(&expr);
                match refs.first() {
                    Some(first) => self.resolve(first),
                    None => CoreValue::Error(ErrorKind::Ref),
                }
            }
            Err(_) => CoreValue::Error(ErrorKind::Ref),
        }
    }
}

/// Maps a core evaluated [`CoreValue`] to the workbook [`Value`] (schema §6).
/// Core arrays (flat or nested rows) become a rectangular 2-D workbook array;
/// a 1×1 array collapses to its scalar (schema §6).
fn core_to_workbook(v: CoreValue) -> Value {
    match v {
        CoreValue::Number(n) => Value::Number(n),
        CoreValue::Text(s) => Value::Text(s),
        CoreValue::Bool(b) => Value::Boolean(b),
        CoreValue::Error(e) => Value::Error(e.to_string()),
        CoreValue::ErrorMsg(e, m) => Value::ErrorMsg(e.to_string(), m),
        CoreValue::Empty => Value::Empty,
        CoreValue::Date(n) => Value::Date(n),
        CoreValue::Zoned(z) => Value::Zoned(z),
        CoreValue::Sparkline(spec) => Value::Sparkline(spec),
        CoreValue::Array(elems) => core_array_to_workbook(elems),
    }
}

/// Normalizes a core array (which may be flat scalars or nested rows) into the
/// workbook's row-major 2-D shape, collapsing a 1×1 array to its scalar.
fn core_array_to_workbook(elems: Vec<CoreValue>) -> Value {
    if elems.is_empty() {
        // An empty array has no scalar form; surface as #REF! (a degenerate
        // spill the P3.5 engine will own). Kept minimal here.
        return Value::Error("#REF!".to_owned());
    }
    let nested = elems.iter().all(|e| matches!(e, CoreValue::Array(_)));
    let rows: Vec<Vec<Value>> = if nested {
        elems
            .into_iter()
            .map(|row| match row {
                CoreValue::Array(cells) => cells.into_iter().map(core_to_workbook).collect(),
                other => vec![core_to_workbook(other)],
            })
            .collect()
    } else {
        vec![elems.into_iter().map(core_to_workbook).collect()]
    };
    if rows.len() == 1 && rows[0].len() == 1 {
        return rows.into_iter().next().unwrap().into_iter().next().unwrap();
    }
    Value::Array(rows)
}

/// Maps a workbook [`Value`] back to a core [`CoreValue`] for feeding a stored
/// cell value into evaluation through the resolver.
fn workbook_to_core(v: &Value) -> CoreValue {
    match v {
        Value::Number(n) => CoreValue::Number(*n),
        Value::Text(s) => CoreValue::Text(s.clone()),
        Value::Boolean(b) => CoreValue::Bool(*b),
        Value::Error(code) | Value::ErrorMsg(code, _) => {
            CoreValue::Error(error_kind_from_code(code))
        }
        Value::Empty => CoreValue::Empty,
        Value::Date(n) => CoreValue::Date(*n),
        Value::Zoned(z) => CoreValue::Zoned(z.clone()),
        Value::Sparkline(spec) => CoreValue::Sparkline(spec.clone()),
        Value::Array(rows) => CoreValue::Array(
            rows.iter()
                .map(|row| CoreValue::Array(row.iter().map(workbook_to_core).collect()))
                .collect(),
        ),
    }
}

/// Parses a Sheets error code string back to a core [`ErrorKind`]; an unknown
/// code maps to `#REF!` (the most conservative reference error).
fn error_kind_from_code(code: &str) -> ErrorKind {
    match code {
        "#DIV/0!" => ErrorKind::DivByZero,
        "#VALUE!" => ErrorKind::Value,
        "#REF!" => ErrorKind::Ref,
        "#NAME?" => ErrorKind::Name,
        "#NUM!" => ErrorKind::Num,
        "#N/A" => ErrorKind::NA,
        "#NULL!" => ErrorKind::Null,
        _ => ErrorKind::Ref,
    }
}

/// Whether `upper` (an upper-cased formula) calls the function `name`, i.e.
/// `name` appears followed by `(` (ignoring spaces). Avoids matching a name
/// that is merely a substring of a longer identifier.
fn contains_call(upper: &str, name: &str) -> bool {
    let bytes = upper.as_bytes();
    let nb = name.as_bytes();
    let mut i = 0;
    while let Some(pos) = find_from(bytes, nb, i) {
        // Preceding char must not be an identifier char.
        let before_ok = pos == 0 || !is_ident_byte(bytes[pos - 1]);
        // Following non-space char must be '('.
        let mut j = pos + nb.len();
        while j < bytes.len() && bytes[j] == b' ' {
            j += 1;
        }
        let after_ok = j < bytes.len() && bytes[j] == b'(';
        if before_ok && after_ok {
            return true;
        }
        i = pos + 1;
    }
    false
}

fn find_from(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from + needle.len() > haystack.len() {
        return None;
    }
    (from..=haystack.len() - needle.len()).find(|&i| &haystack[i..i + needle.len()] == needle)
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The set of `(folded sheet, address)` cells whose spill coverage changed
/// between two anchor-rectangle maps: the union of all cells in any rectangle
/// that appeared, vanished, or resized (schema spec §5). Their readers may now
/// be stale and must be dirtied in an incremental recalc.
fn changed_rectangle_cells(
    before: &BTreeMap<CellRef, SpillRect>,
    after: &BTreeMap<CellRef, SpillRect>,
) -> BTreeSet<(String, Address)> {
    let mut out: BTreeSet<(String, Address)> = BTreeSet::new();
    let mut consider = |anchor: &CellRef, rect: &SpillRect| {
        // The anchor cell itself is a formula node with its own graph edges;
        // only the spilled cells need this spill-aware dirtying.
        for addr in rect.spilled_cells() {
            out.insert((anchor.sheet.clone(), addr));
        }
    };
    for (anchor, rect) in before {
        match after.get(anchor) {
            Some(same) if same == rect => {}
            _ => consider(anchor, rect),
        }
    }
    for (anchor, rect) in after {
        match before.get(anchor) {
            Some(same) if same == rect => {}
            _ => consider(anchor, rect),
        }
    }
    out
}

/// Whether a spill rectangle and a range reference overlap (same sheet assumed
/// checked by the caller): their inclusive row/column extents intersect (issue
/// #591). Used to seed range aggregations that read spilled cells.
fn rect_overlaps_range(rect: &SpillRect, range: &RangeRef) -> bool {
    let rect_r0 = rect.anchor.row;
    let rect_r1 = rect.anchor.row + rect.rows - 1;
    let rect_c0 = rect.anchor.column;
    let rect_c1 = rect.anchor.column + rect.cols - 1;
    rect_r0 <= range.end.row
        && rect_r1 >= range.start.row
        && rect_c0 <= range.end.column
        && rect_c1 >= range.start.column
}

/// SplitMix64 finalizer — a fast, well-distributed integer mix.
fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
