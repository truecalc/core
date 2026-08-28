# Contributing to truecalc/core

Thanks for your interest in contributing! This repo is a Rust workspace
implementing a Google Sheets-compatible spreadsheet formula engine.

## Building and testing locally

Requires a stable Rust toolchain (see `rust-toolchain.toml`; `rustup` will
pick it up automatically) plus `cargo-nextest` for the full test run:

```sh
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p truecalc-core
# or, matching CI exactly:
cargo nextest run --workspace
```

The workspace has several crates:

- `crates/core` (`truecalc-core`) — the engine itself.
- `crates/wasm` (`@truecalc/core` on npm) — the WebAssembly bindings.
- `crates/workbook` / `crates/wasm-workbook` — the workbook runtime.
- `xtask` — code-generation and maintenance tasks.

## Conformance fixtures are immutable ground truth

`crates/core/tests/fixtures/google_sheets/*.tsv` hold the expected value for
every conformance test case, produced by actually evaluating the formula in
Google Sheets. Treat them like a database of ground truth, not like ordinary
test fixtures:

- **Never edit `expected_value` or `expected_type`** on an existing row —
  those values came from Google Sheets, not from this engine.
- **Never add new rows to a category TSV** (`math.tsv`, `statistical.tsv`,
  etc.) with a value you computed yourself. New rows must go through the
  conformance pipeline to be verified against real Google Sheets before they
  land in a category file.
- `bugs.tsv` is the one exception: you may add a row there to acknowledge a
  known failure without pipeline verification. Only remove a row from
  `bugs.tsv` once the underlying bug is fixed and the case has been moved to
  the appropriate category TSV.
- If a rebase or merge conflicts on a fixture file, keep `main`'s version and
  manually re-apply only your intended additions — never let the conflict
  resolution silently restore deleted rows.

If you're adding a new function or fixing a bug and need a new conformance
case, open an issue describing the formula and expected behavior rather than
hand-writing the expected value.

## Branching and pull requests

- Every change ships as a PR against `main` — never commit directly to
  `main`.
- Branch per issue, e.g. `feat/<issue-number>-<short-description>` or
  `fix/<issue-number>-<short-description>`.
- Reference the issue you're closing in the PR description (`closes #123`).
- CI runs `cargo clippy --workspace -- -D warnings` and the full test suite;
  a PR isn't mergeable until CI is green.

## Code style

- Formatting and lints are enforced by `cargo fmt` and
  `cargo clippy --workspace -- -D warnings` in CI.
- Tests live alongside the code they cover as separate files (e.g.
  `#[cfg(test)] mod tests;` pointing at a sibling `tests.rs`), not inline in
  production modules.

## Reporting bugs and requesting features

Please use the issue templates under `.github/ISSUE_TEMPLATE/` — a bug
report needs the formula, the input values, the expected result (ideally
with a Google Sheets screenshot), and the actual result from truecalc.

## Where issues are filed

The two engine crates are heading for different licenses. `truecalc-core`
stays MIT and fully open. `truecalc-workbook` moves to a source-available
license. Issue location follows that line, so it is a rule rather than a
case-by-case judgement:

- Issues about `truecalc-core` — functions, evaluation, conformance with the
  reference spreadsheet — are filed here, in public. If you depend on an MIT
  crate, you should be able to see what is broken in it.
- Issues about `truecalc-workbook` — recalculation, the dependency graph,
  performance work — are filed on a private tracker.
- Security reports never move to the private tracker. They stay public, or
  follow the disclosure process in [SECURITY.md](SECURITY.md).
- Anything that affects users of the MIT crate stays public.

It is worth being precise about what this does and does not do. The source and
every pull request remain public — the repository is public, and a
source-available license requires visibility — so no implementation is hidden.
What moves to the private tracker is the analysis: profiles, measurements, and
the reasoning that located a defect. Finding a problem is the expensive part;
fixing it once you know where it is usually is not.

Issues already filed here remain public. None are being retracted.

## Reporting security issues

Please don't open a public issue for a security vulnerability — see
[SECURITY.md](SECURITY.md) instead.
