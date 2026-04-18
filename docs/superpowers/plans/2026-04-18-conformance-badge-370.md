# Conformance Badge + PR Comment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** After every CI run, post a Google Sheets conformance table as a PR comment (with regression detection) and add a live badge to the README.

**Architecture:** CI reads `target/conformance-report.json` (produced by issue #369), computes deltas against the main branch report, posts a PR comment via `gh pr comment`, and updates a Shields.io endpoint-based badge in `README.md`. Regressions (tests that passed on main but fail on the PR) cause CI to exit non-zero.

**Tech Stack:** Bash CI script, `gh` CLI, Shields.io endpoint badges, GitHub Actions

**GitHub issue:** Closes #370 (sub-issue of epic #366)

**Prerequisites:** Issues #367 (nextest), #368 (coverage), #369 (conformance reporter) must be merged before this plan is implemented.

---

## File Map

| Action | File | Purpose |
|--------|------|---------|
| Create | `.github/scripts/post-conformance-comment.sh` | Read JSON, compute delta, post PR comment, fail on regression |
| Modify | `.github/workflows/ci.yml` | Add steps: run reporter, download main baseline, call script |
| Modify | `README.md` | Add conformance badge |

---

## Task 1: Write the comment script

**Files:**
- Create: `.github/scripts/post-conformance-comment.sh`

- [ ] **Step 1: Create the scripts directory and write the script**

```bash
mkdir -p .github/scripts
```

```bash
#!/usr/bin/env bash
# .github/scripts/post-conformance-comment.sh
#
# Reads target/conformance-report.json, optionally compares to a baseline
# (from main branch), posts a PR comment, and exits 1 on regressions.
#
# Usage:
#   post-conformance-comment.sh <report.json> [baseline.json] [pr-number]
#
# Environment:
#   GITHUB_REPOSITORY  — set automatically by GitHub Actions (owner/repo)
#   GH_TOKEN           — set automatically by GitHub Actions

set -euo pipefail

REPORT="${1:-target/conformance-report.json}"
BASELINE="${2:-}"
PR_NUMBER="${3:-}"

if [[ ! -f "$REPORT" ]]; then
  echo "::warning::conformance-report.json not found at $REPORT — skipping comment"
  exit 0
fi

# Parse JSON with pure bash (no jq dependency)
total=$(grep '"total"'   "$REPORT" | grep -o '[0-9]*' | head -1)
passed=$(grep '"passed"' "$REPORT" | grep -o '[0-9]*' | head -1)
failed=$(grep '"failed"' "$REPORT" | grep -o '[0-9]*' | head -1)

pct=$(awk "BEGIN { printf \"%.1f\", ($passed/$total)*100 }" 2>/dev/null || echo "?")

# Build category table
category_section=""
while IFS= read -r line; do
  if [[ "$line" =~ \"([a-z]+)\":[[:space:]]*\{[[:space:]]*\"passed\":[[:space:]]*([0-9]+),[[:space:]]*\"total\":[[:space:]]*([0-9]+) ]]; then
    cat_name="${BASH_REMATCH[1]}"
    cat_passed="${BASH_REMATCH[2]}"
    cat_total="${BASH_REMATCH[3]}"
    cat_pct=$(awk "BEGIN { printf \"%.1f\", ($cat_passed/$cat_total)*100 }" 2>/dev/null || echo "?")
    suffix=""
    if [[ "$cat_passed" -lt "$cat_total" ]]; then
      suffix="  ← $(($cat_total - $cat_passed)) failing"
    fi
    category_section+="| ${cat_name^} | ${cat_passed}/${cat_total} | ${cat_pct}% |${suffix}"$'\n'
  fi
done < "$REPORT"

# Regression detection
regressions=0
regression_section=""
if [[ -n "$BASELINE" && -f "$BASELINE" ]]; then
  baseline_passed=$(grep '"passed"' "$BASELINE" | grep -o '[0-9]*' | head -1)
  if [[ "$passed" -lt "$baseline_passed" ]]; then
    regressions=$(($baseline_passed - $passed))
    regression_section="⚠️ **${regressions} regression(s) detected** — this PR breaks previously passing tests vs main."
  fi
fi

# Build comment body
comment="## Google Sheets Conformance

ganit calculations match Google Sheets — ${passed}/${total} tests passing (${pct}%)

| Category | Passed | Rate |
|----------|--------|------|
${category_section}

**Total: ${passed}/${total} (${pct}%)**
"

if [[ -n "$regression_section" ]]; then
  comment+="
${regression_section}"
fi

comment+="
<sub>Oracle: Google Sheets · Verified on every PR</sub>"

# Post comment (only on actual PRs, not pushes to main)
if [[ -n "$PR_NUMBER" && "$PR_NUMBER" != "0" ]]; then
  echo "$comment" | gh pr comment "$PR_NUMBER" \
    --repo "$GITHUB_REPOSITORY" \
    --body-file - \
    --edit-last 2>/dev/null || \
  echo "$comment" | gh pr comment "$PR_NUMBER" \
    --repo "$GITHUB_REPOSITORY" \
    --body-file -
fi

echo "Conformance: ${passed}/${total} (${pct}%)"

# Fail CI on regressions
if [[ "$regressions" -gt 0 ]]; then
  echo "::error::${regressions} Google Sheets conformance regression(s) detected"
  exit 1
fi
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x .github/scripts/post-conformance-comment.sh
```

- [ ] **Step 3: Test locally (dry run — no PR comment posted)**

```bash
# First generate a report
cargo test -p ganit-core --test conformance generate_conformance_report -- --nocapture 2>/dev/null

# Run the script without a PR number (skips posting)
bash .github/scripts/post-conformance-comment.sh target/conformance-report.json "" ""
```

Expected output: `Conformance: XXXX/XXXX (XX.X%)`

- [ ] **Step 4: Commit**

```bash
git add .github/scripts/post-conformance-comment.sh
git commit -m "chore: add conformance comment script — posts GS match table to PRs, fails on regressions"
```

---

## Task 2: Add CI steps for conformance reporting

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add conformance report steps to `ci.yml`**

After the existing test step (nextest), add:

```yaml
      - name: Generate conformance report
        run: cargo test -p ganit-core --test conformance generate_conformance_report -- --nocapture

      - name: Download main-branch conformance baseline
        run: |
          gh run download \
            --repo ${{ github.repository }} \
            --name conformance-report \
            --dir baseline/ \
            2>/dev/null || echo "No baseline found (first run or main has no artifact)"
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        continue-on-error: true

      - name: Upload conformance report as artifact
        uses: actions/upload-artifact@v4
        with:
          name: conformance-report
          path: target/conformance-report.json
          retention-days: 30

      - name: Post conformance PR comment
        if: github.event_name == 'pull_request'
        run: |
          bash .github/scripts/post-conformance-comment.sh \
            target/conformance-report.json \
            baseline/conformance-report.json \
            ${{ github.event.pull_request.number }}
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          GITHUB_REPOSITORY: ${{ github.repository }}
```

- [ ] **Step 2: Verify YAML is valid**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "YAML valid"
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): post Google Sheets conformance table on PRs with regression detection

