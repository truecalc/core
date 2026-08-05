use super::*;

#[test]
fn parse_relative() {
    let a = CellAddr::parse("A1").unwrap();
    assert_eq!(a, CellAddr::new(1, 1));
    assert!(!a.col_abs);
    assert!(!a.row_abs);
}

#[test]
fn parse_col_absolute() {
    let a = CellAddr::parse("$A1").unwrap();
    assert_eq!(a, CellAddr::new(1, 1).with_col_abs(true));
}

#[test]
fn parse_row_absolute() {
    let a = CellAddr::parse("A$1").unwrap();
    assert_eq!(a, CellAddr::new(1, 1).with_row_abs(true));
}

#[test]
fn parse_both_absolute() {
    let a = CellAddr::parse("$A$1").unwrap();
    assert_eq!(a, CellAddr::new(1, 1).with_col_abs(true).with_row_abs(true));
}

#[test]
fn display_round_trips_all_four_combinations() {
    for (text, col_abs, row_abs) in [
        ("A1", false, false),
        ("$A1", true, false),
        ("A$1", false, true),
        ("$A$1", true, true),
    ] {
        let a = CellAddr::parse(text).unwrap_or_else(|| panic!("{text} should parse"));
        assert_eq!(a.col_abs, col_abs, "{text} col_abs");
        assert_eq!(a.row_abs, row_abs, "{text} row_abs");
        assert_eq!(a.to_string(), text, "{text} display round-trip");
    }
}

#[test]
fn parse_rejects_malformed_dollar_shapes() {
    for text in ["$", "$$A1", "$1A", "A$$1", "$A$", "A$"] {
        assert!(CellAddr::parse(text).is_none(), "{text:?} should not parse");
    }
}

#[test]
fn parse_still_rejects_row_zero_and_non_addresses() {
    for text in ["", "A", "1", "A0", "$A0", "A$0", "1A", "A1B", "A 1", "A1.5"] {
        assert!(CellAddr::parse(text).is_none(), "{text:?} should not parse");
    }
}

#[test]
fn relative_display_strips_dollar_anchors() {
    let r = Ref::Cell {
        sheet: Some("Data".to_string()),
        addr: CellAddr::new(1, 1).with_col_abs(true).with_row_abs(true),
    };
    assert_eq!(r.relative_display(), "Data!A1");
    assert_eq!(r.to_string(), "Data!$A$1");
}

#[test]
fn relative_display_range_strips_dollar_anchors_per_corner() {
    let r = Ref::Range {
        sheet: None,
        start: CellAddr::new(1, 1).with_col_abs(true),
        end: CellAddr::new(4, 4).with_row_abs(true),
    };
    assert_eq!(r.relative_display(), "A1:D4");
    assert_eq!(r.to_string(), "$A1:D$4");
}

#[test]
fn relative_display_name_is_unchanged() {
    assert_eq!(Ref::Name("TAX_RATE".to_string()).relative_display(), "TAX_RATE");
}
