//! `list_functions()` must report the real registry, not a curated copy of it.
//!
//! It previously returned a hand-written 64-entry `json!` literal while the
//! engine implemented 518 functions (core#810), so npm and JSR consumers saw
//! roughly 12% of the catalogue — and every function added after the literal was
//! written was invisible to them, silently and forever.
//!
//! What these tests guarantee is that the two cannot *drift*: they compare
//! against the registry at runtime, so the next function added to the registry
//! fails them unless the JS surface picks it up too. They do not prove the
//! implementation is not a literal — a hand-written list that happens to be in
//! sync today would pass — only that it cannot stay wrong.

use truecalc_core::Registry;
use truecalc_wasm::list_functions;

#[test]
fn reports_exactly_the_registered_functions() {
    let registry = Registry::new();
    let mut expected: Vec<String> = registry.metadata_names();
    expected.sort();

    let actual: Vec<String> = list_functions().into_iter().map(|f| f.name).collect();

    // Compare the full name vectors, not just their lengths: equal counts plus
    // "every returned name exists in the registry" would still pass if one entry
    // were duplicated while another went missing. This also subsumes the
    // sortedness check, since `expected` is sorted.
    assert_eq!(
        actual, expected,
        "list_functions() does not match the registry. If this fails after \
         adding a function, the JS surface has stopped deriving from the \
         registry — do not 'fix' it by editing a list here."
    );
    // Guard against both sides being trivially empty.
    assert!(expected.len() > 400, "registry unexpectedly small: {}", expected.len());
}

#[test]
fn every_entry_carries_its_registry_metadata() {
    let registry = Registry::new();
    let meta: std::collections::HashMap<_, _> = registry
        .get_metadata()
        .into_iter()
        .map(|e| (e.name.to_string(), e.meta.clone()))
        .collect();

    for info in list_functions() {
        let m = meta
            .get(&info.name)
            .unwrap_or_else(|| panic!("{} is not in the registry", info.name));
        assert_eq!(info.category, m.category, "category drift on {}", info.name);
        // The JS field is `syntax`; the registry calls it `signature`. The
        // rename is deliberately deferred (it breaks every npm consumer), so
        // the mapping is pinned here instead.
        assert_eq!(info.syntax, m.signature, "signature drift on {}", info.name);
        assert_eq!(
            info.description, m.description,
            "description drift on {}",
            info.name
        );
    }
}

#[test]
fn order_is_stable() {
    // The registry is a HashMap, so iteration order varies between calls within
    // one process. Unsorted output would make the array order differ call to
    // call, breaking snapshot tests and any consumer rendering a list.
    let first: Vec<String> = list_functions().into_iter().map(|f| f.name).collect();
    let second: Vec<String> = list_functions().into_iter().map(|f| f.name).collect();
    assert_eq!(first, second, "list_functions() order is not deterministic");

    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(first, sorted, "list_functions() is not sorted by name");
}
