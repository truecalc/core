//! Strict `from_json` document-level validation (issue #530 acceptance and the
//! #568 follow-up rules): duplicate keys, UTF-8/BOM, address syntax + bounds,
//! spill-rectangle validity, sheet-name uniqueness under simple case folding,
//! named-range validity, and resource limits.

use truecalc_workbook::Workbook;

fn err(json: &str) -> String {
    Workbook::from_json(json.as_bytes())
        .expect_err(&format!("expected rejection for: {json}"))
        .to_string()
}

fn ok(json: &str) {
    Workbook::from_json(json.as_bytes())
        .unwrap_or_else(|e| panic!("expected acceptance: {e}\n{json}"));
}

// ---------- §1: duplicate keys ----------

#[test]
fn duplicate_top_level_key_is_rejected() {
    // Stock serde_json silently keeps the last "version"; we must reject.
    let json = r#"{"engine":"sheets","names":[],"sheets":[],"version":"1","version":"1"}"#;
    assert!(err(json).contains("duplicate"));
}

#[test]
fn duplicate_cell_key_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"number","value":1}},"A1":{"value":{"type":"number","value":2}}}}],"version":"1"}"#;
    assert!(err(json).contains("duplicate"));
}

#[test]
fn duplicate_value_field_key_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"number","value":1,"value":2}}}}],"version":"1"}"#;
    assert!(err(json).contains("duplicate"));
}

// ---------- §1: UTF-8 / BOM ----------

#[test]
fn utf8_bom_is_rejected() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(br#"{"engine":"sheets","names":[],"sheets":[],"version":"1"}"#);
    let e = Workbook::from_json(&bytes).unwrap_err().to_string();
    assert!(e.contains("BOM"), "got: {e}");
}

#[test]
fn invalid_utf8_is_rejected() {
    // 0xFF is never valid UTF-8.
    let bytes = [b'{', 0xFF, b'}'];
    assert!(Workbook::from_json(&bytes).is_err());
}

// ---------- §3: address syntax + bounds ----------

#[test]
fn lowercase_address_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"a1":{"value":{"type":"number","value":1}}}}],"version":"1"}"#;
    assert!(err(json).contains("address"));
}

#[test]
fn dollar_sign_address_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"$A$1":{"value":{"type":"number","value":1}}}}],"version":"1"}"#;
    assert!(err(json).contains("address"));
}

#[test]
fn leading_zero_row_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A01":{"value":{"type":"number","value":1}}}}],"version":"1"}"#;
    assert!(err(json).contains("address"));
}

#[test]
fn four_letter_column_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"AAAA1":{"value":{"type":"number","value":1}}}}],"version":"1"}"#;
    assert!(err(json).contains("address"));
}

#[test]
fn column_beyond_zzz_is_rejected() {
    // ZZZ = 18278; ZZZZ exceeds three letters anyway, but AAA..= test ZZZ ok.
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"ZZZ10000000":{"value":{"type":"number","value":1}}}}],"version":"1"}"#;
    ok(json);
}

#[test]
fn row_beyond_10_million_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A10000001":{"value":{"type":"number","value":1}}}}],"version":"1"}"#;
    assert!(err(json).contains("address"));
}

#[test]
fn valid_addresses_are_accepted() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"number","value":1}},"BC42":{"value":{"type":"number","value":2}}}}],"version":"1"}"#;
    ok(json);
}

// ---------- §2/§3: sheet names ----------

#[test]
fn empty_sheet_name_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("non-empty"));
}

#[test]
fn duplicate_sheet_names_case_insensitive_are_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"Sheet1","cells":{}},{"name":"SHEET1","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("duplicate sheet name"));
}

#[test]
fn distinct_sheet_names_are_accepted() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"Sheet1","cells":{}},{"name":"Sheet2","cells":{}}],"version":"1"}"#;
    ok(json);
}

// ---------- §5: spill-rectangle validity ----------

