use std::collections::HashSet;
use std::sync::Arc;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use icu_casemap::CaseMapperBorrowed;

use truecalc_core::Engine;

use crate::canonical;
use crate::casefold::simple_fold;
use crate::engine::EngineFlavor;
use crate::error::WorkbookError;
use crate::graph_cache::{CachedGraph, GraphCache};
use crate::limits;
use crate::named_range::NamedRange;
use crate::named_ref;
use crate::strict_json;
use crate::table::Table;
use crate::validate;
use crate::worksheet::Worksheet;

/// The schema version this library writes (schema spec §10). A string, not
/// an integer: compared by exact match, never numerically.
pub const SCHEMA_VERSION: &str = "2";

/// An engine-locked spreadsheet workbook — a pure value object (no hidden
/// state, no callbacks). Schema spec §2.
///
/// All five *document* fields are always serialized, even when empty. Field
/// declaration order (`engine`, `names`, `sheets`, `tables`, `version`)
/// matches canonical (JCS) key order.
///
/// `graph_cache` is not part of the document: it is derived state the workbook
/// memoizes across recalculations (see the `graph_cache` module docs). It is
/// skipped by serde, ignored by `PartialEq`, and contributes nothing to
/// `Hash`, so the value object is exactly what it was before the cache
/// existed.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workbook {
    engine: EngineFlavor,
    names: Vec<NamedRange>,
    sheets: Vec<Worksheet>,
    #[serde(default)]
    tables: Vec<Table>,
    #[serde(deserialize_with = "de_version")]
    version: String,
    #[serde(skip)]
    graph_cache: GraphCache,
}

impl Workbook {
    /// Creates an empty workbook locked to `engine`.
    ///
    /// The engine flavor is required at creation and immutable for the
    /// workbook's lifetime (ADR 2026-04-27-engine-flavor-explicit-everywhere):
    /// there is no default and no setter.
    pub fn new(engine: EngineFlavor) -> Self {
        Self {
            engine,
            names: Vec::new(),
            sheets: Vec::new(),
            tables: Vec::new(),
            version: SCHEMA_VERSION.to_owned(),
            graph_cache: GraphCache::default(),
        }
    }

    /// The engine flavor every formula in this workbook targets.
    pub fn engine(&self) -> EngineFlavor {
        self.engine
    }

    /// The schema version of this workbook document.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The worksheets, in tab order (array position is tab position).
    pub fn sheets(&self) -> &[Worksheet] {
        &self.sheets
    }

    /// Mutable access to the worksheets.
    ///
    /// Invalidates the dependency-graph cache on the borrow: what the caller
    /// does with a `&mut Vec<Worksheet>` is unobservable from here, so the
    /// only sound assumption is that it changed the graph.
    pub fn sheets_mut(&mut self) -> &mut Vec<Worksheet> {
        self.graph_cache.invalidate();
        &mut self.sheets
    }

    /// The workbook-scoped named ranges.
    pub fn names(&self) -> &[NamedRange] {
        &self.names
    }

    /// Mutable access to the named ranges.
    ///
    /// Invalidates the dependency-graph cache on the borrow (see
    /// [`sheets_mut`](Self::sheets_mut)).
    pub fn names_mut(&mut self) -> &mut Vec<NamedRange> {
        self.graph_cache.invalidate();
        &mut self.names
    }

    /// The workbook-scoped table declarations.
    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    /// Mutable access to the table declarations.
    ///
    /// Invalidates the dependency-graph cache on the borrow (see
    /// [`sheets_mut`](Self::sheets_mut)).
    pub fn tables_mut(&mut self) -> &mut Vec<Table> {
        self.graph_cache.invalidate();
        &mut self.tables
    }

    /// The worksheet named `name` (case-insensitive, simple case folding per
    /// schema spec §2), or `None` if no sheet matches.
    pub fn sheet(&self, name: &str) -> Option<&Worksheet> {
        self.sheet_index(name).map(|i| &self.sheets[i])
    }

