# Licensing

This workspace is not under a single license. Most of it is MIT. One layer —
the workbook — is source-available under the Elastic License 2.0.

| Package | Published as | Source | License |
|---|---|---|---|
| `truecalc-core` | [crates.io](https://crates.io/crates/truecalc-core) | `crates/core` | MIT |
| `truecalc-wasm` | npm / JSR: `@truecalc/core` | `crates/wasm` | MIT |
| `truecalc-python` | PyPI: `truecalc` | `crates/python` | MIT |
| `truecalc-workbook` | [crates.io](https://crates.io/crates/truecalc-workbook) | `crates/workbook` | [Elastic License 2.0](crates/workbook/LICENSE) |
| `truecalc-wasm-workbook` | npm / JSR: `@truecalc/workbook` | `crates/wasm-workbook` | [Elastic License 2.0](crates/wasm-workbook/LICENSE) |

The root [`LICENSE`](LICENSE) is the MIT text and covers the MIT packages above.
The ELv2 text lives with the packages it covers.

## Nothing already published changes

**Every version published before 9.0.0 — every 8.x release and everything
before it, of every package in the table, `truecalc-workbook` and
`@truecalc/workbook` included — was released under MIT and remains MIT
permanently.**

If you have any 8.x of `truecalc-workbook` or `@truecalc/workbook`, you have an
MIT copy and you keep it. Nothing is being relicensed retroactively, withdrawn,
or yanked. Those versions stay on crates.io, npm, JSR, and PyPI under the terms
they were published under.

`9.0.0` is the first version under the new terms.

## What ELv2 permits, and what it does not

You may use, copy, modify, distribute, and make derivative works of
`truecalc-workbook`.

You may not provide it to third parties as a hosted or managed service where
that service gives users access to a substantial set of its features or
functionality. You may not remove or obscure the license notices.

For the overwhelming majority of uses — embedding a spreadsheet engine in an
application, a data pipeline, a desktop tool, an internal service, a commercial
product that is not itself a hosted spreadsheet-engine service — ELv2 and MIT
are indistinguishable in practice. The single thing it prevents is a competing
hosted service built out of this code.

[Read the full text](crates/workbook/LICENSE). It is the official document,
unmodified.

## Why the line falls here

The two engine crates were deliberately kept separate so that their licenses
could diverge. That option is now taken.

**`truecalc-core` is MIT because it is the verifiability claim.** The whole
premise of this project is that a formula engine should be checkable rather than
trusted: you can read the parser, run the evaluator, and confirm the results
against real Google Sheets using the conformance fixtures in this repository.
That claim is worth nothing if the code behind it is not open. A competitor who
copies `truecalc-core` gets a snapshot of correct maths — not the fixtures
pipeline, the conformance process, or the work that keeps it correct as the
reference behaviour shifts underneath it. That is a fair trade.

**`truecalc-workbook` is ELv2 because it is the product.** The document model,
the dependency graph, and recalculation are the parts that turn an evaluator
into a spreadsheet, and they are the parts a competing hosted service would
need. Blocking that one use is what makes it viable to keep the rest open.

## Why ELv2 and not BSL 1.1

Maintenance overhead, not legal strength.

BSL 1.1 requires a Change Date tracked per release, after which each release
converts to an open-source license, plus an Additional Use Grant that has to be
drafted, kept consistent across releases, and defended in interpretation. That
is real, recurring work, and getting it wrong is worse than not doing it.

ELv2 is a single unmodified document. No per-release bookkeeping, no grant to
draft, and it is widely deployed and widely understood — which matters more for
a license than elegance does. The text in this repository is the official one,
byte-for-byte.

## Contributing

The repository stays public and every pull request stays public — a
source-available license requires visibility, so no implementation is hidden.
See [CONTRIBUTING.md](CONTRIBUTING.md) for where issues are filed.

## Third-party notices

`@truecalc/workbook` compiles ELv2 `truecalc-workbook` and MIT `truecalc-core`
into one artifact. The published package ships a `NOTICE` file reproducing
core's MIT copyright and permission notice, as MIT requires.
