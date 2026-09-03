//! `incremental ≡ full` across spill footprint and blocked-status changes
//! (issue #591, found in P3.5 review of PR #590).
//!
//! A spilled cell is not a formula node (P3.2) and a *blocked* anchor stores an
//! error rather than an array reading its blocker, so the dependency graph
//! carries no edge for these transitions; `set` also discards a former anchor's
//! array before recalc runs. Without spill-occupancy seeding, an incremental
//! recalc therefore diverges from a full recalc when a spill shrinks, grows,
//! gets blocked, or gets unblocked. These tests pin the fix: after each such
//! edit, `recalc_incremental` produces a grid byte-identical to a fresh full
//! `recalc()`, and the change events carry the correct `old`/`new` values.
//!
//! Behavioral, per schema spec §5/§6/§12 (the authority cited by #537): the
//! array values come from the engine evaluating the formula, and the assertions
//! concern spill geometry, blocking, and the incremental≡full guarantee.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
    BLOCKED_SPILL_ERROR,
};

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is valid")
}

fn wb() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

fn num(n: f64) -> Value {
    Value::Number(n)
}

/// Apply `edit` to a clone (full recalc) and to the live workbook
/// (incremental recalc from `edited`), and assert the two grids are
/// byte-identical canonical JSON. Returns the incremental change list.
fn assert_incremental_equals_full(
    live: &mut Workbook,
    edited: &[(&str, &str)],
) -> Vec<truecalc_workbook::Change> {
    let mut full = live.clone();
    full.recalc(&ctx());
    let owned: Vec<(String, Address)> = edited
        .iter()
        .map(|(s, a)| ((*s).to_string(), a1(a)))
        .collect();
    let changes = live.recalc_incremental(&ctx(), &owned);
    assert_eq!(
        live.to_json().unwrap(),
        full.to_json().unwrap(),
        "incremental and full grids diverged"
    );
    changes
}

// --- (1) anchor shrink: reader of a vacated cell -----------------------------

#[test]
fn shrink_redirties_reader_of_vacated_cell() {
    // Repro from #591 case 1: A1={10,20} spills A1:B1; D1=B1+1 reads spilled B1.
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20}".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=B1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.resolved("Sheet1", a1("B1")).unwrap().value, num(20.0));
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(21.0));

    // Replace the anchor with a scalar: B1 is vacated, so D1 = empty+1 = 1.
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=7".into()))
        .unwrap();
    let changes = assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);

    assert_eq!(wb.get("Sheet1", a1("A1")).unwrap().value(), &num(7.0));
    assert_eq!(
        wb.get("Sheet1", a1("D1")).unwrap().value(),
        &num(1.0),
        "the reader of the vacated cell must be re-dirtied"
    );
    // B1 is no longer spilled.
    assert!(wb.resolved("Sheet1", a1("B1")).is_none());

    // The change event for D1 carries the pre-edit value 21 and the new value 1.
    let d1 = changes
        .iter()
        .find(|c| c.addr == a1("D1"))
        .expect("D1 changed");
    assert_eq!(d1.old, num(21.0));
    assert_eq!(d1.new, num(1.0));
}

#[test]
fn shrink_to_smaller_array_redirties_reader_of_dropped_column() {
    // A1={10,20,30} spills A1:C1; E1=C1+1 reads spilled C1.
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("Sheet1", a1("E1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("E1")).unwrap().value(), &num(31.0));

    // Shrink to a 1x2 array: C1 is vacated, E1 = empty+1 = 1.
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2}".into()))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);
    assert_eq!(wb.resolved("Sheet1", a1("B1")).unwrap().value, num(2.0));
    assert!(wb.resolved("Sheet1", a1("C1")).is_none());
    assert_eq!(wb.get("Sheet1", a1("E1")).unwrap().value(), &num(1.0));
}

#[test]
fn replace_anchor_with_literal_vacates_and_redirties() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20}".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=B1+1".into()))
        .unwrap();
    wb.recalc(&ctx());

    // Overwrite the anchor with a *literal* (not a formula).
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(5.0)))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(1.0));
}

// --- (2) anchor grow ---------------------------------------------------------

#[test]
fn grow_redirties_reader_of_newly_spilled_cell() {
    // A1={10,20} spills A1:B1; D1=C1+1 reads C1 (empty -> 1).
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20}".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(1.0));

    // Grow to A1:C1: C1 now spills 30, so D1 = 30+1 = 31.
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);
    assert_eq!(wb.resolved("Sheet1", a1("C1")).unwrap().value, num(30.0));
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(31.0));
}

