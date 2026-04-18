# Proptest Expansion: Idempotency, Round-trips, Error Propagation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add three new *classes* of property-based tests: idempotency (applying a function twice gives the same result as once), round-trips (composing inverse functions returns the original value), and error propagation (errors passed to non-error-handling functions always produce errors). These properties are mathematical invariants — any violation is a bug, independent of the specific input.

**Architecture:** Expand `property_text.rs` and `property_math.rs` with new proptest blocks. Create `property_error_propagation.rs` for the error class. All tests use the existing `proptest` + `truecalc_core` dev-dependencies. Error values are injected as variables using `evaluate(formula, &vars)` where `vars` contains a `Value::Error(ErrorKind::...)` entry.

**Tech Stack:** proptest 1.x, truecalc_core (evaluate, Value, ErrorKind)

**GitHub issue:** Closes #371 (sub-issue of epic #366)

---

## File Map

| Action | File | Purpose |
|--------|------|---------|
| Modify | `crates/core/tests/property_text.rs` | Add idempotency properties for text functions |
| Modify | `crates/core/tests/property_math.rs` | Add idempotency and monotonicity properties |
| Create | `crates/core/tests/property_error_propagation.rs` | Error propagation across all function categories |
| Modify | `crates/core/tests/conformance.rs` | Add `mod property_error_propagation;` (if needed by test harness — check if it auto-discovers) |

Note: Rust's test harness auto-discovers `.rs` files in `tests/` only if they are separate integration test files (not `mod` declarations). Since `property_error_propagation.rs` is a new top-level file in `tests/`, it is discovered automatically — no `mod` declaration needed.

---

## Task 1: Add idempotency tests to `property_text.rs`

**Files:**
- Modify: `crates/core/tests/property_text.rs`

- [ ] **Step 1: Write the failing tests first**

Read the current end of `crates/core/tests/property_text.rs` to find the closing `}` of the `proptest!` block, then add these properties inside that block (before the closing `}`):

```rust
    // Idempotency: LOWER(LOWER(s)) == LOWER(s)
    #[test]
    fn lower_idempotent(s in ascii_string()) {
        let vars: HashMap<String, Value> = [(
            "s".to_string(), Value::Text(s.clone()),
        )].into_iter().collect();
        let once = evaluate("=LOWER(s)", &vars);
        if let Value::Text(t) = &once {
            let vars2: HashMap<String, Value> = [(
                "s".to_string(), Value::Text(t.clone()),
            )].into_iter().collect();
            let twice = evaluate("=LOWER(s)", &vars2);
            prop_assert_eq!(once, twice, "LOWER not idempotent on {:?}", s);
        }
    }

    // Idempotency: UPPER(UPPER(s)) == UPPER(s)
    #[test]
    fn upper_idempotent(s in ascii_string()) {
        let vars: HashMap<String, Value> = [(
            "s".to_string(), Value::Text(s.clone()),
        )].into_iter().collect();
        let once = evaluate("=UPPER(s)", &vars);
        if let Value::Text(t) = &once {
            let vars2: HashMap<String, Value> = [(
                "s".to_string(), Value::Text(t.clone()),
            )].into_iter().collect();
            let twice = evaluate("=UPPER(s)", &vars2);
            prop_assert_eq!(once, twice, "UPPER not idempotent on {:?}", s);
        }
    }

    // LEN is non-negative for any string
    #[test]
    fn len_non_negative(s in ascii_string()) {
        let vars: HashMap<String, Value> = [(
            "s".to_string(), Value::Text(s.clone()),
        )].into_iter().collect();
        let result = evaluate("=LEN(s)", &vars);
        if let Value::Number(n) = result {
            prop_assert!(n >= 0.0, "LEN returned negative for {:?}", s);
        }
    }

    // CONCATENATE length: LEN(CONCATENATE(a, b)) == LEN(a) + LEN(b)
    // (Already tested but add explicit assertion message)
    #[test]
    fn concatenate_preserves_total_length(a in ascii_string(), b in ascii_string()) {
        let vars: HashMap<String, Value> = [
            ("a".to_string(), Value::Text(a.clone())),
            ("b".to_string(), Value::Text(b.clone())),
        ].into_iter().collect();
        let ab_len = evaluate("=LEN(CONCATENATE(a,b))", &vars);
        let a_len  = evaluate("=LEN(a)", &vars);
        let b_len  = evaluate("=LEN(b)", &vars);
        if let (Value::Number(total), Value::Number(la), Value::Number(lb)) = (ab_len, a_len, b_len) {
            prop_assert_eq!(total, la + lb,
                "LEN(CONCATENATE({:?},{:?})) != LEN(a)+LEN(b)", a, b);
        }
    }
```

