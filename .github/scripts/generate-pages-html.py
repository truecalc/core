#!/usr/bin/env python3
# Generates index.html for gh-pages with Google Sheets Conformance + Test Coverage by Category.
# Usage: generate-pages-html.py <junit.xml> <conformance.json> <run_url> <hex> <passed> <total> <pct>

import sys
import json
import xml.etree.ElementTree as ET
from collections import defaultdict
from datetime import datetime, timezone

junit_path, conf_path, run_url, hex_color, passed, total, pct = sys.argv[1:8]

# ── Unit tests per category ───────────────────────────────────────────────────
tree = ET.parse(junit_path)
root = tree.getroot()
unit_counts = defaultdict(int)
for suite in root.findall('testsuite'):
    if suite.get('name') == 'truecalc-core':
        for tc in suite.findall('testcase'):
            parts = tc.get('name', '').split('::')
            cat = 'core'
            if 'functions' in parts:
                idx = parts.index('functions')
                if idx + 1 < len(parts) and parts[idx + 1] != 'tests':
                    cat = parts[idx + 1]
            unit_counts[cat] += 1

# ── Proptest suites per category ──────────────────────────────────────────────
SKIP = {'error_propagation', 'conformance'}
CASES = 500
prop_fns = defaultdict(int)
for suite in root.findall('testsuite'):
    sname = suite.get('name', '')
    if sname.startswith('truecalc-core::property_'):
        cat = sname[len('truecalc-core::property_'):]
        if cat not in SKIP:
            prop_fns[cat] += int(suite.get('tests', 0))

# ── Conformance data ──────────────────────────────────────────────────────────
conf = json.load(open(conf_path))
cats_conf = conf.get('by_category', {})

all_cats = sorted(
    (set(unit_counts.keys()) | set(prop_fns.keys()) | set(cats_conf.keys())) - {'core'}
)

# ── Build table rows ──────────────────────────────────────────────────────────
rows_html = []
tot_unit = tot_prop = tot_grand = 0

for cat in all_cats:
    unit = unit_counts.get(cat, 0)
    fn_c = prop_fns.get(cat, 0)
    cd = cats_conf.get(cat)
    cp = cd['passed'] if cd else 0
    ct = cd['total'] if cd else 0

    if cd:
        cls = 'pass' if cp == ct else 'fail'
        mark = '✓' if cp == ct else '⚠'
        conf_cell = f'<span class="{cls}">{cp:,}/{ct:,} {mark}</span>'
    else:
        conf_cell = '—'

    if fn_c:
        prop_cases = fn_c * CASES
        prop_cell = f'{prop_cases:,} ({fn_c}×{CASES:,})'
    else:
        prop_cases = 0
        prop_cell = '—'

    row_total = unit + ct + prop_cases
    display = cat.replace('_', ' ').title()
    rows_html.append(
        f'<tr><td>{display}</td><td>{unit:,}</td>'
        f'<td>{conf_cell}</td><td>{prop_cell}</td><td>{row_total:,}</td></tr>'
    )
    tot_unit += unit
    tot_prop += prop_cases
    tot_grand += unit + ct + prop_cases

core_unit = unit_counts.get('core', 0)
tot_unit += core_unit
tot_grand += core_unit
tot_prop_fns = sum(prop_fns.values())

updated = datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')

print(f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Test Transparency — truecalc</title>
  <style>
    body {{ font-family: system-ui, sans-serif; max-width: 900px; margin: 40px auto; padding: 0 20px; color: #24292f; }}
    h1 {{ font-size: 1.5rem; }}
    h2 {{ font-size: 1.1rem; margin-top: 2rem; color: #57606a; }}
    .badges {{ margin: 8px 0 24px; display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }}
    .badge {{ display: inline-block; background: {hex_color}; color: #fff; padding: 4px 12px; border-radius: 4px; font-size: 1.1rem; font-weight: bold; }}
    .explainer {{ background: #f6f8fa; border: 1px solid #d0d7de; border-radius: 6px; padding: 12px 16px; margin: 16px 0; font-size: 0.9rem; line-height: 1.6; }}
    .explainer strong {{ color: #24292f; }}
    table {{ border-collapse: collapse; width: 100%; margin-top: 8px; }}
    th, td {{ border: 1px solid #d0d7de; padding: 8px 12px; text-align: left; }}
    th {{ background: #f6f8fa; }}
    tfoot td {{ font-weight: bold; background: #f6f8fa; }}
    .pass {{ color: #1a7f37; }}
    .fail {{ color: #cf222e; }}
    footer {{ margin-top: 32px; font-size: 0.85rem; color: #57606a; }}
  </style>
</head>
<body>
  <h1>truecalc — Test Transparency</h1>

  <h2>Google Sheets Conformance</h2>
  <div class="badges">
    <span class="badge">{passed}/{total} · {pct}%</span>
    <img src="https://img.shields.io/codecov/c/github/truecalc/core?label=coverage&style=flat-square" alt="Code Coverage" height="22">
  </div>

  <div class="explainer">
    <strong>What is Google Sheets conformance?</strong><br>
    truecalc evaluates spreadsheet formulas. To verify correctness, every supported formula is run against
    a <em>Google Sheets oracle</em> — real Google Sheets spreadsheets that produce the expected output.
    On every commit to <code>main</code>, truecalc re-runs all {total} conformance cases and compares
    results. A ✓ means truecalc matches Google Sheets exactly; ⚠ means a known, intentional deviation
    (e.g. a locale difference or an unsupported edge case).<br><br>
    <strong>Property tests</strong> go further: for each formula category, randomly generated inputs
    are checked against mathematical invariants (e.g. <code>ABS(x) ≥ 0</code> for all x,
    <code>SQRT(x)² ≈ x</code> for x &gt; 0). Each property runs {CASES:,} random cases per commit.
  </div>

  <h2>Test Coverage by Category</h2>
  <table>
    <thead>
      <tr>
        <th>Category</th>
        <th>Unit Tests</th>
        <th>Google Sheets Conformance</th>
        <th>Property Cases</th>
        <th>Total</th>
      </tr>
    </thead>
    <tbody>
      {''.join(rows_html)}
    </tbody>
    <tfoot>
      <tr>
        <td>Total</td>
        <td>{tot_unit:,}</td>
        <td>{conf['passed']:,}/{conf['total']:,}</td>
        <td>{tot_prop:,} ({tot_prop_fns}×{CASES:,})</td>
        <td>~{tot_grand:,}</td>
      </tr>
    </tfoot>
  </table>

  <footer>
    Oracle: Google Sheets &nbsp;·&nbsp;
    ✓ = 100% passing &nbsp;·&nbsp; ⚠ = known deviation &nbsp;·&nbsp;
    Updated: {updated} &nbsp;·&nbsp;
    <a href="{run_url}">CI run</a>
  </footer>
</body>
</html>""")
