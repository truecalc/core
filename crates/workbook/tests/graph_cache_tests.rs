//! The dependency-graph cache: what it reuses, what invalidates it, and that
//! reusing it can never change an answer.
//!
//! # The two things that must hold
//!
//! 1. **Warm ≡ fresh.** A recalculation against a warm cache must produce the
//!    byte-identical grid a brand-new engine produces for the same document.
//!    This is the property the cache exists to preserve and the one nothing
//!    else in the repo tests. `recalc_differential_tests` compares
//!    `recalc_incremental` against `recalc` on a `clone()` of the same
//!    workbook — and a clone inherits the cache entry, so a graph that is
//!    stale for one arm is stale for the other in exactly the same way and the
//!    comparison stays green. Measured: with sheet operations excluded from
//!    invalidation, that harness passes 20,000 seeds on each of its three
//!    shapes while the sweep below fails at its default 300.
//!
//!    The "fresh engine" arm here is a canonical-JSON round trip — the same
//!    thing a new process does when it loads the document — so it cannot share
//!    any derived state with the warm arm by construction.
//!
//! 2. **Exact build counts.** `Workbook::graph_builds` counts graphs actually
//!    built. Wall clock is machine-dependent; "how many graphs did this
//!    recalculation build?" is not, and it is the metric that tells a stale
//!    cache from a merely slow one. Every mutation class below pins its count.
//!
//! # The rule under test
//!
//! The graph is a function of the sheet name set, every formula cell's
//! `(sheet, address, formula)`, the named-range and table declarations, and —
//! only when a table is declared — the text stored in that table's header row.
//! So a literal write is structure-preserving *only* in a table-free workbook,
//! and `header_text_written_as_a_literal_moves_a_structured_reference` is the
//! test that says why the table clause is not optional.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use truecalc_workbook::{
    Address, Cell, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).expect("valid A1")
}

fn at(row: u32, col: u32) -> Address {
    Address::new(row, col).expect("in-bounds address")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn ctx_later() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000 + 86_400_000, "Etc/GMT", 0).expect("valid tz")
}

/// A workbook with the same document and **no** derived state — what a new
/// process holds after loading the same file. Deliberately built through the
/// serialization boundary rather than by clearing a cache field, so the arm
/// cannot accidentally share anything with the warm one.
fn fresh(wb: &Workbook) -> Workbook {
    let json = wb.to_json().expect("serializable");
    let out = match Workbook::from_json(json.as_bytes()) {
        Ok(loaded) => loaded,
        // An in-memory grid can be one a *load* legitimately rejects: authoring
        // a cell inside a live spill rectangle is an ordinary edit but not a
        // §5-valid document. Fall back to a clone whose cache is dropped
        // through the documented contract — `sheets_mut` invalidates on the
        // borrow — and let the assertion below catch it if it ever stops doing
        // so, rather than silently comparing a warm arm against itself.
        Err(_) => {
            let mut copy = wb.clone();
            let _ = copy.sheets_mut();
            copy
        }
    };
    assert!(
        !out.graph_cache_is_warm(),
        "a freshly loaded workbook must hold no graph"
    );
    out
}

fn hash_of(wb: &Workbook) -> u64 {
    let mut h = DefaultHasher::new();
    wb.hash(&mut h);
    h.finish()
}

/// Two sheets, a name, a small formula web. No tables — the shape the literal
/// write rule is permissive for.
fn small_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.add_sheet(Worksheet::new("T")).unwrap();
    wb.define_name("RATE", "S!A1").unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", addr("A2"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    wb.set("S", addr("B1"), CellInput::Formula("=A1+A2".into()))
        .unwrap();
    wb.set(
        "S",
        addr("B2"),
        CellInput::Formula("=SUM(A1:A2)*RATE".into()),
    )
    .unwrap();
    wb.set("T", addr("A1"), CellInput::Formula("=S!B1+S!B2".into()))
        .unwrap();
    wb
}

// ---------------------------------------------------------------------------
// Exact build counts: what the cache reuses
// ---------------------------------------------------------------------------

