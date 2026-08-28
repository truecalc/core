# @truecalc/core

[![npm](https://img.shields.io/npm/v/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![npm downloads](https://img.shields.io/npm/dm/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![truecalc-core](https://img.shields.io/crates/v/truecalc-core?label=truecalc-core)](https://crates.io/crates/truecalc-core)
[![docs.rs](https://img.shields.io/docsrs/truecalc-core)](https://docs.rs/truecalc-core)
[![license](https://img.shields.io/crates/l/truecalc-core)](LICENSE)
[![functions](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/functions-badge.json)](https://truecalc.github.io/core/)
[![Google Sheets Conformance](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/conformance-badge.json)](https://truecalc.github.io/core/)

WebAssembly-powered spreadsheet formula engine for JavaScript/TypeScript.

> **Licensing:** `truecalc-core` / `@truecalc/core` is MIT. `truecalc-workbook` /
> `@truecalc/workbook` is source-available under the Elastic License 2.0 from
> 9.0.0 onward — **every 8.x release and everything before it remains MIT
> permanently.**
> See [LICENSING.md](LICENSING.md).

[DeepWiki](https://deepwiki.com/truecalc/core)

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

**Return value shape:**

| `type`    | Shape                            |
|-----------|----------------------------------|
| `number`  | `{ type: 'number', value: 6 }`   |
| `text`    | `{ type: 'text', value: 'yes' }` |
| `boolean` | `{ type: 'boolean', value: true }`|
| `error`   | `{ type: 'error', error: '#NAME?' }` |
| `empty`   | `{ type: 'empty', value: null }` |

### `validate(formula)`

Checks whether a formula is syntactically valid without evaluating it.

```js
validate('SUM(A1, B1)')  // => { valid: true }
validate('SUM(A1,')      // => { valid: false, error: '...' }
```

### `list_functions()`

Returns metadata for all built-in functions.

```js
const fns = list_functions();
```

## MCP server

The free `truecalc-mcp` server is **retired**. It is no longer built,
published, or maintained: not to crates.io, not to npm as `@truecalc/mcp`,
not via `cargo install`, `cargo binstall`, `npx`, Homebrew, or the MCP
Registry. Versions already published are left in place; there
will be no new ones.

This does not change the engine. `truecalc-core` and `truecalc-workbook`
remain public and published to crates.io, npm, and JSR — one engine, no
second calculation path. On licensing, see below: `truecalc-core` is MIT,
`truecalc-workbook` is Elastic License 2.0.

## Licensing

| Package | License |
|---|---|
| `truecalc-core` · `@truecalc/core` · `truecalc` (PyPI) | MIT |
| `truecalc-workbook` · `@truecalc/workbook` | [Elastic License 2.0](crates/workbook/LICENSE) |

`truecalc-core` — the parser, the evaluator, and the conformance fixtures it is
checked against — is MIT and stays MIT. `truecalc-workbook` — the document
model, dependency graph, and recalculation — is source-available: you may use,
copy, modify and redistribute it, but not offer it to third parties as a hosted
or managed service.

**Nothing already published changes.** Every version published before 9.0.0 was
released under MIT and remains MIT permanently. `9.0.0` is the first version
under the new terms.

Full detail, and why the line falls where it does: **[LICENSING.md](LICENSING.md)**.

## Documentation

[docs.truecalc.app](https://docs.truecalc.app)
