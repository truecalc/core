# truecalc-core

Spreadsheet formula engine for Python, with exact Google Sheets semantics.

Evaluates the same formula language a spreadsheet does — a comprehensive
function library, conformance-tested against real Google Sheets output — with
no spreadsheet application, no network, and no file format involved.

```sh
pip install truecalc-core
```

```python
from truecalc.core import Engine

engine = Engine.sheets()

engine.evaluate("=SUM(A1,A2,A3)", {"A1": 10, "A2": 20, "A3": 30})
# 60.0

engine.evaluate('=IF(A1>100,"big","small")', {"A1": 150})
# 'big'

engine.evaluate("=TEXT(A1,\"0.00%\")", {"A1": 0.1234})
# '12.34%'
```

## Errors are values, not exceptions

`=1/0` is `#DIV/0!` — a *result*, not a crash. Spreadsheet formulas branch on
errors routinely (`IFERROR`, `ISNA`), and a `SUM` over a range containing one
propagates it. So errors come back as values:

```python
result = engine.evaluate("=1/0")
# Error('#DIV/0!')

result.code       # '#DIV/0!'
bool(result)      # False
```

Opt into exceptions per call when that suits the calling code better:

```python
engine.evaluate("=1/0", raise_on_error=True)
# truecalc.core.FormulaError: #DIV/0!
```

Every spreadsheet error raises the same `FormulaError`, so one `except` clause
catches all of them.

A formula that does not parse is *also* a value — `Error('#VALUE!')` — matching
the Rust and JS surfaces and Sheets itself, which shows an error in the cell
rather than refusing the input. Call `validate()` if you want to check first.

## Types

| Engine value | Python |
|---|---|
| number | `float` |
| text | `str` |
| boolean | `bool` |
| blank | `None` |
| array | `list` (nested for 2-D) |
| date | `Date` — carries `.serial`, converts via `.to_datetime()` |
| zoned instant | `Zoned` — RFC-9557 string |
| error | `Error` — `.code`, `.message` |
| sparkline | `Sparkline` — `.chart_type`, `.data`, `.options` |

`Date` is deliberately not a bare `float`: the engine distinguishes a serial
date from a plain number (`ISDATE` tells them apart), and collapsing them would
erase that.

## Engine flavors

The flavor is required and immutable — build one engine and reuse it.

| Constructor | Target |
|---|---|
| `Engine.sheets()` | Google Sheets |
| `Engine.excel()` | Excel (parse and validate only; evaluation pending) |

## Also available

- `evaluate(formula, variables)` — module-level shortcut over `Engine.sheets()`
- `validate(formula)` — returns `None` when valid, else the parse error
- `list_functions()` — every implemented function with category and signature
- `Engine.translate(formula, d_row, d_col)` — fill/copy reference adjustment
- `Engine.rename_sheet_refs(formula, old, new)` — sheet-rename rewrite

## Related packages

Same engine, other ecosystems:

- [`@truecalc/core`](https://www.npmjs.com/package/@truecalc/core) — JavaScript / TypeScript
- [`truecalc-core`](https://crates.io/crates/truecalc-core) — Rust

## License

MIT
