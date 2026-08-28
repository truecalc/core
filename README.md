# @truecalc/core

[![npm](https://img.shields.io/npm/v/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![npm downloads](https://img.shields.io/npm/dm/@truecalc/core)](https://www.npmjs.com/package/@truecalc/core)
[![truecalc-core](https://img.shields.io/crates/v/truecalc-core?label=truecalc-core)](https://crates.io/crates/truecalc-core)
[![docs.rs](https://img.shields.io/docsrs/truecalc-core)](https://docs.rs/truecalc-core)
[![license](https://img.shields.io/crates/l/truecalc-core)](LICENSE)
[![functions](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/functions-badge.json)](https://truecalc.github.io/core/)
[![Google Sheets Conformance](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/conformance-badge.json)](https://truecalc.github.io/core/)

WebAssembly-powered spreadsheet formula engine for JavaScript/TypeScript.

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

The workspace is not under a single license.

| Package | Where | License |
|---|---|---|
| `truecalc-core` · `@truecalc/core` | `crates/core`, `crates/wasm` | MIT |
| `truecalc` (PyPI) | `crates/python` | MIT |
| `truecalc-workbook` · `@truecalc/workbook` | `crates/workbook`, `crates/wasm-workbook` | [Elastic License 2.0](crates/workbook/LICENSE) |

`truecalc-core` — the parser and evaluator, and the conformance fixtures it is
checked against — is MIT and stays MIT. It is the verifiability claim: anyone
can read it, run it, and confirm the maths against real Google Sheets.

`truecalc-workbook` — the document model, dependency graph, and recalculation —
is source-available under ELv2. You may use, copy, modify and redistribute it.
You may not offer it to third parties as a hosted or managed service that gives
them access to a substantial set of its functionality.

**Nothing already published changes.** Every version up to and including 8.2.2,
of every package above, was released under MIT and remains MIT permanently. If
you installed `truecalc-workbook 8.2.2` or `@truecalc/workbook@8.2.2`, you have
an MIT copy and keep it. The new terms apply only to versions published after
8.2.2.

**Why ELv2 and not BSL 1.1** — maintenance overhead, not legal strength. BSL
requires a Change Date tracked per release plus an Additional Use Grant that has
to be drafted and then defended in interpretation. ELv2 is one unmodified
document with no per-release bookkeeping, and the text here is the official one,
unedited.

## Documentation

[docs.truecalc.app](https://docs.truecalc.app)
