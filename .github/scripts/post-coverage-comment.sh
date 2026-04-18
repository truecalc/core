#!/usr/bin/env bash
# .github/scripts/post-coverage-comment.sh
#
# Builds a "Test Coverage by Category" table from three data sources and
# posts it as a PR comment (idempotent via --edit-last).
#
# Usage:
#   post-coverage-comment.sh <junit.xml> <conformance-report.json> [pr-number]
#
# Environment:
#   GITHUB_REPOSITORY  — set automatically by GitHub Actions (owner/repo)
#   GH_TOKEN           — set automatically by GitHub Actions

set -euo pipefail

JUNIT_XML="${1:-target/nextest/ci/junit.xml}"
CONFORMANCE_JSON="${2:-target/conformance-report.json}"
PR_NUMBER="${3:-}"

if [[ ! -f "$JUNIT_XML" ]]; then
  echo "::warning::junit.xml not found at $JUNIT_XML — skipping coverage comment"
  exit 0
fi

if [[ ! -f "$CONFORMANCE_JSON" ]]; then
  echo "::warning::conformance-report.json not found at $CONFORMANCE_JSON — skipping coverage comment"
  exit 0
fi

# Build the markdown table with Python (available on ubuntu-latest)
COMMENT=$(python3 - "$JUNIT_XML" "$CONFORMANCE_JSON" <<'PYEOF'
import sys, json, xml.etree.ElementTree as ET
from collections import defaultdict

junit_path, conf_path = sys.argv[1], sys.argv[2]

# ── 1. Parse unit tests ───────────────────────────────────────────────────────
tree = ET.parse(junit_path)
root = tree.getroot()

unit_counts = defaultdict(int)
for suite in root.findall('testsuite'):
    if suite.get('name') == 'truecalc-core':
        for tc in suite.findall('testcase'):
            name = tc.get('name', '')
            parts = name.split('::')
            cat = 'core'
            if 'functions' in parts:
                idx = parts.index('functions')
                if idx + 1 < len(parts):
                    candidate = parts[idx + 1]
                    # guard against eval::functions::tests::list_functions_matches_registry
                    if candidate != 'tests':
                        cat = candidate
            unit_counts[cat] += 1

# ── 2. Parse proptest suites ──────────────────────────────────────────────────
# Skip cross-cutting suites that would double-count
SKIP_SUITES = {'error_propagation', 'conformance'}
CASES_PER_PROP = 500

prop_fn_counts = defaultdict(int)  # number of property test functions per category
for suite in root.findall('testsuite'):
    sname = suite.get('name', '')
    if sname.startswith('truecalc-core::property_'):
        cat = sname[len('truecalc-core::property_'):]
        if cat not in SKIP_SUITES:
            prop_fn_counts[cat] += int(suite.get('tests', 0))

# ── 3. Load conformance data ──────────────────────────────────────────────────
conf = json.load(open(conf_path))
cats_conf = conf.get('by_category', {})
total_conf_passed = conf.get('passed', 0)
total_conf_total = conf.get('total', 0)

# ── 4. Build sorted category list (exclude 'core') ───────────────────────────
all_cats = sorted(
    set(unit_counts.keys()) | set(prop_fn_counts.keys()) | set(cats_conf.keys()) - {'core'}
)
# Ensure 'core' is not in the main table (it's a meta bucket)
all_cats = [c for c in all_cats if c != 'core']

# ── 5. Render table ───────────────────────────────────────────────────────────
rows = []
total_unit = 0
total_prop_cases = 0
total_grand = 0

for cat in all_cats:
    unit = unit_counts.get(cat, 0)
    fn_count = prop_fn_counts.get(cat, 0)
    conf_data = cats_conf.get(cat, None)

    # Conformance cell
    if conf_data:
        cp, ct = conf_data['passed'], conf_data['total']
        mark = '✓' if cp == ct else '⚠'
        conf_cell = f'{cp:,}/{ct:,} {mark}'
    else:
        cp, ct = 0, 0
        conf_cell = '—'

    # Proptest cell
    if fn_count:
        prop_cases = fn_count * CASES_PER_PROP
        prop_cell = f'{prop_cases:,} ({fn_count}×{CASES_PER_PROP:,})'
    else:
        prop_cases = 0
        prop_cell = '—'

    row_total = unit + ct + prop_cases
    display = cat.replace('_', ' ').title()

    rows.append(f'| {display} | {unit:,} | {conf_cell} | {prop_cell} | {row_total:,} |')

    total_unit += unit
    total_prop_cases += prop_cases
    total_grand += unit + ct + prop_cases

# Add core to totals (not shown as a separate row)
core_unit = unit_counts.get('core', 0)
total_unit += core_unit
total_grand += core_unit

total_prop_fns = sum(prop_fn_counts.values())
prop_total_cell = f'{total_prop_cases:,} ({total_prop_fns}×{CASES_PER_PROP:,})'
conf_total_cell = f'{total_conf_passed:,}/{total_conf_total:,}'

footer = (
    f'| **Total** | **{total_unit:,}** | **{conf_total_cell}** '
    f'| **{prop_total_cell}** | **~{total_grand:,}** |'
)

table = '\n'.join(rows)

print(f'''## Test Coverage by Category

| Category | Unit Tests | Google Sheets Conformance | Property Cases | Total |
|----------|-----------|--------------------------|----------------|-------|
{table}
{footer}

<sub>Oracle: Google Sheets · ✓ = 100% passing · ⚠ = known deviation</sub>''')
PYEOF
)

echo "$COMMENT"

# Post comment only on actual PRs
if [[ -n "$PR_NUMBER" && "$PR_NUMBER" != "0" ]]; then
  echo "$COMMENT" | gh pr comment "$PR_NUMBER" \
    --repo "$GITHUB_REPOSITORY" \
    --body-file - \
    --edit-last 2>/dev/null || \
  echo "$COMMENT" | gh pr comment "$PR_NUMBER" \
    --repo "$GITHUB_REPOSITORY" \
    --body-file -
fi
