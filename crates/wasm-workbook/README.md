# @truecalc/workbook

[![npm](https://img.shields.io/npm/v/@truecalc/workbook)](https://www.npmjs.com/package/@truecalc/workbook)
[![crates.io](https://img.shields.io/crates/v/truecalc-core)](https://crates.io/crates/truecalc-core)
[![docs.rs](https://img.shields.io/docsrs/truecalc-core)](https://docs.rs/truecalc-core)
[![license](https://img.shields.io/crates/l/truecalc-core)](LICENSE)

Spreadsheet workbook for JavaScript — full recalculation engine compiled to WebAssembly.
Manage multiple sheets, set cell values and formulas, and trigger recalculation with a
single call.

## Install

```sh
npm install @truecalc/workbook
```

## Usage

### Node.js (ESM)

```js
import init, { JsWorkbook } from '@truecalc/workbook';

await init();

const wb = new JsWorkbook('sheets');
wb.addSheet('Sheet1');
wb.set('Sheet1', 'A1', '10');
wb.set('Sheet1', 'A2', '=A1*2');
wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const val = wb.resolved('Sheet1', 'A2');
// => { type: 'number', value: 20 }
```

### Node.js (CJS)

Node.js CJS projects can use a dynamic import:

```js
async function main() {
  const { default: init, JsWorkbook } = await import('@truecalc/workbook');
  await init();

  const wb = new JsWorkbook('sheets');
  wb.addSheet('Sheet1');
  wb.set('Sheet1', 'B1', '=SUM(A1:A3)');
  wb.set('Sheet1', 'A1', '1');
  wb.set('Sheet1', 'A2', '2');
  wb.set('Sheet1', 'A3', '3');
  wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

  console.log(wb.resolved('Sheet1', 'B1'));
  // => { type: 'number', value: 6 }
}

main();
```

### Serialization

```js
const json = wb.toJSON();
const wb2 = JsWorkbook.fromJSON(json);

wb2.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));
console.log(wb2.resolved('Sheet1', 'A2'));
// => { type: 'number', value: 20 }
```

### Vite

Install the wasm plugin first:

```sh
npm install -D vite-plugin-wasm
```

Add it to `vite.config.js`:

```js
import wasm from 'vite-plugin-wasm';

export default {
  plugins: [wasm()],
};
```

Then import and use normally:

```js
import init, { JsWorkbook } from '@truecalc/workbook';

await init();

const wb = new JsWorkbook('sheets');
wb.addSheet('Sheet1');
wb.set('Sheet1', 'A1', '=IF(B1 > 0, "positive", "non-positive")');
wb.set('Sheet1', 'B1', '5');
wb.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));

const result = wb.resolved('Sheet1', 'A1');
// => { type: 'text', value: 'positive' }
```

### webpack 5

webpack 5 supports WebAssembly natively. Enable the experiment in `webpack.config.js`:

```js
module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
};
```

## API Reference

### `new JsWorkbook(flavor)`

Creates a new empty workbook.

- `flavor` — engine flavor string (use `'sheets'` for Google Sheets-compatible behavior).

```js
const wb = new JsWorkbook('sheets');
```

### `wb.addSheet(name)`

Adds a new sheet to the workbook.

- `name` — sheet name, e.g. `'Sheet1'`.

```js
wb.addSheet('Sheet1');
wb.addSheet('Summary');
```

### `wb.set(sheet, cell, value)`

Sets a cell to a literal value or formula. Formulas start with `=`.

- `sheet` — sheet name
- `cell` — A1-style cell reference, e.g. `'B2'`
- `value` — string literal or formula string

```js
wb.set('Sheet1', 'A1', '42');
wb.set('Sheet1', 'A2', '=A1 * 2');
wb.set('Sheet1', 'A3', '=SUM(A1:A2)');
```

### `wb.clear(sheet, cell)`

Clears a cell (removes its value or formula).

- `sheet` — sheet name
- `cell` — A1-style cell reference

```js
wb.clear('Sheet1', 'A1');
```

### `wb.defineName(name, expression)`

Defines a named range or formula that can be referenced by name across sheets.

- `name` — name identifier
- `expression` — formula expression string

```js
wb.defineName('TaxRate', '0.2');
wb.set('Sheet1', 'B1', '=A1 * TaxRate');
```

### `wb.recalc(context_json)`

Recalculates all formulas in the workbook. Must be called after any `set`/`clear`/`defineName`
operations before reading results.

- `context_json` — JSON string with evaluation context:
  - `timestamp_ms` — Unix timestamp in milliseconds (for `TODAY()`, `NOW()`)
  - `timezone` — IANA timezone string, e.g. `'UTC'` or `'America/New_York'`
  - `rng_seed` — integer seed for random functions (`RAND`, `RANDBETWEEN`)

```js
wb.recalc(JSON.stringify({ timestamp_ms: Date.now(), timezone: 'UTC', rng_seed: 0 }));
```

### `wb.resolved(sheet, cell)`

Returns the evaluated result for a cell after recalculation.

- `sheet` — sheet name
- `cell` — A1-style cell reference

Returns a discriminated union object tagged by `type`:

| `type`   | Shape                                                  |
|----------|--------------------------------------------------------|
| `number` | `{ type: 'number', value: 6 }`                         |
| `text`   | `{ type: 'text', value: 'yes' }`                       |
| `bool`   | `{ type: 'bool', value: true }`                        |
| `date`   | `{ type: 'date', value: 46180 }`                       |
| `error`  | `{ type: 'error', error: '#NAME?' }`                   |
| `empty`  | `{ type: 'empty' }`                                    |
| `array`  | `{ type: 'array', value: [ /* EvalResult cells */ ] }` |

```js
const result = wb.resolved('Sheet1', 'A2');
// => { type: 'number', value: 20 }
```

### `wb.toJSON()`

Serializes the entire workbook state (sheets, cell values, formulas, named ranges)
to a JSON string.

```js
const json = wb.toJSON();
```

### `JsWorkbook.fromJSON(json)`

Deserializes a workbook from a JSON string produced by `toJSON()`. Returns a new
`JsWorkbook` instance.

```js
const wb2 = JsWorkbook.fromJSON(json);
wb2.recalc(JSON.stringify({ timestamp_ms: 0, timezone: 'UTC', rng_seed: 0 }));
```