#[test]
fn a_first_recalc_builds_one_graph_and_repeats_build_none() {
    let mut wb = small_workbook();
    assert_eq!(wb.graph_builds(), 0);
    wb.recalc(&ctx());
    assert_eq!(wb.graph_builds(), 1, "the first recalc builds the graph");
    for _ in 0..20 {
        wb.recalc(&ctx());
    }
    assert_eq!(
        wb.graph_builds(),
        1,
        "recalculating an unchanged workbook must not rebuild the graph"
    );
}

#[test]
fn an_incremental_recalc_reuses_the_same_cached_graph_as_a_full_one() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
    wb.recalc(&ctx());
    assert_eq!(
        wb.graph_builds(),
        1,
        "the two recalc paths share one cache, not one each"
    );
}

#[test]
fn writing_a_literal_over_a_literal_keeps_the_graph() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(9.0)))
        .unwrap();
    assert!(wb.graph_cache_is_warm(), "a literal write adds no node");
    wb.recalc(&ctx());
    assert_eq!(wb.graph_builds(), 1);
}

#[test]
fn writing_a_literal_into_an_empty_cell_keeps_the_graph() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.set("S", addr("D9"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    assert!(
        wb.graph_cache_is_warm(),
        "a new literal cell is not a graph node"
    );
}

#[test]
fn clearing_a_literal_keeps_the_graph() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.clear("S", addr("A1"));
    assert!(wb.graph_cache_is_warm(), "a literal is not a graph node");
}

#[test]
fn writing_a_formula_rebuilds_the_graph() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.set("S", addr("B1"), CellInput::Formula("=A2*10".into()))
        .unwrap();
    assert!(!wb.graph_cache_is_warm());
    wb.recalc(&ctx());
    assert_eq!(wb.graph_builds(), 2);
}

#[test]
fn writing_a_literal_over_a_formula_rebuilds_the_graph() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.set("S", addr("B1"), CellInput::Literal(Value::Number(4.0)))
        .unwrap();
    assert!(
        !wb.graph_cache_is_warm(),
        "replacing a formula with a literal removes a node"
    );
}

#[test]
fn clearing_a_formula_rebuilds_the_graph() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.clear("S", addr("B1"));
    assert!(!wb.graph_cache_is_warm());
}

#[test]
fn every_name_operation_rebuilds_the_graph() {
    for op in ["define", "redefine", "remove"] {
        let mut wb = small_workbook();
        wb.recalc(&ctx());
        match op {
            "define" => {
                wb.define_name("OTHER", "S!A2").unwrap();
            }
            "redefine" => {
                wb.redefine_name("RATE", "S!A2").unwrap();
            }
            _ => {
                wb.remove_name("RATE").unwrap();
            }
        }
        assert!(!wb.graph_cache_is_warm(), "{op} name must invalidate");
    }
}

#[test]
fn every_sheet_operation_rebuilds_the_graph() {
    for op in ["add", "rename", "remove", "move"] {
        let mut wb = small_workbook();
        wb.recalc(&ctx());
        match op {
            "add" => {
                wb.add_sheet(Worksheet::new("U")).unwrap();
            }
            "rename" => {
                wb.rename_sheet("T", "T2").unwrap();
            }
            "remove" => {
                wb.remove_sheet("T").unwrap();
            }
            _ => {
                wb.move_sheet(0, 1).unwrap();
            }
        }
        assert!(!wb.graph_cache_is_warm(), "{op} sheet must invalidate");
    }
}

#[test]
fn every_table_operation_rebuilds_the_graph() {
    for op in ["define", "redefine", "remove"] {
        let mut wb = table_workbook();
        wb.recalc(&ctx());
        match op {
            "define" => {
                wb.define_table("Second", "T!A1:A2").unwrap();
            }
            "redefine" => {
                wb.redefine_table("TBL", "S!A1:B4").unwrap();
            }
            _ => {
                wb.remove_table("TBL").unwrap();
            }
        }
        assert!(!wb.graph_cache_is_warm(), "{op} table must invalidate");
    }
}

#[test]
fn every_mutable_accessor_invalidates_on_the_borrow() {
    // What a caller does with a `&mut` into the workbook's interior is
    // unobservable from inside, so handing one out has to be treated as a
    // structural change whether or not the caller makes one.
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    let _ = wb.sheets_mut();
    assert!(!wb.graph_cache_is_warm(), "sheets_mut");

    let mut wb = small_workbook();
    wb.recalc(&ctx());
    let _ = wb.sheet_mut("S");
    assert!(!wb.graph_cache_is_warm(), "sheet_mut");

    let mut wb = small_workbook();
    wb.recalc(&ctx());
    let _ = wb.names_mut();
    assert!(!wb.graph_cache_is_warm(), "names_mut");

    let mut wb = small_workbook();
    wb.recalc(&ctx());
    let _ = wb.tables_mut();
    assert!(!wb.graph_cache_is_warm(), "tables_mut");
}

