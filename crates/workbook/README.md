# truecalc-workbook

[![crates.io](https://img.shields.io/crates/v/truecalc-workbook)](https://crates.io/crates/truecalc-workbook)
[![docs.rs](https://img.shields.io/docsrs/truecalc-workbook)](https://docs.rs/truecalc-workbook)
[![license](https://img.shields.io/crates/l/truecalc-workbook)](../../LICENSE)

Workbook layer for the [truecalc](https://github.com/truecalc/core) spreadsheet engine: engine-locked workbook, worksheet, and cell value types with a canonical JSON serialization contract.

Status: under active development (workbook v1, schema version `"1"`). APIs may change until the first published release.

## Design

- A `Workbook` is a **value object** — no hidden state, no callbacks. `Clone + PartialEq + Hash + Serialize + Deserialize`.
- The **engine flavor** (`sheets` | `excel`) is required at creation and immutable for the workbook's lifetime.
- The **JSON schema is the cross-surface contract**: canonical serialization (RFC 8785 / JCS) produces byte-identical output across Rust, WASM, MCP, and REST surfaces.

## License

MIT, same as the rest of this workspace.
