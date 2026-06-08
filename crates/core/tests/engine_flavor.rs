//! Public API guarantees for the unified engine flavor enum (issue #567).
//!
//! `EngineFlavor` is the single canonical flavor type, owned by truecalc-core
//! and re-exported by truecalc-workbook. These tests guard the public surface
//! and the JSON wire strings pinned by the workbook JSON schema v1.

use truecalc_core::{Engine, EngineFlavor};

#[test]
fn engine_constructors_expose_their_flavor() {
    assert_eq!(Engine::sheets().flavor(), EngineFlavor::Sheets);
    assert_eq!(Engine::excel().flavor(), EngineFlavor::Excel);
}

/// Exhaustive-match guard: adding a variant to `EngineFlavor` forces this
/// match (and therefore the build) to be updated, so the enum cannot silently
/// grow in one place. With a single shared enum this is the cross-crate
/// drift protection requested in issue #567.
#[test]
fn flavor_variants_are_exhaustively_known() {
    for flavor in [EngineFlavor::Sheets, EngineFlavor::Excel] {
        match flavor {
            EngineFlavor::Sheets => {}
            EngineFlavor::Excel => {}
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn flavor_serializes_to_pinned_wire_strings() {
    // Pinned by workbook JSON schema v1 (spec section 2): must never change.
    assert_eq!(serde_json::to_string(&EngineFlavor::Sheets).unwrap(), "\"sheets\"");
    assert_eq!(serde_json::to_string(&EngineFlavor::Excel).unwrap(), "\"excel\"");
}

#[cfg(feature = "serde")]
#[test]
fn flavor_deserializes_from_pinned_wire_strings() {
    let s: EngineFlavor = serde_json::from_str("\"sheets\"").unwrap();
    let e: EngineFlavor = serde_json::from_str("\"excel\"").unwrap();
    assert_eq!(s, EngineFlavor::Sheets);
    assert_eq!(e, EngineFlavor::Excel);
    assert!(serde_json::from_str::<EngineFlavor>("\"lotus123\"").is_err());
}
