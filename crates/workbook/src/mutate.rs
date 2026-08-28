//! Workbook-level mutation API (plan item 3.4): cell `set` / `get` / `clear`
//! and named-range CRUD, all preserving value-object semantics — no interior
//! threads, no I/O, no hidden callbacks (issue #536, scope ADR Accepted).
//!
//! These methods build on P3.1's grid primitives ([`Worksheet::get`] /
//! [`Worksheet::set`] / [`Worksheet::clear`]) and sheet management; they add the
//! *workbook-scoped* invariants P3.1 deliberately deferred to the mutation API:
//!
//! - **Eager limit enforcement.** The per-mutation caps of scope ADR Decision 5
//!   are checked at the point of mutation — the workbook cell count
//!   ([`MAX_CELLS_PER_WORKBOOK`](crate::limits::MAX_CELLS_PER_WORKBOOK)), a
//!   `text` value's length, an `array` value's element count, a formula's
//!   length, and the named-range count — so a mutation that would breach a cap
//!   fails immediately rather than at serialize time. (The cell-count cap is
//!   enforced on `wasm32` only; see the [`limits`](crate::limits) module
//!   docs.) The serialized **byte**
//!   cap is the lone exception: it stays a serialize-time check
//!   ([`to_json`](crate::Workbook::to_json)), since recomputing canonical byte
//!   length per edit is O(document) per mutation (Decision 5, `limits` docs).
//! - **No dangling named ranges.** Named-range CRUD validates the target sheet
//!   exists at definition time (schema spec §7), the mirror of `from_json`'s
//!   dangling-ref rejection. (Sheet *removal* can still orphan a name; that
//!   stays re-checked at the serialization boundary, as P3.1 documents on
//!   [`remove_sheet`](crate::Workbook::remove_sheet).)
//!
//! Formula cells are stored verbatim with a [`Value::Empty`] result — the
//! "not yet evaluated" state. Recalc (P3.3) is out of scope here; `set` only
//! validates a formula's *syntax* against the workbook's locked engine and
//! leaves the cell awaiting a later recalc.

use icu_casemap::CaseMapperBorrowed;

use crate::address::Address;
use crate::casefold::simple_fold;
use crate::cell::Cell;
use crate::error::WorkbookError;
use crate::limits;
use crate::named_range::NamedRange;
use crate::named_ref;
use crate::spill::spill_rect;
use crate::table::Table;
use crate::table_ref::{self, ParsedRangeBounds};
use crate::value::Value;
use crate::workbook::Workbook;

/// What a [`Workbook::set`] writes into a cell: a literal value or a formula.
///
/// This is the parsed shape the caller hands `set`; the surface layers (WASM,
/// MCP, REST) decide how to turn a user's raw string into one of these (a
/// leading `=` selects [`CellInput::Formula`], everything else is a literal).
/// Keeping the discriminator explicit here avoids guessing inside the value
/// object and keeps `set` engine-agnostic about *input* syntax while still
/// validating formula syntax against the locked engine.
#[derive(Debug, Clone, PartialEq)]
pub enum CellInput {
    /// A literal cell value (schema spec §4 — a `value`-only cell). Must not be
    /// [`Value::Empty`]: an empty literal is byte-indistinguishable from an
    /// absent cell, so it is rejected (clear the cell instead).
    Literal(Value),
    /// A formula, stored verbatim including the leading `=`. Its syntax is
    /// validated against the workbook's locked engine on `set`; its value is
    /// [`Value::Empty`] until the next recalc (P3.3).
    Formula(String),
}

/// The effective value at an address after spill resolution (schema spec §5),
/// returned by [`Workbook::resolved`].
///
/// For an authored cell, `anchor` is `None` and `value` is the cell's stored
/// value. For a *spilled* cell, `value` is the reconstructed array element and
/// `anchor` is the address of the spilling anchor on the same sheet (the
/// runtime `spilledFrom` marker of §5 — a derived view, never serialized).
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    /// The effective value at the queried address.
    pub value: Value,
    /// The spill anchor, if the queried cell is spilled; `None` for an authored
    /// cell.
    pub anchor: Option<Address>,
}

