// Rewrite the package.json wasm-pack generated for @truecalc/core before publish.
//
// Two jobs: set the published name, and add the conditional export that gives
// Bun a build it can actually run.
//
// Why the Bun condition exists (core#815). The primary artifact is built
// `--target bundler`, whose output relies on WebAssembly ESM integration — the
// wasm is imported as a module and its exports are wired back into the JS glue.
// Deno 2 and Node support that, which is why they need no `init()` call and no
// permission flags. Bun does not: the glue's `__wbg_set_wasm` never receives a
// usable module, so the first string crossing dereferences `undefined` and
// throws `malloc is not a function`. Measured on Bun 1.3.7, and `bun build
// --target=bun` does not rescue it either — so this is not something callers
// can bundle their way out of.
//
// A `--target web` build works in Bun, at the cost of requiring `await init()`.
// So Bun resolves to the `_bun` glue, and every other runtime keeps the
// init-free build it already had. `@truecalc/workbook` needs none of this: npm
// already ships it `--target web`, which is why it works in Bun today.
//
// Both builds emit a byte-identical `truecalc_wasm_bg.wasm`, so the Bun glue
// sits alongside the default one at the package root and SHARES that single
// wasm — its `new URL('truecalc_wasm_bg.wasm', import.meta.url)` resolves to
// the same file. Shipping the Bun build in a subdirectory instead would
// duplicate 2 MB and roughly double the tarball for every Node and Deno user.
// The workflow verifies the two hashes still match and fails if they diverge.
//
// Usage: node scripts/npm-package-json.mjs <pkg-dir>

import fs from "node:fs";
import path from "node:path";

const pkgDir = process.argv[2];
if (!pkgDir) {
  console.error("usage: node scripts/npm-package-json.mjs <pkg-dir>");
  process.exit(1);
}

const BUN_ENTRY = "truecalc_wasm_bun.js";
const BUN_TYPES = "truecalc_wasm_bun.d.ts";

// Check before writing, so a failure cannot leave a half-rewritten manifest.
// npm publishes only what `files` lists, so a missing entry here would ship a
// condition that resolves to nothing at install time.
const required = [BUN_ENTRY, BUN_TYPES, "truecalc_wasm_bg.wasm"];
const missing = required.filter((f) => !fs.existsSync(path.join(pkgDir, f)));
if (missing.length > 0) {
  console.error(`missing build artifacts: ${missing.join(", ")}`);
  process.exit(1);
}

const pkgPath = path.join(pkgDir, "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));

pkg.name = "@truecalc/core";
pkg.license = "MIT";

// Condition order is load-bearing: Node and Deno do not match "bun", so they
// fall through to "default" and resolve exactly what they resolved before.
//
// "./*" preserves deep imports. Adding an `exports` field otherwise *removes*
// every subpath a consumer could previously reach, which would make this a
// breaking change rather than an additive one.
pkg.exports = {
  ".": {
    bun: {
      types: `./${BUN_TYPES}`,
      default: `./${BUN_ENTRY}`,
    },
    types: "./truecalc_wasm.d.ts",
    default: "./truecalc_wasm.js",
  },
  "./*": "./*",
};

// wasm-pack always emits `files`, and npm packs only what it lists. If that
// ever stops being true, fail — silently no-oping here would publish a tarball
// missing the Bun build (and, because wasm-pack writes a pkg/.gitignore
// containing `*`, missing nearly everything else too).
if (!Array.isArray(pkg.files)) {
  console.error("package.json has no `files` array — refusing to publish blind");
  process.exit(1);
}
for (const f of [BUN_ENTRY, BUN_TYPES]) {
  if (!pkg.files.includes(f)) pkg.files.push(f);
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

console.log(`${pkg.name}@${pkg.version}: exports written, Bun build present`);
