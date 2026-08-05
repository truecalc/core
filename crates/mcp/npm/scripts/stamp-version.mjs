// Stamp the release version into the @truecalc/mcp wrapper package.json:
// its own `version` field, and every optionalDependencies entry (the five
// platform packages, which are published at the same version — see
// crates/mcp/Cargo.toml for why every crate in this workspace shares one
// version). Run this only after all five platform packages have already
// been published at that version, so npm's dependency resolution for the
// wrapper never points at a version that doesn't exist yet.
//
// Usage: node stamp-version.mjs <package.json-path> <version>

import fs from "node:fs";

const [, , pkgJsonPath, version] = process.argv;

if (!pkgJsonPath || !version) {
  console.error("usage: node stamp-version.mjs <package.json-path> <version>");
  process.exit(1);
}
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error(`refusing to stamp non-semver version: '${version}'`);
  process.exit(1);
}

const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, "utf8"));
pkg.version = version;
if (pkg.optionalDependencies) {
  for (const dep of Object.keys(pkg.optionalDependencies)) {
    pkg.optionalDependencies[dep] = version;
  }
}
fs.writeFileSync(pkgJsonPath, JSON.stringify(pkg, null, 2) + "\n");

console.log(`${pkg.name}@${pkg.version}: version stamped`);
