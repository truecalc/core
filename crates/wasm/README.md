# @truecalc/core

[![npm](https://img.shields.io/npm/v/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![npm downloads](https://img.shields.io/npm/dm/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![crates.io](https://img.shields.io/crates/v/truecalc-core)](https://crates.io/crates/truecalc-core)
[![docs.rs](https://img.shields.io/docsrs/truecalc-core)](https://docs.rs/truecalc-core)
[![license](https://img.shields.io/crates/l/truecalc-core)](LICENSE)
[![functions](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/functions-badge.json)](https://truecalc.github.io/core/)

WebAssembly-powered spreadsheet formula engine for JavaScript/TypeScript.

A comprehensive library of spreadsheet functions (see the live count above). Runs in Node.js, Bun, Deno, and the browser — no server needed. Ground-truth conformance against real Google Sheets. The same engine is also available as a [Rust crate](https://crates.io/crates/truecalc-core) and as an [MCP server](https://crates.io/crates/truecalc-mcp) for AI assistants.

```js
const { evaluate } = require('@truecalc/core');
evaluate('SUM(A1, B1)', { A1: 100, B1: 200 })
// => { type: 'number', value: 300 }
```

## Install

```sh
npm install @truecalc/core
```

## Usage

### Node.js (CJS)

Works out of the box — no bundler configuration needed.

```js
const { evaluate, validate, list_functions } = require('@truecalc/core');

const result = evaluate('SUM(A1, B1)', { A1: 100, B1: 200 });
// => { type: 'number', value: 300 }
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
import { evaluate } from '@truecalc/core';

const result = evaluate('IF(A1 > 0, "yes", "no")', { A1: 1 });
// => { type: 'text', value: 'yes' }
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

## API

### `evaluate(formula, variables)`

Evaluates a formula with the given variable bindings.

```js
evaluate('SUM(A1, B1)', { A1: 100, B1: 200 })
// => { type: 'number', value: 300 }

evaluate('CONCAT("Hello, ", name)', { name: 'world' })
// => { type: 'text', value: 'Hello, world' }
```

**Return value shape** (a discriminated union tagged by `type`):

| `type`   | Shape                                                  |
|----------|--------------------------------------------------------|
| `number` | `{ type: 'number', value: 6 }`                         |
| `text`   | `{ type: 'text', value: 'yes' }`                       |
| `bool`   | `{ type: 'bool', value: true }`                        |
| `date`   | `{ type: 'date', value: 46180 }`                       |
| `error`  | `{ type: 'error', error: '#NAME?' }`                   |
| `empty`  | `{ type: 'empty' }`                                    |
| `array`  | `{ type: 'array', value: [ /* EvalResult cells */ ] }` |

`date` carries a spreadsheet **serial number** (`value`); the epoch is implied by
the engine flavor (`google-sheets`: day 0 = 1899-12-30). Format it yourself if you
need a calendar date.

`array` is **recursive**: each element is itself an `EvalResult`, so a 1-D result is
a flat `value` list of scalar cells and a 2-D result is a `value` list of `array`
rows whose elements are scalar cells. Array cells keep their own type (including
nested `date`/`error`/`empty`).

```js
evaluate('SEQUENCE(2,2)')
// => {
//   type: 'array',
//   value: [
//     { type: 'array', value: [ { type: 'number', value: 1 }, { type: 'number', value: 2 } ] },
//     { type: 'array', value: [ { type: 'number', value: 3 }, { type: 'number', value: 4 } ] },
//   ],
// }

evaluate('TODAY()')
// => { type: 'date', value: 46180 }
```

> #### Breaking change in 0.7.0 (surface shape)
>
> 0.7.0 ships the unspilled-array core change (see core PR #566 / issue #569).
> Two observable shapes changed for npm consumers:
>
> - **Array-producing formulas** (`SORT`, `FILTER`, `UNIQUE`, `SEQUENCE`,
>   `TRANSPOSE`, `MMULT`, `HSTACK`/`VSTACK`, `RANDARRAY`, array literals, ...) now
>   return a full `{ type: 'array', value: [...] }` result. In `<= 0.6.x` these
>   returned the top-left anchor-cell **scalar** (and, transiently after #566 but
>   before this fix, an `{ type: 'error', error: 'array not supported' }` object).
>   To recover the old single-cell behavior, read the first cell yourself, e.g.
>   `const tl = r.type === 'array' ? r.value[0] : r;` (recurse once more for 2-D).
> - **Date-producing functions** (`TODAY`, `DATE`, ...) now return
>   `{ type: 'date', value }` instead of `{ type: 'number', value }`. If you were
>   treating the result as a number, also accept `type === 'date'` (the `value`
>   encoding is identical — a serial number).

### `validate(formula)`

Checks whether a formula is syntactically valid without evaluating it.

```js
validate('SUM(A1, B1)')  // => { valid: true }
validate('SUM(A1,')      // => { valid: false, error: '...' }
```

### `list_functions()`

Returns metadata for all built-in functions as an array of `{ name, category, syntax, description }`.

```js
const fns = list_functions();
// [
//   { name: 'SUM',     category: 'math',     syntax: 'SUM(value1, ...)',   description: 'Sum of all arguments' },
//   { name: 'AVERAGE', category: 'math',     syntax: 'AVERAGE(value1, ...)', description: 'Arithmetic mean of all arguments' },
//   { name: 'IF',      category: 'logical',  syntax: 'IF(condition, value_if_true, value_if_false)', description: 'Conditional evaluation' },
//   ...
// ]
```

See the full, live function list at [truecalc.github.io/core](https://truecalc.github.io/core/).

## Documentation

[docs.truecalc.app](https://docs.truecalc.app)
