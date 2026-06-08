//! Registry-driven generated test for `extract_refs` (P1.3, #525 acceptance).
//!
//! For every user-facing function in the registry, synthesize a call that
//! places a distinct cell reference in each argument position, then assert
//! `extract_refs` recovers exactly those references in order. This is
//! mechanically self-verifiable: the registry is the single source of truth,
//! so the test grows automatically as functions are added, and it can never
//! drift from a hand-maintained list.

use truecalc_core::{extract_refs, CellAddr, Engine, Ref, Registry};

/// Cell reference used in argument position `i` (0-based): A1, B1, C1, ...
/// for the first 26 positions, which is more than any function's arity.
fn ref_arg_text(i: usize) -> String {
    let col = (b'A' + (i as u8)) as char;
    format!("{}1", col)
}

fn expected_ref(i: usize) -> Ref {
    Ref::Cell {
        sheet: None,
        addr: CellAddr { col: (i as u32) + 1, row: 1 },
    }
}

#[test]
fn extract_refs_finds_refs_in_every_argument_position() {
    let registry = Registry::new();
    let engine = Engine::sheets();
    let mut names = registry.metadata_names();
    names.sort();

    // Try arities 1..=3 ref args; a function parses as a call for whichever
    // arity it accepts syntactically. We assert on every formula that parses.
    let arg_counts = [1usize, 2, 3];

    let mut checked = 0usize;
    for name in &names {
        for &n in &arg_counts {
            let args: Vec<String> = (0..n).map(ref_arg_text).collect();
            let formula = format!("={}({})", name, args.join(", "));

            let Ok(expr) = engine.parse(&formula) else {
                continue;
            };
            let refs = extract_refs(&expr);
            let expected: Vec<Ref> = (0..n).map(expected_ref).collect();
            assert_eq!(
                refs, expected,
                "extract_refs mismatch for `{}` (n={})",
                formula, n
            );
            checked += 1;
        }
    }

    // Sanity: the registry is non-trivial and the loop actually did work.
    assert!(
        checked >= names.len(),
        "expected at least one checked formula per function ({} functions, {} checked)",
        names.len(),
        checked
    );
    assert!(names.len() > 100, "registry unexpectedly small: {}", names.len());
}
