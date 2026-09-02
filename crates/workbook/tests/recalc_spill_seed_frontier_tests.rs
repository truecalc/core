//! Recalc-level guards for issue #930: a spill-seeded cell's dirtiness must
//! propagate through the frontier, not just land in `dirty`.
//!
//! ## Why this file exists
//!
//! `recalc_incremental` seeds a `dirty` set and drains a `VecDeque` frontier to
//! take the transitive closure of `direct_dependents_of`. `seed_spill_sensitive`
//! widens `dirty` with every spill-occupancy-sensitive cell — a cell whose value
//! can change because a spill footprint moved, with no dependency edge to carry
//! that change. It ran *after* the frontier had already drained and inserted
//! straight into `dirty`, so a cell it seeded recomputed itself but never
//! dirtied anything that reads it: everything downstream kept its stale stored
//! value.
//!
//! This is the same defect issue #926 fixed for volatile seeding, at a second
//! site, which is why the fix under test is structural — one seeding phase, one
//! drain — rather than another relocation.
//!
//! ## Why the existing suites stayed green
//!
//! `spill_incremental_recalc_tests`, `array_spill_tests` and
//! `recalc_incremental_property_tests` all pass with the bug present: every
//! incremental shape they exercise reaches its reader through an ordinary
//! dependency edge or through the spill-rectangle-overlap branch, and the one
//! vacated-spill guard (`authored_cell_index_tests`) asserts the seeded cell
//! itself and stops there. A discriminating shape needs all three of:
//!
//!  * **no array left on the grid** after the edit — a literal overwrite of an
//!    array anchor — so no spill rectangle exists for the overlap branch to fire
//!    on and the widen loop finds no changed rectangle to rescue anything;
//!  * **no dependency edge** from the edited cell to the reader, so the closure
//!    walk from the edit cannot reach it either; and
//!  * an assertion at least **two hops** downstream of the seeded cell, because
//!    a one-hop assertion passes under a half-fix that seeds only the seeded
//!    cell's direct dependents.
//!
//! ## Keeping the incremental tests honest
//!
//! Same two traps as `recalc_dependency_edge_coverage_tests.rs` and
//! `recalc_volatile_frontier_tests.rs`:
//!
//!  * every cell of a range precedent *downstream* of the seeded cell is
//!    **authored**, because an unauthored range cell makes
//!    `seed_spill_sensitive` dirty its reader directly — which would put the
//!    downstream reader in `dirty` on its own account and prove nothing about
//!    frontier propagation; and
//!  * the name-mediated assertion is on a cell **downstream** of the name's
//!    reader, because `precedent_is_spill_sensitive` treats every
//!    `Precedent::Name` as spill-sensitive unconditionally, so the name reader
//!    itself is seeded regardless of this bug.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// A fixed, DST-free context (GMT). Nothing here is volatile.
fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn wb_with(sheets: &[&str]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for name in sheets {
        wb.add_sheet(Worksheet::new(*name)).unwrap();
    }
    wb
}

fn val(wb: &Workbook, sheet: &str, cell: &str) -> Value {
    wb.get(sheet, a1(cell)).unwrap().value().clone()
}

/// Replaces the array anchor `A1` with a literal, vacating its spill footprint,
/// and returns what a **full** recalc of the resulting workbook produces — the
/// grid the incremental recalc under test has to reproduce.
fn overwrite_anchor_with_literal(wb: &mut Workbook) -> Workbook {
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    let mut full = wb.clone();
    full.recalc(&ctx());
    full
}

fn recalc_incremental_on_a1(wb: &mut Workbook) -> Vec<String> {
    wb.recalc_incremental(&ctx(), &[("S".to_string(), a1("A1"))])
        .iter()
        .map(|c| c.addr.to_a1())
        .collect()
}

// ---------------------------------------------------------------------------
// Shape 1: two hops downstream of a spill-seeded cell (the issue's own
// reproduction).
// ---------------------------------------------------------------------------

