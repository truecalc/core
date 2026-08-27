# truecalc-mcp

Registered with the [MCP Registry](https://registry.modelcontextprotocol.io) via `mcp-name: io.github.truecalc/truecalc-mcp`.

[![truecalc-core](https://img.shields.io/crates/v/truecalc-core?label=truecalc-core)](https://crates.io/crates/truecalc-core)
[![truecalc-mcp](https://img.shields.io/crates/v/truecalc-mcp?label=truecalc-mcp)](https://crates.io/crates/truecalc-mcp)
[![crates.io downloads](https://img.shields.io/crates/d/truecalc-mcp)](https://crates.io/crates/truecalc-mcp)
[![npm](https://img.shields.io/npm/v/%40truecalc%2Fmcp?label=%40truecalc%2Fmcp)](https://www.npmjs.com/package/@truecalc/mcp)
[![npm downloads](https://img.shields.io/npm/dm/%40truecalc%2Fmcp)](https://www.npmjs.com/package/@truecalc/mcp)
[![license](https://img.shields.io/crates/l/truecalc-mcp)](LICENSE)
[![functions](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/truecalc/core/gh-pages/functions-badge.json)](https://truecalc.github.io/core/)

MCP server that exposes [truecalc](https://crates.io/crates/truecalc-core) spreadsheet formula evaluation as tools for AI assistants.

A comprehensive spreadsheet function library (see the live count above) — evaluate, validate, and explain formulas without writing any code. Ground-truth conformance against real Google Sheets. Backed by the same engine used in the [Rust crate](https://crates.io/crates/truecalc-core) and [npm package](https://www.npmjs.com/package/@truecalc/core).

```json
// Tool: evaluate
{ "formula": "SUM(A1, B1)", "variables": { "A1": 100, "B1": 200 } }
// => { "value": 300, "type": "number" }
```

## Install

### npx (no install step)

```sh
npx -y @truecalc/mcp
```

Resolves the prebuilt `truecalc-mcp` binary for your OS/CPU via
[`@truecalc/mcp`](https://www.npmjs.com/package/@truecalc/mcp) — same Rust
binary as the crate below, no Rust toolchain required.

### cargo

```sh
cargo install truecalc-mcp --force
```

## Claude Desktop setup

Add the server to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "truecalc": {
      "command": "npx",
      "args": ["-y", "@truecalc/mcp"]
    }
  }
}
```

Using a `cargo install`ed binary instead:

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

Returns: `{ "value": 300, "type": "number", "accepted": { "bound": { "A1": "number", "B1": "number" }, "conformance": "google-sheets" } }`

`accepted` reports what the server resolved the request to — which names bound
at which shape, and the conformance target actually used — so a list read as
text or a defaulted target is visible without a second call. `workbook_set` and
`workbook_get` echo the resolved sheet and cell the same way, and `workbook_set`
also reports the kind it read the value as (`"as": "number"` for `"007"`).

### `validate`

Check whether a formula parses without errors.

```json
{ "formula": "IF(score >= 60, \"pass\", \"fail\")" }
```

Returns: `{ "valid": true }` or `{ "valid": false, "error": "..." }`. Both are
successful calls — `isError` is set only when the server could not carry the
call out at all, never because the payload mentions an error.

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

Return supported spreadsheet functions with category, syntax, and description.
All filters are optional and combine with AND; omitting them returns the full
catalogue as before (~70 KB).

```json
{ "category": "lookup", "name_contains": "lookup", "names": ["SUM", "XLOOKUP"], "limit": 25 }
```

- `category` — exact category name; an unknown one is an error listing the known categories
- `name_contains` — case-insensitive substring of the function name
- `names` — exact names to look up in one call; any that do not exist come back in `not_found`
- `limit` — maximum entries (default: 100 for a filtered call, uncapped otherwise)

Every response carries `total_matched` and `returned`, so a capped page is
visible without a second call.

## Supported functions

Covers math, logical, text, financial, and statistical categories. For the full list with signatures and descriptions, call the `list_functions` tool — it returns the live registry.

## Related

- [`truecalc-core`](https://crates.io/crates/truecalc-core) — the underlying formula engine (Rust library)
- [`@truecalc/core`](https://www.npmjs.com/package/@truecalc/core) — WebAssembly package for JavaScript/TypeScript
- [`@truecalc/mcp`](https://www.npmjs.com/package/@truecalc/mcp) — this server, published to npm for `npx`

## Documentation

[docs.truecalc.app](https://docs.truecalc.app)

## License

MIT
