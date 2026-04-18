# Conformance Reporter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the Google Sheets oracle conformance tests to emit a machine-readable JSON summary (`target/conformance-report.json`) after each run, enabling the badge and PR comment in issue #370.

**Architecture:** Add a `collect_fixture_results` function alongside the existing `run_fixture` (which panics on failure). A new test `#[test] fn generate_conformance_report()` calls `collect_fixture_results` for every fixture file, aggregates into a `ConformanceReport` struct, and serializes to JSON. Known deviations (`#[ignore]`-tagged tests) are tracked via a separate `KNOWN_DEVIATIONS` slice. No new dependencies — JSON is hand-serialized.

**Tech Stack:** Rust std, existing `calamine` + `ganit_core` dev-deps

**GitHub issue:** Closes #369 (sub-issue of epic #366)

---

## File Map

| Action | File | Purpose |
|--------|------|---------|
| Create | `crates/core/tests/conformance_reporter.rs` | `collect_fixture_results`, `ConformanceReport`, JSON serializer, known-deviation list |
| Modify | `crates/core/tests/conformance.rs` | Add `mod conformance_reporter;` and the `generate_conformance_report` test |

---

## Task 1: Create `conformance_reporter.rs`

**Files:**
- Create: `crates/core/tests/conformance_reporter.rs`

- [ ] **Step 1: Write the reporter module**

```rust
// crates/core/tests/conformance_reporter.rs
//
// Collects pass/fail counts per fixture and writes target/conformance-report.json.
// Called by the generate_conformance_report test in conformance.rs.

use calamine::{open_workbook, Data, Reader, Xlsx};
use ganit_core::{evaluate, ErrorKind, Value};
use std::collections::HashMap;
use std::path::Path;

// Re-use the oracle helpers from the parent module via super::
// (parse_error_string, oracle_to_value, values_match, is_volatile_formula
//  are defined in conformance.rs and visible here through the test module tree)

#[derive(Default, Debug)]
pub struct CategoryResult {
    pub passed: usize,
    pub total: usize,
    pub failures: Vec<String>,
}

#[derive(Default, Debug)]
pub struct ConformanceReport {
    pub by_category: HashMap<String, CategoryResult>,
    pub known_deviations: Vec<KnownDeviation>,
}

#[derive(Debug)]
pub struct KnownDeviation {
    pub formula: &'static str,
    pub reason: &'static str,
}

impl ConformanceReport {
    pub fn total_passed(&self) -> usize {
        self.by_category.values().map(|r| r.passed).sum()
    }
    pub fn total_tests(&self) -> usize {
        self.by_category.values().map(|r| r.total).sum()
    }
    pub fn total_failed(&self) -> usize {
        self.total_tests() - self.total_passed()
    }

    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"total\": {},\n", self.total_tests()));
        s.push_str(&format!("  \"passed\": {},\n", self.total_passed()));
        s.push_str(&format!("  \"failed\": {},\n", self.total_failed()));
        s.push_str("  \"by_category\": {\n");
        let mut cats: Vec<(&String, &CategoryResult)> = self.by_category.iter().collect();
        cats.sort_by_key(|(k, _)| k.as_str());
        for (i, (cat, result)) in cats.iter().enumerate() {
            let comma = if i + 1 < cats.len() { "," } else { "" };
            s.push_str(&format!(
                "    \"{}\": {{ \"passed\": {}, \"total\": {} }}{}\n",
                cat, result.passed, result.total, comma
            ));
        }
        s.push_str("  },\n");
        s.push_str("  \"known_deviations\": [\n");
        for (i, dev) in self.known_deviations.iter().enumerate() {
            let comma = if i + 1 < self.known_deviations.len() { "," } else { "" };
            s.push_str(&format!(
                "    {{ \"formula\": \"{}\", \"reason\": \"{}\" }}{}\n",
                dev.formula.replace('"', "\\\""),
                dev.reason.replace('"', "\\\""),
                comma
            ));
        }
        s.push_str("  ]\n");
        s.push_str("}\n");
        s
    }
}

/// Known deviations: cases where ganit intentionally differs from Google Sheets.
/// These are excluded from failure counts but documented in the report.
pub const KNOWN_DEVIATIONS: &[KnownDeviation] = &[
    // Add entries here as they are discovered. Format:
    // KnownDeviation { formula: "=RATE(4*12,-200,8000)", reason: "floating-point iteration limit differs" },
];

/// Collect pass/fail for a single fixture file. Returns results without panicking.
/// `category` is the logical name used in the JSON (e.g. "math", "text").
pub fn collect_fixture_results(
    path: &Path,
    category: &str,
    report: &mut ConformanceReport,
) {
    if !path.exists() {
        return;
    }

    let mut workbook: Xlsx<_> = match open_workbook(path) {
        Ok(w) => w,
        Err(_) => return,
    };

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let vars: HashMap<String, Value> = HashMap::new();
    let entry = report.by_category.entry(category.to_string()).or_default();

    for sheet_name in &sheet_names {
        let range = match workbook.worksheet_range(sheet_name) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for (row_idx, row) in range.rows().enumerate().skip(1) {
            if row.len() < 3 {
                continue;
            }
            let formula = match &row[1] {
                Data::String(s) => s.as_str(),
                _ => continue,
            };
            let expected = match super::oracle_to_value(&row[2]) {
                Some(v) => v,
                None => continue,
            };
            if super::is_volatile_formula(formula) {
                continue;
            }

            entry.total += 1;
            let actual = evaluate(formula, &vars);
            if super::values_match(&actual, &expected) {
                entry.passed += 1;
            } else {
                let desc = match &row[0] { Data::String(s) => s.clone(), _ => String::new() };
                entry.failures.push(format!(
                    "[{}] row {} {}: formula={} expected={:?} got={:?}",
                    sheet_name, row_idx + 2, desc, formula, expected, actual
                ));
            }
        }
    }
}
```

