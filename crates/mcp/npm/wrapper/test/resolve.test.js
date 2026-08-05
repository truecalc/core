"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const { resolveTarget } = require("../lib/resolve.js");

// Every combo npm's os/cpu package.json fields can select across the five
// published platform packages. Values are Node's process.platform/
// process.arch strings, not Rust target-triple naming.
const SUPPORTED_COMBOS = [
  ["darwin", "arm64", "@truecalc/mcp-darwin-arm64", "truecalc-mcp"],
  ["darwin", "x64", "@truecalc/mcp-darwin-x64", "truecalc-mcp"],
  ["linux", "arm64", "@truecalc/mcp-linux-arm64", "truecalc-mcp"],
  ["linux", "x64", "@truecalc/mcp-linux-x64", "truecalc-mcp"],
  ["win32", "x64", "@truecalc/mcp-win32-x64", "truecalc-mcp.exe"],
];

for (const [platform, arch, pkgName, binName] of SUPPORTED_COMBOS) {
  test(`resolves ${platform}/${arch} to ${pkgName}`, () => {
    const result = resolveTarget(platform, arch);
    assert.equal(result.pkgName, pkgName);
    assert.equal(result.binName, binName);
  });
}

test("throws a clear, actionable error for an unsupported platform", () => {
  assert.throws(
    () => resolveTarget("freebsd", "x64"),
    (err) => {
      assert.match(err.message, /does not support freebsd\/x64/);
      assert.match(err.message, /Supported platforms:/);
      assert.match(err.message, /darwin-arm64/);
      return true;
    },
  );
});

test("throws a clear error for an unsupported arch on a supported platform", () => {
  assert.throws(() => resolveTarget("linux", "ia32"), /does not support linux\/ia32/);
});

test("throws a clear error for the Rust target-triple spelling ('aarch64'), not Node's ('arm64')", () => {
  // Guards against a naming mix-up: Node never reports 'aarch64', only
  // 'arm64'. If this ever resolved successfully it would mean the map was
  // keyed on the wrong vocabulary.
  assert.throws(() => resolveTarget("linux", "aarch64"), /does not support linux\/aarch64/);
});