#[test]
fn a_formula_written_through_sheet_mut_is_seen_by_the_next_recalc() {
    // The rebuild-equivalence suite and several others author formulas through
    // `sheet_mut`, bypassing `Workbook::set` entirely. If that path did not
    // invalidate, every one of them would recalc against a graph that predates
    // the formula.
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.sheet_mut("S")
        .unwrap()
        .set(addr("C1"), Cell::with_formula("=A1*100", Value::Empty));
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("S", addr("C1")).unwrap().value(),
        &Value::Number(200.0),
        "a formula authored through sheet_mut must be evaluated"
    );
}

// ---------------------------------------------------------------------------
// Tables: the case where a *value* is a graph input
// ---------------------------------------------------------------------------

/// A one-column table `TBL` over `S!A1:A3` with header text in `A1`, plus a
/// structured reference that reads it, plus a second candidate column so a
/// header rename has somewhere to move the reference to.
fn table_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.add_sheet(Worksheet::new("T")).unwrap();
    wb.set(
        "S",
        addr("A1"),
        CellInput::Literal(Value::Text("qty".into())),
    )
    .unwrap();
    wb.set(
        "S",
        addr("B1"),
        CellInput::Literal(Value::Text("price".into())),
    )
    .unwrap();
    wb.set("S", addr("A2"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr("A3"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", addr("B2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("S", addr("B3"), CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    wb.define_table("TBL", "S!A1:B3").unwrap();
    wb.set("T", addr("A1"), CellInput::Formula("=SUM(TBL[qty])".into()))
        .unwrap();
    wb
}

#[test]
fn header_text_written_as_a_literal_moves_a_structured_reference() {
    // The whole reason "a literal write cannot change the graph" is false. A
    // structured reference resolves its column by matching the header cell's
    // stored text, so swapping the two headers moves `TBL[qty]` from column A
    // to column B without any formula changing. A cache that treated this
    // literal write as structure-preserving would keep summing column A.
    let mut warm = table_workbook();
    warm.recalc(&ctx());
    assert_eq!(
        warm.get("T", addr("A1")).unwrap().value(),
        &Value::Number(3.0)
    );

    warm.set(
        "S",
        addr("A1"),
        CellInput::Literal(Value::Text("other".into())),
    )
    .unwrap();
    warm.set(
        "S",
        addr("B1"),
        CellInput::Literal(Value::Text("qty".into())),
    )
    .unwrap();
    assert!(
        !warm.graph_cache_is_warm(),
        "a literal write in a table-bearing workbook must invalidate"
    );

    let mut cold = fresh(&warm);
    warm.recalc(&ctx());
    cold.recalc(&ctx());
    assert_eq!(
        warm.to_json().unwrap(),
        cold.to_json().unwrap(),
        "the warm arm must follow the header text to column B"
    );
    assert_eq!(
        warm.get("T", addr("A1")).unwrap().value(),
        &Value::Number(30.0),
        "TBL[qty] now names column B"
    );
}

#[test]
fn a_recomputed_header_value_invalidates_the_graph() {
    // The same hazard reached through recalc's own value write-back rather than
    // a caller's `set` — and the reason that clause is not merely conservative.
    //
    // `A1` is a *volatile* header: its text flips with the recalc context, so it
    // changes with no mutation for the mutation API to notice. Column B carries
    // the same header text, so `TBL[qty]` names column A while `A1` reads "qty"
    // and column B while it does not. When the flip is written back and the
    // graph is kept, `D1`'s precedent stays on the column it *used* to read,
    // and the next incremental recalc — whose only dirty seed is the volatile
    // header itself — never reaches `D1`. The stale value then survives every
    // further recalculation, because nothing will ever dirty `D1` again.
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set(
        "S",
        addr("A1"),
        CellInput::Formula("=IF(YEAR(TODAY())>2023,\"qty\",\"zzz\")".into()),
    )
    .unwrap();
    wb.set(
        "S",
        addr("B1"),
        CellInput::Literal(Value::Text("qty".into())),
    )
    .unwrap();
    wb.set("S", addr("A2"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr("A3"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", addr("B2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("S", addr("B3"), CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    wb.define_table("TBL", "S!A1:B3").unwrap();
    wb.set("S", addr("D1"), CellInput::Formula("=SUM(TBL[qty])".into()))
        .unwrap();

    // 2020: the volatile header reads "zzz", so `TBL[qty]` is column B.
    let old = RecalcContext::new(1_600_000_000_000, "Etc/GMT", 0).unwrap();
    // 2026: it reads "qty", so `TBL[qty]` becomes column A.
    let new = RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).unwrap();

    wb.recalc(&old);
    wb.recalc(&old);
    assert_eq!(
        wb.get("S", addr("D1")).unwrap().value(),
        &Value::Number(30.0)
    );
    wb.recalc(&new);
    wb.recalc(&new);
    assert_eq!(
        wb.get("S", addr("D1")).unwrap().value(),
        &Value::Number(3.0)
    );

    // Back to 2020, incrementally and with nothing edited: the header flips
    // again, and `D1` must follow it back to column B.
    let mut cold = fresh(&wb);
    wb.recalc_incremental(&old, &[]);
    cold.recalc_incremental(&old, &[]);
    assert_eq!(
        wb.to_json().unwrap(),
        cold.to_json().unwrap(),
        "a recomputed header value must not leave a stale graph behind"
    );
    assert_eq!(
        wb.get("S", addr("D1")).unwrap().value(),
        &Value::Number(30.0),
        "TBL[qty] must follow the volatile header back to column B"
    );
}

// ---------------------------------------------------------------------------
// The value-object contract is untouched
// ---------------------------------------------------------------------------

#[test]
fn a_warm_cache_changes_nothing_a_caller_can_observe() {
    let mut warm = small_workbook();
    warm.recalc(&ctx());
    let cold = fresh(&warm);

    assert_eq!(warm, cold, "the cache must not participate in equality");
    assert_eq!(
        hash_of(&warm),
        hash_of(&cold),
        "the cache must not participate in Hash"
    );
    assert_eq!(
        warm.to_json().unwrap(),
        cold.to_json().unwrap(),
        "the cache must not be serialized"
    );
    assert!(warm.graph_cache_is_warm() && !cold.graph_cache_is_warm());
}

#[test]
fn a_clone_of_a_warm_workbook_stays_independent() {
    let mut original = small_workbook();
    original.recalc(&ctx());
    let mut copy = original.clone();

    // The clone shares the cached graph. Editing it must not let the original
    // see the edit, nor let the clone recalc against the pre-edit graph.
    copy.set("S", addr("C1"), CellInput::Formula("=A1*1000".into()))
        .unwrap();
    copy.recalc(&ctx());
    assert_eq!(
        copy.get("S", addr("C1")).unwrap().value(),
        &Value::Number(2000.0)
    );
    assert!(original.get("S", addr("C1")).is_none());

    // And the original's own cache is still valid for the original.
    let mut cold = fresh(&original);
    original.recalc(&ctx());
    cold.recalc(&ctx());
    assert_eq!(original.to_json().unwrap(), cold.to_json().unwrap());
}

// ---------------------------------------------------------------------------
// Randomized warm-vs-fresh differential
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5EED_1234_ABCD_9876)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn pick_str(&mut self, xs: &[&'static str]) -> &'static str {
        xs[self.below(xs.len())]
    }
}

/// Sheets that always exist. `X` is added and removed by the edit script, so a
/// reference to it flips between resolved and unresolved — the sheet-name-set
/// half of the graph's inputs. No name or table ever points at `X`, so removing
/// it never leaves a dangling declaration the JSON round trip would reject.
const BASE_SHEETS: [&str; 2] = ["S", "T"];
const ROWS: u32 = 6;
const COLS: u32 = 4;
/// Column A rows 1..=3 on `S` are the table's rectangle; the generator never
/// writes formulas there, so header text stays text and the document keeps
/// round-tripping.
const TABLE_REF: &str = "S!A1:B3";
const HEADERS: [&str; 3] = ["qty", "price", "cost"];
const NAME_REFS: [&str; 5] = ["S!C1", "S!C1:C4", "T!B2", "T!B1:C3", "S!D4"];

fn ref_text(rng: &mut Rng, own_sheet: &str) -> String {
    // Row 4 and below on `S`, and anywhere on `T`/`X`: never inside the table
    // rectangle, so a generated formula cannot collide with header text.
    let sheet = rng.pick_str(&["S", "T", "X"]);
    let row = if sheet == "S" {
        4 + rng.below(3) as u32
    } else {
        1 + rng.below(ROWS as usize) as u32
    };
    let a = Address::new(row, 1 + rng.below(COLS as usize) as u32).expect("in bounds");
    if sheet == own_sheet {
        a.to_a1()
    } else {
        format!("{sheet}!{}", a.to_a1())
    }
}

fn formula(rng: &mut Rng, own_sheet: &str) -> String {
    match rng.below(10) {
        0 => format!("={}+{}", ref_text(rng, own_sheet), ref_text(rng, own_sheet)),
        1 => format!("={}*2", ref_text(rng, own_sheet)),
        2 => format!("=SUM(T!A1:C3)+{}", ref_text(rng, own_sheet)),
        3 => "=SUM(TBL[qty])".to_owned(),
        4 => "=COUNT(TBL[price])".to_owned(),
        5 => "=SUM(TBL[cost])".to_owned(),
        6 => "=NAMEA+1".to_owned(),
        7 => "=SUM(NAMEA)".to_owned(),
        8 => "=TODAY()".to_owned(),
        _ => format!("={{1;2}}+{}", ref_text(rng, own_sheet)),
    }
}

/// A cell that is safe to author a formula or a random literal into: never
/// inside the table rectangle, and always on a sheet that currently exists -
/// `T` is renamed to `T2` and back by `apply_edit`'s rename arm, so `"T"`
/// alone is not always a live sheet name the way it is in `BASE_SHEETS`.
fn free_cell(rng: &mut Rng, wb: &Workbook) -> (&'static str, Address) {
    let sheet = if wb.sheet("T").is_some() {
        rng.pick_str(&BASE_SHEETS)
    } else {
        "S"
    };
    let row = if sheet == "S" {
        4 + rng.below(3) as u32
    } else {
        1 + rng.below(ROWS as usize) as u32
    };
    (
        sheet,
        Address::new(row, 1 + rng.below(COLS as usize) as u32).expect("in bounds"),
    )
}

/// The cells a canonical `Sheet!A1` / `Sheet!A1:B2` reference covers.
fn cells_of_ref(r: &str) -> Vec<(String, Address)> {
    let (sheet, a1) = r.split_once('!').expect("canonical ref carries a sheet");
    let (start, end) = match a1.split_once(':') {
        Some((s, e)) => (addr(s), addr(e)),
        None => {
            let a = addr(a1);
            (a, a)
        }
    };
    let mut out = Vec::new();
    for row in start.row..=end.row {
        for col in start.column..=end.column {
            out.push((sheet.to_owned(), at(row, col)));
        }
    }
    out
}

fn build_workbook(rng: &mut Rng) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in BASE_SHEETS {
        wb.add_sheet(Worksheet::new(s)).unwrap();
    }
    // Header row and data for the table (two columns wide).
    for (i, h) in HEADERS.iter().take(2).enumerate() {
        wb.set(
            "S",
            at(1, 1 + i as u32),
            CellInput::Literal(Value::Text((*h).to_owned())),
        )
        .unwrap();
    }
    for row in 2..=3u32 {
        for col in 1..=2u32 {
            wb.set(
                "S",
                at(row, col),
                CellInput::Literal(Value::Number(rng.below(9) as f64)),
            )
            .unwrap();
        }
    }
    wb.define_table("TBL", TABLE_REF).unwrap();
    wb.define_name("NAMEA", NAME_REFS[rng.below(NAME_REFS.len())])
        .unwrap();

    for sheet in BASE_SHEETS {
        let first_row = if sheet == "S" { 4 } else { 1 };
        for row in first_row..=ROWS {
            for col in 1..=COLS {
                match rng.below(8) {
                    0 | 1 => {}
                    2 | 3 => {
                        wb.set(
                            sheet,
                            at(row, col),
                            CellInput::Literal(Value::Number(rng.below(9) as f64)),
                        )
                        .unwrap();
                    }
                    _ => {
                        let f = formula(rng, sheet);
                        wb.set(sheet, at(row, col), CellInput::Formula(f.clone()))
                            .unwrap_or_else(|e| panic!("generated formula {f} rejected: {e:?}"));
                    }
                }
            }
        }
    }
    // A deterministic sentinel for `apply_edit`'s add-sheet arm (case 8):
    // every random reference to `X!<cell>` this generator can otherwise
    // produce propagates an uncaught `#REF!` while `X` doesn't exist, and
    // `#REF!` is also `spill::BLOCKED_SPILL_ERROR` - so `seed_spill_sensitive`
    // (recalc.rs) conservatively reseeds every such cell on *every*
    // incremental recalc regardless of the dependency-graph edge, which masks
    // a missing `insert_sheet` invalidation no matter how the edited set is
    // built. `IFERROR` catches the error into a plain number instead, so this
    // cell is never in that reseed net, and a missing invalidation is only
    // caught here (`S!D6`, the last cell of `S`'s randomly-filled range,
    // overwritten deterministically so every seed starts with it).
    wb.set(
        "S",
        at(ROWS, COLS),
        CellInput::Formula("=IFERROR(X!A1,42)".to_owned()),
    )
    .unwrap();
    wb
}

/// One random edit, returning the cells to report as `edited`.
fn apply_edit(rng: &mut Rng, wb: &mut Workbook) -> Vec<(String, Address)> {
    match rng.below(13) {
        0 | 1 | 2 => {
            let (sheet, a) = free_cell(rng, wb);
            wb.set(
                sheet,
                a,
                CellInput::Literal(Value::Number(rng.below(20) as f64)),
            )
            .unwrap();
            vec![(sheet.to_owned(), a)]
        }
        3 | 4 => {
            let (sheet, a) = free_cell(rng, wb);
            let f = formula(rng, sheet);
            wb.set(sheet, a, CellInput::Formula(f.clone()))
                .unwrap_or_else(|e| panic!("generated formula {f} rejected: {e:?}"));
            vec![(sheet.to_owned(), a)]
        }
        5 => {
            let (sheet, a) = free_cell(rng, wb);
            wb.clear(sheet, a);
            vec![(sheet.to_owned(), a)]
        }
        6 => {
            // Rotate the table's header texts. Every `TBL[col]` reference in
            // the workbook moves to a different column — or stops resolving —
            // without a single formula changing, and nothing but a literal
            // write happened. Headers stay distinct and non-empty, so the
            // document keeps loading.
            let a = at(1, 1);
            let b = at(1, 2);
            let ha = wb.get("S", a).and_then(|c| match c.value() {
                Value::Text(t) => Some(t.clone()),
                _ => None,
            });
            let hb = wb.get("S", b).and_then(|c| match c.value() {
                Value::Text(t) => Some(t.clone()),
                _ => None,
            });
            let (ha, hb) = (
                ha.unwrap_or_else(|| HEADERS[0].to_owned()),
                hb.unwrap_or_else(|| HEADERS[1].to_owned()),
            );
            // Rotate through the three names so a column can also come to hold
            // a header no formula references at all.
            let next = |h: &str| -> String {
                let i = HEADERS.iter().position(|x| *x == h).unwrap_or(0);
                HEADERS[(i + 1) % HEADERS.len()].to_owned()
            };
            let (na, nb) = if next(&ha) == hb {
                (hb.clone(), ha.clone())
            } else {
                (next(&ha), hb.clone())
            };
            wb.set("S", a, CellInput::Literal(Value::Text(na))).unwrap();
            wb.set("S", b, CellInput::Literal(Value::Text(nb))).unwrap();
            vec![("S".to_owned(), a), ("S".to_owned(), b)]
        }
        7 => {
            let old_ref = wb.name("NAMEA").map(|n| n.r#ref.clone()).expect("defined");
            // `redefine_name` validates that the ref's sheet exists (schema
            // spec §7), unlike a dangling ref left behind by *removing* a
            // sheet - so a `T!...` candidate is only offered while `T` is
            // currently live (the rename arm can have it as `T2` instead).
            let candidates: Vec<&str> = NAME_REFS
                .iter()
                .copied()
                .filter(|r| wb.sheet("T").is_some() || !r.starts_with("T!"))
                .collect();
            let new_ref = candidates[rng.below(candidates.len())];
            wb.redefine_name("NAMEA", new_ref).unwrap();
            // A retarget's caller contract is to report the name's old *and*
            // new target cells (same as `recalc_differential_tests`).
            let mut edited = cells_of_ref(&old_ref);
            edited.extend(cells_of_ref(new_ref));
            edited
        }
        8 => {
            // No trailing write after `add_sheet`: any further `wb.set` -
            // formula or literal, table declared or not - invalidates the
            // cache on its own in this fixture, which would mask a missing
            // invalidation in `insert_sheet` instead of exercising it.
            //
            // Report every cell `ref_text` can address on `X` (its whole
            // `ROWS` x `COLS` surface), not just one: an existing formula
            // elsewhere may reference any of them, and only its *exact*
            // address gets its dependents seeded by `recalc_incremental`'s
            // per-edited-cell frontier - reporting one arbitrary cell would
            // catch a missing invalidation only on the seeds where some
            // formula happens to reference that one cell.
            let adding = wb.sheet("X").is_none();
            if adding {
                wb.add_sheet(Worksheet::new("X")).unwrap();
            } else {
                wb.remove_sheet("X");
            }
            let mut edited = Vec::new();
            for row in 1..=ROWS {
                for col in 1..=COLS {
                    edited.push(("X".to_owned(), at(row, col)));
                }
            }
            edited
        }
        9 => {
            // Author a formula the way half the suite does: straight through
            // the sheet, bypassing `Workbook::set`.
            let (sheet, a) = free_cell(rng, wb);
            let f = formula(rng, sheet);
            wb.sheet_mut(sheet)
                .unwrap()
                .set(a, Cell::with_formula(f, Value::Empty));
            vec![(sheet.to_owned(), a)]
        }
        10 => {
            wb.redefine_table("TBL", TABLE_REF).unwrap();
            vec![("S".to_owned(), at(1, 1))]
        }
        11 => {
            // Rename `T` back and forth. `formula`'s case 2 always emits
            // `SUM(T!A1:C3)` regardless of `own_sheet`, so `T` reliably has
            // dependents on every sheet - a missing invalidation here (unlike
            // the `add_sheet` arm above) is not masked by anything else in
            // this function.
            //
            // Report every cell on the *old* name's `ROWS` x `COLS` surface,
            // not the new one: `T!A1:C3`'s formula text never changes on a
            // rename, so it stays keyed to `from` both before and after -
            // reporting under `to` would query a key nothing in this
            // generator ever references (nothing here ever writes a `T2!...`
            // formula).
            let (from, to) = if wb.sheet("T").is_some() {
                ("T", "T2")
            } else {
                ("T2", "T")
            };
            wb.rename_sheet(from, to).unwrap();
            let mut edited = Vec::new();
            for row in 1..=ROWS {
                for col in 1..=COLS {
                    edited.push((from.to_owned(), at(row, col)));
                }
            }
            edited
        }
        _ => {
            let (sheet, a) = free_cell(rng, wb);
            wb.set(
                sheet,
                a,
                CellInput::Literal(Value::Text(format!("t{}", rng.below(5)))),
            )
            .unwrap();
            vec![(sheet.to_owned(), a)]
        }
    }
}

/// One seed: build, then edit-and-recalculate repeatedly, comparing the warm
/// workbook against a freshly loaded copy of the same document after every
/// step.
fn run_seed(seed: u64) -> Result<(), String> {
    let mut rng = Rng::new(seed);
    let mut warm = build_workbook(&mut rng);

    // The very first recalc is compared too: `warm` has already had its cache
    // warmed and invalidated repeatedly by `build_workbook`'s writes.
    let mut cold = fresh(&warm);
    warm.recalc(&ctx());
    cold.recalc(&ctx());
    if warm.to_json().unwrap() != cold.to_json().unwrap() {
        return Err(format!(
            "seed={seed} step=build\nwarm: {}\nfresh: {}",
            warm.to_json().unwrap(),
            cold.to_json().unwrap()
        ));
    }

    for step in 0..8 {
        let edited = apply_edit(&mut rng, &mut warm);
        let c = if step % 3 == 2 { ctx_later() } else { ctx() };
        let mut cold = fresh(&warm);
        // Alternate the entry point: both read the same cache, and a stale
        // graph would surface on either.
        if step % 2 == 0 {
            warm.recalc(&c);
            cold.recalc(&c);
        } else {
            warm.recalc_incremental(&c, &edited);
            cold.recalc_incremental(&c, &edited);
        }
        let got = warm.to_json().unwrap();
        let want = cold.to_json().unwrap();
        if got != want {
            let touched: Vec<String> = edited
                .iter()
                .map(|(s, a)| format!("{s}!{}", a.to_a1()))
                .collect();
            return Err(format!(
                "seed={seed} step={step} edited={touched:?}\nwarm:  {got}\nfresh: {want}"
            ));
        }
    }
    Ok(())
}

fn seed_count() -> u64 {
    std::env::var("TRUECALC_CACHE_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

fn seed_base() -> u64 {
    std::env::var("TRUECALC_CACHE_SEED_BASE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// The cache-specific differential: the same document, recalculated against
/// whatever cache the live workbook has accumulated, versus recalculated by an
/// engine that has never seen it. Byte-identical or the cache is stale.
///
/// Scale it the way `recalc_differential_tests` scales:
///
/// ```text
/// TRUECALC_CACHE_SEEDS=20000 cargo test --release -p truecalc-workbook \
///     --test graph_cache_tests -- warm_and_fresh
/// ```
#[test]
fn warm_and_fresh_recalculations_agree() {
    let base = seed_base();
    let n = seed_count();
    let mut failures = 0usize;
    let mut reports: Vec<String> = Vec::new();
    for seed in base..base + n {
        if let Err(report) = run_seed(seed) {
            failures += 1;
            println!("  warm/fresh divergence at seed {seed}");
            if reports.len() < 3 {
                reports.push(report);
            }
        }
    }
    println!("warm-vs-fresh: {failures}/{n} divergences");
    assert!(
        failures == 0,
        "{failures} divergence(s) over {n} seeds from {base}:\n\n{}",
        reports.join("\n\n")
    );
}

/// The generator must actually produce the constructs the sweep depends on, or
/// it is green for the wrong reason. Checks the corpus, not one seed.
#[test]
fn the_generator_produces_the_constructs_it_claims() {
    let mut saw_structured = false;
    let mut saw_name = false;
    let mut saw_volatile = false;
    let mut saw_array = false;
    let mut saw_cross_sheet = false;
    let mut saw_missing_sheet_ref = false;

    for seed in 0..60u64 {
        let mut rng = Rng::new(seed);
        let wb = build_workbook(&mut rng);
        assert!(
            !wb.tables().is_empty(),
            "every generated workbook has a table"
        );
        for sheet in BASE_SHEETS {
            for row in 1..=ROWS {
                for col in 1..=COLS {
                    let Some(f) = wb.get(sheet, at(row, col)).and_then(|c| c.formula()) else {
                        continue;
                    };
                    saw_structured |= f.contains("TBL[");
                    saw_name |= f.contains("NAMEA");
                    saw_volatile |= f.contains("TODAY");
                    saw_array |= f.contains('{');
                    saw_cross_sheet |= f.contains("T!") || f.contains("S!");
                    saw_missing_sheet_ref |= f.contains("X!");
                }
            }
        }
    }
    assert!(saw_structured, "no structured references generated");
    assert!(saw_name, "no name references generated");
    assert!(saw_volatile, "no volatile cells generated");
    assert!(saw_array, "no array literals generated");
    assert!(saw_cross_sheet, "no cross-sheet references generated");
    assert!(
        saw_missing_sheet_ref,
        "no references to the add/remove sheet generated"
    );
}

/// The differential must be able to *see* a divergence, not merely fail to
/// produce one: a workbook left un-recalculated after an edit must compare
/// unequal to a freshly loaded copy that has recalculated. Without this, a
/// `fresh` that silently returned the warm workbook itself would look like a
/// passing sweep.
#[test]
fn the_comparison_detects_a_stale_grid() {
    let mut wb = small_workbook();
    wb.recalc(&ctx());
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    let mut cold = fresh(&wb);
    cold.recalc(&ctx());
    assert_ne!(
        wb.to_json().unwrap(),
        cold.to_json().unwrap(),
        "the warm-vs-fresh comparison must be able to observe a stale grid"
    );
}
