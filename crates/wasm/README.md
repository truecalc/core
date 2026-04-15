# @tryganit/core

[![npm](https://img.shields.io/npm/v/@tryganit/core)](https://www.npmjs.com/package/@tryganit/core)
[![crates.io](https://img.shields.io/crates/v/ganit-core)](https://crates.io/crates/ganit-core)
[![docs.rs](https://img.shields.io/docsrs/ganit-core)](https://docs.rs/ganit-core)
[![license](https://img.shields.io/crates/l/ganit-core)](LICENSE)

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

| `type`    | Shape                                |
|-----------|--------------------------------------|
| `number`  | `{ type: 'number', value: 6 }`       |
| `text`    | `{ type: 'text', value: 'yes' }`     |
| `boolean` | `{ type: 'boolean', value: true }`   |
| `error`   | `{ type: 'error', error: '#NAME?' }` |
| `empty`   | `{ type: 'empty', value: null }`     |

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

**Available functions by category:**

| Category   | Functions |
|------------|-----------|
| math       | SUM, AVERAGE, PRODUCT, ROUND, ROUNDUP, ROUNDDOWN, INT, ABS, SIGN, MOD, POWER, SQRT, LOG, LOG10, LN, EXP, CEILING, FLOOR, RAND, RANDBETWEEN, PI, SIN, COS, TAN, QUOTIENT |
| logical    | IF, AND, OR, NOT, IFERROR, IFNA, IFS, SWITCH, ISNUMBER, ISTEXT, ISERROR, ISBLANK, ISNA |
| text       | LEFT, MID, RIGHT, LEN, LOWER, UPPER, TRIM, CONCATENATE, FIND, SUBSTITUTE, REPLACE, TEXT, VALUE, REPT |
| financial  | PMT, NPV, IRR, PV, FV, RATE, NPER |
| statistical | COUNT, COUNTA, MAX, MIN, MEDIAN |
