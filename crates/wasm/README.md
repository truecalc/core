# @truecalc/core

[![npm](https://img.shields.io/npm/v/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![npm downloads](https://img.shields.io/npm/dm/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![crates.io](https://img.shields.io/crates/v/truecalc-core)](https://crates.io/crates/truecalc-core)
[![docs.rs](https://img.shields.io/docsrs/truecalc-core)](https://docs.rs/truecalc-core)
[![license](https://img.shields.io/crates/l/truecalc-core)](LICENSE)
[![functions](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/functions-badge.json)](https://truecalc.github.io/core/)

WebAssembly-powered spreadsheet formula engine for JavaScript/TypeScript.

A comprehensive library of spreadsheet functions (see the live count above). Runs in Node.js, Bun, Deno, and the browser — no server needed. Ground-truth conformance against real Google Sheets. The same engine is also available as a [Rust crate](https://crates.io/crates/truecalc-core).

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

### Bun

Bun resolves to a separate build that requires an explicit `init()` first:

```js
import init, { evaluate } from '@truecalc/core';

await init();
evaluate('SUM(A1, B1)', { A1: 100, B1: 200 });
// => { type: 'number', value: 300 }
```

That extra call is not optional and not needed on any other runtime. The main
build relies on WebAssembly ESM integration, which Node and Deno support and
Bun does not — under Bun it fails with `malloc is not a function`. So
`package.json` routes Bun to a `--target web` build, which works but must be
initialised explicitly.

**TypeScript:** add `"customConditions": ["bun"]` to your `tsconfig.json`
`compilerOptions`. TypeScript does not match the `bun` export condition on its
own — not even under the tsconfig `bun init` generates — so without it the
snippet above reports *"Module has no default export"* and `init` is missing
from autocomplete. Runtime is unaffected either way.

**`bun build` bundling:** the wasm is not emitted as a sibling asset, so
`await init()` cannot find it and fails with `ERR_BODY_ALREADY_USED`. Pass the
bytes explicitly instead:

```js
import init, { evaluate } from '@truecalc/core';
import wasmPath from '@truecalc/core/truecalc_wasm_bg.wasm' with { type: 'file' };

// Resolve against import.meta.url — the imported path is relative to the
// process's working directory, so a bare `Bun.file(wasmPath)` only works when
// you happen to run from the output directory.
await init(await Bun.file(new URL(wasmPath, import.meta.url)).arrayBuffer());
```

Only `bun build --target=bun` is affected; `--target=node` and
`--target=browser` resolve the default build and bundle normally.

Nothing changes for Node, Deno or bundlers, which continue to resolve the
init-free build. `@truecalc/workbook` is unaffected: it already ships in the
form Bun can consume, and its `init()` is part of its documented API.

### Writing a library on top of this

Because only the Bun build has a default export, a library that must work on
every runtime cannot call `init()` unconditionally:

```js
const mod = await import('@truecalc/core');
if (typeof mod.default === 'function') await mod.default();  // Bun only
mod.evaluate('SUM(A1, B1)', { A1: 100, B1: 200 });
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

> #### Result-type change: `MAX` / `MIN` / `MAXA` / `MINA` over dates
>
> Dates now take part in these four aggregates, and the result is **date-typed**
> whenever a date took part — including when a plain number won the comparison.
> `evaluate('MAX(A1:A10)')` over a column of dates returns
> `{ type: 'date', value }` where it previously returned `{ type: 'number', value }`
> (`MAX` over a date-only array literal previously returned an `#REF!` error and
> `MIN` a silent `0`). The `value` encoding is unchanged — still a serial number;
> only `type` moved. This matches Google Sheets, which formats the result cell as
> a date. If you branch on `type`, accept `'date'` anywhere you accepted
> `'number'` from these functions. See issue #776.

### `validate(formula)`

Checks whether a formula is syntactically valid without evaluating it.

```js
validate('SUM(A1, B1)')  // => { valid: true }
validate('SUM(A1,')      // => { valid: false, error: '...' }
```

### `list_functions()`

Returns metadata for every built-in function as an array of
`{ name, category, syntax, description }`, sorted by name.

```js
const fns = list_functions();
fns.length;
// 518

// [
//   { name: 'ABS',  category: 'math',      syntax: 'ABS(number)',                       description: 'Absolute value of a number' },
//   { name: 'LEFT', category: 'text',      syntax: 'LEFT(text, [num_chars])',           description: 'Left portion of a string' },
//   { name: 'PMT',  category: 'financial', syntax: 'PMT(rate, nper, pv, [fv], [type])', description: 'Periodic payment for a loan' },
//   ...
// ]
```

Optional arguments appear in `[brackets]`. Categories are drawn from the engine
registry — there are 17, including `array`, `database`, `filter`, `lookup`,
`parser`, `query` and `timezone` alongside the familiar `math`/`text`/`logical`.

See the full, live function list at [truecalc.github.io/core](https://truecalc.github.io/core/).

## Documentation

[docs.truecalc.app](https://docs.truecalc.app)
