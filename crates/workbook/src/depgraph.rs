//! Dependency graph for the workbook runtime (plan item 3.2, issue #534).
//!
//! The graph records, for every formula cell, *which cells, ranges, and named
//! ranges it reads* — its **precedents** — derived once from the parsed
//! formula via [`truecalc_core::extract_refs`] (P1.3). The reverse edges (a
//! cell's **dependents**: the formula cells that must recalculate when it
//! changes) are what the recalc engine (P3.3, #535) walks to propagate a dirty
//! set, and what topological ordering and cycle detection run over.
//!
//! # What this layer is (and is not)
//!
//! This is the dependency *graph only*. It owns no values and performs no
//! evaluation: [`DependencyGraph::build`] reads a [`Workbook`] and produces the
//! edges; recalculation is P3.3. It exposes a [topological order /
//! cycle-detection primitive](DependencyGraph::topological_order) because P3.3
//! needs it, but it never evaluates a formula.
//!
//! # How edges are derived ([`extract_refs`])
//!
//! For each formula cell the graph parses the verbatim formula (parsing is
//! flavor-independent and does not consult the workbook's locked engine,
//! issue #900), calls [`extract_refs`] on the AST, and resolves
//! each [`Ref`] to a concrete graph node:
//!
//! - [`Ref::Cell`] → a single-cell precedent; a bare `A1` resolves against the
//!   formula cell's *own* sheet, a qualified `Sheet1!A1` against the named
//!   sheet.
//! - [`Ref::Range`] → a **range node** (range-node compression): `A1:A100000`
//!   is one node, not 100 000 edges. A changed cell finds its range-dependents
//!   by testing membership in each live range node, so the graph stays linear
//!   in the number of *distinct ranges*, not their area.
//! - [`Ref::Name`] → a **name node** (name → target indirection): the formula
//!   depends on the name, the name depends on its current target cell/range.
//!   Retargeting a name (P3.4) therefore dirties the name's dependents without
//!   rebuilding their edges, and a write inside a name's target range dirties
//!   the name's dependents transitively.
//!
//! A reference that cannot be resolved (an unknown sheet, an unknown name, a
//! malformed or unparseable formula) is recorded as an [`Unresolved`]
//! precedent rather than dropped: it carries no edge (nothing can dirty it),
//! but it is preserved so the recalc engine can surface the Sheets error the
//! formula will ultimately produce (`#REF!` / `#NAME?`), fixture-verified in
//! P3.3 rather than assumed here.
//!
//! [`extract_refs`]: truecalc_core::extract_refs
//! [`Ref`]: truecalc_core::Ref
//! [`Ref::Cell`]: truecalc_core::Ref::Cell
//! [`Ref::Range`]: truecalc_core::Ref::Range
//! [`Ref::Name`]: truecalc_core::Ref::Name
//! [`Unresolved`]: Precedent::Unresolved

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use icu_casemap::CaseMapperBorrowed;
use truecalc_core::{CellAddr, Ref};

use crate::address::Address;
use crate::casefold::simple_fold;
use crate::named_ref;
use crate::value::Value;
use crate::workbook::Workbook;

/// A fully resolved cell coordinate: a sheet (by its position-independent,
/// case-folded name) and an in-bounds [`Address`].
///
/// Sheets are keyed by **folded name**, not tab index, so the key survives a
/// sheet move (P3.1 `move_sheet`) and matches the case-insensitive sheet
/// lookup of [`Workbook::sheet`](crate::Workbook::sheet). A rename changes the
/// key, which is why a rename forces a graph rebuild (see the module docs and
/// [`DependencyGraph::build`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRef {
    /// The target sheet's name, simple-case-folded (schema spec §2).
    pub sheet: String,
    /// The in-bounds A1 address within that sheet.
    pub addr: Address,
}

impl CellRef {
    fn new(sheet: String, addr: Address) -> Self {
        Self { sheet, addr }
    }

    /// Builds the graph key for `addr` on the sheet named `sheet`, applying the
    /// simple case folding (schema spec §2) the graph indexes sheets by.
    ///
    /// Named `from_display_name` — not `resolve` — because it does not consult
    /// a [`Workbook`](crate::Workbook): it only folds the caller's spelling
    /// into the graph's key form, the same way [`new`](Self::new) builds a
    /// `CellRef` from an already-folded one. `resolve_ref` and
    /// `resolve_query_cell` elsewhere in this crate *do* look a sheet up
    /// against a workbook; this does not, so it does not share their name.
    ///
    /// The [`sheet`](Self::sheet) field is public but holds a *folded* name, so
    /// a `CellRef` constructed literally from a user-facing sheet name (a tab
    /// label, a JSON key, an API argument) silently matches nothing in the
    /// graph whenever that name is not already folded. Any sheet name that did
    /// not come out of the graph itself should reach a query through here.
    ///
    /// Folding is idempotent, so passing an already-folded name is a no-op.
    /// The sheet is *not* required to exist — an unknown sheet simply produces
    /// a key nothing in the graph matches.
    pub fn from_display_name(sheet: &str, addr: Address) -> Self {
        let folder = CaseMapperBorrowed::new();
        Self::new(simple_fold(&folder, sheet), addr)
    }
}

/// A resolved rectangular range: a sheet (folded name) and an inclusive,
/// top-left-first corner pair.
///
/// Range-node compression hinges on this being a single value regardless of
/// area: membership of a [`CellRef`] is an `O(1)` rectangle test
/// ([`RangeRef::contains`]), so finding the formula cells that read a changed
/// cell through a range costs one test per *distinct range*, never one per cell
/// in the range.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RangeRef {
    /// The target sheet's name, simple-case-folded.
    pub sheet: String,
    /// Top-left corner (minimum row, minimum column).
    pub start: Address,
    /// Bottom-right corner (maximum row, maximum column).
    pub end: Address,
}