- [ ] **Step 2: Verify it compiles (it won't yet — `super::` references need the module wired in first; skip compile check, proceed to Task 2)**

---

## Task 2: Wire the reporter into `conformance.rs`

**Files:**
- Modify: `crates/core/tests/conformance.rs`

- [ ] **Step 1: Add module declaration after the existing imports at the top of `conformance.rs`**

After the `use` statements (around line 15), add:

```rust
mod conformance_reporter;
use conformance_reporter::{collect_fixture_results, ConformanceReport, KNOWN_DEVIATIONS};
```

- [ ] **Step 2: Add the `generate_conformance_report` test at the bottom of `conformance.rs`**

After the last `conformance_test!` macro call (after line 261), add:

```rust
// ---------------------------------------------------------------------------
// Conformance report generator — writes target/conformance-report.json
// ---------------------------------------------------------------------------

#[test]
fn generate_conformance_report() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut report = ConformanceReport::default();
    report.known_deviations = KNOWN_DEVIATIONS.to_vec();

    // m1 fixtures
    collect_fixture_results(&fixture("m1", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m1", "Logical.xlsx"),     "logical",     &mut report);
    collect_fixture_results(&fixture("m1", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m1", "Statistical.xlsx"), "statistical", &mut report);
    collect_fixture_results(&fixture("m1", "Operator.xlsx"),    "operator",    &mut report);
    collect_fixture_results(&fixture("m1", "Text.xlsx"),        "text",        &mut report);

    // m2 fixtures
    collect_fixture_results(&fixture("m2", "Date.xlsx"),        "date",        &mut report);
    collect_fixture_results(&fixture("m2", "Engineering.xlsx"), "engineering", &mut report);
    collect_fixture_results(&fixture("m2", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m2", "Logical.xlsx"),     "logical",     &mut report);
    collect_fixture_results(&fixture("m2", "Lookup.xlsx"),      "lookup",      &mut report);
    collect_fixture_results(&fixture("m2", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m2", "Statistical.xlsx"), "statistical", &mut report);
    collect_fixture_results(&fixture("m2", "Text.xlsx"),        "text",        &mut report);

    // m3 fixtures
    collect_fixture_results(&fixture("m3", "Database.xlsx"),    "database",    &mut report);
    collect_fixture_results(&fixture("m3", "Engineering.xlsx"), "engineering", &mut report);
    collect_fixture_results(&fixture("m3", "Financial.xlsx"),   "financial",   &mut report);
    collect_fixture_results(&fixture("m3", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m3", "Lookup.xlsx"),      "lookup",      &mut report);
    collect_fixture_results(&fixture("m3", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m3", "Statistical.xlsx"), "statistical", &mut report);

    // m4 fixtures
    collect_fixture_results(&fixture("m4", "Array.xlsx"),       "array",       &mut report);
    collect_fixture_results(&fixture("m4", "Filter.xlsx"),      "filter",      &mut report);
    collect_fixture_results(&fixture("m4", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m4", "Logical.xlsx"),     "logical",     &mut report);
    collect_fixture_results(&fixture("m4", "Lookup.xlsx"),      "lookup",      &mut report);
    collect_fixture_results(&fixture("m4", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m4", "Operator.xlsx"),    "operator",    &mut report);

    // Write JSON to target/
    let out_dir = manifest.join("../../target");
    std::fs::create_dir_all(&out_dir).ok();
    let out_path = out_dir.join("conformance-report.json");
    std::fs::write(&out_path, report.to_json())
        .expect("failed to write conformance-report.json");

    println!("conformance-report.json written to {}", out_path.display());
    println!(
        "Total: {}/{} passed ({} failed)",
        report.total_passed(),
        report.total_tests(),
        report.total_failed(),
    );
}
```

Note: `KNOWN_DEVIATIONS.to_vec()` requires `KnownDeviation` to implement `Clone`. Add `#[derive(Clone)]` to `KnownDeviation` in `conformance_reporter.rs`.

- [ ] **Step 3: Add `#[derive(Clone)]` to `KnownDeviation` in `conformance_reporter.rs`**

Change:
```rust
#[derive(Debug)]
pub struct KnownDeviation {
```
to:
```rust
#[derive(Debug, Clone)]
pub struct KnownDeviation {
```

- [ ] **Step 4: Compile-check**

```bash
cargo test -p ganit-core --test conformance generate_conformance_report --no-run 2>&1 | tail -10
```

Expected: `Compiling ganit-core` then `Finished` — no errors.

- [ ] **Step 5: Run the report generator**

```bash
cargo test -p ganit-core --test conformance generate_conformance_report -- --nocapture 2>&1 | tail -5
```

Expected output ends with:
```
conformance-report.json written to .../target/conformance-report.json
Total: XXXX/XXXX passed (X failed)
```

- [ ] **Step 6: Inspect the JSON output**

```bash
cat target/conformance-report.json
```

Verify it is valid JSON with `total`, `passed`, `failed`, `by_category`, and `known_deviations` keys.

- [ ] **Step 7: Run existing conformance tests to confirm nothing regressed**

```bash
cargo test -p ganit-core --test conformance 2>&1 | tail -5
```

Expected: all existing tests pass (same count as before this change).

- [ ] **Step 8: Commit**

```bash
git add crates/core/tests/conformance_reporter.rs crates/core/tests/conformance.rs
git commit -m "feat(conformance): emit JSON summary report per-category vs Google Sheets

Adds generate_conformance_report test that writes target/conformance-report.json
with per-category pass/fail counts and known deviations list.

Closes #369"
```

---

## Task 3: Open PR

- [ ] **Step 1: Push and create PR**

```bash
gh pr create \
  --repo tryganit/ganit-core \
  --title "feat(conformance): emit structured JSON summary — per-category pass/fail vs Google Sheets" \
  --assignee hhimanshu \
  --body "$(cat <<'EOF'
## Summary
- Adds `conformance_reporter.rs` with `collect_fixture_results` and `ConformanceReport`
- New `generate_conformance_report` test writes `target/conformance-report.json` after running all 30 fixture files
- JSON format: total, passed, failed, by_category breakdown, known_deviations list
- Known deviations are tracked separately and do not count as failures

## Sample output
\`\`\`json
{
  "total": 2461, "passed": 2430, "failed": 8,
  "by_category": {
    "math": { "passed": 418, "total": 420 },
    ...
  },
  "known_deviations": []
}
\`\`\`

closes #369
EOF
)"
gh pr edit --add-assignee hhimanshu
```

- [ ] **Step 2: Monitor CI**

```bash
gh run list --repo tryganit/ganit-core --limit 3
```

On failure: `gh run view <run-id> --log-failed --repo tryganit/ganit-core`
