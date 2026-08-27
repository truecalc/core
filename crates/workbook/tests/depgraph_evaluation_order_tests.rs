//! `evaluation_order` equals the two-call path it replaces (issue #908).
//!
//! A recalculation needs the evaluation order *and* the set of cells on a
//! cycle. Asking for them separately — `cycle_cells()` then
//! `topological_order()` — derived the same formula-cell adjacency from the
//! precedent lists twice per recalculation and threw it away twice (and, on a
//! cyclic graph, four times: `topological_order` recomputes the cycle set to
//! report it, and `acyclic_order_excluding` builds the adjacency again).
//! `evaluation_order` builds it once.
//!
//! That is only a saving if it answers identically, ordering included: the
//! order is what evaluation follows, so any difference is a difference in
//! results. These tests assert the two paths agree exactly.

use std::collections::BTreeSet;

use truecalc_workbook::{
    Address, Cell, CellRef, DependencyGraph, EngineFlavor, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).unwrap()
}

fn wb_one_sheet() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

fn set_formula(wb: &mut Workbook, a1: &str, formula: &str) {
    wb.sheet_mut("Sheet1")
        .unwrap()
        .set(addr(a1), Cell::with_formula(formula, Value::Empty));
}

/// The path `recompute` used to take, kept here as the reference answer.
fn separate_calls(graph: &DependencyGraph) -> (Vec<CellRef>, BTreeSet<CellRef>) {
    let cycle = graph.cycle_cells();
    let order = match graph.topological_order() {
        Ok(order) => order,
        Err(_) => graph.acyclic_order_excluding(&cycle),
    };
    (order, cycle)
}

fn assert_same(label: &str, wb: &Workbook) {
    let graph = DependencyGraph::build(wb);
    assert_eq!(
        graph.evaluation_order(),
        separate_calls(&graph),
        "{label}: one-pass evaluation order differs from the two-call path"
    );
}

#[test]
fn acyclic_chain() {
    let mut wb = wb_one_sheet();
    wb.sheet_mut("Sheet1")
        .unwrap()
        .set(addr("A1"), Cell::literal(Value::Number(1.0)).unwrap());
    for r in 2..=25u32 {
        let prev = Address::new(r - 1, 1).unwrap().to_a1();
        set_formula(
            &mut wb,
            &Address::new(r, 1).unwrap().to_a1(),
            &format!("={prev}+1"),
        );
    }
    set_formula(&mut wb, "C1", "=SUM(A1:A25)");
    assert_same("acyclic chain", &wb);
}

#[test]
fn empty_workbook_has_an_empty_order() {
    let wb = wb_one_sheet();
    assert_same("empty workbook", &wb);
    assert_eq!(
        DependencyGraph::build(&wb).evaluation_order(),
        (Vec::new(), BTreeSet::new())
    );
}

#[test]
fn two_cell_cycle_with_untouched_and_downstream_cells() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "A1", "=B1+1");
    set_formula(&mut wb, "B1", "=A1+1");
    set_formula(&mut wb, "C1", "=A1*2"); // downstream of the cycle
    set_formula(&mut wb, "D1", "=C1*2"); // downstream of the downstream
    set_formula(&mut wb, "E1", "=1"); // untouched by the cycle
    set_formula(&mut wb, "F1", "=E1+1");
    assert_same("two-cell cycle", &wb);

    let graph = DependencyGraph::build(&wb);
    let (order, cycle) = graph.evaluation_order();
    assert_eq!(
        cycle,
        BTreeSet::from([
            CellRef::from_display_name("Sheet1", addr("A1")),
            CellRef::from_display_name("Sheet1", addr("B1")),
        ])
    );
    // The cells that do not read the cycle still evaluate; the cycle and
    // everything downstream of it does not.
    let placed: BTreeSet<&Address> = order.iter().map(|c| &c.addr).collect();
    assert!(placed.contains(&addr("E1")) && placed.contains(&addr("F1")));
    assert!(!placed.contains(&addr("C1")) && !placed.contains(&addr("D1")));
}

#[test]
fn self_referential_cell() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "A1", "=A1+1");
    set_formula(&mut wb, "B1", "=2");
    assert_same("self loop", &wb);
}

#[test]
fn cycle_through_a_range() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "A1", "=SUM(B1:B3)");
    set_formula(&mut wb, "B1", "=A1+1");
    set_formula(&mut wb, "B2", "=1");
    set_formula(&mut wb, "C1", "=SUM(A1:B3)");
    assert_same("cycle through a range", &wb);
}