impl RangeRef {
    /// Whether `cell` lies inside this range (same sheet, within the inclusive
    /// rectangle). The membership test behind range-node compression.
    pub fn contains(&self, cell: &CellRef) -> bool {
        cell.sheet == self.sheet
            && cell.addr.row >= self.start.row
            && cell.addr.row <= self.end.row
            && cell.addr.column >= self.start.column
            && cell.addr.column <= self.end.column
    }
}

/// One resolved precedent of a formula cell: what a single [`Ref`] in the
/// formula points at, after sheet/name resolution.
///
/// [`extract_refs`](truecalc_core::extract_refs) yields one [`Ref`] per
/// reference occurrence (duplicates preserved); the graph maps each to one of
/// these. [`Unresolved`](Precedent::Unresolved) keeps a reference that has no
/// concrete target (unknown sheet/name, unparseable formula) so the recalc
/// engine can still emit the right Sheets error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Precedent {
    /// A single cell (`A1` on the formula's own sheet, or `Sheet1!A1`).
    Cell(CellRef),
    /// A rectangular range (`A1:D4`, `Sheet1!A1:B2`) — a compressed range node.
    Range(RangeRef),
    /// A workbook-scoped named range, by its case-folded name. The name's
    /// current target (cell or range) supplies the transitive edges.
    Name(String),
    /// A reference that did not resolve to a concrete target: an unknown sheet
    /// or name, or a formula that failed to parse. Carries the canonical
    /// reference text for diagnostics; it produces no dirty-propagation edge.
    Unresolved(String),
}

/// The dependency graph of a [`Workbook`]: precedents and reverse-edge indexes
/// derived from every formula cell via [`extract_refs`](truecalc_core::extract_refs).
///
/// Built with [`DependencyGraph::build`]; queried with
/// [`precedents_of`](Self::precedents_of),
/// [`direct_dependents_of`](Self::direct_dependents_of),
/// [`topological_order`](Self::topological_order), and
/// [`cycle_cells`](Self::cycle_cells). A traversal that walks precedents
/// transitively also wants [`formula_precedent_cells`](Self::formula_precedent_cells)
/// (what to walk next) and [`name_target_of`](Self::name_target_of) (what a
/// name currently points at). It is a pure derived view — it borrows
/// nothing from the workbook after `build` returns and holds no values.
///
/// Rebuild rules (issue #534, "Rebuild rules on set/clear/rename"): the graph
/// is a function of the workbook's formulas, sheet names, and named-range
/// targets, so any edit that changes those — `set`/`clear` of a formula cell,
/// a sheet rename, a named-range retarget — invalidates it. The P3.4 mutation
/// API rebuilds (or incrementally updates) the graph after such edits; the
/// graph-rebuild equivalence tests assert that a from-scratch
/// [`build`](Self::build) after an arbitrary edit sequence equals the
/// maintained graph.
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyGraph {
    /// Every formula cell, with its resolved precedents in formula order
    /// (duplicates from `extract_refs` deduplicated per cell). The key set is
    /// exactly the set of graph nodes that carry a formula.
    precedents: BTreeMap<CellRef, Vec<Precedent>>,
    /// Reverse cell→formula edges: for a precedent *cell*, the formula cells
    /// that read it directly. The `O(1)` half of dependent lookup.
    cell_dependents: HashMap<CellRef, BTreeSet<CellRef>>,
    /// Reverse range edges: each distinct range node and the formula cells that
    /// read it. Range-node compression — one entry per range, tested by
    /// rectangle membership at query time.
    range_dependents: Vec<(RangeRef, BTreeSet<CellRef>)>,
    /// Name → its dependent formula cells (formulas that reference the name).
    name_dependents: HashMap<String, BTreeSet<CellRef>>,
    /// Name → its resolved current target (the indirection layer). Absent if
    /// the name is undefined or dangles; retargeting updates this entry.
    name_targets: HashMap<String, NameTarget>,
    /// Formula cells indexed by sheet and row: `sheet → row → the formula
    /// cells on that row, in column order`. This is what makes "which formula
    /// cells does this range cover?" cost the covered rows rather than the
    /// whole workbook (issue #908).
    ///
    /// Only rows that actually *contain* a formula cell are keyed — never the
    /// dense span of a range — so a reference over ten million mostly-empty
    /// rows visits only the handful of rows that hold formulas.
    formula_rows: HashMap<String, BTreeMap<u32, Vec<Address>>>,
}

/// What a named range currently resolves to (the name→target indirection).
///
/// Exactly the two shapes a resolved name target can take — unlike
/// [`Precedent`], it has no `Name` variant (names do not chain) and no
/// `Unresolved` variant (an unresolved name has no target at all, hence
/// [`name_target_of`](DependencyGraph::name_target_of) returning `None`
/// rather than this type wrapped around an absence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameTarget {
    Cell(CellRef),
    Range(RangeRef),
}