// --- (3) unblock: clear the blocker ------------------------------------------

#[test]
fn clearing_blocker_reexpands_spill() {
    // Repro from #591 case 2: C1=99 blocks A1={1,2,3}; E1=C1+1 reads C1.
    let mut wb = wb();
    wb.set("Sheet1", a1("C1"), CellInput::Literal(num(99.0)))
        .unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2,3}".into()))
        .unwrap();
    wb.set("Sheet1", a1("E1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned()),
        "A1 is blocked by C1"
    );
    assert_eq!(wb.get("Sheet1", a1("E1")).unwrap().value(), &num(100.0));

    // Clear the blocker: A1 must re-expand to [1,2,3], so C1 spills 3 and
    // E1 = 3+1 = 4.
    wb.clear("Sheet1", a1("C1"));
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "C1")]);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Array(vec![vec![num(1.0), num(2.0), num(3.0)]]),
        "A1 re-expands once its blocker is cleared"
    );
    assert_eq!(wb.resolved("Sheet1", a1("C1")).unwrap().value, num(3.0));
    assert_eq!(wb.get("Sheet1", a1("E1")).unwrap().value(), &num(4.0));
}

// --- (3b) blocker overwrite then unblock ------------------------------------

#[test]
fn overwriting_then_clearing_blocker_is_incremental_consistent() {
    // C1=99 blocks A1={1,2,3}; E1=C1+1 reads C1.
    let mut wb = wb();
    wb.set("Sheet1", a1("C1"), CellInput::Literal(num(99.0)))
        .unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2,3}".into()))
        .unwrap();
    wb.set("Sheet1", a1("E1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned())
    );

    // Overwrite the blocker *in place* with a new literal: an authored cell
    // still occupies C1, so A1 stays blocked — and the incremental result must
    // match a full recalc (E1 tracks the new C1 value).
    wb.set("Sheet1", a1("C1"), CellInput::Literal(num(7.0)))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "C1")]);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned()),
        "an in-place overwrite still occupies the cell, so A1 stays blocked"
    );
    assert_eq!(wb.get("Sheet1", a1("E1")).unwrap().value(), &num(8.0));

    // Now overwrite the blocker with a formula (still authored, still blocks),
    // again incremental≡full.
    wb.set("Sheet1", a1("C1"), CellInput::Formula("=1+1".into()))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "C1")]);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned())
    );

    // Finally clear the blocker: A1 re-expands (unblock), incremental≡full.
    wb.clear("Sheet1", a1("C1"));
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "C1")]);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Array(vec![vec![num(1.0), num(2.0), num(3.0)]])
    );
    assert_eq!(wb.resolved("Sheet1", a1("C1")).unwrap().value, num(3.0));
    assert_eq!(wb.get("Sheet1", a1("E1")).unwrap().value(), &num(4.0));
}

// --- (4) block: write into a spill region ------------------------------------

#[test]
fn writing_into_spill_region_blocks_anchor() {
    // A1={1,2,3} spills A1:C1; D1=C1+1 reads spilled C1.
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2,3}".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(4.0));

    // Write a literal into B1 (inside the spill region): A1 must block, so
    // C1 is no longer spilled and D1 = empty+1 = 1.
    wb.set("Sheet1", a1("B1"), CellInput::Literal(num(50.0)))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "B1")]);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned()),
        "writing into the spill region blocks the anchor"
    );
    assert!(wb.resolved("Sheet1", a1("C1")).is_none());
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(1.0));
}

// --- range readers over spills (seeding clause 4) ----------------------------

#[test]
fn range_reader_over_spill_tracks_block_and_unblock() {
    // S=SUM(B1:C1) aggregates the *spilled* cells of A1={1,2,3} (B1=2, C1=3).
    // (The range deliberately excludes the anchor A1 so the test isolates
    // spilled-cell aggregation; an anchor-spanning range has its own
    // double-counting semantics that are out of scope for issue #591.)
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2,3}".into()))
        .unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("A3")).unwrap().value(), &num(5.0));

    // Block by writing into B1: A1 collapses to #REF!, so B1:C1 sees the
    // authored B1=50 and an empty C1 -> SUM = 50.
    wb.set("Sheet1", a1("B1"), CellInput::Literal(num(50.0)))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "B1")]);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned())
    );
    assert_eq!(wb.get("Sheet1", a1("A3")).unwrap().value(), &num(50.0));

    // Unblock by clearing B1: A1 re-expands, B1/C1 spill 2/3 again, SUM = 5.
    wb.clear("Sheet1", a1("B1"));
    assert_incremental_equals_full(&mut wb, &[("Sheet1", "B1")]);
    assert_eq!(wb.get("Sheet1", a1("A3")).unwrap().value(), &num(5.0));
}

