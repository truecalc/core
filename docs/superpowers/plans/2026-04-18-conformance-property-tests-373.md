# Conformance-Driven Property Tests

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add property tests that verify `truecalc(formula) ≈ Google Sheets oracle` across *generated* inputs for each function category — not just the fixed fixture rows. This catches regressions in functions that have oracle fixtures but limited hand-written edge cases.

**Architecture:** A new `property_conformance.rs` integration test file. For each function category (math, text, logical, financial), proptest generates valid inputs and evaluates the formula through truecalc, then asserts the result matches the *mathematical property* that the Google Sheets oracle implies (e.g. ABS always non-negative, ROUND precision, IF branching). This is not a live GS call — the properties are derived from the oracle behavior we've already confirmed in conformance tests.

**Tech Stack:** proptest 1.x, truecalc_core (evaluate, Value, ErrorKind)

**GitHub issue:** Closes #373 (sub-issue of epic #366)

---

## File Map

| Action | File | Purpose |
|--------|------|---------|
| Create | `crates/core/tests/property_conformance.rs` | Conformance-driven property tests per milestone |

---

## Task 1: Create `property_conformance.rs`

**Files:**
- Create: `crates/core/tests/property_conformance.rs`

- [ ] **Step 1: Write the file**

```rust
// crates/core/tests/property_conformance.rs
//
// Conformance-driven property tests: for each function confirmed by Google Sheets
// oracle fixtures, verify that the mathematical property holds across generated inputs.
//
// These tests complement the fixed-row conformance tests in conformance.rs.
// They catch regressions for inputs that don't appear in the fixture files.

use truecalc_core::{evaluate, Value};
use proptest::prelude::*;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn run_n(formula: &str, var: &str, n: f64) -> Value {
    let mut m = HashMap::new();
    m.insert(var.to_string(), Value::Number(n));
    evaluate(formula, &m)
}

fn run_s(formula: &str, var: &str, s: &str) -> Value {
    let mut m = HashMap::new();
    m.insert(var.to_string(), Value::Text(s.to_string()));
    evaluate(formula, &m)
}

fn run_two(formula: &str, a: (&str, f64), b: (&str, f64)) -> Value {
    let mut m = HashMap::new();
    m.insert(a.0.to_string(), Value::Number(a.1));
    m.insert(b.0.to_string(), Value::Number(b.1));
    evaluate(formula, &m)
}

fn small_f64() -> impl Strategy<Value = f64> { -1e6f64..1e6f64 }
fn pos_f64()   -> impl Strategy<Value = f64> { 1e-6f64..1e6f64 }
fn ascii_str() -> impl Strategy<Value = String> { "[a-z]{0,20}" }

// ─── M1: Math ────────────────────────────────────────────────────────────────

proptest! {
    // ABS: confirmed by m1/Math.xlsx — result is always >= 0
    #[test]
    fn m1_abs_non_negative(x in small_f64()) {
        let r = run_n("=ABS(x)", "x", x);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "ABS({}) = {} is negative", x, n);
        }
    }

    // ROUND(x, n): confirmed by m1/Math.xlsx — result differs from x by less than 10^(-n)
    #[test]
    fn m1_round_precision(x in small_f64(), digits in 0i32..=5) {
        let formula = format!("=ROUND(x, {})", digits);
        let r = run_n(&formula, "x", x);
        if let Value::Number(n) = r {
            let tolerance = 10f64.powi(-digits) / 2.0 + 1e-9;
            prop_assert!((n - x).abs() <= tolerance,
                "ROUND({}, {}) = {} differs by {}", x, digits, n, (n-x).abs());
        }
    }

    // SQRT(x): x >= 0 → result >= 0 and result^2 ≈ x
    #[test]
    fn m1_sqrt_inverse_square(x in pos_f64()) {
        let r = run_n("=SQRT(x)", "x", x);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "SQRT({}) = {} is negative", x, n);
            prop_assert!((n * n - x).abs() / x.max(1.0) < 1e-9,
                "SQRT({})^2 = {} (expected {})", x, n*n, x);
        }
    }

    // MOD(a, b): result has same sign as b and |result| < |b|
    #[test]
    fn m1_mod_remainder_bounds(a in small_f64(), b in pos_f64()) {
        let formula = format!("=MOD(x, {})", b);
        let r = run_n(&formula, "x", a);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "MOD({},{}) = {} is negative", a, b, n);
            prop_assert!(n < b + 1e-9, "MOD({},{}) = {} >= b", a, b, n);
        }
    }

    // POWER(x, 2) == x * x for small positive x
    #[test]
    fn m1_power_two_equals_square(x in 0.0f64..1000.0f64) {
        let r = run_n("=POWER(x, 2)", "x", x);
        if let Value::Number(n) = r {
            prop_assert!((n - x * x).abs() < 1e-6,
                "POWER({}, 2) = {} but x*x = {}", x, n, x*x);
        }
    }
}

// ─── M1: Text ────────────────────────────────────────────────────────────────

proptest! {
    // LEN: confirmed by m1/Text.xlsx — always >= 0
    #[test]
    fn m1_len_non_negative(s in ascii_str()) {
        let r = run_s("=LEN(x)", "x", &s);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "LEN({:?}) = {} is negative", s, n);
        }
    }

    // LEN(s) == number of characters in s
    #[test]
    fn m1_len_equals_char_count(s in ascii_str()) {
        let r = run_s("=LEN(x)", "x", &s);
        if let Value::Number(n) = r {
            prop_assert_eq!(n as usize, s.chars().count(),
                "LEN({:?}) = {} but char count = {}", s, n, s.chars().count());
        }
    }

    // LOWER: result is always lowercase
    #[test]
    fn m1_lower_result_is_lowercase(s in ascii_str()) {
        let r = run_s("=LOWER(x)", "x", &s);
        if let Value::Text(t) = r {
            prop_assert_eq!(t, t.to_lowercase(),
                "LOWER({:?}) = {:?} is not lowercase", s, t);
        }
    }

    // UPPER: result is always uppercase
    #[test]
    fn m1_upper_result_is_uppercase(s in ascii_str()) {
        let r = run_s("=UPPER(x)", "x", &s);
        if let Value::Text(t) = r {
            prop_assert_eq!(t, t.to_uppercase(),
                "UPPER({:?}) = {:?} is not uppercase", s, t);
        }
    }
}

// ─── M1: Logical ─────────────────────────────────────────────────────────────

proptest! {
    // IF(TRUE, a, b) == a
    #[test]
    fn m1_if_true_picks_first(a in small_f64(), b in small_f64()) {
        let formula = format!("=IF(TRUE, {}, {})", a, b);
        let r = run(&formula);
        if let Value::Number(n) = r {
            prop_assert!((n - a).abs() < 1e-12,
                "IF(TRUE,{},{}) = {} (expected {})", a, b, n, a);
        }
    }

    // IF(FALSE, a, b) == b
    #[test]
    fn m1_if_false_picks_second(a in small_f64(), b in small_f64()) {
        let formula = format!("=IF(FALSE, {}, {})", a, b);
        let r = run(&formula);
        if let Value::Number(n) = r {
            prop_assert!((n - b).abs() < 1e-12,
                "IF(FALSE,{},{}) = {} (expected {})", a, b, n, b);
        }
    }

    // AND(TRUE, TRUE) == TRUE; AND(TRUE, FALSE) == FALSE
    #[test]
    fn m1_and_both_true(x in proptest::bool::ANY, y in proptest::bool::ANY) {
        let tx = if x { "TRUE" } else { "FALSE" };
        let ty = if y { "TRUE" } else { "FALSE" };
        let formula = format!("=AND({},{})", tx, ty);
        let r = run(&formula);
        if let Value::Bool(b) = r {
            prop_assert_eq!(b, x && y, "AND({},{}) = {} (expected {})", x, y, b, x&&y);
        }
    }

    // OR(x, y) == x || y
    #[test]
    fn m1_or_semantics(x in proptest::bool::ANY, y in proptest::bool::ANY) {
        let tx = if x { "TRUE" } else { "FALSE" };
        let ty = if y { "TRUE" } else { "FALSE" };
        let formula = format!("=OR({},{})", tx, ty);
        let r = run(&formula);
        if let Value::Bool(b) = r {
            prop_assert_eq!(b, x || y, "OR({},{}) = {} (expected {})", x, y, b, x||y);
        }
    }
}

// ─── M2: Math ────────────────────────────────────────────────────────────────

proptest! {
    // SUMPRODUCT([a],[b]) == a*b for single-element arrays
    // Approximated as: =a*b via operator
    #[test]
    fn m2_multiplication_commutative(a in small_f64(), b in small_f64()) {
        let ab = run_two("=x*y", ("x", a), ("y", b));
        let ba = run_two("=x*y", ("x", b), ("y", a));
        prop_assert_eq!(ab, ba, "{}*{} != {}*{}", a, b, b, a);
    }
}
```