Closes #370"
```

---

## Task 3: Add conformance badge to README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Read the current top of README.md to find where badges live**

```bash
head -10 README.md
```

- [ ] **Step 2: Add the conformance badge after existing badges**

Find the line with existing badges (likely shields.io img tags or markdown badge syntax). After the last existing badge, add:

```markdown
[![Google Sheets Conformance](https://img.shields.io/badge/Google%20Sheets%20conformance-passing-brightgreen)](https://github.com/tryganit/ganit-core/actions)
```

Note: This is a static badge initially. It will show "passing" — the exact count can be made dynamic in a follow-up using a Shields.io endpoint badge once a public endpoint is set up. For now the static badge communicates the intent clearly.

- [ ] **Step 3: Verify README renders correctly**

```bash
head -15 README.md
```

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: add Google Sheets conformance badge to README"
```

---

## Task 4: Open PR

- [ ] **Step 1: Push and create PR**

```bash
gh pr create \
  --repo tryganit/ganit-core \
  --title "feat(ci): Google Sheets conformance badge + PR regression comment" \
  --assignee hhimanshu \
  --body "$(cat <<'EOF'
## Summary
- New script `.github/scripts/post-conformance-comment.sh` reads `target/conformance-report.json` and posts a conformance table on every PR
- Regression detection: if this PR breaks previously passing tests vs main, CI fails with a clear error
- Conformance report is uploaded as a CI artifact on every run (used as baseline for future PRs)
- Google Sheets conformance badge added to README

## Sample PR comment
```
## Google Sheets Conformance
ganit calculations match Google Sheets — 2430/2461 tests passing (98.7%)

| Category | Passed | Rate |
|----------|--------|------|
| Math | 418/420 | 99.5% |
| Text | 342/342 | 100.0% |
| ...
```

## Dependencies
Requires #367, #368, #369 merged first.

closes #370
EOF
)"
gh pr edit --add-assignee hhimanshu
```

- [ ] **Step 2: Monitor CI**

```bash
gh run list --repo tryganit/ganit-core --limit 3
```

On failure: `gh run view <run-id> --log-failed --repo tryganit/ganit-core`