    /// Mutable access to the worksheet named `name` (case-insensitive).
    ///
    /// Invalidates the dependency-graph cache on the borrow (see
    /// [`sheets_mut`](Self::sheets_mut)).
    pub fn sheet_mut(&mut self, name: &str) -> Option<&mut Worksheet> {
        self.graph_cache.invalidate();
        match self.sheet_index(name) {
            Some(i) => Some(&mut self.sheets[i]),
            None => None,
        }
    }

    /// The tab position (0-based array index) of the sheet named `name`
    /// (case-insensitive per schema spec §2), or `None` if no sheet matches.
    pub fn sheet_index(&self, name: &str) -> Option<usize> {
        let folder = CaseMapperBorrowed::new();
        let target = simple_fold(&folder, name);
        self.sheets
            .iter()
            .position(|s| simple_fold(&folder, s.name()) == target)
    }

    /// Appends `sheet` after the last tab and returns its 0-based position.
    ///
    /// Errors if the name collides with an existing sheet under simple case
    /// folding (schema spec §2), is empty or too long (schema spec §3), or
    /// would exceed the per-workbook sheet cap (scope ADR Decision 5).
    pub fn add_sheet(&mut self, sheet: Worksheet) -> Result<usize, WorkbookError> {
        let pos = self.sheets.len();
        self.insert_sheet(pos, sheet)?;
        Ok(pos)
    }

    /// Inserts `sheet` at tab position `index`, shifting later tabs right.
    /// `index == sheets().len()` appends. Position semantics: array index is
    /// tab position (schema spec §2 — order is significant).
    ///
    /// Errors on a duplicate name (case-insensitive, §2), an empty/too-long
    /// name (§3), the sheet cap (Decision 5), or `index` out of `0..=len`.
    pub fn insert_sheet(&mut self, index: usize, sheet: Worksheet) -> Result<(), WorkbookError> {
        if self.sheets.len() >= limits::MAX_SHEETS {
            return Err(WorkbookError::SheetManagement(format!(
                "cannot add sheet: workbook already has {} sheets, the limit (scope ADR Decision 5)",
                limits::MAX_SHEETS
            )));
        }
        if index > self.sheets.len() {
            return Err(WorkbookError::SheetManagement(format!(
                "cannot insert sheet at position {index}: only {} tab slots exist",
                self.sheets.len() + 1
            )));
        }
        validate_sheet_name(sheet.name())?;
        if let Some(existing) = self.sheet(sheet.name()) {
            return Err(WorkbookError::SheetManagement(format!(
                "cannot add sheet {:?}: it collides with the existing sheet {:?} under simple \
                 case folding (schema spec §2)",
                sheet.name(),
                existing.name()
            )));
        }
        // A new sheet changes the sheet name set, which is one of the graph's
        // inputs: a formula that referenced this name resolved to `Unresolved`
        // before and resolves for real now.
        self.graph_cache.invalidate();
        self.sheets.insert(index, sheet);
        Ok(())
    }

    /// Removes and returns the sheet named `name` (case-insensitive),
    /// shifting later tabs left, or `None` if no sheet matches.
    ///
    /// A workbook-scoped named range or table may now dangle to the removed
    /// sheet. The dangling-ref invariant is re-checked at
    /// [`to_json`](Self::to_json) and [`from_json`](Self::from_json) (schema
    /// spec §7) — including, since issue #969, at `to_json`, which the earlier
    /// wording claimed but the code did not do. A workbook left holding a
    /// dangling `ref` therefore fails to **save**, rather than saving cleanly
    /// and failing at some later load.
    ///
    /// Removal deliberately does not tidy up for you. It returns
    /// `Option<Worksheet>` and so has no channel to report what it discarded,
    /// and dropping a name or table the caller still wants is a silent loss
    /// they cannot detect; refusing the save names the offending range
    /// instead. Drop the refs you no longer want with
    /// [`remove_name`](Self::remove_name) / [`remove_table`](Self::remove_table),
    /// or repoint them with [`redefine_name`](Self::redefine_name) /
    /// [`redefine_table`](Self::redefine_table).
    pub fn remove_sheet(&mut self, name: &str) -> Option<Worksheet> {
        let i = self.sheet_index(name)?;
        // Removes every formula node on that sheet, and turns every reference
        // to it into `Unresolved`.
        self.graph_cache.invalidate();
        Some(self.sheets.remove(i))
    }