impl DependencyGraph {
    /// Builds the dependency graph from `workbook`.
    ///
    /// Walks every populated cell on every sheet; for each *formula* cell,
    /// parses the formula (flavor-independent, no engine needed — issue #900),
    /// extracts its refs
    /// ([`extract_refs`](truecalc_core::extract_refs)), resolves each to a
    /// concrete node, and records both the forward precedent list and the
    /// reverse edges. Named-range targets are resolved up front so name
    /// indirection edges are available.
    ///
    /// Resolution is total: an unresolvable reference becomes a
    /// [`Precedent::Unresolved`] rather than an error, so a workbook with a
    /// dangling `Sheet9!A1` or an unknown name still builds (the recalc engine
    /// turns those into Sheets errors, fixture-verified in P3.3). Building
    /// therefore never fails.
    pub fn build(workbook: &Workbook) -> Self {
        let folder = CaseMapperBorrowed::new();

        // Resolve named-range targets first (the name → target indirection
        // layer). A name whose ref names a missing sheet, or is itself
        // malformed, simply has no target and contributes no transitive edge.
        let mut name_targets: HashMap<String, NameTarget> = HashMap::new();
        for nr in workbook.names() {
            let folded = simple_fold(&folder, &nr.name);
            if let Some(target) = resolve_name_ref(&nr.r#ref, &folder, workbook) {
                name_targets.insert(folded, target);
            }
        }

        let mut graph = DependencyGraph {
            precedents: BTreeMap::new(),
            cell_dependents: HashMap::new(),
            range_dependents: Vec::new(),
            name_dependents: HashMap::new(),
            name_targets,
            formula_rows: HashMap::new(),
        };
        // Stable index from a range node to its slot in `range_dependents`, so
        // repeated references to the same range share one compressed node.
        let mut range_slots: HashMap<RangeRef, usize> = HashMap::new();

        for sheet in workbook.sheets() {
            let sheet_folded = simple_fold(&folder, sheet.name());
            for (addr, cell) in sheet.iter() {
                let Some(formula) = cell.formula() else {
                    continue;
                };
                let from = CellRef::new(sheet_folded.clone(), addr);

                // Parsed without an `Engine`: parsing is flavor-independent
                // and never reads the function registry, so building one per
                // graph build was pure waste (issue #900).
                let refs = match truecalc_core::parse_formula(formula) {
                    Ok(expr) => truecalc_core::extract_refs(&expr),
                    // An unparseable formula has one self-describing precedent
                    // and no edges — the recalc engine reports the parse error.
                    Err(_) => {
                        graph.precedents.insert(
                            from.clone(),
                            vec![Precedent::Unresolved(formula.to_owned())],
                        );
                        continue;
                    }
                };

                let mut seen: HashSet<Precedent> = HashSet::new();
                let mut resolved: Vec<Precedent> = Vec::new();
                for r in &refs {
                    let prec = resolve_ref(r, &from.sheet, from.addr, &folder, workbook);
                    if seen.insert(prec.clone()) {
                        resolved.push(prec);
                    }
                }

                // Record reverse edges for each resolved precedent.
                for prec in &resolved {
                    match prec {
                        Precedent::Cell(target) => {
                            graph
                                .cell_dependents
                                .entry(target.clone())
                                .or_default()
                                .insert(from.clone());
                        }
                        Precedent::Range(range) => {
                            let slot = *range_slots.entry(range.clone()).or_insert_with(|| {
                                graph
                                    .range_dependents
                                    .push((range.clone(), BTreeSet::new()));
                                graph.range_dependents.len() - 1
                            });
                            graph.range_dependents[slot].1.insert(from.clone());
                        }
                        Precedent::Name(name) => {
                            graph
                                .name_dependents
                                .entry(name.clone())
                                .or_default()
                                .insert(from.clone());
                        }
                        Precedent::Unresolved(_) => {}
                    }
                }

                graph.precedents.insert(from, resolved);
            }
        }

        // Index the formula cells by sheet and row (issue #908). Walking the
        // precedents map yields cells in canonical (sheet, row, column) order,
        // so each row's vector comes out in column order for free, and only
        // occupied rows become keys.
        for cell in graph.precedents.keys() {
            graph
                .formula_rows
                .entry(cell.sheet.clone())
                .or_default()
                .entry(cell.addr.row)
                .or_default()
                .push(cell.addr);
        }

        graph
    }

    /// The resolved precedents of `cell` in formula order, or `None` if `cell`
    /// is not a formula cell (a literal or an empty cell has no precedents).
    pub fn precedents_of(&self, cell: &CellRef) -> Option<&[Precedent]> {
        self.precedents.get(cell).map(Vec::as_slice)
    }

    /// Whether `cell` is a formula cell tracked by the graph.
    pub fn is_formula(&self, cell: &CellRef) -> bool {
        self.precedents.contains_key(cell)
    }

    /// Every formula cell in the graph, in canonical (sheet, address) order.
    pub fn formula_cells(&self) -> impl Iterator<Item = &CellRef> {
        self.precedents.keys()
    }

    /// The formula cells that read `cell` **directly** — through a single-cell
    /// reference, through a range that contains `cell`, or through a named
    /// range whose target contains `cell`.
    ///
    /// This is the dirty-propagation primitive the incremental recalc engine
    /// (P3.3) walks transitively: when `cell` changes, every cell returned here
    /// is dirty, and the walk repeats from each of them. It deliberately
    /// composes all three edge kinds so callers never reason about
    /// range-node compression or name indirection themselves.
    ///
    /// Returned in canonical (sheet, address) order; the set is deduplicated
    /// even when a formula reaches `cell` by more than one path.
    pub fn direct_dependents_of(&self, cell: &CellRef) -> BTreeSet<CellRef> {
        let mut out = BTreeSet::new();
        if let Some(direct) = self.cell_dependents.get(cell) {
            out.extend(direct.iter().cloned());
        }
        for (range, deps) in &self.range_dependents {
            if range.contains(cell) {
                out.extend(deps.iter().cloned());
            }
        }
        // Name indirection: a write inside a name's target dirties the name's
        // dependents.
        for (name, target) in &self.name_targets {
            let hit = match target {
                NameTarget::Cell(c) => c == cell,
                NameTarget::Range(r) => r.contains(cell),
            };
            if hit {
                if let Some(deps) = self.name_dependents.get(name) {
                    out.extend(deps.iter().cloned());
                }
            }
        }
        out
    }

