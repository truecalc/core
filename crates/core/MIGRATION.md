# Migration guide: 0.6.x → 0.7.0

This guide covers the **engine-flavor-explicit** breaking change introduced in
0.7.0 (ADR 2026-04-27-engine-flavor-explicit-everywhere).

## Summary

The free `evaluate()` function in `truecalc-core` and the `Engine::google_sheets()`
constructor are deprecated in 0.7.0. Both will be removed in a coordinated
release. Update call sites now to eliminate deprecation warnings.

---

## `truecalc-core`: free `evaluate()` function

### Before (0.6.x)

```rust
use std::collections::HashMap;
use truecalc_core::{evaluate, Value};

let mut vars = HashMap::new();
vars.insert("A1".to_string(), Value::Number(10.0));

let result = evaluate("=A1 * 2", &vars);
```

### After (0.7+)

```rust
use std::collections::HashMap;
use truecalc_core::{Engine, Value};

let mut vars = HashMap::new();
vars.insert("A1".to_string(), Value::Number(10.0));

let result = Engine::sheets().evaluate("=A1 * 2", &vars);
```

The engine instance is cheap to create and stateless (no heap allocation beyond
the function registry, which is computed once). Creating one per call site is fine.

---

## `truecalc-core`: `Engine::google_sheets()` constructor

### Before (0.6.x / early 0.7.x)

```rust
use truecalc_core::Engine;

let engine = Engine::google_sheets();
```

### After (0.7+)

```rust
use truecalc_core::Engine;

let engine = Engine::sheets();
```

`Engine::google_sheets()` is a deprecated alias for `Engine::sheets()`. Both
constructors produce an identical `EngineFlavor::Sheets` engine; only the name
changed to make the API consistent with the `excel()` constructor.

---

## Why the change?

Before 0.7, the engine flavor was implicit in the free function and had no
effect on callers — all evaluation silently targeted Google Sheets semantics.
As Excel support and the workbook layer were added, the flavor became load-
bearing: the date serial system, cross-surface JSON contracts, and the workbook's
recalc layer all need to know which engine a workbook is locked to.

Making the flavor explicit at the call site:

- Prevents accidental cross-flavor workbooks.
- Makes the engine's date system unambiguous.
- Aligns the Rust, WASM, MCP, and REST surfaces — they all require an explicit
  `"engine": "sheets"` / `"engine": "excel"` field.

---

## `Engine::excel()` — parse/validate only

`Engine::excel()` is available for callers targeting Excel conformance, but
**Excel evaluation is not yet implemented**. Calling `Engine::excel().evaluate()`
returns `Value::Error(ErrorKind::Unsupported)` for every formula.
`Engine::excel().parse()` and `Engine::excel().validate()` work.

Excel evaluation lands in a later phase; this guide will be updated when it does.

---

## Checking for deprecated usage

```sh
# Compile with deprecation warnings promoted to errors to find every call site:
RUSTFLAGS="-D deprecated" cargo check -p truecalc-core
```