impl Workbook {
    /// Writes `input` at `addr` on the sheet named `sheet` (case-insensitive),
    /// returning the cell previously there (if any).
    ///
    /// A literal is stored as a `value`-only cell; a formula is stored verbatim
    /// (with its leading `=`) carrying a [`Value::Empty`] result until the next
    /// recalc — `set` validates only the formula's *syntax* against the
    /// workbook's locked engine, never evaluating it (recalc is P3.3).
    ///
    /// Enforces, eagerly, the per-mutation caps of scope ADR Decision 5: a
    /// formula's length, a `text`/`array` value's size, and — when the write
    /// introduces a *new* populated cell — the per-workbook cell count.
    ///
    /// Errors if the sheet does not exist, the input is an empty literal
    /// (schema spec §4 — clear instead), the formula is syntactically invalid
    /// for the locked engine, or any cap would be exceeded.
    ///
    /// Auto-expand-by-append (structured-references spec §4,
    /// truecalc/core#861): if the written address lands exactly one row
    /// below a table's current range, within that table's column span, the
    /// matching table's `ref` is extended by one row — unless doing so would
    /// overlap another table's range, in which case the expansion is
    /// silently skipped and the write still succeeds as an ordinary cell
    /// write. At most one table can match, since table ranges never overlap
    /// (§4) and this only looks at the row immediately adjacent to a table,
    /// not inside any range.
    pub fn set(
        &mut self,
        sheet: &str,
        addr: Address,
        input: CellInput,
    ) -> Result<Option<Cell>, WorkbookError> {
        // Build (and validate) the cell before touching the grid, so a rejected
        // input leaves the workbook untouched (value-object atomicity).
        let cell = match input {
            CellInput::Literal(value) => {
                if matches!(value, Value::Empty) {
                    return Err(WorkbookError::EmptyLiteral);
                }
                check_value_limits(&value)?;
                Cell::literal(value)?
            }
            CellInput::Formula(formula) => {
                check_formula_limit(&formula)?;
                self.validate_formula(&formula)?;
                // Unevaluated until recalc (P3.3): empty result, formula verbatim.
                Cell::with_formula(formula, Value::Empty)
            }
        };

        let idx = self.sheet_index(sheet).ok_or_else(|| {
            WorkbookError::Mutation(format!("cannot set cell: no sheet named {sheet:?}"))
        })?;

        // Eager workbook cell-count cap: only a *new* cell grows the count.
        // `+ 1` is the cell this call is about to add.
        let introduces_new_cell = !self.sheets()[idx].contains(addr);
        if introduces_new_cell && limits::exceeds_cell_cap(self.total_cells() + 1) {
            return Err(WorkbookError::Mutation(format!(
                "cannot set cell: workbook already holds {} populated cells, the limit \
                 (scope ADR Decision 5)",
                limits::MAX_CELLS_PER_WORKBOOK
            )));
        }

        // Dependency-graph cache (see the `graph_cache` module docs). The
        // graph is a function of the workbook's formula cells, sheet names,
        // name/table declarations, and — only when a table is declared — the
        // text stored in that table's header row. So this write is
        // structure-preserving exactly when it neither creates nor destroys a
        // formula node *and* no table exists whose header text it could be.
        //
        // "A literal write cannot change the graph" is therefore false in
        // general: writing text into a declared table's header cell moves what
        // `T[column]` resolves to. It is true only in a workbook with no
        // tables, which is the condition tested here.
        let writes_formula = cell.formula().is_some();
        let prev = self.sheets_mut_untracked()[idx].set(addr, cell);
        let replaced_formula = prev.as_ref().and_then(Cell::formula).is_some();
        if writes_formula || replaced_formula || !self.tables().is_empty() {
            self.invalidate_graph_cache();
        }
        // Auto-expansion retargets a table `ref`, which is a graph input; it
        // invalidates through `tables_mut` on the path that actually expands,
        // and is a no-op (so correctly non-invalidating) on the path that does
        // not.
        self.expand_table_on_append(idx, addr);
        Ok(prev)
    }

    /// The **authored** cell at `addr` on the sheet named `sheet`
    /// (case-insensitive), or `None` if no cell is authored there.
    ///
    /// This returns only authored cells — a literal or a formula physically
    /// present in `cells`. A *spilled* cell (one materialized by a spill anchor,
    /// schema spec §5) is **not** authored and has no [`Cell`] to borrow, so
    /// `get` returns `None` for it; use [`resolved`](Self::resolved) to read the
    /// effective value at any address (authored *or* spilled) and learn the
    /// spill anchor. Keeping `get` authored-only preserves the structural
    /// distinguishability rule of §5 (a cell is authored iff it has an entry).
    pub fn get(&self, sheet: &str, addr: Address) -> Option<&Cell> {
        self.sheet(sheet).and_then(|ws| ws.get(addr))
    }