    /// The formula cells that depend on the named range `name` (any case),
    /// i.e. would be dirtied by retargeting it (P3.4).
    ///
    /// Retargeting a name changes what its dependents read without changing
    /// their formulas, so the recalc engine dirties exactly this set (the
    /// name → target indirection promised by issue #534).
    pub fn name_dependents_of(&self, name: &str) -> BTreeSet<CellRef> {
        let folder = CaseMapperBorrowed::new();
        let folded = simple_fold(&folder, name);
        self.name_dependents
            .get(&folded)
            .cloned()
            .unwrap_or_default()
    }

    /// The current target of the named range `name` (any case): the cell or
    /// range its dependents actually read, or `None` when the name is not
    /// defined in this workbook or its reference does not resolve.
    ///
    /// The forward half of the name → target indirection whose reverse half is
    /// [`name_dependents_of`](Self::name_dependents_of). A caller walking a
    /// formula's precedents needs it to report what a [`Precedent::Name`]
    /// actually points at; [`NameTarget`] is the two-variant type for exactly
    /// that answer, so the signature itself rules out a name or an
    /// unresolved reference coming back — no doc caveat required.
    pub fn name_target_of(&self, name: &str) -> Option<NameTarget> {
        let folder = CaseMapperBorrowed::new();
        let folded = simple_fold(&folder, name);
        self.name_targets.get(&folded).cloned()
    }

    /// A topological order of the formula cells: every cell appears after all
    /// the formula cells it (transitively) reads, so evaluating in this order
    /// visits each cell only once with its precedents already current.
    ///
    /// Returns `Ok(order)` when the formula-cell subgraph is acyclic, or
    /// `Err(cycle_cells)` listing every formula cell that lies on a cycle (the
    /// set [`cycle_cells`](Self::cycle_cells) returns). Only edges *between
    /// formula cells* participate: a formula that reads a literal cell has
    /// nothing to wait for. This is the ordering primitive P3.3 evaluates in;
    /// the Sheets circular-dependency error semantics for the cells on a cycle
    /// are applied by the recalc engine (fixture-verified there), not here.
    pub fn topological_order(&self) -> Result<Vec<CellRef>, BTreeSet<CellRef>> {
        let edges = self.formula_edges();
        match edges.topological_order() {
            Some(order) => Ok(order),
            // Some cells never reached in-degree 0: they are on or downstream
            // of a cycle. Report exactly the cells *on* a cycle.
            None => Err(edges.cycle_members()),
        }
    }

    /// The evaluation order and the cycle set together, from **one** pass over
    /// the formula-cell edges.
    ///
    /// A recalculation needs both: the order to evaluate in, and the cells to
    /// mark with the circular-dependency error. Asking for them separately
    /// ([`cycle_cells`](Self::cycle_cells) then
    /// [`topological_order`](Self::topological_order)) derives the same
    /// formula-cell adjacency from the precedent lists twice and throws it away
    /// twice. This builds it once. Nothing is cached and nothing has to be
    /// invalidated; it is the same work, done once instead of twice.
    ///
    /// The order is [`topological_order`](Self::topological_order)'s when the
    /// graph is acyclic (and the cycle set is then empty), and
    /// [`acyclic_order_excluding`](Self::acyclic_order_excluding)'s over the
    /// acyclic remainder when it is not.
    pub fn evaluation_order(&self) -> (Vec<CellRef>, BTreeSet<CellRef>) {
        let edges = self.formula_edges();
        match edges.topological_order() {
            Some(order) => (order, BTreeSet::new()),
            None => {
                // The graph has a cycle. Order the acyclic remainder, so cells
                // that do not touch the cycle still evaluate in dependency
                // order; the rest take the circular error.
                let cycle = edges.cycle_members();
                let order = edges.order_excluding(&cycle);
                (order, cycle)
            }
        }
    }

    /// A topological order over the formula cells **not** on a cycle, for the
    /// cyclic-graph case (P3.3): cells that do not transitively read the cycle
    /// still evaluate in dependency order; cells on or downstream of the cycle
    /// are omitted (the recalc engine gives them the circular error). When the
    /// graph is acyclic this equals [`topological_order`](Self::topological_order).
    ///
    /// `cycle` must be the cycle set returned by
    /// [`cycle_cells`](Self::cycle_cells) (passed in so the caller computes it
    /// once). The order is deterministic (canonical tie-breaking), matching
    /// `topological_order`'s discipline.
    pub fn acyclic_order_excluding(&self, cycle: &BTreeSet<CellRef>) -> Vec<CellRef> {
        self.formula_edges().order_excluding(cycle)
    }

    /// Every formula cell that lies on a dependency cycle (a strongly connected
    /// component of size > 1, or a self-referential cell).
    ///
    /// This is the set P3.3 marks with the Sheets circular-dependency error.
    /// Empty iff the formula-cell subgraph is acyclic. Computed independently
    /// of [`topological_order`](Self::topological_order) so it can be queried
    /// directly.
    pub fn cycle_cells(&self) -> BTreeSet<CellRef> {
        self.formula_edges().cycle_members()
    }

