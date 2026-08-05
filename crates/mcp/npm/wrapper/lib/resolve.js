"use strict";

// Maps Node's own process.platform/process.arch values to the npm platform
// package that ships the matching truecalc-mcp binary. Keys use Node's
// naming exactly ('arm64', 'x64', 'darwin', 'win32', 'linux') — NOT the Rust
// target triple's naming ('aarch64', 'x86_64') — because process.arch/
// process.platform are what this shim actually receives at runtime.
const PLATFORM_PACKAGES = {
  "darwin-arm64": "@truecalc/mcp-darwin-arm64",
  "darwin-x64": "@truecalc/mcp-darwin-x64",
  "linux-arm64": "@truecalc/mcp-linux-arm64",
  "linux-x64": "@truecalc/mcp-linux-x64",
  "win32-x64": "@truecalc/mcp-win32-x64",
};

const SUPPORTED = Object.keys(PLATFORM_PACKAGES).sort();

// Pure function: given (platform, arch), return which platform package and
// binary filename it maps to, or throw a clear, actionable error. No
// filesystem or require.resolve calls here — that's what makes it testable
// with mocked process.platform/process.arch values.
function resolveTarget(platform, arch) {
  const key = `${platform}-${arch}`;
  const pkgName = PLATFORM_PACKAGES[key];
  if (!pkgName) {
    throw new Error(
      `@truecalc/mcp does not support ${platform}/${arch}.\n` +
        `Supported platforms: ${SUPPORTED.join(", ")}.\n` +
        "Build truecalc-mcp from source instead: https://github.com/truecalc/core/tree/main/crates/mcp",
    );
  }
  const binName = platform === "win32" ? "truecalc-mcp.exe" : "truecalc-mcp";
  return { pkgName, binName };
}

module.exports = { resolveTarget, PLATFORM_PACKAGES };