    /// The **effective** value at `addr` on the sheet named `sheet`
    /// (case-insensitive), resolving through array spills (schema spec §5).
    ///
    /// Returns `None` only if `addr` is genuinely empty — neither authored nor
    /// covered by a spill. Otherwise the returned [`Resolved`] carries the
    /// value and, for a spilled cell, the `anchor` it spilled from (the
    /// runtime `spilledFrom` view of §5 — never serialized). For an authored
    /// cell `anchor` is `None`. A blocked-spill anchor is just an authored
    /// formula cell whose value is the blocked-spill error, so it resolves as
    /// an ordinary authored cell with no `anchor`.
    ///
    /// Resolution reads the **stored** grid (the last recalc's results): a
    /// spilling anchor stores its full array (§6), and this reconstructs the
    /// spilled element by the five-line rule of §5. It does not recalc.
    pub fn resolved(&self, sheet: &str, addr: Address) -> Option<Resolved> {
        let ws = self.sheet(sheet)?;
        // Authored cell wins (a spill never overlaps an authored cell — §5).
        if let Some(cell) = ws.get(addr) {
            return Some(Resolved {
                value: cell.value().clone(),
                anchor: None,
            });
        }
        // Otherwise look for an anchor whose stored array spills onto `addr`.
        for (anchor_addr, cell) in ws.iter() {
            let Value::Array(rows) = cell.value() else {
                continue;
            };
            let nrows = rows.len();
            let ncols = rows.first().map_or(0, Vec::len);
            let Some(rect) = spill_rect(anchor_addr, nrows, ncols) else {
                continue;
            };
            if anchor_addr == addr {
                continue; // the anchor is authored, handled above
            }
            if let Some((i, j)) = rect.offset_of(addr) {
                let value = rows[i][j].clone();
                return Some(Resolved {
                    value,
                    anchor: Some(anchor_addr),
                });
            }
        }
        None
    }

    /// The spill anchor that materializes `addr` on the sheet named `sheet`
    /// (case-insensitive), or `None` if `addr` is authored or empty (schema
    /// spec §5). Convenience over [`resolved`](Self::resolved) when only the
    /// anchor identity (the `spilledFrom` view) is needed.
    pub fn spill_anchor(&self, sheet: &str, addr: Address) -> Option<Address> {
        self.resolved(sheet, addr).and_then(|r| r.anchor)
    }

    /// Removes the cell at `addr` on the sheet named `sheet` (case-insensitive),
    /// returning it if present. Clearing is *removing the entry*, never writing
    /// an empty value (schema spec §4). Returns `None` if the sheet or cell is
    /// absent.
    pub fn clear(&mut self, sheet: &str, addr: Address) -> Option<Cell> {
        let idx = self.sheet_index(sheet)?;
        let prev = self.sheets_mut_untracked()[idx].clear(addr);
        // Same rule as `set`, minus the "writes a formula" half: removing a
        // literal from a table-free workbook removes no node, no edge, and no
        // header text the graph can see.
        let removed_formula = prev.as_ref().and_then(Cell::formula).is_some();
        if removed_formula || !self.tables().is_empty() {
            self.invalidate_graph_cache();
        }
        prev
    }

    /// The total number of populated cells across every sheet — the quantity
    /// the per-workbook cell cap (scope ADR Decision 5) bounds.
    pub fn total_cells(&self) -> usize {
        self.sheets().iter().map(|s| s.len()).sum()
    }