/// `A1={10,20,30}` spills onto `B1:C1`; `E1=SUM(B1:C1)` reads two spilled
/// cells; `E2` and `E3` chain off `E1`. Overwriting `A1` with a literal leaves
/// no array on the grid, so `E1` enters `dirty` only through
/// `seed_spill_sensitive` (its range holds the now-unauthored `B1`/`C1`), and
/// nothing carries that dirtiness to `E2`/`E3` unless the seed reaches the
/// frontier. `E3` is the discriminating assertion: a fix that seeds only `E1`'s
/// direct dependents refreshes `E2` and leaves `E3` at 50.
#[test]
fn two_hops_downstream_of_a_spill_seeded_cell_are_refreshed() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("S", a1("E1"), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap();
    wb.set("S", a1("E2"), CellInput::Formula("=E1+0".into()))
        .unwrap();
    wb.set("S", a1("E3"), CellInput::Formula("=E2+0".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(val(&wb, "S", "E1"), Value::Number(50.0));
    assert_eq!(val(&wb, "S", "E3"), Value::Number(50.0));

    let full = overwrite_anchor_with_literal(&mut wb);
    let touched = recalc_incremental_on_a1(&mut wb);

    assert_eq!(
        val(&wb, "S", "E1"),
        Value::Number(0.0),
        "E1 reads a spill that no longer exists; spill-occupancy seeding must \
         still put it in the dirty set"
    );
    assert_eq!(
        val(&wb, "S", "E2"),
        Value::Number(0.0),
        "E2 is one hop downstream of the spill-seeded E1; a stale 50 means the \
         spill seed never entered the frontier"
    );
    assert_eq!(
        val(&wb, "S", "E3"),
        Value::Number(0.0),
        "E3 is two hops downstream of the spill-seeded E1 (via E2); a stale 50 \
         here specifically catches a fix that seeds only E1's *direct* \
         dependents instead of feeding the seed into the frontier"
    );
    assert!(touched.contains(&"E2".to_string()), "{touched:?}");
    assert!(touched.contains(&"E3".to_string()), "{touched:?}");
    assert_eq!(wb, full, "incremental recalc did not reproduce full recalc");
}

// ---------------------------------------------------------------------------
// Shape 2: a spill-seeded cell's dependents reached through a range.
// ---------------------------------------------------------------------------

/// `E1` is the spill-seeded cell as above. `G1=SUM(E1:E2)` reaches it through a
/// **range** precedent whose every cell is authored (`E2` is a typed-in
/// literal), so the range branch of `seed_spill_sensitive` cannot seed `G1` on
/// its own account — `G1` is dirty only if the reverse range edge out of `E1`
/// was walked. `H1`, two hops out, is what a direct-dependents-only half-fix
/// still gets wrong.
#[test]
fn a_spill_seeded_cells_range_mediated_dependents_are_refreshed_two_hops_out() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("S", a1("E1"), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap();
    wb.set("S", a1("E2"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("G1"), CellInput::Formula("=SUM(E1:E2)".into()))
        .unwrap();
    wb.set("S", a1("H1"), CellInput::Formula("=G1*2".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(val(&wb, "S", "G1"), Value::Number(51.0));
    assert_eq!(val(&wb, "S", "H1"), Value::Number(102.0));

    let full = overwrite_anchor_with_literal(&mut wb);
    recalc_incremental_on_a1(&mut wb);

    assert_eq!(val(&wb, "S", "E1"), Value::Number(0.0));
    assert_eq!(
        val(&wb, "S", "G1"),
        Value::Number(1.0),
        "G1 reaches the spill-seeded E1 through a fully authored range; a stale \
         51 means the seed never propagated along the range edge"
    );
    assert_eq!(
        val(&wb, "S", "H1"),
        Value::Number(2.0),
        "H1 enters the dirty closure only by walking out of G1, which the range \
         edge E1 -> G1 must have put there; a stale 102 means propagation \
         stopped at the range reader"
    );
    assert_eq!(wb, full, "incremental recalc did not reproduce full recalc");
}

// ---------------------------------------------------------------------------
// Shape 3: a spill-seeded cell's dependents reached through a defined name.
// ---------------------------------------------------------------------------

/// Same shape, but `G1` reads the range through the defined name `SEEDED`.
/// `G1` is *not* the discriminating assertion here: `precedent_is_spill_sensitive`
/// treats every `Precedent::Name` as spill-sensitive unconditionally, so `G1` is
/// in `dirty` whether or not this bug is present — and `H1` is not
/// discriminating either, because a half-fix that seeds each seeded cell's
/// *direct* dependents picks `H1` up as a dependent of the unconditionally
/// seeded `G1`. The assertion that depends on real frontier propagation is
/// `I1`, two hops past the name reader.
#[test]
fn a_spill_seeded_cells_name_mediated_dependent_is_refreshed_two_hops_out() {
    let mut wb = wb_with(&["S"]);
    wb.define_name("SEEDED", "S!E1:E2").unwrap();
    wb.set("S", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("S", a1("E1"), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap();
    wb.set("S", a1("E2"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("G1"), CellInput::Formula("=SUM(SEEDED)".into()))
        .unwrap();
    wb.set("S", a1("H1"), CellInput::Formula("=G1*2".into()))
        .unwrap();
    wb.set("S", a1("I1"), CellInput::Formula("=H1+0".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(val(&wb, "S", "G1"), Value::Number(51.0));
    assert_eq!(val(&wb, "S", "I1"), Value::Number(102.0));

    let full = overwrite_anchor_with_literal(&mut wb);
    recalc_incremental_on_a1(&mut wb);

    assert_eq!(val(&wb, "S", "E1"), Value::Number(0.0));
    assert_eq!(val(&wb, "S", "G1"), Value::Number(1.0));
    assert_eq!(
        val(&wb, "S", "H1"),
        Value::Number(2.0),
        "H1 enters the dirty closure only by walking out of G1, which the name \
         edge SEEDED -> G1 (through the spill-seeded E1) must have put there; a \
         stale 102 means the spill seed never reached the name-mediated reader"
    );
    assert_eq!(
        val(&wb, "S", "I1"),
        Value::Number(2.0),
        "I1 is two hops past the name reader G1; a stale 102 here catches a fix \
         that seeds each seeded cell's direct dependents instead of walking the \
         frontier out of them"
    );
    assert_eq!(wb, full, "incremental recalc did not reproduce full recalc");
}

// ---------------------------------------------------------------------------
// Shape 4: a chain long enough that a fixed-depth fix fails.
// ---------------------------------------------------------------------------

/// Five hops downstream of the spill-seeded `E1` (`E2` through `E6`). A fix that
/// seeds only direct dependents, or that walks a fixed small number of extra
/// hops instead of feeding the seed into the frontier, refreshes some prefix of
/// this chain and then stops short of `E6`.
#[test]
fn a_five_hop_chain_downstream_of_a_spill_seeded_cell_is_fully_refreshed() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("S", a1("E1"), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap();
    for (cell, prev) in [
        ("E2", "E1"),
        ("E3", "E2"),
        ("E4", "E3"),
        ("E5", "E4"),
        ("E6", "E5"),
    ] {
        wb.set("S", a1(cell), CellInput::Formula(format!("={prev}+0")))
            .unwrap();
    }
    wb.recalc(&ctx());
    assert_eq!(val(&wb, "S", "E6"), Value::Number(50.0));

    let full = overwrite_anchor_with_literal(&mut wb);
    let touched = recalc_incremental_on_a1(&mut wb);

    for (cell, hop) in [("E2", 1), ("E3", 2), ("E4", 3), ("E5", 4), ("E6", 5)] {
        assert_eq!(
            val(&wb, "S", cell),
            Value::Number(0.0),
            "{cell} is {hop} hop(s) downstream of the spill-seeded E1; a stale \
             50 here means propagation stopped before reaching it"
        );
        assert!(touched.contains(&cell.to_string()), "{touched:?}");
    }
    assert_eq!(wb, full, "incremental recalc did not reproduce full recalc");
}

// ---------------------------------------------------------------------------
// Shape 5: a cascading two-anchor spill (issue #946).
// ---------------------------------------------------------------------------

/// `A1=SEQUENCE(1,F1)` spills onto `B1`; `C1=SEQUENCE(1,B1)` — a **second**
/// spill anchor whose own column count is driven by reading `A1`'s spilled
/// output — spills onto `D1`; `E1=D1+1000` reads the second spill; `G1=E1+1`
/// is one further hop out. Shrinking `F1` from 2 to 1 collapses `A1`'s spill
/// to a bare scalar, which in turn collapses `C1`'s (`SEQUENCE(1,1)` is a
/// 1x1 array — 1x1 is not spill-eligible per `core_array_to_workbook`, so
/// `C1` becomes `#NUM!`, `SEQUENCE`'s own error for a non-positive column
/// count once `B1` no longer exists). This is new coverage: every other test
/// in this file dirties its reader from **one** spill; here the reader's own
/// precedent is itself a second, cascading spill whose existence depends on
/// reading the first.
///
/// ## What this test does *not* prove
///
/// The plan for this issue proposed asserting this shape can only pass with
/// the widen loop's own `close_over_dependents` call (`recalc.rs`, inside the
/// `for pass in 0..max_widen` loop) in place — i.e. that commenting out just
/// that call would make `G1` go stale. Verified empirically (temporarily
/// commenting out that exact call and running this test, then the full
/// suite): **it does not discriminate that call.** `E1`'s precedent `D1` is,
/// like every non-anchor spilled cell, never authored — so
/// `seed_spill_sensitive`'s rule 3 marks `E1` spill-sensitive *unconditionally*,
/// before the widen loop ever runs, and the seeding-phase closure
/// (`recalc.rs`, immediately after `seed_spill_sensitive`) already walks
/// `E1`'s ordinary graph edge to `G1` — the widen loop's own closure call
/// never gets a chance to matter. This holds for every shape this file's
/// author could construct: any cell a spill can ever grow onto or vacate is,
/// by `place_spill`'s own occupancy rule, never authored, and
/// `seed_spill_sensitive`'s rules 3/4 mark *every* reader of *any*
/// non-authored cell spill-sensitive regardless of spill history — so nothing
/// the widen loop's insert can find is ever new. This is a structural
/// consequence of the current (deliberately conservative,
/// "over-seeding is safe") `seed_spill_sensitive` design, not a gap in this
/// test; see the `close_over_dependents` call site in `recalc.rs` for the
/// same finding recorded where the call actually lives.
///
/// Kept as a real regression test anyway — it is new, legitimate
/// `incremental ≡ full` coverage for a cascading-spill shape no existing test
/// exercises — just not evidence that the widen loop's closure call is
/// reachable.
#[test]
fn a_cascading_two_anchor_spill_is_fully_refreshed() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("F1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", a1("A1"), CellInput::Formula("=SEQUENCE(1,F1)".into()))
        .unwrap();
    wb.set("S", a1("C1"), CellInput::Formula("=SEQUENCE(1,B1)".into()))
        .unwrap();
    wb.set("S", a1("E1"), CellInput::Formula("=D1+1000".into()))
        .unwrap();
    wb.set("S", a1("G1"), CellInput::Formula("=E1+1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(val(&wb, "S", "E1"), Value::Number(1002.0));
    assert_eq!(val(&wb, "S", "G1"), Value::Number(1003.0));

    wb.set("S", a1("F1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    let mut full = wb.clone();
    full.recalc(&ctx());

    let touched = wb
        .recalc_incremental(&ctx(), &[("S".to_string(), a1("F1"))])
        .iter()
        .map(|c| c.addr.to_a1())
        .collect::<Vec<_>>();

    assert_eq!(
        val(&wb, "S", "C1"),
        Value::Error("#NUM!".to_string()),
        "C1's own spill (SEQUENCE(1,B1)) must collapse once A1's spill (and \
         so B1) is gone"
    );
    assert_eq!(
        val(&wb, "S", "E1"),
        Value::Number(1000.0),
        "E1 reads C1's now-vacated spill; a stale 1002 means the cascading \
         spill collapse was not propagated"
    );
    assert_eq!(
        val(&wb, "S", "G1"),
        Value::Number(1001.0),
        "G1 is one hop past E1; a stale 1003 means propagation stopped at E1"
    );
    assert!(touched.contains(&"G1".to_string()), "{touched:?}");
    assert_eq!(wb, full, "incremental recalc did not reproduce full recalc");
}
