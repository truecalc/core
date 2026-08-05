# @truecalc/workbook

Multi-sheet spreadsheet workbooks with dependency-ordered recalculation, for
Deno. Built on the same engine as [`@truecalc/core`](https://jsr.io/@truecalc/core).

```sh
deno add jsr:@truecalc/workbook
```

```ts
import { JsWorkbook } from "@truecalc/workbook";

const wb = new JsWorkbook("sheets");
wb.addSheet("Budget");
wb.set("Budget", "A1", "100");
wb.set("Budget", "A2", "200");
wb.set("Budget", "A3", "=SUM(A1:A2)");

// The recalc context is required in full — it pins the clock, timezone and RNG
// seed so volatile functions (NOW, TODAY, RAND) are deterministic.
const changes = wb.recalc(JSON.stringify({
  timestamp_ms: Date.now(),
  timezone: "UTC",
  rng_seed: 0,
}));

wb.resolved("Budget", "A3");
// '{"type":"number","value":300.0}'   <- a JSON string; parse it if you need the object

changes;
// '[{"sheet":"Budget","addr":"A3","old":{"type":"empty"},"new":{"type":"number","value":300.0}}]'

wb.free();
```

**No `init()` call and no permission flags.** The module is built for wasm ESM
integration, so Deno instantiates the WebAssembly as part of the module graph —
`deno run your-script.ts` needs no `--allow-net` or `--allow-read`.

> **This is the important difference from the npm build.** The npm package is
> built for a different loader and requires `import init, { JsWorkbook } from
> '@truecalc/workbook'; await init();` before use. On JSR there is **no default
> export** — importing one fails with *"does not provide an export named
> 'default'"*. Use the named imports above.

## API

`JsWorkbook`:

| Method | Purpose |
|---|---|
| `new JsWorkbook(engine)` | Create a workbook locked to an engine flavor |
| `addSheet(name)` | Add a worksheet |
| `set(sheet, a1, input)` | Set a cell — literal or `=formula` |
| `setDate(sheet, a1, serial)` | Set a cell to a spreadsheet serial date |
| `clear(sheet, a1)` | Clear a cell |
| `recalc(contextJson)` | Recalculate in dependency order; returns changes as a JSON string. The context must supply `timestamp_ms`, `timezone` and `rng_seed` — all three, or it throws |
| `resolved(sheet, a1)` | The computed value of a cell, as a JSON string |
| `defineName(name, ref)` / `redefineName(name, ref)` | Named ranges |
| `toJSON()` / `JsWorkbook.fromJSON(s)` | Serialise and restore |
| `free()` | Release the underlying wasm memory |

Also exported: `translateFormula(formula, dRow, dCol)` — fill / copy reference
adjustment.

Because the workbook holds wasm-side state, call `free()` when you are done with
one in a long-running process.

## Same engine, other ecosystems

- [`@truecalc/workbook`](https://www.npmjs.com/package/@truecalc/workbook) — npm (requires `await init()`)
- [`truecalc-workbook`](https://crates.io/crates/truecalc-workbook) — Rust
- [`@truecalc/core`](https://jsr.io/@truecalc/core) — stateless formula evaluation only

## License

MIT
