//! P1.3 resolver — trait, `evaluate_with_resolver`, and `extract_refs` (#525).
//!
//! Core stays the language: these tests supply a `Resolver` implementation
//! that owns workbook semantics (a fixed `Data` / `Quoted Name` sheet model
//! and a handful of defined names) and assert the engine evaluates
//! cross-sheet and named references through it. The model here is a *test*
//! resolver — it does not self-confirm Google Sheets values; the canonical
//! values live in the immutable `workbook.tsv` fixture and are exercised by
//! the conformance harness.

use std::collections::HashMap;
use truecalc_core::{extract_refs, CellAddr, Engine, ErrorKind, Ref, Resolver, Value};

/// Resolver over a fixed two-sheet workbook plus a few defined names, matching
/// the layout the `workbook.tsv` conformance fixture was authored against.
struct ModelResolver {
    cells: HashMap<(String, u32, u32), Value>,
    sheets: Vec<String>,
    names: HashMap<String, Ref>,
}

impl ModelResolver {
    fn new() -> Self {
        let mut cells = HashMap::new();
        let mut put = |sheet: &str, a1: &str, v: Value| {
            let addr = CellAddr::parse(a1).unwrap();
            cells.insert((sheet.to_string(), addr.col, addr.row), v);
        };
        put("Data", "A1", Value::Number(10.0));
        put("Data", "A2", Value::Number(20.0));
        put("Data", "A3", Value::Number(30.0));
        put("Data", "B1", Value::Number(5.0));
        put("Data", "B2", Value::Number(20.0));
        put("Data", "B3", Value::Number(40.0));
        put("Data", "C1", Value::Text("hello".to_string()));
        put("Data", "D1", Value::Bool(true));
        put("Data", "E1", Value::Date(46180.0));
        put("Quoted Name", "A1", Value::Number(100.0));
        put("Quoted Name", "A2", Value::Number(200.0));
        put("Quoted Name", "A3", Value::Number(300.0));
        put("Quoted Name", "B1", Value::Number(7.0));

        let mut names = HashMap::new();
        names.insert("TAX_RATE".to_string(), Ref::Cell {
            sheet: Some("Config".to_string()),
            addr: CellAddr::parse("A1").unwrap(),
        });
        names.insert("PRICES".to_string(), Ref::Range {
            sheet: Some("Data".to_string()),
            start: CellAddr::parse("A1").unwrap(),
            end: CellAddr::parse("A3").unwrap(),
        });
        names.insert("GRID".to_string(), Ref::Range {
            sheet: Some("Data".to_string()),
            start: CellAddr::parse("A1").unwrap(),
            end: CellAddr::parse("B1").unwrap(),
        });
        names.insert("QUOTED_VALS".to_string(), Ref::Range {
            sheet: Some("Quoted Name".to_string()),
            start: CellAddr::parse("A1").unwrap(),
            end: CellAddr::parse("A3").unwrap(),
        });
        cells.insert(("Config".to_string(), 1, 1), Value::Number(0.0825));

        Self {
            cells,
            sheets: vec![
                "Data".to_string(),
                "Quoted Name".to_string(),
                "Config".to_string(),
            ],
            names,
        }
    }

    fn sheet_exists(&self, name: &str) -> bool {
        self.sheets.iter().any(|s| s.eq_ignore_ascii_case(name))
    }

    fn canon_sheet(&self, name: &str) -> Option<&str> {
        self.sheets
            .iter()
            .find(|s| s.eq_ignore_ascii_case(name))
            .map(|s| s.as_str())
    }

    fn cell(&self, sheet: &str, addr: &CellAddr) -> Value {
        let Some(canon) = self.canon_sheet(sheet) else {
            return Value::Error(ErrorKind::Ref);
        };
        self.cells
            .get(&(canon.to_string(), addr.col, addr.row))
            .cloned()
            .unwrap_or(Value::Empty)
    }

    fn range(&self, sheet: &str, start: &CellAddr, end: &CellAddr) -> Value {
        if !self.sheet_exists(sheet) {
            return Value::Error(ErrorKind::Ref);
        }
        let mut out = Vec::new();
        for row in start.row..=end.row {
            for col in start.col..=end.col {
                let addr = CellAddr::new(col, row);
                out.push(self.cell(sheet, &addr));
            }
        }
        Value::Array(out)
    }
}

impl Resolver for ModelResolver {
    fn resolve(&mut self, r: &Ref) -> Value {
        match r {
            Ref::Cell { sheet: Some(s), addr } => self.cell(s, addr),
            Ref::Range { sheet: Some(s), start, end } => self.range(s, start, end),
            Ref::Cell { sheet: None, .. } | Ref::Range { sheet: None, .. } => Value::Empty,
            Ref::Name(name) => match self.names.get(&name.to_uppercase()).cloned() {
                Some(target) => self.resolve(&target),
                None => Value::Error(ErrorKind::Name),
            },
        }
    }
}

