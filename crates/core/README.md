# ganit-core

[![crates.io](https://img.shields.io/crates/v/ganit-core)](https://crates.io/crates/ganit-core)
[![docs.rs](https://img.shields.io/docsrs/ganit-core)](https://docs.rs/ganit-core)
[![license](https://img.shields.io/crates/l/ganit-core)](LICENSE)

Spreadsheet formula engine — parser and evaluator for Excel-compatible formulas.

Also available as a WebAssembly npm package: [`@tryganit/core`](https://www.npmjs.com/package/@tryganit/core)

## Install

```toml
[dependencies]
ganit-core = "0.1"
```

Or via cargo:

```sh
cargo add ganit-core
```

## Usage

### Evaluate a formula

```rust
use std::collections::HashMap;
use ganit_core::{evaluate, Value};

let mut vars = HashMap::new();
vars.insert("A1".to_string(), Value::Number(100.0));
vars.insert("B1".to_string(), Value::Number(200.0));

let result = evaluate("SUM(A1, B1)", &vars);
assert_eq!(result, Value::Number(300.0));
```

### Pattern match on the result

```rust
use std::collections::HashMap;
use ganit_core::{evaluate, Value};

let mut vars = HashMap::new();
vars.insert("score".to_string(), Value::Number(85.0));

match evaluate("IF(score >= 60, \"pass\", \"fail\")", &vars) {
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
use ganit_core::validate;

match validate("SUM(A1, B1)") {
    Ok(_)  => println!("valid"),
    Err(e) => eprintln!("parse error at position {}: {}", e.position, e.message),
}
```

### Parse to an AST

```rust
use ganit_core::parse;

let expr = parse("1 + 2 * 3").expect("valid formula");
// expr is an Expr tree you can walk yourself
```

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
use ganit_core::Registry;

let registry = Registry::new();
for (name, meta) in registry.list_functions() {
    println!("{} ({}): {}", name, meta.category, meta.description);
}
```

## License

MIT