    /// Defines a new workbook-scoped named range `name` pointing at the
    /// canonical reference `r` (`Sheet!A1` / `Sheet!A1:B2`), returning the
    /// stored [`NamedRange`].
    ///
    /// Validates everything `from_json` checks for a name (schema spec §7): the
    /// name's shape, the `ref`'s canonical form, that the target sheet exists
    /// (no dangling ref), that the name does not already exist
    /// (case-insensitively) as either a named range or a table
    /// (structured-references spec §4), and that the named-range cap
    /// (Decision 5) is not exceeded. To replace an existing name use
    /// [`redefine_name`](Self::redefine_name).
    pub fn define_name(&mut self, name: &str, r: &str) -> Result<&NamedRange, WorkbookError> {
        self.validate_name_definition(name, r)?;
        if self.names().len() >= limits::MAX_NAMED_RANGES {
            return Err(WorkbookError::Mutation(format!(
                "cannot define named range: workbook already has {} named ranges, the limit \
                 (scope ADR Decision 5)",
                limits::MAX_NAMED_RANGES
            )));
        }
        if let Some(existing) = self.name_index(name) {
            return Err(WorkbookError::Mutation(format!(
                "cannot define named range {name:?}: it collides with the existing name {:?} \
                 under simple case folding (schema spec §7)",
                self.names()[existing].name
            )));
        }
        if let Some(existing) = self.table_index(name) {
            return Err(WorkbookError::Mutation(format!(
                "cannot define named range {name:?}: it collides with the existing table {:?} \
                 under simple case folding (structured-references spec §4)",
                self.tables()[existing].name
            )));
        }
        self.names_mut().push(NamedRange {
            name: name.to_owned(),
            r#ref: r.to_owned(),
        });
        // Borrow the freshly pushed entry (it is last in declaration order;
        // serialization re-sorts by name independently, §8.7).
        Ok(self.names().last().expect("just pushed a named range"))
    }

    /// Redefines the existing named range `name` (case-insensitive) to point at
    /// the canonical reference `r`, returning the updated [`NamedRange`]. The
    /// name's identity and original casing are preserved; only the `ref`
    /// changes.
    ///
    /// Validates the `ref` exactly as [`define_name`](Self::define_name) does,
    /// including that the target sheet exists (no dangling ref). Errors if no
    /// name currently matches `name` (case-insensitively) or if the `ref` is
    /// not a valid canonical reference to an existing sheet.
    pub fn redefine_name(&mut self, name: &str, r: &str) -> Result<&NamedRange, WorkbookError> {
        self.validate_name_definition(name, r)?;
        let idx = self.name_index(name).ok_or_else(|| {
            WorkbookError::Mutation(format!(
                "cannot redefine named range: no name {name:?} exists"
            ))
        })?;
        // The lookup is case-insensitive, so this preserves the existing name's
        // identity (including its original casing) and only swaps the `ref`.
        self.names_mut()[idx].r#ref = r.to_owned();
        Ok(&self.names()[idx])
    }

    /// Removes the named range `name` (case-insensitive), returning it if it
    /// existed, or `None` otherwise.
    pub fn remove_name(&mut self, name: &str) -> Option<NamedRange> {
        self.name_index(name).map(|i| self.names_mut().remove(i))
    }

    /// The named range called `name` (case-insensitive), or `None`. Listing is
    /// [`names`](crate::Workbook::names).
    pub fn name(&self, name: &str) -> Option<&NamedRange> {
        self.name_index(name).map(|i| &self.names()[i])
    }

    /// Declaration-order index of the named range `name` (case-insensitive,
    /// simple case folding per schema spec §2/§7), or `None`.
    fn name_index(&self, name: &str) -> Option<usize> {
        let folder = CaseMapperBorrowed::new();
        let target = simple_fold(&folder, name);
        self.names()
            .iter()
            .position(|n| simple_fold(&folder, &n.name) == target)
    }

