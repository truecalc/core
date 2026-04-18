# CI Infrastructure: nextest + Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `cargo test` with `cargo-nextest` for richer test output and inline PR annotations (#367), then add `cargo-llvm-cov` + codecov for per-PR coverage delta comments with an 80% gate on `ganit-core` (#368).

**Architecture:** Two CI steps replace/extend the existing "Test" step. nextest runs first and writes JUnit XML; a separate coverage step runs llvm-cov and uploads LCOV to codecov.io. Both changes live entirely in `.github/workflows/ci.yml` and new config files.

**Tech Stack:** cargo-nextest, cargo-llvm-cov, dorny/test-reporter GitHub Action, codecov/codecov-action

**GitHub issues:** Closes #367 and #368 (sub-issues of epic #366)

---

## File Map

| Action | File | Purpose |
|--------|------|---------|
| Create | `.config/nextest.toml` | nextest CI profile — JUnit XML output path |
| Modify | `.github/workflows/ci.yml` | Replace `cargo test`, add nextest + test-reporter + llvm-cov + codecov steps |
| Create | `codecov.yml` | Coverage gate: 80% patch minimum on `ganit-core` |

---

## Task 1: Add nextest config file

**Files:**
- Create: `.config/nextest.toml`

- [ ] **Step 1: Create the nextest config**

```toml
# .config/nextest.toml
[profile.ci]
fail-fast = false

[profile.ci.junit]
path = "junit.xml"
```

- [ ] **Step 2: Verify nextest is installed locally and config is valid**

```bash
cargo install cargo-nextest --locked 2>/dev/null || true
cargo nextest run --workspace --profile ci --no-run 2>&1 | head -5
```

Expected: something like `Compiling ...` or `Finished test` — no error about invalid config.

- [ ] **Step 3: Commit**

```bash
git add .config/nextest.toml
git commit -m "chore: add nextest CI profile with JUnit XML output"
```

---

## Task 2: Update CI to use nextest + test-reporter

**Files:**
- Modify: `.github/workflows/ci.yml`

Current "Test" step (lines 22-23):
```yaml
      - name: Test
        run: cargo test --workspace
```

- [ ] **Step 1: Replace the Test step and add test-reporter**

Replace the entire `Test` step with:

```yaml
      - name: Install nextest
        uses: taiki-e/install-action@nextest

      - name: Test
        run: cargo nextest run --workspace --profile ci
        env:
          NEXTEST_EXPERIMENTAL_LIBTEST_JSON: 1

      - name: Publish test results
        uses: dorny/test-reporter@v1
        if: always()
        with:
          name: ganit-core tests
          path: target/nextest/ci/junit.xml
          reporter: java-junit
          fail-on-error: false
```

- [ ] **Step 2: Verify the full updated ci.yml is valid YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "YAML valid"
```

Expected: `YAML valid`

- [ ] **Step 3: Run nextest locally to confirm all tests still pass**

```bash
cargo nextest run --workspace --profile ci 2>&1 | tail -5
```

Expected output ends with something like:
```
    Summary [  X.Xs] 2461 tests run: 2461 passed, 0 skipped
```

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): switch to cargo-nextest with JUnit XML and PR test annotations

Closes #367"
```

---

## Task 3: Add codecov.yml coverage gate

**Files:**
- Create: `codecov.yml`

- [ ] **Step 1: Create the codecov config**

```yaml
# codecov.yml
coverage:
  status:
    patch:
      default:
        target: 80%
        only_pulls: true
        informational: false
    project:
      ganit-core:
        target: 80%
        paths:
          - crates/core/
```

- [ ] **Step 2: Commit**

```bash
git add codecov.yml
git commit -m "chore: add codecov config with 80% coverage gate on ganit-core"
```

---

## Task 4: Add llvm-cov + codecov steps to CI

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add llvm-cov install and coverage steps after the Test step**

After the `Publish test results` step, add:

```yaml
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage report
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info

      - name: Upload coverage to codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: false
          token: ${{ secrets.CODECOV_TOKEN }}
```

- [ ] **Step 2: Verify YAML is still valid**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "YAML valid"
```

Expected: `YAML valid`

- [ ] **Step 3: Run llvm-cov locally to verify it produces output**

```bash
cargo llvm-cov --workspace --lcov --output-path lcov.info 2>&1 | tail -3
ls -lh lcov.info
```

Expected: `lcov.info` exists and is non-empty (several KB at minimum).

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): add llvm-cov coverage measurement and codecov upload

Closes #368"
```

---

## Task 5: Open PR

- [ ] **Step 1: Push branch and open PR**

```bash
gh pr create \
  --repo tryganit/ganit-core \
  --title "feat(ci): nextest + llvm-cov coverage — inline PR annotations and 80% gate" \
  --assignee hhimanshu \
  --body "$(cat <<'EOF'
## Summary
- Replaces `cargo test` with `cargo-nextest` for per-test timing and inline PR failure annotations (via JUnit XML + dorny/test-reporter)
- Adds `cargo-llvm-cov` + codecov integration with an 80% coverage gate on `ganit-core`

## What a failing test looks like in a PR now
The `dorny/test-reporter` action parses JUnit XML and posts inline annotations on the PR diff showing exactly which file/line failed and what the expected vs actual values were.

## Coverage PR comment
codecov will comment on every PR with a per-crate coverage delta table.

closes #367
closes #368
EOF
)"
gh pr edit --add-assignee hhimanshu
```

- [ ] **Step 2: Monitor CI**

```bash
gh run list --repo tryganit/ganit-core --limit 3
```

Wait for the run triggered by the PR push. If it fails:
```bash
gh run view <run-id> --log-failed --repo tryganit/ganit-core
```

Fix the root cause and push a new commit (do not amend).
