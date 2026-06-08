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

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use chrono::{NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use icu_casemap::CaseMapperBorrowed;
use truecalc_core::{Engine, EngineFlavor, ErrorKind, Ref, Resolver, Value as CoreValue};

use crate::address::Address;
use crate::casefold::simple_fold;
use crate::cell::Cell;
use crate::depgraph::{CellRef, DependencyGraph};
use crate::value::Value;
use crate::workbook::Workbook;

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

        self.recompute(&graph, ctx, dirty)
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

        // Cells on a cycle short-circuit to the circular error; the rest are
        // evaluated in topological order. `topological_order` returns the full
        // order when acyclic, else the cycle set; we always have the cycle set
        // available via `cycle_cells` for the tainted-downstream case.
        let cycle = graph.cycle_cells();
        let order = match graph.topological_order() {
            Ok(order) => order,
            Err(_) => {
                // The graph has a cycle. Build a best-effort order over the
                // acyclic remainder by stripping cycle nodes, so cells that do
                // not touch the cycle still evaluate; cycle-tainted cells fall
                // out as the error below.
                graph.acyclic_order_excluding(&cycle)
            }
        };

        // Evaluate in order; only cells in `to_eval` are (re)written, but we
        // walk the full order so a re-evaluated cell sees current precedents.
        let mut new_values: BTreeMap<CellRef, Value> = BTreeMap::new();
        for cell in &order {
            if cycle.contains(cell) {
                continue; // handled in the cycle pass below
            }
            if !to_eval.contains(cell) {
                continue;
            }
            let value = self.eval_formula_cell(cell, now_serial, &new_values, &cycle);
            new_values.insert(cell.clone(), value);
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

    /// Evaluates a single formula cell through a resolver that reads the *new*
    /// values computed so far this recalc, falling back to the stored grid for
    /// everything else.
    fn eval_formula_cell(
        &self,
        cell: &CellRef,
        now_serial: Option<f64>,
        new_values: &BTreeMap<CellRef, Value>,
        cycle: &BTreeSet<CellRef>,
    ) -> Value {
        let formula = match self.cell_at(cell).and_then(Cell::formula) {
            Some(f) => f.to_owned(),
            None => return Value::Empty,
        };
        let engine = match self.engine() {
            EngineFlavor::Sheets => Engine::sheets(),
            EngineFlavor::Excel => Engine::excel(),
        };
        let mut resolver = GridResolver {
            workbook: self,
            own_sheet: &cell.sheet,
            new_values,
            cycle,
        };
        let core = engine.evaluate_with_resolver_at(&formula, &mut resolver, now_serial);
        core_to_workbook(core)
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
    new_values: &'a BTreeMap<CellRef, Value>,
    cycle: &'a BTreeSet<CellRef>,
}

impl GridResolver<'_> {
    /// The current value of a resolved cell: this recalc's fresh value if it
    /// was already computed, else the stored grid value, else empty. A cell on
    /// a cycle resolves to the circular error (so a cell that *reads* a cycle
    /// inherits the taint).
    fn cell_value(&self, sheet_folded: &str, addr: Address) -> CoreValue {
        let key = CellRef {
            sheet: sheet_folded.to_owned(),
            addr,
        };
        if self.cycle.contains(&key) {
            return CoreValue::Error(ErrorKind::Ref);
        }
        if let Some(v) = self.new_values.get(&key) {
            return workbook_to_core(v);
        }
        let folder = CaseMapperBorrowed::new();
        match self
            .workbook
            .sheets()
            .iter()
            .find(|s| simple_fold(&folder, s.name()) == sheet_folded)
            .and_then(|s| s.get(addr))
        {
            Some(c) => workbook_to_core(c.value()),
            None => CoreValue::Empty,
        }
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
        }
    }
}

impl GridResolver<'_> {
    /// Materializes a range as a core `Value::Array` of its cells in row-major
    /// reading order — the flat shape the P1.3 [`Resolver`] contract specifies
    /// ("a range -> a Value::Array of the cells in reading order") and the shape
    /// core's aggregations (SUM/AVERAGE/COUNT/SUMIF) and shape functions
    /// consume. The own/target sheet has already been resolved.
    fn resolve_range(
        &self,
        sheet_folded: &str,
        start: &truecalc_core::CellAddr,
        end: &truecalc_core::CellAddr,
    ) -> CoreValue {
        let (r0, r1) = (start.row.min(end.row), start.row.max(end.row));
        let (c0, c1) = (start.col.min(end.col), start.col.max(end.col));
        let mut cells: Vec<CoreValue> = Vec::new();
        for r in r0..=r1 {
            for c in c0..=c1 {
                match Address::new(r, c) {
                    Some(a) => cells.push(self.cell_value(sheet_folded, a)),
                    None => cells.push(CoreValue::Error(ErrorKind::Ref)),
                }
            }
        }
        CoreValue::Array(cells)
    }

    /// Resolves a named range's canonical `ref` string (`Sheet!A1` /
    /// `Sheet!A1:B2`) the same way a literal reference resolves.
    fn resolve_name_ref(&mut self, r: &str) -> CoreValue {
        let engine = match self.workbook.engine() {
            EngineFlavor::Sheets => Engine::sheets(),
            EngineFlavor::Excel => Engine::excel(),
        };
        // The ref string parses as a one-reference formula; extract and resolve.
        let formula = format!("={r}");
        match engine.parse(&formula) {
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
        CoreValue::Empty => Value::Empty,
        CoreValue::Date(n) => Value::Date(n),
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
        Value::Error(code) => CoreValue::Error(error_kind_from_code(code)),
        Value::Empty => CoreValue::Empty,
        Value::Date(n) => CoreValue::Date(*n),
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

/// SplitMix64 finalizer — a fast, well-distributed integer mix.
fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