    /// Defines a new workbook-scoped table `name` over the canonical range
    /// `r` (`Sheet!A1:B2` — a table `ref` is always a range, never the
    /// single-cell form), returning the stored [`Table`].
    ///
    /// Validates the name's shape, that `r` is a canonical range referencing
    /// an existing sheet (no dangling ref), that the name does not already
    /// collide with an existing table or named range (case-insensitively),
    /// that the range does not overlap an existing table's range, and the
    /// table count cap (Decision 5). Unlike [`Workbook::from_json`], this
    /// does **not** validate the header row's column names — a table may
    /// legitimately be defined ahead of its header cells being written
    /// (define the shape first, fill the headers in later). A table defined
    /// over a headerless or malformed-header region therefore succeeds here,
    /// but the workbook will fail to reload (`from_json`'s load-time
    /// validation *does* check header content) if serialized before real
    /// header text is written at the range's first row
    /// (structured-references spec §4). To replace an existing table's
    /// range use [`redefine_table`](Self::redefine_table).
    pub fn define_table(&mut self, name: &str, r: &str) -> Result<&Table, WorkbookError> {
        let bounds = self.validate_table_definition(name, r)?;
        if self.tables().len() >= limits::MAX_TABLES {
            return Err(WorkbookError::Mutation(format!(
                "cannot define table: workbook already has {} tables, the limit \
                 (scope ADR Decision 5)",
                limits::MAX_TABLES
            )));
        }
        if let Some(existing) = self.table_index(name) {
            return Err(WorkbookError::Mutation(format!(
                "cannot define table {name:?}: it collides with the existing table {:?} \
                 under simple case folding (structured-references spec §4)",
                self.tables()[existing].name
            )));
        }
        if let Some(existing) = self.name_index(name) {
            return Err(WorkbookError::Mutation(format!(
                "cannot define table {name:?}: it collides with the existing named range {:?} \
                 under simple case folding (structured-references spec §4)",
                self.names()[existing].name
            )));
        }
        if let Some(other) = self.overlapping_table(&bounds, None) {
            return Err(WorkbookError::Mutation(format!(
                "cannot define table {name:?}: its range overlaps the existing table {other:?} \
                 (structured-references spec §4)"
            )));
        }
        self.tables_mut().push(Table {
            name: name.to_owned(),
            r#ref: r.to_owned(),
        });
        // Borrow the freshly pushed entry (it is last in declaration order;
        // serialization re-sorts by name independently, §8.7).
        Ok(self.tables().last().expect("just pushed a table"))
    }

    /// Redefines the existing table `name` (case-insensitive) to point at
    /// the canonical range `r`, returning the updated [`Table`]. The name's
    /// identity and original casing are preserved; only the `ref` changes.
    ///
    /// Validates the `ref` exactly as [`define_table`](Self::define_table)
    /// does, including that the target sheet exists (no dangling ref) and
    /// that the new range does not overlap another table's range. Errors if
    /// no table currently matches `name` (case-insensitively) or if the
    /// `ref` is not a valid canonical range to an existing sheet.
    pub fn redefine_table(&mut self, name: &str, r: &str) -> Result<&Table, WorkbookError> {
        let bounds = self.validate_table_definition(name, r)?;
        let idx = self.table_index(name).ok_or_else(|| {
            WorkbookError::Mutation(format!("cannot redefine table: no table {name:?} exists"))
        })?;
        if let Some(other) = self.overlapping_table(&bounds, Some(idx)) {
            return Err(WorkbookError::Mutation(format!(
                "cannot redefine table {name:?}: its range overlaps the existing table {other:?} \
                 (structured-references spec §4)"
            )));
        }
        // The lookup is case-insensitive, so this preserves the existing name's
        // identity (including its original casing) and only swaps the `ref`.
        self.tables_mut()[idx].r#ref = r.to_owned();
        Ok(&self.tables()[idx])
    }

    /// Removes the table `name` (case-insensitive), returning it if it
    /// existed, or `None` otherwise.
    pub fn remove_table(&mut self, name: &str) -> Option<Table> {
        self.table_index(name).map(|i| self.tables_mut().remove(i))
    }