    /// Renames the sheet currently named `from` (case-insensitive) to `to`,
    /// repointing everything in the document that named the old sheet.
    ///
    /// A rename is **holistic**: the workbook owns the dangling-ref invariant
    /// across it (issue #969). Three things move together —
    ///
    /// - the sheet's own name;
    /// - every [`NamedRange`] and [`Table`] `ref` whose sheet token resolves
    ///   to this sheet (case-insensitively, §2). The A1 part is untouched and
    ///   the new sheet token is re-emitted in canonical quoting, so a `ref`
    ///   that was canonical stays canonical (§7);
    /// - every formula that qualifies a cell/range reference with the old
    ///   name, rewritten via [`Engine::rename_sheet_refs`]: unqualified refs,
    ///   refs to other sheets, string literals, function names and defined
    ///   names are left alone. A formula that does not parse has no references
    ///   to rewrite and is left verbatim — formula text carries no document
    ///   invariant, and `from_json` does not validate it either.
    ///
    /// One asymmetry is worth knowing. Sheet *identity* in this crate is
    /// Unicode **simple case folding**, and the `ref` rewrite above uses it.
    /// `Engine::rename_sheet_refs` matches a formula's sheet qualifier with
    /// `str::to_uppercase()` instead (`truecalc-core` does not depend on
    /// `icu_casemap`). The two agree on every name whose characters case-map
    /// one-to-one, and disagree only where they do not — `ß` vs `SS`, or the
    /// Kelvin sign `K` (U+212A) vs ASCII `K`. A workbook holding two sheets
    /// that differ only in such a character can therefore see a formula
    /// qualifier rewritten that pointed at the *other* sheet, or left alone
    /// when it should have moved. Named-range and table refs are exact either
    /// way, so this cannot produce an unloadable document — only a wrong
    /// formula, and only for names built from those characters.
    ///
    /// A pure case change of the *same* sheet is allowed (it does not collide
    /// with itself) and repoints refs and formulas to the new casing. Errors
    /// if `from` does not exist, if `to` is empty/too long (§3), or if `to`
    /// collides with a *different* sheet (§2). Every error is decided before
    /// anything is written, so a rejected rename leaves the document exactly
    /// as it was.
    pub fn rename_sheet(&mut self, from: &str, to: &str) -> Result<(), WorkbookError> {
        let idx = self.sheet_index(from).ok_or_else(|| {
            WorkbookError::SheetManagement(format!("cannot rename: no sheet named {from:?}"))
        })?;
        validate_sheet_name(to)?;
        if let Some(other) = self.sheet_index(to) {
            if other != idx {
                return Err(WorkbookError::SheetManagement(format!(
                    "cannot rename sheet to {to:?}: it collides with another sheet under simple \
                     case folding (schema spec §2)"
                )));
            }
        }
        // Re-keys every node on the sheet and re-resolves every qualified
        // reference to the old and the new name.
        self.graph_cache.invalidate();

        let old = self.sheets[idx].name().to_owned();
        let folder = CaseMapperBorrowed::new();
        let old_folded = simple_fold(&folder, &old);
        let new_token = named_ref::quote_sheet_if_needed(to);
        for r in self
            .names
            .iter_mut()
            .map(|n| &mut n.r#ref)
            .chain(self.tables.iter_mut().map(|t| &mut t.r#ref))
        {
            // A `ref` that will not even split is one `from_json` rejects
            // outright; there is no sheet token to repoint, so leave it for
            // `to_json` to report rather than guessing at a rewrite.
            if let Ok((sheet, a1)) = named_ref::split_sheet_ref(r) {
                if simple_fold(&folder, &sheet) == old_folded {
                    *r = format!("{new_token}!{a1}");
                }
            }
        }

        // The one rewriter, not a second one: `truecalc-core` already ships
        // this transform and the wasm surface exposes it, so a private copy
        // here would be a second implementation to drift.
        let engine = match self.engine {
            EngineFlavor::Sheets => Engine::sheets(),
            EngineFlavor::Excel => Engine::excel(),
        };
        for sheet in &mut self.sheets {
            for cell in sheet.cells_mut().values_mut() {
                let Some(formula) = cell.formula() else {
                    continue;
                };
                // A sheet qualifier is spelled `Sheet!A1`, so a formula with no
                // `!` cannot hold one and needs no parse. A byte scan instead of
                // a parse is what keeps the common case — most cells do not
                // reference another sheet — off the rename's cost.
                if !formula.contains('!') {
                    continue;
                }
                if let Ok(rewritten) = engine.rename_sheet_refs(formula, &old, to) {
                    if rewritten != formula {
                        cell.set_formula(rewritten);
                    }
                }
            }
        }

        self.sheets[idx].set_name(to);
        Ok(())
    }

    /// Moves the sheet at tab position `from` to position `to`, shifting the
    /// sheets in between (schema spec §2 — array position is tab position).
    /// Errors if either index is out of `0..len`.
    pub fn move_sheet(&mut self, from: usize, to: usize) -> Result<(), WorkbookError> {
        let len = self.sheets.len();
        if from >= len || to >= len {
            return Err(WorkbookError::SheetManagement(format!(
                "cannot move sheet from {from} to {to}: valid tab positions are 0..{len}"
            )));
        }
        // Tab order is not a graph input by construction (the graph keys
        // sheets by folded name, never by index), but `DependencyGraph::build`
        // before and after a reorder does *not* compare equal: `range_dependents`
        // (`depgraph.rs`) is a `Vec` ordered by first encounter during the
        // `workbook.sheets()` walk, so tab order leaks into that field. What
        // actually makes a reorder safe to skip is that nothing recalculation
        // observes is sensitive to it: `evaluation_order` comes from a
        // `BTreeMap`, `formula_edges`'s successors are `BTreeSet`s, and
        // `direct_dependents_of` collects into a `BTreeSet` before returning -
        // every order-sensitive part of the graph gets set-ified before a
        // caller can see it. The cache is dropped anyway: a move is a rare,
        // human-scale operation, and "every sheet operation invalidates" is a
        // rule a future reader can apply without re-deriving this.
        self.graph_cache.invalidate();
        let sheet = self.sheets.remove(from);
        self.sheets.insert(to, sheet);
        Ok(())
    }

    /// The cached dependency graph and evaluation order, if the cache is warm.
    ///
    /// Warm means "equal to a build against the workbook as it is now" — see
    /// the `graph_cache` module docs for the invalidation contract that
    /// maintains it. `pub`, not `pub(crate)`, so a read-only, host-facing
    /// graph query that only has `&Workbook` to work with (the wasm
    /// `precedentsOf`/`dependentsOf` binding) can reuse a warm cache instead
    /// of building its own copy — the same constraint
    /// [`trace_cell`](Self::trace_cell) documents for itself: it can read a
    /// warm entry but, taking `&self`, cannot populate a cold one.
    pub fn cached_graph_entry(&self) -> Option<Arc<CachedGraph>> {
        self.graph_cache.get()
    }

    /// Records a freshly built graph as the cache entry.
    pub(crate) fn store_cached_graph(&mut self, entry: Arc<CachedGraph>) {
        self.graph_cache.store(entry);
    }

    /// Drops the cache entry. Always sound; the cost of a spurious call is one
    /// rebuild.
    pub(crate) fn invalidate_graph_cache(&mut self) {
        self.graph_cache.invalidate();
    }

    /// Releases the cached dependency graph, if one is held, reclaiming the
    /// ~545 B/cell (wasm32) / ~856 B/cell (native) it retains for every
    /// formula cell — see the `limits` module docs for the multi-workbook
    /// arithmetic this exists for.
    ///
    /// The workbook itself is unchanged: the next `recalc` / `recalc_incremental`
    /// / `explain` call simply rebuilds the graph, exactly as it would after a
    /// mutation the `graph_cache` module invalidates on (`graph_builds` ticks
    /// up by one).
    ///
    /// Named for what it releases, not for the mechanism, and kept apart from
    /// [`invalidate_graph_cache`](Self::invalidate_graph_cache) (`pub(crate)`)
    /// on purpose: that one is this crate's word for "a mutation made the
    /// entry stale, it must rebuild before next use" — an internal
    /// correctness call the workbook makes about itself. This is a different
    /// call: a still-*valid* cache the *host* chooses to give back for its
    /// memory. The dependency-graph cache is currently the only derived state
    /// a workbook holds, but the name says what a caller gets (memory back),
    /// not how, so a second cache could join it later without renaming this.
    pub fn drop_derived_state(&mut self) {
        self.invalidate_graph_cache();
    }

    /// Mutable access to the worksheets that does **not** invalidate the
    /// dependency-graph cache.
    ///
    /// Every caller must be a write the graph provably cannot see, and must
    /// say which clause of the `graph_cache` contract makes it so. Today that
    /// is exactly two: `Workbook::set`/`Workbook::clear` of a literal over a
    /// non-formula cell, and recalc's value write-back — both only while the
    /// workbook declares no tables, since a table header's stored text *is* a
    /// graph input. If you are not certain, use
    /// [`sheets_mut`](Self::sheets_mut).
    pub(crate) fn sheets_mut_untracked(&mut self) -> &mut Vec<Worksheet> {
        &mut self.sheets
    }

    /// How many dependency graphs this workbook has built.
    ///
    /// Instrumentation, not a feature: "graph builds per recalculation" is the
    /// exact-count metric behind the graph cache, and wall clock is too
    /// machine-dependent to assert on in a test. Hidden from the docs because
    /// no caller needs it.
    ///
    /// **Does not count a cold [`trace_cell`](Self::trace_cell)/`explain`.**
    /// `trace_cell` takes `&self` and so cannot call `store_cached_graph`
    /// (needs `&mut self`); its cold path builds a `DependencyGraph` locally
    /// and discards it without ever calling the `GraphCache` store that is
    /// the only place this counter increments. A cold `explain` on a
    /// workbook therefore leaves this at `0` (and
    /// [`graph_cache_is_warm`](Self::graph_cache_is_warm) at `false`) even
    /// though a graph was, in fact, built — do not write a test asserting
    /// "explain builds no graph" from this counter.
    #[doc(hidden)]
    pub fn graph_builds(&self) -> u64 {
        self.graph_cache.builds()
    }

    /// Whether the dependency-graph cache currently holds an entry.
    /// Instrumentation, same rationale as [`graph_builds`](Self::graph_builds).
    #[doc(hidden)]
    pub fn graph_cache_is_warm(&self) -> bool {
        self.graph_cache.is_warm()
    }

    /// Parses a workbook from JSON bytes, enforcing every document-level rule
    /// of the schema (schema spec §1–§10) and the resource limits of the scope
    /// ADR (Decision 5).
    ///
    /// Accepts any schema-valid JSON — pretty-printed, reordered keys, extra
    /// whitespace are all fine; only the *content* must be valid (schema spec
    /// §8: non-canonical-but-valid input is accepted, output is always
    /// canonical). Beyond the serde layer's checks (unknown fields, value
    /// encodings incl. NaN/Inf and `-0`, empty-literal, exact version match),
    /// this enforces the rules serde cannot express:
    ///
    /// - **§1** duplicate object keys are rejected; a UTF-8 BOM and invalid
    ///   UTF-8 are rejected at the byte boundary (hence `&[u8]`, not `&str`);
    /// - **§2/§3** sheet names are non-empty, ≤ 100 scalar values, and unique
    ///   under Unicode **simple** case folding;
    /// - **§3** cell keys match `^[A-Z]{1,3}[1-9][0-9]{0,7}$` and lie within
    ///   the address bounds;
    /// - **§5** spill rectangles are document-valid (no authored cell inside an
    ///   anchor's rectangle, no overlapping rectangles, none out of bounds);
    /// - **§7** named-range names and `ref`s are valid and canonical, names are
    ///   unique case-insensitively, and no `ref` dangles to a missing sheet;
    /// - **Decision 5** input size and all structural limits are enforced (the
    ///   input-size and cell-count caps on `wasm32` only — see the
    ///   [`limits`](crate::limits) module docs).
    pub fn from_json(bytes: &[u8]) -> Result<Self, WorkbookError> {
        if limits::exceeds_serialized_cap(bytes.len()) {
            return Err(WorkbookError::Validation(format!(
                "input is {} bytes, exceeding the {}-byte limit (scope ADR Decision 5)",
                bytes.len(),
                limits::MAX_SERIALIZED_BYTES
            )));
        }
        // §1: duplicate-key- and BOM-rejecting parse into a JSON tree.
        let tree = strict_json::parse_no_dup_keys(bytes).map_err(WorkbookError::Validation)?;
        // Document-level invariants serde cannot express (§2/§3/§5/§7, limits).
        validate::validate_document(&tree).map_err(WorkbookError::Validation)?;
        // Typed deserialization (unknown fields, value encodings, version,
        // empty-literal) — the serde layer of P2.2.
        serde_json::from_value(tree).map_err(|e| WorkbookError::Validation(e.to_string()))
    }

    /// Serializes the workbook to its canonical RFC 8785 (JCS) byte form
    /// (schema spec §8): one line, no insignificant whitespace, no trailing
    /// newline, object keys sorted by UTF-16 code units, ECMAScript number
    /// formatting, `names` sorted by `name`.
    ///
    /// Errors if a named range or table `ref` dangles to a sheet the workbook
    /// does not have — the §7 invariant [`remove_sheet`](Self::remove_sheet)
    /// and [`rename_sheet`](Self::rename_sheet) are the ways to break, checked
    /// here so a document that cannot be loaded cannot be written — if a value
    /// is non-finite (forbidden, schema spec §8.4), or if the canonical bytes
    /// exceed the 100 MiB cap — enforced on `wasm32` only, see the
    /// [`limits`](crate::limits) module docs (scope ADR Decision 5).
    pub fn to_json(&self) -> Result<String, WorkbookError> {
        // Saving must not produce a document that can never be opened: the §7
        // dangling-ref rule is the one document invariant a structural change
        // can leave broken, so it is re-checked here and not only on load.
        self.check_no_dangling_refs()?;
        // Serialize through the typed serde layer (which already emits the §6
        // value encodings and rejects NaN/Inf), then canonicalize the tree.
        let mut tree =
            serde_json::to_value(self).map_err(|e| WorkbookError::Validation(e.to_string()))?;
        sort_names_by_name(&mut tree);
        sort_tables_by_name(&mut tree);
        let canonical = canonical::to_canonical_string(&tree).map_err(WorkbookError::Validation)?;
        if limits::exceeds_serialized_cap(canonical.len()) {
            return Err(WorkbookError::Validation(format!(
                "canonical workbook is {} bytes, exceeding the {}-byte limit                  (scope ADR Decision 5)",
                canonical.len(),
                limits::MAX_SERIALIZED_BYTES
            )));
        }
        Ok(canonical)
    }

    /// The §7 dangling-sheet-ref invariant, checked against the in-memory
    /// document: every [`NamedRange`] and [`Table`] `ref` must name a sheet
    /// this workbook still has (case-insensitively, §2).
    ///
    /// Deliberately the same rule, applied with the same helper and reported
    /// with the same wording as [`from_json`](Self::from_json), so the two
    /// cannot drift — a save that succeeds is a load that will succeed for
    /// this rule. Deliberately *only* that rule: it costs `O(names + tables)`,
    /// both capped at 10 000, and never touches a cell. The remaining
    /// load-time rules (§2/§3 sheet names, §3 cell keys, §5 spill rectangles,
    /// §7 name shape and uniqueness, table overlap and header-row column
    /// names) stay load-only, because a mutation-API caller cannot break them:
    /// [`add_sheet`](Self::add_sheet), [`define_name`](Self::define_name) and
    /// their siblings check each one at the point of change. Reaching past the
    /// API — [`names_mut`](Self::names_mut), [`tables_mut`](Self::tables_mut),
    /// [`sheets_mut`](Self::sheets_mut) — can still build a document that
    /// saves and will not load; those hand out a raw `&mut` and promise
    /// nothing.
    ///
    /// A formula naming a missing sheet is **not** a violation. `from_json`
    /// does not check formula text at all: such a reference is legal and
    /// resolves to an error at recalculation, so `to_json` accepts it too.
    fn check_no_dangling_refs(&self) -> Result<(), WorkbookError> {
        if self.names.is_empty() && self.tables.is_empty() {
            return Ok(());
        }
        let folder = CaseMapperBorrowed::new();
        // A set, where `from_json`'s equivalent is a `Vec`: same membership
        // test, but this one runs on the save path against up to 20 000 refs,
        // and the linear scan made the sheet count (capped at 256) a factor in
        // it — ~10 ms at both caps, versus ~3 ms here.
        let folded_sheets: HashSet<String> = self
            .sheets
            .iter()
            .map(|s| simple_fold(&folder, s.name()))
            .collect();
        let exists = |sheet: &str| folded_sheets.contains(&simple_fold(&folder, sheet));

        for nr in &self.names {
            let (sheet, _) =
                named_ref::split_sheet_ref(&nr.r#ref).map_err(WorkbookError::Validation)?;
            if !exists(&sheet) {
                return Err(WorkbookError::Validation(format!(
                    "named range {:?} refers to sheet {sheet:?}, which does not exist \
                     (schema spec §7)",
                    nr.name
                )));
            }
        }
        for t in &self.tables {
            let (sheet, _) = named_ref::split_sheet_ref(&t.r#ref)
                .map_err(|e| WorkbookError::Validation(format!("table {:?}: {e}", t.name)))?;
            if !exists(&sheet) {
                return Err(WorkbookError::Validation(format!(
                    "table {:?} refers to sheet {sheet:?}, which does not exist \
                     (structured-references spec §4)",
                    t.name
                )));
            }
        }
        Ok(())
    }
}

/// Validates a sheet name for the mutation API: non-empty and ≤ 100 Unicode
/// scalar values (schema spec §3). Uniqueness is checked separately against the
/// existing sheet set; this is only the per-name shape check, mirroring the
/// rule [`Workbook::from_json`] applies to a deserialized document.
///
/// [`Workbook::from_json`]: crate::Workbook::from_json
fn validate_sheet_name(name: &str) -> Result<(), WorkbookError> {
    let len = name.chars().count();
    if len == 0 {
        return Err(WorkbookError::SheetManagement(
            "a worksheet name must be non-empty (schema spec §3)".to_owned(),
        ));
    }
    if len > limits::MAX_SHEET_NAME_LEN {
        return Err(WorkbookError::SheetManagement(format!(
            "worksheet name {name:?} has {len} scalar values, exceeding the limit of {} \
             (schema spec §3)",
            limits::MAX_SHEET_NAME_LEN
        )));
    }
    Ok(())
}

/// Domain ordering of schema spec §8.7: `names` is serialized sorted by `name`
/// in ascending UTF-16 code-unit order (matching JCS string ordering).
/// `sheets` keeps authored tab order (it is data, not a set) and is left
/// untouched.
fn sort_names_by_name(tree: &mut serde_json::Value) {
    if let Some(names) = tree.get_mut("names").and_then(|v| v.as_array_mut()) {
        names.sort_by(|a, b| {
            let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
            an.encode_utf16().cmp(bn.encode_utf16())
        });
    }
}

/// Domain ordering of schema spec §8.7 (extended by the structured-refs
/// design spec §4): `tables` is serialized sorted by `name`, same rule as
/// `names`.
fn sort_tables_by_name(tree: &mut serde_json::Value) {
    if let Some(tables) = tree.get_mut("tables").and_then(|v| v.as_array_mut()) {
        tables.sort_by(|a, b| {
            let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
            an.encode_utf16().cmp(bn.encode_utf16())
        });
    }
}

/// Reader rule of schema spec §10: accept every version this library
/// knows (`"1"`, `"2"`), reject unknown versions with a clear "upgrade" error.
/// Writer rule of schema spec §10: a loaded document always migrates to
/// [`SCHEMA_VERSION`] on load, so re-serializing it writes the newest version
/// (confirmed empirically: `de_version` only sees the raw field value, so
/// without this the in-memory `version` would keep whatever string was read).
fn de_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != "1" && version != SCHEMA_VERSION {
        return Err(D::Error::custom(format!(
            "unsupported schema version {version:?}: this version of \
             truecalc-workbook reads versions \"1\" and \"2\" (schema spec §10); \
             upgrade truecalc to load this workbook"
        )));
    }
    Ok(SCHEMA_VERSION.to_owned())
}