    /// The formula-cell-only adjacency the ordering and cycle passes both run
    /// over: precedent formula cell → dependent formula cell, with the nodes in
    /// canonical order so every traversal over it is deterministic.
    ///
    /// Deriving it means expanding every precedent to the formula cells it
    /// covers, which is the expensive part of ordering a graph; the callers
    /// that need both an order and the cycle set take it once
    /// ([`evaluation_order`](Self::evaluation_order)) rather than twice.
    fn formula_edges(&self) -> FormulaEdges<'_> {
        let nodes: Vec<&CellRef> = self.precedents.keys().collect();
        let index_of: HashMap<&CellRef, usize> =
            nodes.iter().enumerate().map(|(i, n)| (*n, i)).collect();

        // `BTreeSet` keeps each node's successors in canonical order and dedups
        // parallel edges (a formula reading the same cell twice).
        let mut succ: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); nodes.len()];
        for (i, cell) in nodes.iter().enumerate() {
            for prec in &self.precedents[*cell] {
                for fp in self.formula_precedent_cells(prec) {
                    // Edge fp(j) → cell(i): cell i depends on fp.
                    if let Some(&j) = index_of.get(&fp) {
                        succ[j].insert(i);
                    }
                }
            }
        }

        FormulaEdges { nodes, succ }
    }

    /// Maps a precedent to the *formula* cells it covers (its intersection with
    /// the graph's formula-cell set), following name indirection. Literal and
    /// empty cells are not yielded — only edges between formula cells matter for
    /// ordering and cycles.
    ///
    /// This is the "what do I walk next" primitive of a precedent traversal: a
    /// [`Precedent::Cell`] yields that cell iff it carries a formula, a
    /// [`Precedent::Range`] yields the formula cells inside it (range-node
    /// compression is expanded only here, never in the stored edges), a
    /// [`Precedent::Name`] yields the formula cells its current target covers,
    /// and a [`Precedent::Unresolved`] yields nothing. Returned in canonical
    /// (sheet, address) order.
    ///
    /// Cost is `O(1)` for a cell precedent and, for a range or range-targeted
    /// name, `O(formula cells on the rows the range spans)` — the rows are
    /// indexed and only *occupied* rows are keyed, so neither the empty rows a
    /// tall reference spans nor the formula cells elsewhere in the workbook are
    /// visited (issue #908). Still an upper bound rather than a contract:
    /// callers should not rely on it staying this expensive or this cheap.
    pub fn formula_precedent_cells(&self, prec: &Precedent) -> Vec<CellRef> {
        self.formula_precedent_cells_examined(prec).0
    }

    /// [`formula_precedent_cells`](Self::formula_precedent_cells), plus how
    /// many candidate formula cells the lookup examined to produce it.
    ///
    /// Instrumentation, not a feature: the range index of issue #908 is a
    /// change in *how much is examined*, and wall-clock is too
    /// machine-dependent to pin it. Both values come out of the one lookup, so
    /// the count cannot drift from what the lookup actually does. Hidden from
    /// the docs because callers want
    /// [`formula_precedent_cells`](Self::formula_precedent_cells).
    #[doc(hidden)]
    pub fn formula_precedent_cells_examined(&self, prec: &Precedent) -> (Vec<CellRef>, usize) {
        match prec {
            Precedent::Cell(c) => {
                if self.precedents.contains_key(c) {
                    (vec![c.clone()], 1)
                } else {
                    (Vec::new(), 1)
                }
            }
            Precedent::Range(r) => self.formula_cells_in_range(r),
            Precedent::Name(name) => match self.name_targets.get(name) {
                Some(NameTarget::Cell(c)) if self.precedents.contains_key(c) => {
                    (vec![c.clone()], 1)
                }
                Some(NameTarget::Cell(_)) => (Vec::new(), 1),
                None => (Vec::new(), 0),
                Some(NameTarget::Range(r)) => self.formula_cells_in_range(r),
            },
            Precedent::Unresolved(_) => (Vec::new(), 0),
        }
    }

    /// The formula cells inside `range`, in canonical order, and the number of
    /// formula cells examined to find them.
    ///
    /// Walks only the *occupied* rows the range spans (`formula_rows` is keyed
    /// by row, so a `BTreeMap` range query skips every row that holds no
    /// formula), then filters those rows' cells by column. The cells examined
    /// are therefore the formula cells on the rows the range covers — not the
    /// formula cells of the whole workbook, which is what the scan this
    /// replaced cost (issue #908).
    fn formula_cells_in_range(&self, range: &RangeRef) -> (Vec<CellRef>, usize) {
        let mut out = Vec::new();
        let mut examined = 0usize;
        // Ranges resolved from formulas and names are top-left-first, but
        // `RangeRef`'s fields are public and this is reached from a public
        // entry point, so the corners can arrive the wrong way round.
        // [`RangeRef::contains`] answers "no" to every cell for such a range;
        // answer the same, rather than handing `BTreeMap::range` an inverted
        // bound (which panics).
        if range.start.row > range.end.row || range.start.column > range.end.column {
            return (out, examined);
        }
        let Some(rows) = self.formula_rows.get(&range.sheet) else {
            return (out, examined);
        };
        for addrs in rows.range(range.start.row..=range.end.row).map(|(_, a)| a) {
            examined += addrs.len();
            out.extend(
                addrs
                    .iter()
                    .filter(|a| a.column >= range.start.column && a.column <= range.end.column)
                    .map(|a| CellRef::new(range.sheet.clone(), *a)),
            );
        }
        (out, examined)
    }
}