// --- a non-spill edit still recomputes only its closure ----------------------

#[test]
fn non_spill_edit_does_not_overdirty() {
    // No spills anywhere: editing A1 must not pull unrelated formula cells into
    // the change list (the minimal-closure guarantee still holds).
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(1.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    // Independent chain reading an authored cell only.
    wb.set("Sheet1", a1("B1"), CellInput::Literal(num(100.0)))
        .unwrap();
    wb.set("Sheet1", a1("B2"), CellInput::Formula("=B1+1".into()))
        .unwrap();
    wb.recalc(&ctx());

    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(10.0)))
        .unwrap();
    let changes = assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);
    let touched: Vec<String> = changes.iter().map(|c| c.addr.to_a1()).collect();
    assert!(touched.contains(&"A2".to_string()));
    assert!(
        !touched.iter().any(|t| t.starts_with('B')),
        "B column must be untouched by a non-spill edit: {touched:?}"
    );
}

// --- issue #991: Design A / the AuthoredCellIndex-cache fallback, on real
// spill dynamics rather than a synthetic pre-image count -------------------

/// A pure-scalar edit, several cells upstream of the anchor, that flips the
/// anchor's own formula result from a scalar to an array — creating a spill
/// that did not exist before this call. Exercises Design A's lazy pre-image
/// fold (the anchor's own old/new pair, plus the newly-spilled cell's
/// reader's) and the `AuthoredCellIndex`-cache fallback (the reader's range/
/// cell precedent is examined during the very seeding pass this edit
/// triggers) together, on a genuinely new spill rather than a resize of an
/// existing one.
#[test]
fn scalar_edit_creates_a_new_spill_several_cells_downstream() {
    // A1 (scalar) -> B1 = A1*2 -> C1 = IF(B1>10, {1,2,3}, 9) -> F1 = D1+1,
    // where D1 is C1's spilled cell once C1 actually spills. The edit only
    // ever touches A1, two hops upstream of the anchor and three upstream of
    // F1.
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(5.0)))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1*2".into()))
        .unwrap();
    wb.set(
        "Sheet1",
        a1("C1"),
        CellInput::Formula("=IF(B1>10,{1,2,3},9)".into()),
    )
    .unwrap();
    wb.set("Sheet1", a1("F1"), CellInput::Formula("=D1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("C1")).unwrap().value(), &num(9.0));
    assert!(
        wb.resolved("Sheet1", a1("D1")).is_none(),
        "no spill exists yet"
    );
    assert_eq!(wb.get("Sheet1", a1("F1")).unwrap().value(), &num(1.0));

    // A pure scalar edit, upstream of the anchor, that flips the branch.
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(10.0)))
        .unwrap();
    let changes = assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);

    assert_eq!(wb.resolved("Sheet1", a1("D1")).unwrap().value, num(2.0));
    assert_eq!(wb.get("Sheet1", a1("F1")).unwrap().value(), &num(3.0));

    let c1 = changes
        .iter()
        .find(|c| c.addr == a1("C1"))
        .expect("C1 changed");
    assert_eq!(c1.old, num(9.0), "C1's pre-recalc value was the scalar 9");
    assert_eq!(c1.new, wb.get("Sheet1", a1("C1")).unwrap().value().clone());

    let f1 = changes
        .iter()
        .find(|c| c.addr == a1("F1"))
        .expect("F1 changed");
    assert_eq!(f1.old, num(1.0));
    assert_eq!(f1.new, num(3.0));
}

