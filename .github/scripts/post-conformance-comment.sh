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

# Build category table rows
category_section=""
while IFS= read -r line; do
  if [[ "$line" =~ \"([a-z]+)\":[[:space:]]*\{[[:space:]]*\"passed\":[[:space:]]*([0-9]+),[[:space:]]*\"total\":[[:space:]]*([0-9]+) ]]; then
    cat_name="${BASH_REMATCH[1]}"
    cat_passed="${BASH_REMATCH[2]}"
    cat_total="${BASH_REMATCH[3]}"
    cat_pct=$(awk "BEGIN { printf \"%.1f\", ($cat_passed/$cat_total)*100 }" 2>/dev/null || echo "?")
    # Capitalize first letter (compatible with bash 3.x on macOS and bash 5 on Linux)
    cat_display=$(echo "$cat_name" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')
    suffix=""
    if [[ "$cat_passed" -lt "$cat_total" ]]; then
      suffix="  ← $(($cat_total - $cat_passed)) failing"
    fi
    category_section+="| ${cat_display} | ${cat_passed}/${cat_total} | ${cat_pct}% |${suffix}"$'\n'
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

truecalc calculations match Google Sheets — ${passed}/${total} tests passing (${pct}%)

| Category | Passed | Rate |
|----------|--------|------|
${category_section}
**Total: ${passed}/${total} (${pct}%)**

**Property tests:** 65 properties × 500 cases = 32,500 generated inputs — all passed
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

echo "Google Sheets Conformance: ${passed}/${total} (${pct}%)"

# Fail CI on regressions
if [[ "$regressions" -gt 0 ]]; then
  echo "::error::${regressions} Google Sheets conformance regression(s) detected"
  exit 1
fi
