//! A1 ↔ `(row, column)` conversion and address-bounds tests (plan item 3.1,
//! schema spec §3, limits ADR Decision 5).

use truecalc_workbook::Address;

#[test]
fn a1_round_trips_through_address() {
    for key in [
        "A1",
        "B2",
        "Z9",
        "AA1",
        "AZ10",
        "BA100",
        "ZZ65536",
        "ZZZ10000000",
    ] {
        let addr = Address::from_a1(key).unwrap_or_else(|| panic!("{key} should parse"));
        assert_eq!(addr.to_a1(), key, "{key} should round-trip");
    }
}

#[test]
fn column_letters_map_to_indices() {
    assert_eq!(Address::from_a1("A1").unwrap().column, 1);
    assert_eq!(Address::from_a1("Z1").unwrap().column, 26);
    assert_eq!(Address::from_a1("AA1").unwrap().column, 27);
    assert_eq!(Address::from_a1("AZ1").unwrap().column, 52);
    assert_eq!(Address::from_a1("BA1").unwrap().column, 53);
    // ZZZ is the maximum in-bounds column (18,278).
    assert_eq!(Address::from_a1("ZZZ1").unwrap().column, 18_278);
}

#[test]
fn new_is_the_inverse_of_to_a1() {
    let addr = Address::new(42, 53).unwrap();
    assert_eq!(addr.to_a1(), "BA42");
    assert_eq!(Address::from_a1("BA42"), Some(addr));
}

#[test]
fn rows_at_the_bounds() {
    assert!(Address::new(1, 1).is_some());
    assert!(Address::new(10_000_000, 1).is_some());
    assert!(Address::from_a1("A10000000").is_some());
    // Row 0 and beyond the 10,000,000 cap are out of bounds.
    assert!(Address::new(0, 1).is_none());
    assert!(Address::new(10_000_001, 1).is_none());
    assert!(Address::from_a1("A0").is_none());
    assert!(Address::from_a1("A10000001").is_none());
}

#[test]
fn columns_at_the_bounds() {
    assert!(Address::new(1, 18_278).is_some());
    assert!(Address::new(1, 0).is_none());
    assert!(Address::new(1, 18_279).is_none());
    // Four letters always exceed the bound and the 3-letter key grammar.
    assert!(Address::from_a1("AAAA1").is_none());
}

#[test]
fn malformed_keys_are_rejected() {
    for bad in [
        "",
        "1",
        "A",
        "a1",
        "$A$1",
        "A1:B2",
        "Sheet1!A1",
        "A01",
        "A 1",
        "A1 ",
        " A1",
        "A1.0",
    ] {
        assert!(Address::from_a1(bad).is_none(), "{bad:?} must not parse");
    }
}
