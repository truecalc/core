//! Spill serialization invariants (plan item 3.5, issue #537; schema spec §5):
//! a spilled cell is **never serialized** — only the anchor stores its array.
//! This guards the canonical-byte contract (§8): the wire form of a spilling
//! workbook is identical whether or not the spill has been recalculated into a
//! grid, and a `from_json` of an anchor-only document reconstructs the same
//! spill on recalc.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is valid")
}

fn spilled_wb() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());
    wb
}

#[test]
fn a_spilling_workbook_serializes_only_the_anchor_array() {
    let json = spilled_wb().to_json().unwrap();

    // The anchor cell is present with its array value.
    assert!(json.contains("\"A1\""), "anchor key present");
    assert!(json.contains("\"type\":\"array\""), "anchor array present");

    // No spilled cell key appears in the serialized output (§5): B1, A2, B2 are
    // reconstructed, never written.
    assert!(
        !json.contains("\"B1\""),
        "spilled B1 must not be serialized"
    );
    assert!(
        !json.contains("\"A2\""),
        "spilled A2 must not be serialized"
    );
    assert!(
        !json.contains("\"B2\""),
        "spilled B2 must not be serialized"
    );
}

#[test]
fn spilling_workbook_round_trips_byte_identically() {
    let wb = spilled_wb();
    let json = wb.to_json().unwrap();
    let back = Workbook::from_json(json.as_bytes()).unwrap();
    // to_json ∘ from_json = id, byte-for-byte (§8).
    assert_eq!(back.to_json().unwrap(), json);
    // And structurally only the anchor exists.
    assert_eq!(back.sheet("Sheet1").unwrap().len(), 1);
}

#[test]
fn from_json_of_anchor_only_doc_reconstructs_the_spill_on_recalc() {
    // Serialize a spilling workbook (anchor-only), reload, and recalc: the same
    // spill is reconstructed (§5), proving the spilled cells are pure derived
    // state recoverable from the anchor alone.
    let json = spilled_wb().to_json().unwrap();
    let mut reloaded = Workbook::from_json(json.as_bytes()).unwrap();

    // Before recalc, the spilled cells already resolve (the anchor array is on
    // the grid; `resolved` reconstructs from it).
    assert_eq!(
        reloaded.resolved("Sheet1", a1("B2")).unwrap().value,
        Value::Number(4.0)
    );
    assert_eq!(reloaded.spill_anchor("Sheet1", a1("B2")), Some(a1("A1")));

    // A fresh recalc is a no-op for the byte stream (deterministic).
    reloaded.recalc(&ctx());
    assert_eq!(reloaded.to_json().unwrap(), json);
}

#[test]
fn a_blocked_spill_serializes_the_error_at_the_anchor_and_no_array() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    // D2 authored blocks C1's {1,2;3,4} spill into C1:D2.
    wb.set("Sheet1", a1("D2"), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());

    let json = wb.to_json().unwrap();
    // The anchor carries the blocked-spill error, not an array (§5/§12).
    assert!(
        json.contains("\"type\":\"error\""),
        "blocked anchor is an error"
    );
    assert!(
        !json.contains("\"type\":\"array\""),
        "blocked spill stores no array"
    );
    // The obstructing literal is still serialized.
    assert!(json.contains("\"D2\""));
    // Round-trips byte-identically.
    assert_eq!(
        Workbook::from_json(json.as_bytes())
            .unwrap()
            .to_json()
            .unwrap(),
        json
    );
}