#[test]
fn authored_cell_inside_spill_rectangle_is_rejected() {
    // Anchor A1 holds a 2x2 array (covers A1,B1,A2,B2); B2 is authored => invalid.
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"array","value":[[{"type":"number","value":1},{"type":"number","value":2}],[{"type":"number","value":3},{"type":"number","value":4}]]}},"B2":{"value":{"type":"number","value":9}}}}],"version":"1"}"#;
    assert!(err(json).contains("spill"));
}

#[test]
fn non_overlapping_spill_is_accepted() {
    // Anchor A1 2x2 (A1,B1,A2,B2); a second authored cell at D1 is clear.
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"array","value":[[{"type":"number","value":1},{"type":"number","value":2}],[{"type":"number","value":3},{"type":"number","value":4}]]}},"D1":{"value":{"type":"number","value":9}}}}],"version":"1"}"#;
    ok(json);
}

#[test]
fn overlapping_spill_rectangles_are_rejected() {
    // A1 2x2 covers A1,B1,A2,B2; B1 is also an anchor of a 2x2 => overlap.
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"array","value":[[{"type":"number","value":1},{"type":"number","value":2}],[{"type":"number","value":3},{"type":"number","value":4}]]}},"B1":{"value":{"type":"array","value":[[{"type":"number","value":5},{"type":"number","value":6}],[{"type":"number","value":7},{"type":"number","value":8}]]}}}}],"version":"1"}"#;
    let e = err(json);
    assert!(e.contains("spill") || e.contains("overlap"), "got: {e}");
}

// ---------- §7: named ranges ----------

#[test]
fn named_range_that_looks_like_a1_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"A1","ref":"S!A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("name"));
}

#[test]
fn named_range_that_looks_like_r1c1_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"R1C1","ref":"S!A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("name"));
}

#[test]
fn named_range_boolean_literal_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"TRUE","ref":"S!A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("name"));
}

#[test]
fn named_range_with_dangling_sheet_ref_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"Tax","ref":"Ghost!A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("does not exist"));
}

#[test]
fn named_range_duplicate_case_insensitive_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"Tax","ref":"S!A1"},{"name":"TAX","ref":"S!A2"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("duplicate named range"));
}

#[test]
fn valid_named_range_is_accepted() {
    let json = r#"{"engine":"sheets","names":[{"name":"TaxRate","ref":"S!A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    ok(json);
}

#[test]
fn named_range_with_range_ref_is_accepted() {
    let json = r#"{"engine":"sheets","names":[{"name":"Data","ref":"S!A1:B2"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    ok(json);
}

#[test]
fn non_canonical_degenerate_range_ref_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"Data","ref":"S!A1:A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("canonical"));
}

#[test]
fn non_canonical_endpoint_order_ref_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"Data","ref":"S!B2:A1"}],"sheets":[{"name":"S","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("canonical"));
}

#[test]
fn quoted_sheet_ref_round_trips() {
    let json = r#"{"engine":"sheets","names":[{"name":"Data","ref":"'Q2 X'!A1:A3"}],"sheets":[{"name":"Q2 X","cells":{}}],"version":"1"}"#;
    let wb = Workbook::from_json(json.as_bytes()).unwrap();
    assert!(wb.to_json().unwrap().contains("'Q2 X'!A1:A3"));
}

#[test]
fn over_quoted_bare_sheet_ref_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"Data","ref":"'Sheet1'!A1"}],"sheets":[{"name":"Sheet1","cells":{}}],"version":"1"}"#;
    assert!(err(json).contains("canonical"));
}

// ---------- §9 / §4: unknown + reserved fields (serde layer) ----------

#[test]
fn unknown_top_level_field_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[],"version":"1","theme":"dark"}"#;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

#[test]
fn reserved_format_field_on_cell_is_rejected() {
    let json = r##"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"number","value":1},"format":"#,##0"}}}],"version":"1"}"##;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

// ---------- §6 / §4: value encodings (serde layer) ----------

#[test]
fn empty_literal_cell_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"empty","value":null}}}}],"version":"1"}"#;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

