// Build one npm platform package (@truecalc/mcp-<platform>-<arch>) for a
// release of the truecalc-mcp binary.
//
// Writes <outDir>/package.json and copies the built binary to
// <outDir>/bin/<binName>, so `npm publish` run from <outDir> ships exactly
// the prebuilt binary for one OS/CPU pair. npm's `os`/`cpu` package.json
// fields (set below) are what makes npm skip installing the four
// non-matching platform packages automatically when @truecalc/mcp's
// optionalDependencies are resolved.
//
// `platform`/`arch` must be Node's own process.platform/process.arch
// vocabulary ('darwin'/'linux'/'win32', 'arm64'/'x64') — NOT the Rust target
// triple's naming ('aarch64', 'x86_64') — because that's what npm compares
// against a consumer's `process.platform`/`process.arch` at install time, and
// it's what crates/mcp/npm/wrapper/lib/resolve.js looks up at run time. The
// caller (release.yml) is responsible for mapping its build matrix's Rust
// target triple to this vocabulary.
//
// Usage: node make-platform-package.mjs <outDir> <platform> <arch> <version> <binaryPath>

import fs from "node:fs";
import path from "node:path";

const [, , outDir, platform, arch, version, binaryPath] = process.argv;

const VALID_PLATFORMS = ["darwin", "linux", "win32"];
const VALID_ARCHES = ["arm64", "x64"];

if (!outDir || !platform || !arch || !version || !binaryPath) {
  console.error("usage: node make-platform-package.mjs <outDir> <platform> <arch> <version> <binaryPath>");
  process.exit(1);
}
if (!VALID_PLATFORMS.includes(platform)) {
  console.error(`unsupported platform '${platform}' (expected one of ${VALID_PLATFORMS.join(", ")})`);
  process.exit(1);
}
if (!VALID_ARCHES.includes(arch)) {
  console.error(`unsupported arch '${arch}' (expected one of ${VALID_ARCHES.join(", ")})`);
  process.exit(1);
}
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error(`refusing to stamp non-semver version: '${version}'`);
  process.exit(1);
}
if (!fs.existsSync(binaryPath)) {
  console.error(`binary not found at '${binaryPath}'`);
  process.exit(1);
}

const binName = platform === "win32" ? "truecalc-mcp.exe" : "truecalc-mcp";
const pkgName = `@truecalc/mcp-${platform}-${arch}`;

fs.mkdirSync(path.join(outDir, "bin"), { recursive: true });
fs.copyFileSync(binaryPath, path.join(outDir, "bin", binName));
fs.chmodSync(path.join(outDir, "bin", binName), 0o755);

const pkg = {
  name: pkgName,
  version,
  description: `Prebuilt truecalc-mcp binary for ${platform}/${arch} — installed automatically as an optionalDependency of @truecalc/mcp, not meant to be depended on directly.`,
  license: "MIT",
  repository: { type: "git", url: "https://github.com/truecalc/core", directory: "crates/mcp" },
  os: [platform],
  cpu: [arch],
  files: [`bin/${binName}`],
  publishConfig: { provenance: true, access: "public" },
};

fs.writeFileSync(path.join(outDir, "package.json"), JSON.stringify(pkg, null, 2) + "\n");

console.log(`${pkgName}@${version}: package written to ${outDir}`);
