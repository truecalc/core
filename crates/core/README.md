# truecalc-core

[![crates.io](https://img.shields.io/crates/v/truecalc-core)](https://crates.io/crates/truecalc-core)
[![crates.io downloads](https://img.shields.io/crates/d/truecalc-core)](https://crates.io/crates/truecalc-core)
[![docs.rs](https://img.shields.io/docsrs/truecalc-core)](https://docs.rs/truecalc-core)
[![license](https://img.shields.io/crates/l/truecalc-core)](LICENSE)
[![functions](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/functions-badge.json)](https://truecalc.github.io/core/)

Formula engine with exact Google Sheets semantics — stateless, embeddable, bring your own data model.

A large, growing function library (see the live count above) with exact Google Sheets semantics. Ground-truth conformance validated case by case against real Google Sheets — not a spec interpretation. Stateless and embeddable: bring your own data model, no workbook required. Also available as a [WebAssembly npm package](https://www.npmjs.com/package/@truecalc/core) and as an [MCP server](https://crates.io/crates/truecalc-mcp) for AI assistants.

```rust
let result = Engine::sheets().evaluate("=SUM(A1,B1)", &vars);
// => Value::Number(300.0)
```

## Install

```toml
[dependencies]
truecalc-core = "0.9"
```

Or via cargo:

```sh
cargo add truecalc-core
```

## Quick start

```rust
use std::collections::HashMap;
use truecalc_core::{Engine, Value};

let engine = Engine::sheets();

let mut vars = HashMap::new();
vars.insert("A1".to_string(), Value::Number(100.0));
vars.insert("B1".to_string(), Value::Number(200.0));

let result = engine.evaluate("=SUM(A1,B1)", &vars);
assert_eq!(result, Value::Number(300.0));
```

## Usage

### Evaluate a formula

```rust
use std::collections::HashMap;
use truecalc_core::{Engine, Value};

let engine = Engine::sheets();

let mut vars = HashMap::new();
vars.insert("A1".to_string(), Value::Number(100.0));
vars.insert("B1".to_string(), Value::Number(200.0));

let result = engine.evaluate("=SUM(A1, B1)", &vars);
assert_eq!(result, Value::Number(300.0));
```

### Pattern match on the result

```rust
use std::collections::HashMap;
use truecalc_core::{Engine, Value};

let mut vars = HashMap::new();
vars.insert("score".to_string(), Value::Number(85.0));

match Engine::sheets().evaluate("=IF(score >= 60, \"pass\", \"fail\")", &vars) {
    Value::Text(s)   => println!("{s}"),           // "pass"
    Value::Number(n) => println!("{n}"),
    Value::Bool(b)   => println!("{b}"),
    Value::Error(e)  => eprintln!("formula error: {e}"),
    Value::Empty     => println!("(empty)"),
    Value::Array(_)  => println!("(array)"),
}
```

### Validate without evaluating

```rust
use truecalc_core::Engine;

match Engine::sheets().validate("=SUM(A1, B1)") {
    Ok(_)  => println!("valid"),
    Err(e) => eprintln!("parse error at position {}: {}", e.position, e.message),
}
```

### Parse to an AST

```rust
use truecalc_core::Engine;

let expr = Engine::sheets().parse("=1 + 2 * 3").expect("valid formula");
// expr is an Expr tree you can walk yourself
```

### Evaluate with a Resolver (workbook integration)

For workbook-backed evaluation where references should be resolved against a
live cell grid, implement the [`Resolver`](https://docs.rs/truecalc-core/latest/truecalc_core/eval/trait.Resolver.html) trait:

```rust
use truecalc_core::{Engine, ErrorKind, Ref, Resolver, Value};

struct OneSheet;
impl Resolver for OneSheet {
    fn resolve(&mut self, r: &Ref) -> Value {
        match r {
            Ref::Cell { sheet: Some(s), .. } if s == "Data" => Value::Number(10.0),
            Ref::Cell { sheet: Some(_), .. } => Value::Error(ErrorKind::Ref),
            _ => Value::Empty,
        }
    }
}

let engine = Engine::sheets();
assert_eq!(engine.evaluate_with_resolver("=Data!A1", &mut OneSheet), Value::Number(10.0));
```

For a full workbook implementation, see [`truecalc-workbook`](https://docs.rs/truecalc-workbook).

## Engine flavors

The engine flavor controls formula semantics and the date serial system:

| Constructor | Semantics | Date system |
|---|---|---|
| `Engine::sheets()` | Google Sheets | Day 0 = 1899-12-30, no 1900 leap bug |
| `Engine::excel()` | Excel | Serial 1 = 1900-01-01, Lotus leap-year bug |

> **Note:** Excel evaluation is not yet implemented. `Engine::excel().evaluate()` returns
> `#UNSUPPORTED!`. `parse()` and `validate()` work for both flavors.

## Types

### `Value`

| Variant | Description |
|---------|-------------|
| `Number(f64)` | Finite numeric value (never NaN or infinity) |
| `Text(String)` | String value |
| `Bool(bool)` | Boolean value |
| `Error(ErrorKind)` | Formula error (e.g. `#DIV/0!`) |
| `Empty` | Missing/blank cell reference |
| `Array(Vec<Value>)` | Array of values |

### `ErrorKind`

| Variant | Excel error |
|---------|-------------|
| `DivByZero` | `#DIV/0!` |
| `Value` | `#VALUE!` |
| `Ref` | `#REF!` |
| `Name` | `#NAME?` |
| `Num` | `#NUM!` |
| `NA` | `#N/A` |
| `Null` | `#NULL!` |

## Available functions

Covers math, logical, text, financial, and statistical categories. For the full list with signatures and descriptions, query the live registry:

```rust
use truecalc_core::Registry;

let registry = Registry::new();
for (name, meta) in registry.list_functions() {
    println!("{} ({}): {}", name, meta.category, meta.description);
}
```

## Migration from 0.6.x

See [`MIGRATION.md`](MIGRATION.md) for the full guide.

**Summary:** The free `evaluate()` function and `Engine::google_sheets()` constructor
were deprecated in 0.7.0. Replace them with `Engine::sheets().evaluate()`.

## Related crates

- [`truecalc-workbook`](https://crates.io/crates/truecalc-workbook): full workbook layer with
  engine-locked workbook, worksheet, cell mutation, and recalc.

## Documentation

[docs.truecalc.app](https://docs.truecalc.app)

## License

MIT