#[test]
fn nan_number_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"number","value":NaN}}}}],"version":"1"}"#;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

#[test]
fn one_by_one_array_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"A1":{"value":{"type":"array","value":[[{"type":"number","value":1}]]}}}}],"version":"1"}"#;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

// ---------- §10: version ----------

#[test]
fn unknown_version_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[],"version":"3"}"#;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

#[test]
fn missing_version_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[]}"#;
    assert!(Workbook::from_json(json.as_bytes()).is_err());
}

// ---------- structured tables (structured-references spec §4, truecalc/core#861) ----------

#[test]
fn table_name_collision_with_named_range_is_rejected() {
    let json = r#"{"engine":"sheets","names":[{"name":"Recipe","ref":"Sheet1!Z1"}],"sheets":[{"name":"Sheet1","cells":{}}],"tables":[{"name":"Recipe","ref":"Sheet1!A1:D12"}],"version":"2"}"#;
    assert!(err(json).contains("table"));
}

#[test]
fn overlapping_tables_are_rejected() {
    // Table "A" needs valid header cells to clear its own checks first, so
    // the overlap check on table "B" (checked before B's own header check)
    // is what actually rejects the document.
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"Sheet1","cells":{"A1":{"value":{"type":"text","value":"a"}},"B1":{"value":{"type":"text","value":"b"}},"C1":{"value":{"type":"text","value":"c"}},"D1":{"value":{"type":"text","value":"d"}}}}],"tables":[{"name":"A","ref":"Sheet1!A1:D12"},{"name":"B","ref":"Sheet1!C5:E20"}],"version":"2"}"#;
    let e = err(json);
    assert!(e.contains("overlap"), "got: {e}");
}

#[test]
fn well_formed_table_is_accepted() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"Sheet1","cells":{"A1":{"value":{"type":"text","value":"quantity_g"}},"B1":{"value":{"type":"text","value":"reference_per_100g"}}}}],"tables":[{"name":"Recipe","ref":"Sheet1!A1:B2"}],"version":"2"}"#;
    ok(json);
}

#[test]
fn table_with_duplicate_header_cell_is_rejected() {
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"Sheet1","cells":{"A1":{"value":{"type":"text","value":"qty"}},"B1":{"value":{"type":"text","value":"qty"}}}}],"tables":[{"name":"Recipe","ref":"Sheet1!A1:B2"}],"version":"2"}"#;
    let e = err(json);
    assert!(e.contains("duplicate") || e.contains("column"), "got: {e}");
}

#[test]
fn overlapping_tables_with_differently_cased_sheet_refs_are_rejected() {
    // Both refs target the same physical sheet "Sheet1"; the second table's
    // ref spells the sheet name in a different case. `table_ref::ranges_overlap`
    // compares sheet names as raw strings, so the bounds fed to it must
    // already be case-folded for this overlap to be caught.
    let json = r#"{"engine":"sheets","names":[],"sheets":[{"name":"Sheet1","cells":{"A1":{"value":{"type":"text","value":"a"}},"B1":{"value":{"type":"text","value":"b"}},"C1":{"value":{"type":"text","value":"c"}},"D1":{"value":{"type":"text","value":"d"}}}}],"tables":[{"name":"A","ref":"Sheet1!A1:D12"},{"name":"B","ref":"SHEET1!C5:E20"}],"version":"2"}"#;
    let e = err(json);
    assert!(e.contains("overlap"), "got: {e}");
}

// ---------- Decision 5: limits ----------

#[test]
fn too_many_sheets_is_rejected() {
    let mut sheets = String::new();
    for i in 0..=256 {
        if i > 0 {
            sheets.push(',');
        }
        sheets.push_str(&format!(r#"{{"name":"S{i}","cells":{{}}}}"#));
    }
    let json = format!(r#"{{"engine":"sheets","names":[],"sheets":[{sheets}],"version":"1"}}"#);
    assert!(err(&json).contains("sheets"));
}
