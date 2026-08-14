//! Workbook-level mutation API (plan item 3.4, issue #536): cell set / get /
//! clear and named-range CRUD, plus the eager per-mutation limit enforcement
//! and named-range validity rules of schema spec §7 and scope ADR Decision 5.

use truecalc_workbook::{
    Address, Cell, CellInput, EngineFlavor, Table, Value, Workbook, WorkbookError, Worksheet,
};

fn wb_with_sheet(name: &str) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new(name)).unwrap();
    wb
}

fn a1(key: &str) -> Address {
    Address::from_a1(key).unwrap()
}

// ---- set: literals -------------------------------------------------------

#[test]
fn set_literal_stores_value_only_cell() {
    let mut wb = wb_with_sheet("S");
    let prev = wb
        .set("S", a1("A1"), CellInput::Literal(Value::Number(42.0)))
        .unwrap();
    assert!(prev.is_none());
    let cell = wb.get("S", a1("A1")).unwrap();
    assert_eq!(cell.value(), &Value::Number(42.0));
    assert_eq!(cell.formula(), None);
}

#[test]
fn set_overwrite_returns_previous_cell() {
    let mut wb = wb_with_sheet("S");
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    let prev = wb
        .set("S", a1("A1"), CellInput::Literal(Value::Text("hi".into())))
        .unwrap()
        .unwrap();
    assert_eq!(prev.value(), &Value::Number(1.0));
    assert_eq!(
        wb.get("S", a1("A1")).unwrap().value(),
        &Value::Text("hi".into())
    );
}

#[test]
fn set_empty_literal_is_rejected() {
    let mut wb = wb_with_sheet("S");
    let err = wb
        .set("S", a1("A1"), CellInput::Literal(Value::Empty))
        .unwrap_err();
    assert_eq!(err, WorkbookError::EmptyLiteral);
    // The workbook is untouched (atomicity).
    assert!(wb.get("S", a1("A1")).is_none());
}

