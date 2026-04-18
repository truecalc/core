# truecalc-mcp

[![truecalc-core](https://img.shields.io/crates/v/truecalc-core?label=truecalc-core)](https://crates.io/crates/truecalc-core)
[![truecalc-mcp](https://img.shields.io/crates/v/truecalc-mcp?label=truecalc-mcp)](https://crates.io/crates/truecalc-mcp)
[![license](https://img.shields.io/crates/l/truecalc-mcp)](LICENSE)

MCP server that exposes [truecalc](https://crates.io/crates/truecalc-core) spreadsheet formula evaluation as tools for AI assistants.

Plug it into Claude Desktop (or any MCP-compatible client) and your AI can evaluate, validate, and explain Excel-compatible formulas without writing any code.

## Install

```sh
cargo install truecalc-mcp --force
```

## Claude Desktop setup

Add the server to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "truecalc": {
      "command": "/Users/your-username/.cargo/bin/truecalc-mcp"
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

Covers math, logical, text, financial, and statistical categories. For the full list with signatures and descriptions, call the `list_functions` tool — it returns the live registry.

## Related

- [`truecalc-core`](https://crates.io/crates/truecalc-core) — the underlying formula engine (Rust library)
- [`@truecalc/core`](https://www.npmjs.com/package/@truecalc/core) — WebAssembly package for JavaScript/TypeScript

## License

MIT
