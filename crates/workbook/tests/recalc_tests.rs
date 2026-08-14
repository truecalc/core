//! Recalc engine behavior (P3.3, issue #535): full recalc, dependency order,
//! cross-sheet refs, named ranges, change list, and the value seam P3.5 builds
//! on for arrays.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, CIRCULAR_ERROR,
};

/// A fixed, DST-free context (GMT) pinned to an arbitrary instant. Non-volatile
/// tests are insensitive to it; volatile tests assert against it explicitly.
fn ctx() -> RecalcContext {
    // 2026-06-08T00:00:00Z under Etc/GMT.
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn sheets_wb() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(truecalc_workbook::Worksheet::new("Sheet1"))
        .unwrap();
    wb
}

#[test]
fn full_recalc_evaluates_a_simple_chain_in_order() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=A1*3".into()))
        .unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Formula("=A2+1".into()))
        .unwrap();

    let changes = wb.recalc(&ctx());

    assert_eq!(
        wb.get("Sheet1", a1("A2")).unwrap().value(),
        &Value::Number(6.0)
    );
    assert_eq!(
        wb.get("Sheet1", a1("A3")).unwrap().value(),
        &Value::Number(7.0)
    );
    // Both formula cells went Empty -> value, so both are changes, ordered by row.
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].addr, a1("A2"));
    assert_eq!(changes[1].addr, a1("A3"));
    assert_eq!(changes[0].old, Value::Empty);
    assert_eq!(changes[0].new, Value::Number(6.0));
}

#[test]
fn date_typed_literal_round_trips_exactly_and_propagates_through_arithmetic() {
    // Issue #721: a host stores a serial *as a Date* (CellInput::Literal with
    // Value::Date), and the engine's type propagation keeps offset arithmetic
    // rendering as a date.
    let mut wb = sheets_wb();
    // A modern date serial and a pre-1900 (negative) serial: both must round-trip
    // bit-for-bit — the value is stored verbatim, never rebuilt via DATE(y,m,d).
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Date(46180.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Date(-1.5)))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Formula("=A1-7".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=A1-A2".into()))
        .unwrap();

    wb.recalc(&ctx());

    // Exact serial round-trip on the stored literals.
    assert_eq!(wb.get("Sheet1", a1("A1")).unwrap().value(), &Value::Date(46180.0));
    assert_eq!(wb.get("Sheet1", a1("A2")).unwrap().value(), &Value::Date(-1.5));
    // date + number and date − number stay date-typed.
    assert_eq!(wb.get("Sheet1", a1("B1")).unwrap().value(), &Value::Date(46181.0));
    assert_eq!(wb.get("Sheet1", a1("C1")).unwrap().value(), &Value::Date(46173.0));
    // date − date is a plain day count.
    assert_eq!(
        wb.get("Sheet1", a1("D1")).unwrap().value(),
        &Value::Number(46181.5)
    );
}

#[test]
fn full_recalc_is_idempotent_with_no_changes_second_time() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1+5".into()))
        .unwrap();
    let first = wb.recalc(&ctx());
    assert_eq!(first.len(), 1);
    let second = wb.recalc(&ctx());
    assert!(second.is_empty(), "no values change on a second recalc");
}

#[test]
fn cross_sheet_reference_reads_other_sheet() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(truecalc_workbook::Worksheet::new("Data"))
        .unwrap();
    wb.add_sheet(truecalc_workbook::Worksheet::new("Calc"))
        .unwrap();
    wb.set("Data", a1("A1"), CellInput::Literal(Value::Number(40.0)))
        .unwrap();
    wb.set("Calc", a1("A1"), CellInput::Formula("=Data!A1+2".into()))
        .unwrap();

    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Calc", a1("A1")).unwrap().value(),
        &Value::Number(42.0)
    );
}

#[test]
fn missing_sheet_reference_is_ref_error() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=Nope!A1".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error("#REF!".into())
    );
}

#[test]
fn undefined_name_is_name_error() {
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Formula("=NOT_DEFINED".into()),
    )
    .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error("#NAME?".into())
    );
}

#[test]
fn named_scalar_and_range_resolve_through_recalc() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(Value::Number(30.0)))
        .unwrap();
    wb.define_name("PRICES", "Sheet1!A1:A3").unwrap();
    wb.define_name("FIRST", "Sheet1!A1").unwrap();
    wb.set(
        "Sheet1",
        a1("C1"),
        CellInput::Formula("=SUM(PRICES)".into()),
    )
    .unwrap();
    wb.set("Sheet1", a1("C2"), CellInput::Formula("=FIRST*2".into()))
        .unwrap();

    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("C1")).unwrap().value(),
        &Value::Number(60.0)
    );
    assert_eq!(
        wb.get("Sheet1", a1("C2")).unwrap().value(),
        &Value::Number(20.0)
    );
}

