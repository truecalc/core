# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 5. Worktree-Based Development

**Always develop on branches. Never commit directly to `main`.**

- Each issue gets its own branch: `feat/<issue-number>-<short-description>`
- Branches ship as PRs against `main`; they are never merged locally
- When dispatching parallel agents, always pass `isolation: "worktree"` so each agent commits to its own branch — never to `main`
- Tests must pass on the branch before the PR is opened

## 6. PR Lifecycle — Monitor and Fix CI

**A PR is not done until CI is green.**

After opening or pushing to a PR:
1. **Check CI immediately:** `gh run list --repo truecalc/core --branch <branch> --limit 3`
2. **On failure, read the exact error:** `gh run view <run-id> --log-failed --repo truecalc/core`
3. **Fix root cause** (not symptoms). Common culprits in this repo: `clippy -D warnings`, formatting, test failures from oracle fixture mismatches.
4. **Verify locally before pushing:** `cargo clippy --workspace -- -D warnings && cargo test -p truecalc-core`
5. **Push the fix** and confirm CI re-runs green.

**PR description must include:**
- One `closes #<issue>` line per issue being resolved (not a comma list — one per line)

**PR rules:**
- Never enable auto-merge on a PR
- Always assign the PR to `@hhimanshu` (the CODEOWNER): `gh pr edit <number> --add-assignee hhimanshu`

Do not report a task complete until CI passes.

## 7. Browser Automation

**Use `agent-browser --auto-connect` for all browser tasks.**

- Never use MCP chrome-devtools tools directly — they don't work in this environment.
- Launch browser automation via: `agent-browser --auto-connect`

## 9. Test Files Are Separate From Production Code

**Never put `#[cfg(test)]` blocks inline in production source files.**

- For library modules (e.g. `engine/mod.rs`): use `#[cfg(test)] mod tests;` and put tests in `engine/tests.rs`
- For binary crates (e.g. `crates/mcp/src/main.rs`): put tests in `crates/mcp/tests/` as integration tests
- WASM tests that panic outside WASM context (`JsValue`): omit them entirely

## 8. Conformance Fixture Files Are Immutable Oracle Records

**Never modify fixture TSVs except by adding new rows or removing rows that have been fixed.**

The files under `crates/core/tests/fixtures/google_sheets/` are the source of truth for Google Sheets conformance. They were produced by the oracle pipeline (GAS web app → evaluated expected values). Treat them like a database of ground truth:

- **Do not change `expected_value` or `expected_type`** in any existing row — those values came from the oracle.
- **Do not add rows to category TSVs** (math.tsv, statistical.tsv, etc.) with self-confirmed values (i.e., values truecalc computed itself). New rows must be oracle-verified.
- **Do not remove rows from bugs.tsv** unless the underlying bug is confirmed fixed (the formula now passes in truecalc AND the entry is moved to the appropriate category TSV).
- **Conflict resolution on fixture files**: always take the `--ours` side (current main) and manually re-apply only your intended additions. Never let a rebase silently restore deleted rows.
- **bugs.tsv** is the only file where adding rows without oracle verification is acceptable — it acknowledges known failures, not correct values.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
