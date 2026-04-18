# Test Excellence: Coverage, Conformance & Google Sheets Transparency

**Date:** 2026-04-18
**Status:** Approved for implementation planning

## Summary

truecalc's calculations match Google Sheets — and we can prove it. This spec defines the work to measure, enforce, report, and market that claim through five phases of test infrastructure improvements.

---

## Context

### What we already have

- **~2,461 tests** across unit, integration, and conformance test types
- **Per-function unit tests** structured with `success / failure / edge` categories
- **Oracle conformance tests** against 30 Google Sheets fixture files (`.xlsx` exports), covering m1–m4 milestones across text, math, financial, logical, and date functions
- **Property-based tests** (proptest) for math, text, logical, and financial functions
- **CI** runs `cargo test`, `clippy -D warnings`, `cargo-deny`, and `wasm-pack build`

### What is missing

- No coverage measurement (no tarpaulin, llvm-cov, codecov)
- No coverage enforcement gate on PRs
- No richer test output (nextest, JUnit XML, GitHub inline annotations)
- No conformance summary reporting (badge, PR comment, regression detection)
- No property tests for date, lookup, or array functions
- No idempotency / round-trip / error-propagation property tests
- No conformance-driven property tests (generated inputs against GS oracle)
- No public-facing conformance page on the website

### Conformance oracle stance

**Google Sheets is the primary oracle.** All conformance tests target GS behavior exactly. Known deviations from GS are documented with a reason and tracked separately — they do not fail CI but appear in a dedicated "known deviations" section of the PR conformance comment. Excel compatibility is a future concern (a separate Excel API may be introduced when behaviors differ); it is out of scope for this spec.

---

## Goals

1. **Measure** line/branch coverage per crate and enforce a minimum gate on every PR
2. **Report** test results richly in CI: per-test timing, inline PR failure annotations, coverage delta, conformance table
3. **Prove** Google Sheets conformance publicly: badge in README, automated PR comment with regression detection
4. **Deepen** property-based testing: more properties, more functions, conformance-driven generation
5. **Market** the above on the website (future phase)

---

## Epic

**"Test Excellence: Coverage, Conformance & Google Sheets Transparency"**

One GitHub epic. Eight sub-issues across five phases. Each issue is scoped for an independent agent working on an isolated branch.

---

## Phase Breakdown

### Phase 1 — CI Test Runner (Issue 1)

**Replace `cargo test` with `cargo-nextest`.**

- nextest runs tests in parallel per-test (not per-binary), producing faster feedback and per-test timing
- Outputs JUnit XML; the GitHub test reporter action parses this and posts inline annotations on the PR diff for each failing test
- Failure annotation format:
  ```
  ● truecalc-core/src/eval/functions/concat.rs line 42
    FAILED: concat_numbers
    expected: Text("12")   got: Text("12.0")
  ```
- Files changed: `.github/workflows/ci.yml`, `Cargo.toml` (add nextest config)

### Phase 2 — Coverage Measurement (Issue 2)

**Add `cargo-llvm-cov` + codecov.io integration.**

- `cargo-llvm-cov` instruments the binary and produces LCOV output; codecov.io parses and hosts it
- Every PR gets a comment showing per-crate coverage delta:
  ```
  Coverage Report
  truecalc-core    87.3%  (+2.1%)  ↑
  truecalc-wasm     0.0%  (new)    ⚠
  truecalc-mcp     12.4%  (new)    ⚠
  PR drops overall coverage by 0.2%. Gate: 80%.
  ```
- Coverage gate: 80% minimum on `truecalc-core`. PRs that drop below fail CI.
- Files changed: `.github/workflows/ci.yml`, new `codecov.yml`

### Phase 3 — Conformance Reporter (Issue 3)

**Extend conformance tests to emit a structured JSON summary.**

- After the test run, a summary JSON is written to `target/conformance-report.json`:
  ```json
  {
    "total": 2461, "passed": 2430, "failed": 8, "skipped": 23,
    "by_category": {
      "text":      { "passed": 342, "total": 342 },
      "math":      { "passed": 418, "total": 420 },
      "financial": { "passed": 201, "total": 210 },
      "logical":   { "passed": 89,  "total": 89  },
      "date":      { "passed": 76,  "total": 80  }
    },
    "known_deviations": [
      { "formula": "=RATE(4*12,-200,8000)", "reason": "floating-point iteration limit differs" }
    ]
  }
  ```
- Known deviations are tracked as `#[ignore]` with a structured reason string; the reporter collects them separately
- Files changed: `crates/core/tests/conformance.rs`, new `crates/core/tests/conformance_reporter.rs` (or inline helper)