#[test]
fn self_referential_cell_is_circular_error() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    let changes = wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(CIRCULAR_ERROR.into())
    );
    assert_eq!(changes.len(), 1);
}

#[test]
fn two_cell_cycle_marks_both_cells_and_terminates() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=B1+1".into()))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.recalc(&ctx()); // must not loop forever
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(CIRCULAR_ERROR.into())
    );
    assert_eq!(
        wb.get("Sheet1", a1("B1")).unwrap().value(),
        &Value::Error(CIRCULAR_ERROR.into())
    );
}

#[test]
fn cell_downstream_of_a_cycle_is_tainted_with_the_error() {
    let mut wb = sheets_wb();
    // A1 <-> B1 cycle; C1 reads A1 (downstream of the cycle).
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=B1".into()))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1".into()))
        .unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Formula("=A1+5".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("C1")).unwrap().value(),
        &Value::Error(CIRCULAR_ERROR.into())
    );
}

#[test]
fn acyclic_cells_still_evaluate_when_an_unrelated_cycle_exists() {
    let mut wb = sheets_wb();
    // Independent cycle.
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=A2".into()))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=A1".into()))
        .unwrap();
    // Independent healthy chain.
    wb.set("Sheet1", a1("D1"), CellInput::Literal(Value::Number(4.0)))
        .unwrap();
    wb.set("Sheet1", a1("D2"), CellInput::Formula("=D1*2".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("D2")).unwrap().value(),
        &Value::Number(8.0)
    );
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(CIRCULAR_ERROR.into())
    );
}

#[test]
fn array_result_is_stored_at_the_anchor_pending_spill() {
    // P3.3 stores an array result at the anchor cell; spill placement is P3.5.
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());
    match wb.get("Sheet1", a1("A1")).unwrap().value() {
        Value::Array(rows) => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[0][0], Value::Number(1.0));
            assert_eq!(rows[1][1], Value::Number(4.0));
        }
        other => panic!("expected an array anchor value, got {other:?}"),
    }
}

#[test]
fn whole_column_table_ref_resolves_to_data_array() {
    // truecalc/core#861 PR2: T[col] materializes the column's data-row
    // values; SUM flattens the array regardless of orientation.
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Literal(Value::Text("col".into())),
    )
    .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    wb.define_table("T", "Sheet1!A1:A3").unwrap();
    wb.set(
        "Sheet1",
        a1("B1"),
        CellInput::Formula("=SUM(T[col])".into()),
    )
    .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("B1")).unwrap().value(),
        &Value::Number(30.0)
    );
}

#[test]
fn table_column_ref_is_a_precedent_of_writes_to_that_column() {
    // truecalc/core#861 PR2: depgraph.rs must track a table-column read as a
    // real precedent, so editing the table's data cell dirties the dependent
    // formula on *incremental* recalc, not just a full recalc.
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Literal(Value::Text("col".into())),
    )
    .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.define_table("T", "Sheet1!A1:A2").unwrap();
    wb.set(
        "Sheet1",
        a1("B1"),
        CellInput::Formula("=SUM(T[col])".into()),
    )
    .unwrap();
    wb.recalc(&ctx());

    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    let changes = wb.recalc_incremental(&ctx(), &[("Sheet1".to_string(), a1("A2"))]);
    let touched: Vec<Address> = changes.iter().map(|c| c.addr).collect();
    assert!(
        touched.contains(&a1("B1")),
        "SUM(T[col]) should have recalculated on incremental recalc"
    );
}

#[test]
fn table_formula_reading_a_different_column_of_its_own_table_is_not_a_false_cycle() {
    // truecalc/core#861 final review, Finding 2: `resolve_table_precedent`'s
    // whole-column branch used to record a precedent over the *whole table
    // range*, not just the resolved column. A formula living inside the
    // table that reads a whole *different* column of that same table (e.g.
    // this common percentage-of-total pattern) was then a precedent of
    // itself -- its own cell always falls inside the table's full
    // rectangle -- tripping cycle detection with no real circularity. B2
    // reads column "qty" while writing column "pct"; it must resolve
    // normally, not to the circular-reference error.
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Literal(Value::Text("qty".into())),
    )
    .unwrap();
    wb.set(
        "Sheet1",
        a1("B1"),
        CellInput::Literal(Value::Text("pct".into())),
    )
    .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.set(
        "Sheet1",
        a1("B2"),
        CellInput::Formula("=SUM(T[qty])".into()),
    )
    .unwrap();
    wb.define_table("T", "Sheet1!A1:B2").unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("B2")).unwrap().value(),
        &Value::Number(5.0),
        "B2 reads a different column of its own table; must not be flagged circular"
    );
}