/// The mirror of the test above: a pure-scalar edit, several cells upstream,
/// that flips an anchor from an array back to a scalar — **removing** an
/// existing spill rather than creating one. Exercises the same pre-image fold
/// on a shrink-to-nothing rather than a grow-from-nothing, and confirms the
/// vacated cell's reader is correctly re-dirtied and its `old` value is the
/// true pre-recalc (spilled) value, not the post-vacate one.
#[test]
fn scalar_edit_removes_an_existing_spill_several_cells_downstream() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(10.0)))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1*2".into()))
        .unwrap();
    wb.set(
        "Sheet1",
        a1("C1"),
        CellInput::Formula("=IF(B1>10,{1,2,3},9)".into()),
    )
    .unwrap();
    wb.set("Sheet1", a1("F1"), CellInput::Formula("=D1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.resolved("Sheet1", a1("D1")).unwrap().value, num(2.0));
    assert_eq!(wb.get("Sheet1", a1("F1")).unwrap().value(), &num(3.0));
    // The true pre-recalc value of C1 (the spilling array), captured before
    // the edit so the assertion below cannot be satisfied by a bug that just
    // echoes back this call's own (possibly wrong) bookkeeping.
    let c1_pre_recalc = wb.get("Sheet1", a1("C1")).unwrap().value().clone();

    // A pure scalar edit, upstream of the anchor, that flips the branch back.
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(5.0)))
        .unwrap();
    let changes = assert_incremental_equals_full(&mut wb, &[("Sheet1", "A1")]);

    assert_eq!(wb.get("Sheet1", a1("C1")).unwrap().value(), &num(9.0));
    assert!(
        wb.resolved("Sheet1", a1("D1")).is_none(),
        "the spill must be vacated"
    );
    assert_eq!(wb.get("Sheet1", a1("F1")).unwrap().value(), &num(1.0));

    let c1 = changes
        .iter()
        .find(|c| c.addr == a1("C1"))
        .expect("C1 changed");
    assert_eq!(
        c1.old, c1_pre_recalc,
        "C1's old value must be the true pre-recalc array, not the \
         post-vacate scalar"
    );
    assert_eq!(c1.new, num(9.0));

    let f1 = changes
        .iter()
        .find(|c| c.addr == a1("F1"))
        .expect("F1 changed");
    assert_eq!(f1.old, num(3.0));
    assert_eq!(f1.new, num(1.0));
}

/// A workbook that already holds a real, placed spill on its **stored** grid
/// (loaded via `from_json`, exactly as a host restoring a saved document
/// would) and is recalculated **incrementally on its very first recalc of any
/// kind** — never once calling `recalc()` first. Every one of issue #991's
/// caches (the dependency graph, the spill-anchor rectangles, and the
/// AuthoredCellIndex fallback) must therefore cold-start correctly from
/// scratch inside a single incremental call, against a grid that is not
/// merely empty-of-formulas but already has spill geometry to reason about.
#[test]
fn from_json_workbook_with_a_pre_existing_spill_recalculates_incrementally_on_its_first_call() {
    // Build and fully recalculate a workbook with a real spill once, purely
    // to obtain valid canonical JSON with the spill's placed array already on
    // the grid (schema spec §5/§6) — it is the *document*, not this process's
    // cache state, that `from_json` below actually loads.
    let mut seed = wb();
    seed.set("Sheet1", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    seed.set("Sheet1", a1("E1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    seed.recalc(&ctx());
    assert_eq!(seed.get("Sheet1", a1("E1")).unwrap().value(), &num(31.0));
    let json = seed.to_json().unwrap();

    // A freshly loaded workbook: no dependency-graph cache, no spill-anchor
    // cache, no authored-cell-index cache — all cold.
    let mut live = Workbook::from_json(json.as_bytes()).unwrap();
    assert!(!live.graph_cache_is_warm());
    assert!(!live.anchor_cache_is_warm());
    assert!(!live.authored_index_cache_is_warm());

    // Never call `.recalc()` on `live` — the whole point of this test. Shrink
    // the pre-existing spill straight from the cold start.
    live.set("Sheet1", a1("A1"), CellInput::Formula("={1,2}".into()))
        .unwrap();
    let changes = assert_incremental_equals_full(&mut live, &[("Sheet1", "A1")]);

    assert_eq!(live.resolved("Sheet1", a1("B1")).unwrap().value, num(2.0));
    assert!(
        live.resolved("Sheet1", a1("C1")).is_none(),
        "the shrink vacates C1"
    );
    assert_eq!(live.get("Sheet1", a1("E1")).unwrap().value(), &num(1.0));

    let e1 = changes
        .iter()
        .find(|c| c.addr == a1("E1"))
        .expect("E1 changed");
    assert_eq!(
        e1.old,
        num(31.0),
        "E1's old value must be the value from the loaded document, not \
         Value::Empty from some assumed prior recalc"
    );
    assert_eq!(e1.new, num(1.0));
}