- [ ] **Step 2: Run to verify tests pass**

```bash
cargo test -p truecalc-core --test property_text 2>&1 | tail -8
```

Expected: all tests pass including new ones.

- [ ] **Step 3: Commit**

```bash
git add crates/core/tests/property_text.rs
git commit -m "test(proptest): add idempotency and length properties for text functions"
```

---

## Task 2: Add idempotency and monotonicity to `property_math.rs`

**Files:**
- Modify: `crates/core/tests/property_math.rs`

- [ ] **Step 1: Add new properties inside the existing `proptest!` block**

Append before the closing `}` of the `proptest!` block:

```rust
    // Idempotency: ABS(ABS(x)) == ABS(x)
    #[test]
    fn abs_idempotent(x in finite_f64()) {
        let once  = run_vars("=ABS(x)", vec![("x", x)]);
        let twice = run_vars("=ABS(ABS(x))", vec![("x", x)]);
        prop_assert_eq!(once.clone(), twice,
            "ABS not idempotent on {}", x);
    }

    // Monotonicity: MAX(a,b) >= a and MAX(a,b) >= b
    #[test]
    fn max_dominates_both(a in small_f64(), b in small_f64()) {
        let max = run_vars("=MAX(x, y)", vec![("x", a), ("y", b)]);
        if let Value::Number(m) = max {
            prop_assert!(m >= a - 1e-12, "MAX({},{}) = {} < a", a, b, m);
            prop_assert!(m >= b - 1e-12, "MAX({},{}) = {} < b", a, b, m);
        }
    }

    // Monotonicity: MIN(a,b) <= a and MIN(a,b) <= b
    #[test]
    fn min_dominated_by_both(a in small_f64(), b in small_f64()) {
        let min = run_vars("=MIN(x, y)", vec![("x", a), ("y", b)]);
        if let Value::Number(m) = min {
            prop_assert!(m <= a + 1e-12, "MIN({},{}) = {} > a", a, b, m);
            prop_assert!(m <= b + 1e-12, "MIN({},{}) = {} > b", a, b, m);
        }
    }

    // Round-trip: EXP(LN(x)) ≈ x for x > 0
    #[test]
    fn exp_ln_roundtrip(x in 1e-6f64..1e6f64) {
        let result = run_vars("=EXP(LN(x))", vec![("x", x)]);
        if let Value::Number(n) = result {
            prop_assert!((n - x).abs() / x.abs().max(1.0) < 1e-9,
                "EXP(LN({})) = {} (delta {})", x, n, (n-x).abs());
        }
    }

    // ABS is always >= 0
    #[test]
    fn abs_always_non_negative(x in finite_f64()) {
        let result = run_vars("=ABS(x)", vec![("x", x)]);
        if let Value::Number(n) = result {
            prop_assert!(n >= 0.0, "ABS({}) returned {}", x, n);
        }
    }
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p truecalc-core --test property_math 2>&1 | tail -8
```

Expected: all pass.

- [ ] **Step 3: Commit**

```bash
git add crates/core/tests/property_math.rs
git commit -m "test(proptest): add idempotency, monotonicity, and round-trip properties for math functions"
```

---

## Task 3: Create `property_error_propagation.rs`

**Files:**
- Create: `crates/core/tests/property_error_propagation.rs`

Error propagation rule: for any non-error-handling function `f`, if an input is an error value, the output should also be an error value. We inject errors as variables.

- [ ] **Step 1: Write the file**