#[test]
fn current_row_table_ref_resolves_per_row() {
    // truecalc/core#861 PR2: [@col] resolves to the cell at (current row,
    // column) within the table the formula's own cell is inside.
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Literal(Value::Text("qty".into())),
    )
    .unwrap();
    wb.set(
        "Sheet1",
        a1("B1"),
        CellInput::Literal(Value::Text("price".into())),
    )
    .unwrap();
    wb.set(
        "Sheet1",
        a1("C1"),
        CellInput::Literal(Value::Text("total".into())),
    )
    .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    wb.set("Sheet1", a1("B2"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set(
        "Sheet1",
        a1("C2"),
        CellInput::Formula("=[@qty]*[@price]".into()),
    )
    .unwrap();
    wb.define_table("T", "Sheet1!A1:C2").unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("C2")).unwrap().value(),
        &Value::Number(6.0)
    );
}

#[test]
fn current_row_ref_outside_any_table_is_ref_error() {
    // truecalc/core#861 PR2: an unqualified [@col] outside any table's data
    // rows has nothing to infer the table from, so it stays #REF!.
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("E5"), CellInput::Formula("=[@x]".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("E5")).unwrap().value(),
        &Value::Error("#REF!".into())
    );
}

#[test]
fn table_ref_column_lookup_is_case_insensitive() {
    // truecalc/core#861 PR2 fix round 1 (Finding 1): column-name lookup must
    // case-fold like the table-name and sheet-name lookups in the same
    // method, since column names are already case-folded for uniqueness at
    // table-definition time (`table_ref::header_row_columns`).
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Literal(Value::Text("col".into())),
    )
    .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.define_table("T", "Sheet1!A1:A2").unwrap();
    wb.set(
        "Sheet1",
        a1("B1"),
        CellInput::Formula("=SUM(T[COL])".into()),
    )
    .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("B1")).unwrap().value(),
        &Value::Number(10.0)
    );
}

#[test]
fn whole_column_table_ref_unwraps_spill_anchor_cell_like_resolve_range() {
    // truecalc/core#861 PR2 fix round 1 (Finding 2): a table-column cell
    // whose own stored value is itself an array (the on-grid shape a
    // spill-formula anchor stores, per schema spec §5/§6) must resolve to
    // its own [0][0] scalar, exactly as `resolve_range` does for an
    // equivalent explicit vertical range — not embed the whole array as
    // that row's "scalar". Compare T[col] against the equivalent explicit
    // A2:A3 range: they must produce the same SUM.
    //
    // A3 is authored directly as a literal array (rather than a spilling
    // formula) so the test is independent of dependency-graph evaluation
    // order: `depgraph.rs` currently records a `Ref::Table` precedent as
    // `Precedent::Unresolved`, i.e. a table reference does not yet create a
    // graph edge to the cells it reads, so a formula-based anchor's
    // evaluation order relative to a table-referencing formula is not
    // guaranteed — a separate, pre-existing gap outside this fix round's
    // two findings. `GridResolver::cell_value` reads a literal array cell
    // through the exact same code path as a formula-computed spill anchor
    // (both are simply "a cell whose stored `Value` is `Array`"), so this
    // is a faithful, deterministic test of the fix.
    let mut wb = sheets_wb();
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Literal(Value::Text("col".into())),
    )
    .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.define_table("T", "Sheet1!A1:A3").unwrap();
    wb.set(
        "Sheet1",
        a1("A3"),
        CellInput::Literal(Value::Array(vec![vec![
            Value::Number(10.0),
            Value::Number(20.0),
        ]])),
    )
    .unwrap();
    wb.set(
        "Sheet1",
        a1("D1"),
        CellInput::Formula("=SUM(T[col])".into()),
    )
    .unwrap();
    wb.set("Sheet1", a1("D2"), CellInput::Formula("=SUM(A2:A3)".into()))
        .unwrap();
    wb.recalc(&ctx());
    let table_sum = wb.get("Sheet1", a1("D1")).unwrap().value().clone();
    let range_sum = wb.get("Sheet1", a1("D2")).unwrap().value().clone();
    assert_eq!(table_sum, range_sum);
    assert_eq!(table_sum, Value::Number(15.0));
}

#[test]
fn change_list_is_ordered_by_sheet_then_row_then_column() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(truecalc_workbook::Worksheet::new("S1"))
        .unwrap();
    wb.add_sheet(truecalc_workbook::Worksheet::new("S2"))
        .unwrap();
    wb.set("S2", a1("B2"), CellInput::Formula("=1+1".into()))
        .unwrap();
    wb.set("S1", a1("B1"), CellInput::Formula("=2+2".into()))
        .unwrap();
    wb.set("S1", a1("A1"), CellInput::Formula("=3+3".into()))
        .unwrap();

    let changes = wb.recalc(&ctx());
    let order: Vec<(String, String)> = changes
        .iter()
        .map(|c| (c.sheet.clone(), c.addr.to_a1()))
        .collect();
    assert_eq!(
        order,
        vec![
            ("S1".into(), "A1".into()),
            ("S1".into(), "B1".into()),
            ("S2".into(), "B2".into()),
        ]
    );
}