### Phase 4 — Conformance Badge + PR Comment (Issue 4)

**Publish the conformance summary publicly on every PR.**

Depends on: Issues 1 (nextest/JUnit), 2 (coverage), 3 (reporter JSON) merged.

- CI reads `target/conformance-report.json` and posts a PR comment:
  ```
  Google Sheets Conformance
  ─────────────────────────────────────────────────────
  Category     Passed   Total   Coverage
  text          342/342   100%
  math          418/420    99.5%  (2 known deviations)
  financial     201/210    95.7%  ← regression vs main
  logical        89/89    100%
  date           76/80     95.0%
  ─────────────────────────────────────────────────────
  Total        2126/2141   99.3%   (was 99.5% on main)
  ```
- Regressions (previously passing tests that now fail) cause CI to fail
- Known deviations appear in a separate section and do not fail CI
- A badge is added to `README.md`: `conformance: 2430/2461 · 98.7% · Google Sheets`
- Files changed: `.github/workflows/ci.yml`, `README.md`, new CI script for comment generation

### Phase 5 — Property Test Expansion (Issues 5, 6, 7 — parallel)

**Three independent issues, safe to work simultaneously.**

**Issue 5 — New property classes across existing functions:**
- Idempotency: `TRIM(TRIM(x)) = TRIM(x)`, `LOWER(LOWER(x)) = LOWER(x)`
- Round-trip: `VALUE(TEXT(n, "0")) = n` (within tolerance)
- Error propagation: `f(#VALUE!) = #VALUE!` for all non-error-handling functions
- Monotonicity: `IF(a > b, a, b) >= a` always
- Files: `property_text.rs`, `property_math.rs`, new `property_error_propagation.rs`

**Issue 6 — Proptest for uncovered function categories:**
- Date functions: date arithmetic commutativity, boundary conditions (leap years, month-end)
- Lookup functions: range invariants
- Array functions: length preservation, element-wise properties
- Files: new `property_date.rs`, `property_lookup.rs`, `property_array.rs`

**Issue 7 — Conformance-driven property tests:**
- For each function present in the fixture files, generate varied inputs beyond the fixture rows and verify `truecalc(formula) ≈ google_sheets_oracle` within documented tolerance
- This catches regressions in functions that have oracle fixtures but limited hand-written edge cases
- Files: new `property_conformance.rs`

### Phase 6 — Website (Issue 8 — future)

**A public-facing conformance and testing transparency page.**

- Shows live conformance match rate, coverage badge, property test count, link to GS oracle spreadsheets (pending GS migration to org account — parked)
- Out of scope for current implementation planning; captured here for future reference

---

## Execution Order

```
Week 1:
  Agent A  → Issue 1 (nextest)            ║  Agent C1 → Issue 5 (proptest expand)
                                           ║  Agent C2 → Issue 6 (proptest new fns)
                                           ║  Agent C3 → Issue 7 (conformance props)
Week 2:
  Agent A  → Issue 2 (coverage)           ║  Agent B  → Issue 3 (conformance JSON)

Week 3:
  Agent B  → Issue 4 (badge + PR comment) ← depends on issues 1, 2, 3 merged

Future:
  Issue 8  (website)
```

Issues 5, 6, 7 are pure test additions — no CI files, no shared code — safe to run simultaneously with any other agent.

---

## File Change Map

| Issue | Files |
|-------|-------|
| 1 | `.github/workflows/ci.yml`, `Cargo.toml` (nextest config) |
| 2 | `.github/workflows/ci.yml`, `codecov.yml` |
| 3 | `crates/core/tests/conformance.rs`, `crates/core/tests/conformance_reporter.rs` |
| 4 | `.github/workflows/ci.yml`, `README.md`, CI comment script |
| 5 | `crates/core/tests/property_text.rs`, `property_math.rs`, `property_error_propagation.rs` |
| 6 | `crates/core/tests/property_date.rs`, `property_lookup.rs`, `property_array.rs` |
| 7 | `crates/core/tests/property_conformance.rs` |
| 8 | website repo (future) |

---

## Success Criteria

- Every PR shows: nextest pass/fail with inline annotations, coverage delta with 80% gate, Google Sheets conformance table with regression detection
- `truecalc-core` coverage ≥ 80% enforced on CI
- Conformance badge visible in README: `N/M tests · X% · Google Sheets`
- Property tests cover date, lookup, and array functions
- Known deviations from GS are documented with reasons and do not fail CI
- Website conformance page shipped (future)