- [ ] **Step 2: Run all conformance-driven property tests**

```bash
cargo test -p truecalc-core --test property_conformance -- --nocapture 2>&1 | tail -15
```

Expected: all tests pass. If any fail, the output shows the exact input and formula — investigate the evaluator for that function.

- [ ] **Step 3: Check test count**

```bash
cargo test -p truecalc-core --test property_conformance -- --list 2>&1 | grep "test$" | wc -l
```

Expected: at least 15 tests listed.

- [ ] **Step 4: Commit**

```bash
git add crates/core/tests/property_conformance.rs
git commit -m "test(proptest): conformance-driven property tests across m1-m2 functions

For each function confirmed by Google Sheets oracle fixtures, verify the
mathematical property holds across proptest-generated inputs:
- Math: ABS non-negative, ROUND precision, SQRT inverse, MOD bounds, POWER
- Text: LEN count, LOWER/UPPER case invariants
- Logical: IF branching, AND/OR semantics
- M2: multiplication commutativity

Closes #373"
```

---

## Task 2: Open PR

- [ ] **Step 1: Push and create PR**

```bash
gh pr create \
  --repo truecalc/core \
  --title "test(proptest): conformance-driven property tests — truecalc ≈ Google Sheets across generated inputs" \
  --assignee hhimanshu \
  --body "$(cat <<'EOF'
## Summary
Adds `property_conformance.rs` with property tests derived from what Google Sheets oracle fixtures confirm:

- **M1 Math**: ABS non-negative, ROUND precision, SQRT inverse, MOD bounds, POWER correctness
- **M1 Text**: LEN exact char count, LOWER/UPPER case invariants
- **M1 Logical**: IF branching semantics, AND/OR truth table
- **M2 Math**: multiplication commutativity

These complement the fixed-row conformance tests by generating varied inputs that don't appear in the fixture files, catching regressions more broadly.

closes #373
EOF
)"
gh pr edit --add-assignee hhimanshu
```

- [ ] **Step 2: Monitor CI**

```bash
gh run list --repo truecalc/core --limit 3
```

On failure: `gh run view <run-id> --log-failed --repo truecalc/core`
