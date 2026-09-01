//! The workbook owns the dangling-ref invariant across a structural change
//! (issue #969).
//!
//! Two halves, one rule: `rename_sheet` is holistic — it repoints
//! `NamedRange.ref` / `Table.ref` and rewrites formula text that qualifies a
//! reference with the old sheet name — and `to_json` refuses to emit a
//! document `from_json` would refuse to read. Before both, a rename could
//! produce a workbook that serialized cleanly and could never be loaded again.

use truecalc_workbook::{
    Address, Cell, EngineFlavor, NamedRange, Table, Value, Workbook, Worksheet,
};

fn wb_with(sheets: &[&str]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in sheets {
        wb.add_sheet(Worksheet::new(*s)).unwrap();
    }
    wb
}

fn set_formula(wb: &mut Workbook, sheet: &str, a1: &str, formula: &str) {
    wb.sheet_mut(sheet).unwrap().set(
        Address::from_a1(a1).unwrap(),
        Cell::with_formula(formula, Value::Empty),
    );
}

fn formula_at<'a>(wb: &'a Workbook, sheet: &str, a1: &str) -> &'a str {
    wb.sheet(sheet)
        .unwrap()
        .get(Address::from_a1(a1).unwrap())
        .unwrap()
        .formula()
        .unwrap()
}

fn name_ref<'a>(wb: &'a Workbook, name: &str) -> &'a str {
    &wb.names().iter().find(|n| n.name == name).unwrap().r#ref
}

fn table_ref<'a>(wb: &'a Workbook, name: &str) -> &'a str {
    &wb.tables().iter().find(|t| t.name == name).unwrap().r#ref
}

/// The exact reported failure: a rename that walked cells only produced a
/// document that serialized cleanly and could never be reloaded.
#[test]
fn renaming_a_sheet_a_named_range_points_at_still_round_trips() {
    let mut wb = wb_with(&["Sheet10"]);
    wb.define_name("Revenues", "Sheet10!A1:A3").unwrap();

    wb.rename_sheet("Sheet10", "Revenue2026").unwrap();

    assert_eq!(
        name_ref(&wb, "Revenues"),
        "Revenue2026!A1:A3",
        "the named range must follow the sheet it points at"
    );

    // The whole point: saving succeeds *and* the result loads again.
    let json = wb
        .to_json()
        .expect("a renamed workbook must still serialize");
    let reloaded = Workbook::from_json(json.as_bytes())
        .unwrap_or_else(|e| panic!("the saved document must be loadable, but from_json said: {e}"));
    assert_eq!(reloaded.to_json().unwrap(), json, "canonical round trip");
}

#[test]
fn renaming_a_sheet_repoints_a_table_ref() {
    let mut wb = wb_with(&["Data"]);
    for (a1, header) in [("A1", "Item"), ("B1", "Qty")] {
        wb.sheet_mut("Data").unwrap().set(
            Address::from_a1(a1).unwrap(),
            Cell::literal(Value::Text(header.to_owned())).unwrap(),
        );
    }
    wb.define_table("Recipe", "Data!A1:B4").unwrap();

    wb.rename_sheet("Data", "Ingredients").unwrap();

    assert_eq!(table_ref(&wb, "Recipe"), "Ingredients!A1:B4");
    let json = wb.to_json().unwrap();
    Workbook::from_json(json.as_bytes()).expect("a renamed table ref must reload");
}

#[test]
fn renaming_a_sheet_rewrites_qualified_formula_text() {
    let mut wb = wb_with(&["Sheet1", "Sheet2", "Report"]);
    set_formula(&mut wb, "Report", "A1", "=Sheet1!A1+Sheet1!B2:B4");
    set_formula(&mut wb, "Report", "A2", "=Sheet2!A1"); // another sheet
    set_formula(&mut wb, "Report", "A3", "=A1+1"); // unqualified
    set_formula(&mut wb, "Report", "A4", "=\"Sheet1!A1\""); // string literal

    wb.rename_sheet("Sheet1", "Data").unwrap();

    assert_eq!(formula_at(&wb, "Report", "A1"), "=Data!A1+Data!B2:B4");
    assert_eq!(
        formula_at(&wb, "Report", "A2"),
        "=Sheet2!A1",
        "a reference to a different sheet is untouched"
    );
    assert_eq!(
        formula_at(&wb, "Report", "A3"),
        "=A1+1",
        "an unqualified reference is untouched"
    );
    assert_eq!(
        formula_at(&wb, "Report", "A4"),
        "=\"Sheet1!A1\"",
        "a string literal is untouched"
    );
}

#[test]
fn renaming_to_a_name_that_needs_quoting_requotes_refs_and_formulas() {
    let mut wb = wb_with(&["Data", "Report"]);
    wb.define_name("Revenues", "Data!A1:A3").unwrap();
    set_formula(&mut wb, "Report", "A1", "=SUM(Data!A1:A3)");

    wb.rename_sheet("Data", "Q2 Données").unwrap();

    assert_eq!(name_ref(&wb, "Revenues"), "'Q2 Données'!A1:A3");
    assert_eq!(formula_at(&wb, "Report", "A1"), "=SUM('Q2 Données'!A1:A3)");
    let json = wb.to_json().unwrap();
    Workbook::from_json(json.as_bytes()).expect("a requoted ref must reload");
}

