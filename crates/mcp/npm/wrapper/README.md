# @truecalc/mcp

Run the [truecalc-mcp](https://github.com/truecalc/core/tree/main/crates/mcp) server — spreadsheet formula evaluation for AI assistants — with no install step:

```sh
npx -y @truecalc/mcp
```

This package is a thin wrapper: `npx` resolves it, its `optionalDependencies`
bring in the prebuilt `truecalc-mcp` binary for your OS/CPU
(`@truecalc/mcp-darwin-arm64`, `@truecalc/mcp-darwin-x64`,
`@truecalc/mcp-linux-arm64`, `@truecalc/mcp-linux-x64`,
`@truecalc/mcp-win32-x64`), and the `truecalc-mcp` bin script execs it with
stdio inherited. The server itself is the same Rust binary published to
[crates.io](https://crates.io/crates/truecalc-mcp) — one implementation, no
second copy of tool behavior to keep in sync.

## MCP client configuration

Add this to your client's MCP server config (e.g. Claude Desktop's
`claude_desktop_config.json`):

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

Restart the client. The tools (`evaluate`, `validate`, `explain`,
`batch_evaluate`, `list_functions`) appear automatically.

## Supported platforms

macOS (arm64, x64), Linux (arm64, x64), Windows (x64). On an unsupported
platform, running `npx -y @truecalc/mcp` prints an actionable error naming
the platforms that are supported instead of failing silently.

## Learn more

- [truecalc-mcp](https://github.com/truecalc/core/tree/main/crates/mcp) — full tool documentation, `cargo install` / Homebrew alternatives
- [truecalc-core](https://crates.io/crates/truecalc-core) — the underlying formula engine (Rust)
- [@truecalc/core](https://www.npmjs.com/package/@truecalc/core) — the same engine compiled to WebAssembly
