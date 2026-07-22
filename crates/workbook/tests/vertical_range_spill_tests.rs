//! Elementwise ops over a vertical range spill down, not sideways (issue
//! #724 — the range half of #707; the array-constructor half was fixed in
//! #723).
//!
//! `resolve_range` materializes every range as a core `Value::Array`; a
//! vertical (single-column, multi-row) range must keep its column
//! orientation through that materialization so an elementwise op like
//! `=A1:A3*2` spills down (an Nx1 column) the way Google Sheets does,
//! instead of collapsing to a 1xN row.

use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

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

#[test]
fn elementwise_op_over_a_vertical_range_spills_down_not_sideways() {
    let mut wb = wb();
    // A1:A3 is a vertical (1-column, 3-row) range: 1, 2, 3.
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(1.0))).unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(num(2.0))).unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(num(3.0))).unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1:A3*2".into()))
        .unwrap();
    wb.recalc(&ctx());

    // The anchor stores an Nx1 column — [[2],[4],[6]] — not a 1xN row.
    assert_eq!(
        wb.get("Sheet1", a1("B1")).unwrap().value(),
        &Value::Array(vec![vec![num(2.0)], vec![num(4.0)], vec![num(6.0)]])
    );

    // It spills down: B2 and B3 hold the rest of the column.
    let r = wb.resolved("Sheet1", a1("B2")).expect("B2 spilled");
    assert_eq!(r.value, num(4.0));
    assert_eq!(r.anchor, Some(a1("B1")));
    let r = wb.resolved("Sheet1", a1("B3")).expect("B3 spilled");
    assert_eq!(r.value, num(6.0));
    assert_eq!(r.anchor, Some(a1("B1")));

    // It does NOT spill sideways: C1 (where a wrongly-horizontal 1x3 row
    // would have placed its second element) stays empty.
    assert!(wb.resolved("Sheet1", a1("C1")).is_none());
}
