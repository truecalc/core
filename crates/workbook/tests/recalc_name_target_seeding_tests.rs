//! Guards for the **name** branch of spill-occupancy seeding (issue #925).
//!
//! ## Why this file exists
//!
//! `precedent_is_spill_sensitive` used to answer "yes" for every
//! `Precedent::Name`, whatever the name pointed at. That made every reader of
//! every defined name dirty on every incremental recalc — and it made the
//! name-mediated assertions in `recalc_dependency_edge_coverage_tests.rs`,
//! `recalc_volatile_frontier_tests.rs` and `recalc_spill_seed_frontier_tests.rs`
//! all have to assert a cell *downstream* of the name's reader, because the
//! reader itself was seeded regardless of anything they were testing.
//!
//! The rule is now "put the name's current target through the rule it would
//! have got by being referenced directly". These are the shapes that prove the
//! narrowing did not throw away the case the old rule existed for: a name whose
//! target **is** a spilled cell, and a name whose target is a **range over**
//! spilled cells.
//!
//! ## Why the randomized differential is not enough on its own
//!
//! It is the instrument that found this class, but dropping the name branch
//! entirely diverges on roughly 1 workbook in 1,600 — invisible at the seed
//! count a committed test run can afford, and invisible to every other suite in
//! the repo. A generator finds a class; a named shape pins it for the price of
//! a microsecond. Both are needed.
//!
//! ## Keeping these tests honest
//!
//! Each shape is built so that **only** the name branch can put the reader in
//! the dirty closure:
//!
//!  * the edited cell has no dependency edge to the reader — the reader's only
//!    precedent is the name, and the name's target is not the edited cell;
//!  * after the edit no array is left on the grid, so no spill rectangle exists
//!    for the overlap branch and the widen loop finds no changed rectangle; and
//!  * the assertion is two hops past the seeded cell, so a half-fix that seeds
//!    only the name's direct reader still fails.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn value(wb: &Workbook, cell: &str) -> Value {
    wb.get("S", a1(cell))
        .map(|c| c.value().clone())
        .unwrap_or(Value::Empty)
}

/// `A1` spills `A1:C1`; a name targets one of the spilled cells; the name's
/// reader feeds a second cell. Returns the workbook already fully recalculated.
fn workbook_with(name_ref: &str, reader: &str) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.define_name("TARGET", name_ref).unwrap();
    wb.set("S", a1("E1"), CellInput::Formula(reader.to_owned()))
        .unwrap();
    wb.set("S", a1("F1"), CellInput::Formula("=E1*10".into()))
        .unwrap();
    wb.recalc(&ctx());
    wb
}

/// Overwrites the anchor with a literal — vacating `B1` and `C1` — and
/// recalculates incrementally, reporting the same workbook fully recalculated
/// for comparison.
fn vacate_the_spill(wb: &mut Workbook) -> Workbook {
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    let full = {
        let mut clone = wb.clone();
        clone.recalc(&ctx());
        clone
    };
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), a1("A1"))]);
    full
}

/// A name whose target **is** a spilled cell.
///
/// `TARGET` → `B1`, which `A1`'s array covers. `B1` is not a formula cell, so
/// the graph carries no edge from `A1` to anything reading `B1`; the name's
/// target being unauthored is the only thing that can dirty `E1` when `A1`
/// stops spilling.
#[test]
fn a_name_targeting_a_spilled_cell_refreshes_its_reader() {
    let mut wb = workbook_with("S!B1", "=TARGET+1");
    assert_eq!(value(&wb, "E1"), Value::Number(21.0), "precondition");
    assert_eq!(value(&wb, "F1"), Value::Number(210.0), "precondition");

    let full = vacate_the_spill(&mut wb);

    assert_eq!(
        value(&wb, "F1"),
        Value::Number(10.0),
        "F1 is two hops past the name's reader: with B1 vacated, E1 is 1 and \
         F1 is 10"
    );
    assert_eq!(
        wb.to_json().unwrap(),
        full.to_json().unwrap(),
        "incremental must reproduce the full recalc"
    );
}

/// A name whose target is a **range over** spilled cells.
///
/// `TARGET` → `B1:C1`, both of which `A1`'s array covers. Same argument as
/// above, through the range half of the name rule.
#[test]
fn a_name_targeting_a_range_of_spilled_cells_refreshes_its_reader() {
    let mut wb = workbook_with("S!B1:C1", "=SUM(TARGET)");
    assert_eq!(value(&wb, "E1"), Value::Number(50.0), "precondition");
    assert_eq!(value(&wb, "F1"), Value::Number(500.0), "precondition");

    let full = vacate_the_spill(&mut wb);

    assert_eq!(
        value(&wb, "F1"),
        Value::Number(0.0),
        "F1 is two hops past the name's reader: with B1:C1 vacated, E1 is 0 \
         and F1 is 0"
    );
    assert_eq!(
        wb.to_json().unwrap(),
        full.to_json().unwrap(),
        "incremental must reproduce the full recalc"
    );
}

/// The mirror of the two above: a name pointing at an ordinary authored,
/// non-spilling cell must **not** drag its readers into the closure. Without
/// this, "seed every name" passes the two guards above and the narrowing is
/// unguarded in the direction it was actually made.
#[test]
fn a_name_targeting_an_ordinary_cell_does_not_dirty_its_readers() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(4.0)))
        .unwrap();
    wb.define_name("RATE", "S!A1").unwrap();
    wb.set("S", a1("B1"), CellInput::Formula("=RATE*2".into()))
        .unwrap();
    // An unrelated pair: editing `D1` reaches `D2` and nothing else.
    wb.set("S", a1("D1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("D2"), CellInput::Formula("=D1+1".into()))
        .unwrap();
    wb.recalc(&ctx());

    wb.set("S", a1("D1"), CellInput::Literal(Value::Number(7.0)))
        .unwrap();
    let (_, closure) = wb.recalc_incremental_measured(&ctx(), &[("S".to_owned(), a1("D1"))]);
    assert_eq!(
        closure, 1,
        "only D2 depends on D1; B1 reads a name whose target is an ordinary \
         authored cell and must stay out of the closure"
    );
}