#[test]
fn set_on_unknown_sheet_errors() {
    let mut wb = wb_with_sheet("S");
    let err = wb
        .set("Nope", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
}

#[test]
fn set_resolves_sheet_case_insensitively() {
    let mut wb = wb_with_sheet("Sheet1");
    wb.set("SHEET1", a1("B2"), CellInput::Literal(Value::Boolean(true)))
        .unwrap();
    assert_eq!(
        wb.get("sheet1", a1("B2")).unwrap().value(),
        &Value::Boolean(true)
    );
}

// ---- set: formulas -------------------------------------------------------

#[test]
fn set_valid_formula_stores_verbatim_and_empty_value() {
    let mut wb = wb_with_sheet("S");
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", a1("A2"), CellInput::Formula("=A1*2".into()))
        .unwrap();
    let cell = wb.get("S", a1("A2")).unwrap();
    assert_eq!(cell.formula(), Some("=A1*2"));
    // Unevaluated until recalc (P3.3): the value is empty, verbatim text kept.
    assert_eq!(cell.value(), &Value::Empty);
}

#[test]
fn set_invalid_formula_is_rejected() {
    let mut wb = wb_with_sheet("S");
    let err = wb
        .set("S", a1("A1"), CellInput::Formula("=1 +".into()))
        .unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
    assert!(wb.get("S", a1("A1")).is_none());
}

// ---- get / clear ---------------------------------------------------------

#[test]
fn get_returns_none_for_absent_cell_or_sheet() {
    let wb = wb_with_sheet("S");
    assert!(wb.get("S", a1("Z9")).is_none());
    assert!(wb.get("Other", a1("A1")).is_none());
}

#[test]
fn clear_removes_entry_and_returns_it() {
    let mut wb = wb_with_sheet("S");
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    let removed = wb.clear("S", a1("A1")).unwrap();
    assert_eq!(removed.value(), &Value::Number(5.0));
    assert!(wb.get("S", a1("A1")).is_none());
    // Clearing an absent cell yields None, not an empty entry.
    assert!(wb.clear("S", a1("A1")).is_none());
}

#[test]
fn clear_does_not_serialize_an_empty_entry() {
    // A set-then-clear leaves byte-identical output to never having set.
    let mut a = wb_with_sheet("S");
    let baseline = a.to_json().unwrap();
    a.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    a.clear("S", a1("A1"));
    assert_eq!(a.to_json().unwrap(), baseline);
}

// ---- eager limit enforcement (scope ADR Decision 5) ----------------------

#[test]
fn set_enforces_text_length_cap() {
    let mut wb = wb_with_sheet("S");
    let too_long = "x".repeat(truecalc_workbook::limits::MAX_TEXT_LEN + 1);
    let err = wb
        .set("S", a1("A1"), CellInput::Literal(Value::Text(too_long)))
        .unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
    // At exactly the cap, it succeeds.
    let at_cap = "x".repeat(truecalc_workbook::limits::MAX_TEXT_LEN);
    assert!(wb
        .set("S", a1("A2"), CellInput::Literal(Value::Text(at_cap)))
        .is_ok());
}

#[test]
fn set_enforces_array_element_cap() {
    let mut wb = wb_with_sheet("S");
    let row: Vec<Value> = (0..(truecalc_workbook::limits::MAX_ARRAY_ELEMENTS + 1))
        .map(|i| Value::Number(i as f64))
        .collect();
    let err = wb
        .set("S", a1("A1"), CellInput::Literal(Value::Array(vec![row])))
        .unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
}

#[test]
fn set_enforces_formula_length_cap() {
    let mut wb = wb_with_sheet("S");
    let huge = format!(
        "={}",
        "1+".repeat(truecalc_workbook::limits::MAX_FORMULA_LEN)
    );
    let err = wb.set("S", a1("A1"), CellInput::Formula(huge)).unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
}

#[test]
fn total_cells_counts_across_sheets() {
    let mut wb = wb_with_sheet("A");
    wb.add_sheet(Worksheet::new("B")).unwrap();
    wb.set("A", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("B", a1("A1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("B", a1("A2"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    assert_eq!(wb.total_cells(), 3);
}

#[test]
fn overwrite_does_not_grow_cell_count() {
    let mut wb = wb_with_sheet("S");
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    assert_eq!(wb.total_cells(), 1);
}

// ---- named-range CRUD ----------------------------------------------------

#[test]
fn define_name_stores_and_lists() {
    let mut wb = wb_with_sheet("Sheet2");
    wb.define_name("TaxRate", "Sheet2!B5").unwrap();
    let nr = wb.name("TaxRate").unwrap();
    assert_eq!(nr.name, "TaxRate");
    assert_eq!(nr.r#ref, "Sheet2!B5");
    assert_eq!(wb.names().len(), 1);
}

#[test]
fn name_lookup_is_case_insensitive() {
    let mut wb = wb_with_sheet("Sheet2");
    wb.define_name("TaxRate", "Sheet2!B5").unwrap();
    assert!(wb.name("TAXRATE").is_some());
    assert!(wb.name("taxrate").is_some());
}

#[test]
fn define_duplicate_name_is_rejected_case_insensitively() {
    let mut wb = wb_with_sheet("Sheet2");
    wb.define_name("TaxRate", "Sheet2!B5").unwrap();
    let err = wb.define_name("TAXRATE", "Sheet2!C1").unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
    assert_eq!(wb.names().len(), 1);
}

#[test]
fn define_name_rejects_collision_with_existing_table() {
    // truecalc/core#861 final review, Finding 3: `define_name` must reject
    // a name that collides with an existing *table* name, the same way
    // `define_table` already rejects a table name colliding with an
    // existing named range -- otherwise a workbook built entirely through
    // the public API can end up with a table and a named range sharing a
    // name, which `Workbook::from_json` then rejects on reload.
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("Recipe", "Sheet1!A1:B2").unwrap();
    assert!(matches!(
        wb.define_name("Recipe", "Sheet1!C1").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
    // Case-insensitively too.
    assert!(matches!(
        wb.define_name("RECIPE", "Sheet1!C1").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
    assert!(wb.name("Recipe").is_none());
}

#[test]
fn define_name_rejects_invalid_name() {
    let mut wb = wb_with_sheet("S");
    // A1-address-shaped name is invalid (schema spec §7).
    assert!(matches!(
        wb.define_name("A1", "S!B1").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
    // Boolean literal is invalid.
    assert!(matches!(
        wb.define_name("TRUE", "S!B1").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
}

#[test]
fn define_name_rejects_non_canonical_ref() {
    let mut wb = wb_with_sheet("S");
    // Degenerate range must collapse to single-cell form.
    assert!(matches!(
        wb.define_name("R", "S!A1:A1").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
    // Endpoints must be top-left first.
    assert!(matches!(
        wb.define_name("R2", "S!B2:A1").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
}

#[test]
fn define_name_rejects_dangling_ref() {
    let mut wb = wb_with_sheet("S");
    let err = wb.define_name("R", "Ghost!A1").unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
}

#[test]
fn redefine_name_changes_ref_only() {
    let mut wb = wb_with_sheet("Sheet2");
    wb.define_name("TaxRate", "Sheet2!B5").unwrap();
    wb.redefine_name("taxrate", "Sheet2!C1:D2").unwrap();
    let nr = wb.name("TaxRate").unwrap();
    // Original casing of the name is preserved; only the ref changed.
    assert_eq!(nr.name, "TaxRate");
    assert_eq!(nr.r#ref, "Sheet2!C1:D2");
    assert_eq!(wb.names().len(), 1);
}

#[test]
fn redefine_unknown_name_errors() {
    let mut wb = wb_with_sheet("S");
    let err = wb.redefine_name("Missing", "S!A1").unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
}

#[test]
fn redefine_to_dangling_ref_is_rejected() {
    let mut wb = wb_with_sheet("S");
    wb.define_name("R", "S!A1").unwrap();
    let err = wb.redefine_name("R", "Ghost!A1").unwrap_err();
    assert!(matches!(err, WorkbookError::Mutation(_)));
    // The original ref is unchanged after the rejected redefine.
    assert_eq!(wb.name("R").unwrap().r#ref, "S!A1");
}

#[test]
fn remove_name_returns_entry_then_none() {
    let mut wb = wb_with_sheet("S");
    wb.define_name("R", "S!A1").unwrap();
    let removed = wb.remove_name("r").unwrap();
    assert_eq!(removed.name, "R");
    assert!(wb.name("R").is_none());
    assert!(wb.remove_name("R").is_none());
}

// ---- table CRUD + auto-expand-by-append (truecalc/core#861) --------------

#[test]
fn define_table_adds_it() {
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("Recipe", "Sheet1!A1:B2").unwrap();
    assert_eq!(wb.table("Recipe").unwrap().r#ref, "Sheet1!A1:B2");
}

#[test]
fn define_table_rejects_invalid_name() {
    let mut wb = wb_with_sheet("Sheet1");
    assert!(wb.define_table("A1", "Sheet1!A1:B2").is_err());
}

#[test]
fn remove_table_removes_it() {
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("Recipe", "Sheet1!A1:B2").unwrap();
    assert!(wb.remove_table("Recipe").is_some());
    assert!(wb.table("Recipe").is_none());
}

#[test]
fn set_below_table_range_auto_expands_it() {
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("Recipe", "Sheet1!A1:B2").unwrap(); // header A1:B1, one data row A2:B2
    wb.set(
        "Sheet1",
        Address::new(3, 1).unwrap(),
        CellInput::Literal(Value::Text("flour".into())),
    )
    .unwrap(); // A3, directly below
    assert_eq!(wb.table("Recipe").unwrap().r#ref, "Sheet1!A1:B3");
}

#[test]
fn set_outside_table_column_span_does_not_expand() {
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("Recipe", "Sheet1!A1:B2").unwrap();
    wb.set(
        "Sheet1",
        Address::new(3, 5).unwrap(),
        CellInput::Literal(Value::Text("unrelated".into())),
    )
    .unwrap(); // column E, row 3
    assert_eq!(wb.table("Recipe").unwrap().r#ref, "Sheet1!A1:B2");
}

#[test]
fn set_does_not_expand_a_table_into_an_overlapping_table() {
    // truecalc/core#861 final review, Finding 1 (Critical): auto-expand-by-
    // append must never grow a table into another table's range. Two
    // adjacent, non-overlapping tables are defined (accepted, since neither
    // overlaps the other); writing into the row that would expand the
    // first into the second must silently skip the expansion -- matching
    // real Excel, which does not auto-expand a table into another table's
    // cells -- while the ordinary cell write itself still succeeds.
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("A", "Sheet1!A1:B2").unwrap(); // rows 1-2
    wb.define_table("B", "Sheet1!A3:B5").unwrap(); // rows 3-5, no overlap
    let write = wb.set(
        "Sheet1",
        a1("A3"),
        CellInput::Literal(Value::Text("x".into())),
    );
    assert!(write.is_ok(), "the ordinary cell write must still succeed");
    assert_eq!(
        wb.table("A").unwrap().r#ref,
        "Sheet1!A1:B2",
        "table A must not expand into table B's range"
    );
    assert_eq!(wb.table("B").unwrap().r#ref, "Sheet1!A3:B5");
}

#[test]
fn overlapping_table_check_does_not_panic_on_a_hand_pushed_invalid_ref() {
    // truecalc/core#861 final review, Finding 4: `tables_mut()` is public
    // and lets a caller push a `Table` with an arbitrary `ref`, bypassing
    // `define_table`'s validation entirely. A later `define_table` call
    // (which compares the new range against every existing table via
    // `overlapping_table`) must not panic on that unparseable stored ref --
    // it should just treat that table as "no overlap determinable" and
    // continue.
    let mut wb = wb_with_sheet("Sheet1");
    wb.tables_mut().push(Table {
        name: "Bad".into(),
        r#ref: "not a valid ref".into(),
    });
    // Must not panic; a genuinely valid new table still gets defined.
    assert!(wb.define_table("Good", "Sheet1!A1:B2").is_ok());
}

#[test]
fn define_table_rejects_beyond_the_table_cap() {
    // truecalc/core#861 final review, Finding 8: tables have no resource
    // cap anywhere, unlike named ranges (`MAX_NAMED_RANGES`). Seed up to
    // `MAX_TABLES` via `tables_mut()` directly (bypassing per-table
    // validation, which is irrelevant to this cap check and would make the
    // test needlessly slow), then confirm `define_table` eagerly rejects
    // one more.
    let mut wb = wb_with_sheet("Sheet1");
    for i in 0..truecalc_workbook::limits::MAX_TABLES {
        wb.tables_mut().push(Table {
            name: format!("T{i}"),
            r#ref: "Sheet1!A1:A1".into(),
        });
    }
    assert!(matches!(
        wb.define_table("OneMore", "Sheet1!B1:B2").unwrap_err(),
        WorkbookError::Mutation(_)
    ));
}

#[test]
fn define_table_over_headerless_region_breaks_round_trip_until_headers_are_written() {
    // truecalc/core#861 final review, Finding 9: `define_table` deliberately
    // does not validate header-row content -- a table may be defined ahead
    // of its header cells being written (shape first, headers later). This
    // is a known, documented gap: the table succeeds in memory, but
    // `from_json`'s load-time validation *does* check header content, so
    // the workbook cannot round-trip through JSON until real header text is
    // written into the header row.
    let mut wb = wb_with_sheet("Sheet1");
    wb.define_table("Recipe", "Sheet1!A1:B2").unwrap();
    assert!(
        Workbook::from_json(wb.to_json().unwrap().as_bytes()).is_err(),
        "a table over an empty header row must fail to reload"
    );

    // Writing real header text into the header row heals the round trip.
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
    assert!(Workbook::from_json(wb.to_json().unwrap().as_bytes()).is_ok());
}

// ---- value-object / serialization integrity ------------------------------

#[test]
fn mutations_roundtrip_through_canonical_json() {
    let mut wb = wb_with_sheet("Sheet1");
    wb.add_sheet(Worksheet::new("Sheet2")).unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(100.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.define_name("TaxRate", "Sheet2!B5").unwrap();

    let json = wb.to_json().unwrap();
    let reparsed = Workbook::from_json(json.as_bytes()).unwrap();
    assert_eq!(reparsed.to_json().unwrap(), json);
    assert_eq!(reparsed, wb);
}

#[test]
fn set_then_clear_then_set_matches_direct_set() {
    // Mutation order does not leak into the wire format.
    let mut a = wb_with_sheet("S");
    a.set("S", a1("A1"), CellInput::Literal(Value::Number(9.0)))
        .unwrap();

    let mut b = wb_with_sheet("S");
    b.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    b.clear("S", a1("A1"));
    b.set("S", a1("A1"), CellInput::Literal(Value::Number(9.0)))
        .unwrap();

    assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
}

// Silence unused-import lint if Cell is only referenced in some configs.
#[allow(dead_code)]
fn _uses_cell(_c: &Cell) {}
