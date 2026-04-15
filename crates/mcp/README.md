# ganit-mcp

[![crates.io](https://img.shields.io/crates/v/ganit-mcp)](https://crates.io/crates/ganit-mcp)
[![license](https://img.shields.io/crates/l/ganit-mcp)](LICENSE)

MCP server that exposes [ganit](https://crates.io/crates/ganit-core) spreadsheet formula evaluation as tools for AI assistants.

Plug it into Claude Desktop (or any MCP-compatible client) and your AI can evaluate, validate, and explain Excel-compatible formulas without writing any code.

## Install

```sh
cargo install ganit-mcp
```

## Claude Desktop setup

Add the server to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ganit": {
      "command": "/Users/your-username/.cargo/bin/ganit-mcp"
    }
  }
}
```

Restart Claude Desktop. The tools will appear automatically.

## Tools

### `evaluate`

Evaluate a formula with optional variable bindings.

```json
{ "formula": "SUM(A1, B1)", "variables": { "A1": 100, "B1": 200 } }
```

Returns: `{ "value": 300, "type": "number" }`

### `validate`

Check whether a formula parses without errors.

```json
{ "formula": "IF(score >= 60, \"pass\", \"fail\")" }
```

Returns: `{ "valid": true }` or `{ "valid": false, "error": "..." }`

### `explain`

Describe a formula and list the functions it uses.

```json
{ "formula": "IF(AND(A1 > 0, B1 > 0), SUM(A1, B1), 0)" }
```

Returns: `{ "description": "Formula using: AND, IF, SUM", "functions_used": ["AND", "IF", "SUM"] }`

### `batch_evaluate`

Evaluate multiple formulas sharing the same variable bindings.

```json
{
  "formulas": ["SUM(A1, B1)", "AVERAGE(A1, B1)", "MAX(A1, B1)"],
  "variables": { "A1": 10, "B1": 90 }
}
```

Returns an array of results in the same order.

### `list_functions`

Return the full catalogue of supported spreadsheet functions with category, syntax, and description.

## Supported functions

| Category    | Functions |
|-------------|-----------|
| math        | SUM, AVERAGE, PRODUCT, ROUND, ROUNDUP, ROUNDDOWN, INT, ABS, SIGN, MOD, POWER, SQRT, LOG, LOG10, LN, EXP, CEILING, FLOOR, RAND, RANDBETWEEN, PI, SIN, COS, TAN, QUOTIENT |
| logical     | IF, AND, OR, NOT, IFERROR, IFNA, IFS, SWITCH, ISNUMBER, ISTEXT, ISERROR, ISBLANK, ISNA |
| text        | LEFT, MID, RIGHT, LEN, LOWER, UPPER, TRIM, CONCATENATE, FIND, SUBSTITUTE, REPLACE, TEXT, VALUE, REPT |
| financial   | PMT, NPV, IRR, PV, FV, RATE, NPER |
| statistical | COUNT, COUNTA, MAX, MIN, MEDIAN |

## Related

- [`ganit-core`](https://crates.io/crates/ganit-core) — the underlying formula engine (Rust library)
- [`@tryganit/core`](https://www.npmjs.com/package/@tryganit/core) — WebAssembly package for JavaScript/TypeScript

## License

MIT