    /// The table called `name` (case-insensitive), or `None`. Listing is
    /// [`tables`](crate::Workbook::tables).
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.table_index(name).map(|i| &self.tables()[i])
    }

    /// Declaration-order index of the table `name` (case-insensitive, simple
    /// case folding per structured-references spec §4), or `None`.
    fn table_index(&self, name: &str) -> Option<usize> {
        let folder = CaseMapperBorrowed::new();
        let target = simple_fold(&folder, name);
        self.tables()
            .iter()
            .position(|t| simple_fold(&folder, &t.name) == target)
    }

    /// Shared name/`ref` validation for table define/redefine
    /// (structured-references spec §4): name shape, canonical `ref` in range
    /// form, and the target sheet's existence. Returns the parsed range
    /// bounds (sheet folded, for overlap comparison) on success. Does not
    /// check name uniqueness, range overlap, or the header row — those
    /// depend on whether the call is a define or a redefine and are handled
    /// by each caller (the header row is validated only at document-load
    /// time, since a table may be defined ahead of its header cells being
    /// written).
    fn validate_table_definition(
        &self,
        name: &str,
        r: &str,
    ) -> Result<ParsedRangeBounds, WorkbookError> {
        if !named_ref::is_valid_name(name) {
            return Err(WorkbookError::Mutation(format!(
                "table name {name:?} is invalid: it must match ^[A-Za-z_][A-Za-z0-9_]*$ and \
                 must not be an A1 address, an R1C1-style reference, or a boolean \
                 (structured-references spec §4)"
            )));
        }
        let parsed = named_ref::parse_canonical_ref(r).map_err(WorkbookError::Mutation)?;
        if self.sheet(&parsed.sheet).is_none() {
            return Err(WorkbookError::Mutation(format!(
                "table {name:?} refers to sheet {:?}, which does not exist \
                 (structured-references spec §4)",
                parsed.sheet
            )));
        }
        let mut bounds = table_ref::parsed_range_bounds(r, &parsed).ok_or_else(|| {
            WorkbookError::Mutation(format!(
                "table {name:?} has a malformed ref: a table ref must be a range \
                 (structured-references spec §4)"
            ))
        })?;
        let folder = CaseMapperBorrowed::new();
        bounds.sheet = simple_fold(&folder, &bounds.sheet);
        Ok(bounds)
    }

    /// The name of an existing table (other than the one at `exclude_idx`,
    /// if any) whose range overlaps `bounds` (sheet already folded), or
    /// `None`. `exclude_idx` lets `redefine_table` compare a table's new
    /// range against every *other* table without it overlapping itself.
    fn overlapping_table(
        &self,
        bounds: &ParsedRangeBounds,
        exclude_idx: Option<usize>,
    ) -> Option<String> {
        let folder = CaseMapperBorrowed::new();
        for (i, t) in self.tables().iter().enumerate() {
            if Some(i) == exclude_idx {
                continue;
            }
            // A stored table ref is normally already validated as a
            // canonical range by `define_table`/`redefine_table`, but
            // `tables_mut()` is public and lets a caller push an arbitrary
            // `Table` bypassing that validation — skip (never panic on) a
            // ref this function can't parse, matching
            // `expand_table_on_append`'s `let Ok(..) else { .. }` style.
            let Ok(parsed) = named_ref::parse_canonical_ref(&t.r#ref) else {
                continue;
            };
            let Some(mut other_bounds) = table_ref::parsed_range_bounds(&t.r#ref, &parsed) else {
                continue;
            };
            other_bounds.sheet = simple_fold(&folder, &other_bounds.sheet);
            if table_ref::ranges_overlap(bounds, &other_bounds) {
                return Some(t.name.clone());
            }
        }
        None
    }

    /// Auto-expand-by-append (structured-references spec §4,
    /// truecalc/core#861): if `addr` on the sheet at `sheet_idx` lands
    /// exactly one row below a table's current range, within that table's
    /// column span, extends that table's `ref` by one row — unless the
    /// expanded range would overlap another table's range, in which case
    /// the expansion is silently skipped and the table's `ref` is left
    /// unchanged (the cell write itself still succeeds; auto-expand is a
    /// best-effort convenience, not a hard requirement, matching how real
    /// Excel does not auto-expand a table into another table's cells). A
    /// no-op if no table matches. At most one table's range can be adjacent
    /// to `addr` to begin with, since table ranges never overlap (§4).
    fn expand_table_on_append(&mut self, sheet_idx: usize, addr: Address) {
        let folder = CaseMapperBorrowed::new();
        let sheet_name = simple_fold(&folder, self.sheets()[sheet_idx].name());
        let Some(idx) = self.tables().iter().position(|t| {
            let Ok(parsed) = named_ref::parse_canonical_ref(&t.r#ref) else {
                return false;
            };
            let Some(bounds) = table_ref::parsed_range_bounds(&t.r#ref, &parsed) else {
                return false;
            };
            simple_fold(&folder, &bounds.sheet) == sheet_name
                && bounds.row_end + 1 == addr.row
                && bounds.col_start <= addr.column
                && addr.column <= bounds.col_end
        }) else {
            return;
        };

        let t = &self.tables()[idx];
        // Already validated as canonical by `define_table`/`redefine_table`.
        let parsed = named_ref::parse_canonical_ref(&t.r#ref)
            .expect("stored table ref is already canonical");
        let bounds = table_ref::parsed_range_bounds(&t.r#ref, &parsed)
            .expect("stored table ref is already a validated range");

        // Before committing the expansion, check the *would-be-expanded*
        // rectangle against every other table's range, the same overlap
        // helper `define_table`/`redefine_table` use (Finding 1, final PR2
        // review: an unchecked expansion could silently create two
        // overlapping tables, producing a workbook that can't reload).
        let new_bounds = ParsedRangeBounds {
            sheet: simple_fold(&folder, &bounds.sheet),
            row_start: bounds.row_start,
            row_end: addr.row,
            col_start: bounds.col_start,
            col_end: bounds.col_end,
        };
        if self.overlapping_table(&new_bounds, Some(idx)).is_some() {
            return; // best-effort convenience only; leave the table as-is
        }

        let sheet_token = named_ref::quote_sheet_if_needed(&parsed.sheet);
        let start = Address::new(bounds.row_start, bounds.col_start)
            .expect("bounds were derived from an already-validated ref");
        let end = Address::new(addr.row, bounds.col_end)
            .expect("bounds were derived from an already-validated ref");
        self.tables_mut()[idx].r#ref = format!("{sheet_token}!{}:{}", start.to_a1(), end.to_a1());
    }

    /// Shared name/`ref` validation for define/redefine (schema spec §7):
    /// name shape, canonical `ref`, and the target sheet's existence. Does not
    /// check uniqueness or the count cap — those depend on whether the call is
    /// a define or a redefine and are handled by each caller.
    fn validate_name_definition(&self, name: &str, r: &str) -> Result<(), WorkbookError> {
        if !named_ref::is_valid_name(name) {
            return Err(WorkbookError::Mutation(format!(
                "named-range name {name:?} is invalid: it must match ^[A-Za-z_][A-Za-z0-9_]*$ and \
                 must not be an A1 address, an R1C1-style reference, or a boolean (schema spec §7)"
            )));
        }
        let parsed = named_ref::parse_canonical_ref(r).map_err(WorkbookError::Mutation)?;
        if self.sheet(&parsed.sheet).is_none() {
            return Err(WorkbookError::Mutation(format!(
                "named range {name:?} refers to sheet {:?}, which does not exist (schema spec §7)",
                parsed.sheet
            )));
        }
        Ok(())
    }

    /// Validates a formula's syntax (issue #536: "parsed with the workbook's
    /// locked engine"). Parses only — no evaluation — so an unevaluated formula
    /// cell is still guaranteed to hold syntactically valid text.
    ///
    /// Calls the parser directly rather than through an [`Engine`]: parsing is
    /// flavor-independent (`Engine::parse` ignores the flavor and forwards to
    /// this same entry point) and never reads the function registry, so
    /// building an engine here bought nothing and cost a full 518-function
    /// registry construction on **every formula cell written** — orders of
    /// magnitude more than the parse itself (issue #900).
    fn validate_formula(&self, formula: &str) -> Result<(), WorkbookError> {
        truecalc_core::parse_formula(formula)
            .map(|_| ())
            .map_err(|e| WorkbookError::Mutation(format!("formula {formula:?} is invalid: {e}")))
    }
}

/// Eager `text`/`array` size caps of scope ADR Decision 5, mirroring the
/// serialize-boundary checks in `validate.rs` for the mutation path.
fn check_value_limits(value: &Value) -> Result<(), WorkbookError> {
    match value {
        Value::Text(s) => {
            let len = s.chars().count();
            if len > limits::MAX_TEXT_LEN {
                return Err(WorkbookError::Mutation(format!(
                    "text value has {len} scalar values, exceeding the limit of {} \
                     (scope ADR Decision 5)",
                    limits::MAX_TEXT_LEN
                )));
            }
        }
        Value::Array(rows) => {
            let elems: usize = rows.iter().map(|r| r.len()).sum();
            if elems > limits::MAX_ARRAY_ELEMENTS {
                return Err(WorkbookError::Mutation(format!(
                    "array value has {elems} elements, exceeding the limit of {} \
                     (scope ADR Decision 5)",
                    limits::MAX_ARRAY_ELEMENTS
                )));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Eager formula-length cap of scope ADR Decision 5 (bytes), mirroring the
/// serialize-boundary check for the mutation path.
fn check_formula_limit(formula: &str) -> Result<(), WorkbookError> {
    if formula.len() > limits::MAX_FORMULA_LEN {
        return Err(WorkbookError::Mutation(format!(
            "formula is {} bytes, exceeding the limit of {} bytes (scope ADR Decision 5)",
            formula.len(),
            limits::MAX_FORMULA_LEN
        )));
    }
    Ok(())
}
