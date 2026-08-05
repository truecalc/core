#!/usr/bin/env node
"use strict";

const path = require("node:path");
const fs = require("node:fs");
const { spawnSync } = require("node:child_process");
const { resolveTarget } = require("../lib/resolve.js");

// Resolves the on-disk path to the platform binary. `pkgName` only ever comes
// from the fixed PLATFORM_PACKAGES map in lib/resolve.js (never from argv or
// the environment), so require.resolve here can't be steered by untrusted
// input — it either finds the real optionalDependency package or it doesn't.
function resolveBinaryPath(platform, arch) {
  const { pkgName, binName } = resolveTarget(platform, arch);

  let pkgJsonPath;
  try {
    pkgJsonPath = require.resolve(`${pkgName}/package.json`);
  } catch {
    throw new Error(
      `@truecalc/mcp could not find its optional dependency '${pkgName}'.\n` +
        "npm likely skipped installing it (e.g. --omit=optional, --no-optional, " +
        "or an npm/yarn/pnpm version that mishandles OS/CPU-scoped optionalDependencies).\n" +
        "Reinstall with optional dependencies included: npm install --include=optional @truecalc/mcp",
    );
  }

  const binaryPath = path.join(path.dirname(pkgJsonPath), "bin", binName);
  if (!fs.existsSync(binaryPath)) {
    throw new Error(`@truecalc/mcp found '${pkgName}' but its binary is missing at ${binaryPath}.`);
  }
  return binaryPath;
}

function main() {
  let binaryPath;
  try {
    binaryPath = resolveBinaryPath(process.platform, process.arch);
  } catch (err) {
    process.stderr.write(`${err.message}\n`);
    process.exitCode = 1;
    return;
  }

  // spawnSync with an argv array and no `shell: true`: process.argv and the
  // resolved binary path are passed as discrete arguments to execve, never
  // interpolated into a shell command line, so nothing in either can inject
  // a second command. stdio is inherited because MCP speaks JSON-RPC over
  // stdio — this process is a transparent relay for stdin/stdout/stderr.
  const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
  if (result.error) {
    process.stderr.write(`@truecalc/mcp failed to start ${binaryPath}: ${result.error.message}\n`);
    process.exitCode = 1;
    return;
  }
  // null status means the child was killed by a signal; propagate a non-zero
  // exit rather than silently reporting success.
  process.exitCode = result.status === null ? 1 : result.status;
}

main();