#[test]
fn a_pure_case_change_rename_repoints_refs_too() {
    let mut wb = wb_with(&["data", "Report"]);
    wb.define_name("Revenues", "data!A1:A3").unwrap();
    set_formula(&mut wb, "Report", "A1", "=data!A1");

    wb.rename_sheet("data", "DATA").unwrap();

    assert_eq!(name_ref(&wb, "Revenues"), "DATA!A1:A3");
    assert_eq!(formula_at(&wb, "Report", "A1"), "=DATA!A1");
}

/// A ref stated in a different case than the sheet's declared name still
/// targets that sheet (sheet identity is case-insensitive, §2), so a rename
/// must repoint it.
#[test]
fn renaming_repoints_a_ref_whose_sheet_token_differs_in_case() {
    let mut wb = wb_with(&["Budget"]);
    wb.names_mut().push(NamedRange {
        name: "Totals".into(),
        r#ref: "BUDGET!A1:A3".into(),
    });

    wb.rename_sheet("Budget", "Plan").unwrap();

    assert_eq!(name_ref(&wb, "Totals"), "Plan!A1:A3");
}

#[test]
fn to_json_refuses_a_named_range_dangling_to_a_removed_sheet() {
    let mut wb = wb_with(&["Sheet10", "Other"]);
    wb.define_name("Revenues", "Sheet10!A1:A3").unwrap();

    wb.remove_sheet("Sheet10").unwrap();

    let err = wb
        .to_json()
        .expect_err("saving a document from_json would reject must fail");
    assert_eq!(
        err.to_string(),
        "named range \"Revenues\" refers to sheet \"Sheet10\", \
         which does not exist (schema spec §7)"
    );
}

#[test]
fn to_json_refuses_a_table_dangling_to_a_removed_sheet() {
    let mut wb = wb_with(&["Data", "Other"]);
    wb.tables_mut().push(Table {
        name: "Recipe".into(),
        r#ref: "Data!A1:B4".into(),
    });

    wb.remove_sheet("Data").unwrap();

    let err = wb
        .to_json()
        .expect_err("a dangling table ref must not save");
    assert_eq!(
        err.to_string(),
        "table \"Recipe\" refers to sheet \"Data\", which does not exist \
         (structured-references spec §4)"
    );
}

/// `to_json`'s check and `from_json`'s check must be the same check: whatever
/// message one produces, the other produces for the same document.
#[test]
fn to_json_and_from_json_report_the_same_dangling_ref() {
    let mut wb = wb_with(&["Sheet10", "Keep"]);
    wb.define_name("Revenues", "Sheet10!A1:A3").unwrap();
    let good = wb.to_json().unwrap();

    wb.remove_sheet("Sheet10").unwrap();
    let to_json_err = wb.to_json().unwrap_err().to_string();

    // The same document, hand-built as JSON, as `from_json` sees it.
    let dangling = good.replace("\"name\":\"Sheet10\"", "\"name\":\"Gone\"");
    let from_json_err = Workbook::from_json(dangling.as_bytes())
        .unwrap_err()
        .to_string();

    assert_eq!(to_json_err, from_json_err);
}

/// A formula naming a sheet that does not exist is *not* a §7 violation:
/// `from_json` accepts it (it evaluates to an error at recalc), so `to_json`
/// must accept it too. The two checks agree in both directions.
#[test]
fn a_formula_referencing_a_missing_sheet_still_saves_and_loads() {
    let mut wb = wb_with(&["Report"]);
    set_formula(&mut wb, "Report", "A1", "=Ghost!A1");

    let json = wb
        .to_json()
        .expect("a formula ref to a missing sheet is legal (it evaluates to an error)");
    Workbook::from_json(json.as_bytes()).expect("and from_json accepts it too");
}

#[test]
fn a_failed_rename_leaves_the_document_untouched() {
    let mut wb = wb_with(&["Alpha", "Beta", "Report"]);
    wb.define_name("Totals", "Alpha!A1:A3").unwrap();
    set_formula(&mut wb, "Report", "A1", "=Alpha!A1");

    assert!(wb.rename_sheet("Alpha", "beta").is_err());

    assert_eq!(name_ref(&wb, "Totals"), "Alpha!A1:A3");
    assert_eq!(formula_at(&wb, "Report", "A1"), "=Alpha!A1");
}

/// An unparseable formula has no references to rewrite; a rename must leave it
/// verbatim rather than failing (formula text is not validated on load).
#[test]
fn an_unparseable_formula_is_left_verbatim_by_a_rename() {
    let mut wb = wb_with(&["Data", "Report"]);
    set_formula(&mut wb, "Report", "A1", "=SUM(((");

    wb.rename_sheet("Data", "Facts").unwrap();

    assert_eq!(formula_at(&wb, "Report", "A1"), "=SUM(((");
}