```rust
// crates/core/tests/property_error_propagation.rs
//
// Verifies that non-error-handling functions propagate error values rather
// than silently resolving them. This is a correctness invariant derived from
// Google Sheets behavior: errors are contagious.

use truecalc_core::{evaluate, ErrorKind, Value};
use proptest::prelude::*;
use std::collections::HashMap;

fn error_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Error(ErrorKind::Value)),
        Just(Value::Error(ErrorKind::NA)),
        Just(Value::Error(ErrorKind::DivByZero)),
        Just(Value::Error(ErrorKind::Num)),
    ]
}

fn run_with_error(formula: &str, var: &str, err: Value) -> Value {
    let mut vars = HashMap::new();
    vars.insert(var.to_string(), err);
    evaluate(formula, &vars)
}

fn is_error(v: &Value) -> bool {
    matches!(v, Value::Error(_))
}

proptest! {
    // Math functions propagate errors
    #[test]
    fn abs_propagates_error(e in error_value()) {
        let result = run_with_error("=ABS(x)", "x", e);
        prop_assert!(is_error(&result), "ABS did not propagate error, got {:?}", result);
    }

    #[test]
    fn sqrt_propagates_error(e in error_value()) {
        let result = run_with_error("=SQRT(x)", "x", e);
        prop_assert!(is_error(&result), "SQRT did not propagate error, got {:?}", result);
    }

    #[test]
    fn ln_propagates_error(e in error_value()) {
        let result = run_with_error("=LN(x)", "x", e);
        prop_assert!(is_error(&result), "LN did not propagate error, got {:?}", result);
    }

    #[test]
    fn exp_propagates_error(e in error_value()) {
        let result = run_with_error("=EXP(x)", "x", e);
        prop_assert!(is_error(&result), "EXP did not propagate error, got {:?}", result);
    }

    #[test]
    fn round_propagates_error(e in error_value()) {
        let result = run_with_error("=ROUND(x, 2)", "x", e);
        prop_assert!(is_error(&result), "ROUND did not propagate error, got {:?}", result);
    }

    // Text functions propagate errors
    #[test]
    fn len_propagates_error(e in error_value()) {
        let result = run_with_error("=LEN(x)", "x", e);
        prop_assert!(is_error(&result), "LEN did not propagate error, got {:?}", result);
    }

    #[test]
    fn upper_propagates_error(e in error_value()) {
        let result = run_with_error("=UPPER(x)", "x", e);
        prop_assert!(is_error(&result), "UPPER did not propagate error, got {:?}", result);
    }

    #[test]
    fn lower_propagates_error(e in error_value()) {
        let result = run_with_error("=LOWER(x)", "x", e);
        prop_assert!(is_error(&result), "LOWER did not propagate error, got {:?}", result);
    }

    #[test]
    fn trim_propagates_error(e in error_value()) {
        let result = run_with_error("=TRIM(x)", "x", e);
        prop_assert!(is_error(&result), "TRIM did not propagate error, got {:?}", result);
    }
}
```

- [ ] **Step 2: Run to see if they pass or reveal bugs**

```bash
cargo test -p truecalc-core --test property_error_propagation -- --nocapture 2>&1 | tail -15
```

Expected: all 9 tests pass. If any fail, the failure output will show the exact error variant and function — investigate the evaluator for that function and fix the propagation before committing.

- [ ] **Step 3: Commit**

```bash
git add crates/core/tests/property_error_propagation.rs
git commit -m "test(proptest): add error propagation property tests for math and text functions

Verifies that ABS, SQRT, LN, EXP, ROUND, LEN, UPPER, LOWER, TRIM propagate
error values instead of silently resolving them. Matches Google Sheets behavior.

Closes #371"
```

---

## Task 4: Open PR

- [ ] **Step 1: Push and create PR**

```bash
gh pr create \
  --repo truecalc/core \
  --title "test(proptest): idempotency, round-trips, and error propagation properties" \
  --assignee hhimanshu \
  --body "$(cat <<'EOF'
## Summary
- `property_text.rs`: idempotency for LOWER/UPPER, LEN non-negative, CONCATENATE length preservation
- `property_math.rs`: ABS idempotency, MAX/MIN monotonicity, EXP(LN(x)) ≈ x round-trip
- `property_error_propagation.rs` (new): verifies ABS, SQRT, LN, EXP, ROUND, LEN, UPPER, LOWER, TRIM all propagate error values — matches Google Sheets behavior where errors are contagious

closes #371
EOF
)"
gh pr edit --add-assignee hhimanshu
```

- [ ] **Step 2: Monitor CI**

```bash
gh run list --repo truecalc/core --limit 3
```

On failure: `gh run view <run-id> --log-failed --repo truecalc/core`