/// The formula-cell-only adjacency of a [`DependencyGraph`]: the nodes in
/// canonical order and, for each, the formula cells that read it.
///
/// Built by [`DependencyGraph::formula_edges`] and consumed by the ordering and
/// cycle passes, which used to derive it independently from the precedent lists
/// — the same work, done twice per recalculation.
struct FormulaEdges<'a> {
    /// Every formula cell, in canonical (sheet, address) order. Indices into
    /// this vector are the node ids of `succ`.
    nodes: Vec<&'a CellRef>,
    /// `succ[j]` holds the nodes that read node `j` (edge `j → i` meaning
    /// "`i` depends on `j`"), deduplicated and in canonical order.
    succ: Vec<BTreeSet<usize>>,
}

impl FormulaEdges<'_> {
    /// Kahn's algorithm over the whole node set, or `None` if a cycle prevents
    /// every node from being placed.
    ///
    /// The ready set is a `BTreeSet` of indices, which — because `nodes` is in
    /// canonical order — pops in canonical order, so the order is itself
    /// deterministic.
    fn topological_order(&self) -> Option<Vec<CellRef>> {
        let mut indeg: Vec<usize> = vec![0; self.nodes.len()];
        for deps in &self.succ {
            for &i in deps {
                indeg[i] += 1;
            }
        }

        let mut ready: BTreeSet<usize> = (0..self.nodes.len()).filter(|&i| indeg[i] == 0).collect();
        let mut order: Vec<CellRef> = Vec::with_capacity(self.nodes.len());
        while let Some(&node) = ready.iter().next() {
            ready.remove(&node);
            order.push(self.nodes[node].clone());
            for &dep in &self.succ[node] {
                indeg[dep] -= 1;
                if indeg[dep] == 0 {
                    ready.insert(dep);
                }
            }
        }

        (order.len() == self.nodes.len()).then_some(order)
    }

    /// Kahn's algorithm over the nodes **not** in `cycle`, additionally
    /// dropping every node that reads a cycle node (it takes the circular
    /// error, and so does everything downstream of it).
    fn order_excluding(&self, cycle: &BTreeSet<CellRef>) -> Vec<CellRef> {
        // Surviving nodes, in canonical order; `slot` maps a node id to its
        // position among them (`None` for a cycle node).
        let kept: Vec<usize> = (0..self.nodes.len())
            .filter(|&i| !cycle.contains(self.nodes[i]))
            .collect();
        let mut slot: Vec<Option<usize>> = vec![None; self.nodes.len()];
        for (k, &i) in kept.iter().enumerate() {
            slot[i] = Some(k);
        }

        let mut indeg: Vec<usize> = vec![0; kept.len()];
        // A node that reads the cycle must be excluded from the order even
        // though it has in-degree 0 over the surviving edges.
        let mut tainted: Vec<bool> = vec![false; kept.len()];
        for (j, deps) in self.succ.iter().enumerate() {
            let from_cycle = slot[j].is_none();
            for &i in deps {
                let Some(k) = slot[i] else { continue };
                if from_cycle {
                    tainted[k] = true;
                } else {
                    indeg[k] += 1;
                }
            }
        }

        let mut ready: BTreeSet<usize> = (0..kept.len())
            .filter(|&k| indeg[k] == 0 && !tainted[k])
            .collect();
        let mut order: Vec<CellRef> = Vec::new();
        while let Some(&k) = ready.iter().next() {
            ready.remove(&k);
            order.push(self.nodes[kept[k]].clone());
            for &i in &self.succ[kept[k]] {
                let Some(dep) = slot[i] else { continue };
                indeg[dep] -= 1;
                if indeg[dep] == 0 && !tainted[dep] {
                    ready.insert(dep);
                }
            }
        }
        order
    }

    /// Every node on a cycle: Tarjan's SCC, keeping components of size > 1 and
    /// self loops.
    fn cycle_members(&self) -> BTreeSet<CellRef> {
        TarjanScc::new(&self.succ).cycle_members(&self.nodes)
    }
}

/// Resolves a single parsed [`Ref`] against the workbook, relative to the
/// formula's own (folded) sheet for bare references. `own_addr` is the
/// formula cell's own address (unfolded), needed only by [`Ref::Table`] to
/// infer which table an unqualified `[@column]` belongs to by containment —
/// the same role `recalc.rs`'s `GridResolver.current_cell` plays for real
/// value resolution.
fn resolve_ref(
    r: &Ref,
    own_sheet: &str,
    own_addr: Address,
    folder: &CaseMapperBorrowed<'static>,
    workbook: &Workbook,
) -> Precedent {
    match r {
        Ref::Cell { sheet, addr } => {
            let sheet_folded = match sheet {
                None => own_sheet.to_owned(),
                Some(name) => match workbook.sheet(name) {
                    Some(_) => simple_fold(folder, name),
                    // `relative_display` (not `to_string`) so a missing-sheet
                    // reference reached via `$A$1` dedupes with one reached
                    // via `A1` — `$` anchors don't change what's unresolved.
                    None => return Precedent::Unresolved(r.relative_display()),
                },
            };
            match to_address(addr) {
                Some(a) => Precedent::Cell(CellRef::new(sheet_folded, a)),
                None => Precedent::Unresolved(r.relative_display()),
            }
        }
        Ref::Range { sheet, start, end } => {
            let sheet_folded = match sheet {
                None => own_sheet.to_owned(),
                Some(name) => match workbook.sheet(name) {
                    Some(_) => simple_fold(folder, name),
                    None => return Precedent::Unresolved(r.relative_display()),
                },
            };
            match normalize_range(start, end) {
                Some((s, e)) => Precedent::Range(RangeRef {
                    sheet: sheet_folded,
                    start: s,
                    end: e,
                }),
                None => Precedent::Unresolved(r.relative_display()),
            }
        }
        Ref::Name(name) => {
            let folded = simple_fold(folder, name);
            // A name is a precedent only if the workbook actually defines it;
            // an unknown bare identifier is an unresolved reference (a `#NAME?`
            // in Sheets), not a phantom name node.
            if workbook
                .names()
                .iter()
                .any(|nr| simple_fold(folder, &nr.name) == folded)
            {
                Precedent::Name(folded)
            } else {
                Precedent::Unresolved(name.clone())
            }
        }
        Ref::Table {
            table,
            column,
            this_row,
        } => resolve_table_precedent(
            table.as_deref(),
            column,
            *this_row,
            own_sheet,
            own_addr,
            folder,
            workbook,
        ),
    }
}

