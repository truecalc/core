# @truecalc/core

Spreadsheet formula engine with exact Google Sheets semantics, for Deno.

A comprehensive function library, conformance-tested against real Google Sheets
output, compiled to WebAssembly. No spreadsheet application, no network, no file
format.

```sh
deno add jsr:@truecalc/core
```

```ts
import { evaluate, createEngine, validate } from "@truecalc/core";

evaluate("=SUM(A1,A2)", { A1: 10, A2: 20 });
// { type: "number", value: 30 }

createEngine("google-sheets").evaluate('=UPPER("ok")', {});
// { type: "text", value: "OK" }
```

**No `init()` call and no permission flags.** The module is built for wasm ESM
integration, so Deno instantiates the WebAssembly as part of the module graph —
`deno run your-script.ts` works with no `--allow-net` or `--allow-read`.

> This differs from the npm build's historical usage. If you are following an
> example that calls `await init()` first, that example predates this package.

## Results

Every result is a discriminated union tagged by `type`:

```ts
{ type: "number", value: 1.5 }
{ type: "text",   value: "yes" }
{ type: "bool",   value: true }
{ type: "empty" }
{ type: "date",   value: 46180 }   // spreadsheet serial; day 0 = 1899-12-30
{ type: "error",  error: "#REF!" } // plus an optional `message`
{ type: "array",  value: [ /* recursive: 2-D nests row sub-arrays */ ] }
```

Errors are **values**, not thrown — `=1/0` returns `{ type: "error", error:
"#DIV/0!" }`. Spreadsheet formulas branch on errors (`IFERROR`, `ISNA`) and
`SUM` propagates them, so throwing would diverge from the semantics being
modelled.

## Exports

| Export | Purpose |
|---|---|
| `evaluate(formula, variables)` | Evaluate against Google Sheets conformance |
| `createEngine(target)` | An engine locked to a flavor (`"google-sheets"`) |
| `validate(formula)` | Parse check without evaluating |
| `list_functions()` | A hardcoded subset (64) of the function catalogue — see [#810](https://github.com/truecalc/core/issues/810) |
| `translate_formula(formula, dRow, dCol)` | Fill / copy reference adjustment |
| `rename_sheet_refs(formula, old, new)` | Sheet-rename reference rewrite |

## Same engine, other ecosystems

- [`@truecalc/core`](https://www.npmjs.com/package/@truecalc/core) — npm
- [`truecalc-core`](https://crates.io/crates/truecalc-core) — Rust
- [`@truecalc/workbook`](https://jsr.io/@truecalc/workbook) — multi-sheet workbooks with recalc

## License

MIT
