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

/// A vertical range now materializes with one extra level of `Array`
/// nesting (see above). Statistical functions that flatten their array
/// argument by hand (rather than routing through a fully-recursive helper)
/// must still see every cell — not silently drop them because they only
/// unwrap one level of nesting.
#[test]
fn statistical_functions_still_see_every_cell_of_a_vertical_range() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(1.0))).unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(num(2.0))).unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(num(3.0))).unwrap();
    // A separate vertical range with a repeated value, for MODE.
    wb.set("Sheet1", a1("D1"), CellInput::Literal(num(1.0))).unwrap();
    wb.set("Sheet1", a1("D2"), CellInput::Literal(num(2.0))).unwrap();
    wb.set("Sheet1", a1("D3"), CellInput::Literal(num(2.0))).unwrap();

    wb.set("Sheet1", a1("B1"), CellInput::Formula("=SMALL(A1:A3,1)".into())).unwrap();
    wb.set("Sheet1", a1("B2"), CellInput::Formula("=LARGE(A1:A3,1)".into())).unwrap();
    wb.set("Sheet1", a1("B3"), CellInput::Formula("=GEOMEAN(A1:A3)".into())).unwrap();
    wb.set("Sheet1", a1("B4"), CellInput::Formula("=RANK(A1,A1:A3)".into())).unwrap();
    wb.set("Sheet1", a1("B5"), CellInput::Formula("=MAXA(A1:A3)".into())).unwrap();
    wb.set("Sheet1", a1("B6"), CellInput::Formula("=MINA(A1:A3)".into())).unwrap();
    wb.set("Sheet1", a1("B7"), CellInput::Formula("=MODE(D1:D3)".into())).unwrap();
    wb.set("Sheet1", a1("B8"), CellInput::Formula("=PERCENTRANK(A1:A3,2)".into())).unwrap();
    wb.recalc(&ctx());

    assert_eq!(wb.get("Sheet1", a1("B1")).unwrap().value(), &num(1.0), "SMALL");
    assert_eq!(wb.get("Sheet1", a1("B2")).unwrap().value(), &num(3.0), "LARGE");
    let geomean = wb.get("Sheet1", a1("B3")).unwrap().value().clone();
    match geomean {
        Value::Number(n) => assert!((n - 1.817_120_593).abs() < 1e-6, "GEOMEAN got {n}"),
        other => panic!("GEOMEAN should be a number, got {other:?}"),
    }
    assert_eq!(wb.get("Sheet1", a1("B4")).unwrap().value(), &num(3.0), "RANK");
    assert_eq!(wb.get("Sheet1", a1("B5")).unwrap().value(), &num(3.0), "MAXA");
    assert_eq!(wb.get("Sheet1", a1("B6")).unwrap().value(), &num(1.0), "MINA");
    assert_eq!(wb.get("Sheet1", a1("B7")).unwrap().value(), &num(2.0), "MODE");
    assert_eq!(wb.get("Sheet1", a1("B8")).unwrap().value(), &num(0.5), "PERCENTRANK");
}

/// Issue #840: a *bare* vertical range reference (no elementwise op) must
/// spill down, not sideways. `=A1:A3` should behave exactly like
/// `=A1:A3*2` in the test above, just without the multiplication.
#[test]
fn bare_vertical_range_reference_spills_down_not_sideways() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(num(1.0))).unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(num(2.0))).unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(num(3.0))).unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1:A3".into()))
        .unwrap();
    wb.recalc(&ctx());

    // The anchor stores an Nx1 column — [[1],[2],[3]] — not a 1xN row.
    assert_eq!(
        wb.get("Sheet1", a1("B1")).unwrap().value(),
        &Value::Array(vec![vec![num(1.0)], vec![num(2.0)], vec![num(3.0)]])
    );

    // It spills down: B2 and B3 hold the rest of the column.
    let r = wb.resolved("Sheet1", a1("B2")).expect("B2 spilled");
    assert_eq!(r.value, num(2.0));
    assert_eq!(r.anchor, Some(a1("B1")));
    let r = wb.resolved("Sheet1", a1("B3")).expect("B3 spilled");
    assert_eq!(r.value, num(3.0));
    assert_eq!(r.anchor, Some(a1("B1")));

    // It does NOT spill sideways: C1 stays empty.
    assert!(wb.resolved("Sheet1", a1("C1")).is_none());
}