/// Resolves a `Ref::Table` to its precedent.
///
/// Whole-column (`this_row: false`) precedents are a single
/// [`Precedent::Range`] over just the resolved column (header row through
/// last data row), not the whole table rectangle. The whole-table version
/// used to make an in-table formula that reads a *different* column of its
/// own table (e.g. `=[@qty]/SUM(T[qty])`, a common percentage-of-total
/// pattern) a precedent of itself, since its own cell always falls inside
/// the table's full rectangle — the same false-self-cycle class the
/// current-row branch below already guards against, just previously
/// uncaught for this branch (truecalc/core#861 final review). The header
/// row is kept in the range (not narrowed to just the data rows): editing a
/// header cell can change which column a name resolves to, so it should
/// still dirty dependents — conservative for dirtying purposes while
/// eliminating the false cross-column cycle. A formula in the *same* column
/// that reads its own column remains correctly circular, since its own cell
/// is still inside the narrowed range.
///
/// Current-row (`this_row: true`) precedents are a single [`Precedent::Cell`]
/// at `(own_addr.row, resolved column)` instead: using the whole-table range
/// here would make every in-table `[@col]` formula a *precedent of itself*
/// (its own cell is always inside the table rectangle it reads from), a false
/// self-cycle that would wrongly flag the extremely common "compute a column
/// from sibling columns in the same row" pattern (e.g. `=[@qty]*[@price]`) as
/// circular. A precise single-cell precedent matches exactly what
/// `recalc.rs`'s `GridResolver::resolve_table_ref` actually reads for
/// current-row, so it never over- or under-dirties.
///
/// An unqualified reference (`table: None`) infers the table from
/// `own_addr`'s containment within a table's *data* rows (excluding the
/// header row) on `own_sheet` — the same containment test
/// `recalc.rs`'s `GridResolver::resolve_table_ref` uses via its
/// `current_cell`, so an unqualified `[@column]` picks the same table here as
/// it does for real value resolution. A qualified reference looks the table
/// up by name directly.
fn resolve_table_precedent(
    table: Option<&str>,
    column: &str,
    this_row: bool,
    own_sheet: &str,
    own_addr: Address,
    folder: &CaseMapperBorrowed<'static>,
    workbook: &Workbook,
) -> Precedent {
    let target = match table {
        Some(name) => {
            let folded_name = simple_fold(folder, name);
            workbook
                .tables()
                .iter()
                .find(|t| simple_fold(folder, &t.name) == folded_name)
        }
        None => workbook.tables().iter().find(|t| {
            named_ref::parse_canonical_ref(&t.r#ref)
                .ok()
                .and_then(|parsed| crate::table_ref::parsed_range_bounds(&t.r#ref, &parsed))
                .is_some_and(|b| {
                    simple_fold(folder, &b.sheet) == own_sheet
                        && b.row_start < own_addr.row
                        && own_addr.row <= b.row_end
                        && b.col_start <= own_addr.column
                        && own_addr.column <= b.col_end
                })
        }),
    };
    let Some(t) = target else {
        return Precedent::Unresolved(format!(
            "{}[{}{}]",
            table.unwrap_or(""),
            if this_row { "@" } else { "" },
            column
        ));
    };
    let Ok(parsed) = named_ref::parse_canonical_ref(&t.r#ref) else {
        return Precedent::Unresolved(t.r#ref.clone());
    };
    let Some(bounds) = crate::table_ref::parsed_range_bounds(&t.r#ref, &parsed) else {
        return Precedent::Unresolved(t.r#ref.clone());
    };
    let sheet_folded = simple_fold(folder, &bounds.sheet);

    // Column-index-by-header lookup, same pattern as `recalc.rs`'s
    // `GridResolver::resolve_table_ref`: a column that isn't actually in the
    // table's header row produces no precedent (it's not a real dependency,
    // the formula will error at recalc time regardless of what changes).
    let column_folded = simple_fold(folder, column);
    let sheet = workbook.sheet(&bounds.sheet);
    let mut found = None;
    for c in bounds.col_start..=bounds.col_end {
        let Some(header_addr) = Address::new(bounds.row_start, c) else {
            continue;
        };
        if let Some(Value::Text(header)) = sheet.and_then(|s| s.get(header_addr)).map(|c| c.value())
        {
            if simple_fold(folder, header) == column_folded {
                found = Some(c);
                break;
            }
        }
    }
    let Some(col) = found else {
        return Precedent::Unresolved(t.r#ref.clone());
    };

    if this_row {
        // Precise single-cell precedent (see the function doc comment for
        // why the whole-table range would be wrong here): only valid if the
        // formula's own cell is actually inside this table's data rows.
        if own_sheet != sheet_folded
            || own_addr.row <= bounds.row_start
            || own_addr.row > bounds.row_end
        {
            return Precedent::Unresolved(t.r#ref.clone());
        }
        return match Address::new(own_addr.row, col) {
            Some(a) => Precedent::Cell(CellRef::new(sheet_folded, a)),
            None => Precedent::Unresolved(t.r#ref.clone()),
        };
    }

    Precedent::Range(RangeRef {
        sheet: sheet_folded,
        start: Address::new(bounds.row_start, col).unwrap(),
        end: Address::new(bounds.row_end, col).unwrap(),
    })
}

/// Resolves a named range's canonical `ref` string to its concrete target,
/// requiring the target sheet to exist. Returns `None` when the ref is
/// malformed or names a missing sheet (a dangling name has no target).
fn resolve_name_ref(
    r: &str,
    folder: &CaseMapperBorrowed<'static>,
    workbook: &Workbook,
) -> Option<NameTarget> {
    let parsed = named_ref::parse_canonical_ref(r).ok()?;
    // The ref's sheet must exist; key the target by its folded name.
    let sheet = workbook.sheet(&parsed.sheet)?;
    let sheet_folded = simple_fold(folder, sheet.name());

    // Recover the A1 part (parse_canonical_ref already validated it).
    let a1_part = r.rsplit_once('!').map(|(_, a)| a).unwrap_or(r);
    match a1_part.split_once(':') {
        None => {
            let addr = Address::from_a1(a1_part)?;
            Some(NameTarget::Cell(CellRef::new(sheet_folded, addr)))
        }
        Some((s, e)) => {
            let start = Address::from_a1(s)?;
            let end = Address::from_a1(e)?;
            Some(NameTarget::Range(RangeRef {
                sheet: sheet_folded,
                start,
                end,
            }))
        }
    }
}

/// Converts a core [`CellAddr`] (no upper bound of its own) to a workbook
/// [`Address`], enforcing the workbook's grid bounds. An out-of-bounds ref
/// (legal to *parse*, but off the grid) resolves to `None` → `Unresolved`.
fn to_address(addr: &CellAddr) -> Option<Address> {
    Address::new(addr.row, addr.col)
}

/// Normalizes a parsed range to top-left-first, in-bounds corners. Returns
/// `None` if either corner is off-grid.
fn normalize_range(start: &CellAddr, end: &CellAddr) -> Option<(Address, Address)> {
    let top = Address::new(start.row.min(end.row), start.col.min(end.col))?;
    let bottom = Address::new(start.row.max(end.row), start.col.max(end.col))?;
    Some((top, bottom))
}

/// Tarjan's strongly-connected-components, specialized to return the set of
/// nodes that lie on a cycle (SCCs of size > 1, plus self loops). Iterative to
/// avoid recursion blowup on deep dependency chains (the plan's ≥10k-deep
/// benchmark).
struct TarjanScc<'a> {
    adj: &'a [BTreeSet<usize>],
    index: Vec<Option<usize>>,
    lowlink: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    next_index: usize,
    components: Vec<Vec<usize>>,
}

impl<'a> TarjanScc<'a> {
    fn new(adj: &'a [BTreeSet<usize>]) -> Self {
        let n = adj.len();
        Self {
            adj,
            index: vec![None; n],
            lowlink: vec![0; n],
            on_stack: vec![false; n],
            stack: Vec::new(),
            next_index: 0,
            components: Vec::new(),
        }
    }

    fn cycle_members(mut self, nodes: &[&CellRef]) -> BTreeSet<CellRef> {
        for v in 0..self.adj.len() {
            if self.index[v].is_none() {
                self.strongconnect(v);
            }
        }
        let mut out = BTreeSet::new();
        for comp in &self.components {
            let on_cycle = comp.len() > 1
                // A singleton SCC is on a cycle only via a self loop.
                || (comp.len() == 1 && self.adj[comp[0]].contains(&comp[0]));
            if on_cycle {
                for &i in comp {
                    out.insert(nodes[i].clone());
                }
            }
        }
        out
    }

    fn strongconnect(&mut self, v: usize) {
        let mut call_stack: Vec<(usize, Vec<usize>)> =
            vec![(v, self.adj[v].iter().copied().collect())];
        self.index[v] = Some(self.next_index);
        self.lowlink[v] = self.next_index;
        self.next_index += 1;
        self.stack.push(v);
        self.on_stack[v] = true;

        while let Some((node, successors)) = call_stack.last_mut() {
            let node = *node;
            if let Some(w) = successors.pop() {
                if self.index[w].is_none() {
                    self.index[w] = Some(self.next_index);
                    self.lowlink[w] = self.next_index;
                    self.next_index += 1;
                    self.stack.push(w);
                    self.on_stack[w] = true;
                    call_stack.push((w, self.adj[w].iter().copied().collect()));
                } else if self.on_stack[w] {
                    self.lowlink[node] = self.lowlink[node].min(self.index[w].unwrap());
                }
            } else {
                // All successors processed: finalize this node.
                if self.lowlink[node] == self.index[node].unwrap() {
                    let mut component = Vec::new();
                    loop {
                        let w = self.stack.pop().unwrap();
                        self.on_stack[w] = false;
                        component.push(w);
                        if w == node {
                            break;
                        }
                    }
                    self.components.push(component);
                }
                call_stack.pop();
                if let Some((parent, _)) = call_stack.last() {
                    let parent = *parent;
                    self.lowlink[parent] = self.lowlink[parent].min(self.lowlink[node]);
                }
            }
        }
    }
}
