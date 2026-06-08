//! Sparse worksheet grid: get / set / clear / iterate keyed by a parsed
//! [`Address`] (plan item 3.1, schema spec §3–§4).

use truecalc_workbook::{Address, Cell, Value, Worksheet};

fn addr(key: &str) -> Address {
    Address::from_a1(key).unwrap()
}

#[test]
fn empty_grid_has_no_cells() {
    let ws = Worksheet::new("Sheet1");
    assert!(ws.is_empty());
    assert_eq!(ws.len(), 0);
    assert_eq!(ws.get(addr("A1")), None);
    assert!(!ws.contains(addr("A1")));
}

#[test]
fn set_then_get_round_trips() {
    let mut ws = Worksheet::new("Sheet1");
    let cell = Cell::literal(Value::Number(42.0)).unwrap();
    assert_eq!(ws.set(addr("B2"), cell.clone()), None);
    assert_eq!(ws.get(addr("B2")), Some(&cell));
    assert!(ws.contains(addr("B2")));
    assert_eq!(ws.len(), 1);
}

#[test]
fn set_returns_the_previous_cell() {
    let mut ws = Worksheet::new("Sheet1");
    let first = Cell::literal(Value::Number(1.0)).unwrap();
    let second = Cell::literal(Value::Number(2.0)).unwrap();
    ws.set(addr("C3"), first.clone());
    assert_eq!(ws.set(addr("C3"), second.clone()), Some(first));
    assert_eq!(ws.get(addr("C3")), Some(&second));
    assert_eq!(ws.len(), 1, "overwriting does not grow the grid");
}

#[test]
fn clear_removes_the_entry_and_returns_it() {
    let mut ws = Worksheet::new("Sheet1");
    let cell = Cell::literal(Value::Text("hi".to_owned())).unwrap();
    ws.set(addr("A1"), cell.clone());
    assert_eq!(ws.clear(addr("A1")), Some(cell));
    assert_eq!(
        ws.clear(addr("A1")),
        None,
        "clearing an absent cell is a no-op"
    );
    assert!(ws.is_empty());
}

#[test]
fn get_mut_edits_in_place() {
    let mut ws = Worksheet::new("Sheet1");
    ws.set(addr("A1"), Cell::with_formula("=1+1", Value::Empty));
    *ws.get_mut(addr("A1")).unwrap() = Cell::with_formula("=1+1", Value::Number(2.0));
    assert_eq!(ws.get(addr("A1")).unwrap().value(), &Value::Number(2.0));
}

#[test]
fn iter_yields_addresses_in_canonical_key_order() {
    let mut ws = Worksheet::new("Sheet1");
    // Insert out of order; JCS key order sorts "A10" before "A2".
    for key in ["A2", "A10", "A1", "B1"] {
        ws.set(addr(key), Cell::literal(Value::Number(1.0)).unwrap());
    }
    let keys: Vec<String> = ws.iter().map(|(a, _)| a.to_a1()).collect();
    assert_eq!(keys, vec!["A1", "A10", "A2", "B1"]);
}

#[test]
fn iter_addresses_match_their_cells() {
    let mut ws = Worksheet::new("Sheet1");
    ws.set(addr("ZZ100"), Cell::literal(Value::Number(7.0)).unwrap());
    let (a, cell) = ws.iter().next().unwrap();
    assert_eq!(a, addr("ZZ100"));
    assert_eq!(cell.value(), &Value::Number(7.0));
}
