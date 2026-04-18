# @tryganit/core

[![npm](https://img.shields.io/npm/v/@tryganit/core)](https://www.npmjs.com/package/@tryganit/core)
[![ganit-core](https://img.shields.io/crates/v/ganit-core?label=ganit-core)](https://crates.io/crates/ganit-core)
[![ganit-mcp](https://img.shields.io/crates/v/ganit-mcp?label=ganit-mcp)](https://crates.io/crates/ganit-mcp)
[![docs.rs](https://img.shields.io/docsrs/ganit-core)](https://docs.rs/ganit-core)
[![license](https://img.shields.io/crates/l/ganit-core)](LICENSE)
[![Google Sheets Conformance](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/tryganit/ganit-core/gh-pages/conformance-badge.json&label=Google%20Sheets%20Conformance)](https://github.com/tryganit/ganit-core/actions)

WebAssembly-powered spreadsheet formula engine for JavaScript/TypeScript.

## Install

```sh
npm install @tryganit/core
```

## Usage

### Node.js (CJS)

Works out of the box — no bundler configuration needed.

```js
const { evaluate, validate, list_functions } = require('@tryganit/core');

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
import { evaluate } from '@tryganit/core';

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