fn eval(formula: &str) -> Value {
    let mut r = ModelResolver::new();
    Engine::sheets().evaluate_with_resolver(formula, &mut r)
}

#[test]
fn cross_sheet_cell_to_number() {
    assert_eq!(eval("=Data!A1"), Value::Number(10.0));
}

#[test]
fn cross_sheet_cells_in_arithmetic() {
    assert_eq!(eval("=Data!A1+Data!B1"), Value::Number(15.0));
}

#[test]
fn cross_sheet_lowercase_sheet_name() {
    assert_eq!(eval("=data!A1"), Value::Number(10.0));
}

#[test]
fn cross_sheet_text_cell() {
    assert_eq!(eval("=Data!C1"), Value::Text("hello".to_string()));
}

#[test]
fn cross_sheet_boolean_cell() {
    assert_eq!(eval("=Data!D1"), Value::Bool(true));
}

#[test]
fn cross_sheet_date_cell_is_date_typed() {
    assert_eq!(eval("=Data!E1"), Value::Date(46180.0));
}

#[test]
fn date_typed_cell_stays_date_through_arithmetic() {
    // A cell resolved as Date (E1) carries its type through offset arithmetic,
    // so a host can store a serial as a Date and rely on the engine to keep it
    // rendering as a date (issue #721).
    assert_eq!(eval("=Data!E1+1"), Value::Date(46181.0));
    assert_eq!(eval("=Data!E1-7"), Value::Date(46173.0));
    // Two Date cells subtracted are a plain day count.
    assert_eq!(eval("=Data!E1-Data!E1"), Value::Number(0.0));
}

#[test]
fn cross_sheet_empty_cell_concats_as_blank() {
    assert_eq!(eval("=\"<\"&Data!H9&\">\""), Value::Text("<>".to_string()));
}

#[test]
fn quoted_sheet_name_cell() {
    assert_eq!(eval("='Quoted Name'!A1"), Value::Number(100.0));
}

#[test]
fn quoted_sheet_name_arithmetic() {
    assert_eq!(eval("='Quoted Name'!A1+'Quoted Name'!B1"), Value::Number(107.0));
}

#[test]
fn cross_sheet_column_range_sum() {
    assert_eq!(eval("=SUM(Data!A1:A3)"), Value::Number(60.0));
}

#[test]
fn cross_sheet_rectangular_range_sum() {
    assert_eq!(eval("=SUM(Data!A1:B3)"), Value::Number(125.0));
}

#[test]
fn cross_sheet_range_average() {
    assert_eq!(eval("=AVERAGE(Data!A1:A3)"), Value::Number(20.0));
}

#[test]
fn cross_sheet_count_matches_bare_range() {
    // The resolver delivers the same materialized array a bare range would, so
    // COUNT over the cross-sheet range equals COUNT over the equivalent bare
    // range bound as a variable. (We assert wiring, not COUNT semantics — the
    // canonical COUNT value lives in the immutable workbook.tsv fixture.)
    let bare = Value::Array(vec![
        Value::Number(10.0), Value::Number(5.0), Value::Text("hello".to_string()), Value::Bool(true),
        Value::Number(20.0), Value::Number(20.0), Value::Empty, Value::Empty,
        Value::Number(30.0), Value::Number(40.0), Value::Empty, Value::Empty,
    ]);
    let mut vars = HashMap::new();
    vars.insert("A1:D3".to_string(), bare);
    let via_bare = Engine::sheets().evaluate("=COUNT(A1:D3)", &vars);
    assert_eq!(eval("=COUNT(Data!A1:D3)"), via_bare);
}

#[test]
fn quoted_sheet_range_sum() {
    assert_eq!(eval("=SUM('Quoted Name'!A1:A3)"), Value::Number(600.0));
}

#[test]
fn cross_sheet_ref_in_if() {
    assert_eq!(eval("=IF(Data!A1>5,\"big\",\"small\")"), Value::Text("big".to_string()));
}

#[test]
fn missing_sheet_cell_is_ref_error() {
    assert_eq!(eval("=MissingSheet!A1"), Value::Error(ErrorKind::Ref));
}

#[test]
fn missing_quoted_sheet_is_ref_error() {
    assert_eq!(eval("='No Such Sheet'!A1"), Value::Error(ErrorKind::Ref));
}

#[test]
fn missing_sheet_range_is_ref_error() {
    assert_eq!(eval("=SUM(MissingSheet!A1:A3)"), Value::Error(ErrorKind::Ref));
}

#[test]
fn missing_sheet_wrapped_in_iferror() {
    assert_eq!(eval("=IFERROR(MissingSheet!A1,\"fallback\")"), Value::Text("fallback".to_string()));
}

#[test]
fn scalar_named_range() {
    assert_eq!(eval("=TAX_RATE"), Value::Number(0.0825));
}

#[test]
fn scalar_named_range_arithmetic() {
    assert_eq!(eval("=TAX_RATE*100"), Value::Number(8.25));
}

#[test]
fn scalar_named_range_lowercase() {
    assert_eq!(eval("=tax_rate"), Value::Number(0.0825));
}

#[test]
fn column_range_name_sum() {
    assert_eq!(eval("=SUM(PRICES)"), Value::Number(60.0));
}

#[test]
fn column_range_name_max() {
    assert_eq!(eval("=MAX(PRICES)"), Value::Number(30.0));
}

#[test]
fn rectangular_range_name_sum() {
    assert_eq!(eval("=SUM(GRID)"), Value::Number(15.0));
}

#[test]
fn named_range_targeting_quoted_sheet() {
    assert_eq!(eval("=SUM(QUOTED_VALS)"), Value::Number(600.0));
}

#[test]
fn missing_name_is_name_error() {
    assert_eq!(eval("=NOT_A_DEFINED_NAME"), Value::Error(ErrorKind::Name));
}

#[test]
fn missing_name_inside_sum() {
    assert_eq!(eval("=SUM(NOT_A_DEFINED_NAME)"), Value::Error(ErrorKind::Name));
}

#[test]
fn missing_name_wrapped_in_iferror() {
    assert_eq!(eval("=IFERROR(NOT_A_DEFINED_NAME,\"fallback\")"), Value::Text("fallback".to_string()));
}

#[test]
fn lambda_parameter_shadows_resolver_name() {
    assert_eq!(eval("=LAMBDA(X, X*2)(21)"), Value::Number(42.0));
}

#[test]
fn extract_refs_finds_refs_in_every_position() {
    let expr = Engine::sheets()
        .parse("=SUM(A1, Data!B2, TAX_RATE, 'Quoted Name'!A1:A3)")
        .unwrap();
    assert_eq!(
        extract_refs(&expr),
        vec![
            Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) },
            Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(2, 2) },
            Ref::Name("TAX_RATE".to_string()),
            Ref::Range {
                sheet: Some("Quoted Name".to_string()),
                start: CellAddr::new(1, 1),
                end: CellAddr::new(1, 3),
            },
        ],
    );
}

#[test]
fn extract_refs_classifies_bare_identifiers() {
    let expr = Engine::sheets().parse("=A1:D4 + B7 - MYNAME").unwrap();
    assert_eq!(
        extract_refs(&expr),
        vec![
            Ref::Range {
                sheet: None,
                start: CellAddr::new(1, 1),
                end: CellAddr::new(4, 4),
            },
            Ref::Cell { sheet: None, addr: CellAddr::new(2, 7) },
            Ref::Name("MYNAME".to_string()),
        ],
    );
}

#[test]
fn extract_refs_preserves_duplicates() {
    let expr = Engine::sheets().parse("=A1+A1").unwrap();
    let a1 = Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) };
    assert_eq!(extract_refs(&expr), vec![a1.clone(), a1]);
}

#[test]
fn extract_refs_descends_into_nested_calls_and_arrays() {
    let expr = Engine::sheets().parse("=IF(A1>0, {B2, C3}, Data!D4)").unwrap();
    assert_eq!(
        extract_refs(&expr),
        vec![
            Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) },
            Ref::Cell { sheet: None, addr: CellAddr::new(2, 2) },
            Ref::Cell { sheet: None, addr: CellAddr::new(3, 3) },
            Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(4, 4) },
        ],
    );
}

#[test]
fn extract_refs_ignores_literals() {
    let expr = Engine::sheets().parse("=1 + 2 * 3").unwrap();
    assert!(extract_refs(&expr).is_empty());
}

#[test]
fn evaluate_with_resolver_at_pins_today() {
    let mut r = ModelResolver::new();
    let v = Engine::sheets().evaluate_with_resolver_at("=TODAY()", &mut r, Some(46180.0));
    assert_eq!(v, Value::Date(46180.0));
}

#[test]
fn evaluate_with_resolver_at_rejects_non_finite_now() {
    let mut r = ModelResolver::new();
    let v = Engine::sheets().evaluate_with_resolver_at("=1", &mut r, Some(f64::NAN));
    assert_eq!(v, Value::Error(ErrorKind::Num));
}

#[test]
fn excel_evaluate_with_resolver_is_unsupported() {
    let mut r = ModelResolver::new();
    let v = Engine::excel().evaluate_with_resolver("=Data!A1", &mut r);
    assert_eq!(v, Value::Error(ErrorKind::Unsupported));
}
